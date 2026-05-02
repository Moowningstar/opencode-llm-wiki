use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use super::{handlers, state::AppState};

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/api/llm/stream", post(handlers::stream_chat))
        .route("/api/ingest", post(handlers::ingest_file))
        .route("/api/config/get", post(handlers::get_config))
        .route("/api/config/save", post(handlers::save_config))
        .with_state(state)
}
