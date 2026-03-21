# Martin Fowler Test Plan

## Happy Path Tests
- `test_master_orchestrator_new_returns_valid_struct` - Given valid max_concurrent > 0 and storage, When MasterOrchestrator::new is called, Then returns Ok(MasterOrchestrator) with correct fields
- `test_orchestrator_state_new_creates_empty_instances` - Given OrchestratorState::new(), When called, Then instances HashMap is empty (len == 0)
- `test_orchestrator_state_new_sets_running_count_to_zero` - Given OrchestratorState::new(), When called, Then running_count == 0
- `test_actor_pre_start_initializes_state_correctly` - Given MasterOrchestrator actor, When pre_start is called, Then returns OrchestratorState with empty registry and 0 running count

## Error Path Tests
- `test_master_orchestrator_new_rejects_zero_capacity` - Given max_concurrent = 0, When MasterOrchestrator::new(0, storage) is called, Then returns Err(Error::InvalidCapacity)
- `test_master_orchestrator_new_accepts_minimal_capacity` - Given max_concurrent = 1, When MasterOrchestrator::new(1, storage) is called, Then returns Ok

## Edge Case Tests
- `test_orchestrator_state_default_is_consistent` - Given default construction, When multiple instances created, Then all have same empty/zero state
- `test_orchestrator_state_with_capacity_init` - Given OrchestratorState::with_capacity(c), When called, Then behaves same as new()

## Contract Verification Tests
- `test_invariant_running_count_never_exceeds_max` - Given running_count initialized to 0 and max_concurrent set, When state is accessed, Then invariant holds (running_count <= max_concurrent)
- `test_invariant_instances_keys_are_non_empty` - Given initialized state, When instances is accessed, Then all keys are non-empty strings
- `test_invariant_instances_values_are_valid` - Given initialized state, When instances is accessed, Then all values are (non-empty name, valid ActorRef)

## Contract Violation Tests
- `test_violation_p1_zero_capacity_returns_error` - Given max_concurrent = 0 and valid storage, When MasterOrchestrator::new is called, Then returns Err(Error::InvalidCapacity) -- NOT panic
- `test_violation_q2_empty_instances_after_init` - Given fresh OrchestratorState, When checked, Then instances.len() == 0 -- exact value
- `test_violation_q3_zero_running_count_after_init` - Given fresh OrchestratorState, When checked, Then running_count == 0 -- exact value

## Given-When-Then Scenarios

### Scenario 1: MasterOrchestrator initialization with valid inputs
**Given**: max_concurrent = 3 and a valid Arc<sled::Db>
**When**: MasterOrchestrator::new(max_concurrent, storage) is called
**Then**: 
- Returns Ok(MasterOrchestrator)
- .max_concurrent == 3
- .storage is the same Arc passed in

### Scenario 2: MasterOrchestrator rejects invalid capacity
**Given**: max_concurrent = 0 and a valid Arc<sled::Db>
**When**: MasterOrchestrator::new(0, storage) is called
**Then**: 
- Returns Err(Error::InvalidCapacity)
- Does NOT panic
- Error contains context about capacity being zero

### Scenario 3: OrchestratorState initializes to known state
**Given**: No prior state
**When**: OrchestratorState::new() is called
**Then**:
- instances HashMap has length 0
- running_count is 0
- No entries in instances

### Scenario 4: Actor pre_start produces correct initial state
**Given**: MasterOrchestrator actor with max_concurrent = 3
**When**: pre_start(myself) is called
**Then**:
- Returns Ok(OrchestratorState)
- State.instances.len() == 0
- State.running_count == 0

---

bead_id: wtf-5et
bead_title: bead: MasterOrchestrator struct and OrchestratorState
phase: martin-fowler-tests
updated_at: 2026-03-20T00:00:00Z
