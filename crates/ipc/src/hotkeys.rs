//! Single source of truth for hotkey actions: default binding, human
//! label, and display grouping for each bindable action.
//!
//! Both the daemon (default config, settings UI) and the CLI
//! (`generate-config`) derive everything from [`hotkey_catalog`], so a new
//! hotkey is defined exactly once instead of in `default_bindings`, the
//! config template, the CLI template, and the settings JS tables.

use std::collections::HashMap;

use serde::Serialize;

/// A bindable action plus its presentation metadata.
#[derive(Debug, Clone, Serialize)]
pub struct HotkeyAction {
    /// Action id used in config files and `parse_command`.
    pub id: String,
    /// Default key chord, or `None` for actions with no default binding.
    #[serde(rename = "key")]
    pub default_key: Option<String>,
    /// Human-readable label for the settings UI.
    pub label: String,
    /// Section grouping (drives settings order and config-template comments).
    #[serde(skip)]
    pub group: &'static str,
}

fn action(id: &str, default_key: Option<&str>, label: &str, group: &'static str) -> HotkeyAction {
    HotkeyAction {
        id: id.to_string(),
        default_key: default_key.map(str::to_string),
        label: label.to_string(),
        group,
    }
}

/// The ordered catalog of every bindable action. Order here is the display
/// order in the settings UI; `group` drives section headers there and in
/// the generated config template.
///
/// Personal desktop grammar:
/// - F13 is the dedicated desktop/WM modifier (Caps Lock is remapped to F13 externally).
/// - Shift carries/manipulates the focused object.
/// - Ctrl widens directional H/J/K/L commands from the current monitor to monitors.
pub fn hotkey_catalog() -> Vec<HotkeyAction> {
    let mut v = vec![
        action("focus_left", Some("F13+H"), "Focus left", "Focus"),
        action("focus_right", Some("F13+L"), "Focus right", "Focus"),
        action("focus_up", Some("F13+K"), "Focus up", "Focus"),
        action("focus_down", Some("F13+J"), "Focus down", "Focus"),
        action(
            "focus_start",
            Some("F13+Home"),
            "Focus start of strip",
            "Focus",
        ),
        action(
            "focus_end",
            Some("F13+End"),
            "Focus end of strip",
            "Focus",
        ),
        action(
            "move_column_left",
            Some("F13+Shift+H"),
            "Move column left",
            "Move column",
        ),
        action(
            "move_column_right",
            Some("F13+Shift+L"),
            "Move column right",
            "Move column",
        ),
        action(
            "move_column_to_start",
            Some("F13+Shift+Home"),
            "Move column to start",
            "Move column",
        ),
        action(
            "move_column_to_end",
            Some("F13+Shift+End"),
            "Move column to end",
            "Move column",
        ),
        action(
            "move_window_left",
            Some("F13+["),
            "Move window left",
            "Move window to adjacent column",
        ),
        action(
            "move_window_right",
            Some("F13+]"),
            "Move window right",
            "Move window to adjacent column",
        ),
        action(
            "expel_to_left",
            Some("F13+Shift+["),
            "Expel to left",
            "Expel window to a new column",
        ),
        action(
            "expel_to_right",
            Some("F13+Shift+]"),
            "Expel to right",
            "Expel window to a new column",
        ),
        action(
            "consume_from_left",
            Some("F13+,"),
            "Consume from left",
            "Stack the neighbor's window into the focused column",
        ),
        action(
            "consume_from_right",
            Some("F13+."),
            "Consume from right",
            "Stack the neighbor's window into the focused column",
        ),
        action(
            "move_window_up",
            Some("F13+Shift+K"),
            "Move window up",
            "Move window within column",
        ),
        action(
            "move_window_down",
            Some("F13+Shift+J"),
            "Move window down",
            "Move window within column",
        ),
        action(
            "cycle_width_down",
            Some("F13+-"),
            "Cycle width down",
            "Column width",
        ),
        action(
            "cycle_width_up",
            Some("F13+="),
            "Cycle width up",
            "Column width",
        ),
        action(
            "equalize_widths",
            Some("F13+0"),
            "Equalize widths",
            "Column width",
        ),
        action(
            "cycle_height_down",
            Some("F13+Shift+-"),
            "Cycle height down",
            "Window height",
        ),
        action(
            "cycle_height_up",
            Some("F13+Shift+="),
            "Cycle height up",
            "Window height",
        ),
        action(
            "equalize_heights",
            Some("F13+Shift+0"),
            "Equalize heights",
            "Window height",
        ),
        action(
            "focus_monitor_left",
            Some("F13+Ctrl+H"),
            "Focus monitor left",
            "Monitor focus",
        ),
        action(
            "focus_monitor_right",
            Some("F13+Ctrl+L"),
            "Focus monitor right",
            "Monitor focus",
        ),
        action(
            "focus_monitor_up",
            Some("F13+Ctrl+K"),
            "Focus monitor up",
            "Monitor focus",
        ),
        action(
            "focus_monitor_down",
            Some("F13+Ctrl+J"),
            "Focus monitor down",
            "Monitor focus",
        ),
        action(
            "move_to_monitor_left",
            Some("F13+Ctrl+Shift+H"),
            "Move to monitor left",
            "Move window to monitor",
        ),
        action(
            "move_to_monitor_right",
            Some("F13+Ctrl+Shift+L"),
            "Move to monitor right",
            "Move window to monitor",
        ),
        action(
            "move_to_monitor_up",
            Some("F13+Ctrl+Shift+K"),
            "Move to monitor up",
            "Move window to monitor",
        ),
        action(
            "move_to_monitor_down",
            Some("F13+Ctrl+Shift+J"),
            "Move to monitor down",
            "Move window to monitor",
        ),
        action(
            "center_column",
            Some("F13+C"),
            "Center column",
            "Column layout",
        ),
        action(
            "maximize_column",
            Some("F13+M"),
            "Maximize column",
            "Column layout",
        ),
        action("close_window", Some("F13+W"), "Close window", "Window"),
        action(
            "toggle_floating",
            Some("F13+F"),
            "Toggle floating",
            "Window",
        ),
        action(
            "toggle_fullscreen",
            Some("F13+Shift+F"),
            "Toggle fullscreen",
            "Window",
        ),
        action(
            "toggle_tabbed",
            Some("F13+T"),
            "Toggle tabbed column",
            "Window",
        ),
        action(
            "scratchpad_toggle",
            Some("F13+S"),
            "Toggle scratchpad",
            "Window",
        ),
        action(
            "scratchpad_stash",
            Some("F13+Shift+S"),
            "Stash to scratchpad",
            "Window",
        ),
        action(
            "toggle_sticky",
            Some("F13+Y"),
            "Toggle sticky",
            "Window",
        ),
        action(
            "toggle_new_window_placement",
            None,
            "Toggle new-window placement (new column / in column)",
            "Window",
        ),
        action(
            "toggle_pause",
            Some("F13+P"),
            "Toggle pause",
            "Session",
        ),
        action("refresh", Some("F13+R"), "Refresh", "Session"),
        action(
            "reload",
            Some("F13+Shift+R"),
            "Reload config",
            "Session",
        ),
        action(
            "panic_revert",
            Some("Win+Ctrl+Escape"),
            "Emergency restore",
            "Session",
        ),
        action(
            "toggle_overview",
            Some("F13+Space"),
            "Toggle overview",
            "Workspaces",
        ),
        action(
            "workspace_prev",
            Some("F13+Left"),
            "Previous workspace",
            "Workspaces",
        ),
        action(
            "workspace_next",
            Some("F13+Right"),
            "Next workspace",
            "Workspaces",
        ),
        action(
            "move_to_workspace_prev",
            Some("F13+Shift+Left"),
            "Move window to previous workspace",
            "Move window to workspace",
        ),
        action(
            "move_to_workspace_next",
            Some("F13+Shift+Right"),
            "Move window to next workspace",
            "Move window to workspace",
        ),
    ];
    for i in 1..=9u8 {
        v.push(action(
            &format!("switch_workspace_{i}"),
            Some(&format!("F13+{i}")),
            &format!("Workspace {i}"),
            "Switch to workspace",
        ));
    }
    for i in 1..=9u8 {
        v.push(action(
            &format!("move_to_workspace_{i}"),
            Some(&format!("F13+Shift+{i}")),
            &format!("Move to Workspace {i}"),
            "Move window to workspace",
        ));
    }
    v
}

/// Build the default key-chord -> action-id map from the catalog (skipping
/// actions without a default binding).
pub fn default_bindings_map() -> HashMap<String, String> {
    hotkey_catalog()
        .into_iter()
        .filter_map(|a| a.default_key.map(|key| (key, a.id)))
        .collect()
}

/// Map a normalized (lowercase, underscored) action id to its `IpcCommand`.
/// Covers every catalog id plus non-catalog command aliases (gestures,
/// renames, deprecated width presets) accepted in config files.
pub fn command_for_action(id: &str) -> Option<crate::IpcCommand> {
    use crate::IpcCommand;

    if let Some(suffix) = id.strip_prefix("switch_workspace_") {
        let index: u8 = suffix.parse().ok()?;
        return (1..=9)
            .contains(&index)
            .then_some(IpcCommand::SwitchWorkspace { index });
    }
    if let Some(suffix) = id.strip_prefix("move_to_workspace_") {
        return match suffix {
            "prev" => Some(IpcCommand::MoveToWorkspacePrev),
            "next" => Some(IpcCommand::MoveToWorkspaceNext),
            _ => {
                let index: u8 = suffix.parse().ok()?;
                (1..=9)
                    .contains(&index)
                    .then_some(IpcCommand::MoveToWorkspace { index })
            }
        };
    }

    Some(match id {
        "focus_left" => IpcCommand::FocusLeft,
        "focus_right" => IpcCommand::FocusRight,
        "focus_up" => IpcCommand::FocusUp,
        "focus_down" => IpcCommand::FocusDown,
        "focus_next" => IpcCommand::FocusNext,
        "focus_prev" => IpcCommand::FocusPrev,
        "focus_start" => IpcCommand::FocusStart,
        "focus_end" => IpcCommand::FocusEnd,
        "move_column_left" => IpcCommand::MoveColumnLeft,
        "move_column_right" => IpcCommand::MoveColumnRight,
        "move_column_to_start" => IpcCommand::MoveColumnToStart,
        "move_column_to_end" => IpcCommand::MoveColumnToEnd,
        "focus_monitor_left" => IpcCommand::FocusMonitorLeft,
        "focus_monitor_right" => IpcCommand::FocusMonitorRight,
        "focus_monitor_up" => IpcCommand::FocusMonitorUp,
        "focus_monitor_down" => IpcCommand::FocusMonitorDown,
        "move_to_monitor_left" => IpcCommand::MoveWindowToMonitorLeft,
        "move_to_monitor_right" => IpcCommand::MoveWindowToMonitorRight,
        "move_to_monitor_up" => IpcCommand::MoveWindowToMonitorUp,
        "move_to_monitor_down" => IpcCommand::MoveWindowToMonitorDown,
        "cycle_width_up" | "resize_grow" => IpcCommand::CycleWidthUp,
        "cycle_width_down" | "resize_shrink" => IpcCommand::CycleWidthDown,
        "cycle_height_up" => IpcCommand::CycleHeightUp,
        "cycle_height_down" => IpcCommand::CycleHeightDown,
        "equalize_heights" => IpcCommand::EqualizeColumnHeights,
        "scroll_left" => IpcCommand::Scroll { delta: -100.0 },
        "scroll_right" => IpcCommand::Scroll { delta: 100.0 },
        "refresh" => IpcCommand::Refresh,
        "reload" => IpcCommand::Reload,
        "panic_revert" => IpcCommand::PanicRevert,
        "toggle_pause" => IpcCommand::TogglePause,
        "close_window" => IpcCommand::CloseWindow,
        "toggle_floating" => IpcCommand::ToggleFloating,
        "toggle_fullscreen" => IpcCommand::ToggleFullscreen,
        "scratchpad_stash" => IpcCommand::ScratchpadStash,
        "scratchpad_toggle" => IpcCommand::ScratchpadToggle,
        "toggle_sticky" => IpcCommand::ToggleSticky,
        "toggle_new_window_placement" => IpcCommand::ToggleNewWindowPlacement,
        "toggle_tabbed" => IpcCommand::ToggleTabbed,
        "width_third" => IpcCommand::SetColumnWidth { fraction: 0.333 },
        "width_half" => IpcCommand::SetColumnWidth { fraction: 0.5 },
        "width_two_thirds" => IpcCommand::SetColumnWidth { fraction: 0.667 },
        "center_column" => IpcCommand::CenterColumn,
        "maximize_column" => IpcCommand::MaximizeColumn,
        "equalize_widths" => IpcCommand::EqualizeColumnWidths,
        "move_window_left" => IpcCommand::MoveWindowLeft,
        "move_window_right" => IpcCommand::MoveWindowRight,
        "expel_to_left" => IpcCommand::ExpelToLeft,
        "expel_to_right" => IpcCommand::ExpelToRight,
        "consume_from_left" => IpcCommand::ConsumeFromLeft,
        "consume_from_right" => IpcCommand::ConsumeFromRight,
        "move_window_up" => IpcCommand::MoveWindowUp,
        "move_window_down" => IpcCommand::MoveWindowDown,
        "workspace_prev" => IpcCommand::WorkspacePrev,
        "workspace_next" => IpcCommand::WorkspaceNext,
        "toggle_overview" => IpcCommand::ToggleOverview,
        _ => return None,
    })
}

/// Render the `[hotkeys]` binding lines for the generated config template,
/// grouped with a `# <group>` comment before each section. Does not emit
/// the `[hotkeys]` header or `scroll_modifier` line.
pub fn render_template_block() -> String {
    let mut out = String::new();
    let mut current_group = "";
    for a in hotkey_catalog() {
        let Some(key) = a.default_key else { continue };
        if a.group != current_group {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&format!("# {}\n", a.group));
            current_group = a.group;
        }
        out.push_str(&format!("\"{}\" = \"{}\"\n", key, a.id));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_has_no_duplicate_ids_or_keys() {
        let catalog = hotkey_catalog();
        let mut ids = std::collections::HashSet::new();
        let mut keys = std::collections::HashSet::new();
        for a in &catalog {
            assert!(ids.insert(a.id.clone()), "duplicate action id: {}", a.id);
            if let Some(ref k) = a.default_key {
                assert!(keys.insert(k.clone()), "duplicate default key: {}", k);
            }
        }
    }

    #[test]
    fn test_default_bindings_map_matches_catalog() {
        let map = default_bindings_map();
        let bound = hotkey_catalog()
            .into_iter()
            .filter(|a| a.default_key.is_some())
            .count();
        assert_eq!(map.len(), bound);
        assert_eq!(map.get("F13+H"), Some(&"focus_left".to_string()));
        assert_eq!(map.get("F13+,"), Some(&"consume_from_left".to_string()));
        assert_eq!(map.get("F13+5"), Some(&"switch_workspace_5".to_string()));
    }

    #[test]
    fn test_default_bindings_match_frozen_expected_set() {
        // Golden set: the complete key-chord -> action map the catalog must
        // produce. Guards against transcription drift (a typo would keep the
        // count at 59 but change a binding). Update deliberately when the
        // intended defaults change.
        let expected: &[(&str, &str)] = &[
            ("F13+H", "focus_left"),
            ("F13+L", "focus_right"),
            ("F13+K", "focus_up"),
            ("F13+J", "focus_down"),
            ("F13+Home", "focus_start"),
            ("F13+End", "focus_end"),
            ("F13+Shift+H", "move_column_left"),
            ("F13+Shift+L", "move_column_right"),
            ("F13+Shift+Home", "move_column_to_start"),
            ("F13+Shift+End", "move_column_to_end"),
            ("F13+[", "move_window_left"),
            ("F13+]", "move_window_right"),
            ("F13+Shift+[", "expel_to_left"),
            ("F13+Shift+]", "expel_to_right"),
            ("F13+,", "consume_from_left"),
            ("F13+.", "consume_from_right"),
            ("F13+Shift+K", "move_window_up"),
            ("F13+Shift+J", "move_window_down"),
            ("F13+-", "cycle_width_down"),
            ("F13+=", "cycle_width_up"),
            ("F13+0", "equalize_widths"),
            ("F13+Shift+-", "cycle_height_down"),
            ("F13+Shift+=", "cycle_height_up"),
            ("F13+Shift+0", "equalize_heights"),
            ("F13+Ctrl+H", "focus_monitor_left"),
            ("F13+Ctrl+L", "focus_monitor_right"),
            ("F13+Ctrl+K", "focus_monitor_up"),
            ("F13+Ctrl+J", "focus_monitor_down"),
            ("F13+Ctrl+Shift+H", "move_to_monitor_left"),
            ("F13+Ctrl+Shift+L", "move_to_monitor_right"),
            ("F13+Ctrl+Shift+K", "move_to_monitor_up"),
            ("F13+Ctrl+Shift+J", "move_to_monitor_down"),
            ("F13+C", "center_column"),
            ("F13+M", "maximize_column"),
            ("F13+W", "close_window"),
            ("F13+F", "toggle_floating"),
            ("F13+Shift+F", "toggle_fullscreen"),
            ("F13+T", "toggle_tabbed"),
            ("F13+S", "scratchpad_toggle"),
            ("F13+Shift+S", "scratchpad_stash"),
            ("F13+Y", "toggle_sticky"),
            ("F13+P", "toggle_pause"),
            ("F13+R", "refresh"),
            ("F13+Shift+R", "reload"),
            ("Win+Ctrl+Escape", "panic_revert"),
            ("F13+Space", "toggle_overview"),
            ("F13+Left", "workspace_prev"),
            ("F13+Right", "workspace_next"),
            ("F13+Shift+Left", "move_to_workspace_prev"),
            ("F13+Shift+Right", "move_to_workspace_next"),
            ("F13+1", "switch_workspace_1"),
            ("F13+2", "switch_workspace_2"),
            ("F13+3", "switch_workspace_3"),
            ("F13+4", "switch_workspace_4"),
            ("F13+5", "switch_workspace_5"),
            ("F13+6", "switch_workspace_6"),
            ("F13+7", "switch_workspace_7"),
            ("F13+8", "switch_workspace_8"),
            ("F13+9", "switch_workspace_9"),
            ("F13+Shift+1", "move_to_workspace_1"),
            ("F13+Shift+2", "move_to_workspace_2"),
            ("F13+Shift+3", "move_to_workspace_3"),
            ("F13+Shift+4", "move_to_workspace_4"),
            ("F13+Shift+5", "move_to_workspace_5"),
            ("F13+Shift+6", "move_to_workspace_6"),
            ("F13+Shift+7", "move_to_workspace_7"),
            ("F13+Shift+8", "move_to_workspace_8"),
            ("F13+Shift+9", "move_to_workspace_9"),
        ];
        let expected_map: HashMap<String, String> = expected
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        assert_eq!(default_bindings_map(), expected_map);
    }

    #[test]
    fn test_every_catalog_action_has_command_mapping() {
        // Drift guard: a new catalog row without a command_for_action
        // mapping fails here instead of silently parsing to None.
        for a in hotkey_catalog() {
            assert!(
                command_for_action(&a.id).is_some(),
                "catalog action '{}' has no command_for_action mapping",
                a.id
            );
        }
    }

    #[test]
    fn test_command_for_action_workspace_suffixes() {
        use crate::IpcCommand;
        assert_eq!(
            command_for_action("switch_workspace_5"),
            Some(IpcCommand::SwitchWorkspace { index: 5 })
        );
        assert_eq!(
            command_for_action("move_to_workspace_9"),
            Some(IpcCommand::MoveToWorkspace { index: 9 })
        );
        // Relative next/prev must not be swallowed by the numeric suffix parse.
        assert_eq!(
            command_for_action("move_to_workspace_next"),
            Some(IpcCommand::MoveToWorkspaceNext)
        );
        assert_eq!(
            command_for_action("move_to_workspace_prev"),
            Some(IpcCommand::MoveToWorkspacePrev)
        );
        assert_eq!(command_for_action("switch_workspace_0"), None);
        assert_eq!(command_for_action("switch_workspace_10"), None);
        assert_eq!(command_for_action("move_to_workspace_x"), None);
    }

    #[test]
    fn test_template_block_groups_and_binds() {
        let block = render_template_block();
        assert!(block.contains("# Focus\n"));
        assert!(block.contains("\"F13+H\" = \"focus_left\"\n"));
        assert!(block.contains("\"F13+,\" = \"consume_from_left\"\n"));
        // Numbered workspaces are present.
        assert!(block.contains("\"F13+1\" = \"switch_workspace_1\""));
    }
}
