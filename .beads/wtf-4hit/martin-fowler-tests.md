# Martin Fowler Test Plan

## Test Naming Convention
`test_<scenario>_<expected>`

## Happy Path Tests
- `test_lints_clean_code_with_no_violations` - source with no spawn calls returns empty diagnostics
- `test_returns_parse_error_for_invalid_rust` - invalid Rust source returns LintError::ParseError
- `test_handles_empty_source` - empty source returns empty diagnostics

## Error Path Tests
- `test_violation_tokio_spawn_in_workflow` - tokio::spawn inside workflow produces L005 diagnostic
- `test_violation_tokio_task_spawn_in_workflow` - tokio::task::spawn inside workflow produces L005 diagnostic
- `test_violation_tokio_task_spawn_blocking_in_workflow` - tokio::task::spawn_blocking inside workflow produces L005 diagnostic
- `test_violation_nested_tokio_spawn` - nested tokio::spawn inside if/else produces L005
- `test_violation_tokio_spawn_in_closure_within_workflow` - tokio::spawn inside for_each closure within workflow produces L005

## Edge Case Tests
- `test_multiple_tokio_spawns` - two tokio::spawn calls produce two diagnostics
- `test_multiple_different_spawn_types` - tokio::spawn + tokio::task::spawn + tokio::task::spawn_blocking produce three diagnostics

## False Positive Prevention Tests
- `test_no_false_positive_outside_workflow` - tokio::spawn in regular async fn produces 0 diagnostics
- `test_no_false_positive_different_spawn` - std::thread::spawn in workflow produces 0 diagnostics (L006)
- `test_no_false_positive_qualified_other_spawn` - some_other::spawn in workflow produces 0 diagnostics

## Given-When-Then Scenarios

### Scenario 1: tokio::task::spawn detection
Given: A workflow impl with async execute containing `tokio::task::spawn(async {})`
When: lint_workflow_code is called
Then: Returns Ok with exactly 1 Diagnostic where code == LintCode::L005

### Scenario 2: tokio::task::spawn_blocking detection
Given: A workflow impl with async execute containing `tokio::task::spawn_blocking(|| {})`
When: lint_workflow_code is called
Then: Returns Ok with exactly 1 Diagnostic where code == LintCode::L005

### Scenario 3: No false positive on non-tokio spawn
Given: A workflow impl with async execute containing `custom::spawn(async {})`
When: lint_workflow_code is called
Then: Returns Ok with 0 diagnostics
