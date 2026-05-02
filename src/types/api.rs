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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfigFile {
    #[serde(skip_serializing_if = "Option::is_none", rename = "activePreset")]
    pub active_preset: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub providers: Option<HashMap<String, ProviderOverride>>,
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
