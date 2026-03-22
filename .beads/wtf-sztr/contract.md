bead_id: wtf-sztr
bead_title: wtf-linter: WTF-L004 ctx calls inside non-deterministic closures
phase: contract
updated_at: 2026-03-21T00:00:00Z

# Contract Specification: WTF-L004

## 1. Overview

**What:** Flag `ctx.*` method calls that appear inside closure arguments to iterator methods that may reorder execution.

**Why:** Iterator methods like `.map()`, `.for_each()`, `.fold()`, and `.filter_map()` execute closures non-deterministically with respect to the workflow context. Calling `ctx.activity()` or other `ctx.*` methods inside these closures breaks deterministic replay.

**Severity:** Warning (not Error, per LintCode::L004 severity mapping)

## 2. Rule Definition

### Target Methods (Closure-Dispatching Iterator Methods)
- `map`
- `for_each`
- `fold`
- `filter_map`
- `and_then`
- `flat_map`

### Pattern to Detect
1. Visit `ExprMethodCall` where:
   - The method name is one of the target methods
   - Any argument contains an `ExprClosure` that itself contains a `ctx.*` method call

### ctx.* Methods to Flag
Any method call on an expression that:
- Is a `Path` with segment `ctx`
- Is a `Field` access where the base is `ctx`
- Is a `Reference` to `ctx`

Examples of flagged calls:
- `ctx.activity(...)`
- `ctx.sleep(...)`
- `ctx.random_u64()`
- `some_var.ctx.method()` (field access on ctx)

## 3. Diagnostic Output

**Code:** WTF-L004

**Severity:** Warning

**Message:** "ctx call inside closure may execute in non-deterministic order — use ctx.parallel() or sequential iteration"

**Suggestion:** "Consider using `for item in items { ctx.activity(item) }` for deterministic sequential execution, or `ctx.parallel(items, |item| ctx.activity(item))` if parallel execution is safe."

## 4. Context Awareness

- Only flag inside workflow function bodies (impl blocks with `async fn execute`)
- Do not flag `ctx.*` calls outside of closures passed to these methods
- Do not flag `ctx.*` calls in regular `for` loops, `while` loops, or sequential iterators

## 5. Implementation Contract

### Function Signature
```rust
pub fn lint_workflow_code(source: &str) -> Result<Vec<Diagnostic>, LintError>
```

### Module Structure
- New file: `crates/wtf-linter/src/l004.rs`
- Export added to `crates/wtf-linter/src/lib.rs`

### Visitor Behavior
1. Parse source with `syn::parse_file`
2. Find all workflow impl blocks (impl items with `async fn execute`)
3. For each workflow function body, walk the AST
4. When `ExprMethodCall` is visited with target method name:
   - Check if any argument is an `ExprClosure`
   - If so, recursively visit the closure body looking for `ctx.*` calls
5. Emit diagnostic for each `ctx.*` call found inside qualifying closures

### ctx.* Detection Logic
A `ctx.*` call is identified when:
- The method call's receiver or a path component contains identifier `ctx`
- Specifically: `Expr::MethodCall` where receiver or path segment is `ctx`

## 6. Error Handling

- Parse errors: Return `LintError::ParseError`
- No panics, no unwrap/expect on fallible operations
- All fallible operations use `?` operator or `match`

## 7. Exports

The module must export:
```rust
pub fn lint_workflow_code(source: &str) -> Result<Vec<Diagnostic>, LintError>
```
