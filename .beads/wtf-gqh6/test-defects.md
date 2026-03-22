# Test Defects: wtf-gqh6 TimeTravelScrubber - STATE 2 RE-REVIEW

## Review Summary
**Reviewer**: Test Reviewer (State 2 - RE-REVIEW)  
**Date**: 2026-03-22  
**Status**: REJECTED

---

## DEFECT-001: Test Names Now Follow BDD Format ✓ FIXED
**Severity**: CRITICAL  
**Location**: Both workspace calc.rs and main crates/monitor_mode.rs

**Status**: DEFECT-001 is FIXED.

**Evidence**:
- Workspace tests (calc.rs lines 61-97): `given_valid_seq_when_validating_replay_seq_then_ok`, `given_seq_exceeds_max_when_validating_replay_seq_then_error`
- Main crate tests (monitor_mode.rs lines 46-215): `given_valid_seq_within_bounds_when_validating_then_ok`, `given_seq_exceeds_max_when_validating_then_error`

All test names now follow `given_<context>_when_<action>_then_<outcome>` pattern.

---

## DEFECT-002: Test Count Mismatch - CRITICAL GAP
**Severity**: CRITICAL  
**Framework**: Testing Trophy (Real Execution)

**Status**: REJECT - DEFECT NOT FIXED

**Problem**: martin-fowler-tests.md claims **52 tests** but actual codebase has **~29 tests**.

**Actual Test Count**:
| Location | File | Test Count |
|----------|------|------------|
| wtf-gqh6-workspace | calc.rs | 6 |
| crates/wtf-frontend | monitor_mode.rs | 23 |
| **TOTAL** | | **~29** |

**Claimed in martin-fowler-tests.md**:
| Category | Claimed | Actual |
|----------|---------|--------|
| Happy Path | 6 | ~6 |
| Error Path | 4 | ~4 |
| Edge Case | 7 | ~2 |
| Contract Verification | 8 | ~3 |
| SSE Integration | 3 | 0 |
| Invariant | 10 | ~3 |
| Concurrency | 5 | 0 |
| Property-Based | 4 | 0 |
| Violation | 5 | ~3 |
| **TOTAL** | **52** | **~29** |

**Critical Issue**: The martin-fowler-tests.md describes **52 specific test functions** with **exact names** like:
- `test_given_valid_client_and_seq_50_when_replay_to_50_then_returns_scrubber_state_with_seq_50`

But actual code has **different test names** like:
- `given_valid_seq_within_bounds_when_validating_then_ok`

**The test plan describes fantasy tests that don't exist in the codebase.**

---

## DEFECT-003: Missing SSE Integration Tests - NOT FIXED
**Severity**: CRITICAL  
**Framework**: Testing Trophy (Real Execution)

**Status**: REJECT - DEFECT NOT FIXED

**Problem**: 
- Contract.md specifies [Q4] "Historical mode disables SSE subscription"
- martin-fowler-tests.md specifies 3 SSE integration tests (lines 342-387)
- **ZERO SSE integration tests exist in actual code**

**Evidence**:
- Grep for `sse` in test functions: no matches
- No `#[test]` functions test `is_sse_enabled()` or `set_sse_enabled()`

---

## DEFECT-004: Scenarios Linked but Tests Don't Exist - PARTIALLY FIXED
**Severity**: CRITICAL  
**Framework**: ATDD (Dave Farley)

**Status**: PARTIALLY FIXED - Scenarios now reference actual test function names

**BUT**: The referenced test function names in scenarios (lines 735-780) **DO NOT MATCH** actual test function names in code.

Example:
- Scenario 1 links to: `test_given_valid_client_and_seq_50_when_replay_to_50_then_returns_scrubber_state_with_seq_50`
- Actual test: `given_valid_seq_within_bounds_when_validating_then_ok`

---

## DEFECT-005: Invariant Tests Now Specify Values ✓ FIXED
**Severity**: MAJOR  
**Framework**: Combinatorial Exhaustiveness

**Status**: DEFECT-005 is FIXED.

**Evidence**: Lines 393-438 now enumerate specific boundary values: max_seq ∈ {0, 1, 100, u64::MAX}

---

## DEFECT-006: Missing Concurrency/Reentrancy Tests - NOT FIXED
**Severity**: MAJOR  
**Framework**: TDD (Kent Beck)

**Status**: REJECT - DEFECT NOT FIXED

**Problem**: 
- martin-fowler-tests.md lines 520-596 specify 5 concurrency tests
- **ZERO concurrency tests exist in actual code**
- No tests for `ReplayInProgress` error condition

---

## DEFECT-007: Missing Property-Based Tests - NOT FIXED
**Severity**: MEDIUM  
**Framework**: Combinatorial Permutations

**Status**: REJECT - DEFECT NOT FIXED

**Problem**:
- martin-fowler-tests.md lines 600-658 specify 4 property-based tests using `proptest`
- **NO proptest dependency exists**
- **NO property-based tests exist**

---

## Summary Table

| Defect | Severity | Framework | Status |
|--------|----------|-----------|--------|
| DEFECT-001: BDD naming | CRITICAL | Dan North BDD | ✅ FIXED |
| DEFECT-002: 52 tests exist | CRITICAL | Testing Trophy | ❌ REJECT - 29/52 |
| DEFECT-003: SSE integration | CRITICAL | Testing Trophy | ❌ REJECT - 0 tests |
| DEFECT-004: Scenarios executable | CRITICAL | ATDD | ⚠️ PARTIAL |
| DEFECT-005: Boundary values | MAJOR | Combinatorial | ✅ FIXED |
| DEFECT-006: Concurrency tests | MAJOR | TDD | ❌ REJECT - 0 tests |
| DEFECT-007: Property-based | MEDIUM | Combinatorial | ❌ REJECT - 0 tests |

---

## Overall Assessment: REJECTED

**PASSED**: DEFECT-001 (BDD naming), DEFECT-005 (boundary values)  
**FAILED**: DEFECT-002 (29/52 tests), DEFECT-003 (0 SSE tests), DEFECT-006 (0 concurrency), DEFECT-007 (0 property-based)

**Critical Blockers**:
1. **DEFECT-002**: The test plan describes 52 tests but only ~29 exist. The named tests in martin-fowler-tests.md DO NOT match actual test function names in code.
2. **DEFECT-003**: Zero SSE integration tests exist, but contract requires Q4.
3. **DEFECT-006**: Zero concurrency/reentrancy tests exist, but ScrubberError::ReplayInProgress is not tested.
4. **DEFECT-007**: Zero property-based tests exist.

**Required Actions for Approval**:
1. Implement ALL 52 tests specified in martin-fowler-tests.md with EXACT function names
2. Add SSE integration tests (3 tests for Q4)
3. Add concurrency/reentrancy tests (5 tests for ReplayInProgress)
4. Add property-based tests using proptest (4 tests for seq bounds)
5. Ensure actual test function names MATCH the names referenced in Given-When-Then scenarios

---

*Generated by: Test Reviewer (State 2 - RE-REVIEW)*  
*Previous defects preserved for history*
