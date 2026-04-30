mod common;

use std::sync::Arc;

use axum::http::{Method, StatusCode};
use serde_json::json;

use backend::config::{Config, DEFAULT_OPENROUTER_MODEL};
use backend::provider::ProviderError;
use backend::state::AppState;
use backend::{create_router, db};

const SENTINEL_API_KEY: &str = "test-openrouter-config-secret";

// These tests target the non-stream send endpoint behind the task's chat_send
// filter and keep OpenRouter mocked so automated tests never use real quota.
async fn test_context_with_secret_config() -> common::TestContext {
    let pool = db::connect_and_migrate("sqlite::memory:").await.unwrap();
    let provider = Arc::new(common::FakeProvider::default());
    let config = Config {
        bind_addr: "127.0.0.1:0".to_owned(),
        database_url: "sqlite::memory:".to_owned(),
        openrouter_api_key: Some(SENTINEL_API_KEY.to_owned()),
        openrouter_model: DEFAULT_OPENROUTER_MODEL.to_owned(),
        openrouter_http_referer: None,
        openrouter_title: "gpt-copy-v6-tests".to_owned(),
    };
    let app = create_router(AppState::new(pool.clone(), provider.clone(), config));

    common::TestContext {
        app,
        pool,
        provider,
    }
}

#[tokio::test]
async fn chat_send_success_persists_messages_and_does_not_expose_api_key() {
    let context = test_context_with_secret_config().await;
    context.provider.with_completion(Ok("assistant answer"));
    let conversation_id = common::create_conversation(context.app.clone()).await;

    let (status, body) = common::request_json(
        context.app,
        Method::POST,
        &format!("/conversations/{conversation_id}/messages"),
        Some(json!({ "content": "  Hello provider  " })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["user_message"]["role"], "user");
    assert_eq!(body["user_message"]["content"], "Hello provider");
    assert_eq!(body["assistant_message"]["role"], "assistant");
    assert_eq!(body["assistant_message"]["content"], "assistant answer");
    assert!(!body.to_string().contains(SENTINEL_API_KEY));

    let requests = context.provider.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].model, DEFAULT_OPENROUTER_MODEL);
    assert_eq!(requests[0].messages.len(), 1);
    assert_eq!(requests[0].messages[0].role, "user");
    assert_eq!(requests[0].messages[0].content, "Hello provider");

    let messages: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT role, content, status FROM messages WHERE conversation_id = ? ORDER BY ordinal ASC",
    )
    .bind(&conversation_id)
    .fetch_all(&context.pool)
    .await
    .unwrap();
    assert_eq!(
        messages,
        vec![
            (
                "user".to_owned(),
                "Hello provider".to_owned(),
                "completed".to_owned()
            ),
            (
                "assistant".to_owned(),
                "assistant answer".to_owned(),
                "completed".to_owned()
            ),
        ]
    );

    let request: (String, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT status, assistant_message_id, error_code FROM chat_requests WHERE conversation_id = ?",
    )
    .bind(&conversation_id)
    .fetch_one(&context.pool)
    .await
    .unwrap();
    assert_eq!(request.0, "succeeded");
    assert_eq!(
        request.1.as_deref(),
        body["assistant_message"]["id"].as_str()
    );
    assert!(request.2.is_none());
}

#[tokio::test]
async fn chat_send_provider_failures_return_structured_errors_and_no_assistant_message() {
    let cases = vec![
        (
            ProviderError::RateLimited,
            StatusCode::TOO_MANY_REQUESTS,
            "provider_rate_limited",
        ),
        (
            ProviderError::Timeout,
            StatusCode::GATEWAY_TIMEOUT,
            "provider_timeout",
        ),
        (
            ProviderError::InvalidResponse("missing content".to_owned()),
            StatusCode::BAD_GATEWAY,
            "provider_invalid_response",
        ),
    ];

    for (provider_error, expected_status, expected_code) in cases {
        let context = test_context_with_secret_config().await;
        context.provider.with_completion(Err(provider_error));
        let conversation_id = common::create_conversation(context.app.clone()).await;

        let (status, body) = common::request_json(
            context.app,
            Method::POST,
            &format!("/conversations/{conversation_id}/messages"),
            Some(json!({ "content": "Hello" })),
        )
        .await;

        assert_eq!(status, expected_status, "{body}");
        assert_eq!(body["error"]["code"], expected_code);
        assert!(
            body["error"]["message"]
                .as_str()
                .unwrap()
                .starts_with("OpenRouter")
        );
        assert!(!body.to_string().contains(SENTINEL_API_KEY));

        let counts: (i64, i64) = sqlx::query_as(
            "SELECT
                SUM(CASE WHEN role = 'user' THEN 1 ELSE 0 END),
                SUM(CASE WHEN role = 'assistant' THEN 1 ELSE 0 END)
             FROM messages
             WHERE conversation_id = ?",
        )
        .bind(&conversation_id)
        .fetch_one(&context.pool)
        .await
        .unwrap();
        assert_eq!(counts, (1, 0));

        let request: (String, Option<String>, Option<String>) = sqlx::query_as(
            "SELECT status, assistant_message_id, error_code FROM chat_requests WHERE conversation_id = ?",
        )
        .bind(&conversation_id)
        .fetch_one(&context.pool)
        .await
        .unwrap();
        assert_eq!(request.0, "failed");
        assert!(request.1.is_none());
        assert_eq!(request.2.as_deref(), Some(expected_code));
    }
}
