use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::repositories::users::{self, UserRecord};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    sub: String,
    email: String,
    exp: usize,
}

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub jwt_secret: String,
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

pub fn verify_password(password_hash: &str, password: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(password_hash) else {
        return false;
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn create_token(
    user: &UserRecord,
    jwt_secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expires_at = Utc::now() + Duration::days(30);
    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        exp: expires_at.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
}

pub async fn user_from_authorization(
    pool: &PgPool,
    jwt_secret: &str,
    authorization: Option<&str>,
) -> Option<CurrentUser> {
    let token = authorization?.strip_prefix("Bearer ")?;
    let token = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .ok()?;
    let user_id = Uuid::parse_str(&token.claims.sub).ok()?;
    let user = users::find_user_by_id(pool, user_id).await.ok()??;

    Some(CurrentUser {
        id: user.id,
        email: user.email,
        display_name: user.display_name,
    })
}

pub async fn seed_single_user(
    pool: &PgPool,
    email: Option<&str>,
    password: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (Some(email), Some(password)) = (email, password) else {
        tracing::warn!(
            "APP_USER_EMAIL/APP_USER_PASSWORD not set; login will require an existing user"
        );
        return Ok(());
    };

    let password_hash =
        hash_password(password).map_err(|e| std::io::Error::other(format!("{e}")))?;
    users::upsert_single_user(pool, email, &password_hash).await?;
    tracing::info!("Configured single app user: {email}");

    Ok(())
}
