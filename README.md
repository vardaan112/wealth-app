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
SNAPTRADE_CLIENT_ID=your-snaptrade-client-id
SNAPTRADE_CONSUMER_KEY=your-snaptrade-consumer-key
```

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

### SnapTrade and Robinhood

1. Add these to `services/api/.env`:

   ```bash
   SNAPTRADE_CLIENT_ID=your-snaptrade-client-id
   SNAPTRADE_CONSUMER_KEY=your-snaptrade-consumer-key
   APP_ENCRYPTION_KEY=base64-encoded-32-byte-key
   ```

2. Restart the Rust API from `services/api`:

   ```bash
   cargo run
   ```

3. In the web app, go to `Settings > Connect Robinhood`, complete the SnapTrade portal, return to the app, then use `Sync Robinhood`.

SnapTrade connection creation is wired and stores the provider user secret encrypted. The current `Sync Robinhood` path is still scaffolded and returns zero synced records until real SnapTrade account, holding, and transaction fetching is implemented.

## Notes

- Auth uses email/password with argon2 password hashes and a JWT stored in browser session storage.
- Plaid access tokens and SnapTrade user secrets are encrypted before storage with `APP_ENCRYPTION_KEY`.
- Raw provider responses/events can be stored in `raw_provider_events` for debugging and auditability.
