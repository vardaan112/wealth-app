use std::collections::HashMap;

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::providers::{
    ProviderAccount, ProviderBalanceSnapshot, ProviderHolding, SnapTradeAccount, SnapTradeClient,
    SnapTradePosition,
};
use crate::repositories::{provider_connections, raw_provider_events};
use crate::security::encryption;
use crate::services::provider_sync::{self, SyncResult};

type SnapTradeSyncError = Box<dyn std::error::Error + Send + Sync>;

pub async fn sync_snaptrade_accounts(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<SyncResult, SnapTradeSyncError> {
    let Some(connection) =
        provider_connections::find_provider_connection(pool, user_id, "snaptrade").await?
    else {
        return Err("No SnapTrade connection found. Connect a brokerage account first.".into());
    };

    let Some(snaptrade_user_id) = connection.provider_user_id.clone() else {
        return Err("SnapTrade connection is missing a user id".into());
    };
    let Some(encrypted_user_secret) = connection.encrypted_user_secret.clone() else {
        return Err("SnapTrade connection is missing a user secret".into());
    };
    let user_secret = encryption::decrypt_string(&encrypted_user_secret)
        .map_err(|_| "could not decrypt SnapTrade user secret")?;

    let client = SnapTradeClient::from_env()?;

    let outcome = run_sync(pool, user_id, &client, &snaptrade_user_id, &user_secret).await;

    match &outcome {
        Ok(_) => {
            provider_connections::mark_connection_synced(pool, connection.id, "active", true)
                .await?;
        }
        Err(_) => {
            // Persist the error state without storing the underlying message,
            // which could surface provider internals. The caller receives the
            // detailed (secret-free) error to relay to the user.
            let _ =
                provider_connections::mark_connection_synced(pool, connection.id, "error", false)
                    .await;
        }
    }

    outcome
}

async fn run_sync(
    pool: &PgPool,
    user_id: Uuid,
    client: &SnapTradeClient,
    snaptrade_user_id: &str,
    user_secret: &str,
) -> Result<SyncResult, SnapTradeSyncError> {
    let mut result = SyncResult::default();
    let today = Utc::now().date_naive();

    let accounts_response = client.list_accounts(snaptrade_user_id, user_secret).await?;

    raw_provider_events::create_raw_provider_event(
        pool,
        user_id,
        "snaptrade",
        "accounts/list",
        None,
        accounts_response.raw,
    )
    .await?;

    let mut account_ids = HashMap::<String, Uuid>::new();
    for account in &accounts_response.accounts {
        let external_account_id = account.id.clone();
        let (_, account_id) = provider_sync::upsert_provider_account(
            pool,
            user_id,
            provider_account_from_snaptrade(account),
        )
        .await?;
        account_ids.insert(external_account_id.clone(), account_id);
        result.accounts_synced += 1;

        if let Some(balance) = account.total_balance() {
            provider_sync::upsert_provider_balance_snapshot(
                pool,
                user_id,
                account_id,
                ProviderBalanceSnapshot {
                    provider: "snaptrade".to_string(),
                    external_account_id: external_account_id.clone(),
                    balance_cents: balance.balance_cents,
                    available_balance_cents: None,
                    currency: balance.currency,
                    snapshot_date: today,
                },
            )
            .await?;
            result.balance_snapshots_synced += 1;
        }
    }

    for account in &accounts_response.accounts {
        let external_account_id = account.id.clone();
        let Some(account_id) = account_ids.get(&external_account_id).copied() else {
            continue;
        };

        let positions = match client
            .list_account_positions(snaptrade_user_id, user_secret, &external_account_id)
            .await
        {
            Ok(positions) => positions,
            Err(error) => {
                result.errors.push(format!(
                    "SnapTrade positions fetch failed for account {external_account_id}: {error}"
                ));
                continue;
            }
        };

        raw_provider_events::create_raw_provider_event(
            pool,
            user_id,
            "snaptrade",
            "accounts/positions",
            Some(&external_account_id),
            positions.raw,
        )
        .await?;

        for position in &positions.positions {
            let Some(holding) = provider_holding_from_snaptrade(&external_account_id, position)
            else {
                continue;
            };
            provider_sync::upsert_provider_holding(pool, user_id, account_id, holding).await?;
            result.holdings_synced += 1;
        }
    }

    // Investment transactions (activities) are intentionally left for a future
    // change: SnapTrade's activities endpoint requires date-window paging and a
    // richer type mapping than accounts/holdings, and holdings already drive the
    // Portfolio page. They remain reported as 0 here.

    Ok(result)
}

fn provider_account_from_snaptrade(account: &SnapTradeAccount) -> ProviderAccount {
    let currency = account
        .total_balance()
        .map(|balance| balance.currency)
        .unwrap_or_else(|| "USD".to_string());
    let name = account
        .name
        .clone()
        .filter(|name| !name.trim().is_empty())
        .or_else(|| account.institution_name.clone())
        .unwrap_or_else(|| "Brokerage Account".to_string());

    ProviderAccount {
        provider: "snaptrade".to_string(),
        external_account_id: account.id.clone(),
        account_type: "brokerage".to_string(),
        name,
        official_name: account.institution_name.clone(),
        mask: account.number.as_deref().map(mask_account_number),
        currency,
    }
}

fn provider_holding_from_snaptrade(
    external_account_id: &str,
    position: &SnapTradePosition,
) -> Option<ProviderHolding> {
    let universal = position
        .symbol
        .as_ref()
        .and_then(|symbol| symbol.symbol.as_ref());
    let symbol = universal.and_then(|universal| {
        universal
            .symbol
            .clone()
            .or_else(|| universal.raw_symbol.clone())
    })?;

    let quantity = position.units.unwrap_or(0.0);
    let price_cents = position.price.map(dollars_to_cents);
    let market_value_cents = position
        .price
        .map(|price| dollars_to_cents(price * quantity));
    let cost_basis_cents = position
        .average_purchase_price
        .map(|average| dollars_to_cents(average * quantity));
    let currency = position
        .currency
        .as_ref()
        .and_then(|currency| currency.code.clone())
        .or_else(|| {
            universal
                .and_then(|universal| universal.currency.as_ref())
                .and_then(|currency| currency.code.clone())
        })
        .unwrap_or_else(|| "USD".to_string());
    let asset_type = universal
        .and_then(|universal| universal.security_type.as_ref())
        .and_then(|security_type| security_type.code.as_deref())
        .map(map_asset_type)
        .unwrap_or_else(|| "stock".to_string());
    let asset_name = universal.and_then(|universal| universal.description.clone());

    Some(ProviderHolding {
        provider: "snaptrade".to_string(),
        external_holding_id: None,
        external_account_id: external_account_id.to_string(),
        symbol,
        asset_name,
        asset_type,
        quantity,
        market_value_cents,
        cost_basis_cents,
        price_cents,
        currency,
    })
}

fn dollars_to_cents(amount: f64) -> i64 {
    (amount * 100.0).round() as i64
}

fn mask_account_number(number: &str) -> String {
    let digits: String = number
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    let len = digits.chars().count();
    if len <= 4 {
        digits
    } else {
        digits.chars().skip(len - 4).collect()
    }
}

fn map_asset_type(code: &str) -> String {
    match code.to_lowercase().as_str() {
        "cs" | "ps" | "ad" | "rt" | "wt" | "wi" | "struct" | "ut" => "stock",
        "et" => "etf",
        "oef" | "cef" => "mutual_fund",
        "crypto" => "crypto",
        "bnd" => "bond",
        _ => "other",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::snaptrade::{
        SnapTradeAccountBalance, SnapTradeAmount, SnapTradeCurrency, SnapTradePositionSymbol,
        SnapTradeSecurityType, SnapTradeUniversalSymbol,
    };

    fn position(
        symbol: &str,
        units: f64,
        price: f64,
        average: f64,
        security_code: &str,
    ) -> SnapTradePosition {
        SnapTradePosition {
            symbol: Some(SnapTradePositionSymbol {
                symbol: Some(SnapTradeUniversalSymbol {
                    symbol: Some(symbol.to_string()),
                    raw_symbol: Some(symbol.to_string()),
                    description: Some("Example Security".to_string()),
                    security_type: Some(SnapTradeSecurityType {
                        code: Some(security_code.to_string()),
                    }),
                    currency: Some(SnapTradeCurrency {
                        code: Some("USD".to_string()),
                    }),
                }),
            }),
            units: Some(units),
            price: Some(price),
            average_purchase_price: Some(average),
            currency: Some(SnapTradeCurrency {
                code: Some("USD".to_string()),
            }),
        }
    }

    #[test]
    fn dollars_convert_to_cents_with_rounding() {
        assert_eq!(dollars_to_cents(113.15), 11_315);
        assert_eq!(dollars_to_cents(108.3353), 10_834);
    }

    #[test]
    fn maps_security_codes_to_schema_asset_types() {
        assert_eq!(map_asset_type("cs"), "stock");
        assert_eq!(map_asset_type("et"), "etf");
        assert_eq!(map_asset_type("crypto"), "crypto");
        assert_eq!(map_asset_type("bnd"), "bond");
        assert_eq!(map_asset_type("oef"), "mutual_fund");
        assert_eq!(map_asset_type("unknown"), "other");
    }

    #[test]
    fn masks_account_number_to_last_four() {
        assert_eq!(mask_account_number("Q6542138443"), "8443");
        assert_eq!(mask_account_number("12"), "12");
    }

    #[test]
    fn holding_market_and_cost_basis_use_quantity() {
        let holding = provider_holding_from_snaptrade(
            "acct-1",
            &position("AAPL", 40.0, 113.15, 108.3353, "cs"),
        )
        .expect("holding");

        assert_eq!(holding.symbol, "AAPL");
        assert_eq!(holding.asset_type, "stock");
        assert_eq!(holding.price_cents, Some(11_315));
        assert_eq!(holding.market_value_cents, Some(452_600));
        assert_eq!(holding.cost_basis_cents, Some(433_341));
        assert_eq!(holding.currency, "USD");
    }

    #[test]
    fn skips_position_without_symbol() {
        let mut position = position("AAPL", 1.0, 1.0, 1.0, "cs");
        position.symbol = None;

        assert!(provider_holding_from_snaptrade("acct-1", &position).is_none());
    }

    #[test]
    fn account_balance_converts_to_cents() {
        let account = SnapTradeAccount {
            id: "acct-1".to_string(),
            name: Some("Robinhood Individual".to_string()),
            number: Some("Q6542138443".to_string()),
            institution_name: Some("Robinhood".to_string()),
            balance: Some(SnapTradeAccountBalance {
                total: Some(SnapTradeAmount {
                    amount: Some(15_363.23),
                    currency: Some("USD".to_string()),
                }),
            }),
        };

        let balance = account.total_balance().expect("balance");
        assert_eq!(balance.balance_cents, 1_536_323);
        assert_eq!(balance.currency, "USD");

        let provider_account = provider_account_from_snaptrade(&account);
        assert_eq!(provider_account.account_type, "brokerage");
        assert_eq!(provider_account.mask.as_deref(), Some("8443"));
        assert_eq!(provider_account.currency, "USD");
    }
}
