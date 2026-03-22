bead_id: wtf-sztr
bead_title: wtf-linter: WTF-L004 ctx calls inside non-deterministic closures
phase: red-queen
updated_at: 2026-03-21T00:00:00Z

# Red Queen Report: WTF-L004

## Adversarial Test Results

### Defects Found and Fixed

#### Defect 1: Field Access ctx Detection
**Test:** `items.iter().map(|x| x.ctx.activity("test", x))`
**Expected:** Diagnostic emitted
**Actual (before fix):** No diagnostic
**Root Cause:** `is_ctx_receiver` didn't handle `Expr::Field` where member is "ctx"
**Fix:** Added `Expr::Field` case to check if `field_expr.member == "ctx"` or recursively check base
**Status:** FIXED - all 18 tests pass

### Additional Edge Cases Tested

1. **Multiple ctx calls in same closure**: `ctx.activity("a", x); ctx.sleep(...)` - PASS (1 diagnostic)
2. **Named variable that shadows ctx**: `let local_ctx = ctx; ... local_ctx.activity(...)` - PASS (no false positive)
3. **Nested closures**: `map(|x| { let inner = || ctx.activity(...); inner() })` - PASS
4. **Non-target methods**: `iter().map(|x| x + 1)` - PASS (no diagnostic)
5. **Parse errors**: Invalid Rust syntax - PASS (returns Err)

### Conclusion
All adversarial tests pass. Implementation correctly handles:
- Direct `ctx.method()` calls
- Field access `x.ctx.method()` calls
- Multiple ctx calls per closure
- Nested closures with ctx
- No false positives for non-ctx variables

STATUS: PROCEED to Black Hat review.
