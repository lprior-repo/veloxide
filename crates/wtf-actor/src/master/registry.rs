pub use crate::messages::{WorkflowDefinition, WorkflowParadigm};
use crate::procedural::WorkflowFn;
use std::collections::HashMap;
use std::sync::Arc;

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
