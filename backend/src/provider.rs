use std::pin::Pin;
use std::time::Duration;

use async_stream::try_stream;
use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub type ProviderStream = Pin<Box<dyn Stream<Item = Result<String, ProviderError>> + Send>>;
const OPENROUTER_CHAT_COMPLETIONS_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const OPENROUTER_MAX_TOKENS: u32 = 2048;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProviderMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatProviderRequest {
    pub model: String,
    pub messages: Vec<ProviderMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatProviderResponse {
    pub content: String,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ProviderError {
    #[error("OpenRouter API key is missing")]
    MissingApiKey,
    #[error("OpenRouter rate limit exceeded")]
    RateLimited,
    #[error("OpenRouter request timed out")]
    Timeout,
    #[error("OpenRouter returned an invalid response: {0}")]
    InvalidResponse(String),
    #[error("OpenRouter stream was interrupted: {0}")]
    Interrupted(String),
    #[error("OpenRouter transport error: {0}")]
    Transport(String),
    #[error("OpenRouter returned HTTP status {status}: {message}")]
    UpstreamStatus { status: u16, message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenRouterStreamLine {
    Content(String),
    Done,
    Skip,
}

#[async_trait]
pub trait ChatProvider: Send + Sync {
    async fn complete(
        &self,
        request: ChatProviderRequest,
    ) -> Result<ChatProviderResponse, ProviderError>;

    async fn stream(&self, request: ChatProviderRequest) -> Result<ProviderStream, ProviderError>;
}

#[derive(Clone)]
pub struct OpenRouterProvider {
    client: reqwest::Client,
    api_key: Option<String>,
    http_referer: Option<String>,
    title: String,
    chat_completions_url: String,
}

impl OpenRouterProvider {
    pub fn new(api_key: Option<String>, http_referer: Option<String>, title: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(60))
                .build()
                .expect("failed to build OpenRouter HTTP client"),
            api_key,
            http_referer,
            title,
            chat_completions_url: OPENROUTER_CHAT_COMPLETIONS_URL.to_owned(),
        }
    }

    pub fn with_chat_completions_url(mut self, url: impl Into<String>) -> Self {
        self.chat_completions_url = url.into();
        self
    }

    fn request_builder(&self, url: &str) -> Result<reqwest::RequestBuilder, ProviderError> {
        let api_key = self
            .api_key
            .as_deref()
            .ok_or(ProviderError::MissingApiKey)?;

        let mut builder = self
            .client
            .post(url)
            .bearer_auth(api_key)
            .header("X-Title", &self.title)
            .header("X-OpenRouter-Title", &self.title);

        if let Some(referer) = &self.http_referer {
            builder = builder.header("HTTP-Referer", referer);
        }

        Ok(builder)
    }
}

#[async_trait]
impl ChatProvider for OpenRouterProvider {
    async fn complete(
        &self,
        request: ChatProviderRequest,
    ) -> Result<ChatProviderResponse, ProviderError> {
        let body = OpenRouterRequest {
            model: request.model,
            messages: request.messages,
            stream: false,
            max_tokens: OPENROUTER_MAX_TOKENS,
        };
        let response = self
            .request_builder(&self.chat_completions_url)?
            .json(&body)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        let status = response.status();
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(ProviderError::RateLimited);
        }
        if !status.is_success() {
            return Err(ProviderError::UpstreamStatus {
                status: status.as_u16(),
                message: "chat completion request failed".to_owned(),
            });
        }

        let completion = response
            .json::<OpenRouterCompletionResponse>()
            .await
            .map_err(map_reqwest_error)?;
        let content = completion
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_deref())
            .filter(|content| !content.trim().is_empty())
            .ok_or_else(|| ProviderError::InvalidResponse("missing assistant content".into()))?;

        Ok(ChatProviderResponse {
            content: content.to_owned(),
        })
    }

    async fn stream(&self, request: ChatProviderRequest) -> Result<ProviderStream, ProviderError> {
        let body = OpenRouterRequest {
            model: request.model,
            messages: request.messages,
            stream: true,
            max_tokens: OPENROUTER_MAX_TOKENS,
        };
        let response = self
            .request_builder(&self.chat_completions_url)?
            .json(&body)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        let status = response.status();
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(ProviderError::RateLimited);
        }
        if !status.is_success() {
            return Err(ProviderError::UpstreamStatus {
                status: status.as_u16(),
                message: "chat completion stream failed".to_owned(),
            });
        }

        let mut bytes = response.bytes_stream();
        let stream = try_stream! {
            let mut buffer = Vec::<u8>::new();
            let mut done = false;

            while let Some(chunk) = bytes.next().await {
                let chunk = chunk.map_err(map_reqwest_error)?;
                buffer.extend_from_slice(&chunk);

                while let Some(newline) = buffer.iter().position(|byte| *byte == b'\n') {
                    let line = buffer.drain(..=newline).collect::<Vec<_>>();
                    let line = String::from_utf8(line)
                        .map_err(|error| ProviderError::InvalidResponse(error.to_string()))?;
                    match parse_openrouter_sse_line(&line)? {
                        OpenRouterStreamLine::Content(content) => yield content,
                        OpenRouterStreamLine::Done => {
                            done = true;
                            break;
                        }
                        OpenRouterStreamLine::Skip => {}
                    }
                }

                if done {
                    break;
                }
            }

            if !done && !buffer.iter().all(u8::is_ascii_whitespace) {
                let line = String::from_utf8(buffer)
                    .map_err(|error| ProviderError::InvalidResponse(error.to_string()))?;
                match parse_openrouter_sse_line(&line)? {
                    OpenRouterStreamLine::Content(content) => yield content,
                    OpenRouterStreamLine::Done => done = true,
                    OpenRouterStreamLine::Skip => {}
                }
            }

            if !done {
                Err(ProviderError::Interrupted("stream ended before [DONE]".into()))?;
            }
        };

        Ok(Box::pin(stream))
    }
}

pub fn parse_openrouter_sse_line(line: &str) -> Result<OpenRouterStreamLine, ProviderError> {
    let line = line.trim();

    if line.is_empty() || line.starts_with(':') {
        return Ok(OpenRouterStreamLine::Skip);
    }

    let Some(data) = line.strip_prefix("data:") else {
        return Ok(OpenRouterStreamLine::Skip);
    };
    let data = data.trim();

    if data == "[DONE]" {
        return Ok(OpenRouterStreamLine::Done);
    }

    let value = serde_json::from_str::<Value>(data)
        .map_err(|error| ProviderError::InvalidResponse(error.to_string()))?;

    if let Some(error) = value.get("error") {
        return parse_provider_error_chunk(error);
    }

    let content = value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("delta").or_else(|| choice.get("message")))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str);

    match content {
        Some(content) if !content.is_empty() => {
            Ok(OpenRouterStreamLine::Content(content.to_owned()))
        }
        _ => Ok(OpenRouterStreamLine::Skip),
    }
}

fn parse_provider_error_chunk(error: &Value) -> Result<OpenRouterStreamLine, ProviderError> {
    let status = error
        .get("code")
        .and_then(|code| code.as_u64().or_else(|| code.as_str()?.parse().ok()))
        .and_then(|code| u16::try_from(code).ok())
        .unwrap_or(StatusCode::BAD_GATEWAY.as_u16());
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("provider returned an error")
        .to_owned();

    if status == StatusCode::TOO_MANY_REQUESTS.as_u16() {
        Err(ProviderError::RateLimited)
    } else {
        Err(ProviderError::UpstreamStatus { status, message })
    }
}

fn map_reqwest_error(error: reqwest::Error) -> ProviderError {
    if error.is_timeout() {
        ProviderError::Timeout
    } else if error.is_decode() {
        ProviderError::InvalidResponse(error.to_string())
    } else {
        ProviderError::Transport(error.to_string())
    }
}

#[derive(Debug, Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<ProviderMessage>,
    stream: bool,
    max_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenRouterCompletionResponse {
    choices: Vec<OpenRouterChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterMessage,
}

#[derive(Debug, Deserialize)]
struct OpenRouterMessage {
    content: Option<String>,
}
