//! messages.rs - Actor message types for wtf-actor
//!
//! This module defines the message types used by actors in the orchestration hierarchy.
//! Per ADR-006, the two-level hierarchy uses OrchestratorMsg (to MasterOrchestrator)
//! and InstanceMsg (to WorkflowInstance).

use ractor::RpcReplyPort;
use serde::{Deserialize, Serialize};

/// Message type for WorkflowInstance actor communication.
/// Defined in ADR-006.
#[derive(Debug)]
pub enum InstanceMsg {
    /// Execute a specific step in the workflow
    ExecuteStep {
        step_id: u32,
        reply: RpcReplyPort<Result<StepOutput, StepError>>,
    },
    /// Handle an external signal
    Signal {
        signal_name: String,
        payload: Vec<u8>,
        reply: RpcReplyPort<Result<(), SignalError>>,
    },
    /// Get current instance status
    GetStatus(RpcReplyPort<Option<InstanceStatus>>),
    /// Get the journal of executed steps
    GetJournal(RpcReplyPort<Vec<JournalEntry>>),
    /// Signal workflow completion with output
    Complete { output: Vec<u8> },
    /// Signal workflow failure
    Fail { error: String },
}

/// Placeholder for step output - actual type to be defined in workflow execution bead
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepOutput {
    pub step_id: u32,
    pub result: Vec<u8>,
}

/// Error type for step execution
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum StepError {
    #[error("step {step_id} not found")]
    StepNotFound { step_id: u32 },
    #[error("step execution failed: {reason}")]
    ExecutionFailed { reason: String },
}

/// Error type for signal handling
#[derive(Debug, Clone, thiserror::Error)]
pub enum SignalError {
    #[error("instance not found")]
    InstanceNotFound,
    #[error("invalid signal: {signal_name}")]
    InvalidSignal { signal_name: String },
}

/// Current status of a workflow instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Terminated,
}

/// A journal entry recording a workflow step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub step_id: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub input: Vec<u8>,
    pub output: Vec<u8>,
}

/// OrchestratorMsg - Message type for MasterOrchestrator.
///
/// This is the full enum per ADR-006, though bead wtf-5et only implements
/// the struct definitions. The message handling will be implemented in
/// subsequent beads.
#[derive(Debug)]
pub enum OrchestratorMsg {
    /// Start a new workflow
    StartWorkflow {
        name: String,
        input: Vec<u8>,
        reply: RpcReplyPort<Result<String, StartError>>,
    },
    /// Get status of a running workflow
    GetStatus {
        invocation_id: String,
        reply: RpcReplyPort<Option<InstanceStatus>>,
    },
    /// Send a signal to a workflow
    Signal {
        invocation_id: String,
        signal_name: String,
        payload: Vec<u8>,
        reply: RpcReplyPort<Result<(), SignalError>>,
    },
    /// List all running workflows
    ListWorkflows {
        reply: RpcReplyPort<Vec<WorkflowInfo>>,
    },
    /// Terminate a running workflow
    Terminate {
        invocation_id: String,
        reply: RpcReplyPort<Result<(), TerminateError>>,
    },
    /// Get the journal for a workflow invocation
    GetJournal {
        invocation_id: String,
        reply: RpcReplyPort<Option<Vec<JournalEntry>>>,
    },
}

/// Error type for workflow start failures
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum StartError {
    #[error("orchestrator is at capacity ({running}/{max})")]
    AtCapacity { running: usize, max: usize },
    #[error("workflow name is empty")]
    EmptyWorkflowName,
    #[error("failed to spawn workflow instance")]
    SpawnFailed,
}

/// Information about a running workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInfo {
    pub invocation_id: String,
    pub name: String,
    pub status: InstanceStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

/// Error type for workflow termination
#[derive(Debug, Clone, thiserror::Error)]
pub enum TerminateError {
    #[error("instance not found: {invocation_id}")]
    InstanceNotFound { invocation_id: String },
    #[error("termination failed: {reason}")]
    TerminationFailed { reason: String },
}

/// Response type for terminate operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerminateResponse {
    Success,
    NotFound,
}
