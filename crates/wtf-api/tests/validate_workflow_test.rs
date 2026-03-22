use axum::{
    body::to_bytes,
    body::Body,
    http::{Request, StatusCode},
    routing::post,
    Router,
};
use serde_json::Value;
use tower::util::ServiceExt;

#[tokio::test]
async fn validate_returns_valid_true_for_clean_source() {
    let app = Router::new().route(
        "/workflows/validate",
        post(wtf_api::handlers::validate_workflow),
    );

    let body = r#"{"source":"impl WorkflowFn for W { async fn execute(&self, ctx: WorkflowContext) -> anyhow::Result<()> { let _ = ctx.now(); Ok(()) } }"}"#;
    let req = Request::builder()
        .method("POST")
        .uri("/workflows/validate")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .expect("request");

    let res = app.oneshot(req).await.expect("response");
    assert_eq!(res.status(), StatusCode::OK);

    let body = to_bytes(res.into_body(), usize::MAX).await.expect("body");
    let json: Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(json.get("valid").and_then(Value::as_bool), Some(true));
}

#[tokio::test]
async fn validate_returns_valid_false_for_error_violation() {
    let app = Router::new().route(
        "/workflows/validate",
        post(wtf_api::handlers::validate_workflow),
    );

    let body = r#"{"source":"impl WorkflowFn for W { async fn execute(&self, ctx: WorkflowContext) -> anyhow::Result<()> { let _ = std::time::SystemTime::now(); Ok(()) } }"}"#;
    let req = Request::builder()
        .method("POST")
        .uri("/workflows/validate")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .expect("request");

    let res = app.oneshot(req).await.expect("response");
    assert_eq!(res.status(), StatusCode::OK);

    let body = to_bytes(res.into_body(), usize::MAX).await.expect("body");
    let json: Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(json.get("valid").and_then(Value::as_bool), Some(false));
}

#[tokio::test]
async fn validate_returns_400_for_parse_error() {
    let app = Router::new().route(
        "/workflows/validate",
        post(wtf_api::handlers::validate_workflow),
    );

    let body = r#"{"source":"impl WorkflowFn for W { async fn execute( {"}"#;
    let req = Request::builder()
        .method("POST")
        .uri("/workflows/validate")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .expect("request");

    let res = app.oneshot(req).await.expect("response");
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn validate_returns_valid_true_for_warning_only_violation() {
    let app = Router::new().route(
        "/workflows/validate",
        post(wtf_api::handlers::validate_workflow),
    );

    let body = r#"{"source":"impl WorkflowFn for W { async fn execute(&self, ctx: WorkflowContext) -> anyhow::Result<()> { let xs = vec![1,2,3]; xs.iter().for_each(|_| { let _ = ctx.now(); }); Ok(()) } }"}"#;
    let req = Request::builder()
        .method("POST")
        .uri("/workflows/validate")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .expect("request");

    let res = app.oneshot(req).await.expect("response");
    assert_eq!(res.status(), StatusCode::OK);

    let body = to_bytes(res.into_body(), usize::MAX).await.expect("body");
    let json: Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(json.get("valid").and_then(Value::as_bool), Some(true));
}

#[tokio::test]
async fn validate_returns_400_for_missing_source_field() {
    let app = Router::new().route(
        "/workflows/validate",
        post(wtf_api::handlers::validate_workflow),
    );

    let req = Request::builder()
        .method("POST")
        .uri("/workflows/validate")
        .header("content-type", "application/json")
        .body(Body::from("{}"))
        .expect("request");

    let res = app.oneshot(req).await.expect("response");
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
