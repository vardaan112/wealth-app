use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ImportBatchRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub account_id: Option<Uuid>,
    pub source: String,
    pub filename: Option<String>,
    pub imported_count: i32,
    pub skipped_count: i32,
    pub error_count: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

pub async fn create_import_batch(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    source: String,
) -> Result<ImportBatchRecord, sqlx::Error> {
    sqlx::query_as::<_, ImportBatchRecord>(
        r#"
        INSERT INTO import_batches (user_id, account_id, source, status)
        SELECT $1, accounts.id, $3, 'processing'
        FROM accounts
        WHERE accounts.id = $2
          AND accounts.user_id = $1
        RETURNING
            id,
            user_id,
            account_id,
            source,
            filename,
            imported_count,
            skipped_count,
            error_count,
            status,
            created_at
        "#,
    )
    .bind(user_id)
    .bind(account_id)
    .bind(source)
    .fetch_one(pool)
    .await
}

pub async fn finalize_import_batch(
    pool: &PgPool,
    id: Uuid,
    imported_count: i32,
    skipped_count: i32,
    error_count: i32,
    status: String,
) -> Result<ImportBatchRecord, sqlx::Error> {
    sqlx::query_as::<_, ImportBatchRecord>(
        r#"
        UPDATE import_batches
        SET
            imported_count = $2,
            skipped_count = $3,
            error_count = $4,
            status = $5
        WHERE id = $1
        RETURNING
            id,
            user_id,
            account_id,
            source,
            filename,
            imported_count,
            skipped_count,
            error_count,
            status,
            created_at
        "#,
    )
    .bind(id)
    .bind(imported_count)
    .bind(skipped_count)
    .bind(error_count)
    .bind(status)
    .fetch_one(pool)
    .await
}
