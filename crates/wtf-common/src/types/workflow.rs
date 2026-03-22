//! Workflow paradigm and metadata types.

use crate::{InstanceId, NamespaceId};
use serde::{Deserialize, Serialize};

/// The three execution paradigms supported by wtf-engine (ADR-017).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowParadigm {
    /// State Machine — transitions recorded as `TransitionApplied` events.
    Fsm,
    /// Directed Acyclic Graph — activities dispatched by dependency order.
    Dag,
    /// Arbitrary async Rust code with checkpoint-based determinism.
    Procedural,
}

/// Serialized form of a workflow graph for FSM or DAG paradigms.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowDefinition {
    /// Paradigm this definition implements.
    pub paradigm: WorkflowParadigm,
    /// JSON or YAML representation of the graph (nodes/edges or transitions).
    pub graph_raw: String,
    /// Human-readable description (optional).
    pub description: Option<String>,
}

/// Metadata stored in `wtf-instances` KV for each running instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceMetadata {
    pub namespace: NamespaceId,
    pub instance_id: InstanceId,
    pub workflow_type: String,
    pub paradigm: WorkflowParadigm,
    pub engine_node_id: String,
}
