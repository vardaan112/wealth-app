use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccountBalanceSnapshotRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub balance_cents: i64,
    pub available_balance_cents: Option<i64>,
    pub currency: String,
    pub snapshot_date: NaiveDate,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PortfolioSnapshotRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub cash_cents: i64,
    pub investment_value_cents: i64,
    pub debt_cents: i64,
    pub net_worth_cents: i64,
    pub currency: String,
    pub snapshot_date: NaiveDate,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

pub async fn list_account_balance_snapshots(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<AccountBalanceSnapshotRecord>, sqlx::Error> {
    sqlx::query_as::<_, AccountBalanceSnapshotRecord>(
        r#"
        SELECT
            id,
            user_id,
            account_id,
            balance_cents,
            available_balance_cents,
            currency,
            snapshot_date,
            source,
            created_at
        FROM account_balance_snapshots
        WHERE user_id = $1
        ORDER BY snapshot_date DESC, created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn list_portfolio_snapshots(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<PortfolioSnapshotRecord>, sqlx::Error> {
    sqlx::query_as::<_, PortfolioSnapshotRecord>(
        r#"
        SELECT
            id,
            user_id,
            cash_cents,
            investment_value_cents,
            debt_cents,
            net_worth_cents,
            currency,
            snapshot_date,
            source,
            created_at
        FROM portfolio_snapshots
        WHERE user_id = $1
        ORDER BY snapshot_date ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}
