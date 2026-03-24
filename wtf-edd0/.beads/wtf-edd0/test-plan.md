# Test Plan: wtf-edd0 — Cleanup Pending Entries on `inject_event` Failure

## Summary

- **Bead:** wtf-edd0
- **Feature:** Cleanup pending entries when `inject_event` fails after insertion
- **Behaviors identified:** 6
- **Trophy allocation:** 10 unit / 4 integration / 1 e2e
- **Proptest invariants:** 2
- **Fuzz targets:** 0 (no parsing boundaries in this feature)
- **Kani harnesses:** 1 (cleanup atomicity)
- **Mutation threshold target:** ≥90%

---

## 1. Behavior Inventory

| # | Behavior |
|---|----------|
| 1 | `append_and_inject_event` removes pending activity entry when `inject_event` returns `Err` |
| 2 | `append_and_inject_event` keeps pending activity entry when `inject_event` returns `Ok` |
| 3 | `append_and_inject_timer_event` removes pending timer entry when `inject_event` returns `Err` |
| 4 | `append_and_inject_timer_event` keeps pending timer entry when `inject_event` returns `Ok` |
| 5 | `append_and_inject_event` sends error reply when `inject_event` returns `Err` after pending inserted |
| 6 | Subsequent workflow events can be processed after a prior `inject_event` failure |

---

## 2. Trophy Allocation

| Layer | Count | Rationale |
|-------|-------|-----------|
| **Unit** | 10 | Pure function behavior of the cleanup logic. No I/O. Test each function in isolation with injected failures. 5x coverage (10 tests for 2 functions) as required by review. |
| **Integration** | 4 | Real `EventStore` mock, real `InstanceState`, real `inject_event`. Verifies the full async chain including reply port delivery. |
| **E2E** | 1 | Simulates the complete workflow where one activity fails and a subsequent activity is dispatched — proving the workflow is not blocked. |
| **Static** | — | `cargo clippy -- -D warnings` catches unused variable / type mismatches in the fix. No new static checks needed. |

**Rationale for 5x unit coverage:** Each function has multiple input dimensions (aid present/absent, inject success/failure, timer_id boundary values) requiring exhaustive combinatorial coverage at the unit level before integration testing.

---

## 3. BDD Scenarios

### Behavior 1: Activity pending entry removed when `inject_event` fails

**Unit test:**
```rust
/// append_and_inject_event removes pending activity entry when inject_event returns Err
async fn append_and_inject_event_removes_pending_when_inject_fails() {
    // Given: InstanceState with a real EventStore that succeeds on publish
    //   but a failing inject_event (via mock paradigm_state)
    // When: append_and_inject_event is called with activity_id = Some(aid)
    // Then: reply port receives Err(WtfError::EventInjectionFailed)
    // And:  pending_activity_calls does NOT contain aid
}
```

**Integration test (real async, real reply port):**
```rust
/// append_and_inject_activity_event cleans up pending on inject failure — reply receives error
async fn append_and_inject_activity_event_returns_error_and_cleans_up_on_inject_failure() {
    // Given: an InstanceState with a real EventStore (publish succeeds) and
    //   a paradigm_state that makes inject_event return Err
    // When: handle_dispatch is called
    // Then: the reply port receives Err(WtfError::EventInjectionFailed)
    // And:  pending_activity_calls does not contain the activity_id
}
```

---

### Behavior 2: Activity pending entry kept when `inject_event` succeeds

**Unit test:**
```rust
/// append_and_inject_event keeps pending activity entry when inject_event returns Ok
async fn append_and_inject_event_keeps_pending_when_inject_succeeds() {
    // Given: InstanceState with a real EventStore and a paradigm_state where inject_event succeeds
    // When: append_and_inject_event is called with activity_id = Some(aid)
    // Then: pending_activity_calls contains aid with the reply port after call
}
```

---

### Behavior 3: Timer pending entry removed when `inject_event` fails

**Integration test:**
```rust
/// append_and_inject_timer_event removes pending timer entry when inject_event returns Err
async fn append_and_inject_timer_event_removes_pending_when_inject_fails() {
    // Given: an InstanceState with a real EventStore (publish succeeds) and
    //   a paradigm_state that makes inject_event return Err
    // When: handle_sleep is called
    // Then: pending_timer_calls does not contain the timer_id
    // And:  reply port receives Err(WtfError::EventInjectionFailed)
}
```

---

### Behavior 4: Timer pending entry kept when `inject_event` succeeds

**Integration test:**
```rust
/// append_and_inject_timer_event keeps pending timer entry when inject_event returns Ok
async fn append_and_inject_timer_event_keeps_pending_when_inject_succeeds() {
    // Given: InstanceState with a real EventStore and inject_event that succeeds
    // When: append_and_inject_timer_event is called with timer_id
    // Then: pending_timer_calls contains timer_id with the reply port
}
```

---

### Behavior 5: Activity sends error reply when `inject_event` fails

**Unit test:**
```rust
/// append_and_inject_event sends error reply when inject_event returns Err after pending inserted
async fn append_and_inject_event_sends_error_reply_when_inject_fails() {
    // Given: InstanceState with a real EventStore that succeeds on publish
    //   but a failing inject_event (via mock paradigm_state)
    // When: append_and_inject_event is called with activity_id = Some(aid)
    // Then: reply port receives Err(WtfError::EventInjectionFailed)
    // And:  pending_activity_calls does NOT contain aid
}
```

---

### Behavior 6: Subsequent workflow events process after prior `inject_event` failure

**E2E test:**
```rust
/// Workflow continues to process events after inject_event failure
async fn workflow_continues_after_activity_dispatch_inject_failure() {
    // Given: a running WorkflowInstance in Live phase
    // When: first activity dispatch has inject_event fail (pending inserted then removed)
    // And:  second activity dispatch is made
    // Then: second activity dispatch's pending entry is present (not blocked)
    // And:  second activity dispatch's pending entry has pending_activity_calls.len() == 1
    // And:  second activity dispatch's pending entry key equals the second activity's aid
}
```

---

### Error Variants

**Error Variant 1 — Activity inject_event failure:**
Given: Event store publish succeeds, but `inject_event` fails with `WtfError::EventInjectionFailed`
When: `append_and_inject_event(state, event, Some(activity_id), reply)` is called
Then: `reply` receives `Err(WtfError::EventInjectionFailed)`
And: `pending_activity_calls.contains_key(activity_id)` is `false`

**Error Variant 2 — Timer inject_event failure:**
Given: Event store publish succeeds, but `inject_event` fails with `WtfError::EventInjectionFailed`
When: `append_and_inject_timer_event(state, event, timer_id, reply)` is called
Then: `reply` receives `Err(WtfError::EventInjectionFailed)`
And: `pending_timer_calls.contains_key(timer_id)` is `false`

---

## 4. Unit Tests — Expanded Coverage (10 tests for 2 functions)

### append_and_inject_event Unit Tests (5 tests)

#### UT-1: aid=Some, inject=Err — pending removed, error reply sent
```rust
/// append_and_inject_event_removes_pending_and_sends_error_when_inject_fails
async fn append_and_inject_event_removes_pending_and_sends_error_when_inject_fails_with_aid() {
    // Given: activity_id = Some(42), inject_event fails
    // When: append_and_inject_event is called
    // Then: reply receives Err(WtfError::EventInjectionFailed)
    // And:  pending_activity_calls does not contain 42
}
```

#### UT-2: aid=Some, inject=Ok — pending kept
```rust
/// append_and_inject_event_keeps_pending_when_inject_succeeds_with_aid
async fn append_and_inject_event_keeps_pending_when_inject_succeeds_with_aid() {
    // Given: activity_id = Some(42), inject_event succeeds
    // When: append_and_inject_event is called
    // Then: pending_activity_calls contains 42 with the reply port
}
```

#### UT-3: aid=None, inject=Ok — pending unchanged
```rust
/// append_and_inject_event_does_not_modify_pending_when_aid_is_none
async fn append_and_inject_event_does_not_modify_pending_when_aid_is_none() {
    // Given: activity_id = None, inject_event succeeds
    // When: append_and_inject_event is called
    // Then: pending_activity_calls is unchanged (len stays same)
    // And:  reply port is NOT sent anything
}
```

#### UT-4: aid=None, inject=Err — pending unchanged (no pending to clean)
```rust
/// append_and_inject_event_does_not_modify_pending_when_aid_is_none_and_inject_fails
async fn append_and_inject_event_does_not_modify_pending_when_aid_is_none_and_inject_fails() {
    // Given: activity_id = None, inject_event fails
    // When: append_and_inject_event is called
    // Then: pending_activity_calls is unchanged
    // And:  reply receives Err(WtfError::EventInjectionFailed)
}
```

#### UT-5: aid=Some(i32::MAX), inject=Err — cleanup with max boundary
```rust
/// append_and_inject_event_removes_pending_when_inject_fails_with_max_aid
async fn append_and_inject_event_removes_pending_when_inject_fails_with_max_aid() {
    // Given: activity_id = Some(i32::MAX), inject_event fails
    // When: append_and_inject_event is called
    // Then: reply receives Err(WtfError::EventInjectionFailed)
    // And:  pending_activity_calls does not contain i32::MAX
}
```

### append_and_inject_timer_event Unit Tests (5 tests)

#### UT-6: timer_id valid, inject=Err — pending removed, error reply sent
```rust
/// append_and_inject_timer_event_removes_pending_and_sends_error_when_inject_fails
async fn append_and_inject_timer_event_removes_pending_and_sends_error_when_inject_fails() {
    // Given: timer_id = TimerId(100), inject_event fails
    // When: append_and_inject_timer_event is called
    // Then: reply receives Err(WtfError::EventInjectionFailed)
    // And:  pending_timer_calls does not contain TimerId(100)
}
```

#### UT-7: timer_id valid, inject=Ok — pending kept
```rust
/// append_and_inject_timer_event_keeps_pending_when_inject_succeeds
async fn append_and_inject_timer_event_keeps_pending_when_inject_succeeds() {
    // Given: timer_id = TimerId(100), inject_event succeeds
    // When: append_and_inject_timer_event is called
    // Then: pending_timer_calls contains TimerId(100) with the reply port
}
```

#### UT-8: timer_id=TimerId(0), inject=Err — minimum boundary cleanup
```rust
/// append_and_inject_timer_event_removes_pending_when_inject_fails_with_min_timer_id
async fn append_and_inject_timer_event_removes_pending_when_inject_fails_with_min_timer_id() {
    // Given: timer_id = TimerId(0), inject_event fails
    // When: append_and_inject_timer_event is called
    // Then: reply receives Err(WtfError::EventInjectionFailed)
    // And:  pending_timer_calls does not contain TimerId(0)
}
```

#### UT-9: timer_id=TimerId(i64::MAX), inject=Ok — maximum boundary
```rust
/// append_and_inject_timer_event_keeps_pending_when_inject_succeeds_with_max_timer_id
async fn append_and_inject_timer_event_keeps_pending_when_inject_succeeds_with_max_timer_id() {
    // Given: timer_id = TimerId(i64::MAX), inject_event succeeds
    // When: append_and_inject_timer_event is called
    // Then: pending_timer_calls contains TimerId(i64::MAX) with the reply port
}
```

#### UT-10: inject_event returns different error, cleanup still happens
```rust
/// append_and_inject_timer_event_removes_pending_on_any_inject_error
async fn append_and_inject_timer_event_removes_pending_on_any_inject_error() {
    // Given: timer_id = TimerId(50), inject_event returns Err(WtfError::Other)
    // When: append_and_inject_timer_event is called
    // Then: pending_timer_calls does not contain TimerId(50)
    // And:  reply receives the same error that inject_event returned
}
```

---

## 5. Proptest Invariants

### Invariant 1: Pending activity map correctness after any dispatch outcome

```
Invariant: After any call to handle_dispatch (whether inject_event succeeds or fails),
           pending_activity_calls contains entries ONLY for in-flight activities
           that have not yet received a reply.

Property: let before = pending_activity_calls.len();
          call handle_dispatch(...)
          let after = pending_activity_calls.len();
          // Either: inject succeeded → after = before + 1
          // Or: inject failed → after == before (entry was cleaned up)
          // Never: after = before + 1 with no reply sent (zombie)
```

### Invariant 2: Pending timer map correctness after any sleep outcome

```
Invariant: After any call to handle_sleep, pending_timer_calls contains entries
           only for timers that have not yet fired.

Property: Same pattern as invariant 1 for timer calls.
```

---

## 6. Fuzz Targets

No parsing or deserialization boundaries are introduced by this fix. No fuzz targets required.

---

## 7. Kani Harnesses

### Harness: Cleanup atomicity under inject_event failure

```
Property: If a pending entry is inserted and inject_event returns Err,
         the pending entry is removed before the function returns.

Bound: Single call to append_and_inject_event with one activity_id

Rationale: This is a critical invariant that must hold for ALL error paths
           in inject_event. Proptest can only check the one path we generate;
           Kani proves ALL paths within the bound are covered.
```

---

## 8. Mutation Testing Checkpoints

The following mutations must be caught by the test suite:

| Mutation | Must be caught by test |
|----------|------------------------|
| Remove `state.pending_activity_calls.remove(&aid)` in error branch | `UT-1` (append_and_inject_event_removes_pending_and_sends_error_when_inject_fails_with_aid) |
| Remove `state.pending_timer_calls.remove(&timer_id)` in error branch | `UT-6` (append_and_inject_timer_event_removes_pending_and_sends_error_when_inject_fails) |
| Remove `reply.send(Err(...))` in error branch | `UT-1` and `UT-5` (reply assertion) |
| Swap Ok/Err branches (keep pending on failure) | `UT-1`, `UT-5`, `UT-6`, `UT-8` |
| Remove the entire cleanup block (leave zombie pending) | `UT-1`, `UT-6`, and `workflow_continues_after_activity_dispatch_inject_failure` |
| Remove cleanup when aid=None | `UT-3`, `UT-4` |

**Threshold:** ≥90% mutation kill rate

---

## 9. Combinatorial Coverage Matrix

### `append_and_inject_event`

| Scenario | Input Class | Expected Output | Layer |
|----------|-------------|-----------------|-------|
| happy path | publish=Ok, inject=Ok, aid=Some(42) | pending contains 42, reply not sent | unit (UT-2) |
| inject fails | publish=Ok, inject=Err, aid=Some(42) | pending empty, reply=Err(EventInjectionFailed) | unit (UT-1) |
| aid=None, inject Ok | publish=Ok, inject=Ok, aid=None | pending unchanged, reply not sent | unit (UT-3) |
| aid=None, inject Err | publish=Ok, inject=Err, aid=None | pending unchanged, reply=Err(EventInjectionFailed) | unit (UT-4) |
| aid=max boundary | publish=Ok, inject=Err, aid=Some(i32::MAX) | pending empty, reply=Err | unit (UT-5) |
| store fails | publish=Err, aid=Some | pending empty, reply=Err(publish) | integration |

### `append_and_inject_timer_event`

| Scenario | Input Class | Expected Output | Layer |
|----------|-------------|-----------------|-------|
| happy path | publish=Ok, inject=Ok, timer_id=100 | pending contains 100, reply not sent | unit (UT-7) |
| inject fails | publish=Ok, inject=Err, timer_id=100 | pending empty, reply=Err(EventInjectionFailed) | unit (UT-6) |
| timer_id=0, inject Err | publish=Ok, inject=Err, timer_id=0 | pending empty, reply=Err | unit (UT-8) |
| timer_id=max, inject Ok | publish=Ok, inject=Ok, timer_id=i64::MAX | pending contains i64::MAX | unit (UT-9) |
| any inject error | inject returns other error | pending empty, reply=original error | unit (UT-10) |
| store fails | publish=Err | pending empty, reply=Err(publish) | integration |

### Workflow continuity

| Scenario | Input Class | Expected Output | Layer |
|----------|-------------|-----------------|-------|
| first inject fails, second succeeds | inject_event fails then Ok | first pending cleaned, second pending present | e2e |

---

## 10. Open Questions

- [x] Does `inject_event` failure leave any side effects on `paradigm_state` that need separate cleanup?  
  **Answer:** No — `inject_event` applies the event to paradigm_state atomically. If it returns Err, the state transition was rolled back. No extra cleanup needed.

- [x] Should the reply port be sent the error BEFORE or AFTER removing the pending entry?  
  **Answer:** The contract specifies `remove_pending && send reply error`. Order matters: send error first so caller knows the operation failed, then clean up the pending entry to avoid spurious wakeups.

- [x] Is there a race where a concurrent `handle_inject_event_msg` could remove the pending entry between our insert and our cleanup?  
  **Answer:** No — `handle_inject_event_msg` only removes entries when processing `ActivityCompleted`/`TimerFired` events, which would only arrive after a successful `inject_event`. Since our `inject_event` failed, no completion event will be delivered concurrently.

---

## 11. Contract Changes

The contract.md Error Taxonomy has been updated to clarify:
- `WtfError::EventInjectionFailed` is the only error variant within scope for this cleanup contract
- Other errors (e.g., event store unavailable) occur BEFORE pending insertion and are out of scope — covered by pre-existing tests

---

## 12. Review Fixes Applied

1. **[FIXED] Line 144 assertion:** Changed from vague `> 0` to concrete `pending_activity_calls.len() == 1` with assertion that the second activity's aid is the key present.

2. **[FIXED] Unit test count:** Increased from 3 to 10 (5x for 2 functions). Added UT-3 through UT-10 covering boundary conditions (None aid, max aid, min/max timer_id) and error path combinations.

3. **[FIXED] Reply assertion at unit level:** UT-1 and UT-5 explicitly assert `reply receives Err(WtfError::EventInjectionFailed)`.

4. **[FIXED] Proptest invariants at unit level:** Invariants 1 and 2 are specified with concrete property definitions that can be tested via proptest.
