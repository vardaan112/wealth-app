use async_graphql::{Context, EmptySubscription, InputObject, Object, Schema, SimpleObject, ID};
use chrono::NaiveDate;
use sqlx::PgPool;
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

fn mock_category_spend() -> Vec<CategorySpend> {
    vec![
        CategorySpend {
            category: "Housing".into(),
            amount: Money {
                amount_cents: 1_850_00,
                currency: "USD".into(),
            },
            percent: 38.5,
        },
        CategorySpend {
            category: "Groceries".into(),
            amount: Money {
                amount_cents: 520_00,
                currency: "USD".into(),
            },
            percent: 10.8,
        },
        CategorySpend {
            category: "Transportation".into(),
            amount: Money {
                amount_cents: 380_00,
                currency: "USD".into(),
            },
            percent: 7.9,
        },
        CategorySpend {
            category: "Entertainment".into(),
            amount: Money {
                amount_cents: 245_00,
                currency: "USD".into(),
            },
            percent: 5.1,
        },
        CategorySpend {
            category: "Shopping".into(),
            amount: Money {
                amount_cents: 410_00,
                currency: "USD".into(),
            },
            percent: 8.5,
        },
        CategorySpend {
            category: "Other".into(),
            amount: Money {
                amount_cents: 1_395_00,
                currency: "USD".into(),
            },
            percent: 29.2,
        },
    ]
}

fn mock_monthly_summary(month: &str) -> MonthlySummary {
    MonthlySummary {
        month: month.to_string(),
        income: Money {
            amount_cents: 5_250_00,
            currency: "USD".into(),
        },
        expenses: Money {
            amount_cents: 4_800_00,
            currency: "USD".into(),
        },
        net: Money {
            amount_cents: 450_00,
            currency: "USD".into(),
        },
        savings_rate: 8.57,
        category_spend: mock_category_spend(),
    }
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

    async fn accounts(&self) -> Vec<Account> {
        mock_accounts()
    }

    async fn transactions(&self) -> Vec<Transaction> {
        mock_transactions()
    }

    async fn holdings(&self) -> Vec<Holding> {
        mock_holdings()
    }

    async fn monthly_summary(&self, month: String) -> MonthlySummary {
        mock_monthly_summary(&month)
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
