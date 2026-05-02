use std::path::Path;
use std::fs;
use crate::types::api::LlmConfigFile;

pub fn load_config(project_path: &str) -> Result<LlmConfigFile, String> {
    let config_path = Path::new(project_path)
        .join(".llm-wiki")
        .join("llm-config.json");

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
    let config_path = config_dir.join("llm-config.json");

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
