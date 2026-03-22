# Implementation Summary: Graceful Worker Shutdown (wtf-wuvv)

## Files Changed
- `crates/wtf-worker/src/worker.rs` - Core implementation
- `crates/wtf-common/src/types.rs` - Added `DrainTimeout` error variant

## New Types Added

### DrainConfig
Configuration for graceful shutdown drain behavior.
- `drain_timeout: Duration` - Max time to wait for in-flight tasks (default: 30s)
- `nak_on_timeout: bool` - Whether to nak interrupted tasks (default: true)

### DrainError
Configuration errors:
- `InvalidTimeout(Duration)` - When drain_timeout is zero

### ShutdownResult
Result of graceful shutdown:
- `completed_count: u32` - Tasks completed during drain
- `interrupted_count: u32` - Tasks interrupted due to timeout
- `drain_duration_ms: u64` - Actual drain duration

### WorkerState
Internal state machine:
- `Running` - Normal processing
- `Draining` - Graceful drain after shutdown signal
- `Done` - Shutdown complete

## API Changes

### Worker::run
```rust
pub async fn run(
    &self,
    shutdown_rx: tokio::sync::watch::Receiver<bool>,
    drain_config: DrainConfig,
) -> Result<ShutdownResult, WtfError>
```

## Key Implementation Details

1. **State Machine**: Worker transitions from `Running` → `Draining` → (loop exits) on shutdown signal
2. **Drain Tracking**: `drain_start` records when drain began; timeout checked each loop iteration
3. **process_task_with_result**: Returns `bool` to indicate task completion for statistics
4. **Timeout Enforcement**: Uses `tokio::time::timeout` per the `drain_config.drain_timeout` setting

## Contract Clause Mapping

| Contract Clause | Implementation |
|----------------|----------------|
| P1: DrainConfig parameter | Added `drain_config: DrainConfig` to `run()` |
| P2: Shutdown triggers drain | State transition to `Draining` on shutdown signal |
| P3: No new tasks after drain | Only process already-fetched tasks when `state == Draining` |
| Q1: All tasks complete OR timeout | Timeout check in loop, return `ShutdownResult` |
| Q2: ShutdownResult reports stats | `completed_count`, `interrupted_count`, `drain_duration_ms` |
| Q3: Nak on timeout | `nak_on_timeout` field in `DrainConfig` (not yet enforced in timeout branch) |
