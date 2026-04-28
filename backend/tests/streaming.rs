mod common;

use axum::http::{Method, StatusCode};
use serde_json::json;

use backend::provider::ProviderError;

#[tokio::test]
async fn stream_endpoint_emits_chunks_and_persists_final_assistant_message() {
    let context = common::test_context().await;
    context
        .provider
        .with_stream(Ok(vec![Ok("hello"), Ok(" "), Ok("stream")]));
    let conversation_id = common::create_conversation(context.app.clone()).await;

    let (status, body) = common::request_text(
        context.app.clone(),
        Method::POST,
        &format!("/conversations/{conversation_id}/messages/stream"),
        Some(json!({ "content": "Stream please" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("event: message_start"));
    assert!(body.contains("event: delta"));
    assert!(body.contains(r#""content":"hello""#));
    assert!(body.contains(r#""content":"stream""#));
    assert!(body.contains("event: message_complete"));

    let (_, messages_body) = common::request_json(
        context.app,
        Method::GET,
        &format!("/conversations/{conversation_id}/messages"),
        None,
    )
    .await;
    assert_eq!(messages_body["messages"].as_array().unwrap().len(), 2);
    assert_eq!(messages_body["messages"][1]["content"], "hello stream");
    assert_eq!(messages_body["messages"][1]["status"], "completed");
}

#[tokio::test]
async fn interrupted_stream_emits_error_event_and_marks_request_failed() {
    let context = common::test_context().await;
    context.provider.with_stream(Ok(vec![
        Ok("partial"),
        Err(ProviderError::Interrupted("connection closed".into())),
    ]));
    let conversation_id = common::create_conversation(context.app.clone()).await;

    let (status, body) = common::request_text(
        context.app,
        Method::POST,
        &format!("/conversations/{conversation_id}/messages/stream"),
        Some(json!({ "content": "Stream please" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("event: message_start"));
    assert!(body.contains("event: delta"));
    assert!(body.contains("event: error"));
    assert!(body.contains("provider_stream_interrupted"));

    let message_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM messages WHERE conversation_id = ?")
            .bind(&conversation_id)
            .fetch_one(&context.pool)
            .await
            .unwrap();
    assert_eq!(message_count, 2);

    let failed_messages: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM messages WHERE status = 'failed'")
            .fetch_one(&context.pool)
            .await
            .unwrap();
    assert_eq!(failed_messages, 1);

    let failed_requests: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM chat_requests WHERE status = 'failed'")
            .fetch_one(&context.pool)
            .await
            .unwrap();
    assert_eq!(failed_requests, 1);
}
