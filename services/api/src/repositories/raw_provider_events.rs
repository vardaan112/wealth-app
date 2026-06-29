use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_raw_provider_event(
    pool: &PgPool,
    user_id: Uuid,
    provider: &str,
    endpoint: &str,
    external_id: Option<&str>,
    payload_json: Value,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO raw_provider_events (
            user_id,
            provider,
            endpoint,
            external_id,
            payload_json
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(provider)
    .bind(endpoint)
    .bind(external_id)
    .bind(payload_json)
    .fetch_one(pool)
    .await
}
