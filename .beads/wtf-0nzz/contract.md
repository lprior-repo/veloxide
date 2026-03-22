# Contract Specification: WTF-L002 (non-deterministic-random)

## Context
- Feature: Implement lint rule WTF-L002 to detect non-deterministic randomness in workflow functions
- Domain terms:
  - `uuid::Uuid::new_v4()` - creates a random UUID (non-deterministic)
  - `rand::random::<T>()` / `rand::random()` - generates random values (non-deterministic)
  - `ctx.random_u64()` - deterministic alternative that logs and replays consistently
  - Workflow function - a Rust function annotated with `#[workflow]` or containing `ctx.*` calls
- Assumptions:
  - The syn AST visitor infrastructure exists in wtf-linter
  - Diagnostic, LintCode (L002), Severity types are already defined
  - Rule stubs exist in rules.rs but are not yet implemented
- Open questions:
  - Do we need to detect `rand::rngs::ThreadRng::new()` followed by `.gen()`?
  - Do we need to detect custom Random implementations?

## Preconditions
- [P1] Source file must contain valid Rust syntax (syn can parse it)
- [P2] Analysis is performed on workflow function bodies only (not imports, not module-level code)
- [P3] The visitor must traverse the full AST including nested blocks, closures, and async blocks

## Postconditions
- [Q1] When `uuid::Uuid::new_v4()` call is found inside a workflow function body, exactly one Diagnostic with code L002 is emitted
- [Q2] When `rand::random::<T>()` or `rand::random()` call is found inside a workflow function body, exactly one Diagnostic with code L002 is emitted
- [Q3] No diagnostic is emitted for `ctx.random_u64()` calls (they are deterministic)
- [Q4] No diagnostic is emitted for `uuid::Uuid::nil()` or `uuid::Uuid::parse()` (not random)
- [Q5] Each diagnostic includes: code L002, severity Error, message mentioning "non-deterministic random", suggestion to use ctx.random_u64()
- [Q6] The span (file location) is correctly set to point to the exact method call expression

## Invariants
- [I1] The visitor never panics on valid Rust input (all errors return Result)
- [I2] All L002 diagnostics have severity Error (not Warning)
- [I3] The same violation detected multiple times in the same function produces multiple diagnostics (no deduplication)

## Error Taxonomy
- LintError::ParseError(String) - when syn fails to parse the source file

## Contract Signatures
```rust
// The main entry point for the L002 rule
fn check_random_in_workflow(wf: &WorkflowAnalyzer) -> Vec<Diagnostic>

// Syn visitor trait implementation
impl Visit for RandomDetector {
    fn visit_expr(&mut self, node: &Expr)
}
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Valid Rust syntax | Compile-time (syn) | `syn::parse_file()` returns `Result<File>` |
| P2: Workflow function body | Runtime-checked | `WorkflowAnalyzer::is_workflow_function()` |
| P3: Full AST traversal | Compile-time | syn Visit trait guarantees traversal |

## Violation Examples (REQUIRED)
- VIOLATES Q1: `uuid::Uuid::new_v4()` in workflow function → emits Diagnostic(L002)
- VIOLATES Q2: `rand::random::<u64>()` in workflow function → emits Diagnostic(L002)
- VIOLATES Q3: `ctx.random_u64()` in workflow function → NO diagnostic (correct)
- VIOLATES Q4: `uuid::Uuid::parse("urn:uuid:...")` → NO diagnostic (correct)
- VIOLATES Q5: Diagnostic message does not mention "non-deterministic" → diagnostic rejected
- VIOLATES Q6: Span points to wrong location → diagnostic rejected

## Ownership Contracts
- `RandomDetector` takes `&mut self` for interior mutability during AST traversal
- `Diagnostic` is Clone (owned copy returned to caller)
- No ownership transfer of AST nodes (borrowed references)

## Non-goals
- Implementing L001, L003-L006 rules (separate beads)
- Implementing the full workflow detection (only flagging specific APIs)
- Detecting runtime randomness (only static analysis of specific calls)

---

## Scope Map
| Item | Location |
|---|---|
| Rule implementation | `wtf-linter/src/rules.rs` (add L002 implementation) |
| Visitor module | `wtf-linter/src/visitor.rs` (add RandomDetector struct) |
| Exports | `wtf-linter/src/lib.rs` (export if needed) |
| Tests | `wtf-linter/tests/l002_random_test.rs` (new file) |

## Traceability
- Parent epic: `wtf-6n5n` (epic: Phase 6 — Procedural Workflow Linter)
- Depends on: Diagnostic, LintCode (L002 enum variant), Severity types already exist
- Sibling beads: wtf-gz7z (L001), wtf-7yk2 (L003), wtf-5vs9 (L004), wtf-11rg (L005), wtf-rbay (L006)
