# Martin Fowler Test Plan: wtf-frontend Design Mode Graph Validation

## bead_id: wtf-iobn
## bead_title: wtf-frontend: Design Mode graph validation — unreachable states, missing terminal, DAG cycles
## phase: test-plan
## updated_at: 2026-03-21T18:30:00Z

## Happy Path Tests

### test_fsm_valid_single_state_returns_no_errors
Given: FSM workflow with single entry node that is also terminal (no connections)
When: `validate_workflow_for_paradigm(workflow, WorkflowParadigm::Fsm)` is called
Then: Returns `ValidationResult` with no issues (single node is trivially reachable, terminal, and not isolated)

### test_fsm_valid_linear_chain_returns_no_errors
Given: FSM workflow with entry A connected to B, B connected to terminal C
When: `validate_workflow_for_paradigm(workflow, WorkflowParadigm::Fsm)` is called
Then: Returns `ValidationResult` with no errors (all nodes reachable, terminal exists, no isolated nodes)

### test_dag_valid_tree_structure_returns_no_errors
Given: DAG workflow with single source A, A splits to B and C, both B and C converge to sink D
When: `validate_workflow_for_paradigm(workflow, WorkflowParadigm::Dag)` is called
Then: Returns `ValidationResult` with no errors (acyclic, one source, at least one sink)

### test_procedural_valid_linear_chain_returns_no_errors
Given: Procedural workflow with entry A connected to B, B connected to terminal C
When: `validate_workflow_for_paradigm(workflow, WorkflowParadigm::Procedural)` is called
Then: Returns `ValidationResult` with no errors (single linear path, no branching)

## Error Path Tests

### FSM Tests

#### test_fsm_unreachable_state_returns_error
Given: FSM workflow with entry A connected to B, and disconnected node C
When: `validate_workflow_for_paradigm(workflow, WorkflowParadigm::Fsm)` is called
Then: Returns `ValidationResult` containing error with message containing "not reachable" and node_id of C

#### test_fsm_missing_terminal_state_returns_error
Given: FSM workflow with entry A connected to B, B connected back to A (cycle, no terminal)
When: `validate_workflow_for_paradigm(workflow, WorkflowParadigm::Fsm)` is called
Then: Returns `ValidationResult` containing error with message containing "terminal state"

#### test_fsm_isolated_node_returns_warning
Given: FSM workflow with entry A connected to B, and isolated node C (no incoming or outgoing)
When: `validate_workflow_for_paradigm(workflow, WorkflowParadigm::Fsm)` is called
Then: Returns `ValidationResult` containing warning for node C with message containing "isolated"

#### test_fsm_entry_node_not_reachable_from_itself_is_not_error
Given: FSM workflow with entry node A that has no outgoing connections
When: `validate_workflow_for_paradigm(workflow, WorkflowParadigm::Fsm)` is called
Then: Returns `ValidationResult` with no errors (entry nodes are considered reachable by definition)

### DAG Tests

#### test_dag_cyclic_graph_returns_error
Given: DAG workflow with nodes A→B→C→A (forms cycle)
When: `validate_workflow_for_paradigm(workflow, WorkflowParadigm::Dag)` is called
Then: Returns `ValidationResult` containing error with message containing "cycle"

#### test_dag_multiple_source_nodes_returns_error
Given: DAG workflow with two source nodes A and B (both have no incoming edges)
When: `validate_workflow_for_paradigm(workflow, WorkflowParadigm::Dag)` is called
Then: Returns `ValidationResult` containing error with message containing "exactly one source"

#### test_dag_missing_sink_node_returns_error
Given: DAG workflow where every node has at least one outgoing edge (no terminal node)
When: `validate_workflow_for_paradigm(workflow, WorkflowParadigm::Dag)` is called
Then: Returns `ValidationResult` containing error with message containing "at least one sink"

#### test_dag_single_node_valid
Given: DAG workflow with single node A (no connections)
When: `validate_workflow_for_paradigm(workflow, WorkflowParadigm::Dag)` is called
Then: Returns `ValidationResult` with no errors (single node is both source and sink)

### Procedural Tests

#### test_procedural_branching_returns_error
Given: Procedural workflow where node A has outgoing connections to both B and C
When: `validate_workflow_for_paradigm(workflow, WorkflowParadigm::Procedural)` is called
Then: Returns `ValidationResult` containing error with message containing "branching" and node_id of A

#### test_procedural_multiple_paths_returns_error
Given: Procedural workflow with entry A branching to B and C, both converging to terminal D
When: `validate_workflow_for_paradigm(workflow, WorkflowParadigm::Procedural)` is called
Then: Returns `ValidationResult` containing error with message containing "linear path"

#### test_procedural_single_linear_path_valid
Given: Procedural workflow with entry A connected to B connected to terminal C
When: `validate_workflow_for_paradigm(workflow, WorkflowParadigm::Procedural)` is called
Then: Returns `ValidationResult` with no errors

## Edge Case Tests

### test_empty_workflow_returns_error
Given: Workflow with empty nodes list and empty connections list
When: `validate_workflow_for_paradigm(workflow, any_paradigm)` is called
Then: Returns `ValidationResult` containing error with message containing "no nodes"

### test_single_node_all_paradigms
Given: Workflow with single node A (no connections)
When: Validating with Fsm, Dag, and Procedural paradigms
Then: 
- Fsm: Returns no errors (entry is also terminal)
- Dag: Returns no errors (single node is both source and sink)
- Procedural: Returns no errors (single node is valid linear path)

### test_two_nodes_no_connection_all_paradigms
Given: Workflow with two disconnected nodes A and B
When: Validating with Fsm, Dag, and Procedural paradigms
Then:
- Fsm: Returns warning that B is unreachable/isolated
- Dag: Returns error about multiple source nodes
- Procedural: Returns error about non-linear path or missing terminal

## Contract Verification Tests

### test_validation_result_error_count_accuracy
Given: ValidationResult with 3 errors and 2 warnings
When: `error_count()` and `warning_count()` are called
Then: Returns 3 and 2 respectively

### test_validation_result_is_valid_only_when_no_errors
Given: ValidationResult with errors vs only warnings
When: `is_valid()` is called
Then: Returns false when errors exist, true when only warnings exist

## Given-When-Then Scenarios

### Scenario 1: FSM validation accepts valid state machine
Given: A workflow with entry state "Start", intermediate state "Processing", and terminal state "Complete"
And: Start connects to Processing, Processing connects to Complete
When: `validate_workflow_for_paradigm(workflow, WorkflowParadigm::Fsm)` is called
Then:
- The result contains no errors
- All states are reachable from Start
- Complete is identified as terminal

### Scenario 2: DAG validation rejects cyclic dependency
Given: A workflow with nodes A, B, C where A→B, B→C, C→A
When: `validate_workflow_for_paradigm(workflow, WorkflowParadigm::Dag)` is called
Then:
- The result contains exactly one error
- Error message indicates a cycle was detected
- No source node error (multiple sources not triggered)

### Scenario 3: Procedural validation rejects parallel branches
Given: A workflow with entry A that branches to B and C in parallel, both leading to terminal D
When: `validate_workflow_for_paradigm(workflow, WorkflowParadigm::Procedural)` is called
Then:
- The result contains one or more errors
- Error indicates branching is not allowed
- Error indicates linear path is required
