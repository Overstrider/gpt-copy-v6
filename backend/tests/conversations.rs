mod common;

use axum::http::{Method, StatusCode};
use serde_json::json;

#[tokio::test]
async fn conversations_list_uses_deterministic_tie_breaker_order() {
    let context = common::test_context().await;
    let timestamp = "2026-01-01T00:00:00.000Z";
    let rows = [
        ("33333333-3333-4333-8333-333333333333", "third"),
        ("11111111-1111-4111-8111-111111111111", "first"),
        ("22222222-2222-4222-8222-222222222222", "second"),
    ];

    for (id, title) in rows {
        sqlx::query(
            "INSERT INTO conversations (id, title, created_at, updated_at)
             VALUES (?, ?, ?, ?)",
        )
        .bind(id)
        .bind(title)
        .bind(timestamp)
        .bind(timestamp)
        .execute(&context.pool)
        .await
        .unwrap();
    }

    let (status, body) =
        common::request_json(context.app, Method::GET, "/conversations", None).await;

    assert_eq!(status, StatusCode::OK);
    let ids = body["conversations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|conversation| conversation["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        vec![
            "11111111-1111-4111-8111-111111111111",
            "22222222-2222-4222-8222-222222222222",
            "33333333-3333-4333-8333-333333333333",
        ]
    );
}

#[tokio::test]
async fn create_conversation_accepts_validated_input_and_returns_conversation_json() {
    let context = common::test_context().await;

    let (status, body) = common::request_json(
        context.app,
        Method::POST,
        "/conversations",
        Some(json!({ "title": "  Planning chat  " })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "{body}");
    let conversation = &body["conversation"];
    assert!(conversation["id"].as_str().is_some());
    assert_eq!(conversation["title"], "Planning chat");
    assert!(conversation["created_at"].as_str().is_some());
    assert!(conversation["updated_at"].as_str().is_some());
}

#[tokio::test]
async fn conversation_messages_load_in_persisted_order() {
    let context = common::test_context().await;
    let conversation_id = common::create_conversation(context.app.clone()).await;

    sqlx::query(
        "INSERT INTO messages (id, conversation_id, role, content, ordinal)
         VALUES (?, ?, 'assistant', 'second', 1), (?, ?, 'user', 'first', 0)",
    )
    .bind("22222222-2222-4222-8222-222222222222")
    .bind(&conversation_id)
    .bind("11111111-1111-4111-8111-111111111111")
    .bind(&conversation_id)
    .execute(&context.pool)
    .await
    .unwrap();

    let (status, body) = common::request_json(
        context.app,
        Method::GET,
        &format!("/conversations/{conversation_id}/messages"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    let messages = body["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0]["content"], "first");
    assert_eq!(messages[1]["content"], "second");
}

#[tokio::test]
async fn conversation_validation_errors_use_structured_error_envelope() {
    let context = common::test_context().await;

    let (id_status, id_body) = common::request_json(
        context.app.clone(),
        Method::GET,
        "/conversations/not-a-uuid/messages",
        None,
    )
    .await;
    assert_eq!(id_status, StatusCode::BAD_REQUEST);
    assert_eq!(id_body["error"]["code"], "bad_request");
    assert!(
        id_body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("UUID")
    );

    let (payload_status, payload_body) = common::request_json(
        context.app,
        Method::POST,
        "/conversations",
        Some(json!({ "title": "   " })),
    )
    .await;
    assert_eq!(payload_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload_body["error"]["code"], "validation_failed");
}
