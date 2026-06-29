use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct ProviderConnectionRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub external_item_id: Option<String>,
    pub encrypted_access_token: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn list_provider_connections(
    pool: &PgPool,
    user_id: Uuid,
    provider: &str,
) -> Result<Vec<ProviderConnectionRecord>, sqlx::Error> {
    sqlx::query_as::<_, ProviderConnectionRecord>(
        r#"
        SELECT
            id,
            user_id,
            provider,
            external_item_id,
            encrypted_access_token,
            status,
            created_at,
            updated_at
        FROM provider_connections
        WHERE user_id = $1
          AND provider = $2
          AND status = 'connected'
        ORDER BY created_at ASC
        "#,
    )
    .bind(user_id)
    .bind(provider)
    .fetch_all(pool)
    .await
}

pub async fn find_provider_connection(
    pool: &PgPool,
    user_id: Uuid,
    provider: &str,
) -> Result<Option<ProviderConnectionRecord>, sqlx::Error> {
    sqlx::query_as::<_, ProviderConnectionRecord>(
        r#"
        SELECT
            id,
            user_id,
            provider,
            external_item_id,
            encrypted_access_token,
            status,
            created_at,
            updated_at
        FROM provider_connections
        WHERE user_id = $1
          AND provider = $2
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .bind(provider)
    .fetch_optional(pool)
    .await
}

pub async fn upsert_provider_connection(
    pool: &PgPool,
    user_id: Uuid,
    provider: &str,
    external_item_id: Option<&str>,
    encrypted_access_token: &str,
    status: &str,
) -> Result<Uuid, sqlx::Error> {
    if let Some(external_item_id) = external_item_id {
        if let Some(id) = update_provider_connection(
            pool,
            user_id,
            provider,
            external_item_id,
            encrypted_access_token,
            status,
        )
        .await?
        {
            return Ok(id);
        }
    }

    sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO provider_connections (
            user_id,
            provider,
            external_item_id,
            encrypted_access_token,
            status
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(provider)
    .bind(external_item_id)
    .bind(encrypted_access_token)
    .bind(status)
    .fetch_one(pool)
    .await
}

async fn update_provider_connection(
    pool: &PgPool,
    user_id: Uuid,
    provider: &str,
    external_item_id: &str,
    encrypted_access_token: &str,
    status: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        UPDATE provider_connections
        SET
            encrypted_access_token = $4,
            status = $5,
            updated_at = NOW()
        WHERE user_id = $1
          AND provider = $2
          AND external_item_id = $3
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(provider)
    .bind(external_item_id)
    .bind(encrypted_access_token)
    .bind(status)
    .fetch_optional(pool)
    .await
}
