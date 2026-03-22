bead_id: wtf-sztr
bead_title: wtf-linter: WTF-L004 ctx calls inside non-deterministic closures
phase: qa-review
updated_at: 2026-03-21T00:00:00Z

# QA Review: WTF-L004

## Decision: APPROVED

## QA Summary

### Compilation
- `cargo check -p wtf-linter`: PASS (1 pre-existing warning in visitor.rs, not l004)
- All 15 l004 unit tests pass
- Clippy warnings in l004.rs: FIXED (removed unused_self, match_same_arms, unnecessary_map_or, missing_errors_doc)

### Test Coverage
- Positive cases: map, for_each, fold, filter_map, and_then, flat_map with ctx.*
- Negative cases: regular for loop, ctx outside closure, non-target methods
- Edge cases: nested closures, multiple violations, parse errors, severity, suggestion
- All 15 tests pass

### Code Quality
- No panics/unwrap/expect (verified)
- No unsafe code
- Follows existing l005/l006 patterns
- HashSet-based deduplication prevents duplicate diagnostics

### Issues Found
None in l004 implementation.

### Pre-existing Issues (Not in scope)
- visitor.rs has unused `ExprCall` import
- visitor.rs has several clippy warnings (not addressed as they are pre-existing)

## Conclusion
Implementation is correct, tests pass, code quality is good. PROCEED to adversarial testing.
