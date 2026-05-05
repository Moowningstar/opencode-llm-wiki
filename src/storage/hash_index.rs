use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use chrono::Utc;

use crate::types::storage::{ContentHash, VectorId, HashIndexEntry};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HashIndexData {
    entries: HashMap<ContentHash, HashIndexEntry>,
    version: String,
}

pub struct HashIndex {
    data: HashIndexData,
    db_path: String,
}

impl HashIndex {
    pub fn new(db_path: String) -> Result<Self> {
        let data = if Path::new(&db_path).exists() {
            let content = fs::read_to_string(&db_path)?;
            serde_json::from_str(&content)?
        } else {
            HashIndexData {
                entries: HashMap::new(),
                version: "1.0".to_string(),
            }
        };
        
        Ok(Self { data, db_path })
    }
    
    pub fn get(&self, content_hash: &str) -> Option<&HashIndexEntry> {
        self.data.entries.get(content_hash)
    }
    
    pub fn insert(&mut self, content_hash: ContentHash, vector_id: VectorId) -> Result<()> {
        let entry = HashIndexEntry {
            content_hash: content_hash.clone(),
            vector_id,
            ref_count: 1,
            created_at: Utc::now().to_rfc3339(),
        };
        
        self.data.entries.insert(content_hash, entry);
        self.save()?;
        Ok(())
    }
    
    pub fn increment_ref(&mut self, content_hash: &str) -> Result<()> {
        if let Some(entry) = self.data.entries.get_mut(content_hash) {
            entry.ref_count += 1;
            self.save()?;
        }
        Ok(())
    }
    
    pub fn decrement_ref(&mut self, content_hash: &str) -> Result<Option<VectorId>> {
        if let Some(entry) = self.data.entries.get_mut(content_hash) {
            entry.ref_count = entry.ref_count.saturating_sub(1);
            
            if entry.ref_count == 0 {
                let vector_id = entry.vector_id.clone();
                self.data.entries.remove(content_hash);
                self.save()?;
                return Ok(Some(vector_id));
            }
            
            self.save()?;
        }
        Ok(None)
    }
    
    pub fn get_ref_count(&self, content_hash: &str) -> usize {
        self.data.entries
            .get(content_hash)
            .map(|e| e.ref_count)
            .unwrap_or(0)
    }
    
    pub fn total_entries(&self) -> usize {
        self.data.entries.len()
    }
    
    fn save(&self) -> Result<()> {
        if let Some(parent) = Path::new(&self.db_path).parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.db_path, content)?;
        Ok(())
    }
}

pub fn compute_content_hash(content: &str) -> ContentHash {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_hash_index_basic() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join(".hash_index.db").to_str().unwrap().to_string();
        
        let mut index = HashIndex::new(db_path.clone()).unwrap();
        
        let hash = "abc123";
        let vector_id = "vec_001";
        
        index.insert(hash.to_string(), vector_id.to_string()).unwrap();
        
        let entry = index.get(hash).unwrap();
        assert_eq!(entry.vector_id, vector_id);
        assert_eq!(entry.ref_count, 1);
    }
    
    #[test]
    fn test_ref_counting() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join(".hash_index.db").to_str().unwrap().to_string();
        
        let mut index = HashIndex::new(db_path).unwrap();
        
        let hash = "abc123";
        index.insert(hash.to_string(), "vec_001".to_string()).unwrap();
        
        index.increment_ref(hash).unwrap();
        assert_eq!(index.get_ref_count(hash), 2);
        
        let result = index.decrement_ref(hash).unwrap();
        assert!(result.is_none());
        assert_eq!(index.get_ref_count(hash), 1);
        
        let result = index.decrement_ref(hash).unwrap();
        assert_eq!(result, Some("vec_001".to_string()));
        assert_eq!(index.get_ref_count(hash), 0);
    }
    
    #[test]
    fn test_content_hash() {
        let content1 = "Hello, World!";
        let content2 = "Hello, World!";
        let content3 = "Different content";
        
        let hash1 = compute_content_hash(content1);
        let hash2 = compute_content_hash(content2);
        let hash3 = compute_content_hash(content3);
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64);
    }
}
