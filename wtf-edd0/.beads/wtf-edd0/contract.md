# Contract Specification: wtf-edd0

## Context
- **Feature:** Cleanup pending entries on `inject_event` failure
- **Domain terms:**
  - `pending_activity_calls`: HashMap that tracks activity calls awaiting replies
  - `pending_timer_calls`: HashMap that tracks timer calls awaiting replies
  - `inject_event`: Injects a workflow event, returns `Result`
  - `append_and_inject_event`: Publishes event to store, inserts pending entry, calls `inject_event`
  - `append_and_inject_timer_event`: Same pattern for timer events
- **Assumptions:**
  - `inject_event` can fail after the pending entry is inserted
  - When `inject_event` fails, the caller expects the pending entry to be removed
- **Open questions:** None

## Preconditions
- [ ] Event store is available (existing check)
- [ ] `activity_id` or `timer_id` is provided when a pending entry should be inserted

## Postconditions
- [ ] **CRITICAL:** If `inject_event` returns `Err` after a pending entry was inserted, the pending entry MUST be removed before returning
- [ ] If `inject_event` returns `Ok`, the pending entry remains in the map
- [ ] `reply` port receives the result of `inject_event` failure (if applicable)

## Invariants
- [ ] `pending_activity_calls` only contains entries for in-flight activity calls
- [ ] `pending_timer_calls` only contains entries for in-flight timer calls
- [ ] No zombie pending entries exist after failed `inject_event` calls

## Error Taxonomy
- `WtfError::EventInjectionFailed` - when `inject_event` returns `Err` after pending entry inserted
- Other errors (e.g., event store unavailable) occur BEFORE pending insertion and are out of scope for this cleanup contract — they are covered by pre-existing tests

## Contract Signatures
```rust
// append_and_inject_event: cleanup on inject_event failure
async fn append_and_inject_event(
    state: &mut InstanceState,
    event: WorkflowEvent,
    activity_id: Option<ActivityId>,
    reply: ractor::RpcReplyPort<Result<Bytes, WtfError>>,
) {
    // Pattern: insert pending -> inject_event -> cleanup on Err
}

// append_and_inject_timer_event: cleanup on inject_event failure
async fn append_and_inject_timer_event(
    state: &mut InstanceState,
    event: WorkflowEvent,
    timer_id: wtf_common::TimerId,
    reply: ractor::RpcReplyPort<Result<(), WtfError>>,
) {
    // Pattern: insert pending -> inject_event -> cleanup on Err
}
```

## Cleanup Guarantee (Core Contract)
```
insert_pending(aid) 
-> inject_event(seq, event) 
-> if Err: remove_pending(aid) && send reply error
   if Ok: keep_pending(aid)
```

## Non-goals
- [ ] Changing the event store publish logic
- [ ] Modifying `inject_event` signature
- [ ] Adding new error types beyond what's needed for cleanup notification
