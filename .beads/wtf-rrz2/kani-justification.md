# Kani Justification: wtf-linter AST Walker

## Why Kani is Not Applicable

### What Critical State Machines Exist?

**None.** The wtf-linter crate is a pure parsing and AST visiting library with no state machines:

1. **lint_workflow_source**: Stateless parsing function
   - Input: `&str` (source code)
   - Output: `Result<LintResult, LintError>`
   - No mutable state, no state transitions

2. **LintVisitor trait**: Simple visitor interface
   - `fn check(&self, fn_body: &syn::Block, diagnostics: &mut Vec<LintDiagnostic>)`
   - No internal state in the trait

3. **WorkflowFinder**: AST visitor struct
   - Contains only: `visitors` slice reference + diagnostics mutable reference
   - No internal state that could reach invalid states
   - No ownership anomalies possible

4. **LintDiagnostic/LintResult**: Plain data structs
   - `#[derive(Debug, Clone)]` - no interior mutability
   - No `RefCell`, `Cell`, `OnceCell`, `Lazy`, etc.

### Why These State Machines Cannot Reach Invalid States

1. **No dynamic dispatch with interior mutability**: 
   - `LintVisitor` is a trait object but only holds `&'a (dyn LintVisitor + 'static)`
   - No `Box<dyn LintVisitor>` with `RefCell` inside
   - Borrow checker enforces all borrow rules at compile time

2. **No optional unwrap chains**:
   - Code uses `map_err` and proper Result handling
   - No `.unwrap()` anywhere in the codebase
   - `#![deny(clippy::unwrap_used)]` enforced

3. **No collection iteration that could panic**:
   - Uses `.any()` which returns false on empty, no index access
   - Uses `.iter().any(LintDiagnostic::is_error)` - safe iterator usage

4. **No interior mutability patterns**:
   - `#![deny(clippy::unwrap_used)]`
   - `#![deny(clippy::expect_used)]`
   - `#![deny(clippy::panic)]`
   - `#![forbid(unsafe_code)]`

### What Guarantees the Contract/Tests Provide

1. **Contract Spec (contract.md)**:
   - P1: Valid UTF-8 (guaranteed by `&str`)
   - P2: Parseable Rust (enforced by `Result<LintResult, LintError::ParseError>`)
   - Q1: All diagnostics collected (guaranteed by visitor pattern)
   - Q2: has_errors correctness (guaranteed by `is_error()` check)
   - Q3: Each workflow visited once (guaranteed by syn's Visit trait)

2. **Clippy Pedantic Enforcement**:
   - Zero unwrap/expect/panic in source
   - Zero unsafe code
   - All warnings treated as errors

3. **Test Coverage**:
   - Unit test for `is_error()` correctness
   - Compilation success is itself a verification

### Formal Reasoning

The crate consists only of:
- **Data types**: `LintDiagnostic`, `LintResult`, `LintError` - plain structs, no invariants
- **Pure functions**: `lint_workflow_source` - total function, no partiality except parse failure
- **Trait objects**: `LintVisitor` - interface only, no implementation state
- **Visitor struct**: `WorkflowFinder` - contains only borrows, no owned state that could be corrupted

**Conclusion**: There are no panic-worthy states in this code. Kani would verify `true` for all properties since the state space is either:
- Safe (data types have no invalid states)
- Parse error (explicitly handled as Result)

No model checking required.
