-- 0002_provider_connections.sql

CREATE TABLE provider_connections (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

  provider TEXT NOT NULL,
  provider_item_id TEXT,
  provider_user_id TEXT,

  encrypted_access_token TEXT,
  encrypted_refresh_token TEXT,
  encrypted_user_secret TEXT,

  sync_cursor TEXT,

  status TEXT NOT NULL DEFAULT 'active',
  last_synced_at TIMESTAMPTZ,

  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

  CONSTRAINT provider_connections_provider_check CHECK (
    provider IN (
      'plaid',
      'snaptrade',
      'manual',
      'csv'
    )
  ),

  CONSTRAINT provider_connections_status_check CHECK (
    status IN (
      'active',
      'error',
      'disconnected',
      'pending'
    )
  )
);

CREATE TRIGGER provider_connections_set_updated_at
BEFORE UPDATE ON provider_connections
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_provider_connections_user_id ON provider_connections(user_id);
CREATE INDEX idx_provider_connections_provider ON provider_connections(provider);
CREATE INDEX idx_provider_connections_provider_item_id ON provider_connections(provider_item_id);
CREATE INDEX idx_provider_connections_last_synced_at ON provider_connections(last_synced_at DESC);

CREATE UNIQUE INDEX idx_provider_connections_unique_provider_item
ON provider_connections(user_id, provider, provider_item_id)
WHERE provider_item_id IS NOT NULL;

CREATE TABLE sync_runs (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  provider_connection_id UUID REFERENCES provider_connections(id) ON DELETE SET NULL,

  provider TEXT NOT NULL,
  sync_type TEXT NOT NULL,

  status TEXT NOT NULL DEFAULT 'running',

  started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  finished_at TIMESTAMPTZ,

  accounts_added INTEGER NOT NULL DEFAULT 0,
  accounts_updated INTEGER NOT NULL DEFAULT 0,
  transactions_added INTEGER NOT NULL DEFAULT 0,
  transactions_updated INTEGER NOT NULL DEFAULT 0,
  transactions_removed INTEGER NOT NULL DEFAULT 0,
  holdings_upserted INTEGER NOT NULL DEFAULT 0,
  investment_transactions_added INTEGER NOT NULL DEFAULT 0,

  error_message TEXT,

  CONSTRAINT sync_runs_status_check CHECK (
    status IN (
      'running',
      'success',
      'failed',
      'partial'
    )
  )
);

CREATE INDEX idx_sync_runs_user_id ON sync_runs(user_id);
CREATE INDEX idx_sync_runs_provider_connection_id ON sync_runs(provider_connection_id);
CREATE INDEX idx_sync_runs_provider ON sync_runs(provider);
CREATE INDEX idx_sync_runs_started_at ON sync_runs(started_at DESC);
