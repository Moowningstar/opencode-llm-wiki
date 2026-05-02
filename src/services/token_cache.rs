use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tiktoken_rs::{get_bpe_from_model, CoreBPE};

/// Token 预计算结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizedChunk {
    pub chunk_id: String,
    pub page_id: String,
    pub chunk_index: u32,
    pub chunk_text: String,
    pub token_ids: Vec<usize>,
    pub token_count: usize,
}

/// Token 缓存服务
/// 
/// 核心功能：
/// 1. 预计算文档 chunk 的 token IDs（导入时）
/// 2. 存储 token IDs 到 LanceDB payload（扩展现有 schema）
/// 3. 查询时直接使用缓存的 token IDs，避免重复 tokenization
/// 
/// 性能目标：
/// - 缓存命中率：100%（所有导入的 chunk 都预计算）
/// - Token 消耗减少：70%（查询时不再重复 tokenize 检索结果）
/// - 查询延迟降低：500ms → 150ms（跳过 tokenization 步骤）
pub struct TokenCacheService {
    /// tiktoken BPE 编码器（支持 GPT-3.5/4 系列）
    bpe: CoreBPE,
    /// 内存缓存（可选，用于热数据加速）
    memory_cache: HashMap<String, Vec<usize>>,
}

impl TokenCacheService {
    /// 创建 Token 缓存服务
    /// 
    /// # Arguments
    /// * `model` - 模型名称（如 "gpt-4", "gpt-3.5-turbo"）
    pub fn new(model: &str) -> Result<Self> {
        let bpe = get_bpe_from_model(model)
            .context(format!("Failed to load BPE for model: {}", model))?;
        
        Ok(Self {
            bpe,
            memory_cache: HashMap::new(),
        })
    }

    /// 预计算单个 chunk 的 token IDs
    /// 
    /// # Arguments
    /// * `chunk_id` - Chunk 唯一标识（格式：page_id#chunk_index）
    /// * `text` - Chunk 文本内容
    /// 
    /// # Returns
    /// Token IDs 向量
    pub fn tokenize_chunk(&mut self, chunk_id: &str, text: &str) -> Result<Vec<usize>> {
        // 检查内存缓存
        if let Some(cached) = self.memory_cache.get(chunk_id) {
            return Ok(cached.clone());
        }

        // 执行 tokenization
        let token_ids = self.bpe.encode_with_special_tokens(text);

        // 更新内存缓存
        self.memory_cache.insert(chunk_id.to_string(), token_ids.clone());

        Ok(token_ids)
    }

    /// 批量预计算多个 chunks 的 token IDs
    /// 
    /// # Arguments
    /// * `chunks` - Chunk 列表（chunk_id + text）
    /// 
    /// # Returns
    /// TokenizedChunk 列表（包含 token_ids 和 token_count）
    pub fn tokenize_chunks_batch(
        &mut self,
        chunks: Vec<(String, String, String, u32)>, // (chunk_id, page_id, text, chunk_index)
    ) -> Result<Vec<TokenizedChunk>> {
        let mut results = Vec::with_capacity(chunks.len());

        for (chunk_id, page_id, text, chunk_index) in chunks {
            let token_ids = self.tokenize_chunk(&chunk_id, &text)?;
            let token_count = token_ids.len();

            results.push(TokenizedChunk {
                chunk_id,
                page_id,
                chunk_index,
                chunk_text: text,
                token_ids,
                token_count,
            });
        }

        Ok(results)
    }

    /// 计算文本的 token 数量（不缓存）
    /// 
    /// 用于快速估算 token 预算，不需要完整的 token IDs
    pub fn count_tokens(&self, text: &str) -> usize {
        self.bpe.encode_with_special_tokens(text).len()
    }

    /// 清空内存缓存
    pub fn clear_cache(&mut self) {
        self.memory_cache.clear();
    }

    /// 获取缓存统计信息
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            cached_chunks: self.memory_cache.len(),
            memory_usage_bytes: self.memory_cache
                .values()
                .map(|v| v.len() * std::mem::size_of::<usize>())
                .sum(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub cached_chunks: usize,
    pub memory_usage_bytes: usize,
}

/// 查询优化器
/// 
/// 使用预计算的 token IDs 优化 LLM 查询流程：
/// 1. 从 LanceDB 检索相关 chunks（带 token_ids payload）
/// 2. 根据 token 预算筛选 chunks（无需重新 tokenize）
/// 3. 构建最优上下文（最大化信息密度）
pub struct QueryOptimizer {
    token_cache: TokenCacheService,
}

impl QueryOptimizer {
    pub fn new(model: &str) -> Result<Self> {
        Ok(Self {
            token_cache: TokenCacheService::new(model)?,
        })
    }

    /// 优化检索结果，确保不超过 token 预算
    /// 
    /// # Arguments
    /// * `chunks` - 检索到的 chunks（已包含 token_ids）
    /// * `token_budget` - Token 预算上限
    /// 
    /// # Returns
    /// 筛选后的 chunks（按相关性排序，总 token 数不超过预算）
    pub fn optimize_context(
        &self,
        chunks: Vec<TokenizedChunk>,
        token_budget: usize,
    ) -> Vec<TokenizedChunk> {
        let mut selected = Vec::new();
        let mut total_tokens = 0;

        for chunk in chunks {
            let chunk_tokens = chunk.token_count;
            
            if total_tokens + chunk_tokens <= token_budget {
                total_tokens += chunk_tokens;
                selected.push(chunk);
            } else {
                // 预算耗尽，停止添加
                break;
            }
        }

        selected
    }

    /// 计算查询的 token 消耗（query + context）
    pub fn estimate_query_tokens(
        &self,
        query: &str,
        context_chunks: &[TokenizedChunk],
    ) -> usize {
        let query_tokens = self.token_cache.count_tokens(query);
        let context_tokens: usize = context_chunks.iter().map(|c| c.token_count).sum();
        query_tokens + context_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_chunk() {
        let mut service = TokenCacheService::new("gpt-4").unwrap();
        
        let chunk_id = "test-page#0";
        let text = "Hello, world! This is a test.";
        
        let token_ids = service.tokenize_chunk(chunk_id, text).unwrap();
        assert!(!token_ids.is_empty());
        
        // 验证缓存命中
        let cached_ids = service.tokenize_chunk(chunk_id, text).unwrap();
        assert_eq!(token_ids, cached_ids);
    }

    #[test]
    fn test_batch_tokenization() {
        let mut service = TokenCacheService::new("gpt-4").unwrap();
        
        let chunks = vec![
            ("page1#0".to_string(), "page1".to_string(), "First chunk".to_string(), 0),
            ("page1#1".to_string(), "page1".to_string(), "Second chunk".to_string(), 1),
        ];
        
        let results = service.tokenize_chunks_batch(chunks).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].token_count > 0);
        assert!(results[1].token_count > 0);
    }

    #[test]
    fn test_query_optimizer() {
        let optimizer = QueryOptimizer::new("gpt-4").unwrap();
        
        let chunks = vec![
            TokenizedChunk {
                chunk_id: "page1#0".to_string(),
                page_id: "page1".to_string(),
                chunk_index: 0,
                chunk_text: "First chunk with some content".to_string(),
                token_ids: vec![1, 2, 3, 4, 5],
                token_count: 5,
            },
            TokenizedChunk {
                chunk_id: "page1#1".to_string(),
                page_id: "page1".to_string(),
                chunk_index: 1,
                chunk_text: "Second chunk with more content".to_string(),
                token_ids: vec![6, 7, 8, 9, 10],
                token_count: 5,
            },
        ];
        
        // Token 预算只够第一个 chunk
        let selected = optimizer.optimize_context(chunks.clone(), 5);
        assert_eq!(selected.len(), 1);
        
        // Token 预算够两个 chunks
        let selected = optimizer.optimize_context(chunks, 10);
        assert_eq!(selected.len(), 2);
    }

    #[test]
    fn test_cache_stats() {
        let mut service = TokenCacheService::new("gpt-4").unwrap();
        
        service.tokenize_chunk("test#0", "Hello").unwrap();
        service.tokenize_chunk("test#1", "World").unwrap();
        
        let stats = service.cache_stats();
        assert_eq!(stats.cached_chunks, 2);
        assert!(stats.memory_usage_bytes > 0);
    }
}
