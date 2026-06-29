use std::collections::HashMap;

use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::providers::{
    PlaidAccount, PlaidClient, PlaidTransaction, ProviderAccount, ProviderTransaction,
};
use crate::repositories::{provider_connections, raw_provider_events};
use crate::security::encryption;
use crate::services::provider_sync;

type PlaidSyncError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone, Default)]
pub struct PlaidSyncResult {
    pub connections_synced: i32,
    pub accounts_synced: i32,
    pub transactions_synced: i32,
    pub pending_transactions_synced: i32,
    pub raw_events_stored: i32,
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
        let access_token = match encryption::decrypt_string(&connection.encrypted_access_token) {
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
            connection.external_item_id.as_deref(),
            accounts.raw,
        )
        .await?;
        result.raw_events_stored += 1;

        let mut account_ids = HashMap::<String, Uuid>::new();
        for account in accounts.accounts {
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

        sync_transactions_for_connection(
            pool,
            user_id,
            &plaid,
            &access_token,
            connection.external_item_id.as_deref(),
            &mut account_ids,
            &mut result,
        )
        .await?;

        result.connections_synced += 1;
    }

    Ok(result)
}

async fn sync_transactions_for_connection(
    pool: &PgPool,
    user_id: Uuid,
    plaid: &PlaidClient,
    access_token: &str,
    external_item_id: Option<&str>,
    account_ids: &mut HashMap<String, Uuid>,
    result: &mut PlaidSyncResult,
) -> Result<(), PlaidSyncError> {
    let end_date = Utc::now().date_naive();
    let start_date = end_date - Duration::days(90);
    let count = 500;
    let mut offset = 0;

    loop {
        let page = match plaid
            .get_transactions(access_token, start_date, end_date, count, offset)
            .await
        {
            Ok(page) => page,
            Err(error) => {
                result
                    .errors
                    .push(format!("Plaid transactions fetch failed: {error}"));
                return Ok(());
            }
        };

        raw_provider_events::create_raw_provider_event(
            pool,
            user_id,
            "plaid",
            "transactions/get",
            external_item_id,
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

            provider_sync::upsert_provider_transaction(
                pool,
                user_id,
                account_id,
                provider_transaction_from_plaid(transaction),
            )
            .await?;
            result.transactions_synced += 1;
        }

        offset += transaction_count;
        if transaction_count == 0 || offset >= page.total_transactions {
            break;
        }
    }

    Ok(())
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
    let category_primary = transaction
        .category
        .as_ref()
        .and_then(|category| category.first().cloned());
    let category_detailed = transaction
        .category
        .as_ref()
        .and_then(|category| category.get(1).cloned());

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
        category_primary: category_primary.clone(),
        category_detailed,
        transaction_date: transaction.date,
        authorized_date: transaction.authorized_date,
        pending: transaction.pending,
        transaction_type: normalize_plaid_transaction_type(
            amount_cents,
            category_primary.as_deref(),
        ),
    }
}

fn plaid_amount_to_cents(amount: f64) -> i64 {
    (-amount * 100.0).round() as i64
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

fn normalize_plaid_transaction_type(amount_cents: i64, category_primary: Option<&str>) -> String {
    let category = category_primary.unwrap_or_default().to_lowercase();

    if category.contains("transfer") {
        "transfer"
    } else if category.contains("payment") {
        "payment"
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
    }
}
