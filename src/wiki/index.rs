use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};

use super::filesystem::WikiFileSystem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiIndex {
    pub version: String,
    pub pages: Vec<PageMetadata>,
    pub metadata: IndexMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMetadata {
    pub id: String,
    pub path: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    pub importance: u8,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub word_count: usize,
    pub links_to: Vec<String>,
    pub linked_from: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    pub total_pages: usize,
    pub last_updated: DateTime<Utc>,
    pub categories: Vec<String>,
    pub top_tags: Vec<String>,
}

pub struct IndexManager {
    fs: Arc<WikiFileSystem>,
}

impl IndexManager {
    pub fn new(fs: Arc<WikiFileSystem>) -> Self {
        Self { fs }
    }

    pub fn load(&self) -> Result<WikiIndex> {
        let index_path = self.fs.index_path();
        
        if !index_path.exists() {
            return Ok(Self::empty_index());
        }

        let content = fs::read_to_string(&index_path)
            .context("Failed to read index.json")?;
        
        serde_json::from_str(&content)
            .context("Failed to parse index.json")
    }

    pub fn save(&self, index: &WikiIndex) -> Result<()> {
        let index_path = self.fs.index_path();
        let content = serde_json::to_string_pretty(index)
            .context("Failed to serialize index")?;
        
        fs::write(&index_path, content)
            .context("Failed to write index.json")?;
        
        Ok(())
    }

    pub fn add_page(&self, page: PageMetadata) -> Result<()> {
        let mut index = self.load()?;
        
        index.pages.retain(|p| p.id != page.id);
        index.pages.push(page);
        self.update_metadata(&mut index);
        
        self.save(&index)?;
        Ok(())
    }

    pub fn remove_page(&self, page_id: &str) -> Result<()> {
        let mut index = self.load()?;
        
        index.pages.retain(|p| p.id != page_id);
        self.update_metadata(&mut index);
        
        self.save(&index)?;
        Ok(())
    }

    pub fn get_page(&self, page_id: &str) -> Result<Option<PageMetadata>> {
        let index = self.load()?;
        Ok(index.pages.into_iter().find(|p| p.id == page_id))
    }

    pub fn list_pages(&self) -> Result<Vec<PageMetadata>> {
        let index = self.load()?;
        Ok(index.pages)
    }

    pub fn search_pages(&self, query: &str) -> Result<Vec<PageMetadata>> {
        let index = self.load()?;
        let query_lower = query.to_lowercase();
        
        let results: Vec<PageMetadata> = index.pages
            .into_iter()
            .filter(|p| {
                p.title.to_lowercase().contains(&query_lower) ||
                p.tags.iter().any(|t| t.to_lowercase().contains(&query_lower)) ||
                p.category.as_ref().map_or(false, |c| c.to_lowercase().contains(&query_lower))
            })
            .collect();
        
        Ok(results)
    }

    pub fn rebuild(&self) -> Result<()> {
        let page_ids = self.fs.list_page_ids()?;
        let mut pages = Vec::new();

        for page_id in page_ids {
            let content = self.fs.read_page(&page_id)?;
            let metadata = self.extract_metadata(&page_id, &content)?;
            pages.push(metadata);
        }

        let mut index = WikiIndex {
            version: "1.0".to_string(),
            pages,
            metadata: IndexMetadata {
                total_pages: 0,
                last_updated: Utc::now(),
                categories: Vec::new(),
                top_tags: Vec::new(),
            },
        };

        self.update_metadata(&mut index);
        self.save(&index)?;

        Ok(())
    }

    pub fn extract_metadata(&self, page_id: &str, content: &str) -> Result<PageMetadata> {
        let title = Self::extract_title(content);
        let word_count = content.split_whitespace().count();
        let links_to = Self::extract_links(content);
        
        Ok(PageMetadata {
            id: page_id.to_string(),
            path: format!("pages/{}.md", page_id),
            title,
            summary: None,
            tags: Vec::new(),
            category: None,
            importance: 5,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            word_count,
            links_to,
            linked_from: Vec::new(),
        })
    }

    fn extract_title(content: &str) -> String {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") {
                return trimmed[2..].trim().to_string();
            }
        }
        "Untitled".to_string()
    }

    fn extract_links(content: &str) -> Vec<String> {
        let mut links = Vec::new();
        
        let wiki_link_pattern = regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
        for cap in wiki_link_pattern.captures_iter(content) {
            if let Some(link) = cap.get(1) {
                links.push(link.as_str().to_string());
            }
        }
        
        let markdown_link_pattern = regex::Regex::new(r"\[([^\]]+)\]\(([^)]+\.md)\)").unwrap();
        for cap in markdown_link_pattern.captures_iter(content) {
            if let Some(link) = cap.get(2) {
                let link_str = link.as_str();
                if let Some(page_id) = link_str.strip_suffix(".md") {
                    links.push(page_id.to_string());
                }
            }
        }
        
        links
    }

    fn update_metadata(&self, index: &mut WikiIndex) {
        index.metadata.total_pages = index.pages.len();
        index.metadata.last_updated = Utc::now();
        
        let mut categories: Vec<String> = index.pages
            .iter()
            .filter_map(|p| p.category.clone())
            .collect();
        categories.sort();
        categories.dedup();
        index.metadata.categories = categories;
        
        let mut tag_counts: HashMap<String, usize> = HashMap::new();
        for page in &index.pages {
            for tag in &page.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        
        let mut tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
        tags.sort_by(|a, b| b.1.cmp(&a.1));
        index.metadata.top_tags = tags.into_iter().take(20).map(|(tag, _)| tag).collect();
        
        let mut linked_from_map: HashMap<String, Vec<String>> = HashMap::new();
        for page in &index.pages {
            for link in &page.links_to {
                linked_from_map.entry(link.clone())
                    .or_insert_with(Vec::new)
                    .push(page.id.clone());
            }
        }
        
        for page in &mut index.pages {
            page.linked_from = linked_from_map.get(&page.id).cloned().unwrap_or_default();
        }
    }

    fn empty_index() -> WikiIndex {
        WikiIndex {
            version: "1.0".to_string(),
            pages: Vec::new(),
            metadata: IndexMetadata {
                total_pages: 0,
                last_updated: Utc::now(),
                categories: Vec::new(),
                top_tags: Vec::new(),
            },
        }
    }
}
