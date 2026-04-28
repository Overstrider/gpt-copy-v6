mod common;

use axum::http::{Method, StatusCode};
use serde_json::json;

#[tokio::test]
async fn create_conversation_rejects_overlong_title_with_structured_error() {
    let context = common::test_context().await;
    let long_title = "a".repeat(121);

    let (status, body) = common::request_json(
        context.app,
        Method::POST,
        "/conversations",
        Some(json!({ "title": long_title })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_failed");
    assert!(body["error"]["message"].as_str().unwrap().contains("title"));
}

#[tokio::test]
async fn send_message_rejects_blank_content_with_structured_error() {
    let context = common::test_context().await;
    let conversation_id = common::create_conversation(context.app.clone()).await;

    let (status, body) = common::request_json(
        context.app,
        Method::POST,
        &format!("/conversations/{conversation_id}/messages"),
        Some(json!({ "content": "   " })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_failed");
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("content")
    );
}

#[tokio::test]
async fn unknown_conversation_returns_structured_not_found_error() {
    let context = common::test_context().await;

    let (status, body) = common::request_json(
        context.app,
        Method::GET,
        "/conversations/11111111-1111-4111-8111-111111111111/messages",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
}
