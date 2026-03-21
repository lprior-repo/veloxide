//! master.rs - MasterOrchestrator actor and OrchestratorState

use std::collections::HashMap;
use std::sync::Arc;

use ractor::{Actor, ActorRef, ActorProcessingErr};
use sled::Db;

use crate::messages::InstanceMsg;

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
    type Msg = crate::messages::OrchestratorMsg;
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
        assert!(state.instances.keys().all(|k| !k.is_empty()) || state.instances.is_empty());
    }
}
