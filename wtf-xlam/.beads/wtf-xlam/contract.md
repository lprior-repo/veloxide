# Contract Specification

## Context
- **Feature:** Handle cancellation publish failure with saga pattern
- **Bead ID:** wtf-xlam
- **Problem:** If `store.publish` fails when publishing `InstanceCancelled` event, the actor stops anyway, but the cancellation reason is LOST. On restart, the workflow can be resurrected with no record it was cancelled.
- **Domain terms:**
  - `InstanceCancelled` - event indicating workflow instance was cancelled
  - `event_store.publish` - publishes event to durable store
  - `myself_ref.stop` - signals actor to stop
  - `outbox` - local fallback storage when publish fails
  - `saga/compensation` - pattern where failed operations are undone or compensated

## Assumptions
- `store` is an `Option<EventStore>` available in `InstanceState`
- `event_store.publish` is fallible and returns `Result<(), Error>`
- The actor can be configured with retry limits and outbox capacity
- An outbox pattern can be used as compensation for failed publishes

## Open Questions
- What is the max retry count before falling back to outbox? **Answer: MAX_PUBLISH_RETRIES = 3**
- What is the outbox capacity limit? **Answer: OUTBOX_CAPACITY = 100**
- How is the outbox drained and retried later?
- Should the actor stop immediately on cancellation request even if event persistence is pending? **Answer: NO — actor MUST wait for persistence or outbox**

## Preconditions
- [ ] Cancellation request received with a valid `reason`
- [ ] Actor is in a state that allows cancellation (not already stopped/completed)
- [ ] `InstanceState` contains valid `event_store` reference

## Postconditions
- [ ] **Critical:** `InstanceCancelled` event is persisted BEFORE actor stops
- [ ] If publish fails, compensation action (retry/outbox) is attempted
- [ ] Actor only stops AFTER event is successfully persisted OR safely stored in outbox
- [ ] `InstanceCancelled` event contains the original cancellation reason
- [ ] On next restart, any outboxed events are processed before normal operation

## Invariants
- [ ] A cancelled instance MUST NOT be resurrected without a persisted `InstanceCancelled` event
- [ ] The outbox MUST be drained before normal message processing resumes
- [ ] All persisted events MUST be journaled in order
- [ ] Outbox MUST be persisted to disk for crash recovery (survives actor crash)

## Error Taxonomy

All errors use the `Error` enum prefix (e.g., `Error::PublishFailed`):

| Variant | Trigger Condition |
|---------|-------------------|
| `Error::PublishFailed(underlying)` | `store.publish` fails after all retries exhausted |
| `Error::OutboxFull` | Outbox at capacity limit when compensation attempted |
| `Error::OutboxDrainFailed(underlying)` | `drain_outbox` fails during recovery |
| `Error::CancellationTimeout` | Max retries exhausted AND overall cancellation window exceeded (distinct from PublishFailed) |
| `Error::ActorNotRunning` | Cancellation requested but actor is not in Running state |

**Note:** `CancellationTimeout` is distinct from `PublishFailed` — it implies the cancellation was requested but the operation took too long to complete (exceeded a time budget), not merely that retries were exhausted. Implementations MAY choose to map both to the same underlying error if the distinction is not needed, but tests must verify the contract behavior.

## Contract Signatures

```rust
// Core cancellation flow
async fn handle_cancel(
    state: &InstanceState,
    reason: CancelReason,
    myself_ref: &ActorRef<InstanceCommand>,
    reply: RpcReplyPort<()>,
) -> Result<(), ActorProcessingErr>;

// Compensation/saga step
async fn publish_with_compensation(
    store: &Option<EventStore>,
    namespace: &Namespace,
    instance_id: &InstanceId,
    event: WorkflowEvent,
    outbox: &mut Vec<WorkflowEvent>,
) -> Result<(), Error>;

// Outbox drain on startup
async fn drain_outbox(
    store: &EventStore,
    outbox: &mut Vec<WorkflowEvent>,
) -> Result<(), Error>;
```

## Saga/Compensation Pattern Requirements

### Step 1: Publish with Retry
- Attempt `store.publish` up to `MAX_PUBLISH_RETRIES` times (configurable, default **3**)
- Use **exponential backoff** between retries: initial delay **100ms**, multiplier **2x** (100ms, 200ms, 400ms)
- If all retries fail within the cancellation timeout window, return `Error::CancellationTimeout`
- If retries exhaust but timeout not exceeded, proceed to outbox fallback

### Step 2: Outbox Fallback
- If all retries fail, append event to local outbox (in-memory Vec with disk backup)
- Check outbox capacity; if at limit (`OUTBOX_CAPACITY = 100`), return `Error::OutboxFull`
- Outbox MUST be persisted to disk for crash recovery

### Step 3: Actor Shutdown Gate
- Actor MUST NOT call `myself_ref.stop` until:
  - (a) `publish` succeeds, OR
  - (b) event is safely stored in outbox
- Only after either condition is met, shutdown the actor

### Step 4: Outbox Drain on Recovery
- On actor/instance restart, check for outbox events
- Drain outbox before accepting new messages
- **Fail-fast behavior:** If `store.publish` fails for ANY event during drain, abort and return `Error::OutboxDrainFailed`
- If drain fails, log error and retain in outbox for retry

### Step 5: Cancellation During Shutdown
- If cancellation is requested while actor is already in `Stopping` state:
  - Return `Error::ActorNotRunning` immediately
  - Do NOT attempt to publish or modify outbox
- Actor state transitions are: `Running` → `Stopping` → `Stopped`

## Non-goals
- [ ] Implementing full saga with distributed transactions (this is local compensation only)
- [ ] Handling partial publishes (assumes single-event atomicity)
- [ ] Automatic outbox drain retry loop (manual intervention allowed on persistent failure)

(End file - total 132 lines)