use std::collections::HashMap;

use chrono::{Duration, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::providers::{
    PlaidAccount, PlaidBalances, PlaidClient, PlaidTransaction, ProviderAccount,
    ProviderBalanceSnapshot, ProviderTransaction,
};
use crate::repositories::provider_connections::ProviderConnectionRecord;
use crate::repositories::{provider_connections, raw_provider_events};
use crate::security::encryption;
use crate::services::provider_sync;

type PlaidSyncError = Box<dyn std::error::Error + Send + Sync>;

/// Initial backfill window for a brand-new Plaid connection.
const INITIAL_SYNC_DAYS: i64 = 90;
/// Overlap re-fetched on incremental syncs so pending→posted transitions update
/// existing rows instead of leaving stale pending copies.
const SYNC_OVERLAP_DAYS: i64 = 3;

#[derive(Debug, Clone, Default)]
pub struct PlaidSyncResult {
    pub connections_synced: i32,
    pub accounts_synced: i32,
    pub transactions_synced: i32,
    pub transactions_inserted: i32,
    pub transactions_updated: i32,
    pub pending_transactions_synced: i32,
    pub raw_events_stored: i32,
    pub balance_snapshots_synced: i32,
    pub errors: Vec<String>,
}

pub async fn sync_plaid_transactions(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<PlaidSyncResult, PlaidSyncError> {
    let connections =
        provider_connections::list_provider_connections(pool, user_id, "plaid").await?;
    let mut result = PlaidSyncResult::default();
    if connections.is_empty() {
        return Ok(result);
    }

    let plaid = PlaidClient::from_env()?;

    for connection in connections {
        let Some(encrypted_access_token) = connection.encrypted_access_token.as_deref() else {
            result.errors.push(format!(
                "Plaid connection {} is missing an access token",
                connection.id
            ));
            continue;
        };
        let access_token = match encryption::decrypt_string(encrypted_access_token) {
            Ok(access_token) => access_token,
            Err(_) => {
                result.errors.push(format!(
                    "could not decrypt Plaid connection {}",
                    connection.id
                ));
                continue;
            }
        };

        let accounts = match plaid.get_accounts(&access_token).await {
            Ok(accounts) => accounts,
            Err(error) => {
                result.errors.push(format!(
                    "Plaid accounts fetch failed for connection {}: {error}",
                    connection.id
                ));
                continue;
            }
        };

        raw_provider_events::create_raw_provider_event(
            pool,
            user_id,
            "plaid",
            "accounts/get",
            connection.provider_item_id.as_deref(),
            accounts.raw,
        )
        .await?;
        result.raw_events_stored += 1;

        let mut account_ids = HashMap::<String, Uuid>::new();
        let snapshot_date = Utc::now().date_naive();
        for account in accounts.accounts {
            let external_account_id = account.account_id.clone();
            let (_, account_id) = provider_sync::upsert_provider_account(
                pool,
                user_id,
                provider_account_from_plaid(account.clone()),
            )
            .await?;

            account_ids.insert(external_account_id.clone(), account_id);
            result.accounts_synced += 1;

            if let Some(balance_cents) = plaid_balance_to_cents(&account.balances) {
                let currency = account
                    .balances
                    .iso_currency_code
                    .clone()
                    .or(account.balances.unofficial_currency_code.clone())
                    .unwrap_or_else(|| "USD".to_string());

                provider_sync::upsert_provider_balance_snapshot(
                    pool,
                    user_id,
                    account_id,
                    ProviderBalanceSnapshot {
                        provider: "plaid".to_string(),
                        external_account_id,
                        balance_cents,
                        available_balance_cents: account
                            .balances
                            .available
                            .map(plaid_dollars_to_cents),
                        currency,
                        snapshot_date,
                    },
                )
                .await?;
                result.balance_snapshots_synced += 1;
            }
        }

        let end_date = snapshot_date;
        let fetch_failed = sync_transactions_for_connection(
            pool,
            user_id,
            connection.id,
            &connection,
            &plaid,
            &access_token,
            connection.provider_item_id.as_deref(),
            end_date,
            &mut account_ids,
            &mut result,
        )
        .await?;

        if fetch_failed {
            let _ =
                provider_connections::mark_connection_synced(pool, connection.id, "error", false)
                    .await;
            continue;
        }

        provider_connections::update_plaid_sync_cursor(pool, connection.id, end_date).await?;
        result.connections_synced += 1;
    }

    Ok(result)
}

/// Returns `true` when the Plaid transactions fetch failed and the sync cursor
/// must not advance.
async fn sync_transactions_for_connection(
    pool: &PgPool,
    user_id: Uuid,
    connection_id: Uuid,
    connection: &ProviderConnectionRecord,
    plaid: &PlaidClient,
    access_token: &str,
    provider_item_id: Option<&str>,
    end_date: NaiveDate,
    account_ids: &mut HashMap<String, Uuid>,
    result: &mut PlaidSyncResult,
) -> Result<bool, PlaidSyncError> {
    let start_date = plaid_transaction_fetch_start(connection, end_date);
    let count = 500;
    let mut offset = 0;

    loop {
        let page = match plaid
            .get_transactions(access_token, start_date, end_date, count, offset)
            .await
        {
            Ok(page) => page,
            Err(error) => {
                result.errors.push(format!(
                    "Plaid transactions fetch failed for connection {connection_id}: {error}"
                ));
                return Ok(true);
            }
        };

        raw_provider_events::create_raw_provider_event(
            pool,
            user_id,
            "plaid",
            "transactions/get",
            provider_item_id,
            page.raw,
        )
        .await?;
        result.raw_events_stored += 1;

        for account in page.accounts {
            if account_ids.contains_key(&account.account_id) {
                continue;
            }

            let external_account_id = account.account_id.clone();
            let (_, account_id) = provider_sync::upsert_provider_account(
                pool,
                user_id,
                provider_account_from_plaid(account),
            )
            .await?;
            account_ids.insert(external_account_id, account_id);
            result.accounts_synced += 1;
        }

        let transaction_count = page.transactions.len() as i32;
        for transaction in page.transactions {
            let Some(account_id) = account_ids.get(&transaction.account_id).copied() else {
                result.errors.push(format!(
                    "missing account for Plaid transaction {}",
                    transaction.transaction_id
                ));
                continue;
            };

            if transaction.pending {
                result.pending_transactions_synced += 1;
            }

            let inserted = provider_sync::upsert_provider_transaction(
                pool,
                user_id,
                account_id,
                provider_transaction_from_plaid(transaction),
            )
            .await?;
            result.transactions_synced += 1;
            if inserted {
                result.transactions_inserted += 1;
            } else {
                result.transactions_updated += 1;
            }
        }

        offset += transaction_count;
        if transaction_count == 0 || offset >= page.total_transactions {
            break;
        }
    }

    Ok(false)
}

/// Computes the Plaid `/transactions/get` start date for a connection.
/// First sync uses [`INITIAL_SYNC_DAYS`]; later syncs resume from the stored
/// cursor (or `last_synced_at`) minus [`SYNC_OVERLAP_DAYS`].
fn plaid_transaction_fetch_start(
    connection: &ProviderConnectionRecord,
    end_date: NaiveDate,
) -> NaiveDate {
    if let Some(cursor) = connection.sync_cursor.as_deref() {
        if let Ok(cursor_date) = NaiveDate::parse_from_str(cursor, "%Y-%m-%d") {
            return cursor_date - Duration::days(SYNC_OVERLAP_DAYS);
        }
    }

    if let Some(last_synced_at) = connection.last_synced_at {
        return last_synced_at.date_naive() - Duration::days(SYNC_OVERLAP_DAYS);
    }

    end_date - Duration::days(INITIAL_SYNC_DAYS)
}

fn provider_account_from_plaid(account: PlaidAccount) -> ProviderAccount {
    let currency = account
        .balances
        .iso_currency_code
        .or(account.balances.unofficial_currency_code)
        .unwrap_or_else(|| "USD".to_string());

    ProviderAccount {
        provider: "plaid".to_string(),
        external_account_id: account.account_id,
        account_type: normalize_plaid_account_type(
            &account.account_type,
            account.subtype.as_deref(),
        ),
        name: account.name,
        official_name: account.official_name,
        mask: account.mask,
        currency,
    }
}

fn provider_transaction_from_plaid(transaction: PlaidTransaction) -> ProviderTransaction {
    let amount_cents = plaid_amount_to_cents(transaction.amount);
    let (mut category_primary, mut category_detailed) = plaid_transaction_categories(&transaction);
    let mut transaction_type =
        normalize_plaid_transaction_type(amount_cents, category_primary.as_deref());

    if let Some(poker_detailed) =
        looks_like_poker_transaction(&transaction.name, transaction.merchant_name.as_deref())
    {
        category_primary = Some("Poker".to_string());
        category_detailed = Some(poker_detailed.to_string());
        transaction_type = normalize_plaid_transaction_type(amount_cents, Some("Poker"));
    } else if looks_like_transfer_payment(&transaction.name, transaction.merchant_name.as_deref()) {
        category_detailed = Some(transfer_payment_detailed(
            &transaction.name,
            category_detailed.as_deref(),
        ));
        category_primary = Some("Transfer".to_string());
        transaction_type = "transfer".to_string();
    }

    ProviderTransaction {
        provider: "plaid".to_string(),
        external_transaction_id: transaction.transaction_id,
        external_account_id: transaction.account_id,
        amount_cents,
        currency: transaction
            .iso_currency_code
            .or(transaction.unofficial_currency_code)
            .unwrap_or_else(|| "USD".to_string()),
        merchant_name: transaction.merchant_name,
        raw_description: Some(transaction.name),
        category_primary,
        category_detailed,
        transaction_date: transaction.date,
        authorized_date: transaction.authorized_date,
        pending: transaction.pending,
        transaction_type,
    }
}

fn plaid_transaction_categories(
    transaction: &PlaidTransaction,
) -> (Option<String>, Option<String>) {
    if let Some(personal_finance_category) = &transaction.personal_finance_category {
        let primary = personal_finance_category
            .primary
            .as_deref()
            .and_then(format_plaid_category_label);
        if let Some(primary_category) = primary {
            let detailed = personal_finance_category
                .detailed
                .as_deref()
                .and_then(|detailed| {
                    personal_finance_category
                        .primary
                        .as_deref()
                        .and_then(|raw_primary| {
                            let prefix = format!("{raw_primary}_");
                            detailed.strip_prefix(&prefix)
                        })
                        .or(Some(detailed))
                })
                .and_then(format_plaid_category_label);

            return (Some(primary_category), detailed);
        }
    }

    let category_primary = transaction.category.as_ref().and_then(|category| {
        category
            .first()
            .and_then(|value| non_empty_string(value.as_str()))
    });
    let category_detailed = transaction.category.as_ref().and_then(|category| {
        category
            .get(1)
            .and_then(|value| non_empty_string(value.as_str()))
    });

    (category_primary, category_detailed)
}

fn format_plaid_category_label(value: &str) -> Option<String> {
    let words = value
        .trim()
        .split('_')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let lower = word.to_ascii_lowercase();
            if lower == "and" {
                return lower;
            }

            let mut chars = lower.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>();

    if words.is_empty() {
        None
    } else {
        Some(words.join(" "))
    }
}

fn non_empty_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn plaid_amount_to_cents(amount: f64) -> i64 {
    (-amount * 100.0).round() as i64
}

fn plaid_dollars_to_cents(amount: f64) -> i64 {
    (amount * 100.0).round() as i64
}

fn plaid_balance_to_cents(balances: &PlaidBalances) -> Option<i64> {
    balances
        .current
        .or(balances.available)
        .map(plaid_dollars_to_cents)
}

fn normalize_plaid_account_type(account_type: &str, subtype: Option<&str>) -> String {
    match (account_type, subtype.unwrap_or_default()) {
        ("depository", "checking") => "checking",
        ("depository", "savings") => "savings",
        ("depository", _) => "cash",
        ("credit", _) => "credit_card",
        ("investment", _) => "brokerage",
        ("loan", _) => "loan",
        _ => "other",
    }
    .to_string()
}

fn looks_like_poker_transaction(
    raw_description: &str,
    merchant_name: Option<&str>,
) -> Option<&'static str> {
    let text =
        format!("{} {}", raw_description, merchant_name.unwrap_or_default()).to_ascii_lowercase();

    if text.contains("pure social") {
        return Some("Pure Social");
    }

    if text.contains("pure poker") {
        return Some("Pure Poker");
    }

    let collapsed: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    if collapsed.contains("clubwptgold") {
        return Some("ClubWPT Gold");
    }

    None
}

fn looks_like_transfer_payment(raw_description: &str, merchant_name: Option<&str>) -> bool {
    let text =
        format!("{} {}", raw_description, merchant_name.unwrap_or_default()).to_ascii_lowercase();

    if text.contains("payment thank you")
        || text.contains("autopay payment")
        || text.contains("autopay")
        || text.contains("directpay")
        || text.contains("credit card payment")
        || text.contains("online payment")
        || text.contains("mobile payment")
    {
        return true;
    }

    if text.contains("payment to ") && (text.contains(" card ") || text.contains("card ending")) {
        return true;
    }

    text.contains("real time transfer")
        || text.contains("zelle payment to")
        || text.contains("zelle payment from")
        || text.contains("zelle payment ")
}

fn transfer_payment_detailed(raw_description: &str, category_detailed: Option<&str>) -> String {
    let text = raw_description.to_ascii_lowercase();

    if text.contains("payment thank you") || text.contains("credit card payment") {
        return "Credit Card Payment".to_string();
    }

    if text.contains("payment to ") && text.contains("card") {
        return "Credit Card Payment".to_string();
    }

    if text.contains("zelle") {
        return "Zelle".to_string();
    }

    if text.contains("real time transfer") {
        return "Bank Transfer".to_string();
    }

    category_detailed
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Internal Transfer")
        .to_string()
}

fn normalize_plaid_transaction_type(amount_cents: i64, category_primary: Option<&str>) -> String {
    let category = category_primary.unwrap_or_default().to_lowercase();

    if category.contains("transfer")
        || category.contains("payment")
        || category.contains("loan disbursement")
        || category.contains("loan payment")
    {
        "transfer"
    } else if category.contains("fee") {
        "fee"
    } else if category.contains("interest") {
        "interest"
    } else if category.contains("refund") {
        "refund"
    } else if amount_cents > 0 {
        "income"
    } else if amount_cents < 0 {
        "expense"
    } else {
        "other"
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plaid_amounts_are_converted_to_app_signs() {
        assert_eq!(plaid_amount_to_cents(12.34), -1234);
        assert_eq!(plaid_amount_to_cents(-250.0), 25_000);
    }

    #[test]
    fn plaid_balances_use_current_before_available() {
        let balances = PlaidBalances {
            available: Some(3_766.43),
            current: Some(474.7),
            iso_currency_code: Some("USD".to_string()),
            unofficial_currency_code: None,
        };

        assert_eq!(plaid_balance_to_cents(&balances), Some(47_470));
    }

    #[test]
    fn plaid_account_types_map_to_schema_values() {
        assert_eq!(
            normalize_plaid_account_type("depository", Some("checking")),
            "checking"
        );
        assert_eq!(
            normalize_plaid_account_type("credit", Some("credit card")),
            "credit_card"
        );
        assert_eq!(
            normalize_plaid_account_type("investment", Some("brokerage")),
            "brokerage"
        );
    }

    #[test]
    fn plaid_transaction_types_stay_within_schema_values() {
        assert_eq!(
            normalize_plaid_transaction_type(-2000, Some("Food and Drink")),
            "expense"
        );
        assert_eq!(
            normalize_plaid_transaction_type(2000, Some("Payroll")),
            "income"
        );
        assert_eq!(
            normalize_plaid_transaction_type(-1000, Some("Transfer")),
            "transfer"
        );
        assert_eq!(
            normalize_plaid_transaction_type(-1000, Some("Loan Payments")),
            "transfer"
        );
        assert_eq!(
            normalize_plaid_transaction_type(1000, Some("Loan Disbursements")),
            "transfer"
        );
    }

    #[test]
    fn pure_poker_and_pure_social_map_to_poker_category() {
        let poker = provider_transaction_from_plaid(PlaidTransaction {
            account_id: "account-1".to_string(),
            transaction_id: "transaction-poker".to_string(),
            amount: 25.0,
            iso_currency_code: Some("USD".to_string()),
            unofficial_currency_code: None,
            merchant_name: Some("Pure Poker".to_string()),
            name: "PURE POKER PURCHASE".to_string(),
            category: None,
            personal_finance_category: Some(
                crate::providers::plaid::PlaidPersonalFinanceCategory {
                    primary: Some("ENTERTAINMENT".to_string()),
                    detailed: Some("ENTERTAINMENT_OTHER".to_string()),
                },
            ),
            date: chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            authorized_date: None,
            pending: false,
        });

        assert_eq!(poker.category_primary.as_deref(), Some("Poker"));
        assert_eq!(poker.category_detailed.as_deref(), Some("Pure Poker"));
        assert_eq!(poker.transaction_type, "expense");

        let social = provider_transaction_from_plaid(PlaidTransaction {
            account_id: "account-1".to_string(),
            transaction_id: "transaction-social".to_string(),
            amount: 10.0,
            iso_currency_code: Some("USD".to_string()),
            unofficial_currency_code: None,
            merchant_name: Some("Pure Social".to_string()),
            name: "PURE SOCIAL COINS".to_string(),
            category: None,
            personal_finance_category: None,
            date: chrono::NaiveDate::from_ymd_opt(2026, 6, 16).unwrap(),
            authorized_date: None,
            pending: false,
        });

        assert_eq!(social.category_primary.as_deref(), Some("Poker"));
        assert_eq!(social.category_detailed.as_deref(), Some("Pure Social"));
    }

    #[test]
    fn clubwpt_gold_maps_to_poker_category_for_purchases_and_deposits() {
        let purchase = provider_transaction_from_plaid(PlaidTransaction {
            account_id: "account-1".to_string(),
            transaction_id: "transaction-clubwpt-purchase".to_string(),
            amount: 50.0,
            iso_currency_code: Some("USD".to_string()),
            unofficial_currency_code: None,
            merchant_name: Some("ClubWPT Gold".to_string()),
            name: "CLUBWPTGOLD PURCHASE".to_string(),
            category: None,
            personal_finance_category: None,
            date: chrono::NaiveDate::from_ymd_opt(2026, 6, 17).unwrap(),
            authorized_date: None,
            pending: false,
        });

        assert_eq!(purchase.category_primary.as_deref(), Some("Poker"));
        assert_eq!(purchase.category_detailed.as_deref(), Some("ClubWPT Gold"));
        assert_eq!(purchase.transaction_type, "expense");

        let deposit = provider_transaction_from_plaid(PlaidTransaction {
            account_id: "account-1".to_string(),
            transaction_id: "transaction-clubwpt-deposit".to_string(),
            amount: -100.0,
            iso_currency_code: Some("USD".to_string()),
            unofficial_currency_code: None,
            merchant_name: None,
            name: "CLUB WPT GOLD WITHDRAWAL".to_string(),
            category: None,
            personal_finance_category: Some(
                crate::providers::plaid::PlaidPersonalFinanceCategory {
                    primary: Some("TRANSFER_IN".to_string()),
                    detailed: Some("TRANSFER_IN_DEPOSIT".to_string()),
                },
            ),
            date: chrono::NaiveDate::from_ymd_opt(2026, 6, 18).unwrap(),
            authorized_date: None,
            pending: false,
        });

        assert_eq!(deposit.category_primary.as_deref(), Some("Poker"));
        assert_eq!(deposit.category_detailed.as_deref(), Some("ClubWPT Gold"));
        assert_eq!(deposit.transaction_type, "income");
        assert!(!looks_like_transfer_payment(
            "CLUB WPT GOLD WITHDRAWAL",
            None
        ));
    }

    #[test]
    fn credit_card_payments_are_detected_from_descriptions() {
        assert!(looks_like_transfer_payment(
            "Payment Thank You-Mobile",
            None
        ));
        assert!(looks_like_transfer_payment(
            "Payment to Chase card ending in 0721 06/08",
            None
        ));
        assert!(looks_like_transfer_payment(
            "ZELLE PAYMENT TO VIR TOOLSIDASS JPM99CN9VBG2",
            None
        ));
        assert!(!looks_like_transfer_payment("Chipotle Mexican Grill", None));

        let payment = provider_transaction_from_plaid(PlaidTransaction {
            account_id: "account-1".to_string(),
            transaction_id: "transaction-1".to_string(),
            amount: -711.81,
            iso_currency_code: Some("USD".to_string()),
            unofficial_currency_code: None,
            merchant_name: None,
            name: "Payment Thank You-Mobile".to_string(),
            category: None,
            personal_finance_category: None,
            date: chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            authorized_date: None,
            pending: false,
        });

        assert_eq!(payment.transaction_type, "transfer");
        assert_eq!(payment.category_primary.as_deref(), Some("Transfer"));
        assert_eq!(
            payment.category_detailed.as_deref(),
            Some("Credit Card Payment")
        );
    }

    #[test]
    fn plaid_personal_finance_categories_are_preferred_and_formatted() {
        let transaction = PlaidTransaction {
            account_id: "account-1".to_string(),
            transaction_id: "transaction-1".to_string(),
            amount: 12.34,
            iso_currency_code: Some("USD".to_string()),
            unofficial_currency_code: None,
            merchant_name: Some("Restaurant".to_string()),
            name: "Restaurant".to_string(),
            category: Some(vec![
                "Legacy Primary".to_string(),
                "Legacy Detailed".to_string(),
            ]),
            personal_finance_category: Some(
                crate::providers::plaid::PlaidPersonalFinanceCategory {
                    primary: Some("FOOD_AND_DRINK".to_string()),
                    detailed: Some("FOOD_AND_DRINK_RESTAURANT".to_string()),
                },
            ),
            date: chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            authorized_date: None,
            pending: false,
        };

        let provider_transaction = provider_transaction_from_plaid(transaction);

        assert_eq!(
            provider_transaction.category_primary.as_deref(),
            Some("Food and Drink")
        );
        assert_eq!(
            provider_transaction.category_detailed.as_deref(),
            Some("Restaurant")
        );
        assert_eq!(provider_transaction.transaction_type, "expense");
    }

    #[test]
    fn first_sync_uses_initial_window() {
        let end_date = NaiveDate::from_ymd_opt(2026, 6, 29).unwrap();
        let connection = ProviderConnectionRecord {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            provider: "plaid".to_string(),
            provider_item_id: None,
            provider_user_id: None,
            encrypted_access_token: None,
            encrypted_refresh_token: None,
            encrypted_user_secret: None,
            sync_cursor: None,
            status: "active".to_string(),
            last_synced_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let start = plaid_transaction_fetch_start(&connection, end_date);
        assert_eq!(start, end_date - Duration::days(INITIAL_SYNC_DAYS));
    }

    #[test]
    fn incremental_sync_resumes_from_cursor_with_overlap() {
        let end_date = NaiveDate::from_ymd_opt(2026, 6, 29).unwrap();
        let connection = ProviderConnectionRecord {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            provider: "plaid".to_string(),
            provider_item_id: None,
            provider_user_id: None,
            encrypted_access_token: None,
            encrypted_refresh_token: None,
            encrypted_user_secret: None,
            sync_cursor: Some("2026-06-22".to_string()),
            status: "active".to_string(),
            last_synced_at: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let start = plaid_transaction_fetch_start(&connection, end_date);
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 6, 19).unwrap());
    }

    #[test]
    fn incremental_sync_falls_back_to_last_synced_at() {
        let end_date = NaiveDate::from_ymd_opt(2026, 6, 29).unwrap();
        let last_synced_at = NaiveDate::from_ymd_opt(2026, 6, 20)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc();
        let connection = ProviderConnectionRecord {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            provider: "plaid".to_string(),
            provider_item_id: None,
            provider_user_id: None,
            encrypted_access_token: None,
            encrypted_refresh_token: None,
            encrypted_user_secret: None,
            sync_cursor: None,
            status: "active".to_string(),
            last_synced_at: Some(last_synced_at),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let start = plaid_transaction_fetch_start(&connection, end_date);
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 6, 17).unwrap());
    }

    #[test]
    fn resync_with_empty_incoming_category_keeps_mapped_transfer() {
        let first = provider_transaction_from_plaid(PlaidTransaction {
            account_id: "account-1".to_string(),
            transaction_id: "transaction-1".to_string(),
            amount: -711.81,
            iso_currency_code: Some("USD".to_string()),
            unofficial_currency_code: None,
            merchant_name: None,
            name: "Payment Thank You-Mobile".to_string(),
            category: None,
            personal_finance_category: None,
            date: chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            authorized_date: None,
            pending: false,
        });
        assert_eq!(first.category_primary.as_deref(), Some("Transfer"));

        let resync = provider_transaction_from_plaid(PlaidTransaction {
            account_id: "account-1".to_string(),
            transaction_id: "transaction-1".to_string(),
            amount: -711.81,
            iso_currency_code: Some("USD".to_string()),
            unofficial_currency_code: None,
            merchant_name: None,
            name: "Payment Thank You-Mobile".to_string(),
            category: None,
            personal_finance_category: None,
            date: chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            authorized_date: None,
            pending: false,
        });

        assert_eq!(resync.transaction_type, "transfer");
        assert_eq!(resync.category_primary.as_deref(), Some("Transfer"));
        assert_eq!(
            resync.category_detailed.as_deref(),
            Some("Credit Card Payment")
        );
    }

    #[test]
    fn resync_with_new_plaid_category_updates_mapping() {
        let resync = provider_transaction_from_plaid(PlaidTransaction {
            account_id: "account-1".to_string(),
            transaction_id: "transaction-2".to_string(),
            amount: -42.0,
            iso_currency_code: Some("USD".to_string()),
            unofficial_currency_code: None,
            merchant_name: Some("Grocery".to_string()),
            name: "Grocery".to_string(),
            category: None,
            personal_finance_category: Some(
                crate::providers::plaid::PlaidPersonalFinanceCategory {
                    primary: Some("FOOD_AND_DRINK".to_string()),
                    detailed: Some("FOOD_AND_DRINK_GROCERIES".to_string()),
                },
            ),
            date: chrono::NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
            authorized_date: None,
            pending: false,
        });

        assert_eq!(resync.category_primary.as_deref(), Some("Food and Drink"));
        assert_eq!(resync.category_detailed.as_deref(), Some("Groceries"));
    }

    #[test]
    fn resync_pending_to_posted_keeps_same_provider_transaction_id() {
        let plaid_pending = PlaidTransaction {
            account_id: "account-1".to_string(),
            transaction_id: "transaction-3".to_string(),
            amount: -15.0,
            iso_currency_code: Some("USD".to_string()),
            unofficial_currency_code: None,
            merchant_name: Some("Coffee".to_string()),
            name: "Coffee Shop".to_string(),
            category: None,
            personal_finance_category: Some(
                crate::providers::plaid::PlaidPersonalFinanceCategory {
                    primary: Some("FOOD_AND_DRINK".to_string()),
                    detailed: Some("FOOD_AND_DRINK_COFFEE".to_string()),
                },
            ),
            date: chrono::NaiveDate::from_ymd_opt(2026, 6, 25).unwrap(),
            authorized_date: None,
            pending: true,
        };
        let pending = provider_transaction_from_plaid(plaid_pending.clone());
        let posted = provider_transaction_from_plaid(PlaidTransaction {
            pending: false,
            ..plaid_pending
        });

        assert_eq!(
            pending.external_transaction_id,
            posted.external_transaction_id
        );
        assert!(pending.pending);
        assert!(!posted.pending);
    }
}
