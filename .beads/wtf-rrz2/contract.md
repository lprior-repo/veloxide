# Contract Specification: wtf-linter AST Walker + Diagnostic Infrastructure

## Context
- **Feature**: syn AST walker + diagnostic infrastructure for wtf-linter
- **Bead ID**: wtf-rrz2
- **Domain terms**: LintDiagnostic, LintResult, LintVisitor, workflow functions, AST traversal
- **Assumptions**: 
  - syn crate is available for parsing Rust source
  - Workflow functions are identified by `#[workflow]` attribute or `_workflow` suffix
  - Each rule visitor implements the `LintVisitor` trait
- **Open questions**: None

## Data Types

### LintDiagnostic
```
struct LintDiagnostic {
    code: String,        // e.g., "WTF-L001"
    message: String,     // Human-readable diagnostic message
    file: String,        // Source file path
    line: u32,           // Line number (1-indexed)
    col: u32,            // Column number (1-indexed)
    suggestion: String,   // Optional fix suggestion
}
```

### LintResult
```
struct LintResult {
    diagnostics: Vec<LintDiagnostic>,
    has_errors: bool,    // true if any diagnostic has error severity
}
```

## Preconditions
- [P1] `source` parameter must be valid UTF-8 string
- [P2] `source` parameter must be parseable Rust code (syn::parse_file must succeed)

## Postconditions
- [Q1] Returned `LintResult` contains all diagnostics found by all visitors
- [Q2] `has_errors` is `true` iff at least one diagnostic represents an error (not warning)
- [Q3] All workflow functions in the source are visited exactly once
- [Q4] Each LintDiagnostic has accurate line/col positions within the source

## Invariants
- [I1] `LintResult.diagnostics` never contains duplicate diagnostics for the same location
- [I2] All LintDiagnostic.code values follow pattern `WTF-LXXX` where XXX is 3 digits

## Error Taxonomy
- `Error::ParseFailure` - syn::parse_file failed to parse the source
- `Error::InvalidSource` - source is not valid UTF-8

## Contract Signatures
```rust
pub fn lint_workflow_source(source: &str) -> Result<LintResult, Error>
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: valid UTF-8 | Compile-time | `&str` (guaranteed UTF-8 in Rust) |
| P2: parseable Rust | Runtime-checked | `Result<LintResult, Error::ParseFailure>` |

## Violation Examples
- VIOLATES P2: `lint_workflow_source("not rust code @#$")` → `Err(Error::ParseFailure("..."))`
- VIOLATES Q2: Source with only warnings (no errors) → `LintResult { has_errors: false, ... }`

## Ownership Contracts
- `source: &str` - borrowed input, no ownership transferred
- Return value owns its `Vec<LintDiagnostic>` allocation

## LintVisitor Trait
```rust
pub trait LintVisitor {
    fn check(&self, fn_body: &syn::Block, diagnostics: &mut Vec<LintDiagnostic>);
}
```

## Workflow Function Identification Rules
1. Functions annotated with `#[workflow]` attribute
2. Functions with name ending in `_workflow`

## Non-goals
- Linting of non-workflow functions
- Auto-fixing diagnostics
- Caching parsed results
