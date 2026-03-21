//! types.rs - Core type definitions for wtf-core

use std::sync::Arc;

// Placeholder - actual types to be defined in subsequent beads
pub struct WorkflowGraph;

// InstanceConfig holds runtime configuration for a workflow instance
pub struct InstanceConfig {
    pub invocation_id: String,
    pub input: Vec<u8>,
    pub storage: Arc<sled::Db>,
}

pub struct JournalCursor(pub u32);
