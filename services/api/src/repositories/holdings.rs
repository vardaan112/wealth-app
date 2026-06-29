use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HoldingRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub provider: String,
    pub provider_holding_id: Option<String>,
    pub symbol: String,
    pub asset_name: Option<String>,
    pub asset_type: String,
    pub quantity: f64,
    pub market_value_cents: Option<i64>,
    pub cost_basis_cents: Option<i64>,
    pub price_cents: Option<i64>,
    pub currency: String,
    pub as_of: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpsertHoldingInput {
    pub account_id: Uuid,
    pub provider: Option<String>,
    pub provider_holding_id: Option<String>,
    pub symbol: String,
    pub asset_name: Option<String>,
    pub asset_type: Option<String>,
    pub quantity: f64,
    pub market_value_cents: Option<i64>,
    pub cost_basis_cents: Option<i64>,
    pub price_cents: Option<i64>,
    pub currency: Option<String>,
}

pub async fn list_holdings(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<HoldingRecord>, sqlx::Error> {
    sqlx::query_as::<_, HoldingRecord>(
        r#"
        SELECT
            id,
            user_id,
            account_id,
            provider,
            provider_holding_id,
            symbol,
            asset_name,
            asset_type,
            quantity::DOUBLE PRECISION AS quantity,
            market_value_cents,
            cost_basis_cents,
            price_cents,
            currency,
            as_of,
            created_at,
            updated_at
        FROM holdings
        WHERE user_id = $1
        ORDER BY symbol ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn upsert_holding(
    pool: &PgPool,
    user_id: Uuid,
    input: UpsertHoldingInput,
) -> Result<Option<HoldingRecord>, sqlx::Error> {
    sqlx::query_as::<_, HoldingRecord>(
        r#"
        INSERT INTO holdings (
            user_id,
            account_id,
            provider,
            provider_holding_id,
            symbol,
            asset_name,
            asset_type,
            quantity,
            market_value_cents,
            cost_basis_cents,
            price_cents,
            currency
        )
        SELECT
            $1,
            accounts.id,
            COALESCE($3, 'manual'),
            $4,
            $5,
            $6,
            COALESCE($7, 'stock'),
            $8::NUMERIC,
            $9,
            $10,
            $11,
            COALESCE($12, 'USD')
        FROM accounts
        WHERE accounts.id = $2
          AND accounts.user_id = $1
        ON CONFLICT (account_id, symbol, asset_type)
        DO UPDATE SET
            provider = EXCLUDED.provider,
            provider_holding_id = EXCLUDED.provider_holding_id,
            asset_name = EXCLUDED.asset_name,
            quantity = EXCLUDED.quantity,
            market_value_cents = EXCLUDED.market_value_cents,
            cost_basis_cents = EXCLUDED.cost_basis_cents,
            price_cents = EXCLUDED.price_cents,
            currency = EXCLUDED.currency,
            as_of = NOW()
        WHERE holdings.user_id = EXCLUDED.user_id
        RETURNING
            id,
            user_id,
            account_id,
            provider,
            provider_holding_id,
            symbol,
            asset_name,
            asset_type,
            quantity::DOUBLE PRECISION AS quantity,
            market_value_cents,
            cost_basis_cents,
            price_cents,
            currency,
            as_of,
            created_at,
            updated_at
        "#,
    )
    .bind(user_id)
    .bind(input.account_id)
    .bind(input.provider)
    .bind(input.provider_holding_id)
    .bind(input.symbol)
    .bind(input.asset_name)
    .bind(input.asset_type)
    .bind(input.quantity)
    .bind(input.market_value_cents)
    .bind(input.cost_basis_cents)
    .bind(input.price_cents)
    .bind(input.currency)
    .fetch_optional(pool)
    .await
}
