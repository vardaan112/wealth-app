use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;

const SNAPTRADE_CLIENT_ID_ENV: &str = "SNAPTRADE_CLIENT_ID";
const SNAPTRADE_CONSUMER_KEY_ENV: &str = "SNAPTRADE_CONSUMER_KEY";
const SNAPTRADE_BASE_URL: &str = "https://api.snaptrade.com/api/v1";

type SnapTradeResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct SnapTradeClient {
    http: reqwest::Client,
    client_id: String,
    consumer_key: String,
    base_url: String,
}

#[derive(Debug, Clone)]
pub struct SnapTradeUser {
    pub user_id: String,
    pub user_secret: String,
}

impl SnapTradeClient {
    pub fn from_env() -> SnapTradeResult<Self> {
        Ok(Self {
            http: reqwest::Client::new(),
            client_id: env_required(SNAPTRADE_CLIENT_ID_ENV)?,
            consumer_key: env_required(SNAPTRADE_CONSUMER_KEY_ENV)?,
            base_url: SNAPTRADE_BASE_URL.to_string(),
        })
    }

    pub async fn register_user(&self, user_id: &str) -> SnapTradeResult<SnapTradeUser> {
        let path = "/snapTrade/registerUser";
        let body = RegisterUserRequest {
            user_id: user_id.to_string(),
        };
        let body_value = serde_json::to_value(&body)?;
        let request = self.signed_post(path, vec![], Some(body_value))?;
        let response = request.json(&body).send().await?;
        let response = ensure_success(response)
            .await?
            .json::<RegisterUserResponse>()
            .await?;

        Ok(SnapTradeUser {
            user_id: response.user_id,
            user_secret: response.user_secret,
        })
    }

    pub async fn create_connection_portal_url(
        &self,
        user_id: &str,
        user_secret: &str,
    ) -> SnapTradeResult<String> {
        let path = "/snapTrade/login";
        let request = self.signed_post(
            path,
            vec![
                ("userId", user_id.to_string()),
                ("userSecret", user_secret.to_string()),
            ],
            None,
        )?;
        let response = request.send().await?;
        let response = ensure_success(response)
            .await?
            .json::<LoginResponse>()
            .await?;

        Ok(response.redirect_uri)
    }

    pub async fn list_accounts(
        &self,
        user_id: &str,
        user_secret: &str,
    ) -> SnapTradeResult<SnapTradeAccountsResponse> {
        let request = self.signed_get(
            "/accounts",
            vec![
                ("userId", user_id.to_string()),
                ("userSecret", user_secret.to_string()),
            ],
        )?;
        let raw = ensure_success(request.send().await?)
            .await?
            .json::<Value>()
            .await?;
        let accounts = serde_json::from_value::<Vec<SnapTradeAccount>>(raw.clone())?;

        Ok(SnapTradeAccountsResponse { accounts, raw })
    }

    pub async fn list_account_positions(
        &self,
        user_id: &str,
        user_secret: &str,
        account_id: &str,
    ) -> SnapTradeResult<SnapTradePositionsResponse> {
        let path = format!("/accounts/{account_id}/positions");
        let request = self.signed_get(
            &path,
            vec![
                ("userId", user_id.to_string()),
                ("userSecret", user_secret.to_string()),
            ],
        )?;
        let raw = ensure_success(request.send().await?)
            .await?
            .json::<Value>()
            .await?;
        let positions = serde_json::from_value::<Vec<SnapTradePosition>>(raw.clone())?;

        Ok(SnapTradePositionsResponse { positions, raw })
    }

    fn signed_get(
        &self,
        path: &str,
        query_pairs: Vec<(&str, String)>,
    ) -> SnapTradeResult<reqwest::RequestBuilder> {
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let query = query_string(
            std::iter::once(("clientId", self.client_id.clone()))
                .chain(query_pairs)
                .chain(std::iter::once(("timestamp", timestamp))),
        );
        let signature = sign_request(path, &query, None, &self.consumer_key)?;

        Ok(self
            .http
            .get(format!("{}{}?{}", self.base_url, path, query))
            .header("Signature", signature))
    }

    fn signed_post(
        &self,
        path: &str,
        query_pairs: Vec<(&str, String)>,
        body: Option<Value>,
    ) -> SnapTradeResult<reqwest::RequestBuilder> {
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let query = query_string(
            std::iter::once(("clientId", self.client_id.clone()))
                .chain(query_pairs)
                .chain(std::iter::once(("timestamp", timestamp))),
        );
        let signature = sign_request(path, &query, body.as_ref(), &self.consumer_key)?;

        Ok(self
            .http
            .post(format!("{}{}?{}", self.base_url, path, query))
            .header("Signature", signature))
    }
}

#[derive(Serialize)]
struct RegisterUserRequest {
    #[serde(rename = "userId")]
    user_id: String,
}

#[derive(Deserialize)]
struct RegisterUserResponse {
    #[serde(rename = "userId")]
    user_id: String,
    #[serde(rename = "userSecret")]
    user_secret: String,
}

#[derive(Deserialize)]
struct LoginResponse {
    #[serde(rename = "redirectURI")]
    redirect_uri: String,
}

#[derive(Debug, Clone)]
pub struct SnapTradeAccountsResponse {
    pub accounts: Vec<SnapTradeAccount>,
    pub raw: Value,
}

#[derive(Debug, Clone)]
pub struct SnapTradePositionsResponse {
    pub positions: Vec<SnapTradePosition>,
    pub raw: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SnapTradeAccount {
    pub id: String,
    pub name: Option<String>,
    pub number: Option<String>,
    pub institution_name: Option<String>,
    pub balance: Option<SnapTradeAccountBalance>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SnapTradeAccountBalance {
    pub total: Option<SnapTradeAmount>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SnapTradeAmount {
    pub amount: Option<f64>,
    pub currency: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SnapTradeAccountBalanceCents {
    pub balance_cents: i64,
    pub currency: String,
}

impl SnapTradeAccount {
    pub fn total_balance(&self) -> Option<SnapTradeAccountBalanceCents> {
        let total = self.balance.as_ref()?.total.as_ref()?;
        let amount = total.amount?;
        Some(SnapTradeAccountBalanceCents {
            balance_cents: (amount * 100.0).round() as i64,
            currency: total.currency.clone().unwrap_or_else(|| "USD".to_string()),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SnapTradePosition {
    pub symbol: Option<SnapTradePositionSymbol>,
    pub units: Option<f64>,
    pub price: Option<f64>,
    pub average_purchase_price: Option<f64>,
    pub currency: Option<SnapTradeCurrency>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SnapTradePositionSymbol {
    pub symbol: Option<SnapTradeUniversalSymbol>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SnapTradeUniversalSymbol {
    pub symbol: Option<String>,
    pub raw_symbol: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub security_type: Option<SnapTradeSecurityType>,
    pub currency: Option<SnapTradeCurrency>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SnapTradeSecurityType {
    pub code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SnapTradeCurrency {
    pub code: Option<String>,
}

async fn ensure_success(response: reqwest::Response) -> SnapTradeResult<reqwest::Response> {
    if response.status().is_success() {
        return Ok(response);
    }

    let status = response.status();
    let error = response.json::<Value>().await.ok();
    let message = error
        .and_then(|value| {
            value
                .get("message")
                .and_then(Value::as_str)
                .or_else(|| value.get("detail").and_then(Value::as_str))
                .map(str::to_string)
        })
        .unwrap_or_else(|| format!("SnapTrade request failed with status {status}"));

    Err(std::io::Error::other(message).into())
}

fn sign_request(
    path: &str,
    query: &str,
    body: Option<&Value>,
    consumer_key: &str,
) -> SnapTradeResult<String> {
    let signature_body = json!({
        "content": body,
        "path": format!("/api/v1{path}"),
        "query": query,
    });
    let signature_content = serde_json::to_string(&signature_body)?;
    let mut mac = HmacSha256::new_from_slice(consumer_key.as_bytes())?;
    mac.update(signature_content.as_bytes());

    Ok(general_purpose::STANDARD.encode(mac.finalize().into_bytes()))
}

fn query_string<'a>(pairs: impl IntoIterator<Item = (&'a str, String)>) -> String {
    pairs
        .into_iter()
        .map(|(key, value)| format!("{key}={}", percent_encode(&value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn percent_encode(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                vec![*byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

fn env_required(name: &str) -> SnapTradeResult<String> {
    std::env::var(name).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::NotFound, format!("{name} must be set")).into()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_expected_query_string_order() {
        let query = query_string(
            std::iter::once(("clientId", "client".to_string()))
                .chain(vec![("userId", "user".to_string())])
                .chain(std::iter::once(("timestamp", "123".to_string()))),
        );

        assert_eq!(query, "clientId=client&userId=user&timestamp=123");
    }

    #[test]
    fn query_values_are_percent_encoded() {
        let query = query_string(vec![("userSecret", "a+b c".to_string())]);

        assert_eq!(query, "userSecret=a%2Bb%20c");
    }

    #[test]
    fn signs_request_without_exposing_secret() {
        let body = json!({ "userId": "user-1" });
        let signature = sign_request(
            "/snapTrade/registerUser",
            "clientId=client&timestamp=123",
            Some(&body),
            "consumer-key",
        )
        .unwrap();

        assert!(!signature.is_empty());
        assert!(!signature.contains("consumer-key"));
    }
}
