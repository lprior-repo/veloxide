# Contract Specification: wtf-frontend Design Mode Graph Validation

## Context
- **Feature**: Design Mode graph validation for FSM, DAG, and Procedural paradigms
- **Bead ID**: wtf-iobn
- **Bead Title**: wtf-frontend: Design Mode graph validation — unreachable states, missing terminal, DAG cycles
- **Domain terms**:
  - `Workflow`: Graph of nodes and connections representing a workflow
  - `Paradigm`: Execution model (Fsm, Dag, Procedural)
  - `ValidationError`: Error with severity, message, and optional node association
  - `BFS`: Breadth-first search for reachability
  - `Source node`: Node with no incoming edges
  - `Sink node`: Node with no outgoing edges
  - `Linear path`: Single chain from entry to terminal with no branching
- **Assumptions**:
  - Workflow has nodes and connections stored in `Workflow` struct
  - `WorkflowParadigm` enum has `Fsm`, `Dag`, `Procedural` variants
  - `ValidationResult` contains `Vec<ValidationIssue>` with severity, message, and optional node_id
  - petgraph is available for cycle detection via `petgraph::algo::is_cyclic_directed`
- **Open questions**: None

## Preconditions
- [ ] `workflow` parameter must be non-null reference
- [ ] `paradigm` parameter must be a valid `WorkflowParadigm` variant

## Postconditions
- [ ] Returned `ValidationResult` contains all validation issues for the given paradigm
- [ ] For FSM paradigm: all non-entry nodes are reachable from entry nodes via BFS
- [ ] For FSM paradigm: at least one terminal node (no outgoing edges) exists
- [ ] For FSM paradigm: no isolated nodes (nodes with neither incoming nor outgoing edges, except entry)
- [ ] For DAG paradigm: graph is acyclic (`is_cyclic_directed` returns false)
- [ ] For DAG paradigm: exactly one source node exists (node with zero incoming edges)
- [ ] For DAG paradigm: at least one sink node exists (node with zero outgoing edges)
- [ ] For Procedural paradigm: exactly one linear path from entry to terminal
- [ ] For Procedural paradigm: no branching (every node has at most one outgoing connection, except terminal)

## Invariants
- [ ] `ValidationResult::is_valid()` returns true only when no errors exist
- [ ] `ValidationResult::error_count()` equals count of `Error` severity issues
- [ ] `ValidationResult::warning_count()` equals count of `Warning` severity issues

## Error Taxonomy
- `ValidationIssue::Error` - Critical validation failure that prevents execution
- `ValidationIssue::Warning` - Non-critical issue that should be addressed

### FSM-specific Errors
- `Error::UnreachableState` - A state node is not reachable from any entry point via BFS
- `Error::MissingTerminalState` - No terminal (final) state exists in the FSM
- `Error::IsolatedNode` - A node has neither incoming nor outgoing connections

### DAG-specific Errors
- `Error::CyclicGraph` - The DAG contains a cycle (petgraph `is_cyclic_directed` is true)
- `Error::MultipleSourceNodes` - DAG has more than one source node (nodes with no incoming edges)
- `Error::MissingSinkNode` - DAG has no sink node (node with no outgoing edges)

### Procedural-specific Errors
- `Error::NonLinearPath` - Workflow has multiple paths from entry to terminal
- `Error::BranchingEdge` - A node has more than one outgoing connection (not a linear path)

## Contract Signatures
```rust
// In wtf-frontend/src/graph/validation.rs
pub fn validate_workflow_for_paradigm(
    workflow: &Workflow,
    paradigm: WorkflowParadigm
) -> ValidationResult
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| workflow non-null | Compile-time | `&Workflow` (reference, never null) |
| paradigm valid | Compile-time | `WorkflowParadigm` enum (Fsm/Dag/Procedural) |
| BFS reachability | Runtime-checked | `ValidationResult` with Error issues |
| terminal state exists | Runtime-checked | `ValidationResult` with Error issues |
| graph acyclicity | Runtime-checked (petgraph) | `ValidationResult` with Error issues |
| linear path | Runtime-checked | `ValidationResult` with Error issues |

## Violation Examples (REQUIRED)

### FSM Violations
- **VIOLATES P1 (Unreachable state)**: FSM with entry node A connected to B, but C exists disconnected. When validating FSM, C should produce `Err(ValidationIssue::error_for_node("State 'C' is not reachable from any entry point", node_id))`
- **VIOLATES P2 (Missing terminal)**: FSM with entry node A connected to B, but B has outgoing edge back to A (forming cycle, not terminal). Should produce `Err(ValidationIssue::error("FSM must have at least one terminal state"))`
- **VIOLATES P3 (Isolated node)**: FSM with entry A connected to B, and isolated node C with no connections. When validating FSM, C should produce `Err(ValidationIssue::warning_for_node("Node 'C' is isolated", node_id))`

### DAG Violations
- **VIOLATES P1 (Cyclic graph)**: DAG with nodes A→B→C→A forms cycle. When validating DAG, should produce `Err(ValidationIssue::error("DAG contains a cycle"))`
- **VIOLATES P2 (Multiple sources)**: DAG with two entry nodes A and B, both with no incoming edges. When validating DAG, should produce `Err(ValidationIssue::error("DAG must have exactly one source node"))`
- **VIOLATES P3 (Missing sink)**: DAG where all nodes have outgoing edges (no terminal). When validating DAG, should produce `Err(ValidationIssue::error("DAG must have at least one sink node"))`

### Procedural Violations
- **VIOLATES P1 (Non-linear path)**: Procedural with branching: A→B and A→C (two paths from A). When validating Procedural, should produce `Err(ValidationIssue::error("Procedural workflow must have exactly one linear path"))`
- **VIOLATES P2 (Branching edge)**: Procedural where node A connects to both B and C. Should produce `Err(ValidationIssue::error_for_node("Node 'A' has multiple outgoing connections (branching not allowed in procedural)", node_id))`

## Ownership Contracts (Rust-specific)
- `validate_workflow_for_paradigm(&Workflow, WorkflowParadigm)` - Borrows workflow immutably, no mutation
- All validation functions use shared references (`&T`), no exclusive borrows needed
- No ownership transfer occurs in validation

## Non-goals
- [ ] Validation of node-specific configuration (already done by existing `validate_workflow`)
- [ ] Execution of workflows
- [ ] Persistence of validation results
