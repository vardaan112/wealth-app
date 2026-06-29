use chrono::NaiveDate;
use uuid::Uuid;

use super::types::{
    FinanceProvider, ProviderAccount, ProviderBalanceSnapshot, ProviderHolding,
    ProviderInvestmentTransaction, ProviderTransaction,
};

pub struct MockProvider;

impl MockProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FinanceProvider for MockProvider {
    fn sync_accounts(&self, _user_id: Uuid) -> Vec<ProviderAccount> {
        vec![
            ProviderAccount {
                provider: "chase".to_string(),
                external_account_id: "chase-checking-001".to_string(),
                account_type: "checking".to_string(),
                name: "Chase Total Checking".to_string(),
                official_name: Some("Chase Total Checking".to_string()),
                mask: Some("8821".to_string()),
                currency: "USD".to_string(),
            },
            ProviderAccount {
                provider: "discover".to_string(),
                external_account_id: "discover-card-001".to_string(),
                account_type: "credit_card".to_string(),
                name: "Discover it Cash Back".to_string(),
                official_name: Some("Discover it Cash Back Credit Card".to_string()),
                mask: Some("4409".to_string()),
                currency: "USD".to_string(),
            },
            ProviderAccount {
                provider: "robinhood".to_string(),
                external_account_id: "robinhood-brokerage-001".to_string(),
                account_type: "brokerage".to_string(),
                name: "Robinhood Brokerage".to_string(),
                official_name: Some("Robinhood Individual Brokerage".to_string()),
                mask: Some("1184".to_string()),
                currency: "USD".to_string(),
            },
        ]
    }

    fn sync_transactions(&self, _user_id: Uuid) -> Vec<ProviderTransaction> {
        vec![
            ProviderTransaction {
                provider: "chase".to_string(),
                external_transaction_id: "chase-txn-payroll-2026-06-14".to_string(),
                external_account_id: "chase-checking-001".to_string(),
                amount_cents: 5_250_00,
                currency: "USD".to_string(),
                merchant_name: Some("Acme Corp Payroll".to_string()),
                raw_description: Some("ACH CREDIT ACME CORP PAYROLL".to_string()),
                category_primary: Some("Income".to_string()),
                category_detailed: Some("Payroll".to_string()),
                transaction_date: date(2026, 6, 14),
                authorized_date: Some(date(2026, 6, 14)),
                pending: false,
                transaction_type: "income".to_string(),
            },
            ProviderTransaction {
                provider: "chase".to_string(),
                external_transaction_id: "chase-txn-whole-foods-2026-06-22".to_string(),
                external_account_id: "chase-checking-001".to_string(),
                amount_cents: -96_48,
                currency: "USD".to_string(),
                merchant_name: Some("Whole Foods Market".to_string()),
                raw_description: Some("WHOLEFDS MKT 10247".to_string()),
                category_primary: Some("Groceries".to_string()),
                category_detailed: Some("Supermarkets".to_string()),
                transaction_date: date(2026, 6, 22),
                authorized_date: Some(date(2026, 6, 21)),
                pending: false,
                transaction_type: "expense".to_string(),
            },
            ProviderTransaction {
                provider: "chase".to_string(),
                external_transaction_id: "chase-txn-rent-2026-06-01".to_string(),
                external_account_id: "chase-checking-001".to_string(),
                amount_cents: -2_850_00,
                currency: "USD".to_string(),
                merchant_name: Some("Oak Street Apartments".to_string()),
                raw_description: Some("ONLINE PAYMENT OAK STREET APTS".to_string()),
                category_primary: Some("Housing".to_string()),
                category_detailed: Some("Rent".to_string()),
                transaction_date: date(2026, 6, 1),
                authorized_date: Some(date(2026, 6, 1)),
                pending: false,
                transaction_type: "expense".to_string(),
            },
            ProviderTransaction {
                provider: "discover".to_string(),
                external_transaction_id: "discover-txn-dining-2026-06-18".to_string(),
                external_account_id: "discover-card-001".to_string(),
                amount_cents: -74_25,
                currency: "USD".to_string(),
                merchant_name: Some("Nopa Restaurant".to_string()),
                raw_description: Some("NOPA RESTAURANT SAN FRANCISCO CA".to_string()),
                category_primary: Some("Dining".to_string()),
                category_detailed: Some("Restaurants".to_string()),
                transaction_date: date(2026, 6, 18),
                authorized_date: Some(date(2026, 6, 18)),
                pending: false,
                transaction_type: "expense".to_string(),
            },
            ProviderTransaction {
                provider: "discover".to_string(),
                external_transaction_id: "discover-txn-payment-2026-06-20".to_string(),
                external_account_id: "discover-card-001".to_string(),
                amount_cents: 1_200_00,
                currency: "USD".to_string(),
                merchant_name: Some("Autopay Payment".to_string()),
                raw_description: Some("DIRECTPAY FULL BALANCE PAYMENT".to_string()),
                category_primary: Some("Transfer".to_string()),
                category_detailed: Some("Credit Card Payment".to_string()),
                transaction_date: date(2026, 6, 20),
                authorized_date: Some(date(2026, 6, 20)),
                pending: false,
                transaction_type: "transfer".to_string(),
            },
        ]
    }

    fn sync_holdings(&self, _user_id: Uuid) -> Vec<ProviderHolding> {
        vec![
            ProviderHolding {
                provider: "robinhood".to_string(),
                external_holding_id: Some("rh-holding-vti".to_string()),
                external_account_id: "robinhood-brokerage-001".to_string(),
                symbol: "VTI".to_string(),
                asset_name: Some("Vanguard Total Stock Market ETF".to_string()),
                asset_type: "etf".to_string(),
                quantity: 42.75,
                market_value_cents: Some(12_842_87),
                cost_basis_cents: Some(10_950_00),
                price_cents: Some(30_042),
                currency: "USD".to_string(),
            },
            ProviderHolding {
                provider: "robinhood".to_string(),
                external_holding_id: Some("rh-holding-aapl".to_string()),
                external_account_id: "robinhood-brokerage-001".to_string(),
                symbol: "AAPL".to_string(),
                asset_name: Some("Apple Inc.".to_string()),
                asset_type: "stock".to_string(),
                quantity: 18.0,
                market_value_cents: Some(3_870_00),
                cost_basis_cents: Some(2_940_00),
                price_cents: Some(21_500),
                currency: "USD".to_string(),
            },
            ProviderHolding {
                provider: "robinhood".to_string(),
                external_holding_id: Some("rh-holding-usdc".to_string()),
                external_account_id: "robinhood-brokerage-001".to_string(),
                symbol: "USDC".to_string(),
                asset_name: Some("USD Coin".to_string()),
                asset_type: "crypto".to_string(),
                quantity: 250.0,
                market_value_cents: Some(250_00),
                cost_basis_cents: Some(250_00),
                price_cents: Some(100),
                currency: "USD".to_string(),
            },
        ]
    }

    fn sync_investment_transactions(&self, _user_id: Uuid) -> Vec<ProviderInvestmentTransaction> {
        vec![
            ProviderInvestmentTransaction {
                provider: "robinhood".to_string(),
                external_transaction_id: "rh-investment-buy-vti-2026-06-03".to_string(),
                external_account_id: "robinhood-brokerage-001".to_string(),
                symbol: Some("VTI".to_string()),
                asset_name: Some("Vanguard Total Stock Market ETF".to_string()),
                asset_type: Some("etf".to_string()),
                transaction_type: "buy".to_string(),
                quantity: Some(4.0),
                price_cents: Some(29_840),
                amount_cents: -119_360,
                currency: "USD".to_string(),
                transaction_date: date(2026, 6, 3),
                notes: Some("Mock Robinhood buy".to_string()),
            },
            ProviderInvestmentTransaction {
                provider: "robinhood".to_string(),
                external_transaction_id: "rh-investment-dividend-aapl-2026-06-12".to_string(),
                external_account_id: "robinhood-brokerage-001".to_string(),
                symbol: Some("AAPL".to_string()),
                asset_name: Some("Apple Inc.".to_string()),
                asset_type: Some("stock".to_string()),
                transaction_type: "dividend".to_string(),
                quantity: None,
                price_cents: None,
                amount_cents: 4_68,
                currency: "USD".to_string(),
                transaction_date: date(2026, 6, 12),
                notes: Some("Mock dividend".to_string()),
            },
        ]
    }

    fn sync_balance_snapshots(&self, _user_id: Uuid) -> Vec<ProviderBalanceSnapshot> {
        vec![
            ProviderBalanceSnapshot {
                provider: "chase".to_string(),
                external_account_id: "chase-checking-001".to_string(),
                balance_cents: 14_235_77,
                available_balance_cents: Some(14_235_77),
                currency: "USD".to_string(),
                snapshot_date: date(2026, 6, 28),
            },
            ProviderBalanceSnapshot {
                provider: "discover".to_string(),
                external_account_id: "discover-card-001".to_string(),
                balance_cents: -1_384_22,
                available_balance_cents: Some(8_615_78),
                currency: "USD".to_string(),
                snapshot_date: date(2026, 6, 28),
            },
            ProviderBalanceSnapshot {
                provider: "robinhood".to_string(),
                external_account_id: "robinhood-brokerage-001".to_string(),
                balance_cents: 16_962_87,
                available_balance_cents: Some(620_00),
                currency: "USD".to_string(),
                snapshot_date: date(2026, 6, 28),
            },
        ]
    }
}

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("mock dates must be valid")
}
