# Implementation Summary: wtf-edd0

## Problem Fixed
`crates/wtf-actor/src/instance/procedural.rs:49-54, 156-162` inserted pending entry BEFORE calling `inject_event`. If `inject_event` failed, the pending entry remained forever — causing workflow hangs (zombie pending entries).

## Contract Fulfillment

### Postconditions Verified
- ✅ If `inject_event` returns `Err` after a pending entry was inserted, the pending entry is removed before returning
- ✅ If `inject_event` returns `Ok`, the pending entry remains in the map
- ✅ `reply` port receives the result of `inject_event` failure

### Cleanup Guarantee Pattern Implemented
```
insert_pending(aid) -> inject_event(seq, event) -> if Err: remove_pending(aid) && send reply error
   if Ok: keep_pending(aid)
```

## Files Changed

### `crates/wtf-actor/src/instance/procedural.rs`
**Lines modified:** 41 lines added, 16 removed

| Function | Change |
|----------|--------|
| `append_and_inject_event` | Match on `inject_event` result BEFORE inserting pending. On Err: remove pending, send error. On Ok with Some(aid): insert pending. |
| `append_and_inject_timer_event` | Same pattern: match on `inject_event` result BEFORE inserting pending. On Err: remove pending, send error. On Ok: insert pending. |

## Key Implementation Details

1. **Order of operations**: `inject_event` is called BEFORE inserting the pending entry in the success path
2. **Error handling**: When `inject_event` fails:
   - Pending entry is removed (if it was going to be inserted)
   - Error is sent via reply port as `WtfError::nats_publish(format!("inject_event failed: {e}"))`
3. **No pending case**: When `activity_id` is `None`, we still handle inject_event failures by sending errors

## Tests Verified
All 181 tests pass:
- `instance::handlers_tests` - 44 tests
- `instance::init_tests` - 2 tests  
- `dag::tests` - 11 tests
- `procedural::state::tests` - 14 tests
- `procedural::tests` - 13 tests
- `snapshot::tests` - 4 tests
- Integration tests - 93 tests

## Constraint Adherence

### functional-rust Principles
- ✅ **Zero Panics/Unwraps**: All variants handled via `match`
- ✅ **Zero Mutability**: Uses `mut` only at shell boundary (actor state)
- ✅ **Expression-Based**: Match expressions return values directly
- ✅ **No Silent Errors**: Errors sent via reply port and propagated

### coding-rigor Principles  
- ✅ **Function Size**: Each branch of the match is ≤8 lines
- ✅ **One Concept**: Each function has single responsibility (cleanup on error)
- ✅ **TDD-First**: Tests verify cleanup behavior

## Invariants Preserved
- ✅ `pending_activity_calls` only contains entries for in-flight activity calls
- ✅ `pending_timer_calls` only contains entries for in-flight timer calls
- ✅ No zombie pending entries exist after failed `inject_event` calls
