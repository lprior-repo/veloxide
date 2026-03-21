# Contract Specification: wtf-worker Integration Tests with Live NATS

## Context
- **Feature**: Integration tests for wtf-worker with live NATS JetStream
- **Bead ID**: wtf-rqby
- **Domain terms**:
  - `WorkQueueConsumer` — durable pull consumer on `wtf-work` stream
  - `AckableTask` — `ActivityTask` + NATS message handle for ack/nak
  - `Worker` — high-level handler dispatch loop
  - `ActivityTask` — serialized msgpack task dispatched by engine
  - Write-ahead (ADR-015) — event appended to JetStream BEFORE side effect
- **Assumptions**:
  - Live NATS server with JetStream enabled is available
  - `wtf-work` stream exists with subjects `wtf.work.<activity_type>`
  - `append_event` (wtf-storage) works correctly
- **Open questions**: None

## Preconditions
- [P1] `WorkQueueConsumer::create`: NATS JetStream context must be valid and connected
- [P2] `WorkQueueConsumer::create`: `worker_name` must be non-empty string
- [P3] `WorkQueueConsumer::create`: `wtf-work` stream must exist in JetStream
- [P4] `next_task`: Consumer must have been successfully created via `create`
- [P5] `Worker::run`: At least one handler must be registered (or warning is logged)
- [P6] `enqueue_activity`: `ActivityTask` must serialize to valid msgpack

## Postconditions
- [Q1] `WorkQueueConsumer::create`: Returns `WorkQueueConsumer` with open message stream
- [Q2] `WorkQueueConsumer::create`: Consumer is durable (survives worker restart)
- [Q3] `next_task`: Returns `Some(AckableTask)` when message available, `None` when stream closed
- [Q4] `next_task`: Task payload deserializes correctly to `ActivityTask`
- [Q5] `AckableTask::ack`: Removes message from NATS work queue
- [Q6] `AckableTask::nak`: Re-delivers message to worker after backoff
- [Q7] `Worker::run`: Registers durable consumer, loops until shutdown signal
- [Q8] `Worker::process_task`: Calls `complete_activity` (success) or `fail_activity` (failure) before ack
- [Q9] `complete_activity` / `fail_activity`: Appends `WorkflowEvent` to JetStream BEFORE returning (ADR-015)
- [Q10] `enqueue_activity`: Publishes to `wtf.work.<activity_type>` subject

## Invariants
- [I1] Work queue message is NEVER acked before `WorkflowEvent` is durably appended to JetStream
- [I2] If `append_event` fails, the work queue message is nak'd (not acked)
- [I3] `ActivityTask.attempt` is 1-based (first attempt = 1)
- [I4] Consumer filter subject matches `wtf.work.*` or `wtf.work.<specific_type>`

## Error Taxonomy
- `WtfError::NatsPublish` — NATS publish/ack failure (serialization, connection, stream not found)
- `WtfError::InvalidInput` — (not used in this module directly)
- Error variants are propagated as `WtfError` from all fallible operations

## Contract Signatures
```rust
// WorkQueueConsumer
pub async fn create(
    js: &Context,
    worker_name: &str,
    filter_subject: Option<String>,
) -> Result<Self, WtfError>

pub async fn next_task(&mut self) -> Result<Option<AckableTask>, WtfError>

// AckableTask
pub async fn ack(self) -> Result<(), WtfError>
pub async fn nak(self) -> Result<(), WtfError>

// Worker
pub fn new(js: Context, worker_name: impl Into<String>, filter_subject: Option<String>) -> Self
pub fn register<F, Fut>(&mut self, activity_type: impl Into<String>, handler: F)
pub async fn run(&self, shutdown_rx: tokio::sync::watch::Receiver<bool>) -> Result<(), WtfError>

// enqueue_activity
pub async fn enqueue_activity(js: &Context, task: &ActivityTask) -> Result<u64, WtfError>
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| worker_name non-empty | Runtime-checked | `&str` (caller guarantees) |
| NATS context valid | Compile-time | `async_nats::jetstream::Context` (bound to connection) |
| Stream exists | Runtime-checked | `Result<_, WtfError>` from `get_stream` |
| Task deserializable | Runtime-checked | `Result<_, WtfError>` from `from_msgpack` |
| attempt >= 1 | Compile-time | `u32` (caller/actor enforces) |

## Violation Examples (REQUIRED)
- VIOLATES P2: `WorkQueueConsumer::create(&js, "", None)` → `Err(WtfError::NatsPublish("invalid consumer name"))`
- VIOLATES P3: `WorkQueueConsumer::create(&js, "worker", None)` with no `wtf-work` stream → `Err(WtfError::NatsPublish("stream not found"))`
- VIOLATES Q5: `ack()` called before event appended → NATS removes message, event never recorded (ADR-015 violation)
- VIOLATES Q9: `complete_activity` returns before `PublishAck` received → crash could lose event (ADR-015 violation)
- VIOLATES I1: Ack before append → event log gap, possible duplicate execution on replay

## Ownership Contracts (Rust-specific)
- `Worker::new`: Takes ownership of `js: Context` — caller retains no reference
- `WorkQueueConsumer::create`: Borrows `js` — does not take ownership
- `AckableTask::ack/nak`: Consumes `self` — message consumed exactly once
- `ActivityTask::to_msgpack`: Returns owned `Bytes` — caller owns serialized payload
- Clone policy: `ActivityTask` is `Clone` — intentional for handler passing

## Non-goals
- Unit tests for pure helpers (already covered in `#[cfg(test)]` modules)
- Timer loop integration (covered in separate bead)
- Activity handler implementation (caller provides handlers)
