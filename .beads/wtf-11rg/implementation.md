# Implementation Summary: WTF-L005 tokio-spawn-in-workflow

## Files Changed
- `crates/wtf-linter/src/l005.rs` - New file implementing L005 rule
- `crates/wtf-linter/src/lib.rs` - Added `pub mod l005` and `pub use l005::lint_workflow_code`
- `crates/wtf-linter/tests/l005_test.rs` - Integration tests for L005

## Contract Compliance

| Contract Clause | Implementation Status |
|----------------|----------------------|
| Precondition: Valid Rust source | ✅ `syn::parse_file` returns ParseError |
| Postcondition: All tokio::spawn in workflow fn produce Diagnostic | ✅ Implemented in `visit_expr` |
| Postcondition: No false positives | ✅ `in_workflow_fn` flag ensures only workflow fns checked |
| Postcondition: Span accuracy | ✅ Diagnostic.span populated with byte offset |
| Invariant: No panic on malformed AST | ✅ `LintError::ParseError` for parse failures |
| Invariant: Accurate violation locations | ✅ Span captured from `call.span()` |

## Implementation Details

### Data Structures
- `L005Visitor`: AST visitor with `in_workflow_fn` flag
- `lint_workflow_code`: Entry point function returning `Result<Vec<Diagnostic>, LintError>`

### Detection Logic
1. Parse source with `syn::parse_file`
2. Traverse AST looking for `impl` blocks that implement workflow functions
3. When inside a workflow fn (`execute` with `async`), check all expressions
4. If expression is `tokio::spawn` call, emit `L005` diagnostic

### Key Functions
- `is_workflow_impl`: Checks if impl block has `async fn execute(...)` 
- `is_tokio_spawn_path`: Checks if path is exactly `tokio::spawn`
- `visit_expr`: Main detection logic with `in_workflow_fn` guard
- `visit_block`: Traverses blocks for nested expressions

## Test Coverage

| Test | Status |
|------|--------|
| test_lints_clean_code_with_no_violations | ✅ PASS |
| test_returns_parse_error_for_invalid_rust | ✅ PASS |
| test_handles_empty_source | ✅ PASS |
| test_no_false_positive_outside_workflow | ✅ PASS |
| test_no_false_positive_different_spawn | ✅ PASS |
| test_no_false_positive_qualified_tokio_spawn | ✅ PASS |
| test_violation_tokio_spawn_in_workflow | ✅ PASS |
| test_violation_nested_tokio_spawn | ✅ PASS |
| test_multiple_tokio_spawns | ✅ PASS |
| test_tokio_spawn_in_closure | ✅ PASS |

## Clippy Status
- `cargo clippy --package wtf-linter`: **0 warnings**

## Quality Gates
- Build: ✅ PASS
- Clippy: ✅ PASS  
- Tests: ✅ PASS (10/10)
