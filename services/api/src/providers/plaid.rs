use serde::{Deserialize, Serialize};
use uuid::Uuid;

const PLAID_CLIENT_ID_ENV: &str = "PLAID_CLIENT_ID";
const PLAID_SECRET_ENV: &str = "PLAID_SECRET";
const PLAID_ENV_ENV: &str = "PLAID_ENV";

type PlaidResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone)]
pub struct PlaidClient {
    http: reqwest::Client,
    client_id: String,
    secret: String,
    base_url: String,
}

#[derive(Debug, Clone)]
pub struct PlaidTokenExchange {
    pub access_token: String,
    pub item_id: String,
}

impl PlaidClient {
    pub fn from_env() -> PlaidResult<Self> {
        let client_id = env_required(PLAID_CLIENT_ID_ENV)?;
        let secret = env_required(PLAID_SECRET_ENV)?;
        let plaid_env = std::env::var(PLAID_ENV_ENV).unwrap_or_else(|_| "sandbox".to_string());

        Ok(Self {
            http: reqwest::Client::new(),
            client_id,
            secret,
            base_url: plaid_base_url(&plaid_env)?,
        })
    }

    pub async fn create_link_token(&self, user_id: Uuid) -> PlaidResult<String> {
        let request = LinkTokenCreateRequest {
            client_id: &self.client_id,
            secret: &self.secret,
            client_name: "Wealth App",
            language: "en",
            country_codes: vec!["US"],
            products: vec!["transactions"],
            user: LinkTokenUser {
                client_user_id: user_id.to_string(),
            },
        };

        let response = self
            .http
            .post(format!("{}/link/token/create", self.base_url))
            .json(&request)
            .send()
            .await?;
        ensure_success(response)
            .await?
            .json::<LinkTokenCreateResponse>()
            .await
            .map(|response| response.link_token)
            .map_err(|e| e.into())
    }

    pub async fn exchange_public_token(
        &self,
        public_token: &str,
    ) -> PlaidResult<PlaidTokenExchange> {
        let request = PublicTokenExchangeRequest {
            client_id: &self.client_id,
            secret: &self.secret,
            public_token,
        };

        let response = self
            .http
            .post(format!("{}/item/public_token/exchange", self.base_url))
            .json(&request)
            .send()
            .await?;
        let response = ensure_success(response)
            .await?
            .json::<PublicTokenExchangeResponse>()
            .await?;

        Ok(PlaidTokenExchange {
            access_token: response.access_token,
            item_id: response.item_id,
        })
    }
}

#[derive(Serialize)]
struct LinkTokenCreateRequest<'a> {
    client_id: &'a str,
    secret: &'a str,
    client_name: &'a str,
    language: &'a str,
    country_codes: Vec<&'a str>,
    products: Vec<&'a str>,
    user: LinkTokenUser,
}

#[derive(Serialize)]
struct LinkTokenUser {
    client_user_id: String,
}

#[derive(Deserialize)]
struct LinkTokenCreateResponse {
    link_token: String,
}

#[derive(Serialize)]
struct PublicTokenExchangeRequest<'a> {
    client_id: &'a str,
    secret: &'a str,
    public_token: &'a str,
}

#[derive(Deserialize)]
struct PublicTokenExchangeResponse {
    access_token: String,
    item_id: String,
}

#[derive(Deserialize)]
struct PlaidErrorResponse {
    error_code: Option<String>,
    error_message: Option<String>,
}

async fn ensure_success(response: reqwest::Response) -> PlaidResult<reqwest::Response> {
    if response.status().is_success() {
        return Ok(response);
    }

    let status = response.status();
    let plaid_error = response.json::<PlaidErrorResponse>().await.ok();
    let message = plaid_error
        .and_then(|error| match (error.error_code, error.error_message) {
            (Some(code), Some(message)) => {
                Some(format!("Plaid request failed ({code}): {message}"))
            }
            (_, Some(message)) => Some(format!("Plaid request failed: {message}")),
            _ => None,
        })
        .unwrap_or_else(|| format!("Plaid request failed with status {status}"));

    Err(std::io::Error::other(message).into())
}

fn env_required(name: &str) -> PlaidResult<String> {
    std::env::var(name).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::NotFound, format!("{name} must be set")).into()
    })
}

fn plaid_base_url(plaid_env: &str) -> PlaidResult<String> {
    match plaid_env {
        "sandbox" => Ok("https://sandbox.plaid.com".to_string()),
        "development" => Ok("https://development.plaid.com".to_string()),
        "production" => Ok("https://production.plaid.com".to_string()),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "PLAID_ENV must be sandbox, development, or production",
        )
        .into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_supported_plaid_environments() {
        assert_eq!(
            plaid_base_url("sandbox").unwrap(),
            "https://sandbox.plaid.com"
        );
        assert_eq!(
            plaid_base_url("development").unwrap(),
            "https://development.plaid.com"
        );
        assert_eq!(
            plaid_base_url("production").unwrap(),
            "https://production.plaid.com"
        );
    }

    #[test]
    fn rejects_unknown_plaid_environment() {
        let error = plaid_base_url("local").unwrap_err();

        assert!(error.to_string().contains("PLAID_ENV"));
    }
}
