bead_id: wtf-sztr
bead_title: wtf-linter: WTF-L004 ctx calls inside non-deterministic closures
phase: black-hat
updated_at: 2026-03-21T00:00:00Z

# Black Hat Code Review: WTF-L004

## 5-Phase Review

### Phase 1: Truthfulness
- ✅ Compiles cleanly
- ✅ All 18 unit tests pass
- ✅ No unwrap/expect/panic
- ✅ No unsafe code
- ✅ Returns Err on parse errors

### Phase 2: Completeness
- ✅ `ctx.activity(...)` - path-based ctx detection
- ✅ `x.ctx.activity(...)` - field-based ctx detection
- ✅ `ctx.foo.bar.activity(...)` - nested field ctx detection
- ✅ Handles all 6 target methods: map, for_each, fold, filter_map, and_then, flat_map
- ✅ Nested closures with ctx calls are caught
- ✅ Multiple violations in same source are detected

### Phase 3: Safety (False Positives)
- ✅ `local_ctx = ctx; local_ctx.activity(...)` - no false positive
- ✅ `items.iter().map(|x| x + 1)` - no false positive (closure without ctx)
- ✅ `for item in items { ctx.activity(...) }` - no false positive (sequential iteration)

### Phase 4: Robustness
- ✅ Duplicate closure processing prevented via HashSet
- ✅ Workflow function detection: impl with async fn execute
- ✅ Handles turbofish syntax correctly
- ✅ Handles complex method chaining

### Phase 5: Efficiency
- ✅ Single AST traversal
- ✅ HashSet-based deduplication
- ✅ No unnecessary allocations

## Defects Found
None.

## Final Assessment
STATUS: APPROVED

The implementation correctly:
1. Identifies workflow impl blocks (async fn execute)
2. Detects target iterator methods with ctx-containing closures
3. Handles ctx as path, field access, and nested field access
4. Avoids false positives for non-ctx variables and sequential iteration
5. Uses proper error handling (no panics)

Implementation is production-ready.
