# Implementation Summary

## Bead: wtf-7yk2
## Title: implement wtf-linter WTF-L003: direct-async-io

## What was implemented

- Completed visitor-level contract parity in `crates/wtf-linter/src/visitor.rs`:
  - Added concrete span extraction from syn expression spans (`loc_of`) so L003 diagnostics include location metadata.
  - Preserved explicit parse error behavior via `check_direct_async_io(source)`.
  - Strengthened tests for contract requirements around severity and spans.

- Added/updated tests to cover contract expectations:
  - no diagnostics on clean/ctx.activity-only paths
  - diagnostics for reqwest/sqlx direct async I/O
  - multiple diagnostics for multiple violations
  - parse error on invalid Rust
  - diagnostics include:
    - `LintCode::L003`
    - `Severity::Error`
    - non-empty suggestion
    - present source span

## Contract adherence notes

- Preconditions:
  - valid Rust is runtime-checked with `syn::parse_file`.
- Postconditions:
  - direct async I/O emits L003 diagnostics with error severity and suggestion.
  - ctx.activity path remains unflagged.
  - spans now attached to diagnostics.
- Invariants:
  - no panic behavior for parse failures; returns `LintError::ParseError`.

## Verification

- `cargo test -p wtf-linter visitor -- --nocapture`

## Files changed

- `crates/wtf-linter/src/visitor.rs`
