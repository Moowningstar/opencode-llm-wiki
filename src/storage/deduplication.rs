use crate::storage::{HashIndex, RefCounter};
use crate::types::storage::{ContentHash, DeduplicationResult, VectorId};
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

pub struct VectorDeduplicator {
    hash_index: HashIndex,
    ref_counter: RefCounter,
}

impl VectorDeduplicator {
    pub fn new(storage_root: &Path) -> Result<Self> {
        let hash_index_path = storage_root.join(".vectors").join(".hash_index.json");
        let ref_counter_path = storage_root.join(".vectors").join(".ref_counter.db");

        std::fs::create_dir_all(storage_root.join(".vectors"))
            .context("Failed to create vectors directory")?;

        let hash_index = HashIndex::new(hash_index_path.display().to_string())?;
        let ref_counter = RefCounter::new(&ref_counter_path)?;

        Ok(Self {
            hash_index,
            ref_counter,
        })
    }

    pub fn compute_content_hash(content: &str) -> ContentHash {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn check_or_create(
        &mut self,
        content: &str,
        project_id: &str,
        chunk_id: &str,
    ) -> Result<DeduplicationResult> {
        let content_hash = Self::compute_content_hash(content);

        if let Some(entry) = self.hash_index.get(&content_hash) {
            let vector_id = entry.vector_id.clone();
            self.ref_counter.add_ref(&content_hash, project_id, chunk_id, &vector_id)?;
            
            Ok(DeduplicationResult::Reused {
                vector_id,
                content_hash,
            })
        } else {
            let vector_id = uuid::Uuid::new_v4().to_string();
            
            self.hash_index.insert(content_hash.clone(), vector_id.clone())?;
            self.ref_counter.add_ref(&content_hash, project_id, chunk_id, &vector_id)?;
            
            Ok(DeduplicationResult::Created {
                vector_id,
                content_hash,
            })
        }
    }

    pub fn remove_chunk(
        &mut self,
        content_hash: &str,
        project_id: &str,
        chunk_id: &str,
    ) -> Result<bool> {
        let should_delete = self.ref_counter.remove_ref(content_hash, project_id, chunk_id)?;
        
        if should_delete {
            self.hash_index.decrement_ref(content_hash)?;
        }
        
        Ok(should_delete)
    }

    pub fn remove_project(&mut self, project_id: &str) -> Result<Vec<ContentHash>> {
        let to_delete = self.ref_counter.remove_project_refs(project_id)?;
        
        for content_hash in &to_delete {
            self.hash_index.decrement_ref(content_hash)?;
        }
        
        Ok(to_delete)
    }

    pub fn get_ref_count(&self, content_hash: &str) -> Result<u32> {
        self.ref_counter.get_ref_count(content_hash)
    }

    pub fn get_vector_id(&self, content_hash: &str) -> Result<Option<VectorId>> {
        self.ref_counter.get_vector_id(content_hash)
    }

    pub fn get_stats(&self) -> Result<DeduplicationStats> {
        let ref_stats = self.ref_counter.get_stats()?;
        let hash_count = self.hash_index.total_entries();
        let distribution = self.ref_counter.get_ref_distribution()?;

        let reuse_count: usize = distribution
            .iter()
            .filter(|(&ref_count, _)| ref_count > 1)
            .map(|(_, &count)| count)
            .sum();

        let total_vectors = ref_stats.unique_vectors;
        let reuse_rate = if total_vectors > 0 {
            (reuse_count as f64 / total_vectors as f64) * 100.0
        } else {
            0.0
        };

        Ok(DeduplicationStats {
            total_chunks: ref_stats.total_refs,
            unique_vectors: ref_stats.unique_vectors,
            reused_vectors: reuse_count,
            reuse_rate,
            hash_index_size: hash_count,
            ref_distribution: distribution,
        })
    }

    pub fn verify_integrity(&self) -> Result<IntegrityReport> {
        let orphaned_refs = self.ref_counter.verify_integrity()?;
        let hash_count = self.hash_index.total_entries();
        let ref_count_entries = self.ref_counter.get_stats()?.unique_vectors;

        let mismatch = hash_count != ref_count_entries;

        Ok(IntegrityReport {
            orphaned_refs,
            hash_index_count: hash_count,
            ref_counter_count: ref_count_entries,
            has_mismatch: mismatch,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DeduplicationStats {
    pub total_chunks: usize,
    pub unique_vectors: usize,
    pub reused_vectors: usize,
    pub reuse_rate: f64,
    pub hash_index_size: usize,
    pub ref_distribution: std::collections::HashMap<u32, usize>,
}

#[derive(Debug, Clone)]
pub struct IntegrityReport {
    pub orphaned_refs: Vec<String>,
    pub hash_index_count: usize,
    pub ref_counter_count: usize,
    pub has_mismatch: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_deduplication() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut dedup = VectorDeduplicator::new(temp_dir.path())?;

        let content = "This is a test document";
        
        let result1 = dedup.check_or_create(content, "proj1", "chunk1")?;
        assert!(!result1.is_reused());
        let vector_id1 = result1.vector_id().to_string();

        let result2 = dedup.check_or_create(content, "proj2", "chunk2")?;
        assert!(result2.is_reused());
        assert_eq!(result2.vector_id(), vector_id1);

        let stats = dedup.get_stats()?;
        assert_eq!(stats.total_chunks, 2);
        assert_eq!(stats.unique_vectors, 1);
        assert_eq!(stats.reused_vectors, 1);

        Ok(())
    }

    #[test]
    fn test_removal() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut dedup = VectorDeduplicator::new(temp_dir.path())?;

        let content = "Test content";
        
        dedup.check_or_create(content, "proj1", "chunk1")?;
        let result = dedup.check_or_create(content, "proj2", "chunk2")?;
        let content_hash = result.content_hash().to_string();

        let should_delete = dedup.remove_chunk(&content_hash, "proj1", "chunk1")?;
        assert!(!should_delete);
        assert_eq!(dedup.get_ref_count(&content_hash)?, 1);

        let should_delete = dedup.remove_chunk(&content_hash, "proj2", "chunk2")?;
        assert!(should_delete);
        assert_eq!(dedup.get_ref_count(&content_hash)?, 0);

        Ok(())
    }

    #[test]
    fn test_project_removal() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut dedup = VectorDeduplicator::new(temp_dir.path())?;

        dedup.check_or_create("content1", "proj1", "chunk1")?;
        dedup.check_or_create("content2", "proj1", "chunk2")?;
        dedup.check_or_create("content3", "proj2", "chunk3")?;

        let deleted = dedup.remove_project("proj1")?;
        assert_eq!(deleted.len(), 2);

        let stats = dedup.get_stats()?;
        assert_eq!(stats.total_chunks, 1);
        assert_eq!(stats.unique_vectors, 1);

        Ok(())
    }
}
