use axum::{extract::{Extension, Path}, http::StatusCode, response::{IntoResponse, Response}, Json};
use bytes::Bytes;
use ractor::rpc::CallResult;
use ractor::{ActorRef, MessagingErr};
use wtf_actor::{messages::WorkflowParadigm, OrchestratorMsg, StartError};
use wtf_common::{InstanceId, NamespaceId};
use crate::types::{ApiError, V3StartRequest, V3StartResponse, V3StatusResponse};
use super::{ACTOR_CALL_TIMEOUT, split_path_id, parse_paradigm, paradigm_to_str, phase_to_str, get_nats, get_db};
use tokio_stream::StreamExt;

/// POST /api/v1/workflows — start a new workflow instance.
pub async fn start_workflow(
    Extension(master): Extension<ActorRef<OrchestratorMsg>>,
    Json(req): Json<V3StartRequest>,
) -> impl IntoResponse {
    let (ns, id, p, input) = match validate_start_req(&req) {
        Ok(v) => v,
        Err(e) => return e.into_response(),
    };
    let res = master.call(|tx| OrchestratorMsg::StartWorkflow {
        namespace: ns, instance_id: id, workflow_type: req.workflow_type.clone(), paradigm: p, input, reply: tx,
    }, Some(ACTOR_CALL_TIMEOUT)).await;
    map_start_result(res, req.workflow_type)
}

/// GET /api/v1/workflows/:id — get instance status.
pub async fn get_workflow(
    Extension(master): Extension<ActorRef<OrchestratorMsg>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let (_, inst_id) = match split_path_id(&id) {
        Some(p) => p,
        None => return (StatusCode::BAD_REQUEST, Json(ApiError::new("invalid_id", "bad id"))).into_response(),
    };
    let res = master.call(|tx| OrchestratorMsg::GetStatus { instance_id: inst_id, reply: tx }, Some(ACTOR_CALL_TIMEOUT)).await;
    map_status_result(res, id)
}

/// DELETE /api/v1/workflows/:id — terminate a running instance.
pub async fn terminate_workflow(
    Extension(master): Extension<ActorRef<OrchestratorMsg>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let (_, inst_id) = match split_path_id(&id) {
        Some(p) => p,
        None => return (StatusCode::BAD_REQUEST, Json(ApiError::new("invalid_id", "bad id"))).into_response(),
    };
    let res = master.call(|tx| OrchestratorMsg::Terminate {
        instance_id: inst_id, reason: "api-terminate".to_owned(), reply: tx,
    }, Some(ACTOR_CALL_TIMEOUT)).await;
    map_terminate_result(res)
}

/// GET /api/v1/workflows — list all active workflow instances.
pub async fn list_workflows(
    Extension(master): Extension<ActorRef<OrchestratorMsg>>,
) -> impl IntoResponse {
    let res = master.call(|tx| OrchestratorMsg::ListActive { reply: tx }, Some(ACTOR_CALL_TIMEOUT)).await;
    match res {
        Ok(CallResult::Success(snapshots)) => (StatusCode::OK, Json(snapshots.into_iter().map(V3StatusResponse::from).collect::<Vec<_>>())).into_response(),
        _ => map_actor_error(res).into_response(),
    }
}

/// GET /api/v1/instances/:id/replay-to/:seq — replay instance to a specific sequence.
pub async fn replay_to(
    Extension(master): Extension<ActorRef<OrchestratorMsg>>,
    Path((id, seq)): Path<(String, u64)>,
) -> impl IntoResponse {
    let (_, inst_id) = match split_path_id(&id) {
        Some(p) => p,
        None => return (StatusCode::BAD_REQUEST, Json(ApiError::new("invalid_id", "bad id"))).into_response(),
    };
    let nats = match get_nats(&master).await { Some(n) => n, None => return (StatusCode::SERVICE_UNAVAILABLE, "no_nats").into_response() };
    let db = match get_db(&master).await { Some(d) => d, None => return (StatusCode::SERVICE_UNAVAILABLE, "no_db").into_response() };
    match do_replay_to(nats, db, inst_id, seq).await {
        Ok(state) => (StatusCode::OK, Json(state)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::new("replay_error", e.to_string()))).into_response(),
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn validate_start_req(req: &V3StartRequest) -> Result<(NamespaceId, InstanceId, WorkflowParadigm, Bytes), (StatusCode, Json<ApiError>)> {
    let ns = NamespaceId::try_new(&req.namespace).map_err(|_| (StatusCode::BAD_REQUEST, Json(ApiError::new("invalid_namespace", "bad namespace"))))?;
    let p = parse_paradigm(&req.paradigm).ok_or_else(|| (StatusCode::BAD_REQUEST, Json(ApiError::new("invalid_paradigm", "bad paradigm"))))?;
    let id = req.instance_id.as_ref().map_or(Ok(InstanceId::new(ulid::Ulid::new().to_string())), |s| InstanceId::try_new(s)).map_err(|_| (StatusCode::BAD_REQUEST, Json(ApiError::new("invalid_instance_id", "bad instance_id"))))?;
    let input = serde_json::to_vec(&req.input).map_err(|e| (StatusCode::BAD_REQUEST, Json(ApiError::new("invalid_input", e.to_string()))))?;
    Ok((ns, id, p, Bytes::from(input)))
}

fn map_start_result(res: Result<CallResult<Result<InstanceId, StartError>>, MessagingErr<OrchestratorMsg>>, wf_type: String) -> Response {
    match res {
        Ok(CallResult::Success(Ok(id))) => (StatusCode::CREATED, Json(V3StartResponse { instance_id: id.to_string(), namespace: "".to_owned(), workflow_type: wf_type })).into_response(),
        Ok(CallResult::Success(Err(e))) => map_start_error(e).into_response(),
        _ => map_actor_error(res).into_response(),
    }
}

fn map_start_error(err: StartError) -> impl IntoResponse {
    match err {
        StartError::AtCapacity { running, max } => (StatusCode::SERVICE_UNAVAILABLE, Json(ApiError::new("at_capacity", format!("{running}/{max}")))),
        StartError::AlreadyExists(id) => (StatusCode::CONFLICT, Json(ApiError::new("already_exists", id.to_string()))),
        StartError::SpawnFailed(msg) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::new("spawn_failed", msg))),
    }
}

fn map_status_result(res: Result<CallResult<Option<wtf_actor::messages::InstanceStatusSnapshot>>, MessagingErr<OrchestratorMsg>>, id: String) -> Response {
    match res {
        Ok(CallResult::Success(Some(s))) => (StatusCode::OK, Json(V3StatusResponse::from(s))).into_response(),
        Ok(CallResult::Success(None)) => (StatusCode::NOT_FOUND, Json(ApiError::new("not_found", id))).into_response(),
        _ => map_actor_error(res).into_response(),
    }
}

fn map_terminate_result(res: Result<CallResult<Result<(), wtf_actor::messages::TerminateError>>, MessagingErr<OrchestratorMsg>>) -> Response {
    match res {
        Ok(CallResult::Success(Ok(()))) => StatusCode::NO_CONTENT.into_response(),
        Ok(CallResult::Success(Err(wtf_actor::messages::TerminateError::NotFound(id)))) => (StatusCode::NOT_FOUND, Json(ApiError::new("not_found", id.to_string()))).into_response(),
        _ => map_actor_error(res).into_response(),
    }
}

fn map_actor_error<T>(res: Result<CallResult<T>, MessagingErr<OrchestratorMsg>>) -> impl IntoResponse {
    match res {
        Ok(CallResult::Timeout) => (StatusCode::SERVICE_UNAVAILABLE, Json(ApiError::new("actor_timeout", "timeout"))),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::new("actor_error", "actor failed"))),
    }
}

impl From<wtf_actor::messages::InstanceStatusSnapshot> for V3StatusResponse {
    fn from(s: wtf_actor::messages::InstanceStatusSnapshot) -> Self {
        Self {
            instance_id: s.instance_id.to_string(), namespace: s.namespace.to_string(), workflow_type: s.workflow_type,
            paradigm: paradigm_to_str(s.paradigm).to_owned(), phase: phase_to_str(s.phase).to_owned(), events_applied: s.events_applied,
        }
    }
}

async fn do_replay_to(nats: wtf_storage::NatsClient, db: sled::Db, id: InstanceId, target_seq: u64) -> Result<wtf_actor::instance::lifecycle::ParadigmState, anyhow::Error> {
    let ns = NamespaceId::new("default"); // FIXME: get from metadata
    let (mut p_state, from_seq) = load_snapshot(&db, &id).await?;
    let mut stream = Box::pin(wtf_storage::replay_events(nats.jetstream().clone(), ns, id, wtf_storage::ReplayConfig { from_seq, ..Default::default() }).await?);
    while let Some(res) = stream.next().await {
        let replayed = res?;
        if replayed.seq > target_seq { break; }
        p_state = p_state.apply_event(&replayed.event, replayed.seq, wtf_actor::messages::InstancePhase::Replay).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    }
    Ok(p_state)
}

async fn load_snapshot(db: &sled::Db, id: &InstanceId) -> Result<(wtf_actor::instance::lifecycle::ParadigmState, u64), anyhow::Error> {
    if let Ok(Some(snap)) = wtf_storage::read_snapshot(db, id) {
        let state = wtf_actor::instance::actor::deserialize_paradigm_state(WorkflowParadigm::Fsm, &snap.state_bytes).map_err(|e| anyhow::anyhow!(e.to_string()))?;
        return Ok((state, snap.seq + 1));
    }
    Ok((wtf_actor::instance::state::initialize_paradigm_state(&wtf_actor::messages::InstanceArguments {
        namespace: NamespaceId::new(""), instance_id: id.clone(), workflow_type: "".to_owned(), paradigm: WorkflowParadigm::Fsm,
        input: Bytes::new(), engine_node_id: "".to_owned(), nats: None, snapshot_db: None, procedural_workflow: None, workflow_definition: None,
    }), 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_path_id_valid() {
        let result = split_path_id("payments/01ARZ3NDEKTSV4RRFFQ69G5FAV");
        assert!(result.is_some());
        let (ns, id) = result.expect("some");
        assert_eq!(ns, "payments");
        assert_eq!(id.as_str(), "01ARZ3NDEKTSV4RRFFQ69G5FAV");
    }

    #[test]
    fn split_path_id_missing_slash_returns_none() {
        let result = split_path_id("no-slash-here");
        assert!(result.is_none());
    }

    #[test]
    fn split_path_id_multiple_slashes_splits_on_first() {
        let result = split_path_id("ns/id/extra");
        let (ns, id) = result.expect("some");
        assert_eq!(ns, "ns");
        assert_eq!(id.as_str(), "id/extra");
    }

    #[test]
    fn parse_paradigm_fsm() {
        assert_eq!(parse_paradigm("fsm"), Some(WorkflowParadigm::Fsm));
    }

    #[test]
    fn parse_paradigm_dag() {
        assert_eq!(parse_paradigm("dag"), Some(WorkflowParadigm::Dag));
    }

    #[test]
    fn parse_paradigm_procedural() {
        assert_eq!(
            parse_paradigm("procedural"),
            Some(WorkflowParadigm::Procedural)
        );
    }

    #[test]
    fn parse_paradigm_invalid_returns_none() {
        assert!(parse_paradigm("").is_none());
        assert!(parse_paradigm("FSM").is_none());
        assert!(parse_paradigm("state_machine").is_none());
    }

    #[test]
    fn paradigm_to_str_roundtrip() {
        for p in [
            WorkflowParadigm::Fsm,
            WorkflowParadigm::Dag,
            WorkflowParadigm::Procedural,
        ] {
            let s = paradigm_to_str(p);
            assert_eq!(parse_paradigm(s), Some(p));
        }
    }
}
