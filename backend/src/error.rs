use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;

use crate::provider::ProviderError;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("invalid request body: {0}")]
    InvalidJson(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl AppError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "validation_failed",
            Self::InvalidJson(_) => "invalid_json",
            Self::BadRequest(_) => "bad_request",
            Self::NotFound(_) => "not_found",
            Self::Provider(error) => provider_error_code(error),
            Self::Database(_) => "database_error",
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::InvalidJson(_) | Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Provider(error) => provider_error_status(error),
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn client_message(&self) -> String {
        match self {
            Self::Validation(message)
            | Self::InvalidJson(message)
            | Self::BadRequest(message)
            | Self::NotFound(message) => message.clone(),
            Self::Provider(error) => provider_error_message(error),
            Self::Database(_) => "A database error occurred".to_owned(),
        }
    }

    pub fn response_body(&self) -> ErrorResponse {
        ErrorResponse {
            error: ErrorBody {
                code: self.code().to_owned(),
                message: self.client_message(),
            },
        }
    }
}

impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        Self::InvalidJson(rejection.body_text())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if matches!(self, Self::Database(_)) {
            tracing::error!(error = ?self, "request failed");
        }

        (self.status(), Json(self.response_body())).into_response()
    }
}

fn provider_error_code(error: &ProviderError) -> &'static str {
    match error {
        ProviderError::MissingApiKey => "provider_not_configured",
        ProviderError::RateLimited => "provider_rate_limited",
        ProviderError::Timeout => "provider_timeout",
        ProviderError::InvalidResponse(_) => "provider_invalid_response",
        ProviderError::Interrupted(_) => "provider_stream_interrupted",
        ProviderError::Transport(_) | ProviderError::UpstreamStatus { .. } => "provider_error",
    }
}

fn provider_error_status(error: &ProviderError) -> StatusCode {
    match error {
        ProviderError::MissingApiKey => StatusCode::SERVICE_UNAVAILABLE,
        ProviderError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
        ProviderError::Timeout => StatusCode::GATEWAY_TIMEOUT,
        ProviderError::InvalidResponse(_)
        | ProviderError::Interrupted(_)
        | ProviderError::Transport(_)
        | ProviderError::UpstreamStatus { .. } => StatusCode::BAD_GATEWAY,
    }
}

fn provider_error_message(error: &ProviderError) -> String {
    match error {
        ProviderError::MissingApiKey => "OpenRouter is not configured".to_owned(),
        ProviderError::RateLimited => "OpenRouter rate limit exceeded".to_owned(),
        ProviderError::Timeout => "OpenRouter request timed out".to_owned(),
        ProviderError::InvalidResponse(_) => "OpenRouter returned an invalid response".to_owned(),
        ProviderError::Interrupted(_) => "OpenRouter stream was interrupted".to_owned(),
        ProviderError::Transport(_) => "OpenRouter request failed".to_owned(),
        ProviderError::UpstreamStatus { status, .. } => {
            format!("OpenRouter returned HTTP status {status}")
        }
    }
}
