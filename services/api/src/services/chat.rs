use sqlx::PgPool;
use uuid::Uuid;

use crate::repositories::chat_messages::{self, ChatMessageRecord, CreateChatMessageInput};
use crate::repositories::holdings;
use crate::services::snapshots;

pub struct OpenAiConfig {
    pub api_key: String,
    pub model: String,
}

impl OpenAiConfig {
    pub fn from_env() -> Result<Self, String> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| "Add OPENAI_API_KEY to services/api/.env".to_string())?;
        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
        Ok(Self { api_key, model })
    }
}

#[derive(Debug, Clone)]
pub struct SendChatResult {
    pub user_message: ChatMessageRecord,
    pub assistant_message: ChatMessageRecord,
}

#[derive(serde::Serialize)]
struct OpenAiMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(serde::Serialize)]
struct OpenAiRequest<'a> {
    model: &'a str,
    messages: Vec<OpenAiMessage<'a>>,
}

#[derive(serde::Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
}

#[derive(serde::Deserialize)]
struct OpenAiResponseMessage {
    content: String,
}

#[derive(serde::Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

pub fn format_cents(cents: i64) -> String {
    format!("${:.2}", cents as f64 / 100.0)
}

pub fn build_portfolio_context(
    holdings: &[holdings::HoldingRecord],
    net_worth: Option<&snapshots::CurrentNetWorth>,
) -> String {
    let mut lines = Vec::new();

    if let Some(nw) = net_worth {
        lines.push(format!(
            "Net worth: {} (cash: {}, investments: {}, debt: {})",
            format_cents(nw.net_worth_cents),
            format_cents(nw.cash_cents),
            format_cents(nw.investment_value_cents),
            format_cents(nw.debt_cents),
        ));
    }

    if holdings.is_empty() {
        lines.push("Holdings: none synced yet.".to_string());
    } else {
        lines.push("Holdings:".to_string());
        for holding in holdings {
            let value = holding.market_value_cents.unwrap_or_default();
            lines.push(format!(
                "- {} ({}, {}): qty {}, market value {}",
                holding.symbol,
                holding.asset_name.as_deref().unwrap_or("unknown"),
                holding.asset_type,
                holding.quantity,
                format_cents(value),
            ));
        }
    }

    lines.join("\n")
}

pub fn build_system_prompt(portfolio_context: &str) -> String {
    format!(
        "You are a personal portfolio advisor for a single investor using a private wealth dashboard. \
         Answer clearly and concisely. Base guidance on the user's actual holdings below. \
         When asked for buy/hold/sell guidance, give actionable per-symbol alerts and note this is \
         general information, not licensed financial advice.\n\n\
         Current portfolio:\n{portfolio_context}"
    )
}

pub async fn send_chat_message(
    pool: &PgPool,
    user_id: Uuid,
    content: &str,
    is_briefing: bool,
) -> Result<SendChatResult, String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err("message cannot be empty".to_string());
    }

    let config = OpenAiConfig::from_env()?;
    let holdings = holdings::list_holdings(pool, user_id)
        .await
        .map_err(|e| format!("could not load holdings: {e}"))?;
    let net_worth = snapshots::compute_current_net_worth(pool, user_id)
        .await
        .map_err(|e| format!("could not load net worth: {e}"))?;
    let portfolio_context = build_portfolio_context(&holdings, Some(&net_worth));
    let system_prompt = build_system_prompt(&portfolio_context);

    let history = chat_messages::list_chat_messages(pool, user_id)
        .await
        .map_err(|e| format!("could not load chat history: {e}"))?;

    let mut messages = vec![OpenAiMessage {
        role: "system",
        content: &system_prompt,
    }];

    for message in &history {
        if message.role == "user" || message.role == "assistant" {
            messages.push(OpenAiMessage {
                role: &message.role,
                content: &message.content,
            });
        }
    }

    messages.push(OpenAiMessage {
        role: "user",
        content: trimmed,
    });

    let request = OpenAiRequest {
        model: &config.model,
        messages,
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&config.api_key)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("OpenAI request failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        tracing::warn!("OpenAI API error status={status}");
        return Err(format!(
            "OpenAI request failed ({status}). Check OPENAI_API_KEY and billing."
        ));
    }

    let payload: OpenAiResponse = response
        .json()
        .await
        .map_err(|e| format!("invalid OpenAI response: {e}"))?;

    let assistant_content = payload
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message.content.trim().to_string())
        .filter(|content| !content.is_empty())
        .ok_or_else(|| "OpenAI returned an empty response".to_string())?;

    let user_message = chat_messages::create_chat_message(
        pool,
        user_id,
        CreateChatMessageInput {
            role: "user".to_string(),
            content: trimmed.to_string(),
            is_briefing: false,
        },
    )
    .await
    .map_err(|e| format!("could not save user message: {e}"))?;

    let assistant_message = chat_messages::create_chat_message(
        pool,
        user_id,
        CreateChatMessageInput {
            role: "assistant".to_string(),
            content: assistant_content,
            is_briefing,
        },
    )
    .await
    .map_err(|e| format!("could not save assistant message: {e}"))?;

    Ok(SendChatResult {
        user_message,
        assistant_message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_holding() -> holdings::HoldingRecord {
        holdings::HoldingRecord {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            account_id: Uuid::new_v4(),
            provider: "manual".to_string(),
            provider_holding_id: None,
            symbol: "AAPL".to_string(),
            asset_name: Some("Apple Inc.".to_string()),
            asset_type: "stock".to_string(),
            quantity: 10.0,
            market_value_cents: Some(1_900_00),
            cost_basis_cents: None,
            price_cents: None,
            currency: "USD".to_string(),
            as_of: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn portfolio_context_includes_symbol_and_value() {
        let context = build_portfolio_context(
            &[sample_holding()],
            Some(&snapshots::CurrentNetWorth {
                cash_cents: 1_000_00,
                investment_value_cents: 1_900_00,
                debt_cents: 0,
                net_worth_cents: 2_900_00,
            }),
        );

        assert!(context.contains("AAPL"));
        assert!(context.contains("$1900.00"));
        assert!(context.contains("Net worth"));
    }

    #[test]
    fn system_prompt_mentions_portfolio() {
        let prompt = build_system_prompt("Holdings:\n- VTI");
        assert!(prompt.contains("portfolio advisor"));
        assert!(prompt.contains("VTI"));
    }
}
