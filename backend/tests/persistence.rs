mod common;

use axum::http::{Method, StatusCode};
use serde_json::json;

#[tokio::test]
async fn conversations_and_messages_are_persisted() {
    let context = common::test_context().await;
    context.provider.with_completion(Ok("hello from assistant"));

    let conversation_id = common::create_conversation(context.app.clone()).await;

    let (list_status, list_body) =
        common::request_json(context.app.clone(), Method::GET, "/conversations", None).await;
    assert_eq!(list_status, StatusCode::OK);
    assert_eq!(list_body["conversations"].as_array().unwrap().len(), 1);
    assert_eq!(list_body["conversations"][0]["id"], conversation_id);

    let (send_status, send_body) = common::request_json(
        context.app.clone(),
        Method::POST,
        &format!("/conversations/{conversation_id}/messages"),
        Some(json!({ "content": "Hello" })),
    )
    .await;
    assert_eq!(send_status, StatusCode::OK, "{send_body}");
    assert_eq!(send_body["user_message"]["role"], "user");
    assert_eq!(send_body["assistant_message"]["role"], "assistant");
    assert_eq!(
        send_body["assistant_message"]["content"],
        "hello from assistant"
    );

    let (messages_status, messages_body) = common::request_json(
        context.app,
        Method::GET,
        &format!("/conversations/{conversation_id}/messages"),
        None,
    )
    .await;
    assert_eq!(messages_status, StatusCode::OK);
    assert_eq!(messages_body["messages"].as_array().unwrap().len(), 2);
    assert_eq!(messages_body["messages"][0]["content"], "Hello");
    assert_eq!(
        messages_body["messages"][1]["content"],
        "hello from assistant"
    );
}

#[tokio::test]
async fn failed_provider_call_records_chat_request_without_assistant_message() {
    let context = common::test_context().await;
    context
        .provider
        .with_completion(Err(backend::provider::ProviderError::RateLimited));
    let conversation_id = common::create_conversation(context.app.clone()).await;

    let (status, body) = common::request_json(
        context.app.clone(),
        Method::POST,
        &format!("/conversations/{conversation_id}/messages"),
        Some(json!({ "content": "Hello" })),
    )
    .await;

    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"]["code"], "provider_rate_limited");

    let message_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM messages WHERE conversation_id = ?")
            .bind(&conversation_id)
            .fetch_one(&context.pool)
            .await
            .unwrap();
    assert_eq!(message_count, 1);

    let failed_requests: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM chat_requests WHERE status = 'failed'")
            .fetch_one(&context.pool)
            .await
            .unwrap();
    assert_eq!(failed_requests, 1);
}
