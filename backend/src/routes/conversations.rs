use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::db;
use crate::error::AppError;
use crate::models::{
    ConversationResponse, CreateConversationRequest, ListConversationsResponse, MessagesResponse,
};
use crate::state::AppState;

const DEFAULT_CONVERSATION_TITLE: &str = "New conversation";
const MAX_TITLE_LEN: usize = 120;

pub async fn list_conversations(
    State(state): State<AppState>,
) -> Result<Json<ListConversationsResponse>, AppError> {
    let mut conversations = db::list_conversations(&state.pool).await?;
    conversations.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| right.created_at.cmp(&left.created_at))
            .then_with(|| left.id.cmp(&right.id))
    });

    Ok(Json(ListConversationsResponse { conversations }))
}

pub async fn create_conversation(
    State(state): State<AppState>,
    payload: Result<Json<CreateConversationRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<ConversationResponse>), AppError> {
    let payload = payload.map_err(AppError::from)?.0;
    let title = normalize_title(payload.title)?;
    let conversation = db::create_conversation(&state.pool, &title).await?;

    Ok((
        StatusCode::CREATED,
        Json(ConversationResponse { conversation }),
    ))
}

pub async fn list_messages(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<MessagesResponse>, AppError> {
    let conversation_id = parse_conversation_id(&id)?;
    ensure_conversation(&state, &conversation_id).await?;
    let messages = db::list_messages(&state.pool, &conversation_id).await?;

    Ok(Json(MessagesResponse { messages }))
}

pub async fn ensure_conversation(state: &AppState, conversation_id: &str) -> Result<(), AppError> {
    db::get_conversation(&state.pool, conversation_id)
        .await?
        .ok_or_else(|| AppError::not_found("conversation not found"))?;

    Ok(())
}

pub fn parse_conversation_id(id: &str) -> Result<String, AppError> {
    Uuid::parse_str(id).map_err(|_| AppError::bad_request("conversation id must be a UUID"))?;

    Ok(id.to_owned())
}

fn normalize_title(title: Option<String>) -> Result<String, AppError> {
    match title {
        Some(title) => {
            let title = title.trim();
            if title.is_empty() {
                return Err(AppError::validation("title must not be empty"));
            }
            if title.chars().count() > MAX_TITLE_LEN {
                return Err(AppError::validation(format!(
                    "title must be {MAX_TITLE_LEN} characters or fewer"
                )));
            }

            Ok(title.to_owned())
        }
        None => Ok(DEFAULT_CONVERSATION_TITLE.to_owned()),
    }
}
