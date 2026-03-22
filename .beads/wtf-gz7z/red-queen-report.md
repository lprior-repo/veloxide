# Red Queen Adversarial Test Report: WTF-L001

**Date:** 2026-03-22
**Iteration:** Re-run after bug fixes
**Status:** ALL TESTS PASS ✅

---

## Previous Bugs → Fixed Status

| Bug | Fix Applied | Status |
|-----|-------------|--------|
| Macros bypass linter | Documented as known limitation in contract.md | ✅ Verified |
| Deeply nested paths | Suffix matching implemented in `is_time_now_call()` | ✅ Verified |
| Bare `Utc::now()` not detected | 2-segment path detection added | ✅ Verified |

---

## Adversarial Test Results

### Test Suite: Existing Unit Tests
**Command:** `cargo test -p wtf-linter -- l001_time`
**Result:** 16 tests PASSED, 0 FAILED

| Test | Input | Expected | Actual | Status |
|------|-------|----------|--------|--------|
| `test_emits_diagnostic_when_chrono_utc_now_found` | `chrono::Utc::now()` | Flag | Flag | ✅ |
| `test_emits_diagnostic_when_chrono_local_now_found` | `chrono::Local::now()` | Flag | Flag | ✅ |
| `test_emits_diagnostic_when_system_time_now_found` | `std::time::SystemTime::now()` | Flag | Flag | ✅ |
| `test_emits_diagnostic_when_instant_now_found` | `std::time::Instant::now()` | Flag | Flag | ✅ |
| `test_emits_diagnostic_when_tokio_instant_now_found` | `tokio::time::Instant::now()` | Flag | Flag | ✅ |
| `test_emits_diagnostic_for_bare_utc_now` | `Utc::now()` | Flag | Flag | ✅ |
| `test_emits_diagnostic_for_bare_local_now` | `Local::now()` | Flag | Flag | ✅ |
| `test_emits_diagnostic_for_deep_chrono_path` | `some::deep::chrono::Utc::now()` | Flag | Flag | ✅ |
| `test_emits_no_diagnostic_when_ctx_now_found` | `ctx.now()` | Clean | Clean | ✅ |
| `test_macro_does_not_expand_vec_chrono_utc_now` | `vec![chrono::Utc::now()]` | Clean | Clean | ✅ |
| `test_returns_parse_error_for_invalid_rust` | `async fn workflow { // missing parens` | Error | Error | ✅ |

---

## New Adversarial Edge Cases (Programmatically Verified)

### 1. Deep Path with Multiple Prefixes
**Input:** `some::deep::chrono::Utc::now()`
**Segments:** 5 (some, deep, chrono, Utc, now)
**Detection:** Suffix matching on last 3 segments
**Expected:** Flagged
**Actual:** Flagged ✅
**Analysis:** `segments[len-3] = "chrono"`, `segments[len-2] = "Utc"` → suffix match

### 2. Bare 2-Segment Paths
**Input:** `Utc::now()`, `Local::now()`, `Instant::now()`, `SystemTime::now()`
**Segments:** 2
**Detection:** Direct 2-segment check
**Expected:** Flagged
**Actual:** Flagged ✅

### 3. Alternative chrono Prefix
**Input:** `other::chrono::Utc::now()`
**Segments:** 4 (other, chrono, Utc, now)
**Detection:** Suffix matching on last 3 segments
**Expected:** Flagged
**Actual:** Flagged ✅
**Analysis:** `segments[len-3] = "chrono"`, `segments[len-2] = "Utc"` → suffix match

### 4. Tokio with Extra Prefixes
**Input:** `foo::bar::tokio::time::Instant::now()`
**Segments:** 6
**Detection:** Suffix matching on last 4 segments
**Expected:** Flagged
**Actual:** Flagged ✅
**Analysis:** `segments[len-4] = "tokio"`, `segments[len-3] = "time"`, `segments[len-2] = "Instant"` → suffix match

### 5. ctx.now() Safe Pattern
**Input:** `ctx.now()`
**Detection:** Method call expression (not path call)
**Expected:** NOT flagged
**Actual:** NOT flagged ✅
**Analysis:** `is_time_now_method()` only matches `std::time::*`, `chrono::*`, `tokio::time` receivers — `ctx` is not in the list

### 6. Zero-Segment Parse Error
**Input:** `.now()` (invalid Rust syntax)
**Detection:** syn parse failure
**Expected:** Parse error returned
**Actual:** Parse error returned ✅
**Analysis:** `syn::parse_file()` returns `Err` → propagates as `LintError::ParseError`

### 7. Bare Instant::now()
**Input:** `Instant::now()`
**Segments:** 2
**Detection:** Direct 2-segment check
**Expected:** Flagged
**Actual:** Flagged ✅

---

## Suffix Matching Coverage Analysis

| Pattern | Min Segments | Suffix Start | Detection |
|---------|--------------|--------------|-----------|
| `Utc::now`, `Local::now` | 2 | N/A (full match) | ✅ |
| `chrono::Utc::now` | 3 | -3 | ✅ |
| `chrono::Local::now` | 3 | -3 | ✅ |
| `std::time::SystemTime::now` | 4 | -4 | ✅ |
| `std::time::Instant::now` | 4 | -4 | ✅ |
| `tokio::time::Instant::now` | 4 | -4 | ✅ |
| `*::chrono::Utc::now` | 3+ | -3 | ✅ |
| `*::chrono::Local::now` | 3+ | -3 | ✅ |
| `*::tokio::time::Instant::now` | 4+ | -4 | ✅ |

---

## Known Limitation: Macros

**Status:** Documented in contract.md section "Macro Scope"

The linter does NOT expand macros. Code inside macro invocations is not linted.

**Example:** `vec![chrono::Utc::now()]` is NOT flagged

**Rationale:** Macro expansion requires full compilation. Linter operates on raw syntax trees only. Contract explicitly documents this limitation.

---

## Final Verdict

**All adversarial tests PASSED ✅**

The WTF-L001 linter correctly handles:
- ✅ Deeply nested paths (suffix matching)
- ✅ Bare 2-segment paths (Utc, Local, Instant, SystemTime)
- ✅ Standard library paths (std::time::*)
- ✅ Third-party paths (chrono::*, tokio::*)
- ✅ Safe patterns (ctx.now(), non-.now() calls)
- ✅ Parse errors (graceful handling)

**No new bugs found.**

---

## Full Regression Suite

| Crate/Test File | Tests Passed |
|-----------------|--------------|
| wtf-linter unit tests | 50 |
| integration_test.rs | 8 |
| l002_random_test.rs | 6 |
| l004_test.rs | 18 |
| l005_test.rs | 10 |
| l006_test.rs | 14 |
| **Total** | **106 tests PASSED** |

**No regressions detected.**
