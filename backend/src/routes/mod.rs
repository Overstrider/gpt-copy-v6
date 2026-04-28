mod chat;
mod conversations;
mod health;

use axum::Router;
use axum::routing::{get, post};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route(
            "/conversations",
            get(conversations::list_conversations).post(conversations::create_conversation),
        )
        .route(
            "/conversations/{id}/messages",
            get(conversations::list_messages).post(chat::send_message),
        )
        .route(
            "/conversations/{id}/messages/stream",
            post(chat::stream_message),
        )
        .with_state(state)
}
