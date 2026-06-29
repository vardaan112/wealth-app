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

### 1. Start Postgres

From the repo root:

```bash
docker compose up -d
```

Postgres runs on `localhost:5432` with user/password/database `wealth`.

### 2. Set `DATABASE_URL`

```bash
cd services/api
cp .env.example .env
```

The default in `.env.example` matches Docker Compose:

```
DATABASE_URL=postgres://wealth:wealth@localhost:5432/wealth
```

### 3. Run migrations

The initial schema lives in a single file, `infra/migrations/0001_initial_schema.sql`. It is applied automatically when the Postgres container is **first** created (empty data volume). To reset and re-run the migration:

```bash
docker compose down -v   # removes postgres_data volume
docker compose up -d
```

To verify tables were created:

```sql
\dt
```

### 4. Start the Rust API

```bash
cd services/api
cargo run
```

API listens on `http://localhost:8000`.

- Health: `GET /health`
- GraphQL playground: `GET /graphql`
- GraphQL: `POST /graphql`

Example query:

```graphql
query {
  apiVersion
  databaseStatus
}
```

### 5. Run the web app

```bash
cd apps/web
npm install
npm run dev
```

Web app runs on `http://localhost:5173` and proxies `/graphql` and `/health` to the API.

## Notes

- Auth, Plaid, SnapTrade, and business logic are not included yet.
- Provider-specific tables are not used; raw webhook/import payloads go in `raw_provider_events`.
