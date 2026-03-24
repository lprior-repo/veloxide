# Test Plan Review: ActivityCompleted Idempotency (wtf-l7zi)

## STATUS: APPROVED

---

## Axis 1 — Contract Parity: ✅ PASS

**Contract.md public functions:**
- `pub fn apply_event(state: &ProceduralActorState, event: &WorkflowEvent, seq: u64) -> Result<(ProceduralActorState, ProceduralApplyResult), ProceduralApplyError>`

**Test-plan.md coverage mapping:**

| Contract Requirement | BDD Scenario | Test Function |
|---------------------|--------------|---------------|
| Happy path: activity_id in in_flight → normal completion | Behavior 1 | `apply_event_returns_activity_completed_when_activity_id_in_in_flight` |
| Idempotent: NOT in in_flight, IS in checkpoint_map → AlreadyApplied | Behavior 2 | `apply_event_returns_already_applied_when_duplicate_completion_after_success` |
| Error: NOT in in_flight, NOT in checkpoint_map → UnknownActivityId | Behavior 3 | `apply_event_returns_unknown_activity_id_error_when_truly_unknown` |
| Non-panic: checkpoint_map lookup doesn't crash | Behavior 4 | `apply_event_does_not_crash_on_unknown_activity_id_when_in_checkpoint_map` |
| Invariants: in_flight/checkpoint_map separation | Behavior 5 | `apply_event_maintains_invariants_across_duplicate_completions` |

**Error variant coverage:**
- `ProceduralApplyError::UnknownActivityId(String)` — Behavior 3 asserts exact variant with concrete `activity_id = "ghost"` ✅
- `AlreadyApplied` is `ProceduralApplyResult`, NOT an error — covered by Behavior 2 ✅

**No missing functions. No missing error variants. No is_err() naked assertions.**

---

## Axis 2 — Assertion Sharpness: ✅ PASS

Every "Then:" block specifies concrete values:

| Scenario | Assertion | Sharpness |
|----------|-----------|-----------|
| 1 | `Ok((new_state, ProceduralApplyResult::ActivityCompleted { operation_id: 0, result: Bytes::from_static(b"ok") }))` | ✅ Exact struct with concrete fields |
| 2 | `Ok((state.clone(), ProceduralApplyResult::AlreadyApplied))` | ✅ Exact variant |
| 3 | `Err(ProceduralApplyError::UnknownActivityId(activity_id))` where `activity_id = "ghost"` | ✅ Exact error with concrete string |
| 4 | `Ok((state.clone(), ProceduralApplyResult::AlreadyApplied))` | ✅ Exact variant |
| 5 | Each duplicate → `AlreadyApplied`, `in_flight` never grows, `checkpoint_map` never modified | ✅ Concrete state predicates |

**No instances of:**
- `assert!(result.is_ok())` or `assert!(result.is_err())` — LETHAL pattern absent
- `> 0` without concrete inner value — absent
- `Some(_)` without specifying inner value — absent

---

## Axis 3 — Trophy Allocation: ✅ PASS

| Layer | Count | Rationale |
|-------|-------|-----------|
| Unit | 11 | Pure state transition — no I/O, exhaustive branch coverage |
| Integration | 4 | Real HashMap/HashSet, state transitions |
| E2E | 0 | Correct — no CLI/API changed |
| Proptest | 1 | Idempotency invariant for ActivityCompleted |
| Kani | 1 | Exhaustive state machine completeness proof |
| Fuzz | 0 | Correct — no parsing/deserialization boundary introduced |

**Density:** 11 unit tests for 1 public function = 11x (target ≥5x) ✅

**Unit-heavy allocation is justified:** This is a pure in-memory state machine. No network, no filesystem, no concurrency. The combinatorial space of `in_flight` × `checkpoint_map` membership is best covered by isolated unit tests.

---

## Axis 4 — Boundary Completeness: ⚠️ MINOR (1)

| Boundary | Covered? | Notes |
|----------|----------|-------|
| Minimum valid: activity_id in in_flight | ✅ | Scenario 1 |
| Maximum valid: multiple concurrent in-flight | ✅ | Scenario 5 |
| Empty/zero: empty string activity_id | ⚠️ NOT EXPLICIT | What happens with `activity_id = ""`? |
| Overflow: N/A | N/A | No numeric overflow in this fix |
| One-below-min: activity_id missing from both maps | ✅ | Scenario 3 (UnknownActivityId) |
| One-above-max: N/A | N/A | No value-based upper bound in contract |

**Finding:** The empty string `""` case for `activity_id` is not explicitly named as a boundary condition. While HashMap lookup handles this naturally (returns None), the plan does not explicitly call it out.

**Severity:** 1 missing boundary < 3 threshold → **MINOR only**

---

## Axis 5 — Mutation Survivability: ✅ PASS

| Mutation | Test That Kills It |
|----------|-------------------|
| Change `UnknownActivityId` error path to return `AlreadyApplied` | `apply_event_returns_unknown_activity_id_error_when_truly_unknown` |
| Remove the `checkpoint_map` reverse lookup | `apply_event_does_not_crash_on_unknown_activity_id_when_in_checkpoint_map` |
| Add panic/unwrap after `in_flight` lookup failure | `apply_event_does_not_crash_on_unknown_activity_id_when_in_checkpoint_map` |
| Swap `AlreadyApplied` and `UnknownActivityId` return values | Both error-path tests |
| Mutate checkpoint_map contents during duplicate handling | `apply_event_returns_already_applied_when_duplicate_completion_after_success` |

**Mutation kill rate target: ≥90%** — explicitly stated in plan.

**Mutation checkpoint table is present with exact test mappings.** ✅

---

## Axis 6 — Holzmann Rules (Plan Audit): ✅ PASS

| Rule | Applied? | Evidence |
|------|----------|----------|
| Rule 2 — Bound Every Loop | ✅ | Test scenarios described as linear Given→When→Then prose, no loops mentioned |
| Rule 5 — State Your Assumptions | ✅ | Every BDD scenario has explicit **Given:** block with preconditions |
| Rule 7 — Narrow Your State | ✅ | Each scenario creates isolated ProceduralActorState, no shared mutable state |
| Rule 8 — Surface Your Side Effects | ✅ | No side-effectful helpers with innocent names; setup is explicit in Given blocks |
| Rule 10 — Warnings Are Errors | ✅ | Static analysis layer includes clippy with deny directives |

---

## Severity Aggregation

| Severity | Count | Threshold | Result |
|----------|-------|-----------|--------|
| LETHAL | 0 | Any = REJECTED | ✅ PASS |
| MAJOR | 0 | ≥3 = REJECTED | ✅ PASS |
| MINOR | 1 | ≥5 = REJECTED | ✅ PASS (1 < 5) |

**VERDICT: APPROVED** — 0 LETHAL + 0 MAJOR + 1 MINOR

---

## Recommendations (Non-Blocking)

These are suggested improvements that do not block approval:

1. **Add explicit empty string boundary**: Consider adding a scenario or noting that `activity_id = ""` is covered by the "NOT in in_flight AND NOT in checkpoint_map" path (Scenario 3). The proptest invariant with "any non-empty string" strategy may miss this, but HashMap behavior makes it safe.

---

## Exit Criteria Verification

- ✅ Every public API behavior has at least one BDD scenario (5 behaviors → 5 scenarios)
- ✅ Every pure function with multiple inputs has at least one proptest invariant (1 invariant for `apply_event`)
- ✅ Every parsing/deserialization boundary has a fuzz target (0 — none introduced — CORRECT)
- ✅ Every error variant in the Error enum has an explicit test scenario (`UnknownActivityId` covered with exact variant)
- ✅ The mutation threshold target (≥90%) is stated
- ✅ No test asserts only `is_ok()` or `is_err()` without specifying the value — all scenarios specify exact values or exact error variants

---

**APPROVED FOR IMPLEMENTATION**
