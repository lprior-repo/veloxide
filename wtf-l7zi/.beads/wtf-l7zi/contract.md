# Contract Specification: ActivityCompleted Idempotency

## Context
- **Feature:** `ActivityCompleted` duplicate handling in Procedural workflow state
- **Bead:** `wtf-l7zi`
- **Domain terms:**
  - `in_flight`: `HashMap<operation_id, ActivityId>` — active operations awaiting completion
  - `checkpoint_map`: `HashMap<operation_id, Checkpoint>` — completed operations
  - `applied_seq`: `HashSet<u64>` — idempotency guard for event sequence numbers
- **Assumptions:**
  - `ActivityCompleted` may be delivered more than once (at-least-once delivery)
  - When `activity_id` is not in `in_flight`, it means either: (a) it already completed and was removed, or (b) it's a spurious message
  - The `checkpoint_map` contains the canonical record of completed work
- **Open questions:**
  - None

## Preconditions
- [ ] `seq` is not already in `applied_seq` (checked before this logic)

## Postconditions
- [ ] If `activity_id` exists in `in_flight`: normal completion processing occurs
- [ ] If `activity_id` is NOT in `in_flight` but IS in `checkpoint_map`: return `AlreadyApplied`
- [ ] If `activity_id` is NOT in `in_flight` and NOT in `checkpoint_map`: return `UnknownActivityId` error

## Invariants
- [ ] `in_flight` always contains only operations that have been dispatched but not yet completed
- [ ] `checkpoint_map` contains the union of all completed operations
- [ ] An `activity_id` can only be in one of `in_flight` OR `checkpoint_map`, never both

## Error Taxonomy
- `ProceduralApplyError::UnknownActivityId(String)` — `activity_id` not found in `in_flight` AND not found in `checkpoint_map` (truly unknown)
- Note: `AlreadyApplied` is NOT an error — it's a `ProceduralApplyResult` variant

## Contract Signatures
```rust
// No signature change required — error variant semantics change only
pub fn apply_event(
    state: &ProceduralActorState,
    event: &WorkflowEvent,
    seq: u64,
) -> Result<(ProceduralActorState, ProceduralApplyResult), ProceduralApplyError>
```

## Idempotency Semantics

### Happy Path (normal completion)
```
Given: activity_id X is in in_flight with operation_id Y
When:  ActivityCompleted { activity_id: X } arrives
Then:  Remove X from in_flight, insert checkpoint Y→{result, seq}, return ActivityCompleted { operation_id: Y, result }
```

### Idempotent Path (duplicate after completion)
```
Given: activity_id X is NOT in in_flight (already removed on first completion)
  AND: activity_id X IS in checkpoint_map (via reverse lookup of operation_id→activity_id mapping)
When:  ActivityCompleted { activity_id: X } arrives again
Then:  Return AlreadyApplied, state unchanged
```

### Error Path (truly unknown)
```
Given: activity_id X is NOT in in_flight
  AND: activity_id X is NOT in checkpoint_map (never seen this activity)
When:  ActivityCompleted { activity_id: X } arrives
Then:  Return UnknownActivityId error
```

## Non-goals
- [ ] Changing the HashMap lookup to a different data structure
- [ ] Adding a reverse index from ActivityId → operation_id in checkpoint_map (too invasive for this fix)
- [ ] Handling ActivityFailed duplicates this way — different semantics
