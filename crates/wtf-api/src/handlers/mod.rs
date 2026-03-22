//! HTTP handlers for wtf-api v3 endpoints.

pub mod definitions;
pub mod events;
pub mod signal;
pub mod workflow;

pub use definitions::*;
pub use events::*;
pub use signal::*;
pub use workflow::*;

use std::time::Duration;
use wtf_actor::{messages::WorkflowParadigm, OrchestratorMsg};
use wtf_common::InstanceId;
use ractor::{ActorRef, rpc::CallResult};

/// Timeout for all actor RPC calls from HTTP handlers.
pub const ACTOR_CALL_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn get_nats(master: &ActorRef<OrchestratorMsg>) -> Option<wtf_storage::NatsClient> {
    master.call(|tx| OrchestratorMsg::GetNatsContext { reply: tx }, Some(ACTOR_CALL_TIMEOUT)).await.ok().and_then(|r| match r { CallResult::Success(s) => s, _ => None })
}

pub async fn get_db(master: &ActorRef<OrchestratorMsg>) -> Option<sled::Db> {
    master.call(|tx| OrchestratorMsg::GetSnapshotDb { reply: tx }, Some(ACTOR_CALL_TIMEOUT)).await.ok().and_then(|r| match r { CallResult::Success(s) => s, _ => None })
}

/// Split a path `<namespace>/<instance_id>` into the two parts.
pub(crate) fn split_path_id(path: &str) -> Option<(String, InstanceId)> {
    let slash = path.find('/')?;
    let (ns, id) = path.split_at(slash);
    Some((ns.to_owned(), InstanceId::new(id[1..].to_owned())))
}

pub(crate) fn parse_paradigm(s: &str) -> Option<WorkflowParadigm> {
    match s {
        "fsm" => Some(WorkflowParadigm::Fsm),
        "dag" => Some(WorkflowParadigm::Dag),
        "procedural" => Some(WorkflowParadigm::Procedural),
        _ => None,
    }
}

pub(crate) fn paradigm_to_str(p: WorkflowParadigm) -> &'static str {
    match p {
        WorkflowParadigm::Fsm => "fsm",
        WorkflowParadigm::Dag => "dag",
        WorkflowParadigm::Procedural => "procedural",
    }
}

pub(crate) fn phase_to_str(p: wtf_actor::messages::InstancePhaseView) -> &'static str {
    match p {
        wtf_actor::messages::InstancePhaseView::Replay => "replay",
        wtf_actor::messages::InstancePhaseView::Live => "live",
    }
}
