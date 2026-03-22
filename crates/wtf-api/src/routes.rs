//! routes.rs - HTTP routes for wtf-api

use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};
use ractor::ActorRef;
use wtf_actor::OrchestratorMsg;
use wtf_storage::kv::KvStores;

use crate::{handlers, sse};

pub fn create_routes(master: ActorRef<OrchestratorMsg>, kv: KvStores) -> Router {
    Router::new()
        .route(
            "/workflows",
            get(handlers::list_workflows).post(handlers::start_workflow),
        )
        .route(
            "/workflows/:id",
            get(handlers::get_workflow).delete(handlers::terminate_workflow),
        )
        .route("/workflows/:id/journal", get(handlers::get_journal))
        .route("/workflows/validate", post(handlers::validate_workflow))
        .route("/workflows/:id/signals", post(handlers::send_signal))
        .route("/workflows/:id/events", get(handlers::get_events))
        .route("/instances/:id/replay-to/:seq", get(handlers::replay_to))
        .route("/definitions/:type", post(handlers::ingest_definition))
        .route("/watch", get(sse::watch_all))
        .route("/watch/:namespace", get(sse::watch_namespace))
        .layer(Extension(master))
        .layer(Extension(kv))
}
