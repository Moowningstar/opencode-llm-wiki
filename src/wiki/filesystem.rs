use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};

pub struct WikiFileSystem {
    root_path: PathBuf,
}

impl WikiFileSystem {
    pub fn new(project_path: &str) -> Result<Self> {
        let root_path = Path::new(project_path).join(".wiki");
        Ok(Self { root_path })
    }

    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(self.pages_dir())?;
        fs::create_dir_all(self.meta_dir())?;
        
        let purpose_path = self.root_path.join("purpose.md");
        if !purpose_path.exists() {
            fs::write(&purpose_path, Self::default_purpose())?;
        }
        
        Ok(())
    }

    pub fn pages_dir(&self) -> PathBuf {
        self.root_path.join("pages")
    }

    pub fn meta_dir(&self) -> PathBuf {
        self.root_path.join("_meta")
    }

    pub fn index_path(&self) -> PathBuf {
        self.meta_dir().join("index.json")
    }

    pub fn graph_path(&self) -> PathBuf {
        self.meta_dir().join("graph.json")
    }

    pub fn purpose_path(&self) -> PathBuf {
        self.root_path.join("purpose.md")
    }

    pub fn write_page(&self, page_id: &str, content: &str) -> Result<()> {
        let normalized_id = page_id.strip_suffix(".md").unwrap_or(page_id);
        let page_path = self.pages_dir().join(format!("{}.md", normalized_id));
        fs::write(&page_path, content)
            .with_context(|| format!("Failed to write page: {}", page_id))?;
        Ok(())
    }

    pub fn read_page(&self, page_id: &str) -> Result<String> {
        let normalized_id = page_id.strip_suffix(".md").unwrap_or(page_id);
        let page_path = self.pages_dir().join(format!("{}.md", normalized_id));
        fs::read_to_string(&page_path)
            .with_context(|| format!("Failed to read page: {}", page_id))
    }

    pub fn delete_page(&self, page_id: &str) -> Result<()> {
        let normalized_id = page_id.strip_suffix(".md").unwrap_or(page_id);
        let page_path = self.pages_dir().join(format!("{}.md", normalized_id));
        if page_path.exists() {
            fs::remove_file(&page_path)
                .with_context(|| format!("Failed to delete page: {}", page_id))?;
        }
        Ok(())
    }

    pub fn list_page_ids(&self) -> Result<Vec<String>> {
        let mut page_ids = Vec::new();
        
        if !self.pages_dir().exists() {
            return Ok(page_ids);
        }

        for entry in fs::read_dir(self.pages_dir())? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    page_ids.push(stem.to_string());
                }
            }
        }
        
        Ok(page_ids)
    }

    pub fn page_exists(&self, page_id: &str) -> bool {
        self.pages_dir().join(format!("{}.md", page_id)).exists()
    }

    pub fn read_purpose(&self) -> Result<String> {
        fs::read_to_string(self.purpose_path())
            .with_context(|| "Failed to read purpose.md")
    }

    fn default_purpose() -> &'static str {
        r#"# Wiki Purpose

This wiki serves as persistent memory for AI assistants working on this project.

## Goals

1. **Context Persistence**: Preserve important information across conversation sessions
2. **Knowledge Reuse**: Avoid re-explaining project architecture and decisions
3. **Semantic Search**: Enable AI to quickly find relevant information

## Structure

- `pages/`: All wiki pages (flat structure)
- `_meta/index.json`: Page metadata and index
- `_meta/graph.json`: Knowledge graph (page relationships)

## Usage

When starting a new conversation, the AI will automatically load this purpose file
and relevant pages based on your query.
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_creates_directories() {
        let temp = TempDir::new().unwrap();
        let fs = WikiFileSystem::new(temp.path().to_str().unwrap()).unwrap();
        
        fs.init().unwrap();
        
        assert!(fs.pages_dir().exists());
        assert!(fs.meta_dir().exists());
        assert!(fs.purpose_path().exists());
    }

    #[test]
    fn test_write_and_read_page() {
        let temp = TempDir::new().unwrap();
        let fs = WikiFileSystem::new(temp.path().to_str().unwrap()).unwrap();
        fs.init().unwrap();
        
        let content = "# Test Page\n\nThis is a test.";
        fs.write_page("test-page", content).unwrap();
        
        let read_content = fs.read_page("test-page").unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_list_page_ids() {
        let temp = TempDir::new().unwrap();
        let fs = WikiFileSystem::new(temp.path().to_str().unwrap()).unwrap();
        fs.init().unwrap();
        
        fs.write_page("page-1", "content 1").unwrap();
        fs.write_page("page-2", "content 2").unwrap();
        
        let ids = fs.list_page_ids().unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"page-1".to_string()));
        assert!(ids.contains(&"page-2".to_string()));
    }
}
