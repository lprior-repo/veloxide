# Contract Specification

## Context
- **Feature**: WTF-L005 tokio::spawn detection in workflow functions
- **Bead**: wtf-4hit
- **Domain terms**: 
  - Workflow function: `impl` block with `async fn execute(&self)` method
  - Spawn detection: `tokio::spawn`, `tokio::task::spawn`, `tokio::task::spawn_blocking`
- **Assumptions**: Parent bead wtf-rrz2 provided AST visitor infrastructure
- **Open questions**: None

## Preconditions
- Source code is valid Rust (parseable by syn)

## Postconditions
- All `tokio::spawn`, `tokio::task::spawn`, and `tokio::task::spawn_blocking` calls inside workflow `execute` functions produce exactly one L005 diagnostic each
- No diagnostics emitted for spawn calls outside workflow functions
- No diagnostics emitted for other `tokio::*` calls or `spawn` in different namespaces

## Invariants
- L005 diagnostic code always accompanied by Error severity
- Message references "tokio::spawn" and "workflow" context

## Error Taxonomy
- `LintError::ParseError` - when source cannot be parsed

## Contract Signatures
```rust
pub fn lint_workflow_code(source: &str) -> Result<Vec<Diagnostic>, LintError>
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Valid Rust source | Runtime-checked | `syn::parse_file()` returns Result |
| Spawn path detection | Compile-time | `is_tokio_spawn_path()` pure function |

## Violation Examples (REQUIRED)
- VIOLATES P1 (tokio::spawn in workflow): `tokio::spawn(async {})` inside `async fn execute` → 1 L005 diagnostic
- VIOLATES P1 (tokio::task::spawn in workflow): `tokio::task::spawn(async {})` inside `async fn execute` → 1 L005 diagnostic  
- VIOLATES P1 (tokio::task::spawn_blocking in workflow): `tokio::task::spawn_blocking(|| {})` inside `async fn execute` → 1 L005 diagnostic
- VIOLATES Q1 (no false positive outside workflow): `tokio::spawn(async {})` in regular async fn → 0 diagnostics

## Ownership Contracts
- N/A - pure parsing function, no mutation

## Non-goals
- Detecting `tokio::spawn` inside nested closures outside workflow context
- Linting other tokio APIs
