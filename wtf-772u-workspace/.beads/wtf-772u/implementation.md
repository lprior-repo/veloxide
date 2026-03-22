# Implementation Summary

## Bead: wtf-772u — Implement exponential backoff for activity retries

## Changes Made

### 1. Added `calculate_backoff_delay` function in `wtf-worker/src/activity.rs`

```rust
pub fn calculate_backoff_delay(attempt: u32, retry_policy: &wtf_common::RetryPolicy) -> Option<u64>
```

**Formula**: `delay = min(initial_interval_ms * (backoff_coefficient ^ (attempt - 1)), max_interval_ms)`

**Behavior**:
- Returns `None` if `attempt == 0` (invalid, 1-based numbering)
- Returns calculated delay capped at `max_interval_ms`
- Returns `Some` delay in milliseconds

### 2. Added backoff tests in `wtf-worker/src/activity.rs`

- `calculate_backoff_delay_first_attempt`: Returns initial_interval_ms for attempt 1
- `calculate_backoff_delay_second_attempt`: Returns doubled delay for attempt 2
- `calculate_backoff_delay_third_attempt`: Returns 4x delay for attempt 3
- `calculate_backoff_delay_caps_at_max`: Verifies cap at max_interval
- `calculate_backoff_delay_zero_attempt_returns_none`: Invalid attempt
- `calculate_backoff_delay_linear_coefficient_one`: Coefficient 1.0 means no growth
- `calculate_backoff_delay_fractional_coefficient`: Tests 1.5x growth

### 3. Exported `calculate_backoff_delay` from `wtf-worker/src/lib.rs`

Added to public exports alongside other activity functions.

### 4. Modified `Worker::process_task` in `wtf-worker/src/worker.rs`

When an activity handler returns an error:
1. Check if retries are exhausted using `retries_exhausted(attempt, max_attempts)`
2. If NOT exhausted:
   - Calculate backoff delay via `calculate_backoff_delay`
   - Log retry scheduling info
   - Sleep for the calculated delay via `tokio::time::sleep`
   - Create new `ActivityTask` with `attempt + 1`
   - Enqueue the retry task via `enqueue_activity`
3. Call `fail_activity` with `retries_exhausted` flag
4. Ack the original message

## Key Design Decisions

1. **Re-enqueue before fail_activity**: The retry task is enqueued before recording the failure event, ensuring the retry is scheduled even if event recording fails.

2. **Manual re-enqueue instead of nak**: We manually create and enqueue a new task with incremented attempt rather than using `nak()`. This ensures proper retry tracking with incremented attempt number.

3. **Non-blocking on enqueue failure**: If `enqueue_activity` fails, we log the error but continue to record the failure. This prevents a cascade of retries if NATS is unavailable.

## Files Modified

- `crates/wtf-worker/src/activity.rs`: Added `calculate_backoff_delay` function and 7 tests
- `crates/wtf-worker/src/lib.rs`: Exported new function
- `crates/wtf-worker/src/worker.rs`: Added retry logic in `process_task`

## Verification

- [x] Code compiles with `cargo check`
- [x] All 33 unit tests pass (7 new backoff tests + 26 existing)
- [x] No clippy warnings on wtf-worker
- [x] Follows existing code style (no unwrap/expect, proper error handling)
