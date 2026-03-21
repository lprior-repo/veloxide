# ADR-002: Use petgraph for DAG Representation

## Status

Accepted

## Context

wtf-engine workflows are **Directed Acyclic Graphs (DAGs)** where:

- **Nodes** = States (Pass, Task, Choice, Parallel, Map, Wait, Succeed, Fail)
- **Edges** = Control flow connections (with ports: "main", "true", "false", branch names)
- **Topological order** = Execution order
- **Fan-out** = Parallel branches
- **Fan-in** = Synchronization after Parallel/Map

### Requirements for DAG Implementation

1. **Type-safe nodes and edges** - WorkflowNode + Connection types
2. **Topological sort** - For execution ordering
3. **Edge filtering** - By port name (only follow "true" branch if condition is true)
4. **Parallel branch detection** - Find all outgoing edges from Parallel node
5. **Graph traversal** - Ready nodes (all predecessors completed)
6. **Serialization** - Save/load workflow definitions

### Candidate Libraries

| Library | Pros | Cons |
|---------|------|------|
| **petgraph** | Mature, `toposort`, `FloydWarshall`, well-tested | Generic graph library |
| `graph替代` | Custom implementation | Re-inventing the wheel |
| `petgraph` + custom wrapper | petgraph + typed API | Additional abstraction |

## Decision

We will use **petgraph** for DAG representation, wrapped in a **WorkflowDAG** type that provides a typed, domain-specific API.

### Why petgraph

1. **Battle-tested** - Used in Rust compiler, Cargo, many production systems
2. **Rich algorithms** - `toposort`, `dominators`, `floyd_warshall`
3. **Type-safe** - `Graph<N, E>`, `NodeIndex`, `EdgeIndex`
4. **Serializable** - `serde` derive support
5. **Zero-cost abstraction** - Direct graph access when needed
6. **Well-maintained** - Regular releases, responsive maintainer

### Architecture

```rust
// Core type - petgraph DiGraph with domain types
pub type WorkflowGraph = DiGraph<StateNode, Connection>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateNode {
    pub id: u32,
    pub name: String,
    pub state_type: StateType,
    pub config: StateConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub source_port: String,   // "main", "true", "false", "branch_0", etc.
    pub target_port: String,   // Usually "main"
}
```

### Wrapper API (WorkflowDAG)

```rust
pub struct WorkflowDag {
    graph: WorkflowGraph,
    entry_point: NodeIndex,
}

impl WorkflowDag {
    // Execution ordering via topological sort
    pub fn execution_order(&self) -> Vec<NodeIndex> {
        petgraph::algo::toposort(&self.graph, None)
            .expect("Workflow DAG must be acyclic")
    }

    // Find ready nodes (all predecessors completed)
    pub fn ready_nodes(&self, completed: &HashSet<NodeIndex>) -> Vec<NodeIndex> {
        self.graph.node_indices()
            .filter(|&idx| {
                !completed.contains(&idx)
                && self.graph.neighbors_directed(idx, Incoming)
                    .all(|pred| completed.contains(&pred))
            })
            .collect()
    }

    // Get outgoing edges for a node, filtered by port
    pub fn edges_from(&self, node: NodeIndex, port: &str) -> Vec<(NodeIndex, &Connection)> {
        self.graph.edges_directed(node, Outgoing)
            .filter(|e| e.weight().source_port == port)
            .map(|e| (e.target(), e.weight()))
            .collect()
    }

    // Fan-out for Parallel state
    pub fn parallel_branches(&self, parallel_idx: NodeIndex) -> Vec<Vec<NodeIndex>> {
        // Collect all direct outgoing edges
        // Group by branch identifier
        // Return vector of node ID vectors
    }
}
```

## Consequences

### Positive

- Proven algorithm implementations (toposort)
- Type-safe graph operations
- Easy serialization with serde
- Rich ecosystem of graph algorithms if needed
- Mature, well-tested code

### Negative

- Additional dependency
- petgraph is generic (some ergonomics trade-offs)

### Future Considerations

- Consider `petgraph::algo::dominators` for optimization
- Consider `petgraph::graphviz` for visualization debugging
