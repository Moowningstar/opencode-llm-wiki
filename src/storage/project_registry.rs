use crate::types::storage::{ProjectId, ProjectInfo, ProjectStats};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Project identifier (for registration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectIdentifier {
    pub id: ProjectId,
    pub name: String,
}

/// Manages project registration and metadata
pub struct ProjectRegistry {
    registry_path: PathBuf,
    projects: HashMap<ProjectId, ProjectInfo>,
}

impl ProjectRegistry {
    /// Create or load project registry
    pub fn new(global_storage_root: &Path) -> Result<Self> {
        let registry_path = global_storage_root.join(".projects").join("registry.json");
        
        let projects = if registry_path.exists() {
            let content = fs::read_to_string(&registry_path)
                .context("Failed to read project registry")?;
            serde_json::from_str(&content)
                .context("Failed to parse project registry")?
        } else {
            HashMap::new()
        };

        Ok(Self {
            registry_path,
            projects,
        })
    }

    /// Register a new project or update existing
    pub fn register_project(
        &mut self,
        identifier: &ProjectIdentifier,
        project_path: &Path,
    ) -> Result<ProjectInfo> {
        let now = chrono::Utc::now().to_rfc3339();
        
        let info = self.projects
            .entry(identifier.id.clone())
            .and_modify(|m| {
                m.last_accessed = now.clone();
                m.path = project_path.display().to_string();
            })
            .or_insert_with(|| ProjectInfo {
                name: identifier.name.clone(),
                path: project_path.display().to_string(),
                created_at: now.clone(),
                last_accessed: now,
                stats: ProjectStats::default(),
            })
            .clone();

        self.save()?;
        Ok(info)
    }

    /// Get project metadata by ID
    pub fn get_project(&self, project_id: &str) -> Result<Option<ProjectInfo>> {
        Ok(self.projects.get(project_id).cloned())
    }

    /// Get project by path (reverse lookup)
    pub fn get_project_by_path(&self, path: &Path) -> Option<&ProjectInfo> {
        let path_str = path.display().to_string();
        self.projects.values().find(|m| m.path == path_str)
    }

    /// List all registered projects
    pub fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
        let mut projects: Vec<_> = self.projects.values().cloned().collect();
        projects.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
        Ok(projects)
    }

    /// Update project statistics
    pub fn update_stats(
        &mut self,
        project_id: &str,
        stats: ProjectStats,
    ) -> Result<()> {
        if let Some(info) = self.projects.get_mut(project_id) {
            info.stats = stats;
            info.last_accessed = chrono::Utc::now().to_rfc3339();
            self.save()?;
        }
        Ok(())
    }

    /// Remove project from registry (does not delete data)
    pub fn unregister_project(&mut self, project_id: &str) -> Result<()> {
        self.projects.remove(project_id);
        self.save()
    }

    /// Save registry to disk
    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.registry_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create registry directory")?;
        }

        let content = serde_json::to_string_pretty(&self.projects)
            .context("Failed to serialize project registry")?;
        
        fs::write(&self.registry_path, content)
            .context("Failed to write project registry")?;

        Ok(())
    }

    /// Get total statistics across all projects
    pub fn get_global_stats(&self) -> Result<GlobalStats> {
        let total_projects = self.projects.len();
        let mut total_pages = 0;
        let mut total_chunks = 0;
        let mut total_disk_usage = 0;
        
        for info in self.projects.values() {
            total_pages += info.stats.page_count;
            total_chunks += info.stats.chunk_count;
            total_disk_usage += info.stats.disk_usage;
        }
        
        Ok(GlobalStats {
            total_projects,
            total_pages,
            total_chunks,
            total_disk_usage,
        })
    }
}

#[derive(Debug, Clone)]
pub struct GlobalStats {
    pub total_projects: usize,
    pub total_pages: usize,
    pub total_chunks: usize,
    pub total_disk_usage: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_project_registry() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut registry = ProjectRegistry::new(temp_dir.path())?;

        let identifier = ProjectIdentifier {
            id: "test-project".to_string(),
            name: "Test Project".to_string(),
        };

        let project_path = PathBuf::from("/path/to/project");
        let info = registry.register_project(&identifier, &project_path)?;

        assert_eq!(info.name, "Test Project");
        assert_eq!(info.path, project_path.display().to_string());

        let retrieved = registry.get_project("test-project").unwrap().unwrap();
        assert_eq!(retrieved.name, info.name);

        Ok(())
    }
}
