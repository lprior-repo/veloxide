# Test Defects Report: WTF-L001

## Metadata
- **bead_id**: wtf-gz7z
- **review_phase**: test-plan-review (STATE 2 re-review round 2)
- **reviewer**: test-reviewer
- **status**: REJECTED — FLAWED
- **updated_at**: 2026-03-22T17:00:00Z

---

## Defect Summary

| # | Severity | Category | Description |
|---|----------|----------|-------------|
| 1 | LOW | Test Assertion Contradiction | Scenario 16 title says "NOT flagged" but Then expects diagnostic |

---

## Defect 1: Scenario 16 Title/Assertion Contradiction

**Location**: martin-fowler-tests.md, Scenario 16 (lines 270-285)

**Issue**: The test title says "NOT flagged (implementation gap)" but the Then assertion expects exactly 1 diagnostic.

**Current title**:
```
### Scenario 16: tokio::time::Instant as method call receiver — NOT flagged (implementation gap)
```

**Current Then assertion**:
```
Then the result is `Ok(vec![diagnostic])` with exactly 1 diagnostic
```

**Problem**: 
1. Title says "NOT flagged" suggesting no diagnostic expected
2. But Then clause expects exactly 1 diagnostic
3. This is an internal contradiction in the test specification

**Contract alignment issue**:
- The contract lists `tokio::time::Instant::now()` (path-style) and `Instant::now()` (method-call-style) as flagged patterns
- The contract does NOT explicitly list `tokio::time::Instant.now()` (fully-qualified receiver + method-call-style) as a required pattern
- If the implementation correctly flags `Instant::now()` but NOT `tokio::time::Instant.now()`, the test would fail — but the implementation would be correct per contract

**Why this is a defect**:
1. Internal contradiction between title and assertion
2. Tests behavior beyond contract requirements
3. If implementation doesn't flag `tokio::time::Instant.now()`, test fails but contract is satisfied

**Fix required**: Either:
- Remove Scenario 16 entirely (it tests beyond-contract behavior)
- Or rename title to remove "NOT flagged" and clarify this tests implementation behavior beyond contract

---

## Verification of Previous Defects

| Defect | Previous Status | Current Status |
|--------|----------------|----------------|
| 1. Scenario 1 description bug | REJECTED | ✅ FIXED |
| 2. Scenario 16 code/description mismatch | REJECTED | ⚠️ CODE FIXED but contradiction remains |
| 3. Missing chrono method call tests | REJECTED | ✅ FIXED |
| 4. Edge cases not documented | REJECTED | ✅ FIXED |

---

## Verdict

**STATUS**: REJECTED — FLAWED

**Remaining Issue**: Scenario 16 has a title/assertion contradiction. The title says "NOT flagged" but the assertion expects a diagnostic.

**Progress**: All 4 previous defects addressed in code. The remaining issue is a documentation contradiction in Scenario 16.

**Required Action**:
- Fix Scenario 16 title to match assertion, OR remove the scenario as it tests beyond-contract behavior

---

## Files Affected

- `/home/lewis/src/wtf-engine/.beads/wtf-gz7z/martin-fowler-tests.md`
