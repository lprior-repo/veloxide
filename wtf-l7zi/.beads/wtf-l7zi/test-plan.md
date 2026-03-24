# Test Plan: ActivityCompleted Idempotency Fix (wtf-l7zi)

## Summary

- **Bead:** wtf-l7zi
- **Behaviors identified:** 5 core behaviors
- **Trophy allocation:** 11 unit / 4 integration / 0 e2e / static analysis
- **Proptest invariants:** 1
- **Fuzz targets:** 0 (no parsing boundary introduced by this fix)
- **Kani harnesses:** 1 (state machine completeness)

## 1. Behavior Inventory

1. **[Subject]** `apply_event` **[action]** returns `ActivityCompleted` with correct `operation_id` and result **[outcome]** when `activity_id` exists in `in_flight` **[condition]**
2. **[Subject]** `apply_event` **[action]** returns `AlreadyApplied` and leaves state unchanged **[outcome]** when `activity_id` is NOT in `in_flight` but IS in `checkpoint_map` (duplicate after success) **[condition]**
3. **[Subject]** `apply_event` **[action]** returns `Err(UnknownActivityId)` **[outcome]** when `activity_id` is NOT in `in_flight` and NOT in `checkpoint_map` (truly unknown) **[condition]**
4. **[Subject]** `apply_event` **[action]** does NOT crash/panic **[outcome]** when `activity_id` is unknown but exists in `checkpoint_map` (the "AlreadyApplied" case) **[condition]**
5. **[Subject]** state invariants **[action]** maintain correct separation between `in_flight` and `checkpoint_map` **[outcome]** across all operations **[condition]**

## 2. Trophy Allocation

| Layer | Count | Rationale |
|-------|-------|-----------|
| **Unit** | 11 | Calc layer — pure functions with no I/O. Exhaustive coverage of all `apply_event` branches for `ActivityCompleted`. |
| **Integration** | 4 | Component boundary testing — `ProceduralActorState` + `apply_event` with real HashMap/HashSet. Tests verify state transitions and invariants. |
| **E2E** | 0 | No CLI/API entry point changed. Behavior is entirely internal to the actor state machine. |
| **Static** | clippy + types | `#[deny(clippy::unwrap_used)]`, `#[deny(clippy::panic)]` already enforced in `apply.rs`. No new code paths introduced. |

### Rationale for unit-heavy allocation

This fix is a **pure state transition** — no I/O, no network, no concurrency. The behavior is entirely determined by:
- The contents of `in_flight` HashMap
- The contents of `checkpoint_map` HashMap
- The presence/absence of `activity_id` in either

The combinatorial explosion of states (activity in-flight, activity checkpointed, activity neither) is best covered by unit tests with isolated state objects.

## 3. BDD Scenarios

---

### Behavior: apply_event returns ActivityCompleted when activity_id is in in_flight

**Given:** `ProceduralActorState` with `activity_id="act-1"` present in `in_flight` (mapped to `operation_id=0`)

**When:** `apply_event(state, ActivityCompleted { activity_id: "act-1", result: Bytes::from_static(b"ok") }, seq=2)` is called

**Then:** 
- Returns `Ok((new_state, ProceduralApplyResult::ActivityCompleted { operation_id: 0, result: Bytes::from_static(b"ok") }))`
- `new_state.in_flight` does NOT contain `operation_id=0`
- `new_state.checkpoint_map` contains `operation_id=0` with `result=Bytes::from_static(b"ok")` and `completed_seq=2`

**Test function:** `fn apply_event_returns_activity_completed_when_activity_id_in_in_flight()`

---

### Behavior: apply_event returns AlreadyApplied when activity_id is NOT in in_flight but IS in checkpoint_map (duplicate after success)

**Given:** `ProceduralActorState` where `activity_id="act-1"` was previously completed and removed from `in_flight`; `checkpoint_map` contains `operation_id=0 → Checkpoint { result: ..., completed_seq: 1 }`

**When:** `apply_event(state, ActivityCompleted { activity_id: "act-1", result: Bytes::from_static(b"ok") }, seq=2)` is called

**Then:**
- Returns `Ok((state.clone(), ProceduralApplyResult::AlreadyApplied))`
- Returned state is identical to input state (no mutations)
- `in_flight` remains empty
- `checkpoint_map` is unchanged

**Test function:** `fn apply_event_returns_already_applied_when_duplicate_completion_after_success()`

---

### Behavior: apply_event returns Err(UnknownActivityId) when activity_id is truly unknown

**Given:** `ProceduralActorState` with empty `in_flight` and empty `checkpoint_map`

**When:** `apply_event(state, ActivityCompleted { activity_id: "ghost", result: Bytes::from_static(b"ok") }, seq=1)` is called

**Then:**
- Returns `Err(ProceduralApplyError::UnknownActivityId(activity_id))` where `activity_id = "ghost"`
- State is unchanged from input

**Test function:** `fn apply_event_returns_unknown_activity_id_error_when_truly_unknown()`

---

### Behavior: apply_event does NOT crash when activity_id is NOT in in_flight but IS in checkpoint_map

**Given:** `ProceduralActorState` where `activity_id="act-1"` is NOT in `in_flight` (already completed and removed) but IS in `checkpoint_map` (via reverse lookup of operation_id)

**When:** `apply_event(state, ActivityCompleted { activity_id: "act-1", result: Bytes::from_static(b"dup") }, seq=3)` is called

**Then:**
- Returns `Ok((state.clone(), ProceduralApplyResult::AlreadyApplied))` — NOT a panic
- State is unchanged

**Test function:** `fn apply_event_does_not_crash_on_unknown_activity_id_when_in_checkpoint_map()`

---

### Behavior: apply_event maintains invariants across concurrent duplicate completions

**Given:** `ProceduralActorState` with multiple activities in-flight and checkpointed

**When:** Multiple duplicate `ActivityCompleted` events are applied in sequence for already-completed activities

**Then:**
- Each duplicate returns `AlreadyApplied`
- `in_flight` never grows during duplicate processing
- `checkpoint_map` is never modified during duplicate processing
- All original in-flight and checkpointed entries remain intact

**Test function:** `fn apply_event_maintains_invariants_across_duplicate_completions()`

---

## 4. Proptest Invariants

### Invariant: apply_event is idempotent for ActivityCompleted

**Property:** Applying the same `ActivityCompleted { activity_id, result }` event with the same `seq` always returns the same result and same state (deterministic replay).

**Strategy:** Generate valid combinations of:
- `activity_id`: any non-empty string
- `result`: any Bytes value
- `seq`: any u64

**Anti-invariant:** N/A — all valid inputs must produce deterministic output.

**Note:** The `seq` idempotency is already tested separately (applied_seq HashSet). This invariant focuses on the `activity_id` lookup path.

---

## 5. Fuzz Targets

**None required.** This fix does not introduce any new parsing, deserialization, or user-input boundaries. The change is purely in-memory state machine logic.

---

## 6. Kani Verification Harnesses

### Kani Harness: activity_id lookup is exhaustive

**Property:** For any `ProceduralActorState` and any `ActivityId`, `apply_event` for `ActivityCompleted` either:
1. Finds the `activity_id` in `in_flight` and returns `ActivityCompleted`, OR
2. Does NOT find the `activity_id` in `in_flight` and checks `checkpoint_map` — returning `AlreadyApplied` if found, `UnknownActivityId` error if not found

**Bound:** State with up to 10 entries in `in_flight` and 10 entries in `checkpoint_map`.

**Rationale:** This is a critical safety property — the actor must NEVER panic on an unknown `activity_id`. Formal verification ensures the conditional logic covers all cases without panicking on `.unwrap()` or `.expect()`.

---

## 7. Mutation Testing Checkpoints

### Critical mutations that must be caught:

| Mutation | Must be caught by test |
|----------|------------------------|
| Change `UnknownActivityId` error path to return `AlreadyApplied` | `apply_event_returns_unknown_activity_id_error_when_truly_unknown` |
| Remove the `checkpoint_map` reverse lookup (leaving only `in_flight` check) | `apply_event_does_not_crash_on_unknown_activity_id_when_in_checkpoint_map` |
| Add panic/unwrap after `in_flight` lookup failure | `apply_event_does_not_crash_on_unknown_activity_id_when_in_checkpoint_map` |
| Swap `AlreadyApplied` and `UnknownActivityId` return values | Both error-path tests |
| Mutate checkpoint_map contents during duplicate handling | `apply_event_returns_already_applied_when_duplicate_completion_after_success` |

**Threshold:** ≥90% mutation kill rate.

---

## 8. Combinatorial Coverage Matrix

### `apply_event` for `ActivityCompleted` branch

| Scenario | Input Class | Expected Output | Layer |
|----------|-------------|-----------------|-------|
| Happy path | `activity_id` in `in_flight` | `Ok(ActivityCompleted { op_id, result })` | unit |
| Idempotent dup (after success) | `activity_id` NOT in `in_flight`, IS in `checkpoint_map` | `Ok(AlreadyApplied)` | unit |
| Unknown activity | `activity_id` NOT in `in_flight`, NOT in `checkpoint_map` | `Err(UnknownActivityId(id))` | unit |
| Crash on unknown | `activity_id` NOT in `in_flight`, IS in `checkpoint_map` (pre-fix behavior) | panic | unit (negative) |
| State unchanged | Any duplicate path | State identical to input | unit |
| Invariant: no cross-contamination | Any duplicate path | `in_flight` unchanged, `checkpoint_map` unchanged | unit |
| Multiple in-flight ops | `activity_id` maps to specific `op_id` | Correct `op_id` returned | unit |
| Multiple checkpointed ops | `activity_id` maps to specific `op_id` | `AlreadyApplied` returned | unit |

### `applied_seq` idempotency (pre-existing, baseline)

| Scenario | Input Class | Expected Output | Layer |
|----------|-------------|-----------------|-------|
| Duplicate seq | Same `seq` applied twice | `AlreadyApplied` on second call | unit |
| New seq | Unique `seq` | Normal processing | unit |

---

## 9. Error Variant Coverage

| Error Variant | Test Scenario |
|---------------|---------------|
| `ProceduralApplyError::UnknownActivityId(String)` | `apply_event_returns_unknown_activity_id_error_when_truly_unknown` |
| N/A — `AlreadyApplied` is NOT an error | Covered by `apply_event_returns_already_applied_when_duplicate_completion_after_success` |

---

## 10. Integration Test Scenarios

These tests use real `HashMap`/`HashSet` and exercise `apply_event` with multiple state transitions:

| Test | Description |
|------|-------------|
| `checkpoint_persists_across_crash_state_machine` | Pre-existing. Verifies checkpoint_map correctly stores completed operations. |
| `exactly_once_activity_dispatch_via_checkpoint_map` | Pre-existing. Verifies re-dispatch returns `AlreadyApplied`. |
| `replay_after_crash_restores_checkpoint_state` | Pre-existing. Verifies crash recovery skips completed ops. |
| `apply_event_idempotent_on_duplicate_activity_completion` | **NEW.** Full workflow: dispatch → complete → duplicate complete. State unchanged, `AlreadyApplied` returned. |

---

## Open Questions

1. **None.** The contract is fully specified with no ambiguities.

---

## Exit Criteria Verification

- ✅ Every public API behavior has at least one BDD scenario (5 behaviors → 5 scenarios)
- ✅ Every pure function with multiple inputs has at least one proptest invariant (1 invariant for `apply_event`)
- ✅ Every parsing/deserialization boundary has a fuzz target (0 — none introduced)
- ✅ Every error variant in the Error enum has an explicit test scenario (`UnknownActivityId` covered)
- ✅ The mutation threshold target (≥90%) is stated
- ✅ No test asserts only `is_ok()` or `is_err()` without specifying the value — all scenarios specify exact values or exact error variants
