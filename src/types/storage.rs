//! 新的全局存储架构类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 项目唯一标识符（路径哈希）
pub type ProjectId = String;

/// 内容哈希（SHA256）
pub type ContentHash = String;

/// 向量 ID
pub type VectorId = String;

/// 项目注册表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRegistry {
    /// 项目映射：project_id -> ProjectInfo
    pub projects: HashMap<ProjectId, ProjectInfo>,
    /// 版本号
    pub version: String,
}

/// 项目信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// 项目名称
    pub name: String,
    /// 项目路径
    pub path: String,
    /// 创建时间（ISO 8601）
    pub created_at: String,
    /// 最后访问时间（ISO 8601）
    pub last_accessed: String,
    /// 项目统计
    pub stats: ProjectStats,
}

/// 项目统计信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectStats {
    /// 页面数量
    pub page_count: usize,
    /// chunk 数量
    pub chunk_count: usize,
    /// 唯一向量数量（去重后）
    pub unique_vector_count: usize,
    /// 磁盘占用（字节）
    pub disk_usage: u64,
}

/// 内容哈希索引条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashIndexEntry {
    /// 内容哈希
    pub content_hash: ContentHash,
    /// 向量 ID
    pub vector_id: VectorId,
    /// 引用计数
    pub ref_count: usize,
    /// 创建时间
    pub created_at: String,
}

/// Chunk 到内容哈希的映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMapping {
    /// chunk_id -> content_hash
    pub mappings: HashMap<String, ContentHash>,
    /// 版本号
    pub version: String,
}

/// 全局存储路径配置
#[derive(Debug, Clone)]
pub struct GlobalStoragePaths {
    /// 根目录：~/.opencode-llm-wiki/
    pub root: String,
    /// 向量存储：~/.opencode-llm-wiki/.vectors/.store/
    pub vectors: String,
    /// 哈希索引：~/.opencode-llm-wiki/.vectors/.hash_index.db
    pub hash_index: String,
    /// 图存储：~/.opencode-llm-wiki/.graph/.store/
    pub graph: String,
    /// 项目目录：~/.opencode-llm-wiki/.projects/
    pub projects: String,
    /// 项目注册表：~/.opencode-llm-wiki/.projects/.registry.json
    pub registry: String,
    /// 缓存目录：~/.opencode-llm-wiki/.cache/
    pub cache: String,
    /// 锁目录：~/.opencode-llm-wiki/.locks/
    pub locks: String,
    /// 日志目录：~/.opencode-llm-wiki/.logs/
    pub logs: String,
}

impl GlobalStoragePaths {
    /// 创建默认路径配置（使用 ~/.opencode-llm-wiki）
    pub fn default() -> Self {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        
        let root = format!("{}/.opencode-llm-wiki", home);
        Self::new(root)
    }
    
    /// 使用自定义根目录创建路径配置
    pub fn new(root: String) -> Self {
        Self {
            vectors: format!("{}/.vectors/.store", root),
            hash_index: format!("{}/.vectors/.hash_index.db", root),
            graph: format!("{}/.graph/.store", root),
            projects: format!("{}/.projects", root),
            registry: format!("{}/.projects/.registry.json", root),
            cache: format!("{}/.cache", root),
            locks: format!("{}/.locks", root),
            logs: format!("{}/.logs", root),
            root,
        }
    }
    
    /// 获取项目目录路径
    pub fn project_dir(&self, project_id: &str) -> String {
        format!("{}/.{}", self.projects, project_id)
    }
    
    /// 获取项目 wiki 目录
    pub fn project_wiki(&self, project_id: &str) -> String {
        format!("{}/.wiki", self.project_dir(project_id))
    }
    
    /// 获取项目元数据目录
    pub fn project_meta(&self, project_id: &str) -> String {
        format!("{}/.meta", self.project_dir(project_id))
    }
    
    /// 获取项目信息文件
    pub fn project_info(&self, project_id: &str) -> String {
        format!("{}/.info.json", self.project_dir(project_id))
    }
    
    /// 获取项目 chunk 映射文件
    pub fn project_chunks(&self, project_id: &str) -> String {
        format!("{}/.chunks.json", self.project_meta(project_id))
    }
    
    /// 获取项目引用数据库
    pub fn project_refs(&self, project_id: &str) -> String {
        format!("{}/.refs.db", self.project_dir(project_id))
    }
}

/// 项目标识文件内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMarker {
    /// 项目 ID
    pub id: ProjectId,
    /// 版本号
    pub v: String,
}

impl ProjectMarker {
    pub fn new(id: ProjectId) -> Self {
        Self {
            id,
            v: "1.0".to_string(),
        }
    }
}

/// 向量去重结果
#[derive(Debug, Clone)]
pub enum DeduplicationResult {
    /// 复用已有向量
    Reused {
        vector_id: VectorId,
        content_hash: ContentHash,
    },
    /// 创建新向量
    Created {
        vector_id: VectorId,
        content_hash: ContentHash,
    },
}

impl DeduplicationResult {
    pub fn vector_id(&self) -> &str {
        match self {
            Self::Reused { vector_id, .. } => vector_id,
            Self::Created { vector_id, .. } => vector_id,
        }
    }
    
    pub fn content_hash(&self) -> &str {
        match self {
            Self::Reused { content_hash, .. } => content_hash,
            Self::Created { content_hash, .. } => content_hash,
        }
    }
    
    pub fn is_reused(&self) -> bool {
        matches!(self, Self::Reused { .. })
    }
}
