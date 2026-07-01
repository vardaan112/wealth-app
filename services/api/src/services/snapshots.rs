use std::collections::HashMap;

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
    account_id: Uuid,
    account_type: String,
    snapshot_balance_cents: Option<i64>,
    transaction_balance_cents: i64,
}

#[derive(Debug, FromRow)]
struct AccountHoldingsValue {
    account_id: Uuid,
    market_value_cents: i64,
}

fn account_balance_cents(account: &AccountBalanceInput) -> i64 {
    account
        .snapshot_balance_cents
        .unwrap_or(account.transaction_balance_cents)
}

fn compute_net_worth_components(
    accounts: &[AccountBalanceInput],
    holdings_by_account: &HashMap<Uuid, i64>,
    total_holdings_market_value: i64,
) -> CurrentNetWorth {
    let mut cash_cents = 0;
    let mut debt_cents = 0;
    let mut investment_value_cents = total_holdings_market_value;

    for account in accounts {
        let balance_cents = account_balance_cents(account);

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
            "brokerage" => {
                let holdings_for_account = holdings_by_account
                    .get(&account.account_id)
                    .copied()
                    .unwrap_or(0);

                if holdings_for_account == 0 && balance_cents != 0 {
                    // No holdings synced yet; treat the account balance as investments.
                    investment_value_cents += balance_cents;
                } else if balance_cents > holdings_for_account {
                    // Balance snapshots include holdings plus uninvested cash.
                    cash_cents += balance_cents - holdings_for_account;
                }
            }
            _ => {}
        }
    }

    let net_worth_cents = cash_cents + investment_value_cents - debt_cents;

    CurrentNetWorth {
        cash_cents,
        investment_value_cents,
        debt_cents,
        net_worth_cents,
    }
}

pub async fn compute_current_net_worth(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<CurrentNetWorth, sqlx::Error> {
    let account_balances = sqlx::query_as::<_, AccountBalanceInput>(
        r#"
        SELECT
            accounts.id AS account_id,
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

    let account_holdings = sqlx::query_as::<_, AccountHoldingsValue>(
        r#"
        SELECT
            account_id,
            COALESCE(SUM(COALESCE(market_value_cents, 0)), 0)::BIGINT AS market_value_cents
        FROM holdings
        WHERE user_id = $1
        GROUP BY account_id
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let holdings_by_account = account_holdings
        .into_iter()
        .map(|row| (row.account_id, row.market_value_cents))
        .collect::<HashMap<_, _>>();

    let total_holdings_market_value = holdings_by_account.values().sum();

    Ok(compute_net_worth_components(
        &account_balances,
        &holdings_by_account,
        total_holdings_market_value,
    ))
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
    create_today_portfolio_snapshot(pool, user_id).await?;

    sqlx::query_as::<_, PortfolioSnapshot>(
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
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(
        account_type: &str,
        snapshot_balance_cents: Option<i64>,
        transaction_balance_cents: i64,
    ) -> AccountBalanceInput {
        AccountBalanceInput {
            account_id: Uuid::new_v4(),
            account_type: account_type.to_string(),
            snapshot_balance_cents,
            transaction_balance_cents,
        }
    }

    #[test]
    fn net_worth_includes_holdings_and_cash_minus_debt() {
        let checking = account("checking", Some(10_000_00), 0);
        let credit = account("credit_card", Some(-2_000_00), 0);
        let holdings_by_account = HashMap::from([(Uuid::new_v4(), 5_000_00)]);

        let net_worth =
            compute_net_worth_components(&[checking, credit], &holdings_by_account, 5_000_00);

        assert_eq!(net_worth.cash_cents, 10_000_00);
        assert_eq!(net_worth.investment_value_cents, 5_000_00);
        assert_eq!(net_worth.debt_cents, 2_000_00);
        assert_eq!(net_worth.net_worth_cents, 13_000_00);
    }

    #[test]
    fn brokerage_balance_without_holdings_counts_as_investments() {
        let brokerage_id = Uuid::new_v4();
        let brokerage = AccountBalanceInput {
            account_id: brokerage_id,
            account_type: "brokerage".to_string(),
            snapshot_balance_cents: Some(16_962_87),
            transaction_balance_cents: 0,
        };

        let net_worth = compute_net_worth_components(&[brokerage], &HashMap::new(), 0);

        assert_eq!(net_worth.investment_value_cents, 16_962_87);
        assert_eq!(net_worth.cash_cents, 0);
        assert_eq!(net_worth.net_worth_cents, 16_962_87);
    }

    #[test]
    fn brokerage_uses_holdings_and_adds_uninvested_cash_only() {
        let brokerage_id = Uuid::new_v4();
        let brokerage = AccountBalanceInput {
            account_id: brokerage_id,
            account_type: "brokerage".to_string(),
            snapshot_balance_cents: Some(17_582_87),
            transaction_balance_cents: 0,
        };
        let holdings_by_account = HashMap::from([(brokerage_id, 16_962_87)]);

        let net_worth = compute_net_worth_components(&[brokerage], &holdings_by_account, 16_962_87);

        assert_eq!(net_worth.investment_value_cents, 16_962_87);
        assert_eq!(net_worth.cash_cents, 620_00);
        assert_eq!(net_worth.net_worth_cents, 17_582_87);
    }

    #[test]
    fn brokerage_with_matching_holdings_avoids_double_counting() {
        let brokerage_id = Uuid::new_v4();
        let brokerage = AccountBalanceInput {
            account_id: brokerage_id,
            account_type: "brokerage".to_string(),
            snapshot_balance_cents: Some(16_962_87),
            transaction_balance_cents: 0,
        };
        let holdings_by_account = HashMap::from([(brokerage_id, 16_962_87)]);

        let net_worth = compute_net_worth_components(&[brokerage], &holdings_by_account, 16_962_87);

        assert_eq!(net_worth.investment_value_cents, 16_962_87);
        assert_eq!(net_worth.cash_cents, 0);
        assert_eq!(net_worth.net_worth_cents, 16_962_87);
    }
}
