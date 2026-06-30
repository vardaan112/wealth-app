use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChatMessageRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub content: String,
    pub is_briefing: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateChatMessageInput {
    pub role: String,
    pub content: String,
    pub is_briefing: bool,
}

pub async fn list_chat_messages(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<ChatMessageRecord>, sqlx::Error> {
    sqlx::query_as::<_, ChatMessageRecord>(
        r#"
        SELECT id, user_id, role, content, is_briefing, created_at
        FROM chat_messages
        WHERE user_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn create_chat_message(
    pool: &PgPool,
    user_id: Uuid,
    input: CreateChatMessageInput,
) -> Result<ChatMessageRecord, sqlx::Error> {
    sqlx::query_as::<_, ChatMessageRecord>(
        r#"
        INSERT INTO chat_messages (user_id, role, content, is_briefing)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, role, content, is_briefing, created_at
        "#,
    )
    .bind(user_id)
    .bind(input.role)
    .bind(input.content)
    .bind(input.is_briefing)
    .fetch_one(pool)
    .await
}

pub async fn last_briefing_at(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT created_at
        FROM chat_messages
        WHERE user_id = $1
          AND role = 'assistant'
          AND is_briefing = TRUE
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_input_accepts_briefing_flag() {
        let input = CreateChatMessageInput {
            role: "assistant".to_string(),
            content: "Hold AAPL".to_string(),
            is_briefing: true,
        };
        assert!(input.is_briefing);
        assert_eq!(input.role, "assistant");
    }
}
