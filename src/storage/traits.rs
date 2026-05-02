use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Input for upserting a chunk into vector storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInput {
    pub chunk_index: u32,
    pub chunk_text: String,
    pub heading_path: String,
    pub embedding: Vec<f32>,
    /// Optional pre-computed token IDs for cache
    pub token_ids: Option<Vec<u32>>,
    /// Optional pre-computed token count for cache
    pub token_count: Option<u32>,
}

/// Result from vector search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk_id: String,
    pub page_id: String,
    pub chunk_index: u32,
    pub chunk_text: String,
    pub heading_path: String,
    pub score: f32,
    /// Optional cached token IDs (if available in storage)
    pub token_ids: Option<Vec<u32>>,
    /// Optional cached token count (if available in storage)
    pub token_count: Option<u32>,
}

/// Unified error type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

#[derive(Debug, Clone)]
pub struct StorageError {
    pub message: String,
}

impl StorageError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Storage error: {}", self.message)
    }
}

impl std::error::Error for StorageError {}

impl From<String> for StorageError {
    fn from(s: String) -> Self {
        StorageError::new(s)
    }
}

impl From<&str> for StorageError {
    fn from(s: &str) -> Self {
        StorageError::new(s)
    }
}

/// Trait for vector storage backends (LanceDB, RuVector, etc.)
/// 
/// This abstraction allows swapping storage implementations without
/// changing business logic in Layer 2 (services) or Layer 1 (API/CLI).
#[async_trait]
pub trait VectorStorage: Send + Sync {
    /// Insert or update chunks for a given page
    /// 
    /// # Arguments
    /// * `page_id` - Unique identifier for the page (alphanumeric + [-_.])
    /// * `chunks` - Vector of chunks with embeddings and optional token cache
    /// 
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(StorageError)` if validation fails or storage operation fails
    async fn upsert_chunks(&self, page_id: &str, chunks: Vec<ChunkInput>) -> StorageResult<()>;

    /// Search for similar chunks using vector similarity
    /// 
    /// # Arguments
    /// * `query_embedding` - Query vector (must match storage dimension)
    /// * `top_k` - Maximum number of results to return
    /// 
    /// # Returns
    /// * `Ok(Vec<SearchResult>)` - Results sorted by similarity (highest first)
    /// * `Err(StorageError)` if search fails
    async fn search(
        &self,
        query_embedding: Vec<f32>,
        top_k: usize,
    ) -> StorageResult<Vec<SearchResult>>;

    /// Delete all chunks for a given page
    /// 
    /// # Arguments
    /// * `page_id` - Page identifier to delete
    /// 
    /// # Returns
    /// * `Ok(())` on success (idempotent - no error if page doesn't exist)
    /// * `Err(StorageError)` if deletion fails
    async fn delete_page(&self, page_id: &str) -> StorageResult<()>;

    /// Count total number of chunks in storage
    /// 
    /// # Returns
    /// * `Ok(usize)` - Total chunk count
    /// * `Err(StorageError)` if count operation fails
    async fn count(&self) -> StorageResult<usize>;

    /// Get embedding dimension for this storage backend
    /// 
    /// # Returns
    /// * Embedding dimension (e.g., 1536 for OpenAI text-embedding-ada-002)
    fn embedding_dim(&self) -> usize;
}
