# Kani Justification: WTF-L002

## bead_id: wtf-0nzz
## phase: kani
## updated_at: 2026-03-21T19:50:00Z

## Formal Argument to Skip Kani

### 1. What Critical State Machines Exist?

The L002 implementation does **not** contain any critical state machines:
- `RandomDetector` is a simple AST visitor that collects diagnostics
- No state transitions occur
- No mutable state that could reach invalid states
- No loops with complex termination conditions

### 2. Why Those State Machines Cannot Reach Invalid States

There are **no state machines** in this implementation:
- `RandomDetector` is a stateless visitor
- `check_random_in_workflow` is a pure function composition
- No internal mutable state that affects control flow
- Diagnostics are accumulated in a `Vec<Diagnostic>` which has a valid empty state

### 3. What Guarantees the Contract/Tests Provide

**Contract Guarantees:**
- `check_random_in_workflow` always returns a valid `Vec<Diagnostic>`
- The vector may be empty (no violations) or contain one or more diagnostics
- Each diagnostic has a valid `LintCode::L002`, `Severity::Error`, and message

**Test Coverage:**
- 6 unit tests verify:
  - Detection of `uuid::Uuid::new_v4()`
  - Detection of `rand::random()` and `rand::random::<T>()`
  - Non-detection of `ctx.random_u64()`
  - Non-detection of `uuid::Uuid::nil()`
  - Multiple violations in same function

### 4. Formal Reasoning

The implementation is a **combinator-based pattern matcher** over an immutable AST:
1. Input: `&File` (syn's parsed AST, guaranteed valid by parser)
2. Processing: `Visit` trait walks AST without mutation
3. Output: `Vec<Diagnostic>` (constructed via `push` from known-good types)

There is **no reachable panic state** because:
- All pattern matching is exhaustive (`match` on `Expr` variants)
- No `unwrap`/`expect` calls
- No indexing operations that could panic
- No arithmetic operations
- No state transitions that could reach invalid states

### Formal Argument Summary

Since:
1. The implementation contains **zero state machines**
2. All operations are **pure functions** over immutable data
3. **No panics** are possible (enforced by `#[deny(clippy::panic)]`)
4. The only collection (`Vec<Diagnostic>`) has only valid states (empty or with elements)

**Kani model checking is unnecessary for this implementation.**

## Kani Verdict

**SKIPPED - Formal justification provided above.**
