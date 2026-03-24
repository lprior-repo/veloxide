# Implementation Summary: ActivityCompleted Idempotency Fix (wtf-l7zi)

## Problem
When a duplicate `ActivityCompleted` event arrived for an already-completed activity, the actor crashed with `UnknownActivityId` error instead of handling it idempotently.

## Root Cause
The original code at `apply.rs:72` used `ok_or_else(|| Error)?` which immediately returned an error if the `activity_id` was not found in `in_flight`, without checking if it was already completed and stored in `checkpoint_map`.

## Solution Implemented

### 1. Extended `Checkpoint` struct (`mod.rs`)
Added optional `activity_id` field to enable idempotency lookup:
```rust
pub struct Checkpoint {
    pub result: Bytes,
    pub completed_seq: u64,
    pub activity_id: Option<ActivityId>,  // NEW: for idempotency
}
```

### 2. Updated `apply_event` for `ActivityCompleted` (`apply.rs`)
Changed the logic to handle three cases:
- **Case 1**: `activity_id` found in `in_flight` → normal completion processing
- **Case 2**: `activity_id` NOT in `in_flight` BUT IS in `checkpoint_map` → return `AlreadyApplied` (idempotent)
- **Case 3**: `activity_id` NOT in `in_flight` AND NOT in `checkpoint_map` → return `UnknownActivityId` error

### 3. Updated all Checkpoint creations
- `ActivityCompleted`: stores `activity_id: Some(aid)`
- `NowSampled`, `RandomSampled`, `TimerFired`, `SignalReceived`: stores `activity_id: None`

## Files Changed

| File | Change |
|------|--------|
| `crates/wtf-actor/src/procedural/state/mod.rs` | Added `activity_id: Option<ActivityId>` to `Checkpoint` |
| `crates/wtf-actor/src/procedural/state/apply.rs` | Implemented idempotent handling for duplicate `ActivityCompleted` |

## Constraint Adherence

### Functional Rust Constraints
- ✅ **Zero Panics/Unwraps**: No `unwrap()`, `expect()`, or `panic!()` used
- ✅ **Zero Mutability**: Used `state.clone()` for immutable state transitions  
- ✅ **Expression-Based**: Match expressions for control flow, no imperative loops
- ✅ **Error Handling**: `ProceduralApplyError::UnknownActivityId` returned for truly unknown activities

### Coding Rigor Constraints
- ✅ **Function Size**: `apply_event` is ~150 lines (under 200 limit)
- ✅ **Single Responsibility**: Each match arm handles one event type
- ✅ **Tests Pass**: All 151 tests green

## Verification
- `cargo test -p wtf-actor`: **151 tests passed**
- `cargo build -p wtf-actor --lib`: **Compiles successfully**

## Contract Fulfillment
| Postcondition | Status |
|--------------|--------|
| `activity_id` in `in_flight` → normal completion | ✅ |
| `activity_id` NOT in `in_flight` BUT in `checkpoint_map` → `AlreadyApplied` | ✅ |
| `activity_id` NOT in `in_flight` AND NOT in `checkpoint_map` → `UnknownActivityId` error | ✅ |
