# Implementation: wtf-iobn Design Mode Graph Validation

## bead_id: wtf-iobn
## bead_title: wtf-frontend: Design Mode graph validation — unreachable states, missing terminal, DAG cycles
## phase: implementation
## updated_at: 2026-03-21T18:45:00Z

## Files Changed

1. **crates/wtf-frontend/Cargo.toml**
   - Added `petgraph = { workspace = true }` dependency

2. **crates/wtf-frontend/src/graph/mod.rs**
   - Updated export to include `Paradigm` enum and `validate_workflow_for_paradigm` function

3. **crates/wtf-frontend/src/graph/validation.rs**
   - Added `Paradigm` enum with `Fsm`, `Dag`, `Procedural` variants
   - Added `validate_workflow_for_paradigm(workflow: &Workflow, paradigm: Paradigm) -> ValidationResult`
   - Added `validate_fsm_constraints()` - BFS reachability, terminal state check, isolated node detection
   - Added `validate_dag_constraints()` - cycle detection via petgraph, single source check, sink existence check
   - Added `validate_procedural_constraints()` - linear path validation, branching edge detection
   - Added comprehensive tests for all paradigm validations

## Contract Clause Mapping

| Contract Clause | Implementation |
|----------------|----------------|
| FSM: BFS reachability | `validate_fsm_constraints()` - BFS from entry nodes |
| FSM: terminal state exists | `validate_fsm_constraints()` - checks for nodes with no outgoing edges |
| FSM: no isolated nodes | `validate_fsm_constraints()` - checks for nodes with neither incoming nor outgoing |
| DAG: is_cyclic_directed == false | `validate_dag_constraints()` - uses petgraph::algo::is_cyclic_directed |
| DAG: exactly one source | `validate_dag_constraints()` - counts nodes with zero incoming edges |
| DAG: at least one sink | `validate_dag_constraints()` - counts nodes with zero outgoing edges |
| Procedural: linear path | `validate_procedural_constraints()` - follows single path, checks all nodes visited |
| Procedural: no branching | `validate_procedural_constraints()` - checks each node has at most one outgoing edge |

## Key Design Decisions

1. **Paradigm enum defined locally**: Since wtf-frontend doesn't depend on wtf-actor, a local `Paradigm` enum was created mirroring the actor's `WorkflowParadigm`

2. **petgraph for DAG cycle detection**: Uses `petgraph::algo::is_cyclic_directed` for O(V+E) cycle detection

3. **Functional approach**: All validation functions are pure - they take `&Workflow` and produce `ValidationResult` with no mutation

4. **Tests cover all violation examples**: Each violation example from contract.md has a corresponding test

## No Unsafe Code
All new code uses safe Rust - no `unsafe`, no `unwrap` in source code, no `panic!`
