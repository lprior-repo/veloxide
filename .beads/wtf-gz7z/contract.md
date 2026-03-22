# Contract Specification: WTF-L001 Non-Deterministic Time Detection

## Metadata
- **bead_id**: wtf-gz7z
- **lint_code**: WTF-L001
- **feature**: Detect uses of `std::time::SystemTime::now()` and other non-deterministic time calls in workflow functions
- **phase**: contract-synthesis
- **updated_at**: 2026-03-22T00:00:00Z

---

## Function Signature

```rust
pub fn lint_workflow_code(source: &str) -> Result<Vec<Diagnostic>, LintError>
```

---

## Preconditions

1. **Valid Rust Syntax**: The `source` parameter must be valid Rust code that can be parsed by `syn::parse_file()`. If not, returns `Err(LintError::ParseError(String))`.

2. **Non-Empty Source**: While empty source will parse successfully, the function is designed to lint workflow function definitions. Empty input returns an empty `Vec<Diagnostic>`.

3. **Utf-8 Source**: The source string must be valid UTF-8.

---

## Postconditions

### 1. Successful Linting
On successful parse and lint:
- Returns `Ok(Vec<Diagnostic>)` where the vector contains zero or more diagnostic entries
- Each `Diagnostic` has:
  - `code == LintCode::L001` (WTF-L001)
  - `message` contains "non-deterministic"
  - `suggestion` contains "ctx.now()" with guidance to use deterministic time

### 2. Flagged Patterns (Must Emit Diagnostic)
The linter MUST emit a diagnostic for ALL of the following:

| Pattern | Example |
|---------|---------|
| `std::time::SystemTime::now()` | Function call path |
| `std::time::Instant::now()` | Function call path |
| `chrono::Utc::now()` | Function call path |
| `chrono::Local::now()` | Function call path |
| `tokio::time::Instant::now()` | Function call path |
| `SystemTime::now()` | Method call on SystemTime receiver |
| `Instant::now()` | Method call on Instant receiver |

### 3. Safe Patterns (Must NOT Emit Diagnostic)
The linter MUST NOT emit a diagnostic for:
- `ctx.now()` — the workflow context's deterministic time method
- Any other `ctx.*` method calls
- Time-related code outside of `.now()` calls (e.g., `Duration::from_secs(5)`)

### 4. Parse Failure
On parse failure:
- Returns `Err(LintError::ParseError(String))` where the string describes the parse error
- Does NOT panic or crash

---

## Invariants

1. **No False Negatives**: Every non-deterministic `.now()` call in the source MUST produce exactly one diagnostic
2. **No False Positives**: `ctx.now()` and non-`.now()` time code MUST NOT produce diagnostics
3. **Deterministic Output**: Same source always produces same diagnostics (no randomness in linting)
4. **Error Isolation**: Parse errors do not crash the linter; they return as `LintError::ParseError`

---

## Error Taxonomy

| Error Type | Condition | Return |
|------------|-----------|--------|
| `LintError::ParseError` | Invalid Rust syntax in source | `Err(LintError::ParseError(msg))` |

---

## Ownership Contract

- The function takes `&str` (borrowed reference) — no ownership transfer
- No internal allocations beyond what syn's parser requires
- The returned `Vec<Diagnostic>` is owned by the caller

---

## Type-Level Preconditions (Enforced by Types)

- `source: &str` — borrowed string slice, never null
- Return type `Result<Vec<Diagnostic>, LintError>` — Railway-oriented error handling
- `Diagnostic` fields are public for inspection but owned types

---

## Violation Examples

### Example 1: SystemTime::now() — MUST FLAG
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = std::time::SystemTime::now(); // ← FLAG
    Ok(())
}
```
Expected: 1 diagnostic, code = L001, message contains "non-deterministic"

### Example 2: chrono::Utc::now() — MUST FLAG
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = chrono::Utc::now(); // ← FLAG
    Ok(())
}
```
Expected: 1 diagnostic, code = L001

### Example 3: ctx.now() — MUST NOT FLAG
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = ctx.now(); // ✓ SAFE
    Ok(())
}
```
Expected: 0 diagnostics

### Example 4: Multiple violations — MUST FLAG ALL
```rust
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let a = chrono::Utc::now();
    let b = std::time::SystemTime::now();
    Ok(())
}
```
Expected: 2 diagnostics, each with code = L001

### Example 5: Invalid Rust — MUST RETURN ERROR
```rust
async fn workflow { // missing parentheses
```
Expected: `Err(LintError::ParseError(...))`
