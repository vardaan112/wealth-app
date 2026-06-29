CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Automatically update updated_at columns
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- Users
-- ============================================================

CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email TEXT NOT NULL UNIQUE,
  password_hash TEXT,
  display_name TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER users_set_updated_at
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

-- ============================================================
-- Accounts
-- Represents Chase checking, Discover card, Robinhood, manual cash, etc.
-- ============================================================

CREATE TABLE accounts (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

  provider TEXT NOT NULL DEFAULT 'manual',
  provider_account_id TEXT,

  account_type TEXT NOT NULL,
  name TEXT NOT NULL,
  official_name TEXT,
  mask TEXT,

  currency TEXT NOT NULL DEFAULT 'USD',
  is_active BOOLEAN NOT NULL DEFAULT TRUE,

  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

  CONSTRAINT accounts_account_type_check CHECK (
    account_type IN (
      'checking',
      'savings',
      'credit_card',
      'brokerage',
      'crypto',
      'cash',
      'loan',
      'manual',
      'other'
    )
  )
);

CREATE TRIGGER accounts_set_updated_at
BEFORE UPDATE ON accounts
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_accounts_user_id ON accounts(user_id);
CREATE INDEX idx_accounts_provider_account_id ON accounts(provider_account_id);
CREATE UNIQUE INDEX idx_accounts_unique_provider_account
ON accounts(user_id, provider, provider_account_id)
WHERE provider_account_id IS NOT NULL;

-- ============================================================
-- Transactions
-- Main cashflow ledger: Chase, Discover, manual transactions, etc.
-- Convention:
--   income/inflows = positive amount_cents
--   expenses/outflows = negative amount_cents
-- ============================================================

CREATE TABLE transactions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,

  provider TEXT NOT NULL DEFAULT 'manual',
  provider_transaction_id TEXT,

  amount_cents BIGINT NOT NULL,
  currency TEXT NOT NULL DEFAULT 'USD',

  merchant_name TEXT,
  raw_description TEXT,

  category_primary TEXT,
  category_detailed TEXT,

  transaction_date DATE NOT NULL,
  authorized_date DATE,

  pending BOOLEAN NOT NULL DEFAULT FALSE,

  transaction_type TEXT NOT NULL DEFAULT 'expense',
  notes TEXT,

  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

  CONSTRAINT transactions_transaction_type_check CHECK (
    transaction_type IN (
      'income',
      'expense',
      'transfer',
      'payment',
      'refund',
      'fee',
      'interest',
      'adjustment',
      'other'
    )
  )
);

CREATE TRIGGER transactions_set_updated_at
BEFORE UPDATE ON transactions
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_transactions_user_id ON transactions(user_id);
CREATE INDEX idx_transactions_account_id ON transactions(account_id);
CREATE INDEX idx_transactions_transaction_date ON transactions(transaction_date);
CREATE INDEX idx_transactions_provider_transaction_id ON transactions(provider_transaction_id);
CREATE INDEX idx_transactions_user_date ON transactions(user_id, transaction_date DESC);
CREATE INDEX idx_transactions_account_date ON transactions(account_id, transaction_date DESC);

CREATE UNIQUE INDEX idx_transactions_unique_provider_transaction
ON transactions(user_id, provider, provider_transaction_id)
WHERE provider_transaction_id IS NOT NULL;

-- Useful later for search/filtering
CREATE INDEX idx_transactions_category_primary ON transactions(category_primary);

-- ============================================================
-- Holdings
-- Current investment positions: Robinhood stocks/ETFs/crypto/manual assets
-- ============================================================

CREATE TABLE holdings (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,

  provider TEXT NOT NULL DEFAULT 'manual',
  provider_holding_id TEXT,

  symbol TEXT NOT NULL,
  asset_name TEXT,
  asset_type TEXT NOT NULL DEFAULT 'stock',

  quantity NUMERIC(28, 10) NOT NULL,

  market_value_cents BIGINT,
  cost_basis_cents BIGINT,
  price_cents BIGINT,

  currency TEXT NOT NULL DEFAULT 'USD',
  as_of TIMESTAMPTZ NOT NULL DEFAULT NOW(),

  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

  CONSTRAINT holdings_asset_type_check CHECK (
    asset_type IN (
      'stock',
      'etf',
      'mutual_fund',
      'option',
      'crypto',
      'cash',
      'bond',
      'other'
    )
  )
);

CREATE TRIGGER holdings_set_updated_at
BEFORE UPDATE ON holdings
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_holdings_user_id ON holdings(user_id);
CREATE INDEX idx_holdings_account_id ON holdings(account_id);
CREATE INDEX idx_holdings_symbol ON holdings(symbol);
CREATE INDEX idx_holdings_provider_holding_id ON holdings(provider_holding_id);

CREATE UNIQUE INDEX idx_holdings_unique_account_symbol
ON holdings(account_id, symbol, asset_type);

-- ============================================================
-- Investment Transactions
-- Buys, sells, dividends, deposits, withdrawals, fees, etc.
-- ============================================================

CREATE TABLE investment_transactions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,

  provider TEXT NOT NULL DEFAULT 'manual',
  provider_transaction_id TEXT,

  symbol TEXT,
  asset_name TEXT,
  asset_type TEXT,

  transaction_type TEXT NOT NULL,

  quantity NUMERIC(28, 10),
  price_cents BIGINT,
  amount_cents BIGINT NOT NULL,

  currency TEXT NOT NULL DEFAULT 'USD',

  transaction_date DATE NOT NULL,

  notes TEXT,

  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

  CONSTRAINT investment_transactions_type_check CHECK (
    transaction_type IN (
      'buy',
      'sell',
      'dividend',
      'deposit',
      'withdrawal',
      'fee',
      'interest',
      'transfer',
      'split',
      'adjustment',
      'other'
    )
  )
);

CREATE TRIGGER investment_transactions_set_updated_at
BEFORE UPDATE ON investment_transactions
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_investment_transactions_user_id ON investment_transactions(user_id);
CREATE INDEX idx_investment_transactions_account_id ON investment_transactions(account_id);
CREATE INDEX idx_investment_transactions_transaction_date ON investment_transactions(transaction_date);
CREATE INDEX idx_investment_transactions_provider_transaction_id ON investment_transactions(provider_transaction_id);
CREATE INDEX idx_investment_transactions_symbol ON investment_transactions(symbol);
CREATE INDEX idx_investment_transactions_user_date ON investment_transactions(user_id, transaction_date DESC);

CREATE UNIQUE INDEX idx_investment_transactions_unique_provider_transaction
ON investment_transactions(user_id, provider, provider_transaction_id)
WHERE provider_transaction_id IS NOT NULL;

-- ============================================================
-- Account Balance Snapshots
-- Daily or periodic balance snapshots per account
-- ============================================================

CREATE TABLE account_balance_snapshots (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,

  balance_cents BIGINT NOT NULL,
  available_balance_cents BIGINT,

  currency TEXT NOT NULL DEFAULT 'USD',

  snapshot_date DATE NOT NULL,
  source TEXT NOT NULL DEFAULT 'manual',

  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

  UNIQUE(account_id, snapshot_date)
);

CREATE INDEX idx_account_balance_snapshots_user_id ON account_balance_snapshots(user_id);
CREATE INDEX idx_account_balance_snapshots_account_id ON account_balance_snapshots(account_id);
CREATE INDEX idx_account_balance_snapshots_snapshot_date ON account_balance_snapshots(snapshot_date);
CREATE INDEX idx_account_balance_snapshots_user_date ON account_balance_snapshots(user_id, snapshot_date DESC);

-- ============================================================
-- Portfolio Snapshots
-- Whole-user net worth timeline
-- ============================================================

CREATE TABLE portfolio_snapshots (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

  cash_cents BIGINT NOT NULL DEFAULT 0,
  investment_value_cents BIGINT NOT NULL DEFAULT 0,
  debt_cents BIGINT NOT NULL DEFAULT 0,
  net_worth_cents BIGINT NOT NULL DEFAULT 0,

  currency TEXT NOT NULL DEFAULT 'USD',

  snapshot_date DATE NOT NULL,
  source TEXT NOT NULL DEFAULT 'computed',

  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

  UNIQUE(user_id, snapshot_date)
);

CREATE INDEX idx_portfolio_snapshots_user_id ON portfolio_snapshots(user_id);
CREATE INDEX idx_portfolio_snapshots_snapshot_date ON portfolio_snapshots(snapshot_date);
CREATE INDEX idx_portfolio_snapshots_user_date ON portfolio_snapshots(user_id, snapshot_date DESC);

-- ============================================================
-- Raw Provider Events
-- Store raw Plaid/SnapTrade/manual import payloads for debugging and remapping.
-- Do not store plaintext secrets here.
-- ============================================================

CREATE TABLE raw_provider_events (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

  provider TEXT NOT NULL,
  endpoint TEXT NOT NULL,
  external_id TEXT,

  payload_json JSONB NOT NULL,

  received_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_raw_provider_events_user_id ON raw_provider_events(user_id);
CREATE INDEX idx_raw_provider_events_provider ON raw_provider_events(provider);
CREATE INDEX idx_raw_provider_events_external_id ON raw_provider_events(external_id);
CREATE INDEX idx_raw_provider_events_received_at ON raw_provider_events(received_at DESC);
CREATE INDEX idx_raw_provider_events_payload_json ON raw_provider_events USING GIN(payload_json);

-- ============================================================
-- Import Batches
-- Tracks CSV/manual import batches
-- ============================================================

CREATE TABLE import_batches (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  account_id UUID REFERENCES accounts(id) ON DELETE SET NULL,

  source TEXT NOT NULL,
  filename TEXT,

  imported_count INTEGER NOT NULL DEFAULT 0,
  skipped_count INTEGER NOT NULL DEFAULT 0,
  error_count INTEGER NOT NULL DEFAULT 0,

  status TEXT NOT NULL DEFAULT 'completed',

  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

  CONSTRAINT import_batches_status_check CHECK (
    status IN (
      'pending',
      'processing',
      'completed',
      'failed',
      'partial'
    )
  )
);

CREATE INDEX idx_import_batches_user_id ON import_batches(user_id);
CREATE INDEX idx_import_batches_account_id ON import_batches(account_id);
CREATE INDEX idx_import_batches_created_at ON import_batches(created_at DESC);
