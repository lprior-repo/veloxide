use crate::master::registry::WorkflowRegistry;
use crate::messages::InstanceMsg;
use ractor::ActorRef;
use std::collections::HashMap;
use wtf_common::InstanceId;
use wtf_storage::NatsClient;

/// Configuration for the MasterOrchestrator.
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Maximum number of workflow instances this node may run concurrently.
    pub max_instances: usize,
    /// Unique identifier for this engine node.
    pub engine_node_id: String,
    /// NATS client for JetStream and KV operations.
    pub nats: Option<NatsClient>,
    /// Sled database handle for snapshot storage.
    pub snapshot_db: Option<sled::Db>,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_instances: 1000,
            engine_node_id: "engine-local".into(),
            nats: None,
            snapshot_db: None,
        }
    }
}

/// In-memory state of the MasterOrchestrator.
#[derive(Debug)]
pub struct OrchestratorState {
    /// Registry of all currently active workflow instances.
    pub active: HashMap<InstanceId, ActorRef<InstanceMsg>>,
    /// Configuration (immutable after construction).
    pub config: OrchestratorConfig,
    /// Registry of available workflows.
    pub registry: WorkflowRegistry,
}

impl OrchestratorState {
    /// Create a new empty orchestrator state.
    #[must_use]
    pub fn new(config: OrchestratorConfig) -> Self {
        Self {
            active: HashMap::new(),
            config,
            registry: WorkflowRegistry::new(),
        }
    }

    /// Return the number of currently active instances.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    /// Return `true` if the orchestrator can accept one more instance.
    #[must_use]
    pub fn has_capacity(&self) -> bool {
        self.active.len() < self.config.max_instances
    }

    /// Register a newly spawned instance.
    pub fn register(&mut self, id: InstanceId, actor_ref: ActorRef<InstanceMsg>) {
        self.active.insert(id, actor_ref);
    }

    /// Deregister a stopped instance.
    pub fn deregister(&mut self, id: &InstanceId) {
        self.active.remove(id);
    }

    /// Look up an active instance by ID.
    #[must_use]
    pub fn get(&self, id: &InstanceId) -> Option<&ActorRef<InstanceMsg>> {
        self.active.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> OrchestratorConfig {
        OrchestratorConfig {
            max_instances: 10,
            engine_node_id: "node-test".into(),
            nats: None,
            snapshot_db: None,
        }
    }

    #[test]
    fn new_state_is_empty() {
        let state = OrchestratorState::new(test_config());
        assert_eq!(state.active_count(), 0);
    }

    #[test]
    fn has_capacity_when_empty() {
        let state = OrchestratorState::new(test_config());
        assert!(state.has_capacity());
    }

    #[test]
    fn has_capacity_false_when_at_limit() {
        let config = OrchestratorConfig {
            max_instances: 0,
            engine_node_id: "node".into(),
            nats: None,
            snapshot_db: None,
        };
        let state = OrchestratorState::new(config);
        assert!(!state.has_capacity());
    }

    #[test]
    fn get_returns_none_for_unknown_id() {
        let state = OrchestratorState::new(test_config());
        let id = InstanceId::new("unknown");
        assert!(state.get(&id).is_none());
    }

    #[test]
    fn deregister_removes_entry() {
        let mut state = OrchestratorState::new(test_config());
        let id = InstanceId::new("not-there");
        state.deregister(&id); // should not panic
        assert_eq!(state.active_count(), 0);
    }

    #[test]
    fn orchestrator_config_default_max_instances() {
        let cfg = OrchestratorConfig::default();
        assert_eq!(cfg.max_instances, 1000);
    }

    #[test]
    fn orchestrator_config_default_node_id() {
        let cfg = OrchestratorConfig::default();
        assert_eq!(cfg.engine_node_id, "engine-local");
    }
}
