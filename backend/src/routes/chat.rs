use std::convert::Infallible;

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::{Stream, StreamExt};

use crate::db;
use crate::error::AppError;
use crate::models::{SendMessageRequest, SendMessageResponse};
use crate::provider::{ChatProviderRequest, ChatProviderResponse, ProviderMessage};
use crate::state::AppState;

use super::conversations::{ensure_conversation, parse_conversation_id};

const MAX_MESSAGE_LEN: usize = 16_000;
const MAX_ASSISTANT_LEN: usize = 64_000;

pub async fn send_message(
    Path(id): Path<String>,
    State(state): State<AppState>,
    payload: Result<Json<SendMessageRequest>, JsonRejection>,
) -> Result<Json<SendMessageResponse>, AppError> {
    let payload = payload.map_err(AppError::from)?.0;
    let conversation_id = parse_conversation_id(&id)?;
    ensure_conversation(&state, &conversation_id).await?;
    let content = normalize_message_content(payload.content)?;
    let (user_message_id, user_message, request_id) =
        persist_user_request(&state, &conversation_id, &content).await?;
    let provider_request = provider_request_for_conversation(&state, &conversation_id).await?;

    let provider_response = match state.provider.complete(provider_request).await {
        Ok(response) => response,
        Err(error) => {
            let app_error = AppError::from(error);
            db::mark_chat_request_failed(&state.pool, &request_id, app_error.code()).await?;
            return Err(app_error);
        }
    };

    let assistant_content = validate_provider_response(provider_response)?;
    let assistant_message = db::insert_message(
        &state.pool,
        &conversation_id,
        "assistant",
        &assistant_content,
    )
    .await?;
    db::mark_chat_request_succeeded(&state.pool, &request_id, &assistant_message.id).await?;

    tracing::debug!(%conversation_id, %user_message_id, "chat message completed");

    Ok(Json(SendMessageResponse {
        user_message,
        assistant_message,
    }))
}

pub async fn stream_message(
    Path(id): Path<String>,
    State(state): State<AppState>,
    payload: Result<Json<SendMessageRequest>, JsonRejection>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let payload = payload.map_err(AppError::from)?.0;
    let conversation_id = parse_conversation_id(&id)?;
    ensure_conversation(&state, &conversation_id).await?;
    let content = normalize_message_content(payload.content)?;
    let (_, _, request_id) = persist_user_request(&state, &conversation_id, &content).await?;
    let provider_request = provider_request_for_conversation(&state, &conversation_id).await?;

    let provider_stream = match state.provider.stream(provider_request).await {
        Ok(stream) => stream,
        Err(error) => {
            let app_error = AppError::from(error);
            db::mark_chat_request_failed(&state.pool, &request_id, app_error.code()).await?;
            return Err(app_error);
        }
    };

    let pool = state.pool.clone();
    let conversation_id = conversation_id.clone();
    let stream = async_stream::stream! {
        let mut assistant_content = String::new();
        futures_util::pin_mut!(provider_stream);
        let assistant_message = match db::insert_streaming_assistant_message(&pool, &conversation_id).await {
            Ok(message) => message,
            Err(db_error) => {
                let app_error = AppError::from(db_error);
                yield Ok(error_event(&app_error));
                return;
            }
        };

        yield Ok(json_event(
            "message_start",
            serde_json::json!({ "message": assistant_message }),
        ));

        while let Some(chunk) = provider_stream.next().await {
            match chunk {
                Ok(chunk) => {
                    assistant_content.push_str(&chunk);
                    if assistant_content.chars().count() > MAX_ASSISTANT_LEN {
                        let app_error = AppError::from(crate::provider::ProviderError::InvalidResponse(
                            format!("assistant content exceeded {MAX_ASSISTANT_LEN} characters"),
                        ));
                        if let Err(db_error) =
                            db::update_message_status(&pool, &assistant_message.id, &assistant_content, "failed").await
                        {
                            tracing::error!(error = ?db_error, "failed to mark oversized stream assistant failed");
                        }
                        if let Err(db_error) =
                            db::mark_chat_request_failed(&pool, &request_id, app_error.code()).await
                        {
                            tracing::error!(error = ?db_error, "failed to mark oversized stream request failed");
                        }
                        yield Ok(error_event(&app_error));
                        return;
                    }
                    yield Ok(json_event("delta", serde_json::json!({ "content": chunk })));
                }
                Err(error) => {
                    let app_error = AppError::from(error);
                    if let Err(db_error) =
                        db::update_message_status(&pool, &assistant_message.id, &assistant_content, "failed").await
                    {
                        tracing::error!(error = ?db_error, "failed to mark stream assistant failed");
                    }
                    if let Err(db_error) =
                        db::mark_chat_request_failed(&pool, &request_id, app_error.code()).await
                    {
                        tracing::error!(error = ?db_error, "failed to mark stream request failed");
                    }
                    yield Ok(error_event(&app_error));
                    return;
                }
            }
        }

        if assistant_content.trim().is_empty() {
            let app_error = AppError::from(crate::provider::ProviderError::InvalidResponse(
                "empty streamed assistant content".to_owned(),
            ));
            if let Err(db_error) =
                db::update_message_status(&pool, &assistant_message.id, &assistant_content, "failed").await
            {
                tracing::error!(error = ?db_error, "failed to mark empty stream assistant failed");
            }
            if let Err(db_error) =
                db::mark_chat_request_failed(&pool, &request_id, app_error.code()).await
            {
                tracing::error!(error = ?db_error, "failed to mark empty stream failed");
            }
            yield Ok(error_event(&app_error));
            return;
        }

        match db::update_message_status(&pool, &assistant_message.id, &assistant_content, "completed").await {
            Ok(assistant_message) => {
                if let Err(db_error) =
                    db::mark_chat_request_succeeded(&pool, &request_id, &assistant_message.id).await
                {
                    tracing::error!(error = ?db_error, "failed to mark stream request succeeded");
                    let app_error = AppError::from(db_error);
                    yield Ok(error_event(&app_error));
                    return;
                }

                yield Ok(json_event(
                    "message_complete",
                    serde_json::json!({ "message": assistant_message }),
                ));
            }
            Err(db_error) => {
                tracing::error!(error = ?db_error, "failed to persist streamed assistant message");
                let app_error = AppError::from(db_error);
                yield Ok(error_event(&app_error));
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

fn normalize_message_content(content: String) -> Result<String, AppError> {
    let content = content.trim();
    if content.is_empty() {
        return Err(AppError::validation("content must not be empty"));
    }
    if content.chars().count() > MAX_MESSAGE_LEN {
        return Err(AppError::validation(format!(
            "content must be {MAX_MESSAGE_LEN} characters or fewer"
        )));
    }

    Ok(content.to_owned())
}

async fn persist_user_request(
    state: &AppState,
    conversation_id: &str,
    content: &str,
) -> Result<(String, crate::models::Message, String), AppError> {
    let user_message = db::insert_message(&state.pool, conversation_id, "user", content).await?;
    let request_id =
        db::create_chat_request(&state.pool, conversation_id, &user_message.id).await?;

    Ok((user_message.id.clone(), user_message, request_id))
}

async fn provider_request_for_conversation(
    state: &AppState,
    conversation_id: &str,
) -> Result<ChatProviderRequest, AppError> {
    let messages = db::list_messages_for_provider(&state.pool, conversation_id)
        .await?
        .into_iter()
        .map(|message| ProviderMessage {
            role: message.role,
            content: message.content,
        })
        .collect();

    Ok(ChatProviderRequest {
        model: state.config.openrouter_model.clone(),
        messages,
    })
}

fn validate_provider_response(response: ChatProviderResponse) -> Result<String, AppError> {
    if response.content.trim().is_empty() {
        return Err(AppError::from(
            crate::provider::ProviderError::InvalidResponse("empty assistant content".to_owned()),
        ));
    }
    if response.content.chars().count() > MAX_ASSISTANT_LEN {
        return Err(AppError::from(
            crate::provider::ProviderError::InvalidResponse(format!(
                "assistant content exceeded {MAX_ASSISTANT_LEN} characters"
            )),
        ));
    }

    Ok(response.content)
}

fn error_event(error: &AppError) -> Event {
    let body = serde_json::to_string(&error.response_body()).unwrap_or_else(|_| {
        r#"{"error":{"code":"internal_error","message":"failed to encode error"}}"#.to_owned()
    });

    Event::default().event("error").data(body)
}

fn json_event(event: &'static str, body: serde_json::Value) -> Event {
    Event::default().event(event).data(body.to_string())
}
