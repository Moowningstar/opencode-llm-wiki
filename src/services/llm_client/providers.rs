use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use std::fmt::Debug;

use super::{ChatMessage, RequestOverrides};

pub trait Provider: Send + Sync + Debug {
    fn get_config(&self) -> ProviderConfig;
    fn build_body(&self, messages: &[ChatMessage], overrides: Option<&RequestOverrides>) -> Result<Value>;
    fn parse_stream(&self, line: &str) -> Result<Option<String>>;
}

#[derive(Clone, Debug)]
pub struct ProviderConfig {
    pub url: String,
    pub headers: HeaderMap,
}

pub fn create_provider(
    provider_name: &str,
    api_key: String,
    base_url: Option<String>,
) -> Result<Box<dyn Provider>> {
    match provider_name.to_lowercase().as_str() {
        "openai" => Ok(Box::new(OpenAiProvider::new(api_key, base_url)?)),
        "anthropic" => Ok(Box::new(AnthropicProvider::new(api_key, base_url)?)),
        "google" => Ok(Box::new(GoogleProvider::new(api_key, base_url)?)),
        _ => anyhow::bail!("Unsupported provider: {}", provider_name),
    }
}

// ────────────────────────────────────────────────────────────────────────────
// OpenAI Provider
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct OpenAiProvider {
    config: ProviderConfig,
    model: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String, base_url: Option<String>) -> Result<Self> {
        let url = base_url.unwrap_or_else(|| "https://api.openai.com/v1/chat/completions".to_string());
        
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key))
                .context("Invalid API key format")?,
        );

        Ok(Self {
            config: ProviderConfig { url, headers },
            model: "gpt-4".to_string(),
        })
    }
}

impl Provider for OpenAiProvider {
    fn get_config(&self) -> ProviderConfig {
        self.config.clone()
    }

    fn build_body(&self, messages: &[ChatMessage], overrides: Option<&RequestOverrides>) -> Result<Value> {
        let mut body = json!({
            "model": self.model,
            "messages": messages,
            "stream": true,
        });

        if let Some(overrides) = overrides {
            if let Some(temp) = overrides.temperature {
                body["temperature"] = json!(temp);
            }
            if let Some(max_tokens) = overrides.max_tokens {
                body["max_tokens"] = json!(max_tokens);
            }
            if let Some(top_p) = overrides.top_p {
                body["top_p"] = json!(top_p);
            }
        }

        Ok(body)
    }

    fn parse_stream(&self, line: &str) -> Result<Option<String>> {
        if !line.starts_with("data: ") {
            return Ok(None);
        }

        let data = &line[6..];
        if data == "[DONE]" {
            return Ok(None);
        }

        let parsed: Value = serde_json::from_str(data)
            .context("Failed to parse SSE data")?;

        if let Some(delta) = parsed["choices"][0]["delta"]["content"].as_str() {
            return Ok(Some(delta.to_string()));
        }

        Ok(None)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Anthropic Provider
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct AnthropicProvider {
    config: ProviderConfig,
    model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, base_url: Option<String>) -> Result<Self> {
        let url = base_url.unwrap_or_else(|| "https://api.anthropic.com/v1/messages".to_string());
        
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&api_key).context("Invalid API key format")?,
        );
        headers.insert(
            "anthropic-version",
            HeaderValue::from_static("2023-06-01"),
        );

        Ok(Self {
            config: ProviderConfig { url, headers },
            model: "claude-3-5-sonnet-20241022".to_string(),
        })
    }
}

impl Provider for AnthropicProvider {
    fn get_config(&self) -> ProviderConfig {
        self.config.clone()
    }

    fn build_body(&self, messages: &[ChatMessage], overrides: Option<&RequestOverrides>) -> Result<Value> {
        let mut body = json!({
            "model": self.model,
            "messages": messages,
            "stream": true,
            "max_tokens": 4096,
        });

        if let Some(overrides) = overrides {
            if let Some(temp) = overrides.temperature {
                body["temperature"] = json!(temp);
            }
            if let Some(max_tokens) = overrides.max_tokens {
                body["max_tokens"] = json!(max_tokens);
            }
            if let Some(top_p) = overrides.top_p {
                body["top_p"] = json!(top_p);
            }
        }

        Ok(body)
    }

    fn parse_stream(&self, line: &str) -> Result<Option<String>> {
        if !line.starts_with("data: ") {
            return Ok(None);
        }

        let data = &line[6..];
        let parsed: Value = serde_json::from_str(data)
            .context("Failed to parse SSE data")?;

        if parsed["type"] == "content_block_delta" {
            if let Some(text) = parsed["delta"]["text"].as_str() {
                return Ok(Some(text.to_string()));
            }
        }

        Ok(None)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Google Provider (Gemini)
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct GoogleProvider {
    config: ProviderConfig,
    model: String,
}

impl GoogleProvider {
    pub fn new(api_key: String, base_url: Option<String>) -> Result<Self> {
        let model = "gemini-1.5-pro";
        let url = base_url.unwrap_or_else(|| {
            format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?key={}",
                model, api_key
            )
        });
        
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        Ok(Self {
            config: ProviderConfig { url, headers },
            model: model.to_string(),
        })
    }
}

impl Provider for GoogleProvider {
    fn get_config(&self) -> ProviderConfig {
        self.config.clone()
    }

    fn build_body(&self, messages: &[ChatMessage], overrides: Option<&RequestOverrides>) -> Result<Value> {
        // Convert messages to Gemini format
        let contents: Vec<Value> = messages
            .iter()
            .map(|msg| {
                json!({
                    "role": if msg.role == "assistant" { "model" } else { "user" },
                    "parts": [{"text": msg.content}]
                })
            })
            .collect();

        let mut generation_config = json!({});
        if let Some(overrides) = overrides {
            if let Some(temp) = overrides.temperature {
                generation_config["temperature"] = json!(temp);
            }
            if let Some(max_tokens) = overrides.max_tokens {
                generation_config["maxOutputTokens"] = json!(max_tokens);
            }
            if let Some(top_p) = overrides.top_p {
                generation_config["topP"] = json!(top_p);
            }
        }

        Ok(json!({
            "contents": contents,
            "generationConfig": generation_config,
        }))
    }

    fn parse_stream(&self, line: &str) -> Result<Option<String>> {
        if !line.starts_with("data: ") {
            return Ok(None);
        }

        let data = &line[6..];
        let parsed: Value = serde_json::from_str(data)
            .context("Failed to parse SSE data")?;

        if let Some(text) = parsed["candidates"][0]["content"]["parts"][0]["text"].as_str() {
            return Ok(Some(text.to_string()));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_provider_creation() {
        let provider = OpenAiProvider::new("test-key".to_string(), None).unwrap();
        let config = provider.get_config();
        assert!(config.url.contains("openai.com"));
    }

    #[test]
    fn test_anthropic_provider_creation() {
        let provider = AnthropicProvider::new("test-key".to_string(), None).unwrap();
        let config = provider.get_config();
        assert!(config.url.contains("anthropic.com"));
    }

    #[test]
    fn test_google_provider_creation() {
        let provider = GoogleProvider::new("test-key".to_string(), None).unwrap();
        let config = provider.get_config();
        assert!(config.url.contains("generativelanguage.googleapis.com"));
    }
}
