use std::str::FromStr;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use uuid::Uuid;

use crate::models::{Conversation, Message};

pub async fn connect_and_migrate(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .foreign_keys(true);
    let max_connections = if database_url.contains(":memory:") {
        1
    } else {
        5
    };
    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect_with(options)
        .await?;

    run_migrations(&pool).await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|error| sqlx::Error::Migrate(Box::new(error)))
}

pub async fn create_conversation(
    pool: &SqlitePool,
    title: &str,
) -> Result<Conversation, sqlx::Error> {
    let id = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO conversations (id, title) VALUES (?, ?)")
        .bind(&id)
        .bind(title)
        .execute(pool)
        .await?;

    get_conversation(pool, &id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn get_conversation(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<Conversation>, sqlx::Error> {
    sqlx::query_as::<_, Conversation>(
        "SELECT id, title, created_at, updated_at FROM conversations WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn list_conversations(pool: &SqlitePool) -> Result<Vec<Conversation>, sqlx::Error> {
    sqlx::query_as::<_, Conversation>(
        "SELECT id, title, created_at, updated_at
         FROM conversations
         ORDER BY updated_at DESC, created_at DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_messages(
    pool: &SqlitePool,
    conversation_id: &str,
) -> Result<Vec<Message>, sqlx::Error> {
    sqlx::query_as::<_, Message>(
        "SELECT id, conversation_id, role, content, status, created_at, completed_at
         FROM messages
         WHERE conversation_id = ?
         ORDER BY ordinal ASC",
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await
}

pub async fn insert_message(
    pool: &SqlitePool,
    conversation_id: &str,
    role: &str,
    content: &str,
) -> Result<Message, sqlx::Error> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO messages (id, conversation_id, role, content, ordinal)
         VALUES (
            ?, ?, ?, ?,
            COALESCE(
                (SELECT MAX(ordinal) + 1 FROM messages WHERE conversation_id = ?),
                0
            )
         )",
    )
    .bind(&id)
    .bind(conversation_id)
    .bind(role)
    .bind(content)
    .bind(conversation_id)
    .execute(pool)
    .await?;

    touch_conversation(pool, conversation_id).await?;

    get_message(pool, &id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn insert_streaming_assistant_message(
    pool: &SqlitePool,
    conversation_id: &str,
) -> Result<Message, sqlx::Error> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO messages (id, conversation_id, role, content, status, ordinal, completed_at)
         VALUES (
            ?, ?, 'assistant', '', 'streaming',
            COALESCE(
                (SELECT MAX(ordinal) + 1 FROM messages WHERE conversation_id = ?),
                0
            ),
            NULL
         )",
    )
    .bind(&id)
    .bind(conversation_id)
    .bind(conversation_id)
    .execute(pool)
    .await?;

    touch_conversation(pool, conversation_id).await?;

    get_message(pool, &id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn update_message_status(
    pool: &SqlitePool,
    message_id: &str,
    content: &str,
    status: &str,
) -> Result<Message, sqlx::Error> {
    sqlx::query(
        "UPDATE messages
         SET content = ?,
             status = ?,
             completed_at = CASE
                WHEN ? IN ('completed', 'failed') THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                ELSE NULL
             END
         WHERE id = ?",
    )
    .bind(content)
    .bind(status)
    .bind(status)
    .bind(message_id)
    .execute(pool)
    .await?;

    get_message(pool, message_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn create_chat_request(
    pool: &SqlitePool,
    conversation_id: &str,
    user_message_id: &str,
) -> Result<String, sqlx::Error> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO chat_requests (id, conversation_id, user_message_id, status)
         VALUES (?, ?, ?, 'pending')",
    )
    .bind(&id)
    .bind(conversation_id)
    .bind(user_message_id)
    .execute(pool)
    .await?;

    Ok(id)
}

pub async fn mark_chat_request_succeeded(
    pool: &SqlitePool,
    request_id: &str,
    assistant_message_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE chat_requests
         SET status = 'succeeded',
             assistant_message_id = ?,
             error_code = NULL,
             completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(assistant_message_id)
    .bind(request_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_chat_request_failed(
    pool: &SqlitePool,
    request_id: &str,
    error_code: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE chat_requests
         SET status = 'failed',
             error_code = ?,
             completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(error_code)
    .bind(request_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn get_message(pool: &SqlitePool, id: &str) -> Result<Option<Message>, sqlx::Error> {
    sqlx::query_as::<_, Message>(
        "SELECT id, conversation_id, role, content, status, created_at, completed_at FROM messages WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

async fn touch_conversation(pool: &SqlitePool, conversation_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE conversations
         SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(conversation_id)
    .execute(pool)
    .await?;

    Ok(())
}
