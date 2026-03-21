# Contract Specification

## Context
- Feature: MasterOrchestrator struct and OrchestratorState (ADR-006)
- Bead ID: wtf-5et
- Domain terms: MasterOrchestrator, OrchestratorState, Actor supervision, capacity enforcement
- Assumptions:
  - Using ractor actor framework
  - Using sled for persistent storage
  - Two-level actor hierarchy (MasterOrchestrator → WorkflowInstance)
- Open questions: None

## Preconditions
- [ ] P1: max_concurrent must be > 0 (capacity limit must be valid)
- [ ] P2: storage must be initialized (Arc<sled::Db> must be valid)
- [ ] P3: OrchestratorState must be created via proper constructor (not direct initialization)

## Postconditions
- [ ] Q1: MasterOrchestrator::new() returns valid struct with max_concurrent and storage set
- [ ] Q2: OrchestratorState initializes with empty instances HashMap (len == 0)
- [ ] Q3: OrchestratorState initializes with running_count == 0
- [ ] Q4: Actor impl pre_start initializes state correctly

## Invariants
- [ ] I1: running_count is always <= max_concurrent (capacity never exceeded)
- [ ] I2: instances HashMap keys (invocation_id) are always non-empty strings
- [ ] I3: instances HashMap values are always (workflow_name, valid ActorRef)

## Error Taxonomy
- Error::InvalidCapacity - when max_concurrent == 0
- Error::StorageUnavailable - when sled::Db is not accessible
- Error::StateInitializationFailed - when OrchestratorState cannot be created

## Contract Signatures
```rust
impl MasterOrchestrator {
    pub fn new(max_concurrent: usize, storage: Arc<sled::Db>) -> Result<Self, Error>
    where Self: Sized;
}

impl OrchestratorState {
    pub fn new() -> Self;
    pub fn with_capacity(capacity: usize) -> Self;
}

impl Actor for MasterOrchestrator {
    type Msg = OrchestratorMsg;
    type State = OrchestratorState;
    type Arguments = (); // No arguments to pre_start

    fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
    ) -> Result<Self::State, ActorProcessingErr> {
        // Initializes empty registry and 0 running count
    }
}
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| max_concurrent > 0 | Runtime-checked constructor | `NonZeroUsize::new()` or Result error |
| storage valid | Compile-time | `Arc<sled::Db>` (always valid if constructed properly) |
| state initialized | Compile-time | `OrchestratorState::new()` returns initialized state |

## Violation Examples (REQUIRED)
- VIOLATES P1: `MasterOrchestrator::new(0, storage)` -- should produce `Err(Error::InvalidCapacity)`
- VIOLATES Q2: `OrchestratorState { instances: HashMap::new(), running_count: 0 }.instances.len()` -- after proper `new()`, must be 0
- VIOLATES I1: Any code that allows `running_count > max_concurrent` -- indicates bug

## Ownership Contracts (Rust-specific)
- `Arc<sled::Db>`: Shared ownership, clone on spawn, no mutation of db itself
- `ActorRef<InstanceMsg>`: Cheap clone, no ownership transfer
- `HashMap<String, (String, ActorRef<InstanceMsg>)>`: Internal mutation via state borrow

## Non-goals
- [ ] Full OrchestratorMsg enum implementation (separate bead)
- [ ] spawn_workflow implementation (separate bead)
- [ ] handle_supervisor_evt implementation (separate bead)
- [ ] capacity_check method (separate bead)

---

bead_id: wtf-5et
bead_title: bead: MasterOrchestrator struct and OrchestratorState
phase: contract
updated_at: 2026-03-20T00:00:00Z
