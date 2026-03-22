# Implementation Summary: WTF-L002 (non-deterministic-random)

## bead_id: wtf-0nzz
## bead_title: implement wtf-linter WTF-L002: non-deterministic-random
## phase: implementation
## updated_at: 2026-03-21T19:30:00Z

## Changes Made

### crates/wtf-linter/src/rules.rs
- Added `check_random_in_workflow(file: &File) -> Vec<Diagnostic>` public function
- Added `RandomDetector` struct implementing `Visit` trait from syn
- Detects `uuid::Uuid::new_v4()` calls (ExprCall with path containing "Uuid" and "new_v4")
- Detects `rand::random()` and `rand::random::<T>()` calls (ExprCall with path containing "rand" and "random")
- Emits `Diagnostic` with `LintCode::L002` and severity `Error`
- Added `#[must_use]` attribute to public function
- All clippy warnings resolved (using `is_some_and` instead of `map_or`)

### crates/wtf-linter/src/lib.rs
- Added `pub use rules::check_random_in_workflow` export

### crates/wtf-linter/src/visitor.rs
- Updated to re-export `check_random_in_workflow` from rules module

### crates/wtf-linter/tests/l002_random_test.rs (NEW)
- 6 unit tests covering:
  - `uuid::Uuid::new_v4()` detection
  - `rand::random()` detection  
  - `rand::random::<T>()` detection
  - `ctx.random_u64()` NOT flagged (false positive prevention)
  - `uuid::Uuid::nil()` NOT flagged (false positive prevention)
  - Multiple violations in same function

## Architecture

```
check_random_in_workflow(file: &File)
    └── RandomDetector (Visit trait impl)
            └── visit_expr_call() - checks for uuid::new_v4 and rand::random patterns
```

## Detection Logic

- `is_uuid_new_v4_call`: Matches `ExprCall` where callee path contains segments "Uuid" and "new_v4"
- `is_rand_random_call`: Matches `ExprCall` where callee path contains segments "rand" and "random"

## Testing Results

```
running 6 tests
test test_ctx_random_u64_not_flagged ... ok
test test_rand_random_with_type_detected ... ok
test test_rand_random_detected ... ok
test test_uuid_nil_not_flagged ... ok
test test_uuid_new_v4_detected ... ok
test test_multiple_violations ... ok

test result: ok. 6 passed; 0 failed
```

## Clippy Status

- `cargo fmt --check`: PASSED
- `cargo clippy -p wtf-linter`: PASSED (0 warnings)
