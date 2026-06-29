use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccountRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_account_id: Option<String>,
    pub account_type: String,
    pub name: String,
    pub official_name: Option<String>,
    pub mask: Option<String>,
    pub currency: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAccountInput {
    pub provider: Option<String>,
    pub provider_account_id: Option<String>,
    pub account_type: String,
    pub name: String,
    pub official_name: Option<String>,
    pub mask: Option<String>,
    pub currency: Option<String>,
}

pub async fn list_accounts(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<AccountRecord>, sqlx::Error> {
    sqlx::query_as::<_, AccountRecord>(
        r#"
        SELECT
            id,
            user_id,
            provider,
            provider_account_id,
            account_type,
            name,
            official_name,
            mask,
            currency,
            is_active,
            created_at,
            updated_at
        FROM accounts
        WHERE user_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn create_account(
    pool: &PgPool,
    user_id: Uuid,
    input: CreateAccountInput,
) -> Result<AccountRecord, sqlx::Error> {
    sqlx::query_as::<_, AccountRecord>(
        r#"
        INSERT INTO accounts (
            user_id,
            provider,
            provider_account_id,
            account_type,
            name,
            official_name,
            mask,
            currency
        )
        VALUES (
            $1,
            COALESCE($2, 'manual'),
            $3,
            $4,
            $5,
            $6,
            $7,
            COALESCE($8, 'USD')
        )
        RETURNING
            id,
            user_id,
            provider,
            provider_account_id,
            account_type,
            name,
            official_name,
            mask,
            currency,
            is_active,
            created_at,
            updated_at
        "#,
    )
    .bind(user_id)
    .bind(input.provider)
    .bind(input.provider_account_id)
    .bind(input.account_type)
    .bind(input.name)
    .bind(input.official_name)
    .bind(input.mask)
    .bind(input.currency)
    .fetch_one(pool)
    .await
}
