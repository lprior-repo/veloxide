# Implementation Summary

## Bead: wtf-qsdw
## Title: Per-Activity Timeout Support

## Implemented

- Extended `ActivityTask` with per-activity timeout:
  - Added `timeout: Option<Duration>` field in `crates/wtf-worker/src/queue.rs`.
  - Added msgpack-compatible timeout serialization/deserialization shim.
  - Added `ActivityTask::validate()` enforcing timeout constraints.
  - `from_msgpack()` now validates decoded tasks.

- Updated worker flow to preserve timeout across retries:
  - retry task clone now propagates `timeout` field.

- Added tests for timeout contract boundaries:
  - zero timeout rejected
  - 1ms timeout accepted

- Updated integration fixtures to include `timeout` field in all task constructors.

## Verification

- `cargo test -p wtf-worker -- --nocapture`

## Files changed

- `crates/wtf-worker/src/queue.rs`
- `crates/wtf-worker/src/worker.rs`
- `crates/wtf-worker/tests/worker_integration_tests.rs`
