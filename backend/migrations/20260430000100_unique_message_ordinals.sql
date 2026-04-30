DROP INDEX IF EXISTS messages_conversation_ordinal_idx;

WITH ranked_messages AS (
    SELECT
        id,
        ROW_NUMBER() OVER (
            PARTITION BY conversation_id
            ORDER BY ordinal ASC, created_at ASC, id ASC
        ) - 1 AS next_ordinal
    FROM messages
)
UPDATE messages
SET ordinal = (
    SELECT next_ordinal
    FROM ranked_messages
    WHERE ranked_messages.id = messages.id
);

CREATE UNIQUE INDEX messages_conversation_ordinal_unique_idx
    ON messages (conversation_id, ordinal);
