# Martin Fowler Test Plan: WTF-L001 Non-Deterministic Time Detection

## Metadata
- **bead_id**: wtf-gz7z
- **lint_code**: WTF-L001
- **phase**: contract-synthesis
- **updated_at**: 2026-03-22T00:00:00Z
- **Methodology**: Dan North BDD / Martin Fowler Given-When-Then

---

## Test Suite Overview

Tests follow the Testing Trophy model (Kent C. Dodds):
- Heavy unit testing of the visitor/linter logic
- Integration-level tests for parse + lint pipeline
- Coverage of happy path, error path, and edge cases

---

## Feature: Detecting Non-Deterministic Time Calls

### Story
As a workflow developer, I want the linter to detect uses of non-deterministic time functions (like `SystemTime::now()`) so that I can replace them with the deterministic `ctx.now()` method for reproducible workflow replay.

---

## Test Cases

### Scenario 1: ctx.now() is safe — no diagnostics

**Given** Rust source code containing a workflow function using ctx.now()
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = ctx.now();
    Ok(())
}
```

**When** `lint_workflow_code(source)` is called

**Then** the result is `Ok(vec![])` with zero diagnostics

---

### Scenario 2: SystemTime::now() — emits diagnostic

**Given** Rust source code with `std::time::SystemTime::now()` call
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = std::time::SystemTime::now();
    Ok(())
}
```

**When** `lint_workflow_code(source)` is called

**Then** the result is `Ok(vec![diagnostic])` with exactly 1 diagnostic where:
- `diagnostic.code == LintCode::L001`
- `diagnostic.message.contains("non-deterministic")`
- `diagnostic.suggestion.contains("ctx.now()")`

---

### Scenario 3: chrono::Utc::now() — emits diagnostic

**Given** Rust source code with `chrono::Utc::now()` call
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = chrono::Utc::now();
    Ok(())
}
```

**When** `lint_workflow_code(source)` is called

**Then** the result is `Ok(vec![diagnostic])` with exactly 1 diagnostic where:
- `diagnostic.code == LintCode::L001`

---

### Scenario 4: chrono::Local::now() — emits diagnostic

**Given** Rust source code with `chrono::Local::now()` call
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = chrono::Local::now();
    Ok(())
}
```

**When** `lint_workflow_code(source)` is called

**Then** the result is `Ok(vec![diagnostic])` with exactly 1 diagnostic

---

### Scenario 5: std::time::Instant::now() — emits diagnostic

**Given** Rust source code with `std::time::Instant::now()` call
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = std::time::Instant::now();
    Ok(())
}
```

**When** `lint_workflow_code(source)` is called

**Then** the result is `Ok(vec![diagnostic])` with exactly 1 diagnostic

---

### Scenario 6: tokio::time::Instant::now() — emits diagnostic

**Given** Rust source code with `tokio::time::Instant::now()` call
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = tokio::time::Instant::now();
    Ok(())
}
```

**When** `lint_workflow_code(source)` is called

**Then** the result is `Ok(vec![diagnostic])` with exactly 1 diagnostic

---

### Scenario 7: ctx.now() — does NOT emit diagnostic

**Given** Rust source code with `ctx.now()` call
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = ctx.now();
    Ok(())
}
```

**When** `lint_workflow_code(source)` is called

**Then** the result is `Ok(vec![])` with zero diagnostics

---

### Scenario 8: Multiple violations — emits all diagnostics

**Given** Rust source code with multiple non-deterministic time calls
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let a = chrono::Utc::now();
    let b = std::time::SystemTime::now();
    let c = std::time::Instant::now();
    Ok(())
}
```

**When** `lint_workflow_code(source)` is called

**Then** the result is `Ok(vec![d1, d2, d3])` with exactly 3 diagnostics where:
- All have `code == LintCode::L001`

---

### Scenario 9: Invalid Rust syntax — returns ParseError

**Given** Rust source code that is syntactically invalid
```rust
async fn workflow { // missing parentheses after ()
```

**When** `lint_workflow_code(source)` is called

**Then** the result is `Err(LintError::ParseError(msg))` where `msg` describes the parse failure

---

### Scenario 10: Diagnostic code is WTF-L001

**Given** Rust source code with any non-deterministic time call
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = chrono::Utc::now();
    Ok(())
}
```

**When** `lint_workflow_code(source)` is called

**Then** `diagnostic[0].code.as_str() == "WTF-L001"`

---

### Scenario 11: Diagnostic message contains "non-deterministic"

**Given** Rust source code with `chrono::Utc::now()`
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = chrono::Utc::now();
    Ok(())
}
```

**When** `lint_workflow_code(source)` is called

**Then** `diagnostic[0].message.contains("non-deterministic")`

---

### Scenario 12: Diagnostic suggestion contains ctx.now()

**Given** Rust source code with `chrono::Utc::now()`
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = chrono::Utc::now();
    Ok(())
}
```

**When** `lint_workflow_code(source)` is called

**Then** `diagnostic[0].suggestion.contains("ctx.now()")`

---

### Scenario 13: Method call style — SystemTime::now()

**Given** Rust source code using method call syntax
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = SystemTime::now();
    Ok(())
}
```

**When** `lint_workflow_code(source)` is called

**Then** the result is `Ok(vec![diagnostic])` with exactly 1 diagnostic

---

### Scenario 14: Method call style — Instant::now()

**Given** Rust source code using method call syntax
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = Instant::now();
    Ok(())
}
```

**When** `lint_workflow_code(source)` is called

**Then** the result is `Ok(vec![diagnostic])` with exactly 1 diagnostic

---

### Scenario 15: Empty source — returns empty diagnostics

**Given** Empty Rust source code
```rust
```

**When** `lint_workflow_code(source)` is called

**Then** the result is `Ok(vec![])` with zero diagnostics

---

### Scenario 16: tokio::time::Instant as method call receiver — flagged

**Given** Rust source code using tokio::time::Instant as method call receiver
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = tokio::time::Instant.now();
    Ok(())
}
```

**When** `lint_workflow_code(source)` is called

**Then** the result is `Ok(vec![diagnostic])` with exactly 1 diagnostic

**NOTE**: This tests whether the implementation correctly flags `tokio::time::Instant.now()` when written in method-call style (receiver is `tokio::time::Instant`, method is `now()`). The implementation checks for `tokio::time` in path style, not `tokio::time::Instant` in method-call style. If this does NOT produce a diagnostic, the implementation has a gap and should be fixed to also detect `tokio::time::Instant` in method-call style.

---

### Scenario 17: chrono::Utc as method call receiver — emits diagnostic

**Given** Rust source code using chrono::Utc as method call receiver
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = Utc::now();
    Ok(())
}
```

**When** `lint_workflow_code(source)` is called

**Then** the result is `Ok(vec![diagnostic])` with exactly 1 diagnostic

---

### Scenario 18: chrono::Local as method call receiver — emits diagnostic

**Given** Rust source code using chrono::Local as method call receiver
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = Local::now();
    Ok(())
}
```

**When** `lint_workflow_code(source)` is called

**Then** the result is `Ok(vec![diagnostic])` with exactly 1 diagnostic

---

### Scenario 19: Unused variable with time call — still emits diagnostic

**Given** Rust source code with unused variable containing time call
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let _ = chrono::Utc::now();
    Ok(())
}
```

**When** `lint_workflow_code(source)` is called

**Then** the result is `Ok(vec![diagnostic])` with exactly 1 diagnostic

**NOTE**: Even though the result is assigned to `_` (intentionally unused), the time call is still non-deterministic and should be flagged.

---

### Scenario 20: Time call in string literal — does NOT emit diagnostic

**Given** Rust source code with time call in string literal
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let msg = "chrono::Utc::now()";
    Ok(())
}
```

**When** `lint_workflow_code(source)` is called

**Then** the result is `Ok(vec![])` with zero diagnostics

**NOTE**: String literals contain text that resembles time calls but is not actual code. The parser does not execute code, so this should not be flagged.

---

## Contract Verification Tests

These tests verify the contract invariants directly:

| Test | Invariant Verified |
|------|-------------------|
| Scenario 1 | Invariant: No false positives for ctx.now() |
| Scenario 2-6 | Invariant: No false negatives for each flagged pattern |
| Scenario 7 | Invariant: ctx.now() is safe pattern |
| Scenario 8 | Invariant: Multiple violations produce multiple diagnostics |
| Scenario 9 | Invariant: Parse errors return as LintError::ParseError |
| Scenario 13-14 | Invariant: Method call syntax is detected |
| Scenario 17-18 | Invariant: chrono method call style (Utc::now(), Local::now()) is detected |
| Scenario 19 | Invariant: Unused assignments still flag time calls |
| Scenario 20 | Invariant: String literals do not false-positive |

---

## Edge Cases

1. **Nested expressions**: Time calls inside closures, match arms, if branches
2. **Unused variables**: `let _ = chrono::Utc::now();` — should still flag (Scenario 19)
3. **Comments**: Time calls in comments should NOT flag (syn parses without comments)
4. **String literals**: `"chrono::Utc::now()"` in a string — should NOT flag (Scenario 20)

---

## Test Execution

Tests should be run via:
```bash
cargo test --package wtf-linter --lib l001_time
```
