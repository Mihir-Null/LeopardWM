# Workspace IPC implementation plan

Spec: [workspace-ipc-proposal.md](workspace-ipc-proposal.md).
Branch: feat/workspace-ipc, based on main bad5b42. Implementation authorized.

## Decisions

- Extend Subscribe with EventKind::WorkspaceState (workspace_state). Keep empty
  filters at legacy_default(); all() includes the new kind. Explicit opt-in is a
  compatibility choice submitted for maintainer review.
- Use the existing broadcast_event path. No watch channel. Generate contiguous,
  byte-bounded snapshot frames before broadcasting under the AppState lock.
- Keep protocol 3 provisional because open PR #110 also uses 3; reconcile at merge.
- Preserve code boundaries and MSVC. No YASB edits, live daemon replacement, push,
  or merge in this implementation task.

## Shared interfaces

In leopardwm_ipc, export WorkspaceStateSnapshot { session_id: String, revision: u64,
focused_monitor_device_name: Option<String>, records: Vec<WorkspaceStateRecord> }.
WorkspaceStateRecord is serde-tagged kind (snake_case):
Monitor { monitor_device_name: String, monitor_id: i64, active_workspace_index: u8 },
Workspace { monitor_device_name: String, workspace_index: u8, name: Option<String> },
Window { monitor_device_name: String, workspace_index: u8, hwnd: u64,
is_floating: bool, is_sticky: bool }.
Derive Debug, Clone, PartialEq, Eq, Serialize, Deserialize on state/records.
IpcEvent variants: WorkspaceSnapshotBegin { protocol_version: u32, session_id: String,
revision: u64, focused_monitor_device_name: Option<String> },
WorkspaceSnapshotChunk { revision: u64, records: Vec<WorkspaceStateRecord> },
WorkspaceSnapshotEnd { revision: u64 }, WorkspaceSnapshotError { message: String }.
All map to WorkspaceState. snapshot.events() -> Result<Vec<IpcEvent>, String>
preflights UTF-8 frames plus newline against MAX_IPC_MESSAGE_SIZE.
Commands: QueryWorkspaceState; SwitchWorkspaceOnMonitor { monitor_device_name:
String, index: u8 }. Response: WorkspaceStateReady { protocol_version: u32 }.

## Tasks

- [ ] 1. Protocol and CLI: extend ipc types, filters, snapshot encoder; add
  `lwm subscribe --events workspace_state`, `lwm query workspaces`, and
  `lwm workspace N --monitor NAME`. Keep CLI thin. Tests first: deserialize new
  commands/filter against current code and observe rejection; then add types.
  Test full record round-trips, old defaults, byte-bounded non-ASCII chunking,
  oversize record errors, CLI dispatch, ack consumption, and frame limits.
- [ ] 2. Targeted switching: implement validated explicit-monitor command in
  command_handler.rs, reusing the existing transition/focus path. Regression tests
  cover invalid target/index (no side effects), unfocused target, already-active
  target focus, empty destination, and unchanged other-monitor active workspace.
- [ ] 3. State publication and transport: add workspace_ipc.rs; store dedup state
  in AppState; project all nine slots per live monitor from actual WM membership.
  Publish after event handling and capture under the same lock as Subscribe.
  Route query through existing subscribe snapshot handoff in one-shot mode.
  Test inactive/floating/tabbed membership, labels/topology/focus, geometry dedup,
  startup ordering, lag, mixed subscriptions, and disconnect/write timeout.
- [ ] 4. Maintained docs and changelog: document actual schemas, compatibility,
  recovery, provisional version, and consumer examples in ipc-events.md/README.md;
  record the feature in CHANGELOG.md. Keep this design file consistent with code.
- [ ] 5. Review and verification: review complete diff, resolve findings; run
  cargo fmt --all -- --check, cargo test --all --locked,
  cargo clippy --all --locked -- -D warnings, cargo build --release --locked,
  and existing CI artifact scripts. Commit focused implementation. Report actual
  validation, including that real YASB/two-monitor acceptance awaits opt-in deployment.

## Progress and review ledger

Initial review: tasks 1/3 share IPC types (signatures fixed above); tasks 1/2 share
command variants; task 2 owns command_handler.rs and task 1 owns CLI/ipc files.
Task 3 owns state.rs/main.rs/ipc_server.rs/events.rs and a new workspace_ipc.rs.
Root integrates out-of-band QueryWorkspaceState handler with task 2's command file.
Ruling: use event-loop publication as the common mutation completion boundary,
with explicit query/subscribe synchronization; audit early-continue paths.
Baseline: prior cargo test --all --locked, fmt check, and Clippy passed on unchanged
Rust source. Each implementation task records its new red/green tests here.
