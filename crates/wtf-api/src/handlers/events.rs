use axum::{extract::{Extension, Path, Query}, http::{header, StatusCode}, response::IntoResponse};
use futures::StreamExt;
use ractor::ActorRef;
use wtf_actor::OrchestratorMsg;
use wtf_common::NamespaceId;
use crate::types::EventRecord;
use super::{split_path_id, get_nats};

#[derive(serde::Deserialize)]
pub struct EventsQuery { pub from_seq: Option<u64> }

pub async fn get_events(
    Extension(master): Extension<ActorRef<OrchestratorMsg>>,
    Path(id): Path<String>,
    Query(query): Query<EventsQuery>,
) -> impl IntoResponse {
    let (ns, inst_id) = match split_path_id(&id) {
        Some(p) => (NamespaceId::new(p.0), p.1),
        None => return (StatusCode::BAD_REQUEST, "invalid_id").into_response(),
    };
    let nats = match get_nats(&master).await { Some(n) => n, None => return (StatusCode::SERVICE_UNAVAILABLE, "no_nats").into_response() };
    
    let from_seq = query.from_seq.unwrap_or(0);
    let config = wtf_storage::ReplayConfig { from_seq, ..Default::default() };
    match wtf_storage::replay_events(nats.jetstream().clone(), ns, inst_id, config).await {
        Ok(stream) => (StatusCode::OK, [(header::CONTENT_TYPE, "application/x-ndjson")], axum::body::Body::from_stream(stream.map(map_event))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

fn map_event(res: Result<wtf_storage::ReplayedEvent, wtf_common::WtfError>) -> Result<String, String> {
    let replayed = res.map_err(|e| e.to_string())?;
    let data = serde_json::to_value(&replayed.event).unwrap_or(serde_json::Value::Null);
    let event_type = data.get("type").and_then(|v| v.as_str()).unwrap_or("unknown").to_owned();
    let record = EventRecord { seq: replayed.seq, event_type, data, timestamp: replayed.timestamp };
    Ok(serde_json::to_string(&record).map_err(|e| e.to_string())? + "\n")
}
