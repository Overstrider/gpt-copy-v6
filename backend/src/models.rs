use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ChatRequest {
    pub id: String,
    pub conversation_id: String,
    pub user_message_id: String,
    pub assistant_message_id: Option<String>,
    pub status: String,
    pub error_code: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {
    pub title: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConversationResponse {
    pub conversation: Conversation,
}

#[derive(Debug, Serialize)]
pub struct ListConversationsResponse {
    pub conversations: Vec<Conversation>,
}

#[derive(Debug, Serialize)]
pub struct MessagesResponse {
    pub messages: Vec<Message>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct SendMessageResponse {
    pub user_message: Message,
    pub assistant_message: Message,
}
