//! 数据迁移工具：从旧的项目本地存储迁移到新的全局存储架构

use crate::storage::{VectorDeduplicator, ProjectRegistry, ProjectIdentifier};
use crate::types::storage::GlobalStoragePaths;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::fs;

/// 迁移统计信息
#[derive(Debug, Clone)]
pub struct MigrationStats {
    pub projects_found: usize,
    pub projects_migrated: usize,
    pub total_chunks: usize,
    pub unique_vectors: usize,
    pub reused_vectors: usize,
    pub bytes_saved: u64,
}

/// 数据迁移器
pub struct DataMigrator {
    #[allow(dead_code)]
    global_paths: GlobalStoragePaths,
    registry: ProjectRegistry,
    #[allow(dead_code)]
    deduplicator: VectorDeduplicator,
}

impl DataMigrator {
    /// 创建新的迁移器
    pub fn new(global_root: Option<PathBuf>) -> Result<Self> {
        let global_paths = if let Some(root) = global_root {
            GlobalStoragePaths::new(root.display().to_string())
        } else {
            GlobalStoragePaths::default()
        };
        
        // 确保全局存储目录存在
        fs::create_dir_all(&global_paths.root)
            .context("Failed to create global storage root")?;
        
        let registry = ProjectRegistry::new(Path::new(&global_paths.root))?;
        let deduplicator = VectorDeduplicator::new(Path::new(&global_paths.root))?;
        
        Ok(Self {
            global_paths,
            registry,
            deduplicator,
        })
    }
    
    /// 扫描目录查找旧的项目数据
    pub fn scan_for_old_projects(&self, search_root: &Path) -> Result<Vec<PathBuf>> {
        let mut projects = Vec::new();
        
        if !search_root.exists() {
            return Ok(projects);
        }
        
        // 递归扫描查找包含 .llm-wiki/ruvector/ 的目录
        self.scan_recursive(search_root, &mut projects)?;
        
        Ok(projects)
    }
    
    fn scan_recursive(&self, dir: &Path, projects: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        
        // 检查当前目录是否包含旧的存储结构
        let old_storage = dir.join(".llm-wiki").join("ruvector");
        if old_storage.exists() && old_storage.is_dir() {
            projects.push(dir.to_path_buf());
            return Ok(()); // 不继续递归子目录
        }
        
        // 递归扫描子目录
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(path) = entry.path().canonicalize() {
                    // 跳过隐藏目录和全局存储目录
                    if let Some(name) = path.file_name() {
                        let name_str = name.to_string_lossy();
                        if name_str.starts_with('.') || name_str == "node_modules" || name_str == "target" {
                            continue;
                        }
                    }
                    
                    self.scan_recursive(&path, projects)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// 迁移单个项目
    pub fn migrate_project(&mut self, project_path: &Path, dry_run: bool) -> Result<MigrationStats> {
        let old_storage = project_path.join(".llm-wiki").join("ruvector");
        
        if !old_storage.exists() {
            bail!("No old storage found at {:?}", old_storage);
        }
        
        println!("📦 Migrating project: {}", project_path.display());
        
        // 生成项目 ID（使用路径的哈希）
        let project_id = self.generate_project_id(project_path);
        let project_name = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let mut stats = MigrationStats {
            projects_found: 1,
            projects_migrated: 0,
            total_chunks: 0,
            unique_vectors: 0,
            reused_vectors: 0,
            bytes_saved: 0,
        };
        
        if dry_run {
            println!("   [DRY RUN] Would migrate to project_id: {}", project_id);
            return Ok(stats);
        }
        
        // 注册项目
        let identifier = ProjectIdentifier {
            id: project_id.clone(),
            name: project_name.clone(),
        };
        
        self.registry.register_project(&identifier, project_path)?;
        
        // TODO: 实际的向量数据迁移逻辑
        // 这需要：
        // 1. 读取旧的 RuVector 存储
        // 2. 遍历所有向量
        // 3. 通过 deduplicator 检查每个向量
        // 4. 将向量写入新的全局存储
        // 5. 更新图数据库引用
        
        println!("   ⚠️  Vector data migration not yet implemented");
        println!("   Project registered in global registry");
        
        stats.projects_migrated = 1;
        
        Ok(stats)
    }
    
    /// 迁移多个项目
    pub fn migrate_all(&mut self, projects: Vec<PathBuf>, dry_run: bool) -> Result<MigrationStats> {
        let mut total_stats = MigrationStats {
            projects_found: projects.len(),
            projects_migrated: 0,
            total_chunks: 0,
            unique_vectors: 0,
            reused_vectors: 0,
            bytes_saved: 0,
        };
        
        for project_path in projects {
            match self.migrate_project(&project_path, dry_run) {
                Ok(stats) => {
                    total_stats.projects_migrated += stats.projects_migrated;
                    total_stats.total_chunks += stats.total_chunks;
                    total_stats.unique_vectors += stats.unique_vectors;
                    total_stats.reused_vectors += stats.reused_vectors;
                    total_stats.bytes_saved += stats.bytes_saved;
                }
                Err(e) => {
                    eprintln!("   ❌ Failed to migrate {}: {}", project_path.display(), e);
                }
            }
        }
        
        Ok(total_stats)
    }
    
    /// 生成项目 ID（基于路径的哈希）
    fn generate_project_id(&self, project_path: &Path) -> String {
        use sha2::{Digest, Sha256};
        
        let canonical = project_path
            .canonicalize()
            .unwrap_or_else(|_| project_path.to_path_buf());
        
        let mut hasher = Sha256::new();
        hasher.update(canonical.display().to_string().as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        
        // 使用前 16 个字符作为项目 ID
        hash[..16].to_string()
    }
    
    /// 保存注册表
    pub fn save(&mut self) -> Result<()> {
        self.registry.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_scan_for_projects() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path();
        
        // 创建一个旧项目结构
        let project1 = root.join("project1");
        fs::create_dir_all(project1.join(".llm-wiki").join("ruvector"))?;
        
        // 创建另一个旧项目结构
        let project2 = root.join("nested").join("project2");
        fs::create_dir_all(project2.join(".llm-wiki").join("ruvector"))?;
        
        // 创建一个没有旧存储的目录
        fs::create_dir_all(root.join("other"))?;
        
        let global_root = temp.path().join("global");
        let migrator = DataMigrator::new(Some(global_root))?;
        
        let projects = migrator.scan_for_old_projects(root)?;
        
        assert_eq!(projects.len(), 2);
        
        Ok(())
    }
    
    #[test]
    fn test_generate_project_id() -> Result<()> {
        let temp = TempDir::new()?;
        let global_root = temp.path().join("global");
        let migrator = DataMigrator::new(Some(global_root))?;
        
        let path1 = PathBuf::from("/path/to/project");
        let path2 = PathBuf::from("/path/to/project");
        let path3 = PathBuf::from("/different/path");
        
        let id1 = migrator.generate_project_id(&path1);
        let id2 = migrator.generate_project_id(&path2);
        let id3 = migrator.generate_project_id(&path3);
        
        // 相同路径应该生成相同 ID
        assert_eq!(id1, id2);
        
        // 不同路径应该生成不同 ID
        assert_ne!(id1, id3);
        
        // ID 应该是 16 个字符
        assert_eq!(id1.len(), 16);
        
        Ok(())
    }
}
