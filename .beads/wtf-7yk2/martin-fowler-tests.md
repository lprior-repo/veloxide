# Martin Fowler Test Plan: WTF-L003 direct-async-io

## Test Suite Overview
Test the `DirectAsyncIoVisitor` implementation for detecting direct async I/O calls in workflow functions.

## Happy Path Tests
- `test_emits_no_diagnostic_for_code_without_async_io`
  - Given: Valid Rust source with no `reqwest` or `sqlx` calls
  - When: `check_direct_async_io` is called
  - Then: Returns an empty Vec<Diagnostic>

- `test_emits_no_diagnostic_for_proper_ctx_activity_usage`
  - Given: Rust source that wraps I/O in `ctx.activity("fetch", input).await?`
  - When: `check_direct_async_io` is called
  - Then: Returns an empty Vec<Diagnostic>

## Error Path Tests
- `test_emits_diagnostic_for_reqwest_get_call`
  - Given: Rust source containing `reqwest::get("https://example.com").await?`
  - When: `check_direct_async_io` is called
  - Then: Returns a Diagnostic with code L003, severity Error, and appropriate message

- `test_emits_diagnostic_for_sqlx_query_fetch_one`
  - Given: Rust source containing `sqlx::query("SELECT * FROM users").fetch_one(&pool).await?`
  - When: `check_direct_async_io` is called
  - Then: Returns a Diagnostic with code L003, severity Error, and appropriate message

- `test_emits_diagnostic_for_multiple_violations`
  - Given: Rust source containing multiple direct async I/O calls
  - When: `check_direct_async_io` is called
  - Then: Returns a Diagnostic for each violation

- `test_returns_parse_error_for_invalid_rust`
  - Given: Invalid Rust source that cannot be parsed
  - When: `check_direct_async_io` is called
  - Then: Returns `Err(LintError::ParseError(...))`

## Edge Case Tests
- `test_handles_reqwest_post_method`
  - Given: Rust source containing `reqwest::Client::new().post("https://example.com").await?`
  - When: `check_direct_async_io` is called
  - Then: Returns a Diagnostic with code L003 (all reqwest HTTP methods are forbidden)

- `test_handles_sqlx_fetch_optional`
  - Given: Rust source containing `sqlx::query("...").fetch_optional(&pool).await?`
  - When: `check_direct_async_io` is called
  - Then: Returns a Diagnostic with code L003

- `test_handles_nested_expression_in_macro`
  - Given: Rust source with direct async I/O inside a macro expansion
  - When: `check_direct_async_io` is called
  - Then: The visitor walks into the macro and detects the violation

- `test_handles_multiple_reqwest_calls_in_same_function`
  - Given: Rust source with multiple `reqwest::get()` calls
  - When: `check_direct_async_io` is called
  - Then: Returns one Diagnostic per call

## Contract Verification Tests
- `test_visitor_initializes_with_empty_diagnostics`
  - Given: A new `DirectAsyncIoVisitor`
  - When: Created via `new()`
  - Then: `into_diagnostics()` returns an empty vector

- `test_diagnostic_contains_correct_lint_code`
  - Given: Rust source with a direct async I/O violation
  - When: `check_direct_async_io` is called
  - Then: The returned Diagnostic's `code` field is `LintCode::L003`

- `test_diagnostic_contains_error_severity`
  - Given: Rust source with a direct async I/O violation
  - When: `check_direct_async_io` is called
  - Then: The returned Diagnostic's `severity` field is `Severity::Error`

- `test_diagnostic_contains_suggestion`
  - Given: Rust source with a direct async I/O violation
  - When: `check_direct_async_io` is called
  - Then: The returned Diagnostic's `suggestion` field is `Some("wrap in ctx.activity(...)")`

- `test_diagnostic_contains_span`
  - Given: Rust source with a direct async I/O violation
  - When: `check_direct_async_io` is called
  - Then: The returned Diagnostic's `span` field is `Some((start, end))` where start and end are byte positions

## Given-When-Then Scenarios

### Scenario 1: Detects reqwest::get violation
Given: A workflow function that makes a direct HTTP call:
```rust
async fn fetch_user_data(ctx: &WorkflowCtx, user_id: String) -> Result<String> {
    let resp = reqwest::get("https://api.example.com/users/".to_string() + &user_id).await?;
    Ok(resp.text().await?)
}
```
When: The linter analyzes this source code
Then: A diagnostic is emitted with code WTF-L003 and message "direct async I/O in workflow function"

### Scenario 2: Detects sqlx query violation
Given: A workflow function that queries the database directly:
```rust
async fn get_user(ctx: &WorkflowCtx, user_id: i64) -> Result<User> {
    let row = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&pool)
        .await?;
    Ok(row)
}
```
When: The linter analyzes this source code
Then: A diagnostic is emitted with code WTF-L003 and message "direct async I/O in workflow function"

### Scenario 3: Does not flag ctx.activity wrappers
Given: A workflow function that properly uses ctx.activity:
```rust
async fn fetch_data(ctx: &WorkflowCtx, input: Input) -> Result<Output> {
    let result = ctx.activity("fetch", input).await?;
    Ok(result)
}
```
When: The linter analyzes this source code
Then: No diagnostic is emitted

### Scenario 4: Parse error handling
Given: Invalid Rust source code that cannot be parsed
When: The linter attempts to analyze the source
Then: Returns `Err(LintError::ParseError(...))` with details about the parse failure
