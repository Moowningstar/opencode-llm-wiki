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
    eprintln!("[HANDLER] ingest_file called with path: {}", req.path);
    match state.ingest_service.ingest_from_path(&req.path, req.extensions.clone(), req.recursive).await {
        Ok(result) => {
            eprintln!("[HANDLER] Ingest succeeded, calling sync_wiki_after_ingest...");
            if let Err(e) = sync_wiki_after_ingest(&state, &req.path, req.extensions.clone(), req.recursive) {
                eprintln!("[HANDLER] sync_wiki_after_ingest FAILED: {}", e);
                return Ok(Json(IngestResponse {
                    success: false,
                    pages_processed: result.pages_processed,
                    chunks_created: result.chunks_created,
                    error: Some(format!("Ingest succeeded but wiki sync failed: {}", e)),
                }));
            }
            
            eprintln!("[HANDLER] sync_wiki_after_ingest completed successfully");
            Ok(Json(IngestResponse {
                success: true,
                pages_processed: result.pages_processed,
                chunks_created: result.chunks_created,
                error: if result.warnings.is_empty() {
                    None
                } else {
                    Some(result.warnings.join("; "))
                },
            }))
        }
        Err(e) => {
            eprintln!("[HANDLER] Ingest FAILED: {}", e);
            Ok(Json(IngestResponse {
                success: false,
                pages_processed: 0,
                chunks_created: 0,
                error: Some(e.to_string()),
            }))
        }
    }
}

fn sync_wiki_after_ingest(
    state: &AppState,
    path: &str,
    extensions: Option<Vec<String>>,
    recursive: bool,
) -> anyhow::Result<()> {
    use std::path::Path;
    use walkdir::WalkDir;

    eprintln!("[SYNC] Starting wiki sync for path: {}", path);
    
    let path = Path::new(path);
    let allowed_exts = extensions.unwrap_or_else(|| vec!["md".to_string(), "txt".to_string()]);

    if path.is_file() {
        eprintln!("[SYNC] Processing single file: {:?}", path);
        let content = std::fs::read_to_string(path)?;
        let filename = path.file_name().unwrap().to_string_lossy().replace('/', "-").replace('\\', "-");
        let page_id = format!(".wiki-{}", filename);
        
        eprintln!("[SYNC] Writing page: {}", page_id);
        state.wiki_fs.write_page(&page_id, &content)?;
        eprintln!("[SYNC] Extracting metadata for: {}", page_id);
        let metadata = state.index_manager.extract_metadata(&page_id, &content)?;
        eprintln!("[SYNC] Adding page to index: {}", page_id);
        state.index_manager.add_page(metadata)?;
    } else if path.is_dir() {
        eprintln!("[SYNC] Processing directory: {:?}, recursive: {}", path, recursive);
        let walker = if recursive {
            WalkDir::new(path)
        } else {
            WalkDir::new(path).max_depth(1)
        };

        let mut file_count = 0;
        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }

            let file_path = entry.path();
            if let Some(ext) = file_path.extension() {
                if !allowed_exts.iter().any(|e| e == ext.to_string_lossy().as_ref()) {
                    continue;
                }
            }

            file_count += 1;
            eprintln!("[SYNC] Processing file {}: {:?}", file_count, file_path);
            
            let content = std::fs::read_to_string(file_path)?;
            let relative_path = file_path.strip_prefix(path)
                .unwrap_or(file_path)
                .to_string_lossy()
                .replace('\\', "-")
                .replace('/', "-");
            let page_id = format!(".wiki-{}", relative_path);

            eprintln!("[SYNC] Writing page: {}", page_id);
            state.wiki_fs.write_page(&page_id, &content)?;
            eprintln!("[SYNC] Extracting metadata for: {}", page_id);
            let metadata = state.index_manager.extract_metadata(&page_id, &content)?;
            eprintln!("[SYNC] Adding page to index: {}", page_id);
            state.index_manager.add_page(metadata)?;
        }
        eprintln!("[SYNC] Processed {} files total", file_count);
    }

    eprintln!("[SYNC] Rebuilding graph...");
    state.graph_manager.rebuild(&state.index_manager)?;
    eprintln!("[SYNC] Wiki sync completed successfully");
    Ok(())
}

pub async fn get_config(
    Json(payload): Json<ConfigRequest>,
) -> Result<Json<ConfigResponse>, (StatusCode, Json<ErrorResponse>)> {
    let project_path = if payload.project_path.is_empty() {
        None
    } else {
        Some(payload.project_path.as_str())
    };
    
    match config::load_config(project_path) {
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
    let project_path = if payload.project_path.is_empty() {
        None
    } else {
        Some(payload.project_path.as_str())
    };
    
    match config::save_config(project_path, &payload.config) {
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

pub async fn init_config(
    Json(payload): Json<InitConfigRequest>,
) -> Result<Json<InitConfigResponse>, (StatusCode, Json<ErrorResponse>)> {
    match config::init_config(payload.project_path.as_deref()) {
        Ok(config_path) => {
            Ok(Json(InitConfigResponse {
                success: true,
                config_path: config_path.to_string_lossy().to_string(),
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(InitConfigResponse {
                success: false,
                config_path: String::new(),
                error: Some(e),
            }))
        }
    }
}

pub async fn list_pages(
    State(state): State<Arc<AppState>>,
    Json(_req): Json<ListPagesRequest>,
) -> Result<Json<ListPagesResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.index_manager.list_pages() {
        Ok(pages) => {
            let wiki_pages: Vec<WikiPageInfo> = pages.iter().map(|p| WikiPageInfo {
                path: p.path.clone(),
                title: p.title.clone(),
                size: None,
                modified: Some(p.updated_at.to_rfc3339()),
            }).collect();
            let total = wiki_pages.len();
            Ok(Json(ListPagesResponse { pages: wiki_pages, total }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn read_page(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ReadPageRequest>,
) -> Result<Json<ReadPageResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Extract page_id from path (e.g., "pages/my-page.md" -> "my-page")
    let page_id = req.path
        .trim_start_matches("pages/")
        .trim_end_matches(".md");
    
    match state.wiki_fs.read_page(page_id) {
        Ok(content) => Ok(Json(ReadPageResponse { 
            path: req.path.clone(),
            content,
            metadata: None,
        })),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn keyword_search(
    State(state): State<Arc<AppState>>,
    Json(req): Json<KeywordSearchRequest>,
) -> Result<Json<KeywordSearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.index_manager.search_pages(&req.query) {
        Ok(pages) => {
            let mut results = Vec::new();
            
            for page in pages {
                if let Ok(content) = state.wiki_fs.read_page(&page.id) {
                    let excerpt: String = content.chars().take(200).collect();
                    results.push(SearchMatch {
                        page_path: page.path.clone(),
                        title: page.title.clone(),
                        excerpt,
                        score: 1.0,
                    });
                }
            }
            
            let total = results.len();
            Ok(Json(KeywordSearchResponse { results, total }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn semantic_search(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SemanticSearchRequest>,
) -> Result<Json<SemanticSearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.query_service.query(&req.query).await {
        Ok(query_result) => {
            let matches: Vec<SemanticMatch> = query_result.results.iter()
                .filter(|r| r.score >= req.min_score)
                .take(req.top_k)
                .map(|r| SemanticMatch {
                    page_id: r.page_id.clone(),
                    chunk_index: r.chunk_index,
                    chunk_text: r.chunk_text.clone(),
                    heading_path: r.heading_path.clone(),
                    score: r.score,
                })
                .collect();
            
            Ok(Json(SemanticSearchResponse {
                total: matches.len(),
                results: matches,
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn get_graph(
    State(state): State<Arc<AppState>>,
) -> Result<Json<GetGraphResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.index_manager.load() {
        Ok(index) => {
            let graph = state.graph_manager.generate_from_index(&index);
            
            let nodes = graph.nodes.into_iter().map(|n| crate::types::api::GraphNode {
                id: n.id.clone(),
                label: n.id.clone(),
                page_path: Some(format!("pages/{}.md", n.id)),
            }).collect();
            
            let edges = graph.edges.into_iter().map(|e| crate::types::api::GraphEdge {
                source: e.from,
                target: e.to,
                relation: e.edge_type,
            }).collect();
            
            Ok(Json(GetGraphResponse { nodes, edges }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

pub async fn graph_insights(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<GraphInsightsRequest>,
) -> Result<Json<GraphInsightsResponse>, (StatusCode, Json<ErrorResponse>)> {
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            error: "Not implemented yet".to_string(),
        }),
    ))
}

pub async fn deep_research(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<DeepResearchRequest>,
) -> Result<Json<DeepResearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            error: "Not implemented yet".to_string(),
        }),
    ))
}

pub async fn get_index(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<GetIndexRequest>,
) -> Result<Json<GetIndexResponse>, (StatusCode, Json<ErrorResponse>)> {
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            error: "Not implemented yet".to_string(),
        }),
    ))
}

pub async fn get_overview(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<GetOverviewRequest>,
) -> Result<Json<GetOverviewResponse>, (StatusCode, Json<ErrorResponse>)> {
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            error: "Not implemented yet".to_string(),
        }),
    ))
}

pub async fn get_purpose(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<GetPurposeRequest>,
) -> Result<Json<GetPurposeResponse>, (StatusCode, Json<ErrorResponse>)> {
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            error: "Not implemented yet".to_string(),
        }),
    ))
}
