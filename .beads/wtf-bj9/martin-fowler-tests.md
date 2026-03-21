# Martin Fowler Test Plan

bead_id: wtf-bj9
bead_title: bead: handle StartWorkflow message
phase: test_plan
updated_at: 2026-03-20T22:40:00Z

## Happy Path Tests

### test_start_workflow_succeeds_under_capacity
**Scenario**: Start a workflow when system is under capacity
- **Given**: `state.running_count = 0`, `max_concurrent = 3`, `name = "test-workflow"`, `input = vec![1, 2, 3]`
- **When**: `handle_start_workflow` is called
- **Then**:
  - `reply` receives `Ok(invocation_id)` where `invocation_id` is a valid ULID string
  - `state.running_count` is incremented to 1
  - `spawn_workflow` was called with correct parameters

### test_start_workflow_returns_valid_invocation_id
**Scenario**: Verify the returned invocation_id is properly formatted
- **Given**: `state.running_count = 0`, `max_concurrent = 3`
- **When**: `handle_start_workflow(name="wf1", input=vec![], reply)` is called
- **Then**: `reply` receives `Ok(id)` where `id.len() == 26` and `id.chars().all(|c| c.is_ascii_alphanumeric())`

## Error Path Tests

### test_start_workflow_returns_at_capacity_error
**Scenario**: Start workflow when at max capacity
- **Given**: `state.running_count = 3`, `max_concurrent = 3`
- **When**: `handle_start_workflow(name="wf1", input=vec![], reply)` is called
- **Then**: `reply` receives `Err(StartError::AtCapacity { running: 3, max: 3 })`
- **And**: `state.running_count` remains unchanged at 3
- **And**: `spawn_workflow` was NOT called

### test_start_workflow_does_not_increment_count_on_spawn_failure
**Scenario**: Spawn fails but capacity check passed
- **Given**: `state.running_count = 1`, `max_concurrent = 3`, `spawn_workflow` configured to fail
- **When**: `handle_start_workflow(name="wf1", input=vec![], reply)` is called
- **Then**: `reply` receives an `Err(...)` from spawn failure
- **And**: `state.running_count` remains 1 (not incremented)

### test_start_workflow_with_empty_name_returns_invalid_input
**Scenario**: Validate workflow name is non-empty
- **Given**: `state.running_count = 0`, `max_concurrent = 3`
- **When**: `handle_start_workflow(name="", input=vec![], reply)` is called
- **Then**: `reply` receives `Err(StartError::InvalidInput(...))`
- **And**: `spawn_workflow` was NOT called

## Edge Case Tests

### test_start_workflow_with_empty_input_succeeds
**Scenario**: Empty input is valid
- **Given**: `state.running_count = 0`, `max_concurrent = 3`
- **When**: `handle_start_workflow(name="wf1", input=vec![], reply)` is called
- **Then**: `reply` receives `Ok(invocation_id)`

### test_start_workflow_at_boundary_count_2_of_3
**Scenario**: Just under capacity
- **Given**: `state.running_count = 2`, `max_concurrent = 3`
- **When**: `handle_start_workflow(name="wf3", input=vec![], reply)` is called
- **Then**: `reply` receives `Ok(invocation_id)`
- **And**: `state.running_count` becomes 3

### test_start_workflow_with_max_concurrent_of_1
**Scenario**: Edge case with minimum concurrency setting
- **Given**: `state.running_count = 0`, `max_concurrent = 1`
- **When**: `handle_start_workflow(name="wf1", input=vec![], reply)` is called
- **Then**: `reply` receives `Ok(invocation_id)`
- **And**: `state.running_count` becomes 1

### test_second_start_workflow_blocked_when_at_capacity_1
**Scenario**: Second workflow blocked when max=1
- **Given**: `state.running_count = 1`, `max_concurrent = 1`
- **When**: `handle_start_workflow(name="wf2", input=vec![], reply)` is called
- **Then**: `reply` receives `Err(StartError::AtCapacity { running: 1, max: 1 })`

## Contract Verification Tests

### test_capacity_check_returns_true_when_under_limit
- **Given**: `state.running_count = 2`, `max_concurrent = 3`
- **When**: `capacity_check(state)` is called
- **Then**: returns `true`

### test_capacity_check_returns_false_when_at_limit
- **Given**: `state.running_count = 3`, `max_concurrent = 3`
- **When**: `capacity_check(state)` is called
- **Then**: returns `false`

### test_running_count_increment_is_exactly_one
**Scenario**: Verify atomic increment
- **Given**: `state.running_count = 0`, `max_concurrent = 3`
- **When**: `handle_start_workflow` succeeds
- **Then**: `state.running_count` is exactly 1 (not 2, not 0)

## Contract Violation Tests

### test_p1_violation_empty_name_returns_error
- **Given**: `name = ""`, `state.running_count = 0`
- **When**: `handle_start_workflow(name, input, reply)` is called
- **Then**: returns `Err(StartError::InvalidInput("workflow name cannot be empty".to_string()))` -- NOT a panic

### test_q1_violation_capacity_not_checked
- **Given**: `state.running_count = 3`, `max_concurrent = 3`, but capacity check is bypassed
- **When**: `handle_start_workflow` is called with bypassed check
- **Then**: The invariant `state.running_count <= max_concurrent` would be violated -- this is a critical bug

### test_q3_violation_count_not_incremented
- **Given**: `state.running_count = 1`, `spawn_workflow` succeeds but increment is skipped
- **When**: `handle_start_workflow` completes
- **Then**: `state.running_count` remains 1 (should be 2) -- this is a bug

## Given-When-Then Scenarios

### Scenario 1: Successful workflow start under capacity
**Given**: The orchestrator has `max_concurrent = 3` and `running_count = 1`  
**And**: A valid workflow name `"test-workflow"` and input `vec![1, 2, 3]`  
**When**: A `StartWorkflow` message is received  
**Then**: The system spawns a new `WorkflowInstance` actor  
**And**: The `running_count` increments to 2  
**And**: The reply channel receives `Ok(invocation_id)` where invocation_id is a ULID

### Scenario 2: Workflow start rejected at capacity
**Given**: The orchestrator has `max_concurrent = 3` and `running_count = 3`  
**When**: A `StartWorkflow` message is received  
**Then**: No new actor is spawned  
**And**: The reply channel receives `Err(StartError::AtCapacity { running: 3, max: 3 })`  
**And**: The `running_count` remains at 3

### Scenario 3: Workflow start with invalid name
**Given**: The orchestrator has `max_concurrent = 3` and `running_count = 0`  
**And**: An empty workflow name `""`  
**When**: A `StartWorkflow` message is received  
**Then**: No new actor is spawned  
**And**: The reply channel receives `Err(StartError::InvalidInput(...))`
