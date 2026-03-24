# Test Plan Review: wtf-wnbu (Mode 1 — Plan Inquisition)

## VERDICT: REJECTED

---

## Previous Defect Review

| Defect | Was Lethal? | Status |
|--------|-------------|--------|
| LETHAL 1: Contract defines 1 function, plan tests 4+ | YES | **FIXED** — Contract now defines 5 functions |
| LETHAL 2: Test density 1.75x, not 5x | YES | **NOT FIXED** — Was 1.75x, now 3.2x |
| MAJOR 1: No proptest invariant for instance_id_from_heartbeat_key | YES (Major) | **FIXED** — Three invariants now exist |
| MAJOR 2: check_recovery_preconditions condition order mutation not tested | YES (Major) | **FIXED** — Anti-mutation test exists |

---

## Axis 1 — Contract Parity: PASS

Contract defines **5 functions** with explicit pre/postconditions (lines 45-186):

| # | Function | Lines | Test Coverage |
|---|----------|-------|---------------|
| 1 | `acquire_in_flight_guard()` | 45-71 | 2 BDD scenarios |
| 2 | `check_recovery_preconditions(state, instance_id) -> Option<String>` | 75-100 | 5 BDD scenarios |
| 3 | `attempt_recovery(myself, state, instance_id, in_flight_key) -> ()` | 103-133 | 3 BDD scenarios |
| 4 | `handle_heartbeat_expired(myself, state, instance_id) -> ()` | 136-160 | 2 BDD scenarios |
| 5 | `instance_id_from_heartbeat_key(key: &str) -> Option<InstanceId>` | 163-186 | 10 combinatorial + 3 proptest invariants |

All 5 functions have corresponding BDD scenarios. No missing coverage.

---

## Axis 2 — Assertion Sharpness: PASS

All "Then:" assertions specify concrete values. Sample:

- Line 76: `None is returned immediately`
- Line 91-92: `in_flight_key = instance_id.to_string() is returned` AND `in_flight_key is inserted into the in_flight HashSet`
- Line 118: `Some("") is returned`
- Line 136: `None is returned` + `in_flight HashSet remains empty`

No `is_ok()`, `is_err()`, `Some(_)`, or boolean-without-value assertions found. Axis passes.

---

## Axis 3 — Trophy Allocation: **LETHAL**

**LETHAL FINDING: test-plan.md:31-40 + line 407**

**Math error in density calculation:**

The plan claims (line 407):
> "Test density meets threshold (LETHAL 2 fixed — 16 unit tests for 4 functions = 4x)"

**This is factually incorrect. The contract defines 5 functions, not 4.**

| Function Count | Unit Tests | Actual Density | Required | Gap |
|---------------|------------|----------------|----------|-----|
| 5 | 16 | **3.2x** | 5x (25 tests) | **9 tests missing** |

Evidence that there are 5 functions:
- Line 163-186: `### Function 5: instance_id_from_heartbeat_key(key: &str) -> Option<InstanceId>`
- Exit criteria (line 403): "Contract defines all 5 functions being tested"
- The previous review (line 27 of prior review): "Contract now defines all 5 functions"

The plan counts 4 functions ("4 primary functions" at line 30-31) but the contract has 5. Even accepting the erroneous 4-function count, 16/4 = 4x is still **below the required 5x**.

**Required:** 5 functions × 5 = **25 unit tests minimum**
**Planned:** 16 unit tests
**Gap:** 9 unit tests missing

---

## Axis 4 — Boundary Completeness: PASS

| Function | Boundaries Named | Missing |
|----------|-----------------|---------|
| `instance_id_from_heartbeat_key` | Empty string, `hb/`, wrong prefix, extra segment, valid with underscore/hyphen, no prefix, space in prefix, very long strings (fuzz) | None beyond fuzz |
| `check_recovery_preconditions` | active check, duplicate in-flight, empty instance_id, condition order | None critical |

The fuzz target (lines 285-300) explicitly includes "very long strings for memory/allocation testing." This covers the one-above-maximum boundary for the parsing function.

---

## Axis 5 — Mutation Survivability: PASS

All critical mutations are covered:

| Mutation | Test | Lines |
|----------|------|-------|
| Condition order swapped in `check_recovery_preconditions` | `check_recovery_preconditions_checks_active_before_insert_to_prevent_guard_modification` | 130-141 |
| `.await` removed from `acquire_in_flight_guard` | Integration tests + Kani harness | 305-318 |
| `remove(in_flight_key)` call removed | `duplicate_heartbeat_expired_triggers_single_recovery` | 329 |
| Key cleanup ordering | `attempt_recovery_removes_key_after_spawn_not_before` | 191 |

---

## Axis 6 — Proptest Invariants: PASS

Three invariants for `instance_id_from_heartbeat_key` (lines 252-279):
1. Non-"hb/" strings → `None`
2. Valid "hb/[a-zA-Z0-9_-]+" → `Some`
3. Invalid "hb/" variants (`"hb/"`, `"hb//"`, `"hb/01ARZ/extra"`) → `None`

Plus 2 invariants for `acquire_in_flight_guard` (lines 222-248).

---

## Summary of Findings

| Severity | Count | Threshold | Action |
|----------|-------|-----------|--------|
| **LETHAL** | 1 | Any = REJECTED | Stop |
| MAJOR | 0 | ≥3 = REJECTED | — |
| MINOR | 0 | ≥5 = REJECTED | — |

---

## LETHAL FINDINGS

1. **test-plan.md:31-40, 407** — Test density **3.2x** below required **5x** threshold
   - 5 functions in contract × 5 = **25 unit tests required**
   - Plan provides: **16 unit tests**
   - Gap: **9 unit tests missing**
   - The plan claims "16 unit tests for 4 functions = 4x" but contract has **5 functions** (see Function 5 at lines 163-186)
   - Even accepting the erroneous 4-function count, 16/4 = 4x is still below 5x

---

## MAJOR FINDINGS (0)

None.

---

## MINOR FINDINGS (0)

None.

---

## MANDATE

To achieve APPROVED, add **9 unit tests** to reach 5x density:

### High Priority Gaps (from combinatorial matrix)

1. `instance_id_from_heartbeat_key` edge cases (from lines 358-372):
   - `instance_id_from_heartbeat_key_rejects_unicode_in_key_portion()`
   - `instance_id_from_heartbeat_key_rejects_control_characters()`
   - `instance_id_from_heartbeat_key_handles_max_length_key()` — boundary: one-above-maximum

2. `check_recovery_preconditions` scenarios (from lines 341-347):
   - `check_recovery_preconditions_handles_instance_id_with_special_characters()`
   - `check_recovery_preconditions_concurrent_safe_with_multiple_instances()`

3. `attempt_recovery` coverage:
   - `attempt_recovery_removes_key_even_when_fetch_metadata_returns_error()`
   - `attempt_recovery_does_not_register_on_spawn_error_after_metadata_fetch()`

4. Async safety verification:
   - `acquire_in_flight_guard_guard_drop_releases_to_executor_immediately()`
   - `handle_heartbeat_expired_drops_guard_before_awaiting_attempt_recovery()`

### Correct the Density Calculation

Line 407 should read: "Test density meets threshold (16 unit tests / 5 functions = 3.2x — **9 tests short of 5x target**)"

Or add 9 tests to reach 25 total.

---

**REJECTED — Resubmit after addressing the 9-test gap and correcting the function count to 5.**
