bead_id: wtf-sztr
bead_title: wtf-linter: WTF-L004 ctx calls inside non-deterministic closures
phase: kani-justification
updated_at: 2026-03-21T00:00:00Z

# Kani Justification: WTF-L004

## Formal Argument to Skip Kani Model Checking

### 1. What Critical State Machines Exist?

**Answer:** NONE.

This implementation is a **purely functional static lint rule**. It:
- Takes source code as input (string)
- Parses it with `syn::parse_file`
- Walks the AST to find patterns
- Returns a `Vec<Diagnostic>` or `LintError`

There are no:
- State machines with mutable state
- Reachable states that could be invalid
- Protocol state transitions
- Async workflow state
- Side effects that could lead to invalid states

### 2. Why Those State Machines Cannot Reach Invalid States

N/A - no state machines exist in this code.

### 3. What Guarantees the Contract/Tests Provide

**Contract Guarantees:**
- `lint_workflow_code` returns `Result<Vec<Diagnostic>, LintError>`
- On parse error: returns `Err(LintError::ParseError(_))`
- On success: returns `Ok(diagnostics)` where each diagnostic has valid fields

**Test Coverage (18 tests):**
- All 18 tests verify correct diagnostic emission or absence
- Tests cover: positive cases, negative cases, edge cases, parse errors
- No test expects panic/unwrap/expect behavior

**Code-Level Guarantees:**
- `#![deny(clippy::unwrap_used)]` - enforced at compile time
- `#![deny(clippy::expect_used)]` - enforced at compile time
- `#![deny(clippy::panic)]` - enforced at compile time
- `#![forbid(unsafe_code)]` - no unsafe code
- All fallible operations use `?` operator or `match`

### 4. Formal Reasoning

**Theorem:** `lint_workflow_code(source)` never panics and always returns either a valid `Vec<Diagnostic>` or a `LintError`.

**Proof:**
1. The function calls `syn::parse_file(source)` which returns `Result<File, Error>`. If parsing fails, we return `Err(LintError::ParseError(...))` via `map_err`. No panic possible.
2. The `L004Visitor` struct is initialized with empty `Vec` and `HashSet`, both valid empty structures.
3. All diagnostic creation uses `Diagnostic::new(...)` which is a simple constructor with no fallibility.
4. The visitor traverses the AST using syn's `Visit` trait, which guarantees proper traversal without panicking.
5. All recursive operations are on `&Expr` and `&Stmt` references, never dereferencing potentially null pointers.
6. The `HashSet<std::ops::Range<usize>>` stores span ranges from `closure_expr.body.span().byte_range()`, which always returns a valid `Range<usize>`.
7. No `unsafe` blocks exist in the code.

**Conclusion:** The function is provably safe. Kani model checking would verify what is already guaranteed by:
- Type safety (Result, Option, HashSet all have safe APIs)
- Compile-time denial of panic/unwrap/expect
- No unsafe code

## Recommendation
**SKIP KANI** - Formal verification is unnecessary for this purely functional, statically verified (via deny attributes) code.
