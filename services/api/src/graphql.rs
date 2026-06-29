use async_graphql::{Context, EmptySubscription, InputObject, Object, Schema, SimpleObject, ID};
use chrono::NaiveDate;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::repositories;
use crate::repositories::accounts;
use crate::repositories::holdings;
use crate::repositories::transactions;

const API_VERSION: &str = "0.1.0";

#[derive(SimpleObject, Clone)]
struct Money {
    #[graphql(name = "amountCents")]
    amount_cents: i64,
    currency: String,
}

#[derive(SimpleObject, Clone)]
struct User {
    id: String,
    email: String,
    #[graphql(name = "displayName")]
    display_name: String,
}

#[derive(SimpleObject, Clone)]
struct Account {
    id: String,
    name: String,
    #[graphql(name = "accountType")]
    account_type: String,
    provider: String,
    currency: String,
    balance: Money,
    #[graphql(name = "isActive")]
    is_active: bool,
}

#[derive(SimpleObject, Clone)]
struct Transaction {
    id: String,
    #[graphql(name = "accountId")]
    account_id: String,
    #[graphql(name = "merchantName")]
    merchant_name: String,
    amount: Money,
    #[graphql(name = "categoryPrimary")]
    category_primary: String,
    #[graphql(name = "categoryDetailed")]
    category_detailed: Option<String>,
    #[graphql(name = "transactionDate")]
    transaction_date: String,
    #[graphql(name = "transactionType")]
    transaction_type: String,
    pending: bool,
}

#[derive(SimpleObject, Clone)]
struct Holding {
    id: String,
    #[graphql(name = "accountId")]
    account_id: String,
    symbol: String,
    #[graphql(name = "assetName")]
    asset_name: String,
    #[graphql(name = "assetType")]
    asset_type: String,
    quantity: f64,
    #[graphql(name = "marketValue")]
    market_value: Money,
}

#[derive(SimpleObject, Clone)]
#[allow(dead_code)]
struct InvestmentTransaction {
    id: String,
    #[graphql(name = "accountId")]
    account_id: String,
    symbol: String,
    #[graphql(name = "transactionType")]
    transaction_type: String,
    amount: Money,
    quantity: f64,
    #[graphql(name = "transactionDate")]
    transaction_date: String,
}

#[derive(SimpleObject, Clone)]
struct CategorySpend {
    category: String,
    amount: Money,
    percent: f64,
}

#[derive(SimpleObject, Clone)]
struct MonthlySummary {
    month: String,
    income: Money,
    expenses: Money,
    net: Money,
    #[graphql(name = "savingsRate")]
    savings_rate: f64,
    #[graphql(name = "categorySpend")]
    category_spend: Vec<CategorySpend>,
    #[graphql(name = "transactionCount")]
    transaction_count: i32,
    #[graphql(name = "largestTransaction")]
    largest_transaction: Option<Transaction>,
}

#[derive(SimpleObject, Clone)]
struct NetWorthPoint {
    date: String,
    #[graphql(name = "netWorth")]
    net_worth: Money,
    cash: Money,
    investments: Money,
    debt: Money,
}

#[derive(InputObject)]
struct ManualAccountInput {
    name: String,
    #[graphql(name = "accountType")]
    account_type: String,
    provider: Option<String>,
    currency: Option<String>,
}

#[derive(InputObject)]
struct ManualTransactionInput {
    #[graphql(name = "accountId")]
    account_id: ID,
    #[graphql(name = "amountCents")]
    amount_cents: i64,
    currency: Option<String>,
    #[graphql(name = "merchantName")]
    merchant_name: Option<String>,
    #[graphql(name = "rawDescription")]
    raw_description: Option<String>,
    #[graphql(name = "categoryPrimary")]
    category_primary: Option<String>,
    #[graphql(name = "categoryDetailed")]
    category_detailed: Option<String>,
    #[graphql(name = "transactionDate")]
    transaction_date: String,
    pending: Option<bool>,
    #[graphql(name = "transactionType")]
    transaction_type: Option<String>,
    notes: Option<String>,
}

#[derive(InputObject)]
struct ManualHoldingInput {
    #[graphql(name = "accountId")]
    account_id: ID,
    symbol: String,
    #[graphql(name = "assetName")]
    asset_name: Option<String>,
    #[graphql(name = "assetType")]
    asset_type: Option<String>,
    quantity: f64,
    #[graphql(name = "marketValueCents")]
    market_value_cents: Option<i64>,
    #[graphql(name = "costBasisCents")]
    cost_basis_cents: Option<i64>,
    #[graphql(name = "priceCents")]
    price_cents: Option<i64>,
    currency: Option<String>,
}

fn mock_user() -> User {
    User {
        id: "user-001".into(),
        email: "alex.morgan@example.com".into(),
        display_name: "Alex Morgan".into(),
    }
}

fn mock_accounts() -> Vec<Account> {
    vec![
        Account {
            id: "acct-checking-001".into(),
            name: "Primary Checking".into(),
            account_type: "checking".into(),
            provider: "Chase".into(),
            currency: "USD".into(),
            balance: Money {
                amount_cents: 12_450_00,
                currency: "USD".into(),
            },
            is_active: true,
        },
        Account {
            id: "acct-savings-001".into(),
            name: "Emergency Fund".into(),
            account_type: "savings".into(),
            provider: "Ally".into(),
            currency: "USD".into(),
            balance: Money {
                amount_cents: 28_500_00,
                currency: "USD".into(),
            },
            is_active: true,
        },
        Account {
            id: "acct-brokerage-001".into(),
            name: "Investment Portfolio".into(),
            account_type: "investment".into(),
            provider: "Fidelity".into(),
            currency: "USD".into(),
            balance: Money {
                amount_cents: 156_320_00,
                currency: "USD".into(),
            },
            is_active: true,
        },
    ]
}

fn mock_transactions() -> Vec<Transaction> {
    vec![
        Transaction {
            id: "txn-001".into(),
            account_id: "acct-checking-001".into(),
            merchant_name: "Whole Foods Market".into(),
            amount: Money {
                amount_cents: -87_42,
                currency: "USD".into(),
            },
            category_primary: "Groceries".into(),
            category_detailed: Some("Supermarkets".into()),
            transaction_date: "2026-06-25".into(),
            transaction_type: "debit".into(),
            pending: false,
        },
        Transaction {
            id: "txn-002".into(),
            account_id: "acct-checking-001".into(),
            merchant_name: "Acme Corp Payroll".into(),
            amount: Money {
                amount_cents: 5_250_00,
                currency: "USD".into(),
            },
            category_primary: "Income".into(),
            category_detailed: Some("Payroll".into()),
            transaction_date: "2026-06-15".into(),
            transaction_type: "credit".into(),
            pending: false,
        },
        Transaction {
            id: "txn-003".into(),
            account_id: "acct-checking-001".into(),
            merchant_name: "Netflix".into(),
            amount: Money {
                amount_cents: -15_99,
                currency: "USD".into(),
            },
            category_primary: "Entertainment".into(),
            category_detailed: Some("Streaming".into()),
            transaction_date: "2026-06-12".into(),
            transaction_type: "debit".into(),
            pending: false,
        },
        Transaction {
            id: "txn-004".into(),
            account_id: "acct-checking-001".into(),
            merchant_name: "Shell Gas Station".into(),
            amount: Money {
                amount_cents: -52_30,
                currency: "USD".into(),
            },
            category_primary: "Transportation".into(),
            category_detailed: Some("Fuel".into()),
            transaction_date: "2026-06-10".into(),
            transaction_type: "debit".into(),
            pending: false,
        },
        Transaction {
            id: "txn-005".into(),
            account_id: "acct-savings-001".into(),
            merchant_name: "Monthly Transfer".into(),
            amount: Money {
                amount_cents: 500_00,
                currency: "USD".into(),
            },
            category_primary: "Transfer".into(),
            category_detailed: Some("Savings".into()),
            transaction_date: "2026-06-01".into(),
            transaction_type: "credit".into(),
            pending: false,
        },
        Transaction {
            id: "txn-006".into(),
            account_id: "acct-checking-001".into(),
            merchant_name: "Amazon".into(),
            amount: Money {
                amount_cents: -134_27,
                currency: "USD".into(),
            },
            category_primary: "Shopping".into(),
            category_detailed: Some("Online".into()),
            transaction_date: "2026-06-28".into(),
            transaction_type: "debit".into(),
            pending: true,
        },
    ]
}

fn mock_holdings() -> Vec<Holding> {
    vec![
        Holding {
            id: "hold-001".into(),
            account_id: "acct-brokerage-001".into(),
            symbol: "VTI".into(),
            asset_name: "Vanguard Total Stock Market ETF".into(),
            asset_type: "etf".into(),
            quantity: 120.5,
            market_value: Money {
                amount_cents: 28_450_00,
                currency: "USD".into(),
            },
        },
        Holding {
            id: "hold-002".into(),
            account_id: "acct-brokerage-001".into(),
            symbol: "AAPL".into(),
            asset_name: "Apple Inc.".into(),
            asset_type: "stock".into(),
            quantity: 45.0,
            market_value: Money {
                amount_cents: 9_875_00,
                currency: "USD".into(),
            },
        },
        Holding {
            id: "hold-003".into(),
            account_id: "acct-brokerage-001".into(),
            symbol: "BND".into(),
            asset_name: "Vanguard Total Bond Market ETF".into(),
            asset_type: "etf".into(),
            quantity: 200.0,
            market_value: Money {
                amount_cents: 14_820_00,
                currency: "USD".into(),
            },
        },
    ]
}

fn mock_net_worth_timeline() -> Vec<NetWorthPoint> {
    vec![
        NetWorthPoint {
            date: "2026-01-01".into(),
            net_worth: Money {
                amount_cents: 175_000_00,
                currency: "USD".into(),
            },
            cash: Money {
                amount_cents: 32_000_00,
                currency: "USD".into(),
            },
            investments: Money {
                amount_cents: 148_000_00,
                currency: "USD".into(),
            },
            debt: Money {
                amount_cents: 5_000_00,
                currency: "USD".into(),
            },
        },
        NetWorthPoint {
            date: "2026-02-01".into(),
            net_worth: Money {
                amount_cents: 178_500_00,
                currency: "USD".into(),
            },
            cash: Money {
                amount_cents: 33_200_00,
                currency: "USD".into(),
            },
            investments: Money {
                amount_cents: 150_800_00,
                currency: "USD".into(),
            },
            debt: Money {
                amount_cents: 5_500_00,
                currency: "USD".into(),
            },
        },
        NetWorthPoint {
            date: "2026-03-01".into(),
            net_worth: Money {
                amount_cents: 182_100_00,
                currency: "USD".into(),
            },
            cash: Money {
                amount_cents: 34_500_00,
                currency: "USD".into(),
            },
            investments: Money {
                amount_cents: 153_600_00,
                currency: "USD".into(),
            },
            debt: Money {
                amount_cents: 6_000_00,
                currency: "USD".into(),
            },
        },
        NetWorthPoint {
            date: "2026-04-01".into(),
            net_worth: Money {
                amount_cents: 185_750_00,
                currency: "USD".into(),
            },
            cash: Money {
                amount_cents: 35_800_00,
                currency: "USD".into(),
            },
            investments: Money {
                amount_cents: 156_450_00,
                currency: "USD".into(),
            },
            debt: Money {
                amount_cents: 6_500_00,
                currency: "USD".into(),
            },
        },
        NetWorthPoint {
            date: "2026-05-01".into(),
            net_worth: Money {
                amount_cents: 189_400_00,
                currency: "USD".into(),
            },
            cash: Money {
                amount_cents: 37_100_00,
                currency: "USD".into(),
            },
            investments: Money {
                amount_cents: 159_300_00,
                currency: "USD".into(),
            },
            debt: Money {
                amount_cents: 7_000_00,
                currency: "USD".into(),
            },
        },
        NetWorthPoint {
            date: "2026-06-01".into(),
            net_worth: Money {
                amount_cents: 193_270_00,
                currency: "USD".into(),
            },
            cash: Money {
                amount_cents: 40_950_00,
                currency: "USD".into(),
            },
            investments: Money {
                amount_cents: 162_320_00,
                currency: "USD".into(),
            },
            debt: Money {
                amount_cents: 10_000_00,
                currency: "USD".into(),
            },
        },
    ]
}

fn parse_uuid(id: &ID, field: &str) -> Result<Uuid, async_graphql::Error> {
    Uuid::parse_str(id.as_str())
        .map_err(|e| async_graphql::Error::new(format!("invalid {field}: {e}")))
}

fn parse_date(value: &str, field: &str) -> Result<NaiveDate, async_graphql::Error> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|e| {
        async_graphql::Error::new(format!("invalid {field}; expected YYYY-MM-DD: {e}"))
    })
}

fn account_from_record(record: accounts::AccountRecord) -> Account {
    Account {
        id: record.id.to_string(),
        name: record.name,
        account_type: record.account_type,
        provider: record.provider,
        currency: record.currency.clone(),
        balance: Money {
            amount_cents: 0,
            currency: record.currency,
        },
        is_active: record.is_active,
    }
}

fn transaction_from_record(record: transactions::TransactionRecord) -> Transaction {
    Transaction {
        id: record.id.to_string(),
        account_id: record.account_id.to_string(),
        merchant_name: record
            .merchant_name
            .or(record.raw_description)
            .unwrap_or_else(|| "Manual transaction".to_string()),
        amount: Money {
            amount_cents: record.amount_cents,
            currency: record.currency,
        },
        category_primary: record
            .category_primary
            .unwrap_or_else(|| "Uncategorized".to_string()),
        category_detailed: record.category_detailed,
        transaction_date: record.transaction_date.to_string(),
        transaction_type: record.transaction_type,
        pending: record.pending,
    }
}

fn holding_from_record(record: holdings::HoldingRecord) -> Holding {
    Holding {
        id: record.id.to_string(),
        account_id: record.account_id.to_string(),
        symbol: record.symbol,
        asset_name: record
            .asset_name
            .unwrap_or_else(|| "Manual asset".to_string()),
        asset_type: record.asset_type,
        quantity: record.quantity,
        market_value: Money {
            amount_cents: record.market_value_cents.unwrap_or_default(),
            currency: record.currency,
        },
    }
}

fn transaction_is_transfer(transaction: &transactions::TransactionRecord) -> bool {
    transaction
        .transaction_type
        .eq_ignore_ascii_case("transfer")
}

fn transaction_is_income(transaction: &transactions::TransactionRecord) -> bool {
    transaction.amount_cents > 0 || transaction.transaction_type.eq_ignore_ascii_case("income")
}

fn transaction_is_spending(transaction: &transactions::TransactionRecord) -> bool {
    transaction.amount_cents < 0 || transaction.transaction_type.eq_ignore_ascii_case("expense")
}

fn calculate_monthly_summary(
    month: &str,
    transactions: &[transactions::TransactionRecord],
) -> MonthlySummary {
    let included_transactions: Vec<&transactions::TransactionRecord> = transactions
        .iter()
        .filter(|transaction| !transaction_is_transfer(transaction))
        .collect();

    let income_cents = included_transactions
        .iter()
        .filter(|transaction| transaction_is_income(transaction))
        .map(|transaction| transaction.amount_cents.abs())
        .sum::<i64>();

    let spending_cents = included_transactions
        .iter()
        .filter(|transaction| {
            transaction_is_spending(transaction) && !transaction_is_income(transaction)
        })
        .map(|transaction| transaction.amount_cents.abs())
        .sum::<i64>();

    let mut category_totals = HashMap::<String, i64>::new();
    for transaction in included_transactions.iter().filter(|transaction| {
        transaction_is_spending(transaction) && !transaction_is_income(transaction)
    }) {
        let category = transaction
            .category_primary
            .clone()
            .unwrap_or_else(|| "Uncategorized".to_string());
        *category_totals.entry(category).or_default() += transaction.amount_cents.abs();
    }

    let mut category_spend = category_totals
        .into_iter()
        .map(|(category, amount_cents)| CategorySpend {
            category,
            amount: Money {
                amount_cents,
                currency: "USD".to_string(),
            },
            percent: if spending_cents > 0 {
                amount_cents as f64 / spending_cents as f64 * 100.0
            } else {
                0.0
            },
        })
        .collect::<Vec<_>>();
    category_spend.sort_by(|a, b| {
        b.amount
            .amount_cents
            .cmp(&a.amount.amount_cents)
            .then_with(|| a.category.cmp(&b.category))
    });

    let largest_transaction = included_transactions
        .iter()
        .max_by_key(|transaction| transaction.amount_cents.abs())
        .map(|transaction| transaction_from_record((*transaction).clone()));

    let net_cents = income_cents - spending_cents;

    MonthlySummary {
        month: month.to_string(),
        income: Money {
            amount_cents: income_cents,
            currency: "USD".to_string(),
        },
        expenses: Money {
            amount_cents: spending_cents,
            currency: "USD".to_string(),
        },
        net: Money {
            amount_cents: net_cents,
            currency: "USD".to_string(),
        },
        savings_rate: if income_cents > 0 {
            net_cents as f64 / income_cents as f64 * 100.0
        } else {
            0.0
        },
        category_spend,
        transaction_count: included_transactions.len() as i32,
        largest_transaction,
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn api_version(&self) -> String {
        API_VERSION.to_string()
    }

    async fn database_status(&self, ctx: &Context<'_>) -> Result<String, async_graphql::Error> {
        let pool = ctx.data::<PgPool>()?;
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(pool)
            .await
            .map_err(|e| async_graphql::Error::new(format!("database ping failed: {e}")))?;
        Ok("connected".to_string())
    }

    async fn me(&self) -> User {
        mock_user()
    }

    async fn accounts(&self, ctx: &Context<'_>) -> Result<Vec<Account>, async_graphql::Error> {
        let pool = ctx.data::<PgPool>()?;
        let user_id = repositories::ensure_dev_user(pool).await?;
        let records = accounts::list_accounts(pool, user_id).await?;

        if records.is_empty() {
            return Ok(mock_accounts());
        }

        Ok(records.into_iter().map(account_from_record).collect())
    }

    async fn transactions(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<Transaction>, async_graphql::Error> {
        let pool = ctx.data::<PgPool>()?;
        let user_id = repositories::ensure_dev_user(pool).await?;
        let records = transactions::list_transactions(pool, user_id, None).await?;

        if records.is_empty() {
            return Ok(mock_transactions());
        }

        Ok(records.into_iter().map(transaction_from_record).collect())
    }

    async fn holdings(&self, ctx: &Context<'_>) -> Result<Vec<Holding>, async_graphql::Error> {
        let pool = ctx.data::<PgPool>()?;
        let user_id = repositories::ensure_dev_user(pool).await?;
        let records = holdings::list_holdings(pool, user_id).await?;

        if records.is_empty() {
            return Ok(mock_holdings());
        }

        Ok(records.into_iter().map(holding_from_record).collect())
    }

    async fn monthly_summary(
        &self,
        ctx: &Context<'_>,
        month: String,
    ) -> Result<MonthlySummary, async_graphql::Error> {
        let pool = ctx.data::<PgPool>()?;
        let user_id = repositories::ensure_dev_user(pool).await?;
        let records = transactions::list_transactions(pool, user_id, Some(month.clone())).await?;

        Ok(calculate_monthly_summary(&month, &records))
    }

    async fn net_worth_timeline(&self) -> Vec<NetWorthPoint> {
        mock_net_worth_timeline()
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_manual_account(
        &self,
        ctx: &Context<'_>,
        input: ManualAccountInput,
    ) -> Result<Account, async_graphql::Error> {
        let pool = ctx.data::<PgPool>()?;
        let user_id = repositories::ensure_dev_user(pool).await?;

        let account = accounts::create_account(
            pool,
            user_id,
            accounts::CreateAccountInput {
                provider: input.provider,
                provider_account_id: None,
                account_type: input.account_type,
                name: input.name,
                official_name: None,
                mask: None,
                currency: input.currency,
            },
        )
        .await?;

        Ok(account_from_record(account))
    }

    async fn create_manual_transaction(
        &self,
        ctx: &Context<'_>,
        input: ManualTransactionInput,
    ) -> Result<Transaction, async_graphql::Error> {
        let pool = ctx.data::<PgPool>()?;
        let user_id = repositories::ensure_dev_user(pool).await?;
        let account_id = parse_uuid(&input.account_id, "accountId")?;
        let transaction_date = parse_date(&input.transaction_date, "transactionDate")?;

        let transaction = transactions::create_transaction(
            pool,
            user_id,
            transactions::CreateTransactionInput {
                account_id,
                provider: Some("manual".to_string()),
                provider_transaction_id: None,
                amount_cents: input.amount_cents,
                currency: input.currency,
                merchant_name: input.merchant_name,
                raw_description: input.raw_description,
                category_primary: input.category_primary,
                category_detailed: input.category_detailed,
                transaction_date,
                authorized_date: None,
                pending: input.pending,
                transaction_type: input.transaction_type,
                notes: input.notes,
            },
        )
        .await?;

        Ok(transaction_from_record(transaction))
    }

    async fn update_transaction_category(
        &self,
        ctx: &Context<'_>,
        id: ID,
        #[graphql(name = "categoryPrimary")] category_primary: String,
        #[graphql(name = "categoryDetailed")] category_detailed: Option<String>,
    ) -> Result<Transaction, async_graphql::Error> {
        let pool = ctx.data::<PgPool>()?;
        let user_id = repositories::dev_user_id();
        let transaction_id = parse_uuid(&id, "id")?;

        let Some(transaction) = transactions::update_transaction_category(
            pool,
            user_id,
            transaction_id,
            category_primary,
            category_detailed,
        )
        .await?
        else {
            return Err(async_graphql::Error::new("transaction not found"));
        };

        Ok(transaction_from_record(transaction))
    }

    async fn create_manual_holding(
        &self,
        ctx: &Context<'_>,
        input: ManualHoldingInput,
    ) -> Result<Holding, async_graphql::Error> {
        let pool = ctx.data::<PgPool>()?;
        let user_id = repositories::ensure_dev_user(pool).await?;
        let account_id = parse_uuid(&input.account_id, "accountId")?;

        let Some(holding) = holdings::upsert_holding(
            pool,
            user_id,
            holdings::UpsertHoldingInput {
                account_id,
                provider: Some("manual".to_string()),
                provider_holding_id: None,
                symbol: input.symbol,
                asset_name: input.asset_name,
                asset_type: input.asset_type,
                quantity: input.quantity,
                market_value_cents: input.market_value_cents,
                cost_basis_cents: input.cost_basis_cents,
                price_cents: input.price_cents,
                currency: input.currency,
            },
        )
        .await?
        else {
            return Err(async_graphql::Error::new("account not found"));
        };

        Ok(holding_from_record(holding))
    }
}

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn build_schema(pool: PgPool) -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(pool)
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn transaction(
        amount_cents: i64,
        transaction_type: &str,
        category_primary: Option<&str>,
    ) -> transactions::TransactionRecord {
        transactions::TransactionRecord {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            account_id: Uuid::new_v4(),
            provider: "manual".to_string(),
            provider_transaction_id: None,
            amount_cents,
            currency: "USD".to_string(),
            merchant_name: Some("Test merchant".to_string()),
            raw_description: None,
            category_primary: category_primary.map(str::to_string),
            category_detailed: None,
            transaction_date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            authorized_date: None,
            pending: false,
            transaction_type: transaction_type.to_string(),
            notes: None,
            created_at: Utc.with_ymd_and_hms(2026, 6, 15, 12, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 6, 15, 12, 0, 0).unwrap(),
        }
    }

    #[test]
    fn monthly_summary_calculates_income_spending_and_excludes_transfers() {
        let transactions = vec![
            transaction(500_00, "income", Some("Income")),
            transaction(250_00, "other", Some("Refund")),
            transaction(-125_00, "expense", Some("Dining")),
            transaction(-75_00, "expense", Some("Groceries")),
            transaction(-1_000_00, "transfer", Some("Transfer")),
        ];

        let summary = calculate_monthly_summary("2026-06", &transactions);

        assert_eq!(summary.income.amount_cents, 750_00);
        assert_eq!(summary.expenses.amount_cents, 200_00);
        assert_eq!(summary.net.amount_cents, 550_00);
        assert_eq!(summary.transaction_count, 4);
        assert!((summary.savings_rate - 73.33333333333333).abs() < 0.0001);
    }

    #[test]
    fn monthly_summary_returns_zero_values_for_no_transactions() {
        let summary = calculate_monthly_summary("2026-06", &[]);

        assert_eq!(summary.income.amount_cents, 0);
        assert_eq!(summary.expenses.amount_cents, 0);
        assert_eq!(summary.net.amount_cents, 0);
        assert_eq!(summary.savings_rate, 0.0);
        assert_eq!(summary.transaction_count, 0);
        assert!(summary.category_spend.is_empty());
        assert!(summary.largest_transaction.is_none());
    }

    #[test]
    fn monthly_summary_sorts_spending_categories_and_tracks_largest_transaction() {
        let transactions = vec![
            transaction(-50_00, "expense", Some("Dining")),
            transaction(-125_00, "expense", Some("Groceries")),
            transaction(-75_00, "expense", Some("Dining")),
            transaction(300_00, "income", Some("Income")),
        ];

        let summary = calculate_monthly_summary("2026-06", &transactions);

        assert_eq!(summary.category_spend[0].category, "Dining");
        assert_eq!(summary.category_spend[0].amount.amount_cents, 125_00);
        assert_eq!(summary.category_spend[1].category, "Groceries");
        assert_eq!(summary.category_spend[1].amount.amount_cents, 125_00);
        assert_eq!(
            summary
                .largest_transaction
                .as_ref()
                .unwrap()
                .amount
                .amount_cents,
            300_00
        );
    }
}
