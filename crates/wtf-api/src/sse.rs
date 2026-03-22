//! Server-Sent Events (SSE) for NATS KV watches.
//!
//! Provides real-time updates for workflow instance changes.

use std::{convert::Infallible, time::Duration};

use async_nats::jetstream::kv::{Entry, Operation};
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
};
use futures::stream::{Stream, StreamExt};
use wtf_storage::kv::KvStores;

/// GET /api/v1/watch — watch all workflow instances across all namespaces.
pub async fn watch_all(Extension(kv): Extension<KvStores>) -> Response {
    match kv.instances.watch(">").await {
        Ok(stream) => Sse::new(map_kv_stream(stream))
            .keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
            .into_response(),
        Err(e) => {
            tracing::error!("failed to watch all instances: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// GET /api/v1/watch/:namespace — watch instances in a specific namespace.
pub async fn watch_namespace(
    Extension(kv): Extension<KvStores>,
    Path(ns): Path<String>,
) -> Response {
    match kv.instances.watch(&format!("{ns}/*")).await {
        Ok(stream) => Sse::new(map_kv_stream(stream))
            .keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
            .into_response(),
        Err(e) => {
            tracing::error!("failed to watch namespace {ns}: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Map NATS KV entry stream to Axum SSE event stream.
///
/// Only `Put` operations are forwarded. Errors are logged and skipped.
fn map_kv_stream<E: std::fmt::Display + Send + 'static>(
    stream: impl Stream<Item = Result<Entry, E>> + Send + 'static,
) -> impl Stream<Item = Result<Event, Infallible>> {
    stream.filter_map(|res| async move {
        let entry = match res {
            Ok(e) => e,
            Err(e) => {
                tracing::error!("watch stream error: {e}");
                return None;
            }
        };
        (entry.operation == Operation::Put).then(|| {
            let val = String::from_utf8_lossy(&entry.value);
            let data = format!("{}:{}", entry.key, val);
            Ok(Event::default().event("put").data(data))
        })
    })
}
