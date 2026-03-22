# Contract Specification: Graceful Worker Shutdown (wtf-wuvv)

## Context
- **Feature**: Improve graceful shutdown for worker
- **Bead ID**: wtf-wuvv
- **Domain terms**: drain, in-flight activity, shutdown signal, partial completion
- **Assumptions**: Worker uses tokio watch channel for shutdown, NATS JetStream for queue
- **Open questions**: None

## Current Behavior (Binary Shutdown)
- `shutdown_rx` fires → loop breaks immediately
- In-flight tasks are interrupted mid-execution
- No drain phase, no timeout, no partial completion reporting

## Desired Behavior (Graceful Shutdown)
1. When `shutdown_rx` fires, enter `DRAINING` state (don't accept new tasks)
2. Continue processing already-fetched in-flight tasks to completion
3. Configurable drain timeout (default 30s)
4. Report partial completion: which tasks completed, which timed out

## Preconditions
- [ ] `Worker::run` must accept a `DrainConfig` parameter for timeout configuration
- [ ] Shutdown signal (`shutdown_rx`) triggers drain phase, not immediate exit
- [ ] No new tasks are fetched after drain begins

## Postconditions
- [ ] All in-flight tasks complete OR drain timeout expires
- [ ] `ShutdownResult` reports: `completed_count`, `interrupted_count`, `drain_duration_ms`
- [ ] If timeout expires, interrupted tasks are nak'd back to queue
- [ ] Worker logs final shutdown summary with counts

## Invariants
- [ ] Work queue message acks happen exactly once per task (no double-ack, no lost ack)
- [ ] Activity result events are appended to JetStream before ack
- [ ] Drain timeout is enforced viatokio::time::timeout

## Error Taxonomy
- `Error::DrainTimeout` — drain phase exceeded configured timeout
- `Error::NatsPublish` — NATS publish/ack failure (from existing code)
- `Error::QueueClosed` — work queue closed during drain

## Contract Signatures

```rust
/// Configuration for graceful shutdown drain behavior.
#[derive(Debug, Clone)]
pub struct DrainConfig {
    /// Maximum time to wait for in-flight tasks to complete during drain.
    pub drain_timeout: std::time::Duration,
    /// Whether to nak interrupted tasks on timeout (default: true).
    pub nak_on_timeout: bool,
}

impl Default for DrainConfig {
    fn default() -> Self {
        Self {
            drain_timeout: std::time::Duration::from_secs(30),
            nak_on_timeout: true,
        }
    }
}

/// Result of a graceful shutdown, containing drain statistics.
#[derive(Debug, Clone)]
pub struct ShutdownResult {
    pub completed_count: u32,
    pub interrupted_count: u32,
    pub drain_duration_ms: u64,
}

/// Worker::run signature with drain config:
pub async fn run(
    &self,
    shutdown_rx: tokio::sync::watch::Receiver<bool>,
    drain_config: DrainConfig,
) -> Result<ShutdownResult, WtfError>;
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| drain_timeout > 0 | Runtime-checked constructor | `DrainConfig::new() -> Result<DrainConfig, Error>` |
| shutdown signal is WatchReceiver | Compile-time | `tokio::sync::watch::Receiver<bool>` |
| drain phase doesn't accept new tasks | Compile-time | State machine: `Running` → `Draining` → `Done` |

## Violation Examples (REQUIRED)
- VIOLATES <P1>: `DrainConfig::new(Duration::ZERO)` → returns `Err(Error::InvalidDrainTimeout)`
- VIOLATES <P2>: `run()` called without shutdown signal → compile error (Receiver required)
- VIOLATES <Q1>: drain timeout expires with 3 in-flight tasks → `interrupted_count = 3`, `completed_count = 0`

## Ownership Contracts
- `DrainConfig` — cloned into worker, no ownership transfer
- `shutdown_rx` — borrowed for duration of `run()`, caller retains ownership
- `ShutdownResult` — owned by caller after `run()` returns

## Non-goals
- [ ] Cancelling individual in-flight tasks mid-execution (tasks run to completion or timeout)
- [ ] Re-queuing completed-but-unacked tasks (ack happens in process_task)
