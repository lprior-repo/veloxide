# Contract Specification

bead_id: wtf-bj9
bead_title: bead: handle StartWorkflow message
phase: contract
updated_at: 2026-03-20T22:40:00Z

## Context

- **Feature**: Implement `StartWorkflow` handling in the `MasterOrchestrator::handle()` method
- **Domain terms**:
  - `MasterOrchestrator`: Root actor that manages workflow lifecycle
  - `OrchestratorMsg::StartWorkflow`: Message to initiate a new workflow
  - `OrchestratorState`: Holds running instance registry and counters
  - `capacity_check()`: Returns true if `running_count < max_concurrent`
  - `spawn_workflow()`: Creates a linked `WorkflowInstance` actor
  - `invocation_id`: ULID-based unique identifier for a workflow invocation
- **Assumptions**:
  - `capacity_check()` is already implemented
  - `spawn_workflow()` is already implemented  
  - `StartError` enum is already defined with `AtCapacity`, `WorkflowNotFound`, `InvalidInput` variants
  - `OrchestratorState.running_count` is already maintained by supervisor events
- **Open questions**: None

## Preconditions

- [P1] The orchestrator must have `max_concurrent` configured (positive usize)
- [P2] The orchestrator must have access to `OrchestratorState.running_count`
- [P3] The `name` parameter must be a non-empty string (valid workflow identifier)
- [P4] The `input` parameter must be a valid byte vector (can be empty but must be owned `Vec<u8>`)
- [P5] The `reply` channel must be open and able to send

## Postconditions

- [Q1] If `capacity_check()` returns false, `reply.send(Err(StartError::AtCapacity {...}))` must be called with `running` and `max` values
- [Q2] If `capacity_check()` returns true, `spawn_workflow()` must be called with correct parameters
- [Q3] If spawn succeeds, `state.running_count` must be incremented by exactly 1
- [Q4] If spawn succeeds, `reply.send(Ok(invocation_id))` must be called with a valid ULID string
- [Q5] If spawn fails, the error must be propagated via `reply.send(Err(...))` without incrementing count
- [Q6] The function must return `Ok(())` after handling regardless of outcome (no panic paths)

## Invariants

- [I1] `state.running_count` never exceeds `max_concurrent`
- [I2] `state.running_count` is always non-negative (using usize)
- [I3] Each entry in `state.instances` has a unique `invocation_id`

## Error Taxonomy

```rust
pub enum StartError {
    #[error("at capacity: {running} workflows running (max {max})")]
    AtCapacity { running: usize, max: usize },
    
    #[error("workflow not found: {0}")]
    WorkflowNotFound(String),
    
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
```

## Contract Signatures

```rust
// The handle method signature (simplified view of the match arm)
async fn handle_start_workflow(
    &self,
    myself: ActorRef<Self::Msg>,
    state: &mut OrchestratorState,
    name: String,
    input: Vec<u8>,
    reply: RpcReplyPort<Result<String, StartError>>,
) -> Result<(), ActorProcessingErr>
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| name non-empty | Runtime-checked constructor | Validate before call |
| input is valid Vec<u8> | Compile-time (strongest) | `Vec<u8>` (owned) |
| reply channel open | Runtime-checked | `RpcReplyPort` type |
| capacity not exceeded | Runtime-checked | `capacity_check()` returns bool |

## Violation Examples (REQUIRED)

- VIOLATES P3 (empty name): Calling `handle_start_workflow` with `name = ""` -- should produce `Err(StartError::InvalidInput("workflow name cannot be empty".to_string()))` via validation before spawn
- VIOLATES Q1 (capacity not checked): If `capacity_check()` returns false but we call `spawn_workflow` anyway -- would violate the capacity invariant
- VIOLATES Q3 (count not incremented): If spawn succeeds but `running_count` is not incremented -- system would accept more workflows than allowed
- VIOLATES Q6 (panic path): If `reply.send()` panics instead of returning Result -- would crash the actor

## Ownership Contracts

- **Ownership transfer**: `name: String` and `input: Vec<u8>` are moved into `spawn_workflow`, caller loses ownership
- **Shared borrow**: `&self` is shared, no mutation of orchestrator internal state except via `state: &mut OrchestratorState`
- **Exclusive borrow**: `state: &mut OrchestratorState` mutates `running_count` and `instances` HashMap
- **Clone policy**: `myself: ActorRef<Self::Msg>` is cloned for the supervisor link, original is consumed

## Non-goals

- [ ] Implementing `capacity_check()` (assumed done)
- [ ] Implementing `spawn_workflow()` (assumed done)
- [ ] Implementing supervisor event handling (separate concern)
- [ ] Error mapping for API layer (separate concern)
