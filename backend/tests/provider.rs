mod common;

use axum::Json;
use axum::body::{Body, Bytes};
use axum::extract::State;
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::Response;
use axum::routing::post;
use futures_util::{StreamExt, stream};
use serde_json::json;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};

use backend::provider::{
    ChatProvider, ChatProviderRequest, OpenRouterProvider, OpenRouterStreamLine, ProviderError,
    ProviderMessage, parse_openrouter_sse_line,
};

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

#[tokio::test]
async fn oversized_completion_is_rejected_and_request_marked_failed() {
    let context = common::test_context().await;
    context
        .provider
        .with_completion_text(Ok("x".repeat(64_001)));
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

    let failed_requests: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM chat_requests WHERE status = 'failed'")
            .fetch_one(&context.pool)
            .await
            .unwrap();
    assert_eq!(failed_requests, 1);
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

#[tokio::test]
async fn openrouter_provider_sends_expected_headers_and_body_to_chat_api() {
    #[derive(Default)]
    struct Capture {
        authorization: Option<String>,
        title: Option<String>,
        referer: Option<String>,
        body: Option<serde_json::Value>,
    }

    async fn handler(
        State(capture): State<Arc<Mutex<Capture>>>,
        headers: HeaderMap,
        Json(body): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        let mut capture = capture.lock().unwrap();
        capture.authorization = headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        capture.title = headers
            .get("x-title")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        capture.referer = headers
            .get("http-referer")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        capture.body = Some(body);

        Json(json!({
            "choices": [
                { "message": { "content": "adapter response" } }
            ]
        }))
    }

    let capture = Arc::new(Mutex::new(Capture::default()));
    let app = axum::Router::new()
        .route("/chat/completions", post(handler))
        .with_state(capture.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let provider = OpenRouterProvider::new(
        Some("test-api-key".to_owned()),
        Some("http://localhost:3000".to_owned()),
        "gpt-copy-v6".to_owned(),
    )
    .with_chat_completions_url(format!("http://{addr}/chat/completions"));

    let response = provider
        .complete(ChatProviderRequest {
            model: "test-model".to_owned(),
            messages: vec![ProviderMessage {
                role: "user".to_owned(),
                content: "hello".to_owned(),
            }],
        })
        .await
        .unwrap();

    assert_eq!(response.content, "adapter response");
    let captured = capture.lock().unwrap();
    assert_eq!(
        captured.authorization.as_deref(),
        Some("Bearer test-api-key")
    );
    assert_eq!(captured.title.as_deref(), Some("gpt-copy-v6"));
    assert_eq!(captured.referer.as_deref(), Some("http://localhost:3000"));
    assert_eq!(captured.body.as_ref().unwrap()["model"], "test-model");
    assert_eq!(captured.body.as_ref().unwrap()["stream"], false);
    assert_eq!(captured.body.as_ref().unwrap()["max_tokens"], 2048);

    server.abort();
}

#[tokio::test]
async fn openrouter_provider_stream_preserves_utf8_split_across_http_chunks() {
    #[derive(Default)]
    struct Capture {
        body: Option<serde_json::Value>,
    }

    async fn handler(
        State(capture): State<Arc<Mutex<Capture>>>,
        Json(body): Json<serde_json::Value>,
    ) -> Response {
        capture.lock().unwrap().body = Some(body);
        let payload = r#"data: {"choices":[{"delta":{"content":"olá"}}]}

data: [DONE]

"#;
        let bytes = payload.as_bytes();
        let split = bytes
            .iter()
            .position(|byte| *byte == 0xc3)
            .expect("test payload should contain a multi-byte character")
            + 1;
        let chunks = vec![
            Ok::<Bytes, Infallible>(Bytes::copy_from_slice(&bytes[..split])),
            Ok::<Bytes, Infallible>(Bytes::copy_from_slice(&bytes[split..])),
        ];

        Response::builder()
            .header("content-type", "text/event-stream")
            .body(Body::from_stream(stream::iter(chunks)))
            .unwrap()
    }

    let capture = Arc::new(Mutex::new(Capture::default()));
    let app = axum::Router::new()
        .route("/chat/completions", post(handler))
        .with_state(capture.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let provider = OpenRouterProvider::new(
        Some("test-api-key".to_owned()),
        None,
        "gpt-copy-v6".to_owned(),
    )
    .with_chat_completions_url(format!("http://{addr}/chat/completions"));
    let mut stream = provider
        .stream(ChatProviderRequest {
            model: "test-model".to_owned(),
            messages: vec![ProviderMessage {
                role: "user".to_owned(),
                content: "hello".to_owned(),
            }],
        })
        .await
        .unwrap();
    let mut chunks = Vec::new();

    while let Some(chunk) = stream.next().await {
        chunks.push(chunk.unwrap());
    }

    assert_eq!(chunks, vec!["olá"]);
    let captured = capture.lock().unwrap();
    assert_eq!(captured.body.as_ref().unwrap()["model"], "test-model");
    assert_eq!(
        captured.body.as_ref().unwrap()["messages"][0]["role"],
        "user"
    );
    assert_eq!(
        captured.body.as_ref().unwrap()["messages"][0]["content"],
        "hello"
    );
    assert_eq!(captured.body.as_ref().unwrap()["stream"], true);
    assert_eq!(captured.body.as_ref().unwrap()["max_tokens"], 2048);

    server.abort();
}

#[tokio::test]
async fn openrouter_provider_stream_maps_upstream_rate_limits() {
    async fn handler() -> Response {
        Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .body(Body::empty())
            .unwrap()
    }

    let app = axum::Router::new().route("/chat/completions", post(handler));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let provider = OpenRouterProvider::new(
        Some("test-api-key".to_owned()),
        None,
        "gpt-copy-v6".to_owned(),
    )
    .with_chat_completions_url(format!("http://{addr}/chat/completions"));

    let error = match provider
        .stream(ChatProviderRequest {
            model: "test-model".to_owned(),
            messages: vec![ProviderMessage {
                role: "user".to_owned(),
                content: "hello".to_owned(),
            }],
        })
        .await
    {
        Ok(_) => panic!("expected upstream 429 to map to ProviderError::RateLimited"),
        Err(error) => error,
    };

    assert!(matches!(error, ProviderError::RateLimited));

    server.abort();
}
