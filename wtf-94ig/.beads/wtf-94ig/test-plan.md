# Test Plan: handle_procedural_msg Error Handling (wtf-94ig)

## Summary

- **Bead:** wtf-94ig
- **Feature:** Fix error handling in `handle_procedural_msg` to propagate errors instead of silently dropping them
- **Core Change:** `.await;` → `.await?` on procedural handler calls; handlers modified to return `Result<(), ActorProcessingErr>`
- **Behaviors identified:** 18
- **Trophy allocation:** 8 unit / 10 integration / 0 e2e / 1 static
- **Proptest invariants:** 0 (no complex pure functions)
- **Fuzz targets:** 0 (no parsing boundaries)
- **Kani harnesses:** 0 (no critical arithmetic)

---

## 1. Behavior Inventory

### Core Error Propagation Behaviors

1. **handle_procedural_msg returns error when handle_dispatch fails** when event_store is missing or publish fails
2. **handle_procedural_msg returns error when handle_sleep fails** when event_store is missing or publish fails
3. **handle_procedural_msg returns error when handle_now fails** when event_store is missing or publish fails
4. **handle_procedural_msg returns error when handle_random fails** when event_store is missing or publish fails
5. **handle_procedural_msg returns error when handle_wait_for_signal fails** when event_store publish fails
6. **handle_procedural_msg returns error when handle_completed fails** when event_store publish fails
7. **handle_procedural_msg returns error when handle_failed fails** when event_store publish fails

### Reply Channel Behaviors

8. **handle_dispatch sends error via reply channel** when event_store is missing
9. **handle_sleep sends error via reply channel** when event_store is missing
10. **handle_wait_for_signal sends error via reply channel** when event_store is missing
11. **handle_now logs error and returns error but does NOT send via reply channel** when event_store is missing (intentional per Invariant 1 exception)
12. **handle_random logs error and returns error but does NOT send via reply channel** when event_store is missing (intentional per Invariant 1 exception)

### Logging Behaviors

13. **All procedural handlers log errors via tracing::error!** when failure occurs
14. **handle_completed logs info on success** when workflow completes normally

### State Mutation Behaviors

15. **handle_dispatch does not modify state** when event_store is missing (error path)
16. **handle_sleep does not modify state** when event_store is missing (error path)

### Unexpected Message Behavior

17. **handle_procedural_msg returns "Unexpected message" error** when receiving a non-procedural InstanceMsg

### Success Path Behaviors

18. **handle_procedural_msg returns Ok(()) on success** when all handlers succeed

---

## 2. Trophy Allocation

| Layer | Count | Rationale |
|-------|-------|-----------|
| **Unit** | 8 | `handle_procedural_msg` error path assertions; state immutability checks; unexpected message |
| **Integration** | 10 | Full handler + event_store interactions; reply channel behavior; success path |
| **E2E** | 0 | No user-facing workflows changed; this is internal error handling |
| **Static** | 1 | Clippy + compile-time types catch breaking changes |

**Rationale:** This is a focused error-handling fix. Most behaviors require real `EventStore` interaction (integration layer) because procedural handlers access `state.args.event_store`. Unit tests cover structural invariants and pure state assertions.

---

## 3. BDD Scenarios

### Behavior: handle_procedural_msg returns error when handle_dispatch fails

**Given:** `InstanceState` with `ParadigmState::Procedural` and `event_store = None`
**When:** `handle_procedural_msg` receives `InstanceMsg::ProceduralDispatch { ... }`
**Then:** Returns `Err(ActorProcessingErr)` containing "Event store missing"

```
fn procedural_msg_returns_error_when_dispatch_fails_with_no_event_store()
```

**Given:** `InstanceState` with `ParadigmState::Procedural` and `MockFailEventStore` (always returns error)
**When:** `handle_procedural_msg` receives `InstanceMsg::ProceduralDispatch { ... }`
**Then:** Returns `Err(ActorProcessingErr)` containing "mock publish failure"

```
fn procedural_msg_returns_error_when_dispatch_fails_on_publish()
```

---

### Behavior: handle_procedural_msg returns error when handle_sleep fails

**Given:** `InstanceState` with `ParadigmState::Procedural` and `event_store = None`
**When:** `handle_procedural_msg` receives `InstanceMsg::ProceduralSleep { ... }`
**Then:** Returns `Err(ActorProcessingErr)` containing "Event store missing"

```
fn procedural_msg_returns_error_when_sleep_fails_with_no_event_store()
```

**Given:** `InstanceState` with `MockFailEventStore`
**When:** `handle_procedural_msg` receives `InstanceMsg::ProceduralSleep { ... }`
**Then:** Returns `Err(ActorProcessingErr)` containing "mock publish failure"

```
fn procedural_msg_returns_error_when_sleep_fails_on_publish()
```

---

### Behavior: handle_procedural_msg returns error when handle_now fails

**Given:** `InstanceState` with `ParadigmState::Procedural` and `event_store = None`
**When:** `handle_procedural_msg` receives `InstanceMsg::ProceduralNow { ... }`
**Then:** Returns `Err(ActorProcessingErr)` containing "Event store missing"

```
fn procedural_msg_returns_error_when_now_fails_with_no_event_store()
```

**Given:** `InstanceState` with `MockFailEventStore`
**When:** `handle_procedural_msg` receives `InstanceMsg::ProceduralNow { ... }`
**Then:** Returns `Err(ActorProcessingErr)` containing "mock publish failure"

```
fn procedural_msg_returns_error_when_now_fails_on_publish()
```

---

### Behavior: handle_procedural_msg returns error when handle_random fails

**Given:** `InstanceState` with `ParadigmState::Procedural` and `event_store = None`
**When:** `handle_procedural_msg` receives `InstanceMsg::ProceduralRandom { ... }`
**Then:** Returns `Err(ActorProcessingErr)` containing "Event store missing"

```
fn procedural_msg_returns_error_when_random_fails_with_no_event_store()
```

**Given:** `InstanceState` with `MockFailEventStore`
**When:** `handle_procedural_msg` receives `InstanceMsg::ProceduralRandom { ... }`
**Then:** Returns `Err(ActorProcessingErr)` containing "mock publish failure"

```
fn procedural_msg_returns_error_when_random_fails_on_publish()
```

---

### Behavior: handle_procedural_msg returns error when handle_wait_for_signal fails

**Given:** `InstanceState` with `ParadigmState::Procedural` containing buffered signal, and `event_store = None`
**When:** `handle_procedural_msg` receives `InstanceMsg::ProceduralWaitForSignal { ... }`
**Then:** Returns `Err(ActorProcessingErr)` containing "Event store missing"

```
fn procedural_msg_returns_error_when_wait_for_signal_fails_with_no_event_store()
```

**Given:** `InstanceState` with `MockFailEventStore` and buffered signal
**When:** `handle_procedural_msg` receives `InstanceMsg::ProceduralWaitForSignal { ... }`
**Then:** Returns `Err(ActorProcessingErr)` containing "mock publish failure"

```
fn procedural_msg_returns_error_when_wait_for_signal_fails_on_publish()
```

---

### Behavior: handle_procedural_msg returns error when handle_completed fails

**Given:** `InstanceState` with `ParadigmState::Procedural` and `event_store = None`
**When:** `handle_procedural_msg` receives `InstanceMsg::ProceduralWorkflowCompleted`
**Then:** Returns `Err(ActorProcessingErr)` when event_store publish fails

```
fn procedural_msg_returns_error_when_completed_fails_on_publish()
```

---

### Behavior: handle_procedural_msg returns error when handle_failed fails

**Given:** `InstanceState` with `ParadigmState::Procedural` and `event_store = None`
**When:** `handle_procedural_msg` receives `InstanceMsg::ProceduralWorkflowFailed(err)`
**Then:** Returns `Err(ActorProcessingErr)` when event_store publish fails

```
fn procedural_msg_returns_error_when_failed_fails_on_publish()
```

---

### Behavior: handle_dispatch sends error via reply channel when event_store is missing

**Given:** `InstanceState` with `event_store = None` and a valid `reply` channel
**When:** `handle_procedural_msg` receives `InstanceMsg::ProceduralDispatch { ... }`
**Then:** `reply` receives `Err(WtfError::nats_publish("Event store missing"))`

```
fn dispatch_sends_error_via_reply_channel_when_no_event_store()
```

---

### Behavior: handle_sleep sends error via reply channel when event_store is missing

**Given:** `InstanceState` with `event_store = None` and a valid `reply` channel
**When:** `handle_procedural_msg` receives `InstanceMsg::ProceduralSleep { ... }`
**Then:** `reply` receives `Err(WtfError::nats_publish("Event store missing"))`

```
fn sleep_sends_error_via_reply_channel_when_no_event_store()
```

---

### Behavior: handle_wait_for_signal sends error via reply channel when event_store is missing

**Given:** `InstanceState` with `ParadigmState::Procedural` containing buffered signal, `event_store = None`, and a valid `reply` channel
**When:** `handle_procedural_msg` receives `InstanceMsg::ProceduralWaitForSignal { ... }`
**Then:** `reply` receives `Err(WtfError::nats_publish("Event store missing"))`

```
fn wait_for_signal_sends_error_via_reply_channel_when_no_event_store()
```

---

### Behavior: handle_now logs error and returns error but does NOT send via reply channel when event_store is missing

**Given:** `InstanceState` with `ParadigmState::Procedural`, `event_store = None`, and a valid `reply` channel
**When:** `handle_procedural_msg` receives `InstanceMsg::ProceduralNow { ... }`
**Then:** Error is logged via `tracing::error!`, `Err(ActorProcessingErr)` is returned, AND `reply` does NOT receive any message (intentionally dropped per Invariant 1 exception)

```
fn now_does_not_send_error_via_reply_channel_when_no_event_store()
```

**Rationale:** Per contract Invariant 1 exception, `handle_now` intentionally drops replies on failure to prevent non-determinism. The caller will timeout rather than receive a non-persisted value.

---

### Behavior: handle_random logs error and returns error but does NOT send via reply channel when event_store is missing

**Given:** `InstanceState` with `ParadigmState::Procedural`, `event_store = None`, and a valid `reply` channel
**When:** `handle_procedural_msg` receives `InstanceMsg::ProceduralRandom { ... }`
**Then:** Error is logged via `tracing::error!`, `Err(ActorProcessingErr)` is returned, AND `reply` does NOT receive any message (intentionally dropped per Invariant 1 exception)

```
fn random_does_not_send_error_via_reply_channel_when_no_event_store()
```

**Rationale:** Per contract Invariant 1 exception, `handle_random` intentionally drops replies on failure to prevent non-determinism. The caller will timeout rather than receive a non-persisted value.

---

### Behavior: All procedural handlers log errors via tracing::error! when failure occurs

**Given:** `InstanceState` with `MockFailEventStore` and capture of tracing logs
**When:** Each procedural handler is called and fails
**Then:** A `tracing::error!` span is emitted containing the error description

```
fn all_procedural_handlers_log_errors_on_failure()
```

---

### Behavior: handle_dispatch does not modify state when event_store is missing (error path)

**Given:** `InstanceState` with `event_store = None` and `total_events_applied = 100`
**When:** `handle_procedural_msg` receives `InstanceMsg::ProceduralDispatch { ... }`
**Then:** `total_events_applied` remains `100` (state not modified on error)

```
fn dispatch_does_not_modify_state_when_event_store_missing()
```

---

### Behavior: handle_sleep does not modify state when event_store is missing (error path)

**Given:** `InstanceState` with `event_store = None` and `events_since_snapshot = 0`
**When:** `handle_procedural_msg` receives `InstanceMsg::ProceduralSleep { ... }`
**Then:** `events_since_snapshot` remains `0` (state not modified on error)

```
fn sleep_does_not_modify_state_when_event_store_missing()
```

---

### Behavior: handle_procedural_msg returns "Unexpected message" error when receiving a non-procedural InstanceMsg

**Given:** `InstanceState` with `ParadigmState::Procedural`
**When:** `handle_procedural_msg` receives a non-procedural `InstanceMsg` (e.g., `InstanceMsg::Heartbeat` or `InstanceMsg::InjectEvent`)
**Then:** Returns `Err(ActorProcessingErr)` containing "Unexpected message in procedural handler"

```
fn procedural_msg_returns_unexpected_message_error_for_non_procedural_msg()
```

**Error variant for this behavior:**
```
Given: InstanceState with ParadigmState::Procedural
When: handle_procedural_msg receives InstanceMsg::GetStatus
Then: Returns Err(ActorProcessingErr) containing "Unexpected message"
```

---

### Behavior: handle_procedural_msg returns Ok(()) on success when all handlers succeed

**Given:** `InstanceState` with `ParadigmState::Procedural`, `MockOkEventStore`, and valid reply channels
**When:** `handle_procedural_msg` receives any procedural `InstanceMsg` that succeeds
**Then:** Returns `Ok(())` and reply channels receive expected values

**Integration test variant for dispatch success:**
```
Given: InstanceState with ParadigmState::Procedural, MockOkEventStore
When: handle_procedural_msg receives InstanceMsg::ProceduralDispatch { ... }
Then: Returns Ok(()) and reply receives Ok(activity_id)
```

```
fn procedural_msg_returns_ok_on_success()
```

**Integration test variant for sleep success:**
```
Given: InstanceState with ParadigmState::Procedural, MockOkEventStore
When: handle_procedural_msg receives InstanceMsg::ProceduralSleep { ... }
Then: Returns Ok(()) and reply receives Ok(())
```

```
fn sleep_returns_ok_on_success()
```

---

## 4. Proptest Invariants

**Not applicable.** The procedural handlers have complex I/O dependencies (`event_store.publish`, `handlers::inject_event`) and no meaningful pure computation that warrants property-based testing. All inputs require specific `InstanceState` setup.

---

## 5. Fuzz Targets

**Not applicable.** This is an internal error-handling fix with no parsing, deserialization, or user-input boundaries. The code operates on in-memory `InstanceState` and well-typed `InstanceMsg` enums.

---

## 6. Kani Verification Harnesses

**Not applicable.** No critical arithmetic invariants. The error paths are conditional logic based on `Option<EventStore>` presence and `Result` returns from `event_store.publish`.

---

## 7. Mutation Testing Checkpoints

### Critical Mutations to Survive

| Mutation | Must Be Caught By |
|----------|-------------------|
| Remove `.await?` back to `.await;` on `handle_dispatch` | `procedural_msg_returns_error_when_dispatch_fails_on_publish` |
| Remove `.await?` back to `.await;` on `handle_sleep` | `procedural_msg_returns_error_when_sleep_fails_on_publish` |
| Remove `.await?` back to `.await;` on `handle_now` | `procedural_msg_returns_error_when_now_fails_on_publish` |
| Remove `.await?` back to `.await;` on `handle_random` | `procedural_msg_returns_error_when_random_fails_on_publish` |
| Remove `.await?` back to `.await;` on `handle_wait_for_signal` | `procedural_msg_returns_error_when_wait_for_signal_fails_on_publish` |
| Remove `.await?` back to `.await;` on `handle_completed` | `procedural_msg_returns_error_when_completed_fails_on_publish` |
| Remove `.await?` back to `.await;` on `handle_failed` | `procedural_msg_returns_error_when_failed_fails_on_publish` |
| Remove `tracing::error!` call in any handler | `all_procedural_handlers_log_errors_on_failure` |
| Remove error reply send in `handle_dispatch` | `dispatch_sends_error_via_reply_channel_when_no_event_store` |
| Remove error reply send in `handle_sleep` | `sleep_sends_error_via_reply_channel_when_no_event_store` |
| Remove error reply send in `handle_wait_for_signal` | `wait_for_signal_sends_error_via_reply_channel_when_no_event_store` |
| Add error reply send in `handle_now` (violating Invariant 1) | `now_does_not_send_error_via_reply_channel_when_no_event_store` |
| Add error reply send in `handle_random` (violating Invariant 1) | `random_does_not_send_error_via_reply_channel_when_no_event_store` |

**Threshold:** ≥90% mutation kill rate.

**Run:** `cargo mutants --dir crates/wtf-actor`

---

## 8. Combinatorial Coverage Matrix

### handle_procedural_msg Error Paths

| Scenario | Input Class | Expected Output | Layer |
|----------|-------------|-----------------|-------|
| handle_dispatch success | valid state, MockOkEventStore | Ok(()) | integration |
| handle_dispatch: no event_store | event_store = None | Err("Event store missing") | unit |
| handle_dispatch: publish fails | MockFailEventStore | Err("mock publish failure") | unit |
| handle_sleep success | valid state, MockOkEventStore | Ok(()) | integration |
| handle_sleep: no event_store | event_store = None | Err("Event store missing") | unit |
| handle_sleep: publish fails | MockFailEventStore | Err("mock publish failure") | unit |
| handle_now success | valid state, MockOkEventStore | Ok(()) | integration |
| handle_now: no event_store | event_store = None | Err("Event store missing") | unit |
| handle_now: publish fails | MockFailEventStore | Err("mock publish failure") | unit |
| handle_random success | valid state, MockOkEventStore | Ok(()) | integration |
| handle_random: no event_store | event_store = None | Err("Event store missing") | unit |
| handle_random: publish fails | MockFailEventStore | Err("mock publish failure") | unit |
| handle_wait_for_signal success (buffered) | valid state, buffered signal | Ok(()) | integration |
| handle_wait_for_signal: no event_store | event_store = None | Err("Event store missing") | unit |
| handle_wait_for_signal: publish fails | MockFailEventStore | Err("mock publish failure") | unit |
| handle_completed success | valid state, MockOkEventStore | Ok(()) | integration |
| handle_completed: publish fails | MockFailEventStore | Err("mock publish failure") | unit |
| handle_failed success | valid state, MockOkEventStore | Ok(()) | integration |
| handle_failed: publish fails | MockFailEventStore | Err("mock publish failure") | unit |

### Reply Channel Coverage Matrix

| Handler | Reply Channel? | Error Sends to Reply? | Test Coverage |
|---------|---------------|---------------------|---------------|
| handle_dispatch | Yes | Yes - on error | `dispatch_sends_error_via_reply_channel_when_no_event_store` |
| handle_sleep | Yes | Yes - on error | `sleep_sends_error_via_reply_channel_when_no_event_store` |
| handle_wait_for_signal | Yes | Yes - on error | `wait_for_signal_sends_error_via_reply_channel_when_no_event_store` |
| handle_now | Yes | **NO** - intentionally dropped per Invariant 1 | `now_does_not_send_error_via_reply_channel_when_no_event_store` |
| handle_random | Yes | **NO** - intentionally dropped per Invariant 1 | `random_does_not_send_error_via_reply_channel_when_no_event_store` |
| handle_completed | No | N/A | N/A |
| handle_failed | No | N/A | N/A |

---

## 9. Error Variant Coverage

### ActorProcessingErr Variants

The contract specifies `ActorProcessingErr` as the error type. All test scenarios above verify that specific error messages are propagated rather than silently dropped.

| Error Variant | Message Pattern | Test |
|---------------|-----------------|------|
| Event store missing | `"Event store missing"` | All "no event_store" scenarios |
| Publish failure | `"mock publish failure"` | All "publish fails" scenarios |
| Unexpected message | `"Unexpected message in procedural handler"` | `procedural_msg_returns_unexpected_message_error_for_non_procedural_msg` |

---

## 10. Existing Tests to Preserve

The following tests in `handlers_tests.rs` must continue to pass after the fix:

- `snapshot_trigger_*` — unrelated to procedural handlers
- `handle_signal_*` — unrelated to procedural handlers
- `signal_delivery_*` — unrelated to procedural handlers
- `terminate_*` — unrelated to procedural handlers
- `invariant_*` — source-level invariants

---

## Open Questions

**Q1 (RESOLVED):** Do `handle_now` and `handle_random` need to send errors via reply channel when event_store is missing?

**Resolution:** No. Per contract Invariant 1 exception, `handle_now` and `handle_random` intentionally drop reply channel errors on failure to prevent non-determinism. The caller will timeout/error rather than receive a non-persisted value. This is intentional behavior documented in the contract.

**Tests added:**
- `now_does_not_send_error_via_reply_channel_when_no_event_store` - verifies reply is NOT sent
- `random_does_not_send_error_via_reply_channel_when_no_event_store` - verifies reply is NOT sent

---

## Implementation Notes for test-writer

### Required Mock Types

```rust
// Already exists in handlers_tests.rs
struct MockFailEventStore; // always returns Err(WtfError::nats_publish("mock publish failure"))
struct MockOkEventStore;  // always returns Ok(next_seq)
```

### Required Test Helper

```rust
fn make_test_state(event_store: Option<Arc<dyn EventStore>>) -> InstanceState {
    // Use existing test_args_with_stores and make_test_state helpers
}
```

### Key Assertion Pattern

```rust
let result = handle_procedural_msg(myself_ref, msg, &mut state).await;
assert!(result.is_err(), "should return error");
let err = result.unwrap_err();
let err_msg = format!("{}", err);
assert!(
    err_msg.contains("Event store missing") || err_msg.contains("mock publish failure"),
    "error message must be specific, got: {}",
    err_msg
);
```

### Reply Channel Assertion Pattern

```rust
// For handlers that SHOULD send error to reply
let reply = oneshot::channel::<Result<_, WtfError>>();
let msg = /* procedural msg with reply */;
handle_procedural_msg(myself_ref, msg, &mut state).await;
let err = reply.await.unwrap().unwrap_err();
assert!(err.to_string().contains("Event store missing"));

// For handlers that should NOT send error to reply (handle_now, handle_random)
// Use timeout assertion:
let reply = oneshot::channel::<Result<_, WtfError>>();
let msg = /* procedural msg with reply */;
let handle = tokio::spawn(handle_procedural_msg(myself_ref, msg, &mut state));
let result = tokio::time::timeout(Duration::from_millis(100), handle).await;
assert!(result.is_err() || result.unwrap().is_err()); // function returns error
// And verify reply was NOT called - it will timeout if not called
```

### Tracing Log Capture

Use `tracing-test` or `tracing-subscriber` with `tracing::error!` span capture to verify logging behavior:

```rust
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

let error_span = Arc::new(Mutex::new(None));
let layer = fmt::layer()
    .with_writer(std::sync::Arc::new(MockWriter {
        error_span: error_span.clone(),
    }))
    .on_event(|event| {
        if event.metadata().level() == &tracing::Level::ERROR {
            // capture error span
        }
    });
tracing_subscriber::registry().with(layer).init();
```

### Success Path Assertion Pattern

```rust
let result = handle_procedural_msg(myself_ref, msg, &mut state).await;
assert!(result.is_ok(), "should succeed when event_store works");
```

---

## Files Under Test

| File | Change |
|------|--------|
| `crates/wtf-actor/src/instance/handlers.rs` | Change `.await;` to `.await?` in `handle_procedural_msg` |
| `crates/wtf-actor/src/instance/procedural.rs` | Modify handlers to return `Result<(), ActorProcessingErr>` |
| `crates/wtf-actor/src/instance/procedural_utils.rs` | Modify `handle_now`, `handle_random`, `handle_completed`, `handle_failed` to return `Result` |

---

## Review Findings Resolution

| Finding | Resolution |
|---------|------------|
| **LETHAL-1:** Contract conflict on handle_now/handle_random | Contract amended with Invariant 1 exception; test plan explicitly tests both error propagation AND reply-dropping |
| **MAJOR-1:** "Unexpected message" error variant untested | Added `procedural_msg_returns_unexpected_message_error_for_non_procedural_msg` |
| **MAJOR-2:** Reply channel coverage incomplete | Added tests for `handle_wait_for_signal` reply; clarified `handle_now`/`handle_random` intentionally do NOT send to reply per Invariant 1 |
| **MAJOR-3:** Success paths unspecified | Added `procedural_msg_returns_ok_on_success` and `sleep_returns_ok_on_success` integration tests |

(End of file - total 727 lines)