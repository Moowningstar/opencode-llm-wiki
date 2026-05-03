use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tracing::info;

use super::{routes::create_router, state::AppState, config};
use crate::services::embedding::EmbeddingConfig;
use crate::services::ingest::IngestConfig;
use crate::services::chunking::ChunkingConfig;
use crate::services::query::QueryConfig;
use crate::services::llm_client::LlmClient;

pub async fn start_api_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config_file = config::load_config(None)
        .unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load config: {}. Using defaults.", e);
            Default::default()
        });

    let embedding_model_str = config_file.embedding_model
        .as_ref()
        .ok_or("No embedding model configured")?;
    
    let parsed_embedding = config::parse_model_string(embedding_model_str)?;
    let embedding_provider_config = config::get_provider_config(&config_file, &parsed_embedding.provider)?;
    
    let embedding_options = embedding_provider_config.options
        .as_ref()
        .ok_or("No options configured for embedding provider")?;
    
    let embedding_api_key = embedding_options.api_key
        .clone()
        .ok_or("No API key configured for embedding provider")?;
    let embedding_base_url = embedding_options.base_url
        .clone()
        .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
    
    let embedding_dimension = config_file.embedding_dimension
        .ok_or("No embedding dimension configured")?;

    let context_model_str = config_file.context_model
        .as_ref()
        .ok_or("No context model configured")?;
    
    let parsed_context = config::parse_model_string(context_model_str)?;
    let context_provider_config = config::get_provider_config(&config_file, &parsed_context.provider)?;
    
    let context_options = context_provider_config.options
        .as_ref()
        .ok_or("No options configured for context provider")?;
    
    let context_api_key = context_options.api_key
        .clone()
        .ok_or("No API key configured for context provider")?;
    let context_base_url = context_options.base_url
        .clone()
        .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

    let embedding_config = EmbeddingConfig {
        provider: parsed_embedding.provider.clone(),
        model: parsed_embedding.model.clone(),
        api_key: embedding_api_key.clone(),
        base_url: embedding_base_url.clone(),
        dimension: embedding_dimension as usize,
    };

    info!("Embedding config loaded:");
    info!("  Provider: {}", parsed_embedding.provider);
    info!("  Model: {}", parsed_embedding.model);
    info!("  Base URL: {}", embedding_base_url);
    info!("  Dimension: {}", embedding_dimension);
    info!("  API Key: {}...{}", 
        &embedding_api_key[..8.min(embedding_api_key.len())],
        if embedding_api_key.len() > 8 { &embedding_api_key[embedding_api_key.len()-4..] } else { "" }
    );

    let llm_client = LlmClient::new(
        &parsed_context.provider,
        context_api_key,
        Some(context_base_url),
    )?;

    let db_path = config_file.storage
        .as_ref()
        .and_then(|s| s.path.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("./data/lancedb");

    let project_path = std::env::current_dir()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| ".".to_string());

    let chunking_config = if let Some(chunking) = &config_file.chunking {
        ChunkingConfig {
            target_chars: chunking.target_chars.unwrap_or(1000),
            overlap_chars: chunking.overlap_chars.unwrap_or(200),
            max_chars: chunking.max_chars.unwrap_or(2000),
            ..Default::default()
        }
    } else {
        ChunkingConfig::default()
    };

    let ingest_config = IngestConfig {
        chunking: chunking_config,
        embedding_model: parsed_embedding.model.clone(),
        tokenizer_model: "gpt-4".to_string(),
    };

    let state = Arc::new(AppState::new(
        db_path,
        &project_path,
        embedding_config,
        ingest_config,
        QueryConfig::default(),
        llm_client,
    ).await?);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = create_router(state).layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("API server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
