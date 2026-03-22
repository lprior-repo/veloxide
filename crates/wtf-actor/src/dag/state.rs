//! DAG paradigm state types.

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use wtf_common::ActivityId;

/// A node in the workflow DAG.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DagNode {
    pub activity_type: String,
    pub predecessors: Vec<NodeId>,
}

/// Stable, author-assigned identifier for a node in the workflow DAG.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&ActivityId> for NodeId {
    fn from(id: &ActivityId) -> Self {
        Self(id.as_str().to_owned())
    }
}

/// In-memory state for a DAG workflow actor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagActorState {
    pub nodes: HashMap<NodeId, DagNode>,
    pub completed: HashMap<NodeId, Bytes>,
    pub in_flight: HashSet<NodeId>,
    pub failed: HashSet<NodeId>,
    pub applied_seq: HashSet<u64>,
    pub events_since_snapshot: u32,
}

impl DagActorState {
    #[must_use]
    pub fn new(nodes: HashMap<NodeId, DagNode>) -> Self {
        Self {
            nodes,
            completed: HashMap::new(),
            in_flight: HashSet::new(),
            failed: HashSet::new(),
            applied_seq: HashSet::new(),
            events_since_snapshot: 0,
        }
    }
}
