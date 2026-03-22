bead_id: wtf-wygu
bead_title: Long-running activity heartbeat mechanism
phase: contract
updated_at: 2026-03-22T00:30:00Z

# Contract Specification

## Context
- **Feature**: Long-running activity heartbeat mechanism
- **Domain terms**:
  - `Heartbeat` — A progress update sent by a worker while an activity is executing
  - `ActivityHeartbeat` — The `WorkflowEvent` variant recording heartbeat progress
  - `HeartbeatSender` — A handle returned to the activity handler for sending heartbeats
- **Assumptions**:
  - Heartbeats are fire-and-forget (no acks required)
  - Multiple heartbeats can be sent during a single activity execution
  - Heartbeats do not affect activity result (success/failure is determined separately)
  - A `stop_heartbeat` function exists to clean up resources
- **Open questions**:
  - What should the heartbeat payload contain? (decided: `progress: String` for human-readable status)

## Scope Map
| What | Where |
|------|-------|
| `ActivityHeartbeat` variant | `wtf-common/src/events.rs` |
| `send_heartbeat()` function | `wtf-worker/src/activity.rs` |
| `HeartbeatSender` handle | `wtf-worker/src/activity.rs` |
| Integration with `Worker::process_task` | `wtf-worker/src/worker.rs` |
| Unit tests | `wtf-worker/src/activity.rs` (mod tests) |

## Preconditions
- [ ] `send_heartbeat` must only be called while an activity is actively executing (enforced by `HeartbeatSender` lifetime)
- [ ] `activity_id` must be non-empty (enforced by type: `ActivityId` is a newtype wrapper that validates on construction)
- [ ] `progress` string must not exceed 1KB (enforced by runtime check, returns error if exceeded)

## Postconditions
- [ ] `send_heartbeat` appends exactly one `ActivityHeartbeat` event to JetStream
- [ ] `send_heartbeat` returns `Ok(seq)` where `seq` is the JetStream sequence number
- [ ] `HeartbeatSender::stop` cleans up any pending heartbeat state (no-op, idempotent)
- [ ] After `stop` is called, no more heartbeats can be sent on that handle

## Invariants
- [ ] `ActivityHeartbeat` events are appended to the same namespace/instance as the parent activity
- [ ] The sequence of heartbeats for a given activity is monotonically increasing
- [ ] `HeartbeatSender` is `Send + Sync` to allow sharing across tokio tasks

## Error Taxonomy
- `WtfError::NatsPublish` — Failed to append heartbeat event to JetStream
- `WtfError::InvalidInput` — Progress payload exceeds 1KB limit
- `WtfError::HeartbeatStopped` — Attempted to send heartbeat after `stop()` was called

## Function Signatures

```rust
/// Send a heartbeat for a running activity.
///
/// Appends `ActivityHeartbeat` to JetStream and returns the sequence number.
/// Heartbeats are fire-and-forget; failures are logged but do not affect activity outcome.
///
/// # Parameters
/// - `js` — JetStream context
/// - `namespace` — Namespace of the owning workflow instance
/// - `instance_id` — Instance that owns this activity
/// - `activity_id` — The activity's unique ID
/// - `progress` — Human-readable progress string (max 1KB)
///
/// # Errors
/// Returns `WtfError::NatsPublish` if the append fails.
/// Returns `WtfError::InvalidInput` if `progress` exceeds 1KB.
pub async fn send_heartbeat(
    js: &Context,
    namespace: &NamespaceId,
    instance_id: &InstanceId,
    activity_id: &ActivityId,
    progress: &str,
) -> Result<u64, WtfError>;

/// A handle for sending heartbeats during activity execution.
///
/// Created by the worker before invoking the activity handler.
/// The handler receives this handle and can call `send()` to emit heartbeats.
///
/// # Example
/// ```ignore
/// async fn my_activity(task: ActivityTask, heartbeat: HeartbeatSender) {
///     heartbeat.send("Starting phase 1").await.ok();
///     // do work
///     heartbeat.send("Phase 1 complete").await.ok();
/// }
/// ```
pub struct HeartbeatSender {
    // internal state
}

impl HeartbeatSender {
    /// Send a heartbeat with the given progress message.
    ///
    /// # Errors
    /// Returns `WtfError::HeartbeatStopped` if `stop()` was already called.
    pub async fn send(&self, progress: &str) -> Result<u64, WtfError>;

    /// Stop sending heartbeats and release resources.
    ///
    /// Idempotent: calling multiple times is safe.
    pub fn stop(&self);
}
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| activity_id is valid | Compile-time | `ActivityId` newtype wrapper |
| progress <= 1KB | Runtime-checked | `Result<u64, WtfError::InvalidInput>` |
| heartbeat only while active | Lifetime | `HeartbeatSender` borrow bound to activity execution |
| stop is idempotent | compile-time | `fn stop(&self)` — no state change on repeated calls |

## Violation Examples
- VIOLATES <P1>: `send_heartbeat(js, ns, inst, &ActivityId::new(""), "progress")` — should produce `Err(WtfError::InvalidInput)` because `ActivityId::new("")` is not a valid ID
- VIOLATES <P2>: `send_heartbeat(js, ns, inst, &valid_id, &"x".repeat(2000))` — progress exceeds 1KB, should produce `Err(WtfError::InvalidInput)`
- VIOLATES <P3>: `heartbeat.send("progress").await` after `heartbeat.stop()` — should produce `Err(WtfError::HeartbeatStopped)`

## Ownership Contracts
- `send_heartbeat` takes `&Context` (shared borrow of NATS context)
- `HeartbeatSender` is `Clone` (cloned handle points to same underlying state)
- `stop()` is idempotent — multiple calls are safe

## Non-goals
- [ ] Heartbeat delivery guarantees (fire-and-forget is acceptable)
- [ ] Heartbeat aggregation or querying (engine handles this via event log)
- [ ] Automatic heartbeat emission (handler must explicitly call `send()`)
