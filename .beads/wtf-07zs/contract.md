# Contract Specification

## Context
- Feature: Heartbeat-driven crash recovery for workflow instances
- Domain terms:
  - `HeartbeatExpired`: NATS KV event emitted when a heartbeat TTL (10s) expires
  - `WorkflowInstance`: ractor actor processing events for a single workflow
  - `Recovery`: re-spawning a WorkflowInstance after crash with state reconstructed from JetStream replay
  - `LastSnapshotSeq`: JetStream sequence number of the last applied snapshot event
- Assumptions:
  - JetStream is the source of truth (ADR-013)
  - Heartbeat TTL is 10s, heartbeat interval is ≤5s
  - Recovery replays from the last `SnapshotTaken` event's seq
- Open questions: None

## Preconditions
- [ ] P1: `HeartbeatExpired` message received only for instances that were previously started
- [ ] P2: NATS client is available for KV and JetStream operations
- [ ] P3: Instance metadata (namespace, workflow_type, paradigm) exists in `wtf-instances` KV

## Postconditions
- [ ] Q1: If instance is not in local registry and metadata exists, a new WorkflowInstance is spawned
- [ ] Q2: The recovered instance replays events from the last snapshot sequence (not from seq=1)
- [ ] Q3: The recovered instance resumes heartbeat emission within 5 seconds of recovery
- [ ] Q4: If instance IS in local registry, no spurious recovery is triggered
- [ ] Q5: If instance metadata does not exist in KV, recovery is skipped with a warning log

## Invariants
- [ ] I1: At most one recovery attempt is in flight for a given instance_id at any time
- [ ] I2: Recovery never re-plays events before the last snapshot sequence
- [ ] I3: The recovered instance has the same instance_id as the crashed one

## Error Taxonomy
- `RecoveryError::InstanceNotFound` - instance metadata absent from `wtf-instances` KV
- `RecoveryError::ReplayFailed` - JetStream replay consumer creation failed
- `RecoveryError::SpawnFailed` - WorkflowInstance actor spawn failed
- `RecoveryError::NoNatsClient` - NATS client unavailable

## Contract Signatures
```rust
// In MasterOrchestrator handle_supervisor_evt or handle function:
async fn handle_heartbeat_expired(
    state: &mut OrchestratorState,
    instance_id: InstanceId,
) -> Result<Option<ActorRef<InstanceMsg>>, RecoveryError>

// RecoveryError enum:
pub enum RecoveryError {
    InstanceNotFound(InstanceId),
    ReplayFailed(String),
    SpawnFailed(String),
    NoNatsClient,
}
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Instance was started | Runtime-checked | Check `wtf-instances` KV |
| P2: NATS available | Runtime-checked | `state.config.nats.is_some()` |
| P3: Metadata exists | Runtime-checked | KV GET returns `Some(InstanceMetadata)` |

## Violation Examples (REQUIRED)
- VIOLATES Q4: `HeartbeatExpired` received for `inst-abc` which IS in `state.active` -- should log debug and ignore, NOT trigger recovery
- VIOLATES Q5: `HeartbeatExpired` for `inst-xyz` with no KV metadata -- should log warn and skip recovery

## Ownership Contracts
- `OrchestratorState::active`: HashMap ownership held by the orchestrator; recovered actors register themselves on spawn
- `instance_id`: Passed by value, cloned as needed for KV lookups and actor refs

## Non-goals
- [ ] Full JetStream stream repair (separate bead)
- [ ] Cross-node instance migration (not supported in v3)
- [ ] Manual recovery API (future work)
