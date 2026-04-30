use std::{str::FromStr, time::Duration};

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use uuid::Uuid;

use crate::models::{ChatRequest, Conversation, Message};

pub async fn connect_and_migrate(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));
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

pub async fn list_messages_for_provider(
    pool: &SqlitePool,
    conversation_id: &str,
) -> Result<Vec<Message>, sqlx::Error> {
    sqlx::query_as::<_, Message>(
        "SELECT id, conversation_id, role, content, status, created_at, completed_at
         FROM messages
         WHERE conversation_id = ?
           AND status = 'completed'
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

    insert_message_row(pool, &id, conversation_id, role, content, "completed").await?;

    get_message(pool, &id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn insert_streaming_assistant_message(
    pool: &SqlitePool,
    conversation_id: &str,
) -> Result<Message, sqlx::Error> {
    let id = Uuid::new_v4().to_string();

    insert_message_row(pool, &id, conversation_id, "assistant", "", "streaming").await?;

    get_message(pool, &id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

async fn insert_message_row(
    pool: &SqlitePool,
    id: &str,
    conversation_id: &str,
    role: &str,
    content: &str,
    status: &str,
) -> Result<(), sqlx::Error> {
    let mut connection = pool.acquire().await?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *connection)
        .await?;

    let result = async {
        let ordinal = sqlx::query_scalar::<_, i64>(
            "SELECT COALESCE(MAX(ordinal) + 1, 0)
             FROM messages
             WHERE conversation_id = ?",
        )
        .bind(conversation_id)
        .fetch_one(&mut *connection)
        .await?;

        sqlx::query(
            "INSERT INTO messages
                (id, conversation_id, role, content, status, ordinal)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(conversation_id)
        .bind(role)
        .bind(content)
        .bind(status)
        .bind(ordinal)
        .execute(&mut *connection)
        .await?;

        sqlx::query(
            "UPDATE conversations
             SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?",
        )
        .bind(conversation_id)
        .execute(&mut *connection)
        .await?;

        Ok::<(), sqlx::Error>(())
    }
    .await;

    if let Err(error) = result {
        let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
        return Err(error);
    }

    if let Err(error) = sqlx::query("COMMIT").execute(&mut *connection).await {
        let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
        return Err(error);
    }

    Ok(())
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

pub async fn get_chat_request(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<ChatRequest>, sqlx::Error> {
    sqlx::query_as::<_, ChatRequest>(
        "SELECT id,
                conversation_id,
                user_message_id,
                assistant_message_id,
                status,
                error_code,
                created_at,
                completed_at
         FROM chat_requests
         WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn mark_chat_request_succeeded(
    pool: &SqlitePool,
    request_id: &str,
    assistant_message_id: &str,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query(
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
    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

pub async fn mark_chat_request_failed(
    pool: &SqlitePool,
    request_id: &str,
    error_code: &str,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query(
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
    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    // These tests pin the repository helpers directly so the task's filtered
    // verification commands exercise SQLite persistence instead of only routes.
    #[tokio::test]
    async fn db_repository_helpers_persist_conversation_messages_and_request_status() {
        let pool = connect_and_migrate("sqlite::memory:").await.unwrap();

        let conversation = create_conversation(&pool, "Persistence test")
            .await
            .unwrap();
        let conversations = list_conversations(&pool).await.unwrap();
        assert_eq!(conversations.len(), 1);
        assert_eq!(conversations[0].id, conversation.id);

        let user_message = insert_message(&pool, &conversation.id, "user", "hello")
            .await
            .unwrap();
        let request_id = create_chat_request(&pool, &conversation.id, &user_message.id)
            .await
            .unwrap();
        let pending_request = get_chat_request(&pool, &request_id).await.unwrap().unwrap();
        assert_eq!(pending_request.status, "pending");
        assert!(pending_request.completed_at.is_none());

        let assistant_message = insert_message(&pool, &conversation.id, "assistant", "hi")
            .await
            .unwrap();
        mark_chat_request_succeeded(&pool, &request_id, &assistant_message.id)
            .await
            .unwrap();
        let succeeded_request = get_chat_request(&pool, &request_id).await.unwrap().unwrap();
        assert_eq!(succeeded_request.status, "succeeded");
        assert_eq!(
            succeeded_request.assistant_message_id.as_deref(),
            Some(assistant_message.id.as_str())
        );
        assert!(succeeded_request.completed_at.is_some());

        let failed_user_message = insert_message(&pool, &conversation.id, "user", "retry")
            .await
            .unwrap();
        let failed_request_id =
            create_chat_request(&pool, &conversation.id, &failed_user_message.id)
                .await
                .unwrap();
        mark_chat_request_failed(&pool, &failed_request_id, "provider_rate_limited")
            .await
            .unwrap();
        let failed_request = get_chat_request(&pool, &failed_request_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(failed_request.status, "failed");
        assert_eq!(
            failed_request.error_code.as_deref(),
            Some("provider_rate_limited")
        );
        assert!(failed_request.assistant_message_id.is_none());
        assert!(failed_request.completed_at.is_some());

        let messages = list_messages(&pool, &conversation.id).await.unwrap();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].id, user_message.id);
        assert_eq!(messages[1].id, assistant_message.id);
        assert_eq!(messages[2].id, failed_user_message.id);
    }

    #[tokio::test]
    async fn persistence_in_memory_databases_are_isolated_per_pool() {
        let first_pool = connect_and_migrate("sqlite::memory:").await.unwrap();
        let second_pool = connect_and_migrate("sqlite::memory:").await.unwrap();

        create_conversation(&first_pool, "first").await.unwrap();

        let first_conversations = list_conversations(&first_pool).await.unwrap();
        let second_conversations = list_conversations(&second_pool).await.unwrap();
        assert_eq!(first_conversations.len(), 1);
        assert!(second_conversations.is_empty());
    }
}
