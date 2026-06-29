use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TransactionRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub provider: String,
    pub provider_transaction_id: Option<String>,
    pub amount_cents: i64,
    pub currency: String,
    pub merchant_name: Option<String>,
    pub raw_description: Option<String>,
    pub category_primary: Option<String>,
    pub category_detailed: Option<String>,
    pub transaction_date: NaiveDate,
    pub authorized_date: Option<NaiveDate>,
    pub pending: bool,
    pub transaction_type: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTransactionInput {
    pub account_id: Uuid,
    pub provider: Option<String>,
    pub provider_transaction_id: Option<String>,
    pub amount_cents: i64,
    pub currency: Option<String>,
    pub merchant_name: Option<String>,
    pub raw_description: Option<String>,
    pub category_primary: Option<String>,
    pub category_detailed: Option<String>,
    pub transaction_date: NaiveDate,
    pub authorized_date: Option<NaiveDate>,
    pub pending: Option<bool>,
    pub transaction_type: Option<String>,
    pub notes: Option<String>,
}

pub async fn list_transactions(
    pool: &PgPool,
    user_id: Uuid,
    month: Option<String>,
) -> Result<Vec<TransactionRecord>, sqlx::Error> {
    sqlx::query_as::<_, TransactionRecord>(
        r#"
        SELECT
            id,
            user_id,
            account_id,
            provider,
            provider_transaction_id,
            amount_cents,
            currency,
            merchant_name,
            raw_description,
            category_primary,
            category_detailed,
            transaction_date,
            authorized_date,
            pending,
            transaction_type,
            notes,
            created_at,
            updated_at
        FROM transactions
        WHERE user_id = $1
          AND ($2::TEXT IS NULL OR to_char(transaction_date, 'YYYY-MM') = $2)
        ORDER BY transaction_date DESC, created_at DESC
        "#,
    )
    .bind(user_id)
    .bind(month)
    .fetch_all(pool)
    .await
}

pub async fn create_transaction(
    pool: &PgPool,
    user_id: Uuid,
    input: CreateTransactionInput,
) -> Result<TransactionRecord, sqlx::Error> {
    sqlx::query_as::<_, TransactionRecord>(
        r#"
        INSERT INTO transactions (
            user_id,
            account_id,
            provider,
            provider_transaction_id,
            amount_cents,
            currency,
            merchant_name,
            raw_description,
            category_primary,
            category_detailed,
            transaction_date,
            authorized_date,
            pending,
            transaction_type,
            notes
        )
        SELECT
            $1,
            accounts.id,
            COALESCE($3, 'manual'),
            $4,
            $5,
            COALESCE($6, 'USD'),
            $7,
            $8,
            $9,
            $10,
            $11,
            $12,
            COALESCE($13, FALSE),
            COALESCE($14, 'expense'),
            $15
        FROM accounts
        WHERE accounts.id = $2
          AND accounts.user_id = $1
        RETURNING
            id,
            user_id,
            account_id,
            provider,
            provider_transaction_id,
            amount_cents,
            currency,
            merchant_name,
            raw_description,
            category_primary,
            category_detailed,
            transaction_date,
            authorized_date,
            pending,
            transaction_type,
            notes,
            created_at,
            updated_at
        "#,
    )
    .bind(user_id)
    .bind(input.account_id)
    .bind(input.provider)
    .bind(input.provider_transaction_id)
    .bind(input.amount_cents)
    .bind(input.currency)
    .bind(input.merchant_name)
    .bind(input.raw_description)
    .bind(input.category_primary)
    .bind(input.category_detailed)
    .bind(input.transaction_date)
    .bind(input.authorized_date)
    .bind(input.pending)
    .bind(input.transaction_type)
    .bind(input.notes)
    .fetch_one(pool)
    .await
}

pub async fn update_transaction_category(
    pool: &PgPool,
    user_id: Uuid,
    transaction_id: Uuid,
    category_primary: String,
    category_detailed: Option<String>,
) -> Result<Option<TransactionRecord>, sqlx::Error> {
    sqlx::query_as::<_, TransactionRecord>(
        r#"
        UPDATE transactions
        SET
            category_primary = $3,
            category_detailed = $4
        WHERE user_id = $1
          AND id = $2
        RETURNING
            id,
            user_id,
            account_id,
            provider,
            provider_transaction_id,
            amount_cents,
            currency,
            merchant_name,
            raw_description,
            category_primary,
            category_detailed,
            transaction_date,
            authorized_date,
            pending,
            transaction_type,
            notes,
            created_at,
            updated_at
        "#,
    )
    .bind(user_id)
    .bind(transaction_id)
    .bind(category_primary)
    .bind(category_detailed)
    .fetch_optional(pool)
    .await
}
