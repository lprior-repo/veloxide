use axum::{
    body::{to_bytes, Body},
    extract::Extension,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use ractor::{Actor, ActorProcessingErr};
use serde_json::Value;
use tower::ServiceExt;
use wtf_actor::OrchestratorMsg;

/// Integration tests for GET /api/v1/workflows/:id/journal endpoint.
///
/// These tests focus on the HTTP layer: request parsing, routing, and error responses.
/// A MockOrchestrator provides the required Extension<ActorRef<OrchestratorMsg>>
/// and replies None to GetEventStore, so valid namespaced IDs yield 500.

use wtf_common::storage::{EventStore, ReplayBatch, ReplayedEvent, ReplayStream};
use wtf_common::{WorkflowEvent, WtfError, NamespaceId, InstanceId};
use chrono::Utc;
use std::sync::Arc;

struct MockOrchestrator {
    return_store: bool,
}

#[ractor::async_trait]
impl Actor for MockOrchestrator {
    type Msg = OrchestratorMsg;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(())
    }

    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        msg: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let OrchestratorMsg::GetEventStore { reply } = msg {
            if self.return_store {
                let _ = reply.send(Some(Arc::new(HappyEventStore)));
            } else {
                let _ = reply.send(None);
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct HappyEventStore;

#[async_trait::async_trait]
impl EventStore for HappyEventStore {
    async fn publish(
        &self,
        _ns: &NamespaceId,
        _inst_id: &InstanceId,
        _event: WorkflowEvent,
    ) -> Result<u64, WtfError> {
        Ok(1)
    }

    async fn open_replay_stream(
        &self,
        _ns: &NamespaceId,
        _inst_id: &InstanceId,
        _from_seq: u64,
    ) -> Result<Box<dyn ReplayStream>, WtfError> {
        Ok(Box::new(HappyReplayStream { returned: false }))
    }
}

struct HappyReplayStream {
    returned: bool,
}

#[async_trait::async_trait]
impl ReplayStream for HappyReplayStream {
    async fn next_event(&mut self) -> Result<ReplayBatch, WtfError> {
        if !self.returned {
            self.returned = true;
            Ok(ReplayBatch::Event(ReplayedEvent {
                seq: 1,
                timestamp: Utc::now(),
                event: WorkflowEvent::ActivityDispatched {
                    activity_id: "test-act".into(),
                    activity_type: "test-type".into(),
                    payload: bytes::Bytes::new(),
                    retry_policy: wtf_common::RetryPolicy::default(),
                    attempt: 1,
                },
            }))
        } else {
            Ok(ReplayBatch::TailReached)
        }
    }

    async fn next_live_event(&mut self) -> Result<ReplayedEvent, WtfError> {
        futures::future::pending().await
    }
}

/// Helper: build the app Router with MockOrchestrator Extension layer.
async fn app(return_store: bool) -> Router {
    let (actor, _handle) = Actor::spawn(None, MockOrchestrator { return_store }, ()).await.unwrap();
    Router::new()
        .route(
            "/api/v1/workflows/:id/journal",
            get(wtf_api::handlers::get_journal),
        )
        .layer(Extension(actor))
}

#[tokio::test]
async fn given_empty_id_when_get_journal_then_bad_request() {
    let app = app(false).await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/workflows//journal")
        .body(Body::empty())
        .expect("request");

    let res = app.oneshot(req).await.expect("response");
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn given_whitespace_id_when_get_journal_then_bad_request() {
    let app = app(false).await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/workflows/%20%20%20/journal")
        .body(Body::empty())
        .expect("request");

    let res = app.oneshot(req).await.expect("response");
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn given_id_without_namespace_when_get_journal_then_bad_request() {
    let app = app(false).await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/workflows/01ARZ3NDEKTSV4RRFFQ69G5FAV/journal")
        .body(Body::empty())
        .expect("request");

    let res = app.oneshot(req).await.expect("response");
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn given_valid_namespaced_id_when_get_journal_without_actor_then_internal_error() {
    let app = app(false).await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/workflows/payments%2F01ARZ3NDEKTSV4RRFFQ69G5FAV/journal")
        .body(Body::empty())
        .expect("request");

    let res = app.oneshot(req).await.expect("response");
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = to_bytes(res.into_body(), usize::MAX).await.expect("body");
    let json: Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(
        json.get("error").and_then(Value::as_str),
        Some("internal_error")
    );
}

#[tokio::test]
async fn journal_endpoint_route_is_configured() {
    let app = app(false).await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/workflows/test%2Finstance123/journal")
        .body(Body::empty())
        .expect("request");

    let res = app.oneshot(req).await.expect("response");
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn journal_response_structure_is_valid_json() {
    let app = app(false).await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/workflows/payments%2F01ARZ3NDEKTSV4RRFFQ69G5FAV/journal")
        .body(Body::empty())
        .expect("request");

    let res = app.oneshot(req).await.expect("response");
    let body = to_bytes(res.into_body(), usize::MAX).await.expect("body");

    let json: Result<Value, _> = serde_json::from_slice(&body);
    assert!(json.is_ok(), "Response should be valid JSON even on error");
}

#[tokio::test]
async fn journal_endpoint_returns_correct_content_type() {
    let app = app(false).await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/workflows/payments%2F01ARZ3NDEKTSV4RRFFQ69G5FAV/journal")
        .body(Body::empty())
        .expect("request");

    let res = app.oneshot(req).await.expect("response");
    assert!(
        res.headers()
            .get("content-type")
            .is_some_and(|v| v.to_str().is_ok_and(|s| s.contains("application/json"))),
        "Should return application/json content-type"
    );
}

#[tokio::test]
async fn given_valid_namespaced_id_when_get_journal_with_actor_then_ok() {
    let app = app(true).await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/workflows/payments%2F01ARZ3NDEKTSV4RRFFQ69G5FAV/journal")
        .body(Body::empty())
        .expect("request");

    let res = app.oneshot(req).await.expect("response");
    assert_eq!(res.status(), StatusCode::OK);

    let body = to_bytes(res.into_body(), usize::MAX).await.expect("body");
    let json: Value = serde_json::from_slice(&body).expect("json");
    
    let entries = json.get("entries").expect("entries").as_array().expect("array");
    assert_eq!(entries.len(), 1);
    
    let first = &entries[0];
    assert_eq!(first.get("seq").and_then(Value::as_u64), Some(1));
    assert_eq!(first.get("type").and_then(Value::as_str), Some("Run"));
    assert_eq!(first.get("status").and_then(Value::as_str), Some("dispatched"));
    assert_eq!(first.get("name").and_then(Value::as_str), Some("test-type"));
}
