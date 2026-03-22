# Martin Fowler Test Plan: WTF-L002 (non-deterministic-random)

## Test Naming Convention
Tests follow `test_<scenario>_<expected_outcome>` pattern per Dave Farley/Dan North BDD style.

## Happy Path Tests

### test_uuid_new_v4_detected_in_workflow_function
Given: A Rust workflow function containing `uuid::Uuid::new_v4()` call
When: L002 rule analyzes the function
Then: A diagnostic with code L002 is emitted
And: The diagnostic message mentions "non-deterministic random"
And: The diagnostic suggestion mentions `ctx.random_u64()`

### test_rand_random_detected_in_workflow_function
Given: A Rust workflow function containing `rand::random::<u64>()` call
When: L002 rule analyzes the function
Then: A diagnostic with code L002 is emitted
And: The diagnostic message mentions "non-deterministic random"

### test_rand_random_without_type_parameter_detected
Given: A Rust workflow function containing `rand::random()` call (without explicit type)
When: L002 rule analyzes the function
Then: A diagnostic with code L002 is emitted

### test_multiple_violations_in_same_function
Given: A Rust workflow function containing both `uuid::Uuid::new_v4()` and `rand::random::<u32>()`
When: L002 rule analyzes the function
Then: Two diagnostics are emitted (one for each violation)
And: Each diagnostic has code L002

## Error Path Tests

### test_ctx_random_u64_not_flagged
Given: A Rust workflow function containing `ctx.random_u64()` call
When: L002 rule analyzes the function
Then: No diagnostic is emitted (this is the deterministic alternative)

### test_uuid_nil_not_flagged
Given: A Rust workflow function containing `uuid::Uuid::nil()` call
When: L002 rule analyzes the function
Then: No diagnostic is emitted (nil is deterministic)

### test_uuid_parse_not_flagged
Given: A Rust workflow function containing `uuid::Uuid::parse("urn:uuid:...")` call
When: L002 rule analyzes the function
Then: No diagnostic is emitted (parsing is deterministic)

### test_non_workflow_function_not_analyzed
Given: A regular Rust function (not a workflow) containing `uuid::Uuid::new_v4()`
When: L002 rule analyzes the function
Then: No diagnostic is emitted (linter only checks workflow functions)

### test_import_statement_not_flagged
Given: Rust source with `use uuid::Uuid;` at module level (no actual call)
When: L002 rule analyzes the file
Then: No diagnostic is emitted (imports are not violations)

## Edge Case Tests

### test_nested_closure_inside_workflow
Given: A workflow function with a nested closure containing `rand::random()`
When: L002 rule analyzes the function
Then: A diagnostic is emitted (closures inside workflow are still analyzed)

### test_async_block_inside_workflow
Given: A workflow function with an async block containing `uuid::Uuid::new_v4()`
When: L002 rule analyzes the function
Then: A diagnostic is emitted (async blocks are part of the function body)

### test_multiple_workflow_functions
Given: A file with two workflow functions, only one containing a violation
When: L002 rule analyzes the file
Then: Only the violating function produces a diagnostic

### test_span_points_to_exact_method_call
Given: A workflow function with `let id = uuid::Uuid::new_v4();`
When: L002 rule emits a diagnostic
Then: The diagnostic span start column points to the `uuid` in `uuid::Uuid::new_v4()`

### test_violation_in_macro_expansion
Given: A workflow function with a macro that expands to `uuid::Uuid::new_v4()`
When: L002 rule analyzes the function
Then: No diagnostic is emitted (macros are not expanded by syn Visit)

## Contract Verification Tests

### test_precondition_parse_error_returns_error
Given: A file with syntax errors
When: L002 rule attempts to analyze
Then: Returns `Err(LintError::ParseError(_))`

### test_postcondition_diagnostic_code_is_l002
Given: A workflow function with a violation
When: A diagnostic is produced
Then: `diagnostic.code == LintCode::L002`

### test_invariant_no_panic_on_empty_function
Given: An empty workflow function `fn my_workflow(ctx: &Ctx) {}`
When: L002 rule analyzes the function
Then: No panic occurs, no diagnostic is emitted

### test_invariant_error_severity_not_warning
Given: A violation is detected
When: A diagnostic is created
Then: `diagnostic.severity == Severity::Error` (not Warning)

## Given-When-Then Scenarios

### Scenario 1: Detect UUID v4 in workflow
Given: A workflow function at `./src/checkout.rs` containing:
```rust
async fn checkout(ctx: &Ctx) -> Result<OrderId> {
    let order_id = uuid::Uuid::new_v4();
    // ...
}
```
When: L002 rule lints this file
Then: Output contains `error[WTF-L002]: non-deterministic random call in workflow function`
And: Span points to line with `new_v4()`
And: Suggestion mentions `ctx.random_u64()`

### Scenario 2: Detect rand random with generic type
Given: A workflow function containing:
```rust
let session_token: u128 = rand::random();
```
When: L002 rule lints this file
Then: Output contains `error[WTF-L002]`
And: Diagnostic is emitted with correct span

### Scenario 3: No false positive on ctx.random_u64
Given: A workflow function containing:
```rust
let nonce = ctx.random_u64();
```
When: L002 rule lints this file
Then: No diagnostic is emitted

---

## Diagnostic Format Specification

### Expected Output Format
```
error[WTF-L002]: non-deterministic random call in workflow function
  --> src/workflow.rs:15:5
   |
15 |     let id = uuid::Uuid::new_v4();
   |               ^^^^^^^^^^^^^^^^^ use `ctx.random_u64()` instead
   |
   = note: non-deterministic values produce different results on replay
```

### JSON Output Format
```json
[
  {
    "code": "WTF-L002",
    "severity": "error",
    "message": "non-deterministic random call in workflow function",
    "suggestion": "use `ctx.random_u64()` instead",
    "span": {"start": 142, "end": 165},
    "file": "src/workflow.rs"
  }
]
```

---

## Implementation Test Cases

### test_syn_visit_expr_method_call_detection
Verifies that the visitor correctly traverses ExprMethodCall nodes and matches the method name.

### test_syn_visit_expr_path_call_detection
Verifies that the visitor correctly traverses ExprCall nodes where the callee is a Path containing `Uuid::new_v4` or `rand::random`.

### test_no_false_positives_on_deterministic_variants
White-box test verifying that nil(), parse(), and ctx.random_u64() do not trigger diagnostics.
