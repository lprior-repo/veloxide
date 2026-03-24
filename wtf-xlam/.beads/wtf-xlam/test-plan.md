# Test Plan: wtf-xlam — Cancellation Saga / Compensation Pattern

## Summary

- **Bead:** wtf-xlam
- **Feature:** Handle cancellation publish failure with saga pattern
- **Behaviors identified:** 16
- **Trophy allocation:** 5 unit / 9 integration / 1 e2e / 1 static
- **Proptest invariants:** 3
- **Fuzz targets:** 2
- **Kani harnesses:** 2
- **Mutation checkpoint kill rate target:** ≥90%

---

## 1. Behavior Inventory

### Core Cancellation Flow

1. **[Subject]** `handle_cancel` **[action]** persists `InstanceCancelled` event **[outcome]** before stopping actor **[condition]** when cancellation request received with valid reason
2. **[Subject]** `handle_cancel` **[action]** returns `Ok(())` **[outcome]** only after event is persisted or outboxed **[condition]** when actor shutdown gate passes
3. **[Subject]** `handle_cancel` **[action]** signals actor to stop via `myself_ref.stop` **[outcome]** only after successful publish or outbox store **[condition]** when cancellation completes
4. **[Subject]** `handle_cancel` **[action]** returns `Err(Error::ActorNotRunning)` **[outcome]** immediately **[condition]** when actor is already in Stopping/Stopped state

### Saga / Compensation Steps

5. **[Subject]** `publish_with_compensation` **[action]** attempts publish **[outcome]** up to MAX_PUBLISH_RETRIES (3) with exponential backoff (100ms, 200ms, 400ms) **[condition]** when store is available
6. **[Subject]** `publish_with_compensation` **[action]** returns `Err(Error::PublishFailed)` **[outcome]** after all retries exhausted **[condition]** when store.publish fails consistently
7. **[Subject]** `publish_with_compensation` **[action]** returns `Err(Error::CancellationTimeout)` **[outcome]** when max retries exhausted AND cancellation timeout exceeded **[condition]** when store.publish fails and timeout budget depleted
8. **[Subject]** `publish_with_compensation` **[action]** appends event to outbox **[outcome]** when all retries fail and outbox has capacity **[condition]** when outbox has capacity (< 100)
9. **[Subject]** `publish_with_compensation` **[action]** returns `Err(Error::OutboxFull)` **[outcome]** when outbox at capacity limit (100) **[condition]** when retries exhausted and outbox full
10. **[Subject]** `publish_with_compensation` **[action]** returns `Ok(())` **[outcome]** immediately on first successful publish **[condition]** when store.publish succeeds

### Actor Shutdown Gate

11. **[Subject]** actor **[action]** does NOT stop **[outcome]** when event persistence is still pending **[condition]** when cancellation requested but publish in progress
12. **[Subject]** actor **[action]** stops **[outcome]** after event safely in outbox **[condition]** when outbox fallback activated
13. **[Subject]** actor **[action]** stops **[outcome]** after successful publish **[condition]** when publish succeeds

### Outbox Drain on Recovery

14. **[Subject]** `drain_outbox` **[action]** publishes all outboxed events **[outcome]** in order **[condition]** when actor restarts with pending outbox
15. **[Subject]** `drain_outbox` **[action]** returns `Err(Error::OutboxDrainFailed)` and retains events **[outcome]** when store fails on any event **[condition]** when store unavailable during drain (fail-fast)
16. **[Subject]** `drain_outbox` **[action]** empties outbox **[outcome]** after successful drain **[condition]** when all events published

---

## 2. Trophy Allocation

| Behavior | Layer | Rationale |
|----------|-------|-----------|
| handle_cancel persistence-before-stop gate | Integration | Tests real actor lifecycle, requires actor runtime |
| handle_cancel returns only after gate passes | Integration | Real async runtime, actor ref interactions |
| handle_cancel returns ActorNotRunning when already stopping | Integration | Requires actor state machine verification |
| publish_with_compensation retry logic | Unit | Pure function, no I/O, exhaustive combinatorics |
| publish_with_compensation outbox fallback | Unit | Pure logic, deterministic |
| publish_with_compensation error variants (all 4) | Unit | Every error path tested in isolation |
| Outbox capacity enforcement | Unit | Boundary value testing |
| CancellationTimeout distinct from PublishFailed | Unit | Separate error paths must be verified |
| Actor does NOT stop during pending publish | Integration | Requires actor runtime to verify stop is NOT called |
| Actor stops after outbox store | Integration | Real actor lifecycle verification |
| Actor stops after successful publish | Integration | Real actor lifecycle verification |
| drain_outbox processes events in order | Integration | Real EventStore interaction |
| drain_outbox fail-fast on any failure | Integration | Real store failure injection |
| drain_outbox empties after success | Integration | Real state transition |
| Outbox survives actor crash (disk persistence) | Integration | Verifies outbox persistence requirement |
| Fuzz: event serialization round-trip | Fuzz | Untrusted bytes → validated data |
| Fuzz: cancellation reason parsing | Fuzz | User input boundary |
| Kani: publish retry bounded model | Kani | Critical invariant: no infinite loops |
| Kani: outbox capacity never exceeded | Kani | Critical invariant: memory safety |
| Static: clippy + cargo-deny | Static | Free, compile-time |

**Ratios:** 5 unit (31%) / 9 integration (56%) / 1 e2e (6%) / 1 static (6%)
- Slightly heavier on integration due to actor lifecycle verification requirements
- E2E is minimal: saga pattern is an internal concern, not user-facing

---

## 3. BDD Scenarios

### Behavior: Cancellation persists event before actor stops

```
### Scenario: handle_cancel_persists_event_before_stop_when_publish_succeeds_first_try
Given: Actor in Running state with valid EventStore and empty outbox
When: handle_cancel is called with CancelReason::UserRequested
Then: InstanceCancelled event is published to store
And: myself_ref.stop is called
And: store has exactly one event of type InstanceCancelled
And: event.reason equals CancelReason::UserRequested
And: outbox remains empty

### Scenario: handle_cancel_persists_event_before_stop_when_publish_succeeds_after_retries
Given: Actor in Running state with EventStore that fails first N-1 attempts then succeeds
When: handle_cancel is called with CancelReason::Timeout
Then: InstanceCancelled event is published to store after N retries
And: myself_ref.stop is called
And: store has exactly one event of type InstanceCancelled
And: event.reason equals CancelReason::Timeout

### Scenario: handle_cancel_uses_outbox_when_publish_always_fails
Given: Actor in Running state with EventStore that always fails and outbox with capacity
When: handle_cancel is called with CancelReason::AdminCancelled
Then: InstanceCancelled event is appended to outbox
And: myself_ref.stop is called after outbox store succeeds
And: store has zero events
And: outbox contains exactly one InstanceCancelled event
And: event.reason equals CancelReason::AdminCancelled

### Scenario: handle_cancel_returns_error_when_outbox_full
Given: Actor in Running state with EventStore that always fails and outbox at capacity (100)
When: handle_cancel is called with CancelReason::UserRequested
Then: Err(Error::OutboxFull) is returned
And: myself_ref.stop is NOT called
And: outbox remains at capacity (unchanged)
```

### Behavior: Actor refuses cancellation when already stopping

```
### Scenario: handle_cancel_returns_actor_not_running_when_already_stopping
Given: Actor in Stopping state (cancellation already in progress)
When: handle_cancel is called with CancelReason::UserRequested
Then: Err(Error::ActorNotRunning) is returned immediately
And: myself_ref.stop is NOT called again
And: store has zero events published by this call
And: outbox is unchanged

### Scenario: handle_cancel_returns_actor_not_running_when_stopped
Given: Actor in Stopped state
When: handle_cancel is called with CancelReason::UserRequested
Then: Err(Error::ActorNotRunning) is returned immediately
And: myself_ref.stop is NOT called
And: store has zero events
And: outbox is unchanged
```

### Behavior: Actor does NOT stop until cancellation persisted

```
### Scenario: actor_does_not_stop_during_pending_publish
Given: Actor in Running state with slow EventStore (simulated delay of 500ms)
When: handle_cancel is called
Then: myself_ref.stop is NOT called for at least 500ms after handle_cancel enters
And: myself_ref.stop is NOT called before store.publish returns
And: After store.publish returns success, myself_ref.stop IS called within 50ms

### Scenario: actor_does_not_stop_when_outbox_store_fails_then_succeeds
Given: Actor in Running state with EventStore that fails and outbox that fails first write then succeeds
When: handle_cancel is called
Then: myself_ref.stop is NOT called until outbox write succeeds
And: After outbox store succeeds, myself_ref.stop IS called
```

### Behavior: publish_with_compensation retry and fallback

```
### Scenario: publish_with_compensation_retries_with_exponential_backoff
Given: EventStore that fails 2 times then succeeds
When: publish_with_compensation is called
Then: store.publish is called 3 times total
And: delay between 1st and 2nd call is 100ms (±10ms)
And: delay between 2nd and 3rd call is 200ms (±10ms)
And: final result is Ok(())

### Scenario: publish_with_compensation_returns_publish_failed_after_max_retries
Given: EventStore that always fails and cancellation timeout NOT exceeded
When: publish_with_compensation is called with MAX_PUBLISH_RETRIES = 3
Then: store.publish is called exactly 3 times
And: result is Err(Error::PublishFailed(underlying_error))

### Scenario: publish_with_compensation_returns_cancellation_timeout_when_timeout_exceeded
Given: EventStore that always fails and cancellation timeout IS exceeded (simulated via clock)
When: publish_with_compensation is called after timeout budget depleted
Then: store.publish is called up to 3 times
And: result is Err(Error::CancellationTimeout)
And: outbox is NOT used (timeout occurred before fallback)

### Scenario: publish_with_compensation_falls_back_to_outbox
Given: EventStore that always fails and outbox with available capacity (< 100)
When: publish_with_compensation is called
Then: result is Ok(()) after all retries exhausted
And: outbox contains the event

### Scenario: publish_with_compensation_returns_outbox_full_when_at_capacity
Given: Outbox at capacity limit (100 events)
When: publish_with_compensation is called (after retries exhausted)
Then: result is Err(Error::OutboxFull)
And: outbox remains unchanged (still at capacity)
```

### Behavior: drain_outbox on recovery

```
### Scenario: drain_outbox_publishes_all_events_in_order
Given: EventStore (empty) and outbox with 3 events [A, B, C] in order
When: drain_outbox is called
Then: store.publish is called 3 times with events in order [A, B, C]
And: outbox is empty after drain

### Scenario: drain_outbox_returns_error_and_retains_events_when_store_fails
Given: EventStore that fails and outbox with 2 events
When: drain_outbox is called
Then: store.publish is called at least once
And: result is Err(Error::OutboxDrainFailed(underlying))
And: outbox still contains all original events

### Scenario: drain_outbox_failfast_stops_on_first_failure
Given: EventStore that fails on first event, outbox with events [X, Y, Z]
When: drain_outbox is called
Then: store.publish called for X only
And: result is Err(Error::OutboxDrainFailed)
And: outbox still contains [X, Y, Z]
And: store.publish is NOT called for Y or Z

### Scenario: drain_outbox_is_called_before_normal_message_processing
Given: Actor restarting with non-empty outbox containing event [E]
When: actor begins message processing
Then: drain_outbox completes successfully first
And: event [E] appears in store before any new events are processed

### Scenario: outbox_survives_actor_crash_and_is_drained_on_restart
Given: Actor with outbox containing [E] and simulated crash (process kill)
When: Actor restarts
Then: outbox still contains [E] after restart (persisted to disk)
And: drain_outbox publishes [E] to store
And: outbox is empty after drain
```

---

## 4. Proptest Invariants

### Proptest: publish_with_compensation always terminates

```
Invariant: publish_with_compensation must return within bounded time regardless of store behavior
Strategy:
  - store: func that returns any Result (Succeed, Fail with any error)
  - outbox_capacity: 0..100
  - event: any valid WorkflowEvent
  - max_retries: 1..5
Anti-invariant: An implementation that loops forever should be caught by timeout
```

### Proptest: Outbox capacity never exceeded

```
Invariant: After publish_with_compensation returns Ok(()), outbox.len() <= outbox.capacity
Strategy:
  - Initial outbox: vec with 0..capacity-1 events
  - capacity: 1..100
  - event: any valid WorkflowEvent
  - store: always fails to force outbox path
Anti-invariant: Implementation that blindly pushes without checking len should fail this invariant
```

### Proptest: Event order preserved in outbox

```
Invariant: Events appended to outbox maintain insertion order; drain returns them FIFO
Strategy:
  - Generate sequence of 1..20 unique events
  - Append each to outbox
  - Drain and verify output order matches input order
Anti-invariant: Implementation using unordered collection (HashSet) should fail
```

---

## 5. Fuzz Targets

### Fuzz Target: WorkflowEvent serialization round-trip

```
Input type: arbitrary bytes representing serialized WorkflowEvent
Risk:
  - Panic on deserialization of malformed data
  - OOM on specially crafted nested structure
  - Logic error: valid event deserializes incorrectly
Corpus seeds:
  - Minimal valid InstanceCancelled event
  - InstanceCancelled with empty reason
  - InstanceCancelled with maximum-length reason string
  - JSON-encoded event with unicode in reason field
```

### Fuzz Target: CancelReason parsing

```
Input type: arbitrary string for CancelReason
Risk:
  - Panic on invalid reason string
  - Rejection of valid reason strings (false positive validation)
  - Acceptance of invalid reason strings (false negative)
Corpus seeds:
  - Empty string
  - "UserRequested"
  - "Timeout"
  - "AdminCancelled"
  - Maximum length string (4096 chars)
  - Unicode characters
  - SQL injection / log injection patterns
```

---

## 6. Kani Harnesses

### Kani Harness: publish_with_compensation bounded termination

```
Property: publish_with_compensation returns after at most MAX_PUBLISH_RETRIES attempts
Bound: max_retries = 5, store can fail up to 100 times (but we only check 5)
Rationale: Critical to prove no infinite loop in retry logic. Any implementation that
           does not decrement retry counter or uses while(true) without break will be caught.
```

### Kani Harness: Outbox capacity invariant

```
Property: outbox.len() <= outbox.capacity() after every call to publish_with_compensation
Bound: capacity = 10, initial len = 0..10
Rationale: Memory safety critical. An overflow could cause OOM. Proptest catches typical
           cases but Kani proves no path can exceed capacity.
```

---

## 7. Mutation Checkpoints

### Critical mutations that MUST be caught:

| Mutation | Checkpoint Test |
|----------|-----------------|
| Remove retry counter decrement | `publish_with_compensation_returns_publish_failed_after_max_retries` |
| Change backoff to fixed delay | `publish_with_compensation_retries_with_exponential_backoff` |
| Remove outbox capacity check before push | `publish_with_compensation_returns_outbox_full_when_at_capacity` |
| Remove `myself_ref.stop` call (actor never stops) | `handle_cancel_persists_event_before_stop_when_publish_succeeds_first_try` |
| Call `myself_ref.stop` before publish completes | `actor_does_not_stop_during_pending_publish` |
| Skip outbox drain on startup | `drain_outbox_is_called_before_normal_message_processing` |
| Call `store.publish` after outbox full | `handle_cancel_returns_error_when_outbox_full` |
| Change FIFO to LIFO in drain | `drain_outbox_publishes_all_events_in_order` |
| Continue drain after failure (ignore fail-fast) | `drain_outbox_failfast_stops_on_first_failure` |
| Return PublishFailed instead of CancellationTimeout | `publish_with_compensation_returns_cancellation_timeout_when_timeout_exceeded` |
| Skip ActorNotRunning check | `handle_cancel_returns_actor_not_running_when_already_stopping` |
| Persist to outbox even when timeout exceeded | `publish_with_compensation_returns_cancellation_timeout_when_timeout_exceeded` |

**Threshold:** ≥90% mutation kill rate via cargo-mutants

---

## 8. Combinatorial Coverage Matrix

### handle_cancel

| Scenario | Input Class | Expected Output | Layer |
|----------|-------------|-----------------|-------|
| publish succeeds first try | store returns Ok | Ok(()), stop called | integration |
| publish succeeds after retries | store fails N-1 then Ok | Ok(()), stop called | integration |
| publish always fails, outbox has space | store always fails, outbox available | Ok(()), event in outbox, stop called | integration |
| publish always fails, outbox full | store always fails, outbox at capacity | Err(OutboxFull), stop NOT called | integration |
| actor already stopping | actor in Stopping state | Err(ActorNotRunning), stop NOT called | integration |
| actor already stopped | actor in Stopped state | Err(ActorNotRunning), stop NOT called | integration |
| timeout exceeded before fallback | store fails, timeout exceeded | Err(CancellationTimeout) | unit |

### publish_with_compensation

| Scenario | Input Class | Expected Output | Layer |
|----------|-------------|-----------------|-------|
| happy path | store returns Ok immediately | Ok(()) | unit |
| retry exhausted | store returns Err N times, timeout NOT exceeded | Err(PublishFailed) | unit |
| timeout exceeded | store returns Err, timeout exceeded | Err(CancellationTimeout) | unit |
| outbox fallback | store fails, timeout NOT exceeded, outbox has capacity | Ok(()), event in outbox | unit |
| outbox full | store fails, outbox at capacity (100) | Err(OutboxFull) | unit |
| capacity: min (1) | capacity = 1, outbox empty | Ok(()), len = 1 | unit |
| capacity: boundary (100) | capacity = 100 | Ok(()) or Err depending on state | unit |
| capacity: zero | capacity = 0 | Err(OutboxFull) immediately | unit |
| backoff timing | store fails then succeeds | delays: ~100ms, ~200ms | unit |

### drain_outbox

| Scenario | Input Class | Expected Output | Layer |
|----------|-------------|-----------------|-------|
| empty outbox | outbox empty | Ok(()), outbox still empty | unit |
| single event | outbox has 1 event | Ok(()), event published, outbox empty | integration |
| multiple events in order | outbox has [A, B, C] | Ok(()), published in order [A, B, C] | integration |
| store fails on first | store fails on first event | Err(OutboxDrainFailed), outbox unchanged | integration |
| fail-fast behavior | store fails on X, outbox has [X, Y, Z] | Err on X, Y/Z not attempted | integration |
| outbox persisted to disk | simulate crash after outbox write | outbox survives restart | integration |

---

## 9. Error Variant Coverage

Every error in `Error` enum MUST have a test:

| Error Variant | Test Scenario |
|---------------|---------------|
| `Error::PublishFailed(underlying)` | `publish_with_compensation_returns_publish_failed_after_max_retries` |
| `Error::OutboxFull` | `publish_with_compensation_returns_outbox_full_when_at_capacity` |
| `Error::OutboxDrainFailed(underlying)` | `drain_outbox_returns_error_and_retains_events_when_store_fails` |
| `Error::CancellationTimeout` | `publish_with_compensation_returns_cancellation_timeout_when_timeout_exceeded` |
| `Error::ActorNotRunning` | `handle_cancel_returns_actor_not_running_when_already_stopping` |

---

## Open Questions

1. ~~**MAX_PUBLISH_RETRIES default?**~~ **RESOLVED: 3**
2. ~~**Outbox capacity default?**~~ **RESOLVED: 100**
3. ~~**Exponential backoff formula?**~~ **RESOLVED: 100ms initial, 2x multiplier**
4. ~~**Should drain_outbox fail-fast?**~~ **RESOLVED: Yes, stop on first failure**
5. ~~**Cancel while already stopping?**~~ **RESOLVED: Return ActorNotRunning**
6. ~~**Outbox disk persistence?**~~ **RESOLVED: Required, must survive crash**

---

## Exit Criteria Checklist

- [x] Every public API behavior has at least one BDD scenario
- [x] Every Error variant in Error enum has an explicit test scenario (5/5)
- [x] No test asserts only `is_ok()` or `is_err()` — all specify exact values
- [x] Every pure function with multiple inputs has at least one proptest invariant
- [x] Every parsing/deserialization boundary has a fuzz target
- [x] Critical invariants (bounded retry, outbox capacity) have Kani harnesses
- [x] Mutation checkpoint list includes all critical code paths
- [x] ≥90% mutation kill rate threshold stated
- [x] Actor does NOT stop until cancellation persisted — explicit scenarios for this
- [x] Cancellation persists even when first publish fails — explicit scenario for outbox fallback
- [x] CancellationTimeout tested as distinct from PublishFailed
- [x] ActorNotRunning scenario explicit in both contract and plan
- [x] Concrete backoff timing values specified
- [x] Outbox crash survival verified

---

## Contract Changes Summary

The following changes were made to `contract.md`:

1. **Added `ActorNotRunning` to Error Taxonomy** — Resolves LETHAL #2
2. **Added `CancellationTimeout` distinction** — Explicit note that it's distinct from `PublishFailed`
3. **Answered Open Questions with concrete values:**
   - MAX_PUBLISH_RETRIES = 3
   - OUTBOX_CAPACITY = 100
   - Backoff: 100ms initial, 2x multiplier
4. **Added Step 5: Cancellation During Shutdown** — Defines ActorNotRunning behavior
5. **Specified drain_outbox fail-fast behavior** — Stop on first failure
6. **Added disk persistence requirement** — Outbox must survive crash

(End file - total 459 lines)