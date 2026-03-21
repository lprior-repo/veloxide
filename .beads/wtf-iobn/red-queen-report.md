# Red Queen Report: wtf-iobn

## bead_id: wtf-iobn
## phase: red-queen
## updated_at: 2026-03-21T19:05:00Z

## Adversarial Test Cases

### Edge Case 1: Empty Workflow
- **Input**: Workflow { nodes: [], connections: [] }
- **Expected**: Error "Workflow has no nodes"
- **Implementation**: Returns early with error ✓

### Edge Case 2: Single Node (All Paradigms)
- **Input**: Workflow with single node, no connections
- **Expected**: Valid for all paradigms
- **Implementation**: 
  - FSM: Valid (entry is terminal) ✓
  - DAG: Valid (single node is both source and sink) ✓
  - Procedural: Valid (single linear path) ✓

### Edge Case 3: Cycle Detection
- **Input**: A→B→C→A
- **Expected**: DAG returns "cycle detected" error
- **Implementation**: Uses petgraph::algo::is_cyclic_directed ✓

### Edge Case 4: Multiple Sources
- **Input**: Two disconnected nodes
- **Expected**: DAG returns "exactly one source" error
- **Implementation**: Counts nodes with zero incoming edges ✓

### Edge Case 5: Self-Loop
- **Input**: Node A with connection to itself
- **Expected**: 
  - FSM: Warning (unreachable from entry, isolated)
  - DAG: Cycle detected ✓
  - Procedural: Error (branching not allowed, non-linear path)
- **Implementation**: Handled correctly ✓

### Edge Case 6: Disconnected Subgraph
- **Input**: Entry A connected to B, disconnected C→D
- **Expected**: C and D unreachable warnings for FSM
- **Implementation**: BFS from entry won't reach C/D ✓

### Edge Case 7: Diamond Pattern
- **Input**: A→B, A→C, B→D, C→D
- **Expected**:
  - FSM: Valid (no issues)
  - DAG: Valid (tree with converge)
  - Procedural: Error (branching)
- **Implementation**: Correctly differentiates by paradigm ✓

## Red Queen Decision
**PASS** - All adversarial cases handled correctly.
