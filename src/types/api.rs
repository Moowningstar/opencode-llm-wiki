use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderOverride {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ProviderOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderOptions {
    #[serde(skip_serializing_if = "Option::is_none", rename = "apiKey")]
    pub api_key: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none", rename = "baseURL")]
    pub base_url: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none", rename = "apiMode")]
    pub api_mode: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxContextSize")]
    pub max_context_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmConfigFile {
    #[serde(skip_serializing_if = "Option::is_none", rename = "contextModel")]
    pub context_model: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none", rename = "embeddingModel")]
    pub embedding_model: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none", rename = "embeddingDimension")]
    pub embedding_dimension: Option<usize>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub providers: Option<HashMap<String, ProviderOverride>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunking: Option<ChunkingOptions>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<StorageOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingOptions {
    #[serde(skip_serializing_if = "Option::is_none", rename = "targetChars")]
    pub target_chars: Option<usize>,
    
    #[serde(skip_serializing_if = "Option::is_none", rename = "overlapChars")]
    pub overlap_chars: Option<usize>,
    
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxChars")]
    pub max_chars: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageOptions {
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub storage_type: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamChatRequest {
    pub config: LlmConfig,
    pub messages: Vec<ChatMessage>,
    pub model: String,
    pub stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamChatResponse {
    pub token: Option<String>,
    pub done: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IngestRequest {
    /// Path to a file or directory to ingest
    pub path: String,
    /// Optional: specific file extensions to include (e.g., ["md", "txt"])
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,
    /// Optional: whether to recursively scan directories
    #[serde(default = "default_recursive")]
    pub recursive: bool,
}

fn default_recursive() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IngestResponse {
    pub success: bool,
    pub pages_processed: usize,
    pub chunks_created: usize,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigRequest {
    pub project_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub config: Option<LlmConfigFile>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveConfigRequest {
    pub project_path: String,
    pub config: LlmConfigFile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveConfigResponse {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitConfigRequest {
    /// Optional: custom project path. If None, uses default user config directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitConfigResponse {
    pub success: bool,
    pub config_path: String,
    pub error: Option<String>,
}

// ============================================================================
// Wiki API Types
// ============================================================================

/// Request to list all wiki pages
#[derive(Debug, Serialize, Deserialize)]
pub struct ListPagesRequest {
    /// Optional: filter by scope (global, project:name, all)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Optional: project root path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

/// Response containing list of wiki pages
#[derive(Debug, Serialize, Deserialize)]
pub struct ListPagesResponse {
    pub pages: Vec<WikiPageInfo>,
    pub total: usize,
}

/// Information about a single wiki page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPageInfo {
    pub path: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
}

/// Request to read a single wiki page
#[derive(Debug, Serialize, Deserialize)]
pub struct ReadPageRequest {
    /// Page path relative to .wiki/ directory
    pub path: String,
    /// Optional: scope (global, project:name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Optional: project root path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

/// Response containing page content
#[derive(Debug, Serialize, Deserialize)]
pub struct ReadPageResponse {
    pub path: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<PageMetadata>,
}

/// Metadata for a wiki page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<String>>,
}

/// Request for keyword search
#[derive(Debug, Serialize, Deserialize)]
pub struct KeywordSearchRequest {
    /// Search query string
    pub query: String,
    /// Optional: scope filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Optional: project path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

/// Response for keyword search
#[derive(Debug, Serialize, Deserialize)]
pub struct KeywordSearchResponse {
    pub results: Vec<SearchMatch>,
    pub total: usize,
}

/// A single search match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    pub page_path: String,
    pub title: String,
    pub excerpt: String,
    pub score: f32,
}

/// Request for semantic (vector) search
#[derive(Debug, Serialize, Deserialize)]
pub struct SemanticSearchRequest {
    /// Search query text
    pub query: String,
    /// Maximum number of results
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    /// Minimum similarity score (0.0 - 1.0)
    #[serde(default)]
    pub min_score: f32,
    /// Optional: scope filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Optional: project path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

fn default_top_k() -> usize {
    10
}

/// Response for semantic search
#[derive(Debug, Serialize, Deserialize)]
pub struct SemanticSearchResponse {
    pub results: Vec<SemanticMatch>,
    pub total: usize,
}

/// A single semantic search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMatch {
    pub page_id: String,
    pub chunk_index: u32,
    pub chunk_text: String,
    pub heading_path: String,
    pub score: f32,
}

/// Request to get knowledge graph data
#[derive(Debug, Serialize, Deserialize)]
pub struct GetGraphRequest {
    /// Optional: scope filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Optional: project path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

/// Response containing graph data
#[derive(Debug, Serialize, Deserialize)]
pub struct GetGraphResponse {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

/// A node in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_path: Option<String>,
}

/// An edge in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
}

/// Request for graph insights analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct GraphInsightsRequest {
    /// Type of analysis: isolated, surprising, bridges, stats, all
    #[serde(default = "default_analysis_type")]
    pub analysis_type: String,
    /// Optional: scope filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Optional: project path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

fn default_analysis_type() -> String {
    "all".to_string()
}

/// Response containing graph insights
#[derive(Debug, Serialize, Deserialize)]
pub struct GraphInsightsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isolated_pages: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bridge_nodes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surprising_connections: Option<Vec<(String, String)>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<GraphStats>,
}

/// Graph statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub avg_degree: f32,
}

/// Request for deep research
#[derive(Debug, Serialize, Deserialize)]
pub struct DeepResearchRequest {
    /// Research query or topic
    pub query: String,
    /// Maximum graph traversal depth
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
    /// Maximum number of pages to include
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    /// Optional: scope filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Optional: project path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

fn default_max_depth() -> usize {
    3
}

fn default_max_results() -> usize {
    10
}

/// Response for deep research
#[derive(Debug, Serialize, Deserialize)]
pub struct DeepResearchResponse {
    pub summary: String,
    pub pages: Vec<String>,
    pub connections: Vec<(String, String)>,
}

/// Request to get index.md
#[derive(Debug, Serialize, Deserialize)]
pub struct GetIndexRequest {
    /// Optional: scope filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Optional: project path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

/// Response containing index content
#[derive(Debug, Serialize, Deserialize)]
pub struct GetIndexResponse {
    pub content: String,
}

/// Request to get overview.md
#[derive(Debug, Serialize, Deserialize)]
pub struct GetOverviewRequest {
    /// Optional: scope filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Optional: project path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

/// Response containing overview content
#[derive(Debug, Serialize, Deserialize)]
pub struct GetOverviewResponse {
    pub content: String,
}

/// Request to get purpose.md
#[derive(Debug, Serialize, Deserialize)]
pub struct GetPurposeRequest {
    /// Optional: scope filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Optional: project path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

/// Response containing purpose content
#[derive(Debug, Serialize, Deserialize)]
pub struct GetPurposeResponse {
    pub content: String,
}
