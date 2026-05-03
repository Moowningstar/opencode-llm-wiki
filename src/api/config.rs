use std::path::{Path, PathBuf};
use std::fs;
use crate::types::api::{LlmConfigFile, ProviderOverride};

pub struct ParsedModel {
    pub provider: String,
    pub model: String,
}

pub fn parse_model_string(model_string: &str) -> Result<ParsedModel, String> {
    let parts: Vec<&str> = model_string.splitn(2, '/').collect();
    
    if parts.len() != 2 {
        return Err(format!(
            "Invalid model format '{}'. Expected 'provider/model-name'",
            model_string
        ));
    }
    
    Ok(ParsedModel {
        provider: parts[0].to_string(),
        model: parts[1].to_string(),
    })
}

pub fn get_provider_config(
    config: &LlmConfigFile,
    provider_name: &str,
) -> Result<ProviderOverride, String> {
    config
        .providers
        .as_ref()
        .and_then(|providers| providers.get(provider_name))
        .cloned()
        .ok_or_else(|| format!("Provider '{}' not found in config", provider_name))
}

pub fn get_default_config_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    
    Path::new(&home).join(".config").join("opencode-llm-wiki")
}

pub fn get_config_path(project_path: Option<&str>) -> PathBuf {
    // Priority: llm-wiki.jsonc > llm-config.jsonc
    // Locations: ~/.config/opencode-llm-wiki/ > ~/.config/opencode/
    
    let candidates = if let Some(path) = project_path {
        vec![
            Path::new(path).join(".llm-wiki").join("llm-wiki.jsonc"),
            Path::new(path).join(".llm-wiki").join("llm-config.jsonc"),
        ]
    } else {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        
        vec![
            Path::new(&home).join(".config").join("opencode-llm-wiki").join("llm-wiki.jsonc"),
            Path::new(&home).join(".config").join("opencode-llm-wiki").join("llm-config.jsonc"),
            Path::new(&home).join(".config").join("opencode").join("llm-wiki.jsonc"),
            Path::new(&home).join(".config").join("opencode").join("llm-config.jsonc"),
        ]
    };
    
    // Return first existing file, or first candidate if none exist
    candidates.iter()
        .find(|p| p.exists())
        .cloned()
        .unwrap_or_else(|| candidates[0].clone())
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

pub fn load_config(project_path: Option<&str>) -> Result<LlmConfigFile, String> {
    let config_path = get_config_path(project_path);

    if !config_path.exists() {
        return Ok(LlmConfigFile {
            context_model: None,
            embedding_model: None,
            embedding_dimension: None,
            providers: None,
            chunking: None,
            storage: None,
        });
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let cleaned = remove_jsonc_comments(&content);
    
    // Debug: print cleaned JSON
    eprintln!("=== CLEANED JSON (first 500 chars) ===");
    eprintln!("{}", &cleaned.chars().take(500).collect::<String>());
    eprintln!("=== END CLEANED JSON ===");

    serde_json::from_str(&cleaned)
        .map_err(|e| format!("Failed to parse config file: {}", e))
}

pub fn save_config(project_path: Option<&str>, config: &LlmConfigFile) -> Result<(), String> {
    let config_path = get_config_path(project_path);
    let config_dir = config_path.parent().unwrap();

    fs::create_dir_all(config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config file: {}", e))
}

fn remove_jsonc_comments(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escape_next = false;
    
    while let Some(ch) = chars.next() {
        if escape_next {
            result.push(ch);
            escape_next = false;
            continue;
        }
        
        if ch == '\\' && in_string {
            result.push(ch);
            escape_next = true;
            continue;
        }
        
        if ch == '"' {
            in_string = !in_string;
            result.push(ch);
            continue;
        }
        
        if !in_string && ch == '/' {
            if let Some(&next_ch) = chars.peek() {
                if next_ch == '/' {
                    chars.next();
                    while let Some(&line_ch) = chars.peek() {
                        if line_ch == '\n' {
                            break;
                        }
                        chars.next();
                    }
                    continue;
                } else if next_ch == '*' {
                    chars.next();
                    let mut found_end = false;
                    while let Some(comment_ch) = chars.next() {
                        if comment_ch == '*' {
                            if let Some(&slash) = chars.peek() {
                                if slash == '/' {
                                    chars.next();
                                    found_end = true;
                                    break;
                                }
                            }
                        }
                    }
                    if found_end {
                        continue;
                    }
                }
            }
        }
        
        result.push(ch);
    }
    
    result
}
