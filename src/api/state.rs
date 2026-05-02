use std::sync::Arc;
use anyhow::Result;

use crate::storage::lancedb_impl::LanceDbStorage;
use crate::storage::traits::VectorStorage;
use crate::services::embedding::{EmbeddingService, EmbeddingConfig};
use crate::services::ingest::{IngestService, IngestConfig};
use crate::services::query::{QueryService, QueryConfig};
use crate::services::llm_client::LlmClient;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<dyn VectorStorage>,
    pub embedding_service: EmbeddingService,
    pub ingest_service: Arc<IngestService>,
    pub query_service: Arc<QueryService>,
    pub llm_client: Arc<LlmClient>,
}

impl AppState {
    pub async fn new(
        db_path: &str,
        embedding_config: EmbeddingConfig,
        ingest_config: IngestConfig,
        query_config: QueryConfig,
        llm_client: LlmClient,
    ) -> Result<Self> {
        let storage = Arc::new(LanceDbStorage::new(db_path.to_string()));
        
        let embedding_service = EmbeddingService::new(embedding_config.clone())?;
        
        let ingest_service = Arc::new(IngestService::new(
            storage.clone(),
            embedding_service.clone(),
            ingest_config,
        )?);
        
        let query_service = Arc::new(QueryService::new(
            storage.clone(),
            embedding_service.clone(),
            query_config,
        ));

        Ok(Self {
            storage,
            embedding_service,
            ingest_service,
            query_service,
            llm_client: Arc::new(llm_client),
        })
    }

    pub async fn with_defaults(db_path: &str, api_key: String) -> Result<Self> {
        let embedding_config = EmbeddingConfig {
            api_key: api_key.clone(),
            ..Default::default()
        };

        let llm_client = LlmClient::new(
            "openai",
            api_key,
            Some("https://api.openai.com/v1".to_string()),
        )?;

        Self::new(
            db_path,
            embedding_config,
            IngestConfig::default(),
            QueryConfig::default(),
            llm_client,
        ).await
    }
}
