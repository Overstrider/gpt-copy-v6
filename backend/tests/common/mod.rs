#![allow(dead_code)]

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode};
use futures_util::stream;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tower::ServiceExt;

use backend::config::Config;
use backend::db;
use backend::provider::{
    ChatProvider, ChatProviderRequest, ChatProviderResponse, ProviderError, ProviderStream,
};
use backend::state::AppState;

type CompletionResult = Result<String, ProviderError>;
type StreamChunks = Vec<Result<String, ProviderError>>;
type StreamResult = Result<StreamChunks, ProviderError>;

#[derive(Default)]
pub struct FakeProvider {
    completions: Mutex<VecDeque<CompletionResult>>,
    streams: Mutex<VecDeque<StreamResult>>,
    requests: Mutex<Vec<ChatProviderRequest>>,
}

impl FakeProvider {
    pub fn with_completion(self: &Arc<Self>, result: Result<&str, ProviderError>) {
        let mapped = result.map(str::to_owned);
        self.completions.lock().unwrap().push_back(mapped);
    }

    pub fn with_stream(
        self: &Arc<Self>,
        result: Result<Vec<Result<&str, ProviderError>>, ProviderError>,
    ) {
        let mapped = result.map(|chunks| {
            chunks
                .into_iter()
                .map(|chunk| chunk.map(str::to_owned))
                .collect::<Vec<_>>()
        });
        self.streams.lock().unwrap().push_back(mapped);
    }

    pub fn requests(&self) -> Vec<ChatProviderRequest> {
        self.requests.lock().unwrap().clone()
    }
}

#[async_trait]
impl ChatProvider for FakeProvider {
    async fn complete(
        &self,
        request: ChatProviderRequest,
    ) -> Result<ChatProviderResponse, ProviderError> {
        self.requests.lock().unwrap().push(request);
        let result = self
            .completions
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| Ok("mock assistant reply".to_owned()))?;

        Ok(ChatProviderResponse { content: result })
    }

    async fn stream(&self, request: ChatProviderRequest) -> Result<ProviderStream, ProviderError> {
        self.requests.lock().unwrap().push(request);
        let chunks = self
            .streams
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| Ok(vec![Ok("mock ".to_owned()), Ok("stream".to_owned())]))?;

        Ok(Box::pin(stream::iter(chunks)))
    }
}

pub struct TestContext {
    pub app: Router,
    pub pool: SqlitePool,
    pub provider: Arc<FakeProvider>,
}

pub async fn test_context() -> TestContext {
    let pool = db::connect_and_migrate("sqlite::memory:").await.unwrap();
    let provider = Arc::new(FakeProvider::default());
    let state = AppState::new(pool.clone(), provider.clone(), Config::for_tests());
    let app = backend::create_router(state);

    TestContext {
        app,
        pool,
        provider,
    }
}

pub async fn request_json(
    app: Router,
    method: Method,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }

    let request = builder
        .body(match body {
            Some(value) => Body::from(value.to_string()),
            None => Body::empty(),
        })
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value = if bytes.is_empty() {
        json!(null)
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };

    (status, value)
}

pub async fn request_text(
    app: Router,
    method: Method,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, String) {
    let mut builder = Request::builder().method(method).uri(uri);
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }

    let request = builder
        .body(match body {
            Some(value) => Body::from(value.to_string()),
            None => Body::empty(),
        })
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();

    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

pub async fn create_conversation(app: Router) -> String {
    let (status, body) = request_json(
        app,
        Method::POST,
        "/conversations",
        Some(json!({ "title": "Test chat" })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "{body}");
    body["conversation"]["id"].as_str().unwrap().to_owned()
}
