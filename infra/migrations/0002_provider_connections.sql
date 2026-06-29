-- Secure storage for future provider connections.
-- Tokens are stored encrypted by the API before insertion.

CREATE TABLE provider_connections (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  provider TEXT NOT NULL,
  external_item_id TEXT,
  encrypted_access_token TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'active',
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER provider_connections_set_updated_at
BEFORE UPDATE ON provider_connections
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_provider_connections_user_id ON provider_connections(user_id);
CREATE INDEX idx_provider_connections_provider ON provider_connections(provider);
CREATE INDEX idx_provider_connections_status ON provider_connections(status);
