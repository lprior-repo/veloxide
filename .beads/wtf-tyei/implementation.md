# Implementation Summary: POST /api/v1/workflows/validate

bead_id: wtf-tyei
bead_title: wtf-api: POST /api/v1/workflows/validate — workflow definition linting endpoint
phase: implementation
updated_at: 2026-03-21T23:45:00Z

## Files Changed

### crates/wtf-api/Cargo.toml
- Added `wtf-linter` dependency
- Added `syn` and `quote` dependencies for AST parsing

### crates/wtf-api/src/types.rs
- Added `ValidateWorkflowRequest` struct (request body)
- Added `ValidateWorkflowResponse` struct (response body)
- Added `DiagnosticEntry` struct (individual diagnostic)

### crates/wtf-api/src/handlers.rs
- Added `validate_workflow` handler function
- Added `collect_lint_diagnostics` helper function
- Added `find_*` helper functions for each lint rule
- Added unit tests for linting functions

### crates/wtf-api/src/routes.rs
- Added POST `/workflows/validate` route

## Contract Clauses Mapping

| Contract Clause | Implementation |
|---|---|
| P1: Valid JSON with source field | `Json<ValidateWorkflowRequest>` with serde Deserialize |
| P2: source is string | Type enforcement via serde |
| Q1: Response has valid and diagnostics | `ValidateWorkflowResponse` struct |
| Q2: valid true when no errors | Logic: `!has_errors` |
| Q3: diagnostics contain all violations | Pattern matching functions for L001-L006 |
| Q4: diagnostic entry structure | `DiagnosticEntry` with code, severity, message, suggestion, span |
| Q5: 400 on parse error | Error handling for `syn::parse_file` |

## Notes

- Clippy warnings in `wtf-linter/src/diagnostic.rs` are pre-existing (doc markup issues)
- Implementation uses simple pattern-based detection (not full AST-based rule evaluation)
- Lint rules L001-L006 are detected via string pattern matching in source
