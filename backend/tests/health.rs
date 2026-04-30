mod common;

use axum::body::{Body, to_bytes};
use axum::http::{Method, StatusCode, header::CONTENT_TYPE};
use serde_json::Value;
use tower::ServiceExt;

use backend::config::Config;

#[tokio::test]
async fn health_returns_ok_json() {
    let context = common::test_context().await;

    let (status, body) = common::request_json(context.app, Method::GET, "/health", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn rejects_non_loopback_host_headers() {
    let context = common::test_context().await;
    let request = axum::http::Request::builder()
        .method(Method::GET)
        .uri("/health")
        .header("host", "evil.example")
        .body(Body::empty())
        .unwrap();

    let response = context.app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = serde_json::from_slice::<Value>(&bytes).unwrap();

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "bad_request");
}

#[tokio::test]
async fn rejects_state_changing_requests_from_untrusted_origins() {
    let context = common::test_context().await;
    let request = axum::http::Request::builder()
        .method(Method::POST)
        .uri("/conversations")
        .header("host", "localhost:8080")
        .header("origin", "http://evil.example")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"title":"Cross-site"}"#))
        .unwrap();

    let response = context.app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = serde_json::from_slice::<Value>(&bytes).unwrap();

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "bad_request");
}

#[tokio::test]
async fn accepts_state_changing_requests_from_trusted_frontend_origin() {
    let context = common::test_context().await;
    let request = axum::http::Request::builder()
        .method(Method::POST)
        .uri("/conversations")
        .header("host", "localhost:8080")
        .header("origin", "http://localhost:3000")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"title":"Trusted"}"#))
        .unwrap();

    let response = context.app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[test]
fn rejects_non_loopback_bind_addresses_without_authentication() {
    let mut config = Config::for_tests();
    config.bind_addr = "0.0.0.0:8080".to_owned();

    let error = config.bind_socket_addr().unwrap_err().to_string();

    assert!(error.contains("BACKEND_BIND_ADDR must bind to a loopback address"));
}
