use chrono::NaiveDate;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ProviderAccount {
    pub provider: String,
    pub external_account_id: String,
    pub account_type: String,
    pub name: String,
    pub official_name: Option<String>,
    pub mask: Option<String>,
    pub currency: String,
}

#[derive(Debug, Clone)]
pub struct ProviderTransaction {
    pub provider: String,
    pub external_transaction_id: String,
    pub external_account_id: String,
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
}

#[derive(Debug, Clone)]
pub struct ProviderHolding {
    pub provider: String,
    pub external_holding_id: Option<String>,
    pub external_account_id: String,
    pub symbol: String,
    pub asset_name: Option<String>,
    pub asset_type: String,
    pub quantity: f64,
    pub market_value_cents: Option<i64>,
    pub cost_basis_cents: Option<i64>,
    pub price_cents: Option<i64>,
    pub currency: String,
}

#[derive(Debug, Clone)]
pub struct ProviderInvestmentTransaction {
    pub provider: String,
    pub external_transaction_id: String,
    pub external_account_id: String,
    pub symbol: Option<String>,
    pub asset_name: Option<String>,
    pub asset_type: Option<String>,
    pub transaction_type: String,
    pub quantity: Option<f64>,
    pub price_cents: Option<i64>,
    pub amount_cents: i64,
    pub currency: String,
    pub transaction_date: NaiveDate,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProviderBalanceSnapshot {
    pub provider: String,
    pub external_account_id: String,
    pub balance_cents: i64,
    pub available_balance_cents: Option<i64>,
    pub currency: String,
    pub snapshot_date: NaiveDate,
}

pub trait FinanceProvider {
    fn sync_accounts(&self, user_id: Uuid) -> Vec<ProviderAccount>;
    fn sync_transactions(&self, user_id: Uuid) -> Vec<ProviderTransaction>;
    fn sync_holdings(&self, user_id: Uuid) -> Vec<ProviderHolding>;
    fn sync_investment_transactions(&self, user_id: Uuid) -> Vec<ProviderInvestmentTransaction>;

    fn sync_balance_snapshots(&self, _user_id: Uuid) -> Vec<ProviderBalanceSnapshot> {
        Vec::new()
    }
}
