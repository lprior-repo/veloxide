# Contract Specification: WTF-L005 tokio-spawn-in-workflow

## Context
- **Feature**: Detect `tokio::spawn()` calls inside procedural workflow functions (ADR-020)
- **Bead ID**: wtf-11rg
- **Bead Title**: implement wtf-linter WTF-L005: tokio-spawn-in-workflow
- **Domain Terms**:
  - Workflow function: `impl WorkflowFn` with `async fn execute(&self, ctx: WorkflowContext) -> anyhow::Result<()>`
  - WorkflowContext: The context parameter passed to workflow execution
  - tokio::spawn: The Tokio runtime spawn function that detaches a future
- **Assumptions**:
  - Workflow functions are identified by implementing `WorkflowFn` trait
  - The linter operates on source code via syn AST parsing
  - The visitor traverses the execute function body looking for tokio::spawn calls
- **Open Questions**: None

## Preconditions
- Source code must be valid Rust (parseable by syn)
- Workflow function body must be reachable via syn AST traversal

## Postconditions
- All `tokio::spawn()` calls found within a workflow function `execute` body must produce a `Diagnostic` with code `L005`
- The diagnostic must include the span (byte position) of the violation
- No false positives: tokio::spawn outside workflow functions must not trigger the diagnostic
- No false negatives: all tokio::spawn calls inside workflow functions must be detected

## Invariants
- The linter must not panic on malformed AST (must handle ParseError gracefully)
- The Diagnostic span must accurately point to the tokio::spawn call location

## Error Taxonomy
- `LintError::ParseError(String)` - Source code cannot be parsed (non-Rust or syntax error)
- `Diagnostic` with `LintCode::L005` - tokio::spawn detected in workflow function
  - Severity: Error
  - Message: "tokio::spawn() is not allowed inside a procedural workflow function. Spawned tasks detach from the workflow context and violate determinism."
  - Suggestion: "Use workflow context's spawn method or convert to a child activity instead."

## Contract Signatures
```rust
// The lint function that scans source code
pub fn lint_workflow_code(source: &str) -> Result<Vec<Diagnostic>, LintError>

// Check if an expression is tokio::spawn
fn is_tokio_spawn(expr: &syn::Expr) -> bool

// Check if we are inside a workflow function body
fn is_inside_workflow_fn(expr: &syn::Expr) -> bool
```

## Type Encoding
| Concern | Enforcement Level | Type / Pattern |
|---|---|---|
| Valid Rust source | Runtime-checked | `Result<Vec<Diagnostic>, LintError>` |
| tokio::spawn detection | Compile-time | Pattern match on `syn::Expr::Call` with path `tokio::spawn` |
| WorkflowFn trait detection | Compile-time | syn Visit trait implementation |
| Span accuracy | Runtime-checked | Byte offsets from syn's Spanned trait |

## Violation Examples (REQUIRED)

### VIOLATES <P1>: tokio_spawn_in_workflow_basic
```rust
#[derive(Debug)]
struct MyWorkflow;
#[async_trait::async_trait]
impl WorkflowFn for MyWorkflow {
    async fn execute(&self, ctx: WorkflowContext) -> anyhow::Result<()> {
        tokio::spawn(async {  // <-- VIOLATION HERE
            println!("detached task");
        });
        Ok(())
    }
}
```
Expected: `Err(LintError::Violation(L005, span_of_tokio_spawn))`

### VIOLATES <P2>: nested_tokio_spawn_in_workflow
```rust
impl WorkflowFn for MyWorkflow {
    async fn execute(&self, ctx: WorkflowContext) -> anyhow::Result<()> {
        let _ = if condition {
            tokio::spawn(async {  // <-- VIOLATION HERE
                do_work().await;
            })
        } else {
            Ok(())
        };
        Ok(())
    }
}
```
Expected: `Err(LintError::Violation(L005, span_of_tokio_spawn))`

### VIOLATES <Q1>: no_false_positive_outside_workflow
```rust
// This is NOT a workflow function - should NOT trigger L005
async fn helper_function() {
    tokio::spawn(async {  // <-- OK - not in workflow fn
        println!("helper task");
    });
}
```
Expected: `Ok(empty_diagnostics)` - no violation

### VIOLATES <Q2>: no_false_positive_on_different_spawn
```rust
impl WorkflowFn for MyWorkflow {
    async fn execute(&self, ctx: WorkflowContext) -> anyhow::Result<()> {
        // Only tokio::spawn is flagged, not other spawn-like patterns
        let handle = std::thread::spawn(|| {  // <-- OK - not tokio::spawn
            println!("thread");
        });
        Ok(())
    }
}
```
Expected: `Ok(empty_diagnostics)` - only L006 should catch std::thread::spawn

## Ownership Contracts
- `lint_workflow_code` takes `&str` (no ownership transfer, borrowed)
- All diagnostic items are owned (`Vec<Diagnostic>`)
- No mutation of input source

## Non-goals
- Detecting `tokio::spawn` in non-workflow functions (false positive prevention)
- Detecting other spawn variants (L001-L004, L006 are separate rules)
- Fixing or auto-correcting violations (detection only)
