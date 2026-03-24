# Contract Specification: handle_procedural_msg Error Handling

## Context
- **Feature:** Fix error handling in `handle_procedural_msg` to prevent silent failures
- **Location:** `crates/wtf-actor/src/instance/handlers.rs`
- **Domain terms:** Procedural handlers, reply channels, ActorProcessingErr
- **Assumptions:** Procedural handlers return `Result<T, ActorProcessingErr>` or similar error type
- **Open questions:** None (resolved - see Invariant 1 amendment)

## Problem Statement
`handle_procedural_msg` calls procedural handlers (e.g., `procedural::handle_dispatch`) but ignores their return values. Errors are silently dropped, causing callers to timeout without knowing the dispatch failed.

## Preconditions
- [ ] `state` must be a valid `&mut InstanceState`
- [ ] `myself_ref` must be a valid `ActorRef<InstanceMsg>`
- [ ] `msg` must be one of the procedural message variants

## Postconditions
- [ ] All errors from procedural handlers are logged via `tracing::error!`
- [ ] If a reply channel exists on `handle_dispatch` or `handle_sleep`, error is sent back via that channel
- [ ] If no reply channel exists on `handle_dispatch` or `handle_sleep`, error is propagated up via return value
- [ ] `handle_procedural_msg` returns `Result<(), ActorProcessingErr>` with actual error on failure

## Invariants

### Invariant 1 (Amended): Error Reporting Guarantee
**Base rule:** Errors are never silently dropped without at least logging.

**Exception for `handle_now` and `handle_random`:** Due to non-determinism semantics, these handlers MAY silently drop reply channels when:
- `event_store` is missing (operation cannot be made deterministic)
- event publish fails (caller receiving a non-persisted value would cause divergence)

In these cases, the caller will timeout/error rather than receive a non-deterministic value that wasn't persisted. This is intentional behavior, not silent error suppression.

**All other handlers (`handle_dispatch`, `handle_sleep`, `handle_wait_for_signal`):** Must always report errors via reply channel AND/OR propagation.

### Invariant 2 (Amended): Reply Channel Error Handling
**For handlers with reply channels (`handle_dispatch`, `handle_sleep`, `handle_wait_for_signal`):**
- If a reply port exists, error MUST be sent via that channel
- Error must ALSO be logged via `tracing::error!`

**For handlers without reply channels (`handle_completed`, `handle_failed`):**
- Errors are logged via `tracing::error!`
- No reply to send

**For `handle_now` and `handle_random`:** Reply channel errors are intentionally dropped per Invariant 1 exception.

## Error Taxonomy
- `ActorProcessingErr` - Base error type from ractor framework
- Errors may originate from:
  - `procedural::handle_dispatch` - Activity dispatch failures
  - `procedural::handle_sleep` - Timer/sleep failures
  - `procedural::handle_now` - Time query failures (reply-dropping permitted on failure)
  - `procedural::handle_random` - Random generation failures (reply-dropping permitted on failure)
  - `procedural::handle_wait_for_signal` - Signal wait failures
  - `procedural::handle_completed` - Workflow completion handling failures
  - `procedural::handle_failed` - Workflow failure handling failures

### Error Variants
| Variant | Cause | Handler |
|---------|-------|---------|
| "Event store missing" | `state.args.event_store` is `None` | dispatch, sleep, now*, random*, wait_for_signal |
| "mock publish failure" | `event_store.publish()` returns error | dispatch, sleep, now*, random*, wait_for_signal |
| "Unexpected message in procedural handler" | Non-procedural `InstanceMsg` received | handle_procedural_msg catch-all |

*Note: `handle_now` and `handle_random` log errors but do NOT propagate them to reply channel on failure (Intentionally dropped per Invariant 1 exception).

## Contract Signatures
```rust
async fn handle_procedural_msg(
    myself_ref: ActorRef<InstanceMsg>,
    msg: InstanceMsg,
    state: &mut InstanceState,
) -> Result<(), ActorProcessingErr>
```

### Per-Handler Requirements:

**handle_dispatch, handle_sleep, handle_wait_for_signal:**
1. Await the result: `.await?` instead of `.await;`
2. On error:
   - Log with `tracing::error!(?err, "description")`
   - If `reply` port exists, send error via `reply.send(Err(err.clone()))`
   - Return the error via `?`

**handle_now, handle_random:**
1. Await the result: `.await?` instead of `.await;`
2. On error:
   - Log with `tracing::error!(?err, "description")`
   - **DO NOT** attempt to send error via reply (intentionally dropped per Invariant 1)
   - Return the error via `?`

**handle_completed, handle_failed:**
1. Await the result: `.await?` instead of `.await;`
2. On error:
   - Log with `tracing::error!(?err, "description")`
   - Return the error via `?`
   - Note: No reply channel exists for these handlers

## Non-goals
- [ ] Modifying the underlying procedural handler implementations
- [ ] Changing error types or adding new error variants
- [ ] Adding retry logic or compensation

---

## Resolution of Previous Contract Conflict (LETHAL-1)

**Original conflict:** Contract stated "Errors are never silently dropped" but `handle_now` and `handle_random` intentionally drop replies on failure to prevent non-determinism.

**Resolution:** Invariant 1 has been AMENDED to include an explicit exception for `handle_now` and `handle_random`. The key insight is:
- Errors ARE logged (not truly "silent")
- Reply channel is dropped to prevent non-deterministic values from being returned
- The caller will timeout/error rather than receive a value that wasn't persisted

This is intentional behavior for correctness, not a bug. The contract now explicitly documents this exception.

(End of file - total 119 lines)