use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::Path;

/// Reference counter for tracking vector usage across projects
/// 
/// Schema:
/// - refs: (content_hash TEXT, project_id TEXT, chunk_id TEXT, PRIMARY KEY (content_hash, project_id, chunk_id))
/// - ref_counts: (content_hash TEXT PRIMARY KEY, ref_count INTEGER)
pub struct RefCounter {
    conn: Connection,
}

impl RefCounter {
    /// Create or open reference counter database
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)
            .context("Failed to open reference counter database")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS refs (
                content_hash TEXT NOT NULL,
                project_id TEXT NOT NULL,
                chunk_id TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (content_hash, project_id, chunk_id)
            )",
            [],
        )
        .context("Failed to create refs table")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS ref_counts (
                content_hash TEXT PRIMARY KEY,
                ref_count INTEGER NOT NULL,
                vector_id TEXT NOT NULL
            )",
            [],
        )
        .context("Failed to create ref_counts table")?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_refs_project ON refs(project_id)",
            [],
        )
        .context("Failed to create project index")?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_refs_hash ON refs(content_hash)",
            [],
        )
        .context("Failed to create hash index")?;

        Ok(Self { conn })
    }

    /// Add a reference from a project chunk to a vector
    pub fn add_ref(
        &mut self,
        content_hash: &str,
        project_id: &str,
        chunk_id: &str,
        vector_id: &str,
    ) -> Result<()> {
        let tx = self.conn.transaction()?;

        // Insert reference
        tx.execute(
            "INSERT OR IGNORE INTO refs (content_hash, project_id, chunk_id, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                content_hash,
                project_id,
                chunk_id,
                chrono::Utc::now().timestamp()
            ],
        )
        .context("Failed to insert reference")?;

        // Update reference count
        tx.execute(
            "INSERT INTO ref_counts (content_hash, ref_count, vector_id)
             VALUES (?1, 1, ?2)
             ON CONFLICT(content_hash) DO UPDATE SET ref_count = ref_count + 1",
            params![content_hash, vector_id],
        )
        .context("Failed to update reference count")?;

        tx.commit()?;
        Ok(())
    }

    /// Remove a reference from a project chunk
    /// Returns true if the vector should be deleted (ref_count reached 0)
    pub fn remove_ref(
        &mut self,
        content_hash: &str,
        project_id: &str,
        chunk_id: &str,
    ) -> Result<bool> {
        let tx = self.conn.transaction()?;

        // Delete reference
        let deleted = tx.execute(
            "DELETE FROM refs WHERE content_hash = ?1 AND project_id = ?2 AND chunk_id = ?3",
            params![content_hash, project_id, chunk_id],
        )?;

        if deleted == 0 {
            tx.commit()?;
            return Ok(false);
        }

        // Decrement reference count
        tx.execute(
            "UPDATE ref_counts SET ref_count = ref_count - 1 WHERE content_hash = ?1",
            params![content_hash],
        )?;

        // Check if ref_count reached 0
        let ref_count: i64 = tx.query_row(
            "SELECT ref_count FROM ref_counts WHERE content_hash = ?1",
            params![content_hash],
            |row| row.get(0),
        )?;

        let should_delete = ref_count <= 0;

        if should_delete {
            // Delete from ref_counts
            tx.execute(
                "DELETE FROM ref_counts WHERE content_hash = ?1",
                params![content_hash],
            )?;
        }

        tx.commit()?;
        Ok(should_delete)
    }

    /// Get reference count for a content hash
    pub fn get_ref_count(&self, content_hash: &str) -> Result<u32> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT ref_count FROM ref_counts WHERE content_hash = ?1",
                params![content_hash],
                |row| row.get(0),
            )
            .unwrap_or(0);

        Ok(count as u32)
    }

    /// Get vector ID for a content hash
    pub fn get_vector_id(&self, content_hash: &str) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT vector_id FROM ref_counts WHERE content_hash = ?1",
            params![content_hash],
            |row| row.get(0),
        );

        match result {
            Ok(id) => Ok(Some(id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get all references for a project
    pub fn get_project_refs(&self, project_id: &str) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT content_hash, chunk_id FROM refs WHERE project_id = ?1",
        )?;

        let refs = stmt
            .query_map(params![project_id], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(refs)
    }

    /// Remove all references for a project
    /// Returns list of content hashes that should be deleted (ref_count reached 0)
    pub fn remove_project_refs(&mut self, project_id: &str) -> Result<Vec<String>> {
        let refs = self.get_project_refs(project_id)?;
        let mut to_delete = Vec::new();

        for (content_hash, chunk_id) in refs {
            if self.remove_ref(&content_hash, project_id, &chunk_id)? {
                to_delete.push(content_hash);
            }
        }

        Ok(to_delete)
    }

    /// Get statistics
    pub fn get_stats(&self) -> Result<RefCounterStats> {
        let total_refs: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM refs",
            [],
            |row| row.get(0),
        )?;

        let unique_vectors: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM ref_counts",
            [],
            |row| row.get(0),
        )?;

        let project_count: i64 = self.conn.query_row(
            "SELECT COUNT(DISTINCT project_id) FROM refs",
            [],
            |row| row.get(0),
        )?;

        Ok(RefCounterStats {
            total_refs: total_refs as usize,
            unique_vectors: unique_vectors as usize,
            project_count: project_count as usize,
        })
    }

    /// Get reference distribution (how many vectors have N references)
    pub fn get_ref_distribution(&self) -> Result<HashMap<u32, usize>> {
        let mut stmt = self.conn.prepare(
            "SELECT ref_count, COUNT(*) FROM ref_counts GROUP BY ref_count",
        )?;

        let distribution = stmt
            .query_map([], |row| {
                let ref_count: i64 = row.get(0)?;
                let count: i64 = row.get(1)?;
                Ok((ref_count as u32, count as usize))
            })?
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(distribution)
    }

    /// Verify integrity (check for orphaned refs)
    pub fn verify_integrity(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT content_hash FROM refs 
             WHERE content_hash NOT IN (SELECT content_hash FROM ref_counts)",
        )?;

        let orphaned = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(orphaned)
    }
}

#[derive(Debug, Clone)]
pub struct RefCounterStats {
    pub total_refs: usize,
    pub unique_vectors: usize,
    pub project_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_ref_counter() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut counter = RefCounter::new(temp_file.path())?;

        // Add first reference
        counter.add_ref("hash1", "proj1", "chunk1", "vec1")?;
        assert_eq!(counter.get_ref_count("hash1")?, 1);
        assert_eq!(counter.get_vector_id("hash1")?, Some("vec1".to_string()));

        // Add second reference to same vector
        counter.add_ref("hash1", "proj2", "chunk2", "vec1")?;
        assert_eq!(counter.get_ref_count("hash1")?, 2);

        // Remove first reference
        let should_delete = counter.remove_ref("hash1", "proj1", "chunk1")?;
        assert!(!should_delete);
        assert_eq!(counter.get_ref_count("hash1")?, 1);

        // Remove second reference
        let should_delete = counter.remove_ref("hash1", "proj2", "chunk2")?;
        assert!(should_delete);
        assert_eq!(counter.get_ref_count("hash1")?, 0);

        Ok(())
    }

    #[test]
    fn test_project_refs() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut counter = RefCounter::new(temp_file.path())?;

        counter.add_ref("hash1", "proj1", "chunk1", "vec1")?;
        counter.add_ref("hash2", "proj1", "chunk2", "vec2")?;
        counter.add_ref("hash3", "proj2", "chunk3", "vec3")?;

        let proj1_refs = counter.get_project_refs("proj1")?;
        assert_eq!(proj1_refs.len(), 2);

        let to_delete = counter.remove_project_refs("proj1")?;
        assert_eq!(to_delete.len(), 2);
        assert!(to_delete.contains(&"hash1".to_string()));
        assert!(to_delete.contains(&"hash2".to_string()));

        Ok(())
    }
}
