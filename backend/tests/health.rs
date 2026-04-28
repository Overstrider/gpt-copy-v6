mod common;

use axum::http::{Method, StatusCode};

#[tokio::test]
async fn health_returns_ok_json() {
    let context = common::test_context().await;

    let (status, body) = common::request_json(context.app, Method::GET, "/health", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}
