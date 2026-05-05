use anyhow::{Context, Result};
use std::sync::Arc;

use crate::storage::traits::{SearchResult, VectorStorage};
use super::embedding::EmbeddingService;

#[derive(Debug, Clone)]
pub struct QueryConfig {
    pub top_k: usize,
    pub min_score: f32,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            top_k: 10,
            min_score: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct QueryResult {
    pub results: Vec<SearchResult>,
    pub query_embedding: Vec<f32>,
}

pub struct QueryService {
    storage: Arc<dyn VectorStorage>,
    embedding_service: EmbeddingService,
    config: QueryConfig,
}

impl QueryService {
    pub fn new(
        storage: Arc<dyn VectorStorage>,
        embedding_service: EmbeddingService,
        config: QueryConfig,
    ) -> Self {
        Self {
            storage,
            embedding_service,
            config,
        }
    }

    pub async fn query(&self, query_text: &str) -> Result<QueryResult> {
        self.query_with_filter(query_text, None).await
    }

    pub async fn query_with_filter(&self, query_text: &str, project_filter: Option<&str>) -> Result<QueryResult> {
        let query_embedding = self.embedding_service.embed_text(query_text).await
            .context("Failed to generate query embedding")?;

        let mut results = self.storage.search(query_embedding.clone(), self.config.top_k, project_filter).await
            .context("Failed to search vector storage")?;

        if self.config.min_score > 0.0 {
            results.retain(|r| r.score >= self.config.min_score);
        }

        Ok(QueryResult {
            results,
            query_embedding,
        })
    }

    pub async fn query_with_embedding(&self, query_embedding: Vec<f32>) -> Result<QueryResult> {
        self.query_with_embedding_and_filter(query_embedding, None).await
    }

    pub async fn query_with_embedding_and_filter(&self, query_embedding: Vec<f32>, project_filter: Option<&str>) -> Result<QueryResult> {
        let mut results = self.storage.search(query_embedding.clone(), self.config.top_k, project_filter).await
            .context("Failed to search vector storage")?;

        if self.config.min_score > 0.0 {
            results.retain(|r| r.score >= self.config.min_score);
        }

        Ok(QueryResult {
            results,
            query_embedding,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_config_default() {
        let config = QueryConfig::default();
        assert_eq!(config.top_k, 10);
        assert_eq!(config.min_score, 0.0);
    }
}
