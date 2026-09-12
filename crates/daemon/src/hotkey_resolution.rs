//! Deterministic configuration resolution shared by registration and queries.

use crate::config::{self, HotkeyConfig};
use leopardwm_ipc::{HotkeyIssue, IpcCommand};
use leopardwm_platform_win32::{fn_mod_bit, parse_hotkey_string, Hotkey, HotkeyBind, HotkeyId};
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct ResolvedHotkey {
    pub binding: String,
    pub configured_action: String,
    pub action_id: String,
    pub command: IpcCommand,
    pub hook_binding: HotkeyBind,
    pub executable: bool,
}

#[derive(Debug)]
pub(crate) struct ResolvedHotkeys {
    /// Deduplicated hook inputs, including F-key triggers swallowed as modifiers.
    /// Keeping those inputs preserves the hook's configured modifier mask.
    pub bindings: Vec<ResolvedHotkey>,
    pub issues: Vec<HotkeyIssue>,
}

/// Resolve loaded configuration without installing a hook or mutating the config.
/// The lexicographically first valid (binding, configured action) wins each
/// physical chord. Stable IDs retain their physical meaning across reloads.
pub(crate) fn resolve_hotkeys(config: &HotkeyConfig) -> ResolvedHotkeys {
    let mut configured: Vec<_> = config.bindings.iter().collect();
    configured.sort_by(
        |(left_binding, left_action), (right_binding, right_action)| {
            left_binding
                .cmp(right_binding)
                .then_with(|| left_action.cmp(right_action))
        },
    );

    let mut bindings: Vec<ResolvedHotkey> = Vec::new();
    let mut indexes: HashMap<HotkeyId, usize> = HashMap::new();
    let mut issues = Vec::new();
    for (binding, configured_action) in configured {
        let command = config::parse_command(configured_action);
        let parsed = parse_hotkey_string(binding);
        if command.is_none() {
            issues.push(HotkeyIssue {
                binding: binding.clone(),
                action_id: configured_action.clone(),
                message: "unknown action identifier".to_string(),
            });
        }
        if parsed.is_none() {
            issues.push(HotkeyIssue {
                binding: binding.clone(),
                action_id: configured_action.clone(),
                message: "invalid key chord".to_string(),
            });
        }
        let (Some(command), Some((modifiers, vk))) = (command, parsed) else {
            continue;
        };
        let id = Hotkey::stable_id(modifiers, vk);
        if let Some(&index) = indexes.get(&id) {
            let winner = &bindings[index];
            issues.push(HotkeyIssue {
                binding: binding.clone(),
                action_id: configured_action.clone(),
                message: format!(
                    "duplicate physical chord; ignored in favor of '{}' -> '{}'",
                    winner.binding, winner.configured_action
                ),
            });
            continue;
        }
        indexes.insert(id, bindings.len());
        bindings.push(ResolvedHotkey {
            binding: binding.clone(),
            configured_action: configured_action.clone(),
            action_id: config::canonical_action_id(configured_action),
            command,
            hook_binding: HotkeyBind { modifiers, vk, id },
            executable: true,
        });
    }

    // Use exactly the same inputs as install_keyboard_hook. Do not remove
    // blocked trigger records: they can themselves reserve another F modifier.
    let fn_modifier_mask = bindings.iter().fold(0u16, |mask, entry| {
        mask | entry.hook_binding.modifiers.fn_mods
    });
    for entry in &mut bindings {
        if fn_mod_bit(entry.hook_binding.vk).is_some_and(|bit| bit & fn_modifier_mask != 0) {
            entry.executable = false;
            issues.push(HotkeyIssue {
                binding: entry.binding.clone(),
                action_id: entry.configured_action.clone(),
                message: "trigger F-key is also configured as a modifier".to_string(),
            });
        }
    }
    issues.sort_by(|left, right| {
        left.binding
            .cmp(&right.binding)
            .then_with(|| left.action_id.cmp(&right.action_id))
            .then_with(|| left.message.cmp(&right.message))
    });
    ResolvedHotkeys { bindings, issues }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with(entries: &[(&str, &str)]) -> HotkeyConfig {
        HotkeyConfig {
            bindings: entries
                .iter()
                .map(|(binding, action)| ((*binding).into(), (*action).into()))
                .collect(),
            ..HotkeyConfig::default()
        }
    }

    #[test]
    fn equivalent_spellings_have_one_deterministic_runtime_command() {
        // Removing physical-ID deduplication, or using hash iteration to choose
        // the winner, would return extra bindings or the wrong command here.
        for (winner, loser, expected_id) in [
            ("Alt+Control+h", "Ctrl+Alt+H", 0x348),
            ("Meta+Left", "Win+Left", 0x825),
            ("CTRL+H", "ctrl+h", 0x148),
            ("Ctrl + H", "Ctrl+H", 0x148),
        ] {
            for reverse in [false, true] {
                let mut entries = vec![(loser, "focus_right"), (winner, "Focus-Left")];
                if reverse {
                    entries.reverse();
                }
                let resolved = resolve_hotkeys(&config_with(&entries));
                assert_eq!(resolved.bindings.len(), 1);
                let entry = &resolved.bindings[0];
                assert_eq!(entry.binding, winner);
                assert_eq!(entry.command, IpcCommand::FocusLeft);
                assert_eq!(entry.hook_binding.id, expected_id);
                assert_eq!(entry.action_id, "focus_left");
                assert!(entry.executable);
                assert_eq!(resolved.issues.len(), 1);
                assert_eq!(resolved.issues[0].binding, loser);
                assert_eq!(resolved.issues[0].action_id, "focus_right");
            }
        }
    }

    #[test]
    fn invalid_action_does_not_claim_a_physical_chord() {
        let resolved = resolve_hotkeys(&config_with(&[
            ("Alt+Ctrl+H", "missing_action"),
            ("Ctrl+Alt+H", "focus_left"),
            ("Ctrl+Nope", "another_missing_action"),
        ]));
        assert_eq!(resolved.bindings.len(), 1);
        assert_eq!(resolved.bindings[0].binding, "Ctrl+Alt+H");
        assert_eq!(resolved.bindings[0].command, IpcCommand::FocusLeft);
        assert_eq!(resolved.issues.len(), 3);
        assert_eq!(resolved.issues[0].binding, "Alt+Ctrl+H");
        assert_eq!(resolved.issues[1].message, "invalid key chord");
        assert_eq!(resolved.issues[2].message, "unknown action identifier");
    }

    #[test]
    fn blocked_triggers_still_contribute_to_the_hook_modifier_mask() {
        // F14+H reserves F14; F13+F14 cannot fire but must still reserve F13.
        // Dropping it before building hook inputs would make Ctrl+F13 fire.
        let resolved = resolve_hotkeys(&config_with(&[
            ("F14+H", "focus_left"),
            ("F13+F14", "focus_right"),
            ("Ctrl+F13", "Resize-Grow"),
        ]));
        assert_eq!(resolved.bindings.len(), 3);
        let executable: Vec<_> = resolved
            .bindings
            .iter()
            .filter(|entry| entry.executable)
            .map(|entry| entry.binding.as_str())
            .collect();
        assert_eq!(executable, vec!["F14+H"]);
        let blocked = &resolved.bindings[0];
        assert_eq!(blocked.binding, "Ctrl+F13");
        assert_eq!(blocked.action_id, "cycle_width_up");
        assert_eq!(blocked.command, IpcCommand::CycleWidthUp);
        assert_eq!(resolved.issues.len(), 2);
        assert_eq!(resolved.issues[0].action_id, "Resize-Grow");
        assert_eq!(resolved.issues[1].binding, "F13+F14");
    }

    #[test]
    fn invalid_modifier_binding_does_not_block_an_f_key_trigger() {
        let resolved = resolve_hotkeys(&config_with(&[
            ("F13+H", "missing_action"),
            ("Ctrl+F13", "resize_grow"),
        ]));
        assert_eq!(resolved.bindings.len(), 1);
        assert!(resolved.bindings[0].executable);
        assert_eq!(resolved.bindings[0].command, IpcCommand::CycleWidthUp);
        assert_eq!(resolved.issues.len(), 1);
        assert_eq!(resolved.issues[0].binding, "F13+H");
    }
}
