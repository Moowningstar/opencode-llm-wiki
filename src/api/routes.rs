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
        .route("/api/config/init", post(handlers::init_config))
        .route("/api/pages", post(handlers::list_pages))
        .route("/api/pages/read", post(handlers::read_page))
        .route("/api/search/keyword", post(handlers::keyword_search))
        .route("/api/search/semantic", post(handlers::semantic_search))
        .route("/api/graph", post(handlers::get_graph))
        .route("/api/graph/insights", post(handlers::graph_insights))
        .route("/api/research", post(handlers::deep_research))
        .route("/api/meta/index", post(handlers::get_index))
        .route("/api/meta/overview", post(handlers::get_overview))
        .route("/api/meta/purpose", post(handlers::get_purpose))
        .with_state(state)
}
