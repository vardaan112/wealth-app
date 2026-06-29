mod auth;
mod config;
mod db;
mod graphql;
mod providers;
mod repositories;
mod security;
mod services;

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{Request, ServerError, Variables};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::http::HeaderMap;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    schema: graphql::AppSchema,
    pool: sqlx::PgPool,
    jwt_secret: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn graphql_get_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let Some(query) = params.get("query") else {
        return Html(playground_source(GraphQLPlaygroundConfig::new("/graphql"))).into_response();
    };

    let mut request = Request::new(query.clone());

    if let Some(operation_name) = params.get("operationName") {
        request = request.operation_name(operation_name.clone());
    }

    if let Some(variables) = params.get("variables") {
        match serde_json::from_str::<serde_json::Value>(variables) {
            Ok(value) => {
                request = request.variables(Variables::from_json(value));
            }
            Err(e) => {
                let response = async_graphql::Response::from_errors(vec![ServerError::new(
                    format!("invalid GraphQL variables: {e}"),
                    None,
                )]);
                return GraphQLResponse::from(response).into_response();
            }
        }
    }

    if let Some(user) = auth::user_from_authorization(
        &state.pool,
        &state.jwt_secret,
        headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok()),
    )
    .await
    {
        request = request.data(user);
    }

    GraphQLResponse::from(state.schema.execute(request).await).into_response()
}

async fn graphql_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut request = req.into_inner();

    if let Some(user) = auth::user_from_authorization(
        &state.pool,
        &state.jwt_secret,
        headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok()),
    )
    .await
    {
        request = request.data(user);
    }

    state.schema.execute(request).await.into()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "wealth_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let config = config::Config::from_env();
    let pool = match db::create_pool(&config.database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Failed to connect to database: {e}");
            std::process::exit(1);
        }
    };
    auth::seed_single_user(
        &pool,
        config.app_user_email.as_deref(),
        config.app_user_password.as_deref(),
    )
    .await?;
    let state = AppState {
        schema: graphql::build_schema(pool.clone(), config.jwt_secret.clone()),
        pool,
        jwt_secret: config.jwt_secret,
    };
    tracing::info!("Connected to database (pool size: {})", state.pool.size());

    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:5173"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/graphql", get(graphql_get_handler).post(graphql_handler))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("API listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
