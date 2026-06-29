use std::env;

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{Value, json};

use crate::cli::ProviderKind;

pub struct ChatRequest {
    pub project_name: String,
    pub message: String,
}

pub struct ChatResponse {
    pub message: String,
}

#[async_trait]
pub trait LlmProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
}

pub fn provider_from_env(kind: ProviderKind) -> Result<Box<dyn LlmProvider + Send + Sync>> {
    match kind {
        ProviderKind::Openai => Ok(Box::new(OpenAiProvider::from_env()?)),
        ProviderKind::Claude => Ok(Box::new(ClaudeProvider::from_env()?)),
        ProviderKind::Gemini => Ok(Box::new(GeminiProvider::from_env()?)),
        ProviderKind::Grok => Ok(Box::new(GrokProvider::from_env()?)),
    }
}

struct OpenAiProvider {
    client: Client,
    api_key: String,
    model: String,
}

struct ClaudeProvider {
    client: Client,
    api_key: String,
    model: String,
}

struct GeminiProvider {
    client: Client,
    api_key: String,
    model: String,
}

struct GrokProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAiProvider {
    fn from_env() -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            api_key: required_env("OPENAI_API_KEY")?,
            model: env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4.1".to_string()),
        })
    }
}

impl ClaudeProvider {
    fn from_env() -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            api_key: required_env("ANTHROPIC_API_KEY")?,
            model: env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-sonnet-4-5".to_string()),
        })
    }
}

impl GeminiProvider {
    fn from_env() -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            api_key: required_env("GEMINI_API_KEY")?,
            model: env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.5-pro".to_string()),
        })
    }
}

impl GrokProvider {
    fn from_env() -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            api_key: required_env("XAI_API_KEY")?,
            model: env::var("XAI_MODEL").unwrap_or_else(|_| "grok-4".to_string()),
        })
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let body = json!({
            "model": self.model,
            "messages": messages(&request),
        });
        let value = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;
        extract_openai_message(value)
    }
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let body = json!({
            "model": self.model,
            "max_tokens": 1200,
            "system": system_prompt(&request),
            "messages": [{ "role": "user", "content": request.message }],
        });
        let value = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;
        let message = value["content"]
            .as_array()
            .and_then(|items| items.iter().find_map(|item| item["text"].as_str()))
            .context("Anthropic response did not contain text content")?;
        Ok(ChatResponse {
            message: message.to_string(),
        })
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );
        let body = json!({
            "systemInstruction": {
                "parts": [{ "text": system_prompt(&request) }]
            },
            "contents": [{
                "role": "user",
                "parts": [{ "text": request.message }]
            }]
        });
        let value = self
            .client
            .post(url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;
        let message = value["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .context("Gemini response did not contain text content")?;
        Ok(ChatResponse {
            message: message.to_string(),
        })
    }
}

#[async_trait]
impl LlmProvider for GrokProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let body = json!({
            "model": self.model,
            "messages": messages(&request),
        });
        let value = self
            .client
            .post("https://api.x.ai/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;
        extract_openai_message(value)
    }
}

fn messages(request: &ChatRequest) -> Vec<Value> {
    vec![
        json!({ "role": "system", "content": system_prompt(request) }),
        json!({ "role": "user", "content": request.message }),
    ]
}

fn system_prompt(request: &ChatRequest) -> String {
    format!(
        "You are a concise project planning assistant for the '{}' project. Help turn discussions into clear todos and roadmap steps.",
        request.project_name
    )
}

fn extract_openai_message(value: Value) -> Result<ChatResponse> {
    let message = value["choices"][0]["message"]["content"]
        .as_str()
        .context("chat completion response did not contain message content")?;
    Ok(ChatResponse {
        message: message.to_string(),
    })
}

fn required_env(name: &str) -> Result<String> {
    env::var(name).map_err(|_| anyhow!("{name} is required for this provider"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_openai_compatible_message() {
        let value = json!({
            "choices": [{ "message": { "content": "hello" } }]
        });

        let response = extract_openai_message(value).unwrap();
        assert_eq!(response.message, "hello");
    }
}
