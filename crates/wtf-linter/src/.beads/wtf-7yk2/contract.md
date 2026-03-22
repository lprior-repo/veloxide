# Contract Specification: WTF-L003 direct-async-io

## Context
- **Feature**: Implement lint rule WTF-L003 to detect direct async I/O calls in workflow functions
- **Bead ID**: wtf-7yk2
- **Bead Title**: implement wtf-linter WTF-L003: direct-async-io
- **ADR Reference**: ADR-020 (Procedural Workflow Static Linter)
- **Domain Terms**:
  - `workflow function` - A Rust function annotated with workflow semantics that calls `ctx.*` methods
  - `direct async I/O` - Blocking async operations like `reqwest::get()` or `sqlx::query().fetch_one()` called without going through `ctx.activity()`
  - `syn Visit` - The visitor pattern used to walk the Rust AST
- **Assumptions**:
  - Workflow functions are identified by being within a module/file that defines workflow behavior
  - The linter operates on parsed Rust source code using the `syn` crate
  - The linter is conservative (may have false negatives but no false positives)
- **Open Questions**:
  - Should the linter detect ALL reqwest methods (`get`, `post`, `put`, `delete`, etc.) or only `get`?
  - Should `sqlx::query().fetch()` variants (`fetch_one`, `fetch_all`, `fetch_optional`) all be detected?

## Preconditions
- [ ] Source code is valid Rust (parseable by syn)
- [ ] The linter rule is registered in the LintCode enum as L003
- [ ] The visitor module has a `visit_expr` implementation for detecting direct async I/O

## Postconditions
- [ ] When `reqwest::get(...)` is called within a workflow function, a Diagnostic with code L003 is emitted
- [ ] When `sqlx::query(...).fetch_one(...)` is called within a workflow function, a Diagnostic with code L003 is emitted
- [ ] No Diagnostic is emitted for code that properly wraps I/O in `ctx.activity(...)`
- [ ] The Diagnostic contains:
  - `code: LintCode::L003`
  - `severity: Severity::Error`
  - A human-readable message explaining the violation
  - A suggestion to use `ctx.activity(...)` instead
  - The source span (byte position) of the violation

## Invariants
- [ ] The L003 rule never emits a diagnostic for valid workflow code
- [ ] The L003 rule always emits diagnostics for the specific forbidden patterns
- [ ] The linter can be invoked on any valid Rust source file

## Error Taxonomy
- `LintError::ParseError(String)` - When syn fails to parse the source code

## Contract Signatures
```rust
// In wtf-linter/src/visitor.rs
pub struct DirectAsyncIoVisitor {
    diagnostics: Vec<Diagnostic>,
}

impl DirectAsyncIoVisitor {
    pub fn new() -> Self;
    pub fn into_diagnostics(self) -> Vec<Diagnostic>;
}

impl<'ast> Visit<'ast> for DirectAsyncIoVisitor {
    fn visit_expr(&mut self, expr: &'ast syn::Expr);
}

// In wtf-linter/src/lib.rs (or a new rule module)
pub fn check_direct_async_io(source: &str) -> Result<Vec<Diagnostic>, LintError>;
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Source is valid Rust | Runtime-checked | `syn::parse_file()` returns `Result` |
| L003 registered | Compile-time | `LintCode::L003` enum variant exists |
| Visitor produces diagnostics | Compile-time | `Vec<Diagnostic>` returned by visitor |

## Violation Examples (REQUIRED)
- VIOLATES <P1>: `let _ = reqwest::get("https://api.example.com").await;` -- should produce `Err(LintError::ParseError)` if parse fails, but for valid code produces `Diagnostic { code: L003, ... }`
- VIOLATES <P2>: `let row = sqlx::query("SELECT * FROM users").fetch_one(&pool).await?;` -- should produce `Diagnostic { code: L003, ... }`
- VIOLATES <Q1>: After visiting the expression, `self.diagnostics` should contain one Diagnostic with code L003

## Ownership Contracts (Rust-specific)
- `DirectAsyncIoVisitor::new()` creates a visitor with empty diagnostics vector, caller owns the visitor
- `into_diagnostics()` transfers ownership of diagnostics to caller
- `Visit::visit_expr` borrows the expression immutably, does not mutate the AST

## Non-goals
- [ ] Detecting indirect async I/O through custom wrappers
- [ ] Detecting async I/O in non-workflow functions
- [ ] Fixing or auto-correcting violations (only detection)
