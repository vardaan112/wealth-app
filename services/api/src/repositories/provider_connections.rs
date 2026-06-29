use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct ProviderConnectionRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_item_id: Option<String>,
    pub provider_user_id: Option<String>,
    pub encrypted_access_token: Option<String>,
    pub encrypted_refresh_token: Option<String>,
    pub encrypted_user_secret: Option<String>,
    pub sync_cursor: Option<String>,
    pub status: String,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct UpsertProviderConnectionInput<'a> {
    pub user_id: Uuid,
    pub provider: &'a str,
    pub provider_item_id: Option<&'a str>,
    pub provider_user_id: Option<&'a str>,
    pub encrypted_access_token: Option<&'a str>,
    pub encrypted_refresh_token: Option<&'a str>,
    pub encrypted_user_secret: Option<&'a str>,
    pub sync_cursor: Option<&'a str>,
    pub status: &'a str,
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
            provider_item_id,
            provider_user_id,
            encrypted_access_token,
            encrypted_refresh_token,
            encrypted_user_secret,
            sync_cursor,
            status,
            last_synced_at,
            created_at,
            updated_at
        FROM provider_connections
        WHERE user_id = $1
          AND provider = $2
          AND status = 'active'
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
            provider_item_id,
            provider_user_id,
            encrypted_access_token,
            encrypted_refresh_token,
            encrypted_user_secret,
            sync_cursor,
            status,
            last_synced_at,
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
    input: UpsertProviderConnectionInput<'_>,
) -> Result<Uuid, sqlx::Error> {
    if let Some(provider_item_id) = input.provider_item_id {
        if let Some(id) =
            update_provider_connection_by_item_id(pool, &input, provider_item_id).await?
        {
            return Ok(id);
        }
    } else if let Some(provider_user_id) = input.provider_user_id {
        if let Some(id) =
            update_provider_connection_by_user_id(pool, &input, provider_user_id).await?
        {
            return Ok(id);
        }
    }

    sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO provider_connections (
            user_id,
            provider,
            provider_item_id,
            provider_user_id,
            encrypted_access_token,
            encrypted_refresh_token,
            encrypted_user_secret,
            sync_cursor,
            status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
        "#,
    )
    .bind(input.user_id)
    .bind(input.provider)
    .bind(input.provider_item_id)
    .bind(input.provider_user_id)
    .bind(input.encrypted_access_token)
    .bind(input.encrypted_refresh_token)
    .bind(input.encrypted_user_secret)
    .bind(input.sync_cursor)
    .bind(input.status)
    .fetch_one(pool)
    .await
}

/// Records a successful Plaid fetch through `sync_end_date` (ISO `YYYY-MM-DD`).
/// The cursor drives incremental `/transactions/get` windows on subsequent syncs.
pub async fn update_plaid_sync_cursor(
    pool: &PgPool,
    connection_id: Uuid,
    sync_end_date: NaiveDate,
) -> Result<(), sqlx::Error> {
    let cursor = sync_end_date.format("%Y-%m-%d").to_string();

    sqlx::query(
        r#"
        UPDATE provider_connections
        SET
            sync_cursor = $2,
            last_synced_at = NOW(),
            status = 'active',
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(connection_id)
    .bind(cursor)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_connection_synced(
    pool: &PgPool,
    connection_id: Uuid,
    status: &str,
    touch_last_synced: bool,
) -> Result<(), sqlx::Error> {
    let query = if touch_last_synced {
        r#"
        UPDATE provider_connections
        SET status = $2, last_synced_at = NOW(), updated_at = NOW()
        WHERE id = $1
        "#
    } else {
        r#"
        UPDATE provider_connections
        SET status = $2, updated_at = NOW()
        WHERE id = $1
        "#
    };

    sqlx::query(query)
        .bind(connection_id)
        .bind(status)
        .execute(pool)
        .await?;

    Ok(())
}

async fn update_provider_connection_by_item_id(
    pool: &PgPool,
    input: &UpsertProviderConnectionInput<'_>,
    provider_item_id: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        UPDATE provider_connections
        SET
            provider_user_id = $4,
            encrypted_access_token = $5,
            encrypted_refresh_token = $6,
            encrypted_user_secret = $7,
            sync_cursor = $8,
            status = $9,
            updated_at = NOW()
        WHERE user_id = $1
          AND provider = $2
          AND provider_item_id = $3
        RETURNING id
        "#,
    )
    .bind(input.user_id)
    .bind(input.provider)
    .bind(provider_item_id)
    .bind(input.provider_user_id)
    .bind(input.encrypted_access_token)
    .bind(input.encrypted_refresh_token)
    .bind(input.encrypted_user_secret)
    .bind(input.sync_cursor)
    .bind(input.status)
    .fetch_optional(pool)
    .await
}

async fn update_provider_connection_by_user_id(
    pool: &PgPool,
    input: &UpsertProviderConnectionInput<'_>,
    provider_user_id: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        UPDATE provider_connections
        SET
            provider_item_id = $4,
            encrypted_access_token = $5,
            encrypted_refresh_token = $6,
            encrypted_user_secret = $7,
            sync_cursor = $8,
            status = $9,
            updated_at = NOW()
        WHERE user_id = $1
          AND provider = $2
          AND provider_user_id = $3
        RETURNING id
        "#,
    )
    .bind(input.user_id)
    .bind(input.provider)
    .bind(provider_user_id)
    .bind(input.provider_item_id)
    .bind(input.encrypted_access_token)
    .bind(input.encrypted_refresh_token)
    .bind(input.encrypted_user_secret)
    .bind(input.sync_cursor)
    .bind(input.status)
    .fetch_optional(pool)
    .await
}
