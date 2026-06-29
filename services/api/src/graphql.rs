use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema};
use sqlx::PgPool;

const API_VERSION: &str = "0.1.0";

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn api_version(&self) -> String {
        API_VERSION.to_string()
    }

    async fn database_status(&self, ctx: &Context<'_>) -> Result<String, async_graphql::Error> {
        let pool = ctx.data::<PgPool>()?;
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(pool)
            .await
            .map_err(|e| async_graphql::Error::new(format!("database ping failed: {e}")))?;
        Ok("connected".to_string())
    }
}

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub fn build_schema(pool: PgPool) -> AppSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(pool)
        .finish()
}
