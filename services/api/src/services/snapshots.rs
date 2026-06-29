use chrono::{NaiveDate, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct CurrentNetWorth {
    pub cash_cents: i64,
    pub investment_value_cents: i64,
    pub debt_cents: i64,
    pub net_worth_cents: i64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct PortfolioSnapshot {
    pub id: Uuid,
    pub user_id: Uuid,
    pub cash_cents: i64,
    pub investment_value_cents: i64,
    pub debt_cents: i64,
    pub net_worth_cents: i64,
    pub currency: String,
    pub snapshot_date: NaiveDate,
    pub source: String,
}

#[derive(Debug, FromRow)]
struct AccountBalanceInput {
    account_type: String,
    snapshot_balance_cents: Option<i64>,
    transaction_balance_cents: i64,
}

pub async fn compute_current_net_worth(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<CurrentNetWorth, sqlx::Error> {
    let account_balances = sqlx::query_as::<_, AccountBalanceInput>(
        r#"
        SELECT
            accounts.account_type,
            latest_balance.balance_cents AS snapshot_balance_cents,
            COALESCE(transaction_balances.balance_cents, 0)::BIGINT AS transaction_balance_cents
        FROM accounts
        LEFT JOIN LATERAL (
            SELECT balance_cents
            FROM account_balance_snapshots
            WHERE account_balance_snapshots.account_id = accounts.id
            ORDER BY snapshot_date DESC, created_at DESC
            LIMIT 1
        ) latest_balance ON TRUE
        LEFT JOIN (
            SELECT
                account_id,
                SUM(amount_cents)::BIGINT AS balance_cents
            FROM transactions
            WHERE user_id = $1
              AND pending = FALSE
            GROUP BY account_id
        ) transaction_balances ON transaction_balances.account_id = accounts.id
        WHERE accounts.user_id = $1
          AND accounts.is_active = TRUE
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let investment_value_cents = sqlx::query_scalar::<_, Option<i64>>(
        r#"
            SELECT COALESCE(SUM(COALESCE(market_value_cents, 0)), 0)::BIGINT
            FROM holdings
            WHERE user_id = $1
            "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?
    .unwrap_or_default();

    let mut cash_cents = 0;
    let mut debt_cents = 0;

    for account in account_balances {
        let balance_cents = account
            .snapshot_balance_cents
            .unwrap_or(account.transaction_balance_cents);

        match account.account_type.as_str() {
            "checking" | "savings" | "cash" | "manual" => {
                cash_cents += balance_cents;
            }
            "credit_card" => {
                if balance_cents < 0 {
                    debt_cents += balance_cents.abs();
                } else {
                    debt_cents += balance_cents;
                }
            }
            _ => {}
        }
    }

    let net_worth_cents = cash_cents + investment_value_cents - debt_cents;

    Ok(CurrentNetWorth {
        cash_cents,
        investment_value_cents,
        debt_cents,
        net_worth_cents,
    })
}

pub async fn create_today_portfolio_snapshot(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<PortfolioSnapshot, sqlx::Error> {
    let net_worth = compute_current_net_worth(pool, user_id).await?;
    let today = Utc::now().date_naive();

    sqlx::query_as::<_, PortfolioSnapshot>(
        r#"
        INSERT INTO portfolio_snapshots (
            user_id,
            cash_cents,
            investment_value_cents,
            debt_cents,
            net_worth_cents,
            snapshot_date,
            source
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'computed')
        ON CONFLICT (user_id, snapshot_date)
        DO UPDATE SET
            cash_cents = EXCLUDED.cash_cents,
            investment_value_cents = EXCLUDED.investment_value_cents,
            debt_cents = EXCLUDED.debt_cents,
            net_worth_cents = EXCLUDED.net_worth_cents,
            source = EXCLUDED.source
        RETURNING
            id,
            user_id,
            cash_cents,
            investment_value_cents,
            debt_cents,
            net_worth_cents,
            currency,
            snapshot_date,
            source
        "#,
    )
    .bind(user_id)
    .bind(net_worth.cash_cents)
    .bind(net_worth.investment_value_cents)
    .bind(net_worth.debt_cents)
    .bind(net_worth.net_worth_cents)
    .bind(today)
    .fetch_one(pool)
    .await
}

pub async fn get_net_worth_timeline(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<PortfolioSnapshot>, sqlx::Error> {
    let mut snapshots = sqlx::query_as::<_, PortfolioSnapshot>(
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
            source
        FROM portfolio_snapshots
        WHERE user_id = $1
        ORDER BY snapshot_date ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    if snapshots.is_empty() {
        snapshots.push(create_today_portfolio_snapshot(pool, user_id).await?);
    }

    Ok(snapshots)
}
