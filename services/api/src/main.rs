use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

#[derive(SimpleObject)]
struct Health {
    status: String,
    database: String,
}

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn health(&self, ctx: &async_graphql::Context<'_>) -> Health {
        let db = ctx
            .data::<sqlx::PgPool>()
            .expect("database pool not configured");

        let database = match sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(db)
            .await
        {
            Ok(_) => "connected".to_string(),
            Err(_) => "disconnected".to_string(),
        };

        Health {
            status: "ok".to_string(),
            database,
        }
    }
}

type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://wealth:wealth@localhost:5432/wealth".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/graphql", post(graphql_handler))
        .layer(cors)
        .with_state((schema, pool));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("API listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn graphql_handler(
    State((schema, pool)): State<(AppSchema, sqlx::PgPool)>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut request = req.into_inner();
    request = request.data(pool);
    schema.execute(request).await.into()
}
