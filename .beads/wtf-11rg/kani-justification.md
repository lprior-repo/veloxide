# Kani Justification: WTF-L005

## Formal Argument to Skip Kani Model Checking

### 1. What Critical State Machines Exist (or Don't)
The L005 lint rule implementation does NOT contain any critical state machines:
- No stateful actors or workflows
- No state transitions with invariants to verify
- No concurrent state modifications
- The implementation is a pure AST traversal with no mutable state beyond a `Vec<Diagnostic>` collector

### 2. Why Those State Machines Cannot Reach Invalid States
The implementation cannot reach invalid states because:
- `L005Visitor` is constructed once with `in_workflow_fn = false`
- `in_workflow_fn` is only set to `true` when inside a workflow fn body
- `in_workflow_fn` is restored to previous value after visiting the fn
- The only mutable operation is `push` to `diagnostics` Vec
- No branching logic that could lead to inconsistent state

### 3. What Guarantees the Contract/Tests Provide
- Contract: `lint_workflow_code` returns `Result<Vec<Diagnostic>, LintError>`
- Tests verify:
  - Parse errors return `Err(LintError::ParseError)`
  - Empty/no violations return `Ok(empty_vec)`
  - Violations return `Ok(vec_with_diagnostics)`
- No test ever expects a panic or inconsistent state

### 4. Formal Reasoning (Not Hand-Waving)
The implementation is a deterministic function:
```
lint_workflow_code(source) → Result<Vec<Diagnostic>, LintError>
```

For any valid input `source`:
1. `syn::parse_file(source)` either succeeds or returns `ParseError`
2. If parsing fails, we return `Err(LintError::ParseError(...))`
3. If parsing succeeds, we traverse the AST
4. For each expression, we either:
   - Add a diagnostic (if tokio::spawn in workflow fn)
   - Recurse into sub-expressions
5. We never modify already-collected diagnostics
6. We never "forget" to process any expression
7. The final `Vec<Diagnostic>` is always well-formed

### Conclusion
Kani model checking is unnecessary because:
1. No state machines with complex invariants exist
2. The implementation is referentially transparent (same input → same output)
3. All paths through the code either return `Err` or `Ok(diagnostics)`
4. The `Vec<Diagnostic>` accumulator is monotonic (only grows via `push`)

**Recommendation**: Skip Kani. Proceed to State 7 (Architectural Drift).
