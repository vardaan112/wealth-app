use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRecord {
    pub id: Uuid,
    pub email: String,
    pub password_hash: Option<String>,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn find_user_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<UserRecord>, sqlx::Error> {
    sqlx::query_as::<_, UserRecord>(
        r#"
        SELECT id, email, password_hash, display_name, created_at, updated_at
        FROM users
        WHERE LOWER(email) = LOWER($1)
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await
}

pub async fn find_user_by_id(pool: &PgPool, id: Uuid) -> Result<Option<UserRecord>, sqlx::Error> {
    sqlx::query_as::<_, UserRecord>(
        r#"
        SELECT id, email, password_hash, display_name, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn create_user(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
    display_name: Option<&str>,
) -> Result<UserRecord, sqlx::Error> {
    sqlx::query_as::<_, UserRecord>(
        r#"
        INSERT INTO users (email, password_hash, display_name)
        VALUES ($1, $2, $3)
        RETURNING id, email, password_hash, display_name, created_at, updated_at
        "#,
    )
    .bind(email)
    .bind(password_hash)
    .bind(display_name)
    .fetch_one(pool)
    .await
}

pub async fn upsert_single_user(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
) -> Result<UserRecord, sqlx::Error> {
    sqlx::query_as::<_, UserRecord>(
        r#"
        INSERT INTO users (email, password_hash, display_name)
        VALUES ($1, $2, 'Owner')
        ON CONFLICT (email)
        DO UPDATE SET
            password_hash = EXCLUDED.password_hash,
            updated_at = NOW()
        RETURNING id, email, password_hash, display_name, created_at, updated_at
        "#,
    )
    .bind(email)
    .bind(password_hash)
    .fetch_one(pool)
    .await
}
