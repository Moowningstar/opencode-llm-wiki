use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod providers;

use providers::Provider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct RequestOverrides {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
}

pub struct LlmClient {
    client: Client,
    provider: Box<dyn Provider>,
}

impl LlmClient {
    pub fn new(provider_name: &str, api_key: String, base_url: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30 * 60)) // 30 min backstop
            .build()
            .context("Failed to create HTTP client")?;

        let provider = providers::create_provider(provider_name, api_key, base_url)?;

        Ok(Self { client, provider })
    }

    pub async fn stream_chat<F>(
        &self,
        messages: Vec<ChatMessage>,
        mut on_token: F,
        overrides: Option<RequestOverrides>,
    ) -> Result<()>
    where
        F: FnMut(String) + Send,
    {
        let config = self.provider.get_config();
        let body = self.provider.build_body(&messages, overrides.as_ref())?;

        let response = self
            .client
            .post(&config.url)
            .headers(config.headers.clone())
            .json(&body)
            .send()
            .await
            .context("Failed to send request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("HTTP {}: {}", status, body);
        }

        let mut stream = response.bytes_stream();
        let mut line_buffer = String::new();

        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Stream read error")?;
            let text = String::from_utf8_lossy(&chunk);
            line_buffer.push_str(&text);

            while let Some(newline_pos) = line_buffer.find('\n') {
                let line = line_buffer[..newline_pos].trim().to_string();
                line_buffer = line_buffer[newline_pos + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

                if let Some(token) = self.provider.parse_stream(&line)? {
                    on_token(token);
                }
            }
        }

        // Process remaining buffer
        if !line_buffer.trim().is_empty() {
            if let Some(token) = self.provider.parse_stream(line_buffer.trim())? {
                on_token(token);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_serialization() {
        let msg = ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("user"));
        assert!(json.contains("Hello"));
    }
}
