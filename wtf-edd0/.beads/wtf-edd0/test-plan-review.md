# Test Plan Review: wtf-edd0

## VERDICT: REJECTED

---

## Executive Summary

The test plan has **1 LETHAL finding** and **2 MAJOR findings** that must be resolved before resubmission. The plan attempts to specify a cleanup-on-failure contract but contains a critical error variant mismatch and incomplete unit test assertions that would allow mutations to survive.

---

## LETHAL FINDINGS

### LETHAL #1: Error Variant Mismatch — `nats_publish` not in Contract

**Location:** test-plan.md:145 — Error Variants section
**Contract:** contract.md:31 — Error Taxonomy

The contract defines only one error variant introduced by this feature:
```
WtfError::EventInjectionFailed - when `inject_event` returns `Err`
```

The Error Variants section specifies a **different** error for the missing event store case:
```
Then: reply receives Err(WtfError::nats_publish("Event store missing"))
```

`WtfError::nats_publish("Event store missing")` is **not defined in the contract's Error Taxonomy**. The test plan is verifying behavior against an error variant that does not exist in the contract specification.

**Required fix:** Either (a) add `WtfError::nats_publish` to the contract's Error Taxonomy, or (b) change the test assertion to `Err(WtfError::EventInjectionFailed)` to match the contract.

---

## MAJOR FINDINGS

### MAJOR #1: Unit Test Assertion Gap — `reply.send` Not Verified at Unit Level

**Location:** test-plan.md:49-55 — Behavior 1 unit test description

The unit test for Behavior 1 (`append_and_inject_event removes pending activity entry when inject_event returns Err`) only asserts:
```
Then: pending_activity_calls does NOT contain aid after call
And: aid is not in pending_activity_calls.keys()
```

**Missing assertion:** No unit test explicitly verifies that `reply.send(Err(...))` is called in the error branch.

**Mutation vulnerability:** If a developer removes `reply.send(Err(WtfError::EventInjectionFailed))` from the error branch of `append_and_inject_event`, the unit test for Behavior 1 would **still pass**.

**Evidence:** The mutation table (test-plan.md:207) claims `append_and_inject_activity_event_returns_error_and_cleans_up_on_inject_failure` catches removal of `reply.send(Err(...))`, but this is an **integration test** (test-plan.md:60-67), not a unit test. The unit test description at line 49-55 has no assertion on the reply port.

**Required fix:** Add explicit reply assertion to the unit test description:
```
Then: reply port receives Err(WtfError::EventInjectionFailed)
And: pending_activity_calls does NOT contain aid
```

---

### MAJOR #2: Test Count Ambiguity — 3 Unit Tests Claimed, 2 Described

**Location:** test-plan.md:8 — Trophy allocation

The plan states:
- Unit tests: **3**
- Integration tests: **4**
- E2E: **1**
- Total: **8**

The behavior inventory lists **6 behaviors**. The plan claims 8 tests but:
- Only **2** have explicit unit test code blocks (Behavior 1 unit, Behavior 2 unit)
- The **3rd unit test** is not clearly identified or described
- The Error Variants section describes 3 scenarios that appear to be integration tests

If the Error Variants are counted as integration tests (their descriptions use "integration test" language like "real EventStore", "real reply port"), then:
- Unit tests: 2 (Behavior 1, Behavior 2) — but plan says 3
- Integration tests: 6 (Behavior 1 integration, Behavior 3, Behavior 4, + 3 Error Variants) — but plan says 4

**Required fix:** Either (a) add a 3rd explicit unit test scenario to the plan, or (b) correct the trophy allocation count to match the actual described tests.

---

## MINOR FINDINGS

### MINOR #1: Open Question 2 (Ordering) Not Tested
- **Location:** test-plan.md:250-251
- The contract specifies `remove_pending && send reply error` as the cleanup order
- No test scenario explicitly verifies the ORDER of these two operations
- Tests only verify final state (pending removed, reply has error), not sequence
- **Impact:** Mutation that swaps the order would not be caught

### MINOR #2: Function Naming Ambiguity
- **Location:** test-plan.md:64, 94 — "When: handle_dispatch/handle_sleep is called"
- But the contract functions under test are `append_and_inject_event` and `append_and_inject_timer_event`
- Unclear if tests call contract functions directly or through `handle_dispatch`/`handle_sleep` wrappers
- This creates traceability ambiguity between test scenarios and contract functions

### MINOR #3: Boundary Coverage — Max Pending Entries Not Specified
- **Location:** test-plan.md:217-242 — Combinatorial Coverage Matrix
- No scenario tests behavior when pending maps reach maximum capacity
- No "one-above-max" or "at-capacity" boundary tests defined

### MINOR #4: Behavior 5 Unit Test Not Described
- **Location:** test-plan.md:103-111
- Behavior 4 (`append_and_inject_timer_event keeps pending timer entry when inject_event returns Ok`) has a unit test description
- Behavior 5 (timer keeps pending) has a unit test listed in trophy allocation but no explicit unit test description block
- Behavior 4 is actually a unit test (per test-plan.md:103 header), but Behavior 5 is also listed as a unit test with no description

---

## AXIS-BY-AXIS SUMMARY

### Axis 1 — Contract Parity
| Contract Function | Covered by Plan? | Test Scenario(s) |
|---|---|---|
| `append_and_inject_event` | ✓ | Behaviors 1, 2 + Error Variant 1 |
| `append_and_inject_timer_event` | ✓ | Behaviors 3, 4 + Error Variant 2 |

**Finding:** Functions are covered. **LETHAL: Error Variant 3 uses undefined error variant.**

### Axis 2 — Assertion Sharpness
| Scenario | Assertion Type | Verdict |
|---|---|---|
| Behavior 1 unit | `pending_activity_calls` does NOT contain aid | Concrete ✓ |
| Behavior 1 integration | `reply receives Err(WtfError::EventInjectionFailed)` | Concrete ✓ |
| Behavior 2 unit | `pending_activity_calls` contains aid with reply port | Concrete ✓ |
| Behavior 3 integration | `pending_timer_calls` does NOT contain timer_id | Concrete ✓ |
| Behavior 4 unit | `pending_timer_calls` contains timer_id with reply port | Concrete ✓ |
| Behavior 6 E2E | `total_events_applied for second activity > 0` | **Vague — MAJOR** |
| Error Variant 1 | `reply receives Err(WtfError::EventInjectionFailed)` | Concrete ✓ |
| Error Variant 2 | `reply receives Err(WtfError::EventInjectionFailed)` | Concrete ✓ |
| Error Variant 3 | `reply receives Err(WtfError::nats_publish("..."))` | **Mismatch with contract — LETHAL** |

### Axis 3 — Trophy Allocation
- Claimed: 3 unit / 4 integration / 1 e2e = 8 total
- **Ambiguity: Only 2 unit test descriptions found. 3rd unit test unclear.**
- Density: 8 tests / 2 functions = 4x — **below 5x target from plan itself**

### Axis 4 — Boundary Completeness
| Boundary | Covered? |
|---|---|
| Minimum (empty/None) | ✓ event_store=None scenario |
| inject_event success | ✓ |
| inject_event failure | ✓ |
| activity_id Some/None | ✓ |
| Max pending capacity | ✗ Not specified |

### Axis 5 — Mutation Survivability
| Mutation | Test That Catches It | Gap? |
|---|---|---|
| Remove `remove(&aid)` | Behavior 1 unit test | ✓ |
| Remove `remove(&timer_id)` | Behavior 3 integration test | ✓ |
| Remove `reply.send(Err(...))` — activity | Integration test only | **MAJOR: unit test gap** |
| Remove `reply.send(Err(...))` — timer | Error Variant 2 | ✓ |
| Swap Ok/Err branches | Multiple tests | ✓ |
| Remove entire cleanup block | Behavior 6 E2E | ✓ |

### Axis 6 — Holzmann Rules
| Rule | Applied? | Finding |
|---|---|---|
| Rule 2 (Bound loops) | ✓ No loops in test bodies | — |
| Rule 5 (State assumptions) | Partial | **Open question 2 ordering not verified** |
| Rule 8 (Surface side effects) | Partial | **Helper function naming ambiguous** |

---

## MANDATE

The following must exist before resubmission:

1. **[LETHAL]** `test-plan.md:145`: Change `Err(WtfError::nats_publish("Event store missing"))` to `Err(WtfError::EventInjectionFailed)` to match contract, OR add `WtfError::nats_publish` to the contract's Error Taxonomy.

2. **[MAJOR]** `test-plan.md:49-55`: Add explicit reply assertion to Behavior 1 unit test:
   ```
   Then: reply port receives Err(WtfError::EventInjectionFailed)
   And: pending_activity_calls does NOT contain aid
   ```

3. **[MAJOR]** `test-plan.md:8`: Clarify the 3rd unit test — either add an explicit unit test scenario description for Behavior 5 (timer success) or correct the trophy allocation count.

4. **[MINOR]** `test-plan.md`: Add a test scenario that explicitly verifies the ORDER of `remove_pending` before `send reply error` to address Open Question 2.

5. **[MINOR]** `test-plan.md:64,94`: Clarify whether tests call `append_and_inject_event` directly or through `handle_dispatch`/`handle_sleep` wrappers.

After fixes, resubmit for full Mode 1 re-review from the beginning.
