//! instance.rs - WorkflowInstance actor
//!
//! Per ADR-006, WorkflowInstance is the per-workflow actor that:
//! - Owns workflow DAG (petgraph)
//! - Owns journal cursor
//! - Executes steps
//! - Reports to MasterOrchestrator

use ractor::{Actor, ActorRef, ActorProcessingErr};

use crate::messages::InstanceMsg;
use wtf_core::InstanceConfig;

/// WorkflowInstance is the per-workflow actor that manages execution.
///
/// Per ADR-006:
/// - Owns workflow DAG (petgraph)
/// - Owns journal cursor  
/// - Executes steps
/// - Reports to MasterOrchestrator
pub struct WorkflowInstance {
    workflow_name: String,
}

impl WorkflowInstance {
    /// Creates a new WorkflowInstance.
    pub fn new(workflow_name: String) -> Self {
        Self { workflow_name }
    }
}

/// InstanceState for WorkflowInstance actor.
#[derive(Debug, Clone)]
pub struct InstanceState {
    pub workflow_name: String,
    pub invocation_id: String,
}

/// Actor implementation for WorkflowInstance.
///
/// This is a minimal implementation - full step execution will be in subsequent beads.
#[ractor::async_trait]
impl Actor for WorkflowInstance {
    type Msg = InstanceMsg;
    type State = InstanceState;
    type Arguments = InstanceConfig;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(InstanceState {
            workflow_name: self.workflow_name.clone(),
            invocation_id: args.invocation_id,
        })
    }
}
