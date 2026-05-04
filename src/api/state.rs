use std::sync::Arc;
use anyhow::Result;

#[cfg(feature = "ruvector")]
use crate::storage::ruvector_impl::RuVectorStorage;

#[cfg(feature = "lancedb-backend")]
use crate::storage::lancedb_impl::LanceDbStorage;

use crate::storage::traits::VectorStorage;
use crate::services::embedding::{EmbeddingService, EmbeddingConfig};
use crate::services::ingest::{IngestService, IngestConfig};
use crate::services::query::{QueryService, QueryConfig};
use crate::services::llm_client::LlmClient;
use crate::wiki::filesystem::WikiFileSystem;
use crate::wiki::index::IndexManager;
use crate::wiki::graph::GraphManager;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<dyn VectorStorage>,
    pub embedding_service: EmbeddingService,
    pub ingest_service: Arc<IngestService>,
    pub query_service: Arc<QueryService>,
    pub llm_client: Arc<LlmClient>,
    pub wiki_fs: Arc<WikiFileSystem>,
    pub index_manager: Arc<IndexManager>,
    pub graph_manager: Arc<GraphManager>,
}

impl AppState {
    pub async fn new(
        _db_path: &str,
        project_path: &str,
        embedding_config: EmbeddingConfig,
        ingest_config: IngestConfig,
        query_config: QueryConfig,
        llm_client: LlmClient,
    ) -> Result<Self> {
        #[cfg(feature = "ruvector")]
        let storage: Arc<dyn VectorStorage> = Arc::new(
            RuVectorStorage::new(project_path.to_string(), 2048).await?
        );

        #[cfg(all(feature = "lancedb-backend", not(feature = "ruvector")))]
        let storage: Arc<dyn VectorStorage> = Arc::new(
            LanceDbStorage::new(db_path.to_string())
        );

        #[cfg(not(any(feature = "ruvector", feature = "lancedb-backend")))]
        compile_error!("Either 'ruvector' or 'lancedb-backend' feature must be enabled");
        
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

        let wiki_fs = Arc::new(WikiFileSystem::new(project_path)?);
        wiki_fs.init()?;
        
        let index_manager = Arc::new(IndexManager::new(wiki_fs.clone()));
        let graph_manager = Arc::new(GraphManager::new(wiki_fs.clone()));

        Ok(Self {
            storage,
            embedding_service,
            ingest_service,
            query_service,
            llm_client: Arc::new(llm_client),
            wiki_fs,
            index_manager,
            graph_manager,
        })
    }

    pub async fn with_defaults(db_path: &str, project_path: &str, api_key: String) -> Result<Self> {
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
            project_path,
            embedding_config,
            IngestConfig::default(),
            QueryConfig::default(),
            llm_client,
        ).await
    }
}
