use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

#[derive(Debug, Clone, Deserialize)]
pub struct PlaidAccount {
    pub account_id: String,
    pub balances: PlaidBalances,
    pub mask: Option<String>,
    pub name: String,
    pub official_name: Option<String>,
    #[serde(rename = "type")]
    pub account_type: String,
    pub subtype: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PlaidBalances {
    pub available: Option<f64>,
    pub current: Option<f64>,
    pub iso_currency_code: Option<String>,
    pub unofficial_currency_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaidTransaction {
    pub account_id: String,
    pub transaction_id: String,
    pub amount: f64,
    pub iso_currency_code: Option<String>,
    pub unofficial_currency_code: Option<String>,
    pub merchant_name: Option<String>,
    pub name: String,
    pub category: Option<Vec<String>>,
    pub date: NaiveDate,
    pub authorized_date: Option<NaiveDate>,
    pub pending: bool,
}

#[derive(Debug, Clone)]
pub struct PlaidAccountsGet {
    pub accounts: Vec<PlaidAccount>,
    pub raw: Value,
}

#[derive(Debug, Clone)]
pub struct PlaidTransactionsGet {
    pub accounts: Vec<PlaidAccount>,
    pub transactions: Vec<PlaidTransaction>,
    pub total_transactions: i32,
    pub raw: Value,
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

    pub async fn get_accounts(&self, access_token: &str) -> PlaidResult<PlaidAccountsGet> {
        let request = AccountsGetRequest {
            client_id: &self.client_id,
            secret: &self.secret,
            access_token,
        };

        let raw = self.post_json("/accounts/get", &request).await?;
        let response = serde_json::from_value::<AccountsGetResponse>(raw.clone())?;

        Ok(PlaidAccountsGet {
            accounts: response.accounts,
            raw,
        })
    }

    pub async fn get_transactions(
        &self,
        access_token: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        count: i32,
        offset: i32,
    ) -> PlaidResult<PlaidTransactionsGet> {
        let request = TransactionsGetRequest {
            client_id: &self.client_id,
            secret: &self.secret,
            access_token,
            start_date,
            end_date,
            options: TransactionsGetOptions { count, offset },
        };

        let raw = self.post_json("/transactions/get", &request).await?;
        let response = serde_json::from_value::<TransactionsGetResponse>(raw.clone())?;

        Ok(PlaidTransactionsGet {
            accounts: response.accounts,
            transactions: response.transactions,
            total_transactions: response.total_transactions,
            raw,
        })
    }

    async fn post_json<T: Serialize>(&self, path: &str, request: &T) -> PlaidResult<Value> {
        let response = self
            .http
            .post(format!("{}{}", self.base_url, path))
            .json(request)
            .send()
            .await?;

        ensure_success(response)
            .await?
            .json::<Value>()
            .await
            .map_err(|e| e.into())
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

#[derive(Serialize)]
struct AccountsGetRequest<'a> {
    client_id: &'a str,
    secret: &'a str,
    access_token: &'a str,
}

#[derive(Deserialize)]
struct AccountsGetResponse {
    accounts: Vec<PlaidAccount>,
}

#[derive(Serialize)]
struct TransactionsGetRequest<'a> {
    client_id: &'a str,
    secret: &'a str,
    access_token: &'a str,
    start_date: NaiveDate,
    end_date: NaiveDate,
    options: TransactionsGetOptions,
}

#[derive(Serialize)]
struct TransactionsGetOptions {
    count: i32,
    offset: i32,
}

#[derive(Deserialize)]
struct TransactionsGetResponse {
    accounts: Vec<PlaidAccount>,
    transactions: Vec<PlaidTransaction>,
    total_transactions: i32,
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
