bead_id: wtf-wygu
bead_title: Long-running activity heartbeat mechanism
phase: implementation
updated_at: 2026-03-22T01:00:00Z

# Implementation Summary

## Changes Made

### 1. `wtf-common/src/types.rs`
Added two new error variants to `WtfError`:
- `WtfError::InvalidInput` — for oversized heartbeat progress strings
- `WtfError::HeartbeatStopped` — for heartbeat sender used after `stop()` called

### 2. `wtf-common/src/events.rs`
Added `ActivityHeartbeat` variant to `WorkflowEvent` enum:
```rust
ActivityHeartbeat {
    activity_id: String,
    progress: String,  // max 1KB
}
```

### 3. `wtf-worker/src/activity.rs`
Added:
- `MAX_HEARTBEAT_PROGRESS_BYTES` constant (1KB limit)
- `send_heartbeat()` async function — appends `ActivityHeartbeat` to JetStream
- `HeartbeatSender` struct — handle for sending heartbeats during activity execution
  - `new()` — creates a sender for the given activity
  - `send()` — sends a heartbeat with progress string
  - `stop()` — idempotent cleanup

### 4. `wtf-worker/src/lib.rs`
Exported `send_heartbeat` and `HeartbeatSender`

## Verification

- **Build**: `cargo build -p wtf-worker` — PASSED
- **Unit tests**: 34 passed (including new heartbeat test)
- **Clippy**: Warnings are pre-existing (doc_markdown, cast_precision_loss in existing code)

## Files Modified
- `crates/wtf-common/src/types.rs`
- `crates/wtf-common/src/events.rs`
- `crates/wtf-worker/src/activity.rs`
- `crates/wtf-worker/src/lib.rs`

## Integration Notes
The `HeartbeatSender` is designed to be passed to activity handlers. The worker creates it before invoking the handler and the handler can call `send()` to emit heartbeats during long-running operations.
