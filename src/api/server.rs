use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tracing::info;

use super::{routes::create_router, state::AppState};

pub async fn start_api_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let api_key = std::env::var("OPENAI_API_KEY")
        .unwrap_or_else(|_| "sk-placeholder".to_string());
    
    let state = Arc::new(AppState::with_defaults("./data", api_key).await?);

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
