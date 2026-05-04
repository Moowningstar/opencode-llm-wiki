use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::traits::{VectorStorage, ChunkInput, SearchResult, StorageResult, StorageError};

use ruvector_core::{VectorDB, DistanceMetric};
use ruvector_core::types::DbOptions;
use ruvector_graph::{GraphDB, Node, Edge, Properties, PropertyValue};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub chunk_id: String,
    pub page_id: String,
    pub chunk_index: u32,
    pub chunk_text: String,
    pub heading_path: String,
    pub token_ids: Option<Vec<u32>>,
    pub token_count: Option<u32>,
}

#[cfg(feature = "ruvector")]
pub struct RuVectorStorage {
    vector_db: Arc<RwLock<VectorDB>>,
    graph_db: Arc<RwLock<GraphDB>>,
    dimension: usize,
}

#[cfg(feature = "ruvector")]
impl RuVectorStorage {
    pub async fn new(project_path: String, embedding_dim: usize) -> StorageResult<Self> {
        let db_path = Path::new(&project_path)
            .join(".llm-wiki/ruvector")
            .to_string_lossy()
            .to_string();

        std::fs::create_dir_all(Path::new(&project_path).join(".llm-wiki"))
            .map_err(|e| StorageError::new(format!("Failed to create directory: {}", e)))?;

        let vector_options = DbOptions {
            storage_path: format!("{}/vectors", db_path),
            dimensions: embedding_dim,
            distance_metric: DistanceMetric::Cosine,
            hnsw_config: None,
            quantization: None,
        };
        
        let vector_db = VectorDB::new(vector_options)
            .map_err(|e| StorageError::new(format!("Failed to create vector DB: {}", e)))?;
        
        let graph_db = GraphDB::with_storage(&format!("{}/graph", db_path))
            .map_err(|e| StorageError::new(format!("Failed to create graph DB: {}", e)))?;
        
        Ok(Self {
            vector_db: Arc::new(RwLock::new(vector_db)),
            graph_db: Arc::new(RwLock::new(graph_db)),
            dimension: embedding_dim,
        })
    }

    pub async fn add_edge(&self, from: &str, to: &str, edge_type: &str) -> StorageResult<()> {
        let graph = self.graph_db.write().await;
        
        let edge = Edge::create(from.to_string(), to.to_string(), edge_type);

        graph.create_edge(edge)
            .map_err(|e| StorageError::new(format!("Failed to add edge: {}", e)))?;

        Ok(())
    }

    pub async fn get_neighbors(&self, node_id: &str) -> StorageResult<Vec<String>> {
        let graph = self.graph_db.read().await;
        
        let neighbors: Vec<String> = graph.get_outgoing_edges(&node_id.to_string())
            .into_iter()
            .map(|edge| edge.to)
            .collect();

        Ok(neighbors)
    }

    pub async fn bfs(&self, start: &str, max_depth: usize) -> StorageResult<Vec<String>> {
        use std::collections::{HashSet, VecDeque};
        
        let graph = self.graph_db.read().await;
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();
        
        queue.push_back((start.to_string(), 0));
        visited.insert(start.to_string());
        
        while let Some((current, depth)) = queue.pop_front() {
            if depth > max_depth {
                continue;
            }
            
            result.push(current.clone());
            
            let edges = graph.get_outgoing_edges(&current);
            for edge in edges {
                if !visited.contains(&edge.to) {
                    visited.insert(edge.to.clone());
                    queue.push_back((edge.to, depth + 1));
                }
            }
        }

        Ok(result)
    }
}

#[cfg(feature = "ruvector")]
#[async_trait]
impl VectorStorage for RuVectorStorage {
    async fn upsert_chunks(&self, page_id: &str, chunks: Vec<ChunkInput>) -> StorageResult<()> {
        if chunks.is_empty() {
            return Ok(());
        }

        let dim = chunks[0].embedding.len();
        if dim != self.dimension {
            return Err(StorageError::new(format!(
                "Embedding dimension mismatch: expected {}, got {}",
                self.dimension, dim
            )));
        }

        let vector_db = self.vector_db.write().await;
        let graph_db = self.graph_db.write().await;

        for chunk in chunks {
            let chunk_id = format!("{}#{}", page_id, chunk.chunk_index);
            
            let metadata = ChunkMetadata {
                chunk_id: chunk_id.clone(),
                page_id: page_id.to_string(),
                chunk_index: chunk.chunk_index,
                chunk_text: chunk.chunk_text.clone(),
                heading_path: chunk.heading_path.clone(),
                token_ids: chunk.token_ids.clone(),
                token_count: chunk.token_count,
            };

            let mut metadata_map = std::collections::HashMap::new();
            metadata_map.insert("data".to_string(), serde_json::to_value(&metadata)
                .map_err(|e| StorageError::new(format!("Failed to serialize metadata: {}", e)))?);

            let entry = ruvector_core::VectorEntry {
                id: Some(chunk_id.clone()),
                vector: chunk.embedding,
                metadata: Some(metadata_map.clone()),
            };

            vector_db.insert(entry)
                .map_err(|e| StorageError::new(format!("Failed to insert vector: {}", e)))?;

            let mut properties = Properties::new();
            for (k, v) in metadata_map {
                properties.insert(k, PropertyValue::String(v.to_string()));
            }
            
            let node = Node::new(chunk_id.clone(), vec![], properties);

            graph_db.create_node(node)
                .map_err(|e| StorageError::new(format!("Failed to add graph node: {}", e)))?;
        }

        Ok(())
    }

    async fn search(&self, query_embedding: Vec<f32>, top_k: usize) -> StorageResult<Vec<SearchResult>> {
        if query_embedding.len() != self.dimension {
            return Err(StorageError::new(format!(
                "Query embedding dimension mismatch: expected {}, got {}",
                self.dimension, query_embedding.len()
            )));
        }

        let vector_db = self.vector_db.read().await;

        let query = ruvector_core::SearchQuery {
            vector: query_embedding,
            k: top_k,
            filter: None,
            ef_search: None,
        };

        let results = vector_db.search(query)
            .map_err(|e| StorageError::new(format!("Failed to search: {}", e)))?;

        let search_results: Vec<SearchResult> = results.into_iter()
            .filter_map(|result| {
                let metadata_map = result.metadata?;
                let metadata_value = metadata_map.get("data")?;
                let metadata: ChunkMetadata = serde_json::from_value(metadata_value.clone()).ok()?;
                
                Some(SearchResult {
                    chunk_id: metadata.chunk_id,
                    page_id: metadata.page_id,
                    chunk_index: metadata.chunk_index,
                    chunk_text: metadata.chunk_text,
                    heading_path: metadata.heading_path,
                    score: result.score,
                    token_ids: metadata.token_ids,
                    token_count: metadata.token_count,
                })
            })
            .collect();

        Ok(search_results)
    }

    async fn delete_page(&self, page_id: &str) -> StorageResult<()> {
        let vector_db = self.vector_db.write().await;
        let graph_db = self.graph_db.write().await;

        let query = ruvector_core::SearchQuery {
            vector: vec![0.0; self.dimension],
            k: 10000,
            filter: None,
            ef_search: None,
        };

        let all_results = vector_db.search(query)
            .map_err(|e| StorageError::new(format!("Failed to search for deletion: {}", e)))?;

        for result in all_results {
            if result.id.starts_with(&format!("{}#", page_id)) {
                vector_db.delete(&result.id)
                    .map_err(|e| StorageError::new(format!("Failed to delete vector: {}", e)))?;
                
                let _ = graph_db.delete_node(&result.id);
            }
        }

        Ok(())
    }

    async fn count(&self) -> StorageResult<usize> {
        let vector_db = self.vector_db.read().await;
        
        let count = vector_db.len()
            .map_err(|e| StorageError::new(format!("Failed to count: {}", e)))?;

        Ok(count)
    }

    fn embedding_dim(&self) -> usize {
        self.dimension
    }
}

#[cfg(not(feature = "ruvector"))]
pub struct RuVectorStorage;

#[cfg(not(feature = "ruvector"))]
impl RuVectorStorage {
    pub async fn new(_project_path: String, _embedding_dim: usize) -> StorageResult<Self> {
        Err(StorageError::new(
            "RuVector feature is not enabled. Compile with --features ruvector"
        ))
    }
}

#[cfg(all(test, feature = "ruvector"))]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp_project() -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let p = std::env::temp_dir().join(format!("llm-wiki-rvtest-{}-{}", ts, id));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn fake_embedding(seed: u32, dim: usize) -> Vec<f32> {
        (0..dim)
            .map(|i| {
                let x = ((seed.wrapping_mul(2654435761)) ^ (i as u32)) as f32;
                (x / u32::MAX as f32).sin()
            })
            .collect()
    }

    fn make_chunks(page_id: &str, n: u32, dim: usize) -> Vec<ChunkInput> {
        (0..n)
            .map(|i| ChunkInput {
                chunk_index: i,
                chunk_text: format!("{} chunk {}", page_id, i),
                heading_path: format!("## Heading {}", i),
                embedding: fake_embedding(i, dim),
                token_ids: None,
                token_count: None,
            })
            .collect()
    }

    #[tokio::test]
    async fn test_upsert_and_count() {
        let p = tmp_project();
        let pp = p.to_string_lossy().to_string();
        let storage = RuVectorStorage::new(pp, 16).await.unwrap();

        let chunks = make_chunks("test-page", 3, 16);
        storage.upsert_chunks("test-page", chunks).await.unwrap();

        let count = storage.count().await.unwrap();
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_vector_search() {
        let p = tmp_project();
        let pp = p.to_string_lossy().to_string();
        let storage = RuVectorStorage::new(pp, 16).await.unwrap();

        let chunks = make_chunks("page-a", 5, 16);
        storage.upsert_chunks("page-a", chunks).await.unwrap();

        let query = fake_embedding(2, 16);
        let results = storage.search(query, 3).await.unwrap();

        assert!(!results.is_empty());
        assert!(results.len() <= 3);
        
        for result in &results {
            assert_eq!(result.page_id, "page-a");
            assert!(result.chunk_text.contains("chunk"));
        }
    }

    #[tokio::test]
    async fn test_graph_operations() {
        let p = tmp_project();
        let pp = p.to_string_lossy().to_string();
        let storage = RuVectorStorage::new(pp, 16).await.unwrap();

        let chunks_a = make_chunks("page-a", 2, 16);
        let chunks_b = make_chunks("page-b", 2, 16);
        
        storage.upsert_chunks("page-a", chunks_a).await.unwrap();
        storage.upsert_chunks("page-b", chunks_b).await.unwrap();

        storage.add_edge("page-a#0", "page-b#0", "references").await.unwrap();
        storage.add_edge("page-a#0", "page-b#1", "references").await.unwrap();

        let neighbors = storage.get_neighbors("page-a#0").await.unwrap();
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&"page-b#0".to_string()));
        assert!(neighbors.contains(&"page-b#1".to_string()));
    }

    #[tokio::test]
    async fn test_bfs_traversal() {
        let p = tmp_project();
        let pp = p.to_string_lossy().to_string();
        let storage = RuVectorStorage::new(pp, 16).await.unwrap();

        let chunks = make_chunks("page-a", 1, 16);
        storage.upsert_chunks("page-a", chunks).await.unwrap();
        
        let chunks = make_chunks("page-b", 1, 16);
        storage.upsert_chunks("page-b", chunks).await.unwrap();
        
        let chunks = make_chunks("page-c", 1, 16);
        storage.upsert_chunks("page-c", chunks).await.unwrap();

        storage.add_edge("page-a#0", "page-b#0", "references").await.unwrap();
        storage.add_edge("page-b#0", "page-c#0", "references").await.unwrap();

        let visited = storage.bfs("page-a#0", 2).await.unwrap();
        
        assert!(visited.contains(&"page-a#0".to_string()));
        assert!(visited.contains(&"page-b#0".to_string()));
        assert!(visited.contains(&"page-c#0".to_string()));
    }

    #[tokio::test]
    async fn test_delete_page() {
        let p = tmp_project();
        let pp = p.to_string_lossy().to_string();
        let storage = RuVectorStorage::new(pp, 16).await.unwrap();

        let chunks_a = make_chunks("page-a", 3, 16);
        let chunks_b = make_chunks("page-b", 2, 16);
        
        storage.upsert_chunks("page-a", chunks_a).await.unwrap();
        storage.upsert_chunks("page-b", chunks_b).await.unwrap();
        
        assert_eq!(storage.count().await.unwrap(), 5);

        storage.delete_page("page-a").await.unwrap();
        
        assert_eq!(storage.count().await.unwrap(), 2);
    }
}
