# wealth-app

Personal finance web app monorepo.

## Structure

```
wealth-app/
  apps/web/           React + TypeScript + Vite + Tailwind
  services/api/       Rust + Axum + async-graphql + sqlx
  infra/migrations/   Postgres SQL migrations
  docker-compose.yml  Local Postgres
```

## Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Docker](https://www.docker.com/) (for Postgres)

## Local development

### 1. Start or reset Postgres

From the repo root:

```bash
docker compose up -d
```

Postgres runs on `localhost:5432` with user `wealth_user`, password `wealth_password`, and database `wealth_app`.

Docker Compose mounts `./infra/migrations:/docker-entrypoint-initdb.d`, so migrations run automatically only when the Postgres volume is first created. After migration changes, reset the local DB from the repo root:

```bash
docker compose down -v
docker compose up -d
```

That deletes the local Postgres volume and recreates it with all SQL files in `infra/migrations`.

### 2. Configure the API environment

```bash
cd services/api
cp .env.example .env
```

The default `DATABASE_URL` in `.env.example` matches Docker Compose. Edit `services/api/.env` and set these backend-only values:

```bash
DATABASE_URL=postgres://wealth_user:wealth_password@localhost:5432/wealth_app
JWT_SECRET=change-this-long-random-secret
APP_USER_EMAIL=you@example.com
APP_USER_PASSWORD=change-this-password
APP_ENCRYPTION_KEY=base64-encoded-32-byte-key
PLAID_CLIENT_ID=your-plaid-client-id
PLAID_SECRET=your-plaid-secret
PLAID_ENV=sandbox
PLAID_REDIRECT_URI=http://localhost:5173/plaid-oauth
SNAPTRADE_CLIENT_ID=your-snaptrade-client-id
SNAPTRADE_CONSUMER_KEY=your-snaptrade-consumer-key
```

`PLAID_REDIRECT_URI` is optional and defaults to `http://localhost:5173/plaid-oauth`. It is required for OAuth banks (see [Plaid OAuth banks](#plaid-oauth-banks-eg-chase) below).

Plaid and SnapTrade credentials are separate. Plaid uses `PLAID_CLIENT_ID` and
`PLAID_SECRET`; it does not use a consumer key. SnapTrade uses
`SNAPTRADE_CLIENT_ID` and `SNAPTRADE_CONSUMER_KEY`.

Generate `APP_ENCRYPTION_KEY` with:

```bash
openssl rand -base64 32
```

`APP_USER_EMAIL` and `APP_USER_PASSWORD` are optional local seed values. The app also supports signing up from the web login page. Do not put Plaid or SnapTrade secrets in frontend env files; the React app calls the Rust API, and the API talks to providers.

### 3. Start the Rust API

```bash
cd services/api
cargo run
```

API listens on `http://localhost:8000`.

- Health: `GET /health`
- GraphQL playground: `GET /graphql`
- GraphQL: `POST /graphql`

Quick API check:

```graphql
query {
  apiVersion
  databaseStatus
}
```

### 4. Start the web app

```bash
cd apps/web
npm install
npm run dev
```

Web app runs on `http://localhost:5173` and proxies `/graphql` and `/health` to the API.

## Provider Setup

### Plaid sandbox

1. Add these to `services/api/.env`:

   ```bash
   PLAID_CLIENT_ID=your-plaid-client-id
   PLAID_SECRET=your-plaid-sandbox-secret
   PLAID_ENV=sandbox
   APP_ENCRYPTION_KEY=base64-encoded-32-byte-key
   ```

2. Restart the Rust API from `services/api`:

   ```bash
   cargo run
   ```

3. In the web app, go to `Settings > Connect Bank`, complete Plaid Link, then use `Sync Bank Transactions`.

For Plaid production, set `PLAID_ENV=production` and use the **Production**
`PLAID_SECRET` from the Plaid Dashboard. The Plaid account must also have
Production access enabled; a sandbox secret or an account without Production
access will produce `INVALID_API_KEYS` even when the client id is correct.

### Plaid OAuth banks (e.g. Chase)

Some institutions—most notably Chase—require an OAuth redirect flow. Plaid relaunches Link after the user authenticates on the bank's site, redirecting back to a redirect URI you control.

1. Register the redirect URI in the [Plaid Dashboard](https://dashboard.plaid.com/) under **Developers > API > Allowed redirect URIs**. By default the app uses:

   ```
   http://localhost:5173/plaid-oauth
   ```

   If you override `PLAID_REDIRECT_URI` in `services/api/.env`, register that exact URI instead. In Production the redirect URI must be HTTPS.

2. (Optional) Set the redirect URI in `services/api/.env` and restart the API. If unset, it defaults to the URL above:

   ```bash
   PLAID_REDIRECT_URI=http://localhost:5173/plaid-oauth
   ```

3. Connect as usual from `Settings > Connect Bank`. For OAuth banks the browser redirects to `/plaid-oauth`, which resumes Link automatically and returns you to Settings once connected. The link token is persisted in `localStorage` so the flow survives the redirect's page reload.

### SnapTrade and Robinhood

1. Add these to `services/api/.env`:

   ```bash
   SNAPTRADE_CLIENT_ID=your-snaptrade-client-id
   SNAPTRADE_CONSUMER_KEY=your-snaptrade-consumer-key
   APP_ENCRYPTION_KEY=base64-encoded-32-byte-key
   ```

   If your SnapTrade client ID is a **personal** key (the `PERS-` prefix), no extra
   values are required. SnapTrade provisions a single account-owner user for you at
   signup and the `registerUser` endpoint is not available for personal keys.
   The backend uses SnapTrade's **Personal API Key Authentication**: it signs each
   request with the client id + consumer key and omits `userId`/`userSecret`, so
   SnapTrade resolves the account owner from the key. The client id + consumer key
   above are the only SnapTrade credentials you need to open the connection portal
   and sync accounts and holdings.

   `SNAPTRADE_CONSUMER_KEY` must be the consumer key from the same SnapTrade API
   key as `SNAPTRADE_CLIENT_ID`. If SnapTrade returns `Unable to verify signature
   sent`, regenerate/copy the matching consumer key in the SnapTrade Dashboard;
   this is unrelated to Plaid's `PLAID_SECRET`.

2. Restart the Rust API from `services/api`:

   ```bash
   cargo run
   ```

3. In the web app, go to `Settings > Connect Robinhood`, complete the SnapTrade portal, return to the app, then use `Sync Robinhood`.

SnapTrade connection creation stores a connection marker for personal keys, and
`Sync Robinhood` fetches accounts and holdings for the authenticated personal key.

## Notes

- Auth uses email/password with argon2 password hashes and a JWT stored in browser session storage.
- Plaid access tokens and SnapTrade user secrets are encrypted before storage with `APP_ENCRYPTION_KEY`.
- Raw provider responses/events can be stored in `raw_provider_events` for debugging and auditability.
