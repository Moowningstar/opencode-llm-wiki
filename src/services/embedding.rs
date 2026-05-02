use anyhow::{Context, Result};
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE}};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Configuration for embedding service
#[derive(Clone, Debug)]
pub struct EmbeddingConfig {
    /// API endpoint URL
    pub api_url: String,
    /// API key for authentication
    pub api_key: String,
    /// Model name (e.g., "text-embedding-ada-002", "text-embedding-3-small")
    pub model: String,
    /// Expected embedding dimension
    pub dimension: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            api_url: "https://api.openai.com/v1/embeddings".to_string(),
            api_key: String::new(),
            model: "text-embedding-ada-002".to_string(),
            dimension: 1536,
        }
    }
}

/// Service for generating text embeddings
#[derive(Clone)]
pub struct EmbeddingService {
    client: Client,
    config: Arc<EmbeddingConfig>,
}

impl EmbeddingService {
    /// Create a new embedding service with the given configuration
    pub fn new(config: EmbeddingConfig) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", config.api_key))
                .context("Invalid API key format")?,
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            config: Arc::new(config),
        })
    }

    /// Generate embedding for a single text
    ///
    /// # Arguments
    /// * `text` - Input text to embed
    ///
    /// # Returns
    /// * `Result<Vec<f32>>` - Embedding vector
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.embed_batch(vec![text]).await?;
        embeddings
            .into_iter()
            .next()
            .context("No embedding returned from API")
    }

    /// Generate embeddings for multiple texts in a single batch request
    ///
    /// # Arguments
    /// * `texts` - Vector of input texts to embed
    ///
    /// # Returns
    /// * `Result<Vec<Vec<f32>>>` - Vector of embedding vectors
    pub async fn embed_batch(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let request_body = EmbeddingRequest {
            input: texts.iter().map(|s| s.to_string()).collect(),
            model: self.config.model.clone(),
        };

        let response = self
            .client
            .post(&self.config.api_url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send embedding request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Embedding API error ({}): {}", status, error_text);
        }

        let response_body: EmbeddingResponse = response
            .json()
            .await
            .context("Failed to parse embedding response")?;

        for (i, embedding_data) in response_body.data.iter().enumerate() {
            if embedding_data.embedding.len() != self.config.dimension {
                anyhow::bail!(
                    "Embedding #{} has dimension {} but expected {}",
                    i,
                    embedding_data.embedding.len(),
                    self.config.dimension
                );
            }
        }

        Ok(response_body
            .data
            .into_iter()
            .map(|d| d.embedding)
            .collect())
    }

    pub async fn embed_texts(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        self.embed_batch(text_refs).await
    }

    pub fn dimension(&self) -> usize {
        self.config.dimension
    }
}

#[derive(Serialize)]
struct EmbeddingRequest {
    input: Vec<String>,
    model: String,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_config_default() {
        let config = EmbeddingConfig::default();
        assert_eq!(config.dimension, 1536);
        assert_eq!(config.model, "text-embedding-ada-002");
        assert!(config.api_url.contains("openai.com"));
    }

    #[test]
    fn test_embedding_service_creation() {
        let config = EmbeddingConfig {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let service = EmbeddingService::new(config);
        assert!(service.is_ok());
    }

    #[test]
    fn test_embedding_service_dimension() {
        let config = EmbeddingConfig {
            api_key: "test-key".to_string(),
            dimension: 768,
            ..Default::default()
        };
        let service = EmbeddingService::new(config).unwrap();
        assert_eq!(service.dimension(), 768);
    }
}
