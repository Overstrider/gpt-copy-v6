mod common;

use axum::http::{Method, StatusCode};
use serde_json::json;

use backend::provider::{OpenRouterStreamLine, ProviderError, parse_openrouter_sse_line};

#[tokio::test]
async fn mocked_provider_receives_default_model_and_conversation_history() {
    let context = common::test_context().await;
    context.provider.with_completion(Ok("first answer"));
    let conversation_id = common::create_conversation(context.app.clone()).await;

    let (status, body) = common::request_json(
        context.app,
        Method::POST,
        &format!("/conversations/{conversation_id}/messages"),
        Some(json!({ "content": "What is Rust?" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    let requests = context.provider.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].model, "nvidia/nemotron-3-super-120b-a12b:free");
    assert_eq!(requests[0].messages.last().unwrap().role, "user");
    assert_eq!(
        requests[0].messages.last().unwrap().content,
        "What is Rust?"
    );
}

#[tokio::test]
async fn provider_timeout_maps_to_gateway_timeout() {
    let context = common::test_context().await;
    context
        .provider
        .with_completion(Err(ProviderError::Timeout));
    let conversation_id = common::create_conversation(context.app.clone()).await;

    let (status, body) = common::request_json(
        context.app,
        Method::POST,
        &format!("/conversations/{conversation_id}/messages"),
        Some(json!({ "content": "Hello" })),
    )
    .await;

    assert_eq!(status, StatusCode::GATEWAY_TIMEOUT);
    assert_eq!(body["error"]["code"], "provider_timeout");
}

#[tokio::test]
async fn provider_invalid_response_maps_to_bad_gateway() {
    let context = common::test_context().await;
    context
        .provider
        .with_completion(Err(ProviderError::InvalidResponse(
            "missing content".into(),
        )));
    let conversation_id = common::create_conversation(context.app.clone()).await;

    let (status, body) = common::request_json(
        context.app,
        Method::POST,
        &format!("/conversations/{conversation_id}/messages"),
        Some(json!({ "content": "Hello" })),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert_eq!(body["error"]["code"], "provider_invalid_response");
}

#[test]
fn openrouter_stream_parser_skips_comments_and_reports_provider_error_chunks() {
    assert_eq!(
        parse_openrouter_sse_line(": keep-alive").unwrap(),
        OpenRouterStreamLine::Skip
    );
    assert_eq!(
        parse_openrouter_sse_line("data: [DONE]").unwrap(),
        OpenRouterStreamLine::Done
    );
    assert_eq!(
        parse_openrouter_sse_line(r#"data: {"choices":[{"delta":{"content":"hi"}}]}"#).unwrap(),
        OpenRouterStreamLine::Content("hi".to_owned())
    );

    let error =
        parse_openrouter_sse_line(r#"data: {"error":{"message":"rate limited","code":429}}"#)
            .unwrap_err();
    assert!(matches!(error, ProviderError::RateLimited));
}
