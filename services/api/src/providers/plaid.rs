use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

const PLAID_CLIENT_ID_ENV: &str = "PLAID_CLIENT_ID";
const PLAID_SECRET_ENV: &str = "PLAID_SECRET";
const PLAID_ENV_ENV: &str = "PLAID_ENV";
const PLAID_REDIRECT_URI_ENV: &str = "PLAID_REDIRECT_URI";
const DEFAULT_PLAID_REDIRECT_URI: &str = "http://localhost:5173/plaid-oauth";

type PlaidResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone)]
pub struct PlaidClient {
    http: reqwest::Client,
    client_id: String,
    secret: String,
    base_url: String,
    redirect_uri: Option<String>,
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
    pub personal_finance_category: Option<PlaidPersonalFinanceCategory>,
    pub date: NaiveDate,
    pub authorized_date: Option<NaiveDate>,
    pub pending: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaidPersonalFinanceCategory {
    pub primary: Option<String>,
    pub detailed: Option<String>,
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
        let plaid_env = std::env::var(PLAID_ENV_ENV).unwrap_or_default();
        let redirect_uri = plaid_redirect_uri();

        Ok(Self {
            http: reqwest::Client::new(),
            client_id,
            secret,
            base_url: plaid_base_url(&plaid_env),
            redirect_uri,
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
            redirect_uri: self.redirect_uri.as_deref(),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect_uri: Option<&'a str>,
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
    request_id: Option<String>,
}

async fn ensure_success(response: reqwest::Response) -> PlaidResult<reqwest::Response> {
    if response.status().is_success() {
        return Ok(response);
    }

    let status = response.status();
    let plaid_error = response.json::<PlaidErrorResponse>().await.ok();
    let message = plaid_error
        .and_then(|error| match (error.error_code, error.error_message) {
            (Some(code), Some(message)) if code == "INVALID_API_KEYS" => {
                let request_id = error
                    .request_id
                    .map(|request_id| format!(" Request ID: {request_id}."))
                    .unwrap_or_default();
                Some(format!(
                    "Plaid request failed ({code}): {message}. Check PLAID_CLIENT_ID and PLAID_SECRET for the configured PLAID_ENV. For PLAID_ENV=production, the Plaid account must have Production access enabled and PLAID_SECRET must be the Production secret, not the sandbox secret.{request_id}"
                ))
            }
            (Some(code), Some(message)) => {
                let request_id = error
                    .request_id
                    .map(|request_id| format!(" Request ID: {request_id}."))
                    .unwrap_or_default();
                Some(format!("Plaid request failed ({code}): {message}.{request_id}"))
            }
            (_, Some(message)) => Some(format!("Plaid request failed: {message}")),
            _ => None,
        })
        .unwrap_or_else(|| format!("Plaid request failed with status {status}"));

    Err(std::io::Error::other(message).into())
}

fn env_required(name: &str) -> PlaidResult<String> {
    let value = std::env::var(name).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::NotFound, format!("{name} must be set"))
    })?;

    // Trim so a stray trailing newline/space in `.env` cannot corrupt the
    // credential and trigger a misleading INVALID_API_KEYS from Plaid.
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("{name} must be set"),
        )
        .into());
    }

    Ok(trimmed.to_string())
}

fn plaid_redirect_uri() -> Option<String> {
    let value = std::env::var(PLAID_REDIRECT_URI_ENV)
        .unwrap_or_else(|_| DEFAULT_PLAID_REDIRECT_URI.to_string());

    match value.trim() {
        "" => None,
        trimmed => Some(trimmed.to_string()),
    }
}

fn plaid_base_url(plaid_env: &str) -> String {
    // Plaid has two live environments: Sandbox and Production. The legacy
    // `development` host was decommissioned (June 2024), so anything that is not
    // explicitly `production` falls back to the safe Sandbox default. Matching is
    // case-insensitive and whitespace-tolerant so `PLAID_ENV=production` always
    // reaches https://production.plaid.com.
    match plaid_env.trim().to_ascii_lowercase().as_str() {
        "production" => "https://production.plaid.com".to_string(),
        _ => "https://sandbox.plaid.com".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_supported_plaid_environments() {
        assert_eq!(plaid_base_url("sandbox"), "https://sandbox.plaid.com");
        assert_eq!(plaid_base_url("production"), "https://production.plaid.com");
        // Case and surrounding whitespace are normalized so production
        // credentials are never accidentally sent to the sandbox host.
        assert_eq!(
            plaid_base_url("  Production\n"),
            "https://production.plaid.com"
        );
    }

    #[test]
    fn defaults_unknown_or_unset_plaid_environment_to_sandbox() {
        assert_eq!(plaid_base_url(""), "https://sandbox.plaid.com");
        assert_eq!(plaid_base_url("local"), "https://sandbox.plaid.com");
        // `development` was decommissioned by Plaid; treat it as unknown.
        assert_eq!(plaid_base_url("development"), "https://sandbox.plaid.com");
    }

    #[test]
    fn resolves_redirect_uri_from_env() {
        // The default applies when unset, a custom value is used verbatim, and a
        // blank value is omitted from the request. Kept in one test to avoid
        // races on the shared environment variable across parallel tests.
        // SAFETY: this is the only test touching PLAID_REDIRECT_URI.
        unsafe {
            std::env::remove_var(PLAID_REDIRECT_URI_ENV);
        }
        assert_eq!(
            plaid_redirect_uri().as_deref(),
            Some(DEFAULT_PLAID_REDIRECT_URI)
        );

        unsafe {
            std::env::set_var(
                PLAID_REDIRECT_URI_ENV,
                "https://app.example.com/plaid-oauth",
            );
        }
        assert_eq!(
            plaid_redirect_uri().as_deref(),
            Some("https://app.example.com/plaid-oauth")
        );

        unsafe {
            std::env::set_var(PLAID_REDIRECT_URI_ENV, "   ");
        }
        assert_eq!(plaid_redirect_uri(), None);

        unsafe {
            std::env::remove_var(PLAID_REDIRECT_URI_ENV);
        }
    }
}
