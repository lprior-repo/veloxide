bead_id: wtf-sztr
bead_title: wtf-linter: WTF-L004 ctx calls inside non-deterministic closures
phase: implementation
updated_at: 2026-03-21T00:00:00Z

# Implementation Summary: WTF-L004

## Files Created/Modified

### crates/wtf-linter/src/l004.rs (NEW)
- Implements `lint_workflow_code(source: &str) -> Result<Vec<Diagnostic>, LintError>`
- `L004Visitor` struct with full AST traversal
- Detects `ctx.*` calls inside closures passed to: `map`, `for_each`, `fold`, `filter_map`, `and_then`, `flat_map`
- Exports 15 unit tests covering positive, negative, and boundary cases

### crates/wtf-linter/src/lib.rs (MODIFIED)
- Added `pub mod l004`
- Added `pub use l004::lint_workflow_code as lint_workflow_code_l004`
- Renamed l005/l006 exports to `lint_workflow_code_l005/l006` for consistency

## Key Implementation Details

### ctx.* Detection
- `is_ctx_receiver()`: checks if method call receiver is `ctx` path
- `is_ctx_path()`: verifies path segment is exactly `ctx`
- `expr_contains_ctx_call()`: recursively searches expression tree

### Closure Detection
- When `ExprMethodCall` with target method name is found
- Checks if any argument is an `ExprClosure`
- Recursively checks closure body for ctx calls
- Emits single diagnostic per violation

### Severity
- Per `diagnostic.rs`, L004 has `Severity::Warning` (not Error)
- Suggestion: "use ctx.parallel() or sequential iteration for deterministic ctx calls in closures"

## Test Coverage
- 15 unit tests in l004.rs
- All tests pass
- Covers: map, for_each, fold, filter_map, and_then, flat_map
- Negative cases: regular for loops, ctx outside closures, non-target methods
- Edge cases: nested closures, multiple violations, field access ctx
