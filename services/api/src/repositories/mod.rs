#![allow(dead_code)]

pub mod accounts;
pub mod holdings;
pub mod snapshots;
pub mod transactions;

pub fn dev_user_id() -> uuid::Uuid {
    uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .expect("hardcoded dev user id must be a valid UUID")
}

pub async fn ensure_dev_user(pool: &sqlx::PgPool) -> Result<uuid::Uuid, sqlx::Error> {
    let id = dev_user_id();

    sqlx::query(
        r#"
        INSERT INTO users (id, email, display_name)
        VALUES ($1, 'dev@wealth.local', 'Dev User')
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(id)
}
