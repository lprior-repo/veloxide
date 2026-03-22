# Implementation: wtf-linter WTF-L006 std::thread::spawn + L006b

bead_id: wtf-wqc7
bead_title: wtf-linter: WTF-L006 std::thread::spawn in workflow function
phase: implementation
updated_at: 2026-03-21T00:00:00Z

## Changes Made

### 1. diagnostic.rs
- Added `LintCode::L006b` variant for `std::thread::sleep` detection
- Updated `as_str()` to return "WTF-L006b"

### 2. l006.rs
- Extended `L006Visitor::visit_expr()` to check for `std::thread::sleep` calls
- Added `is_std_thread_sleep_path()` helper function
- Emits `LintCode::L006b` with message suggesting `ctx.sleep()` usage

### 3. lib.rs
- Added `LintResult` struct with `diagnostics` and `has_errors` fields
- Added `lint_workflow_source()` function that:
  - Runs all rule visitors (L005, L006)
  - Deduplicates diagnostics by code
  - Returns `LintResult`
- Re-exports `LintError` for caller use

### 4. integration_test.rs
- Created integration test with all violation types (L005, L006, L006b)
- Tests: has_errors flag, ctx.sleep non-false-positive, thread functions outside workflow

## Files Modified
- `crates/wtf-linter/src/diagnostic.rs`
- `crates/wtf-linter/src/l006.rs`
- `crates/wtf-linter/src/lib.rs`
- `crates/wtf-linter/tests/integration_test.rs` (new)

## Verification
- No panics/unwrap/expect used
- All clippy lints pass
- All tests pass
