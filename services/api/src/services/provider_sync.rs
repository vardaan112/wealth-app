use std::collections::HashMap;

use sqlx::PgPool;
use uuid::Uuid;

use crate::providers::{
    FinanceProvider, ProviderAccount, ProviderBalanceSnapshot, ProviderHolding,
    ProviderInvestmentTransaction, ProviderTransaction,
};

#[derive(Debug, Clone, Default)]
pub struct SyncResult {
    pub accounts_synced: i32,
    pub transactions_synced: i32,
    pub holdings_synced: i32,
    pub investment_transactions_synced: i32,
    pub balance_snapshots_synced: i32,
    pub errors: Vec<String>,
}

pub async fn sync_provider<P: FinanceProvider>(
    pool: &PgPool,
    user_id: Uuid,
    provider: &P,
) -> Result<SyncResult, sqlx::Error> {
    let mut result = SyncResult::default();
    let mut account_ids = HashMap::<String, Uuid>::new();

    for account in provider.sync_accounts(user_id) {
        let account_id = upsert_provider_account(pool, user_id, account).await?;
        account_ids.insert(account_id.0, account_id.1);
        result.accounts_synced += 1;
    }

    for transaction in provider.sync_transactions(user_id) {
        let Some(account_id) = account_ids.get(&account_key(
            &transaction.provider,
            &transaction.external_account_id,
        )) else {
            result.errors.push(format!(
                "missing account for transaction {}",
                transaction.external_transaction_id
            ));
            continue;
        };
        upsert_provider_transaction(pool, user_id, *account_id, transaction).await?;
        result.transactions_synced += 1;
    }

    for holding in provider.sync_holdings(user_id) {
        let Some(account_id) = account_ids.get(&account_key(
            &holding.provider,
            &holding.external_account_id,
        )) else {
            result
                .errors
                .push(format!("missing account for holding {}", holding.symbol));
            continue;
        };
        upsert_provider_holding(pool, user_id, *account_id, holding).await?;
        result.holdings_synced += 1;
    }

    for transaction in provider.sync_investment_transactions(user_id) {
        let Some(account_id) = account_ids.get(&account_key(
            &transaction.provider,
            &transaction.external_account_id,
        )) else {
            result.errors.push(format!(
                "missing account for investment transaction {}",
                transaction.external_transaction_id
            ));
            continue;
        };
        upsert_provider_investment_transaction(pool, user_id, *account_id, transaction).await?;
        result.investment_transactions_synced += 1;
    }

    for snapshot in provider.sync_balance_snapshots(user_id) {
        let Some(account_id) = account_ids.get(&account_key(
            &snapshot.provider,
            &snapshot.external_account_id,
        )) else {
            result.errors.push(format!(
                "missing account for balance snapshot {}",
                snapshot.external_account_id
            ));
            continue;
        };
        upsert_provider_balance_snapshot(pool, user_id, *account_id, snapshot).await?;
        result.balance_snapshots_synced += 1;
    }

    Ok(result)
}

pub(crate) async fn upsert_provider_account(
    pool: &PgPool,
    user_id: Uuid,
    account: ProviderAccount,
) -> Result<(String, Uuid), sqlx::Error> {
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO accounts (
            user_id,
            provider,
            provider_account_id,
            account_type,
            name,
            official_name,
            mask,
            currency,
            is_active
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, TRUE)
        ON CONFLICT (user_id, provider, provider_account_id)
        WHERE provider_account_id IS NOT NULL
        DO UPDATE SET
            account_type = EXCLUDED.account_type,
            name = EXCLUDED.name,
            official_name = EXCLUDED.official_name,
            mask = EXCLUDED.mask,
            currency = EXCLUDED.currency,
            is_active = TRUE
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(&account.provider)
    .bind(&account.external_account_id)
    .bind(account.account_type)
    .bind(account.name)
    .bind(account.official_name)
    .bind(account.mask)
    .bind(account.currency)
    .fetch_one(pool)
    .await?;

    Ok((
        account_key(&account.provider, &account.external_account_id),
        id,
    ))
}

pub(crate) async fn upsert_provider_transaction(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    transaction: ProviderTransaction,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO transactions (
            user_id,
            account_id,
            provider,
            provider_transaction_id,
            amount_cents,
            currency,
            merchant_name,
            raw_description,
            category_primary,
            category_detailed,
            transaction_date,
            authorized_date,
            pending,
            transaction_type
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        ON CONFLICT (user_id, provider, provider_transaction_id)
        WHERE provider_transaction_id IS NOT NULL
        DO UPDATE SET
            account_id = EXCLUDED.account_id,
            amount_cents = EXCLUDED.amount_cents,
            currency = EXCLUDED.currency,
            merchant_name = EXCLUDED.merchant_name,
            raw_description = EXCLUDED.raw_description,
            category_primary = EXCLUDED.category_primary,
            category_detailed = EXCLUDED.category_detailed,
            transaction_date = EXCLUDED.transaction_date,
            authorized_date = EXCLUDED.authorized_date,
            pending = EXCLUDED.pending,
            transaction_type = EXCLUDED.transaction_type
        "#,
    )
    .bind(user_id)
    .bind(account_id)
    .bind(transaction.provider)
    .bind(transaction.external_transaction_id)
    .bind(transaction.amount_cents)
    .bind(transaction.currency)
    .bind(transaction.merchant_name)
    .bind(transaction.raw_description)
    .bind(transaction.category_primary)
    .bind(transaction.category_detailed)
    .bind(transaction.transaction_date)
    .bind(transaction.authorized_date)
    .bind(transaction.pending)
    .bind(transaction.transaction_type)
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn upsert_provider_holding(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    holding: ProviderHolding,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO holdings (
            user_id,
            account_id,
            provider,
            provider_holding_id,
            symbol,
            asset_name,
            asset_type,
            quantity,
            market_value_cents,
            cost_basis_cents,
            price_cents,
            currency
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8::NUMERIC, $9, $10, $11, $12)
        ON CONFLICT (account_id, symbol, asset_type)
        DO UPDATE SET
            provider = EXCLUDED.provider,
            provider_holding_id = EXCLUDED.provider_holding_id,
            asset_name = EXCLUDED.asset_name,
            quantity = EXCLUDED.quantity,
            market_value_cents = EXCLUDED.market_value_cents,
            cost_basis_cents = EXCLUDED.cost_basis_cents,
            price_cents = EXCLUDED.price_cents,
            currency = EXCLUDED.currency,
            as_of = NOW()
        "#,
    )
    .bind(user_id)
    .bind(account_id)
    .bind(holding.provider)
    .bind(holding.external_holding_id)
    .bind(holding.symbol)
    .bind(holding.asset_name)
    .bind(holding.asset_type)
    .bind(holding.quantity)
    .bind(holding.market_value_cents)
    .bind(holding.cost_basis_cents)
    .bind(holding.price_cents)
    .bind(holding.currency)
    .execute(pool)
    .await?;

    Ok(())
}

async fn upsert_provider_investment_transaction(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    transaction: ProviderInvestmentTransaction,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO investment_transactions (
            user_id,
            account_id,
            provider,
            provider_transaction_id,
            symbol,
            asset_name,
            asset_type,
            transaction_type,
            quantity,
            price_cents,
            amount_cents,
            currency,
            transaction_date,
            notes
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::NUMERIC, $10, $11, $12, $13, $14)
        ON CONFLICT (user_id, provider, provider_transaction_id)
        WHERE provider_transaction_id IS NOT NULL
        DO UPDATE SET
            account_id = EXCLUDED.account_id,
            symbol = EXCLUDED.symbol,
            asset_name = EXCLUDED.asset_name,
            asset_type = EXCLUDED.asset_type,
            transaction_type = EXCLUDED.transaction_type,
            quantity = EXCLUDED.quantity,
            price_cents = EXCLUDED.price_cents,
            amount_cents = EXCLUDED.amount_cents,
            currency = EXCLUDED.currency,
            transaction_date = EXCLUDED.transaction_date,
            notes = EXCLUDED.notes
        "#,
    )
    .bind(user_id)
    .bind(account_id)
    .bind(transaction.provider)
    .bind(transaction.external_transaction_id)
    .bind(transaction.symbol)
    .bind(transaction.asset_name)
    .bind(transaction.asset_type)
    .bind(transaction.transaction_type)
    .bind(transaction.quantity)
    .bind(transaction.price_cents)
    .bind(transaction.amount_cents)
    .bind(transaction.currency)
    .bind(transaction.transaction_date)
    .bind(transaction.notes)
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn upsert_provider_balance_snapshot(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    snapshot: ProviderBalanceSnapshot,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO account_balance_snapshots (
            user_id,
            account_id,
            balance_cents,
            available_balance_cents,
            currency,
            snapshot_date,
            source
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (account_id, snapshot_date)
        DO UPDATE SET
            balance_cents = EXCLUDED.balance_cents,
            available_balance_cents = EXCLUDED.available_balance_cents,
            currency = EXCLUDED.currency,
            source = EXCLUDED.source
        "#,
    )
    .bind(user_id)
    .bind(account_id)
    .bind(snapshot.balance_cents)
    .bind(snapshot.available_balance_cents)
    .bind(snapshot.currency)
    .bind(snapshot.snapshot_date)
    .bind(snapshot.provider)
    .execute(pool)
    .await?;

    Ok(())
}

fn account_key(provider: &str, external_account_id: &str) -> String {
    format!("{provider}:{external_account_id}")
}
