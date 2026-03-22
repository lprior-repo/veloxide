# Defects Report: WTF-L001 Non-Deterministic Time Detection

**Date**: 2026-03-22
**Reviewer**: Black Hat Reviewer
**Status**: REJECTED

---

## Summary

The Red Queen found 3 bugs. Upon deep inspection:

- **2 defects are in the CONTRACT** (specification gaps)
- **1 defect is in the IMPLEMENTATION SUMMARY** (false documentation claim)
- **0 defects are in the IMPLEMENTATION CODE** (follows contract correctly)

The implementation code is competent. The contract is incomplete. The documentation is misleading.

---

## Defect 1: Macros Bypass Linter

**Bug**: `vec![chrono::Utc::now()]` produces 0 diagnostics.

**Root Cause**: `syn::visit::Visit` does NOT expand macros. `vec![...]` is parsed as `Expr::Macro` and the visitor does not traverse into macro arguments.

**Location**: CONTRACT GAP (contract.md)

**Contract Issue**: Contract line 68 says "every `.now()` call" but does not specify whether macro contents should be linted. The contract should explicitly address macro behavior.

**Fix Required**: 
- Option A: Contract specifies "macros are out of scope" (limitation of syn-based parsing)
- Option B: Contract specifies "must detect `.now()` in macro arguments" (requires macro expansion via `quote::expand` or similar)

**Severity**: Medium - this is a fundamental limitation of syn-based parsing.

---

## Defect 2: Deeply Nested Paths Not Detected

**Bug**: `some::deep::chrono::Utc::now()` produces 0 diagnostics.

**Root Cause**: `is_time_now_call` at line 58 of l001_time.rs checks:
```rust
if segments.len() == 3 && segments[0].ident == "chrono" && segments[1].ident == "Utc"
```
This is an **exact 3-segment match**. `some::deep::chrono::Utc::now()` has **5 segments**.

**Location**: CONTRACT DEFECT (contract.md)

**Contract Issue**: Contract line 47 specifies `chrono::Utc::now()` as exactly 3 segments. This does not account for path prefixes (e.g., `use some::deep::chrono::Utc; Utc::now()` or aliased imports).

**Fix Required**:
Change contract from "exact 3-segment match" to "suffix match: last 3 segments must be `chrono::Utc::now`".

**Implementation Change**:
```rust
// Instead of:
if segments.len() == 3 && segments[0].ident == "chrono" && segments[1].ident == "Utc"

// Should be:
if segments.len() >= 3 
    && segments[segments.len() - 3].ident == "chrono"
    && segments[segments.len() - 2].ident == "Utc"
    && segments[segments.len() - 1].ident == "now"
```

**Severity**: High - creates false negatives with common Rust import patterns.

---

## Defect 3: Bare `Utc::now()` Not Detected

**Bug**: `Utc::now()` (without `chrono::` prefix) produces 0 diagnostics.

**Root Cause**: `is_time_now_call` only handles 3+ segment paths. `Utc::now()` is a 2-segment path (`[Utc, now]`).

**Location**: CONTRACT DEFECT + IMPLEMENTATION SUMMARY ERROR

**Contract Issue**: Contract line 47 specifies `chrono::Utc::now()` but NOT bare `Utc::now()`. These are syntactically distinct in Rust.

**Implementation Summary Error**: implementation.md lines 96-103 falsely claim:
```
Utc.now() → segments: [chrono, Utc]  // FALSE
```
`Utc::now()` in path-style is 2 segments: `[Utc, now]`. It is NOT handled by `is_time_now_call`.

**Fix Required**:
1. CONTRACT: Add `Utc::now()` and `Local::now()` (without `chrono::` prefix) to flagged patterns
2. IMPLEMENTATION: Handle 2-segment paths ending in `now` with appropriate receivers
3. IMPLEMENTATION SUMMARY: Fix the false claim at lines 96-103

**Severity**: High - bare `Utc::now()` is a common import alias pattern.

---

## Contract Completeness Issues

The contract is missing explicit coverage for:

| Missing Case | Example | Impact |
|--------------|---------|--------|
| Macro contents | `vec![chrono::Utc::now()]` | False negative |
| Path prefixes | `some::deep::chrono::Utc::now()` | False negative |
| Bare imports | `Utc::now()` | False negative |

---

## Functional-Rust Assessment: PASS (Implementation Code)

| Principle | Status |
|-----------|--------|
| Zero unwrap/panic in core | ✅ PASS |
| thiserror for errors | ✅ PASS |
| Data→Calc→Actions separation | ✅ PASS |
| Parse at boundary, don't validate | ✅ PASS |
| Types as documentation | ✅ PASS |

The **implementation code** follows functional-rust principles correctly. The issues are in the **contract** (incomplete specification) and **implementation summary** (misleading documentation).

---

## Verdict

**STATUS**: REJECTED

**Reason**: The contract is incomplete and the implementation summary contains false claims. The implementation code is correct but cannot be approved because:

1. The contract does not specify behavior for macros, deeply nested paths, or bare `Utc::now()`
2. The implementation summary falsely claims coverage that doesn't exist
3. A developer reading the contract + implementation summary would have wrong expectations

**Required Actions**:
1. Update contract.md to explicitly specify macro behavior, suffix matching for paths, and bare `Utc::now()` handling
2. Fix implementation.md lines 96-103 to remove false claims
3. Implement suffix matching in `is_time_now_call`
4. Add tests for deeply nested paths and bare `Utc::now()`
