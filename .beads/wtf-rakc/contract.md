# Contract Specification: FSM Crash-and-Replay Integration Test

bead_id: wtf-rakc
bead_title: integration test: FSM crash-and-replay — crash after JetStream ACK before KV write
phase: contract
updated_at: 2026-03-22T00:00:00Z

## Context

- Feature: Integration test validating ADR-015 crash window handling for FSM workflow instances
- Domain terms:
  - FSM (Finite State Machine) workflow with states: Created → Processing → Authorized
  - JetStream event log (wtf.log.<ns>.<id>) with events: InstanceStarted, TransitionApplied
  - KV store (wtf-instances) for instance state persistence
  - Crash window: period between JetStream ACK and KV write completion
- Assumptions:
  - NATS server is available via Command::new("nats-server") or testcontainers
  - Sled database for snapshots is available
  - Embedded FSM engine can be paused (SIGSTOP) and killed (SIGKILL)
- Open questions:
  - What is the exact JetStream subject naming convention? (wtf.log.<ns>.<id>)
  - What is the KV store key format for wtf-instances?

## Preconditions

- P1: NATS server must be running and accessible on localhost:4222
- P2: FSM workflow definition for "checkout" must exist and have Authorized as a valid terminal-adjacent state
- P3: Sled database must be initialized and accessible
- P4: Engine process must support SIGSTOP/SIGKILL signals
- P5: KV store (wtf-instances) must be empty before test start

## Postconditions

- Q1: After restart, the instance MUST appear in wtf-instances KV with current_state == 'Authorized'
- Q2: JetStream log MUST contain exactly 2 events: InstanceStarted, TransitionApplied (no duplicates)
- Q3: After restart, the instance's FSM state MUST be exactly 'Authorized' (the state at crash time)
- Q4: No additional TransitionApplied events should appear in JetStream log after restart
- Q5: Engine must successfully replay from JetStream log without re-dispatching activities

## Invariants

- I1: Instance ID remains constant across crash/replay cycle
- I2: Event sequence in JetStream log is immutable (append-only)
- I3: KV store current_state always reflects the latest acknowledged FSM state
- I4: No two TransitionApplied events for the same state transition in JetStream log

## Error Taxonomy

- Error::NatsNotAvailable - NATS server not running or connection failed
- Error::InstanceNotFound - Instance does not appear in KV store after restart timeout
- Error::UnexpectedEventCount - JetStream log has unexpected number of events
- Error::StateMismatch - Restored state does not match expected state
- Error::DuplicateTransition - Duplicate TransitionApplied event detected
- Error::EngineStartTimeout - Engine failed to restart within timeout

## Contract Signatures

```rust
async fn test_fsm_crash_replay() -> Result<(), Error>
async fn setup_nats_server() -> Result<NatsServerHandle, Error>
async fn start_fsm_workflow(workflow_name: &str) -> Result<InstanceId, Error>
async fn advance_to_state(instance_id: &InstanceId, target_state: &str) -> Result<(), Error>
async fn pause_engine_after_event(event_type: &str) -> Result<(), Error>
async fn kill_engine() -> Result<(), Error>
async fn restart_engine() -> Result<(), Error>
async fn wait_for_instance_kv(instance_id: &InstanceId, timeout: Duration) -> Result<String, Error>
async fn get_jetstream_events(instance_id: &InstanceId) -> Result<Vec<JetStreamEvent>, Error>
fn assert_state_eq(actual: &str, expected: &str) -> Result<(), Error>
fn assert_event_count(events: &[JetStreamEvent], expected: usize) -> Result<(), Error>
fn assert_no_duplicate_transitions(events: &[JetStreamEvent]) -> Result<(), Error>
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: NATS available | Runtime-checked | `Result<NatsClient, Error::NatsNotAvailable>` |
| P2: Workflow exists | Compile-time | `WorkflowName: AsRef<str>` with validation |
| P3: Sled accessible | Runtime-checked | `Result<Arc<sled::Db>, Error::SledNotAvailable>` |
| P4: Signal support | Platform assumption | `SIGSTOP/SIGKILL` on Unix |
| P5: KV empty | Runtime-checked | Check before test start |

## Violation Examples (REQUIRED)

- VIOLATES P1: `start_fsm_workflow("checkout")` with NATS not running → returns `Err(Error::NatsNotAvailable)`
- VIOLATES Q1: After restart, wait_for_instance_kv times out → returns `Err(Error::InstanceNotFound)`
- VIOLATES Q2: JetStream log has 3 events instead of 2 → returns `Err(Error::UnexpectedEventCount(3, 2))`
- VIOLATES Q3: Restored state is "Processing" instead of "Authorized" → returns `Err(Error::StateMismatch("Processing", "Authorized"))`
- VIOLATES Q4: Additional TransitionApplied appears after restart → returns `Err(Error::DuplicateTransition)`

## Ownership Contracts

- Engine process handle: owned by test, must be killed and reaped
- NATS server handle: owned by test, must be terminated after test
- InstanceId: Copy type, no ownership transfer
- JetStreamEvent: Clone, no ownership concerns

## Non-goals

- Testing multiple crash windows in sequence
- Testing non-FSM workflow crash recovery
- Testing NATS cluster failure modes
