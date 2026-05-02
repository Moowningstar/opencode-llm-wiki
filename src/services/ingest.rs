use anyhow::{Context, Result};
use std::sync::Arc;

use crate::storage::traits::{ChunkInput, VectorStorage};
use super::chunking::{ChunkingService, ChunkingConfig};
use super::embedding::EmbeddingService;
use super::ingest_engine::{parse_file_blocks, is_safe_ingest_path};

/// Configuration for the ingestion pipeline.
#[derive(Debug, Clone)]
pub struct IngestConfig {
    pub chunking: ChunkingConfig,
    pub embedding_model: String,
    pub tokenizer_model: String,
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            chunking: ChunkingConfig::default(),
            embedding_model: "text-embedding-ada-002".to_string(),
            tokenizer_model: "gpt-4".to_string(),
        }
    }
}

/// Result of ingesting content.
#[derive(Debug)]
pub struct IngestResult {
    pub pages_processed: usize,
    pub chunks_created: usize,
    pub warnings: Vec<String>,
}

/// Service for ingesting markdown content into the vector store.
pub struct IngestService {
    storage: Arc<dyn VectorStorage>,
    embedding_service: EmbeddingService,
    chunking_service: ChunkingService,
    config: IngestConfig,
}

impl IngestService {
    /// Create a new ingest service.
    pub fn new(
        storage: Arc<dyn VectorStorage>,
        embedding_service: EmbeddingService,
        config: IngestConfig,
    ) -> Result<Self> {
        let tokenizer = tiktoken_rs::get_bpe_from_model(&config.tokenizer_model)
            .context("Failed to load tokenizer")?;

        let chunking_service = ChunkingService::with_config(config.chunking.clone())
            .with_tokenizer(tokenizer);

        Ok(Self {
            storage,
            embedding_service,
            chunking_service,
            config,
        })
    }

    /// Ingest content containing FILE blocks.
    pub async fn ingest_file_blocks(&self, content: &str) -> Result<IngestResult> {
        let parse_result = parse_file_blocks(content);
        let mut warnings = parse_result.warnings;
        let mut total_chunks = 0;
        let blocks_count = parse_result.blocks.len();

        for block in parse_result.blocks {
            if !is_safe_ingest_path(&block.path) {
                warnings.push(format!("Skipping unsafe path: {}", block.path));
                continue;
            }

            let page_id = block.path.clone();
            let chunks = self.chunking_service.chunk_markdown(&block.content)
                .context(format!("Failed to chunk page: {}", page_id))?;

            if chunks.is_empty() {
                warnings.push(format!("No chunks generated for page: {}", page_id));
                continue;
            }

            let texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
            let embeddings = self.embedding_service.embed_texts(&texts).await
                .context(format!("Failed to generate embeddings for page: {}", page_id))?;

            if embeddings.len() != chunks.len() {
                warnings.push(format!(
                    "Embedding count mismatch for page {}: expected {}, got {}",
                    page_id, chunks.len(), embeddings.len()
                ));
                continue;
            }

            let chunk_inputs: Vec<ChunkInput> = chunks.iter().zip(embeddings.iter())
                .map(|(chunk, embedding)| ChunkInput {
                    chunk_index: chunk.index as u32,
                    chunk_text: chunk.text.clone(),
                    heading_path: chunk.heading_path.clone(),
                    embedding: embedding.clone(),
                    token_ids: chunk.token_ids.clone(),
                    token_count: chunk.token_count,
                })
                .collect();

            self.storage.upsert_chunks(&page_id, chunk_inputs).await
                .context(format!("Failed to store chunks for page: {}", page_id))?;

            total_chunks += chunks.len();
        }

        Ok(IngestResult {
            pages_processed: blocks_count,
            chunks_created: total_chunks,
            warnings,
        })
    }

    /// Ingest a single markdown page.
    pub async fn ingest_page(&self, page_id: &str, content: &str) -> Result<IngestResult> {
        if !is_safe_ingest_path(page_id) {
            anyhow::bail!("Unsafe page path: {}", page_id);
        }

        let chunks = self.chunking_service.chunk_markdown(content)
            .context("Failed to chunk page")?;

        if chunks.is_empty() {
            return Ok(IngestResult {
                pages_processed: 1,
                chunks_created: 0,
                warnings: vec!["No chunks generated".to_string()],
            });
        }

        let texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
        let embeddings = self.embedding_service.embed_texts(&texts).await
            .context("Failed to generate embeddings")?;

        if embeddings.len() != chunks.len() {
            anyhow::bail!(
                "Embedding count mismatch: expected {}, got {}",
                chunks.len(),
                embeddings.len()
            );
        }

        let chunk_inputs: Vec<ChunkInput> = chunks.iter().zip(embeddings.iter())
            .map(|(chunk, embedding)| ChunkInput {
                chunk_index: chunk.index as u32,
                chunk_text: chunk.text.clone(),
                heading_path: chunk.heading_path.clone(),
                embedding: embedding.clone(),
                token_ids: chunk.token_ids.clone(),
                token_count: chunk.token_count,
            })
            .collect();

        self.storage.upsert_chunks(page_id, chunk_inputs).await
            .context("Failed to store chunks")?;

        Ok(IngestResult {
            pages_processed: 1,
            chunks_created: chunks.len(),
            warnings: Vec::new(),
        })
    }

    /// Delete a page and all its chunks from the vector store.
    pub async fn delete_page(&self, page_id: &str) -> Result<()> {
        self.storage.delete_page(page_id).await
            .context("Failed to delete page")
    }

    /// Get the total number of chunks in the vector store.
    pub async fn count_chunks(&self) -> Result<usize> {
        self.storage.count().await
            .context("Failed to count chunks")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ingest_config_default() {
        let config = IngestConfig::default();
        assert_eq!(config.embedding_model, "text-embedding-ada-002");
        assert_eq!(config.tokenizer_model, "gpt-4");
        assert_eq!(config.chunking.target_chars, 1000);
    }
}
