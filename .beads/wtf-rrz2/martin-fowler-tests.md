# Martin Fowler Test Plan: wtf-linter AST Walker + Diagnostic Infrastructure

## Happy Path Tests
- `test_returns_empty_diagnostics_when_no_workflow_functions`
- `test_returns_empty_diagnostics_when_workflow_has_no_violations`
- `test_identifies_workflow_function_by_attribute`
- `test_identifies_workflow_function_by_suffix`
- `test_collects_diagnostics_from_all_visitors`

## Error Path Tests
- `test_returns_parse_failure_for_invalid_rust_code`
- `test_returns_error_for_totally_invalid_source`

## Edge Case Tests
- `test_handles_empty_source_file`
- `test_handles_source_with_only_comments`
- `test_handles_multiple_workflow_functions`
- `test_handles_workflow_function_with_empty_body`

## Contract Verification Tests
- `test_postcondition_q1_all_diagnostics_collected`
- `test_postcondition_q2_has_errors_reflects_diagnostic_severity`
- `test_postcondition_q3_each_workflow_visited_once`

## Contract Violation Tests
- `test_p2_violation_invalid_rust_code_returns_parse_failure`
  Given: `lint_workflow_source("not rust code @#$")`
  When: function is called
  Then: returns `Err(Error::ParseFailure)`

## Integration Test Scenario
### Scenario: Full lint workflow with violations
Given: Rust source file containing:
```rust
#[workflow]
fn my_workflow() {
    let x = chrono::Utc::now(); // L001 violation
    let y = rand::random::<u64>(); // L002 violation
}
```
When: `lint_workflow_source(source)` is called
Then:
- Returns `Ok(LintResult { has_errors: true, diagnostics: [...] })`
- Contains exactly 2 diagnostics
- First diagnostic has code "WTF-L001"
- Second diagnostic has code "WTF-L002"
