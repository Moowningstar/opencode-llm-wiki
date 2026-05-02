use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::sse::{Event, Sse},
};
use futures::stream::{self, Stream};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use crate::api::state::AppState;
use crate::types::api::*;
use super::config;

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub async fn stream_chat(
    State(_state): State<Arc<AppState>>,
    Json(_payload): Json<StreamChatRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, Json<ErrorResponse>)> {
    let stream = stream::iter(vec![
        Ok(Event::default().data(serde_json::to_string(&StreamChatResponse {
            token: Some("Placeholder".to_string()),
            done: false,
            error: None,
        }).unwrap())),
        Ok(Event::default().data(serde_json::to_string(&StreamChatResponse {
            token: None,
            done: true,
            error: None,
        }).unwrap())),
    ]);

    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive"),
    ))
}

pub async fn ingest_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<IngestRequest>,
) -> Result<Json<IngestResponse>, (StatusCode, String)> {
    match state.ingest_service.ingest_file_blocks(&req.content).await {
        Ok(result) => Ok(Json(IngestResponse {
            success: true,
            pages_processed: result.pages_processed,
            chunks_created: result.chunks_created,
            error: None,
        })),
        Err(e) => Ok(Json(IngestResponse {
            success: false,
            pages_processed: 0,
            chunks_created: 0,
            error: Some(e.to_string()),
        })),
    }
}

pub async fn get_config(
    Json(payload): Json<ConfigRequest>,
) -> Result<Json<ConfigResponse>, (StatusCode, Json<ErrorResponse>)> {
    match config::load_config(&payload.project_path) {
        Ok(config_file) => {
            Ok(Json(ConfigResponse {
                config: Some(config_file),
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(ConfigResponse {
                config: None,
                error: Some(e),
            }))
        }
    }
}

pub async fn save_config(
    Json(payload): Json<SaveConfigRequest>,
) -> Result<Json<SaveConfigResponse>, (StatusCode, Json<ErrorResponse>)> {
    match config::save_config(&payload.project_path, &payload.config) {
        Ok(_) => {
            Ok(Json(SaveConfigResponse {
                success: true,
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(SaveConfigResponse {
                success: false,
                error: Some(e),
            }))
        }
    }
}
