//! master.rs - MasterOrchestrator actor and OrchestratorState

use std::collections::HashMap;
use std::sync::Arc;

use ractor::{Actor, ActorRef, ActorProcessingErr, RpcReplyPort};
use sled::Db;
use ulid::Ulid;

use crate::instance::WorkflowInstance;
use crate::messages::{InstanceMsg, InstanceStatus, JournalEntry, OrchestratorMsg, SignalError, StartError, TerminateError, WorkflowInfo};
use wtf_core::InstanceConfig;

/// Error types for MasterOrchestrator initialization and operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("max_concurrent must be > 0, got 0")]
    InvalidCapacity,

    #[error("storage is not available")]
    StorageUnavailable,

    #[error("failed to initialize orchestrator state")]
    StateInitializationFailed,
}

/// MasterOrchestrator manages workflow instances with capacity enforcement.
///
/// Per ADR-006:
/// - max_concurrent: usize (capacity limit)
/// - storage: Arc<sled::Db> (journal persistence)
pub struct MasterOrchestrator {
    /// Maximum number of concurrent workflow instances allowed
    max_concurrent: usize,
    /// Persistent storage for journal and state
    storage: Arc<Db>,
}

impl MasterOrchestrator {
    /// Creates a new MasterOrchestrator with the specified capacity and storage.
    ///
    /// # Errors
    /// Returns `Error::InvalidCapacity` if `max_concurrent` is 0.
    pub fn new(max_concurrent: usize, storage: Arc<Db>) -> Result<Self, Error> {
        if max_concurrent == 0 {
            return Err(Error::InvalidCapacity);
        }
        Ok(Self {
            max_concurrent,
            storage,
        })
    }

    /// Returns the maximum concurrent workflow capacity.
    #[inline]
    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    /// Returns a reference to the persistent storage.
    #[inline]
    pub fn storage(&self) -> &Arc<Db> {
        &self.storage
    }

    /// Validates a workflow name is non-empty.
    ///
    /// # Errors
    /// Returns `StartError::EmptyWorkflowName` if the name is empty.
    #[inline]
    pub fn validate_workflow_name(name: &str) -> Result<(), StartError> {
        if name.is_empty() {
            Err(StartError::EmptyWorkflowName)
        } else {
            Ok(())
        }
    }

    /// Checks if the orchestrator is under capacity.
    ///
    /// Returns true if `running_count < max_concurrent`.
    #[inline]
    pub fn capacity_check(&self, state: &OrchestratorState) -> bool {
        state.running_count < self.max_concurrent
    }

    /// Generates a new ULID-based invocation ID.
    #[inline]
    fn generate_invocation_id() -> String {
        Ulid::new().to_string()
    }

    /// Spawns a new workflow instance actor.
    ///
    /// Creates a linked `WorkflowInstance` actor and registers it in the state.
    async fn spawn_workflow(
        &self,
        myself: ActorRef<OrchestratorMsg>,
        state: &mut OrchestratorState,
        name: String,
        invocation_id: String,
        input: Vec<u8>,
    ) -> Result<ActorRef<InstanceMsg>, StartError> {
        // Create the workflow instance actor
        let instance = WorkflowInstance::new(name.clone());

        // Create the instance config
        let config = InstanceConfig {
            invocation_id: invocation_id.clone(),
            input,
            storage: self.storage.clone(),
        };

        // Spawn the actor linked to this orchestrator
        let actor_name = format!("{}:{}", name, invocation_id);
        let (actor_ref, _handle) = Actor::spawn_linked(
            Some(actor_name),
            instance,
            config,
            myself.clone().into(),
        )
        .await
        .map_err(|_| StartError::SpawnFailed)?;

        // Register the instance in state
        state.instances.insert(
            invocation_id.clone(),
            (name, actor_ref.clone()),
        );

        Ok(actor_ref)
    }

    /// Handles a StartWorkflow message.
    ///
    /// Per the contract:
    /// 1. Check capacity with capacity_check()
    /// 2. If at capacity, reply with StartError::AtCapacity
    /// 3. Validate workflow name is non-empty
    /// 4. Generate invocation_id (ULID-based)
    /// 5. Call spawn_workflow to create actor
    /// 6. Increment state.running_count
    /// 7. Reply with invocation_id
    /// 8. Return Ok(()) after handling
    async fn handle_start_workflow(
        &self,
        myself: ActorRef<OrchestratorMsg>,
        state: &mut OrchestratorState,
        name: String,
        input: Vec<u8>,
        reply: RpcReplyPort<Result<String, StartError>>,
    ) -> Result<(), ActorProcessingErr> {
        // Step 1: Capacity check
        if !self.capacity_check(state) {
            let _ = reply.send(Err(StartError::AtCapacity {
                running: state.running_count,
                max: self.max_concurrent,
            }));
            return Ok(());
        }

        // Step 2: Validate workflow name
        if let Err(e) = Self::validate_workflow_name(&name) {
            let _ = reply.send(Err(e));
            return Ok(());
        }

        // Step 3: Generate invocation_id
        let invocation_id = Self::generate_invocation_id();

        // Step 4 & 5: Spawn workflow
        match self
            .spawn_workflow(myself, state, name.clone(), invocation_id.clone(), input)
            .await
        {
            Ok(_actor_ref) => {
                // Step 6: Increment running count
                state.running_count += 1;

                // Step 7: Reply with invocation_id
                let _ = reply.send(Ok(invocation_id));
            }
            Err(e) => {
                // Spawn failed - propagate error without incrementing count
                let _ = reply.send(Err(e));
            }
        }

        // Step 8: Return Ok(())
        Ok(())
    }

    /// Handles a Terminate message.
    ///
    /// Terminates a running workflow by invocation_id.
    async fn handle_terminate(
        &self,
        state: &mut OrchestratorState,
        invocation_id: String,
        reply: RpcReplyPort<Result<(), TerminateError>>,
    ) -> Result<(), ActorProcessingErr> {
        // Check if the instance exists
        match state.instances.remove(&invocation_id) {
            Some((name, actor_ref)) => {
                // Decrement running count
                state.running_count = state.running_count.saturating_sub(1);

                // Send terminate signal to the instance (fire-and-forget)
                let _ = actor_ref.send_message(InstanceMsg::Fail {
                    error: "Terminated by orchestrator".to_string(),
                });

                tracing::info!(
                    invocation_id = %invocation_id,
                    workflow_name = %name,
                    "Workflow terminated"
                );
                let _ = reply.send(Ok(()));
            }
            None => {
                tracing::warn!(invocation_id = %invocation_id, "Workflow not found for termination");
                let _ = reply.send(Err(TerminateError::InstanceNotFound {
                    invocation_id,
                }));
            }
        }
        Ok(())
    }

    /// Handles a ListWorkflows message.
    ///
    /// Returns information about all running workflows.
    async fn handle_list_workflows(
        &self,
        state: &OrchestratorState,
        reply: RpcReplyPort<Vec<WorkflowInfo>>,
    ) -> Result<(), ActorProcessingErr> {
        use chrono::Utc;

        let workflows: Vec<WorkflowInfo> = state
            .instances
            .iter()
            .map(|(invocation_id, (name, _))| WorkflowInfo {
                invocation_id: invocation_id.clone(),
                name: name.clone(),
                status: InstanceStatus::Running,
                started_at: Utc::now(),
            })
            .collect();

        let _ = reply.send(workflows);
        Ok(())
    }

    /// Handles a GetStatus message.
    ///
    /// Returns the status of a workflow by invocation_id.
    async fn handle_get_status(
        &self,
        state: &OrchestratorState,
        invocation_id: String,
        reply: RpcReplyPort<Option<InstanceStatus>>,
    ) -> Result<(), ActorProcessingErr> {
        let status = if state.instances.contains_key(&invocation_id) {
            Some(InstanceStatus::Running)
        } else {
            None
        };

        let _ = reply.send(status);
        Ok(())
    }

    /// Handles a GetJournal message.
    ///
    /// Returns the journal entries for a workflow by invocation_id.
    async fn handle_get_journal(
        &self,
        state: &OrchestratorState,
        invocation_id: String,
        reply: RpcReplyPort<Option<Vec<JournalEntry>>>,
    ) -> Result<(), ActorProcessingErr> {
        // Check if the instance exists
        if state.instances.contains_key(&invocation_id) {
            // For now, return an empty journal - actual journal would come from instance
            let _ = reply.send(Some(Vec::new()));
        } else {
            let _ = reply.send(None);
        }
        Ok(())
    }

    /// Handles a Signal message.
    ///
    /// Forwards a signal to a workflow instance.
    async fn handle_signal(
        &self,
        state: &OrchestratorState,
        invocation_id: String,
        signal_name: String,
        payload: Vec<u8>,
        reply: RpcReplyPort<Result<(), SignalError>>,
    ) -> Result<(), ActorProcessingErr> {
        match state.instances.get(&invocation_id) {
            Some((_, actor_ref)) => {
                // Forward the signal to the instance
                let _ = actor_ref.send_message(InstanceMsg::Signal {
                    signal_name,
                    payload,
                    reply,
                });
            }
            None => {
                let _ = reply.send(Err(SignalError::InstanceNotFound));
            }
        }
        Ok(())
    }
}

/// OrchestratorState maintains the registry of running workflow instances.
///
/// Per ADR-006:
/// - instances: HashMap<invocation_id, (workflow_name, actor_ref)>
/// - running_count: usize
pub struct OrchestratorState {
    /// Registry of active workflow instances: invocation_id -> (workflow_name, actor_ref)
    instances: HashMap<String, (String, ActorRef<InstanceMsg>)>,
    /// Number of currently running workflow instances
    running_count: usize,
}

impl OrchestratorState {
    /// Creates a new OrchestratorState with empty registry and zero running count.
    #[inline]
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
            running_count: 0,
        }
    }

    /// Creates a new OrchestratorState with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            instances: HashMap::with_capacity(capacity),
            running_count: 0,
        }
    }

    /// Returns the number of active workflow instances.
    #[inline]
    pub fn running_count(&self) -> usize {
        self.running_count
    }

    /// Returns the number of registered workflow instances.
    #[inline]
    pub fn instances_len(&self) -> usize {
        self.instances.len()
    }

    /// Returns true if there are no running workflow instances.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    /// Returns a reference to the instances registry.
    #[inline]
    pub fn instances(&self) -> &HashMap<String, (String, ActorRef<InstanceMsg>)> {
        &self.instances
    }
}

impl Default for OrchestratorState {
    fn default() -> Self {
        Self::new()
    }
}

/// Actor implementation for MasterOrchestrator.
///
/// Per ADR-006, the MasterOrchestrator:
/// - Is the root supervisor in the actor hierarchy
/// - Initializes state via pre_start with empty registry and 0 running count
#[ractor::async_trait]
impl Actor for MasterOrchestrator {
    type Msg = OrchestratorMsg;
    type State = OrchestratorState;
    type Arguments = ();

    /// Initialize the actor state.
    ///
    /// Returns an empty OrchestratorState with:
    /// - Empty instances HashMap
    /// - running_count = 0
    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(OrchestratorState::new())
    }

    /// Handle incoming messages.
    ///
    /// Dispatches to the appropriate handler based on message variant.
    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            OrchestratorMsg::StartWorkflow { name, input, reply } => {
                self.handle_start_workflow(_myself, state, name, input, reply).await?;
            }
            OrchestratorMsg::Terminate { invocation_id, reply } => {
                self.handle_terminate(state, invocation_id, reply).await?;
            }
            OrchestratorMsg::ListWorkflows { reply } => {
                self.handle_list_workflows(state, reply).await?;
            }
            OrchestratorMsg::GetStatus { invocation_id, reply } => {
                self.handle_get_status(state, invocation_id, reply).await?;
            }
            OrchestratorMsg::GetJournal { invocation_id, reply } => {
                self.handle_get_journal(state, invocation_id, reply).await?;
            }
            OrchestratorMsg::Signal { invocation_id, signal_name, payload, reply } => {
                self.handle_signal(state, invocation_id, signal_name, payload, reply).await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_state_new_creates_empty_instances() {
        let state = OrchestratorState::new();
        assert_eq!(state.instances.len(), 0);
    }

    #[test]
    fn test_orchestrator_state_new_sets_running_count_to_zero() {
        let state = OrchestratorState::new();
        assert_eq!(state.running_count, 0);
    }

    #[test]
    fn test_orchestrator_state_default_is_consistent() {
        let state1 = OrchestratorState::new();
        let state2 = OrchestratorState::default();
        assert_eq!(state1.instances.len(), state2.instances.len());
        assert_eq!(state1.running_count, state2.running_count);
    }

    #[test]
    fn test_master_orchestrator_new_rejects_zero_capacity() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = sled::open(temp_dir.path()).expect("sled db");
        let storage = Arc::new(db);

        let result = MasterOrchestrator::new(0, storage);
        assert!(result.is_err());
        if let Err(Error::InvalidCapacity) = result {
            // Expected error type
        } else {
            panic!("Expected Error::InvalidCapacity");
        }
    }

    #[test]
    fn test_master_orchestrator_new_accepts_minimal_capacity() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = sled::open(temp_dir.path()).expect("sled db");
        let storage = Arc::new(db);

        let result = MasterOrchestrator::new(1, storage);
        assert!(result.is_ok());
        let orchestrator = result.unwrap();
        assert_eq!(orchestrator.max_concurrent(), 1);
    }

    #[test]
    fn test_invariant_running_count_initially_zero() {
        let state = OrchestratorState::new();
        let max_concurrent = 3; // hypothetical max
        assert!(state.running_count <= max_concurrent);
    }

    #[test]
    fn test_invariant_instances_keys_non_empty_after_init() {
        let state = OrchestratorState::new();
        // After initialization, instances is empty, so the invariant vacuously holds
        // This test documents that the invariant must be maintained when instances are added
        assert!(state.instances.keys().all(|k| !k.is_empty()) || state.instances.is_empty());
    }
}
