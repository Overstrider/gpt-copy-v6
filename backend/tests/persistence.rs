mod common;

use std::path::PathBuf;

use axum::http::{Method, StatusCode};
use futures_util::future::join_all;
use serde_json::json;
use uuid::Uuid;

use backend::db;

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

#[tokio::test]
async fn overlapping_message_inserts_keep_unique_conversation_ordinals() {
    let (database_path, database_url) = temporary_database_url();
    let pool = db::connect_and_migrate(&database_url).await.unwrap();
    let conversation = db::create_conversation(&pool, "Concurrent ordinals")
        .await
        .unwrap();

    let inserts = (0..12).map(|index| {
        let pool = pool.clone();
        let conversation_id = conversation.id.clone();

        async move {
            if index % 2 == 0 {
                db::insert_message(
                    &pool,
                    &conversation_id,
                    "user",
                    &format!("parallel message {index}"),
                )
                .await
            } else {
                db::insert_streaming_assistant_message(&pool, &conversation_id).await
            }
        }
    });
    let inserted = join_all(inserts)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(inserted.len(), 12);
    let ordinals = sqlx::query_scalar::<_, i64>(
        "SELECT ordinal FROM messages WHERE conversation_id = ? ORDER BY ordinal ASC",
    )
    .bind(&conversation.id)
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(ordinals, (0..12).collect::<Vec<_>>());

    pool.close().await;
    let _ = std::fs::remove_file(&database_path);
}

fn temporary_database_url() -> (PathBuf, String) {
    let database_path = std::env::temp_dir().join(format!("gpt-copy-v6-{}.db", Uuid::new_v4()));
    let database_url = format!(
        "sqlite:{}",
        database_path.to_string_lossy().replace('\\', "/")
    );

    (database_path, database_url)
}
