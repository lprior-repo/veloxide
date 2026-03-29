// Stub file
pub mod messages {
    #[derive(Debug)]
    pub enum TerminateError {
        NotFound(String),
        Failed(String),
    }

    #[derive(Debug)]
    pub enum WorkflowParadigm {
        Default,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum InstancePhaseView {
        Replay,
        Live,
    }
}

pub mod heartbeat {
    pub fn run_heartbeat_watcher() {}
}

pub mod master {
    pub struct MasterOrchestrator;
    pub struct OrchestratorConfig;
}

#[derive(Debug)]
pub struct OrchestratorMsg;

#[derive(Debug)]
pub struct StartError;
