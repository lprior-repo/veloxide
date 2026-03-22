bead_id: wtf-sztr
bead_title: wtf-linter: WTF-L004 ctx calls inside non-deterministic closures
phase: martin-fowler-tests
updated_at: 2026-03-21T00:00:00Z

# Martin Fowler Test Plan: WTF-L004

## Given-When-Then Test Cases

---

### Test 1: map with ctx.activity triggers L004

**Given:** A workflow function body containing `items.iter().map(|x| ctx.activity("test", x))`
**When:** L004 visitor analyzes the code
**Then:** A diagnostic is emitted with code WTF-L004 and message containing "ctx call inside closure"

---

### Test 2: for_each with ctx.sleep triggers L004

**Given:** A workflow function body containing `items.iter().for_each(|x| ctx.sleep(Duration::ZERO))`
**When:** L004 visitor analyzes the code
**Then:** A diagnostic is emitted with code WTF-L004

---

### Test 3: fold with ctx.random_u64 triggers L004

**Given:** A workflow function body containing `items.iter().fold(0u64, |acc, _| ctx.random_u64())`
**When:** L004 visitor analyzes the code
**Then:** A diagnostic is emitted with code WTF-L004

---

### Test 4: filter_map with ctx.activity triggers L004

**Given:** A workflow function body containing `items.iter().filter_map(|x| Some(ctx.activity("test", x)))`
**When:** L004 visitor analyzes the code
**Then:** A diagnostic is emitted with code WTF-L004

---

### Test 5: No diagnostic for ctx call in regular for loop

**Given:** A workflow function body containing:
```rust
for item in items {
    ctx.activity("test", item);
}
```
**When:** L004 visitor analyzes the code
**Then:** No diagnostic is emitted (sequential iteration is deterministic)

---

### Test 6: No diagnostic for ctx call outside of target closures

**Given:** A workflow function body containing:
```rust
let x = ctx.random_u64();
items.iter().map(|y| y + 1)
```
**When:** L004 visitor analyzes the code
**Then:** No diagnostic is emitted (ctx call is not inside a closure argument to target method)

---

### Test 7: No diagnostic for ctx call in closure to non-target method

**Given:** A workflow function body containing:
```rust
items.iter().collect::<Vec<_>>().map(|x| ctx.activity("test", x))
```
(Note: `collect` is not a target method)
**When:** L004 visitor analyzes the code
**Then:** No diagnostic is emitted (closure is not an argument to target method)

---

### Test 8: Multiple violations in same source

**Given:** A workflow function body containing:
```rust
items.iter().map(|x| ctx.activity("a", x));
other.iter().for_each(|y| ctx.sleep(Duration::ZERO));
```
**When:** L004 visitor analyzes the code
**Then:** Two diagnostics are emitted, one for each violation

---

### Test 9: Nested closures - only flag direct ctx calls in target closures

**Given:** A workflow function body containing:
```rust
items.iter().map(|x| {
    let inner = || ctx.activity("test", x);
    inner()
})
```
**When:** L004 visitor analyzes the code
**Then:** A diagnostic IS emitted (ctx call is still inside the map closure, even if wrapped in inner closure)

---

### Test 10: ctx on receiver field access triggers L004

**Given:** A workflow function body containing:
```rust
items.iter().map(|x| x.ctx.activity("test"))
```
(Note: method call on a field named ctx)
**When:** L004 visitor analyzes the code
**Then:** A diagnostic is emitted with code WTF-L004

---

### Test 11: Parse error returns Err

**Given:** Invalid Rust source code
**When:** `lint_workflow_code` is called
**Then:** Returns `Err(LintError::ParseError(_))`

---

### Test 12: Diagnostic has correct severity Warning

**Given:** A workflow function body with a violation
**When:** L004 visitor analyzes the code
**Then:** The diagnostic has severity `Severity::Warning`

---

### Test 13: Diagnostic has correct suggestion

**Given:** A workflow function body with a violation
**When:** L004 visitor analyzes the code
**Then:** The diagnostic has suggestion containing "ctx.parallel()" or "sequential iteration"

---

### Test 14: No false positive on closure returning non-ctx value

**Given:** A workflow function body containing:
```rust
items.iter().map(|x| x + 1)
```
**When:** L004 visitor analyzes the code
**Then:** No diagnostic is emitted

---

### Test 15: flat_map with ctx.activity triggers L004

**Given:** A workflow function body containing `items.iter().flat_map(|x| ctx.activity("test", x))`
**When:** L004 visitor analyzes the code
**Then:** A diagnostic is emitted with code WTF-L004

---

## Boundary Conditions

- Empty iterator: `items.is_empty()` then `.map(...)` — no violation possible
- Nested iterator chains: `a.iter().map(...).filter_map(...)` — check each target method
- Closure with multiple ctx calls: emit one diagnostic per ctx call, not per closure
- and_then method: triggers L004 when closure contains ctx call

## Anti-Patterns to Reject

1. Flagging `ctx` that is not the workflow context (e.g., variable named `ctx` that is not the workflow ctx)
2. Flagging `ctx.*` in non-workflow functions
3. Treating `ctx` in comments or strings as violations
4. Missing violations when closure is wrapped in parentheses: `(|x| ctx.activity(x))`
