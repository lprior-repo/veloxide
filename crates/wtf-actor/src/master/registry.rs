use crate::procedural::WorkflowFn;
use std::collections::HashMap;
use std::sync::Arc;
pub use wtf_common::{WorkflowDefinition, WorkflowParadigm};

/// Registry of available workflows known to this orchestrator node.
#[derive(Debug, Default)]
pub struct WorkflowRegistry {
    /// In-memory procedural workflows.
    procedural: HashMap<String, Arc<dyn WorkflowFn>>,
    /// Definitions for FSM/DAG paradigms.
    definitions: HashMap<String, WorkflowDefinition>,
}

impl WorkflowRegistry {
    /// Create a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a procedural workflow.
    pub fn register_procedural(&mut self, name: &str, workflow_fn: Arc<dyn WorkflowFn>) {
        self.procedural.insert(name.to_owned(), workflow_fn);
    }

    /// Register an FSM or DAG definition.
    pub fn register_definition(&mut self, name: &str, definition: WorkflowDefinition) {
        self.definitions.insert(name.to_owned(), definition);
    }

    /// Look up a procedural workflow by type name.
    #[must_use]
    pub fn get_procedural(&self, name: &str) -> Option<Arc<dyn WorkflowFn>> {
        self.procedural.get(name).cloned()
    }

    /// Look up an FSM or DAG definition by type name.
    #[must_use]
    pub fn get_definition(&self, name: &str) -> Option<WorkflowDefinition> {
        self.definitions.get(name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::{WorkflowDefinition, WorkflowParadigm, WorkflowRegistry};

    fn make_test_definition(paradigm: WorkflowParadigm) -> WorkflowDefinition {
        WorkflowDefinition {
            paradigm,
            graph_raw: r#"{"nodes":[],"edges":[]}"#.to_owned(),
            description: None,
        }
    }

    #[test]
    fn new_registry_is_empty() {
        let registry = WorkflowRegistry::new();
        assert!(registry.get_definition("anything").is_none());
    }

    #[test]
    fn register_and_get_definition() {
        let mut registry = WorkflowRegistry::new();
        let def = make_test_definition(WorkflowParadigm::Fsm);
        registry.register_definition("checkout", def.clone());
        let retrieved = registry.get_definition("checkout");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().paradigm, WorkflowParadigm::Fsm);
    }

    #[test]
    fn get_nonexistent_definition_returns_none() {
        let registry = WorkflowRegistry::new();
        assert!(registry.get_definition("nonexistent").is_none());
    }

    #[test]
    fn register_multiple_definitions() {
        let mut registry = WorkflowRegistry::new();
        registry.register_definition("w1", make_test_definition(WorkflowParadigm::Fsm));
        registry.register_definition("w2", make_test_definition(WorkflowParadigm::Dag));
        assert!(registry.get_definition("w1").is_some());
        assert!(registry.get_definition("w2").is_some());
        assert!(registry.get_definition("w3").is_none());
    }

    #[test]
    fn definition_keys_are_case_sensitive() {
        let mut registry = WorkflowRegistry::new();
        registry.register_definition("Checkout", make_test_definition(WorkflowParadigm::Fsm));
        assert!(registry.get_definition("Checkout").is_some());
        assert!(registry.get_definition("checkout").is_none());
    }

    #[test]
    fn replacing_definition_overwrites() {
        let mut registry = WorkflowRegistry::new();
        registry.register_definition("wf", make_test_definition(WorkflowParadigm::Fsm));
        let dag_def = make_test_definition(WorkflowParadigm::Dag);
        registry.register_definition("wf", dag_def);
        let retrieved = registry.get_definition("wf").unwrap();
        assert_eq!(retrieved.paradigm, WorkflowParadigm::Dag);
    }
}
