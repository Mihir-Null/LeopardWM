# Workspace IPC for native status-bar integration

Status: proposed contract; this commit changes documentation only.
Baseline: `main` at `bad5b42` (LeopardWM 0.2.8, IPC protocol 2).
Target consumer: a native YASB workspace widget, with no YASB dependency in LeopardWM.

## Outcome

A bar obtains every monitor's workspace membership in one coherent subscription,
including inactive and empty workspaces, and switches the workspace on the monitor
the user actually clicked. YASB controls filtering, icons, labels, and styling.
The persisted workspace-state.json file is no longer the integration API.

## Existing implementation and gaps

- `crates/ipc/src/lib.rs`: Subscribe, WorkspaceChanged, LayoutChanged, and
  SwitchWorkspace already exist. Preserve these messages and their semantics.
- `crates/daemon/src/main.rs::handle_ipc_subscribe`: receiver creation and startup
  snapshot are atomic under AppState's mutex. Reuse this ordering guarantee.
  Existing startup layout data covers only the focused workspace.
- LayoutChanged contains tiled columns, not floating membership. It is unsuitable
  as the authoritative state of all workspaces.
- SwitchWorkspace takes a 1-based index and implicitly uses focused_monitor.
- Workspaces are lazily allocated; persistence omits inactive empty workspaces.
  A bar must still be able to show all nine configured workspace slots.
- `ipc_server.rs` caps each serialized JSON line at 64 KiB. A full-state protocol
  must account for that bound rather than silently dropping a large snapshot.
- The existing IpcEvent enum has no unknown-event fallback. Adding a new event to
  the old Subscribe empty-filter default would break older typed clients.

## Chosen approach and alternatives

Add an explicit workspace-state stream with full replacement snapshots, plus a
monitor-targeted switch command. Retain the named-pipe transport and JSON-line
framing. Protocol 3 advertises this capability; protocol 1/2 commands remain usable.

A file watcher would reduce polling but retain persistence-schema coupling and
poor liveness semantics. Extending the old layout stream would require clients to
reconstruct missing background state and risks old enum deserializers. Fine-grained
membership deltas would save bandwidth but require more recovery and ordering
logic. Start with complete, deduplicated snapshots; optimize only with measurements.

## Proposed commands

| Wire command | CLI | Result |
| --- | --- | --- |
| `{"type":"subscribe_workspace_state"}` | `lwm subscribe-workspaces` | Ack, initial snapshot, replacement snapshots, heartbeats |
| `{"type":"query_workspace_state"}` | `lwm query workspaces` | Ack, one snapshot, then EOF |
| `{"type":"switch_workspace_on_monitor","monitor_device_name":"\\\\.\\DISPLAY2","index":2}` | `lwm workspace 2 --monitor '\\.\DISPLAY2'` | Existing ok/error response |

The query deliberately uses the same framed snapshot format as the subscription;
it is not one unbounded JSON object. The CLI forwards the ack and frames as NDJSON
for both new read commands. Existing `lwm subscribe` output stays unchanged.
Commands and queries use separate connections from subscriptions.

## State model

The snapshot is a compact semantic view built from in-memory AppState.

| Record | Fields and meaning |
| --- | --- |
| Monitor | `monitor_device_name: string`, `monitor_id: i64` (current HMONITOR), `active_workspace_index: u8` |
| Workspace | `monitor_device_name: string`, `workspace_index: u8`, `name: string or null` |
| Window membership | `monitor_device_name: string`, `workspace_index: u8`, `hwnd: u64`, `is_floating: bool`, `is_sticky: bool` |

- Snapshot begin also contains `focused_monitor_device_name: string or null`.
- Emit all connected monitors and all nine slots (indexes 0 through 8) for each,
  even if an empty workspace has not been instantiated in AppState.
- Serialize monitors by device name, workspaces by index, and memberships by HWND
  for deterministic output and exact equality comparison.
- Exclude drag placeholders and internal overlay windows. Retain managed minimized
  and inactive tab windows; visibility is not the same as membership.
- Sticky windows are reported in their current owning workspace with is_sticky.
  Clients may project them across that monitor's workspaces as a display policy.
  Hidden scratchpad windows are omitted until assigned to a workspace again.
- Membership is authoritative WM state, not the set of windows currently visible
  to desktop enumeration. Icon lookup failure must not make a workspace empty.
- Device names identify monitors within the current topology, not permanently
  across hardware changes. HWND and HMONITOR values are transient and must never
  be persisted as stable identities by clients.
- No application titles, executable lookups, images, glyphs, or filesystem paths
  are required in this API. YASB resolves HWNDs through its existing Windows icon
  and process utilities, caches results, and supplies generic fallbacks. Keep
  potentially blocking Win32 icon/process calls out of AppState's lock.

Active and focused are distinct: each monitor has one active workspace; the
snapshot separately identifies the globally focused monitor. Population follows
membership, independently of both. These are the inputs for show-inactive,
show-empty, show-icons, include-floating, and icon-deduplication options in YASB.

## Snapshot framing and consistency

Illustrative sequence for a new workspace stream (arrays shortened for clarity):

```json
{"status":"workspace_state_ready","protocol_version":3,"session_id":"opaque-daemon-instance"}
{"type":"workspace_snapshot_begin","revision":12,"focused_monitor_device_name":"\\\\.\\DISPLAY2"}
{"type":"workspace_snapshot_chunk","revision":12,"records":[{"kind":"monitor","monitor_device_name":"\\\\.\\DISPLAY2","monitor_id":65537,"active_workspace_index":1}]}
{"type":"workspace_snapshot_chunk","revision":12,"records":[{"kind":"workspace","monitor_device_name":"\\\\.\\DISPLAY2","workspace_index":1,"name":"Code"},{"kind":"window","monitor_device_name":"\\\\.\\DISPLAY2","workspace_index":1,"hwnd":123456,"is_floating":false,"is_sticky":false}]}
{"type":"workspace_snapshot_end","revision":12}
```

The real sequence includes all nine workspace records for every connected monitor.

1. Capture an immutable snapshot and attach its update receiver atomically under
   the state mutex. Start at revision 0; advance monotonically when semantic state
   changes. The session ID changes on daemon restart, not on client reconnect.
2. Serialize and write outside the lock. Pack records by actual UTF-8 serialized
   byte length, including the trailing newline; every frame is at most 64 KiB.
   Preflight all record sizes. If any one record cannot fit, emit a bounded stream
   error and close rather than silently truncate or enter a reconnect loop.
3. The client stages begin/chunk records and replaces its displayed model only at
   the matching end. EOF/error before end discards the staged snapshot. Never mix
   chunks from different revisions or daemon sessions.
4. Reuse a shared latest-state channel (Tokio watch with immutable Arc snapshots).
   Slow consumers finish their current captured snapshot and then receive the
   newest revision. Skipped intermediate revisions are valid because snapshots
   replace all state. No unbounded per-client queue or historical replay.
5. Heartbeat every 30 seconds while no snapshot is being written. Heartbeats may
   occur between snapshots, never inside a snapshot transaction. EOF/write failure
   ends the subscription. Clients reconnect with backoff and accept a fresh initial
   snapshot. A write timeout closes a stalled connection and releases its resources.
6. Read-only queries and new subscriptions do not themselves increment revisions.
   The one-shot query terminates after its complete snapshot, with no heartbeat.

Use a separate WorkspaceStateFrame enum and writer. The old IpcEvent broadcast,
Lagged behavior, and Subscribe default filter remain unchanged. Returning a
workspace_state_ready ack is an explicit parser transition for the new commands.
Older daemons reject these unknown commands; the client reports an unsupported
capability instead of falling back silently to inaccurate membership data.

## Change publication

Add an AppState helper that constructs the compact workspace view and compares it
with the previous view. Invoke publication after each completed daemon event,
including window lifecycle, hotkey/IPC commands, configuration reload, display
reconfiguration, and focus changes. Initialize the publisher before serving clients.
Audit early-continue paths so none bypass a relevant state publication.

Compare semantic values exactly, not only a hash. Window rectangles, animation
progress, scroll positions, and icon changes are intentionally absent, so they do
not trigger workspace snapshots. Name changes and background window removal do.
Do not reuse persisted_signature: it includes geometry and omits relevant labels.

The initial snapshot operation must publish/capture any current pending view under
the same lock before attaching the receiver. This prevents a query/subscription
from seeing older cached state between a mutation and the event-loop publication.

## Monitor-targeted switching

The new command validates the device name and index before any state mutation.
Unknown/disconnected monitor or index outside 1..9 returns error without changing
focus, dismissing an overview, or cancelling a drag.

On success it activates the requested monitor and workspace and restores that
workspace's eligible focused window. Clicking an already active workspace on an
unfocused monitor still focuses that monitor. Empty destinations select that
monitor/workspace without inventing a window to focus. Other monitors retain their
active workspace indexes.

Refactor the existing switch implementation to accept an explicit monitor target;
preserve the old command as a wrapper supplying focused_monitor. Audit its use of
focused_workspace, floating_focus, sticky rehoming, overview dismissal, drag
cleanup, and pending transition guards. Do not implement this as two independently
queued focus-monitor and switch-workspace commands: intervening focus changes
would recreate the race the explicit target is intended to remove.

Snapshot and membership indexes stay zero-based; switch-command/CLI indexes stay
one-based to preserve the established API. Convert once at the command boundary.

## Implementation units and inspectable diffs

1. **Contract and serialization** — add record/frame structs and the three command
   variants in `crates/ipc/src/lib.rs` (use a focused workspace_state submodule if
   appropriate). Add protocol 3 tests without changing old subscription defaults.
2. **Snapshot builder and publication** — add
   `crates/daemon/src/workspace_ipc.rs`; wire state, startup/subscription handling,
   and post-event publication through `state.rs`, `events.rs`, and `main.rs`.
   Keep snapshot projection testable with synthetic monitor/workspace data.
3. **Transport** — route the two new read commands in `ipc_server.rs`; share a
   byte-bounded encoder and writer, atomic initial capture, latest-state delivery,
   timeout, heartbeat, and disconnect cleanup.
4. **Switch command and CLI** — update `command_handler.rs`, CLI args/dispatch,
   response handling, and daemon_cmds streaming support. Reuse transition behavior.
5. **Documentation and consumer contract** — update `agent_docs/ipc-events.md`
   with complete sample output, indexes, monitor semantics, and recovery rules.
   YASB implementation remains a separate change in its own repository.

Each implementation commit should include its relevant tests. No generated files,
new third-party dependencies, runtime deployment, or YASB edits are required here.

## Required verification before implementation is called complete

| Test | Expected evidence |
| --- | --- |
| Legacy wire fixtures | Old query/switch/subscribe messages round-trip unchanged; old all-events subscription never sees new frames |
| Full initial state | Two monitors, nine slots each, inactive tiled and floating windows included without first switching to them |
| Projection policies | Minimized/tabbed membership retained; placeholders excluded; sticky and scratchpad cases match documented ownership |
| Membership updates | Open/close/move/float/sticky transitions update correct source and destination, including inactive workspaces |
| Names/topology/focus | Same-length name changes, monitor add/remove, and focus onto an empty monitor update snapshots |
| Deduplication | Geometry/animation-only changes do not advance revision or publish another snapshot |
| Atomic startup | A change racing subscription is present in initial state or subsequent replacement, never lost |
| Slow reader/reconnect | Bounded retained state; newest complete snapshot after skipped revisions; fresh session after restart |
| Framing | More than 64 KiB total succeeds across bounded frames; non-ASCII byte sizes counted correctly; partial transaction never commits |
| Explicit targeting | Switching monitor B while A is focused changes only B's active index; active-on-B click focuses B; invalid target has no side effects |
| Build gates | `cargo fmt --all -- --check`, `cargo test --all --locked`, `cargo build --release --locked` |
| Desktop acceptance | Separate opt-in deployment verifies two-monitor switching, floating focus restoration, restart, and monitor disconnect |

## Research basis

YASB already separates widget state from presentation and uses Windows HWND icon
resolution in its Komorebi and GlazeWM workspace widgets. This proposal supplies
that same category of data without importing their WM-specific layout semantics.

- https://github.com/amnweb/yasb/blob/main/src/core/widgets/komorebi/workspaces.py
- https://github.com/amnweb/yasb/blob/main/src/core/widgets/glazewm/workspaces.py
- https://github.com/amnweb/yasb/blob/main/src/core/utils/win32/app_icons.py
- Existing LeopardWM contract: [ipc-events.md](ipc-events.md).

The references describe upstream patterns. The baseline analysis above was checked
against this feature worktree, and the local YASB checkout was inspected separately.
