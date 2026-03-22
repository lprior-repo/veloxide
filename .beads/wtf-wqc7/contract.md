# Contract: wtf-linter WTF-L006 std::thread::spawn

bead_id: wtf-wqc7
bead_title: wtf-linter: WTF-L006 std::thread::spawn in workflow function
phase: contract
updated_at: 2026-03-21T00:00:00Z

## Overview
Implement rule WTF-L006 in `crates/wtf-linter/src/rules/l006_thread.rs`:
- Flag `std::thread::spawn(...)` in workflow functions with error "std::thread::spawn in workflow function — not replayable"
- Flag `std::thread::sleep(...)` with suggestion to use `ctx.sleep()` instead (WTF-L006b)
- Register all 6 rules in `crates/wtf-linter/src/lib.rs lint_workflow_source()`
- Integration test: source file with all 6 violation types produces exactly 6+ diagnostics with correct codes

## Function Signature
```rust
pub fn check_l006_thread(
    fn_body: &syn::Block,
    diagnostics: &mut Vec<LintDiagnostic>
)
```

## Preconditions
- `fn_body` is a valid syn::Block representing a workflow function body
- `diagnostics` is a valid, non-null Vec that can accept LintDiagnostic entries

## Postconditions
- All `std::thread::spawn` calls within `fn_body` generate a LintDiagnostic with code "WTF-L006"
- All `std::thread::sleep` calls within `fn_body` generate a LintDiagnostic with code "WTF-L006b"
- No false positives for non-thread-related code
- No panics or unwrap calls

## Invariants
- Function never panics
- All paths return normally
- Diagnostics vector is only appended to (no removal)
