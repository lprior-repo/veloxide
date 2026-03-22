use axum::{extract::{Extension, Path}, http::StatusCode, response::IntoResponse, Json};
use bytes::Bytes;
use ractor::rpc::CallResult;
use ractor::{ActorRef, MessagingErr};
use wtf_actor::OrchestratorMsg;
use crate::types::{ApiError, V3SignalRequest};
use super::{ACTOR_CALL_TIMEOUT, split_path_id};

/// POST /api/v1/workflows/:id/signals — send a signal to a running instance.
pub async fn send_signal(
    Extension(master): Extension<ActorRef<OrchestratorMsg>>,
    Path(id): Path<String>,
    Json(req): Json<V3SignalRequest>,
) -> impl IntoResponse {
    let (_, instance_id) = match split_path_id(&id) {
        Some(p) => p,
        None => return (StatusCode::BAD_REQUEST, Json(ApiError::new("invalid_id", "bad id"))).into_response(),
    };
    let payload = match serde_json::to_vec(&req.payload) {
        Ok(v) => Bytes::from(v),
        Err(e) => return (StatusCode::BAD_REQUEST, Json(ApiError::new("invalid_payload", e.to_string()))).into_response(),
    };
    let res = master.call(|tx| OrchestratorMsg::Signal {
        instance_id, signal_name: req.signal_name.clone(), payload, reply: tx,
    }, Some(ACTOR_CALL_TIMEOUT)).await;
    map_signal_result(res).into_response()
}

fn map_signal_result(res: Result<CallResult<Result<(), wtf_common::WtfError>>, MessagingErr<OrchestratorMsg>>) -> impl IntoResponse {
    match res {
        Ok(CallResult::Success(Ok(()))) => StatusCode::ACCEPTED.into_response(),
        Ok(CallResult::Success(Err(e))) => (StatusCode::NOT_FOUND, Json(ApiError::new("signal_failed", e.to_string()))).into_response(),
        Ok(CallResult::Timeout) => (StatusCode::SERVICE_UNAVAILABLE, Json(ApiError::new("actor_timeout", "timeout"))).into_response(),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::new("actor_error", "actor failed"))).into_response(),
    }
}
