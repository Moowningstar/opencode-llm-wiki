use std::path::{Path, PathBuf};
use std::fs;
use crate::types::api::LlmConfigFile;

pub fn get_default_config_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    
    Path::new(&home).join(".config").join("opencode-llm-wiki")
}

pub fn get_config_path(project_path: Option<&str>) -> PathBuf {
    match project_path {
        Some(path) => Path::new(path).join(".llm-wiki").join("llm-config.jsonc"),
        None => get_default_config_dir().join("llm-config.jsonc"),
    }
}

pub fn init_config(project_path: Option<&str>) -> Result<PathBuf, String> {
    let config_path = get_config_path(project_path);
    let config_dir = config_path.parent().unwrap();
    
    fs::create_dir_all(config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;
    
    if config_path.exists() {
        return Ok(config_path);
    }
    
    let default_config = r#"{
  // Active LLM provider preset
  "activePreset": "openai",
  
  // Provider configurations
  "providers": {
    "openai": {
      "options": {
        "apiKey": "your-openai-api-key-here",
        "model": "gpt-4",
        "baseURL": "https://api.openai.com/v1",
        "maxContextSize": 128000
      }
    },
    "anthropic": {
      "options": {
        "apiKey": "your-anthropic-api-key-here",
        "model": "claude-3-5-sonnet-20241022",
        "baseURL": "https://api.anthropic.com",
        "maxContextSize": 200000
      }
    },
    "ollama": {
      "options": {
        "model": "qwen2.5:14b",
        "baseURL": "http://localhost:11434",
        "maxContextSize": 32768
      }
    },
    "custom": {
      "options": {
        "apiKey": "your-api-key-here",
        "model": "your-model-name",
        "baseURL": "https://your-api-endpoint.com/v1",
        "maxContextSize": 8192
      }
    }
  },
  
  // Embedding configuration
  "embedding": {
    "provider": "openai",
    "model": "text-embedding-3-small",
    "apiKey": "your-openai-api-key-here",
    "baseURL": "https://api.openai.com/v1"
  },
  
  // Chunking configuration
  "chunking": {
    "targetChars": 1000,
    "overlapChars": 200,
    "maxChars": 2000
  },
  
  // Storage configuration
  "storage": {
    "type": "lancedb",
    "path": "./data/lancedb"
  }
}
"#;
    
    fs::write(&config_path, default_config)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    
    Ok(config_path)
}

pub fn load_config(project_path: &str) -> Result<LlmConfigFile, String> {
    let config_path = Path::new(project_path)
        .join(".llm-wiki")
        .join("llm-config.jsonc");

    if !config_path.exists() {
        return Ok(LlmConfigFile {
            active_preset: None,
            providers: None,
        });
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let cleaned = remove_jsonc_comments(&content);

    serde_json::from_str(&cleaned)
        .map_err(|e| format!("Failed to parse config file: {}", e))
}

pub fn save_config(project_path: &str, config: &LlmConfigFile) -> Result<(), String> {
    let config_dir = Path::new(project_path).join(".llm-wiki");
    let config_path = config_dir.join("llm-config.jsonc");

    fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config file: {}", e))
}

fn remove_jsonc_comments(input: &str) -> String {
    let re_single = regex::Regex::new(r"//.*").unwrap();
    let without_single = re_single.replace_all(input, "");
    
    let re_multi = regex::Regex::new(r"/\*[\s\S]*?\*/").unwrap();
    let without_multi = re_multi.replace_all(&without_single, "");
    
    without_multi.to_string()
}
