use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::traits::{VectorStorage, ChunkInput, SearchResult, StorageResult, StorageError};
use super::deduplication::VectorDeduplicator;
use crate::types::storage::GlobalStoragePaths;

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
    deduplicator: Option<Arc<tokio::sync::Mutex<VectorDeduplicator>>>,
    #[allow(dead_code)]
    global_paths: Option<GlobalStoragePaths>,
}

#[cfg(feature = "ruvector")]
impl RuVectorStorage {
    pub async fn new(project_path: String, embedding_dim: usize) -> StorageResult<Self> {
        Self::new_with_global_root(project_path, embedding_dim, None).await
    }
    
    pub async fn new_with_global_root(
        project_path: String, 
        embedding_dim: usize,
        global_root: Option<String>
    ) -> StorageResult<Self> {
        let global_paths = match global_root {
            Some(root) => GlobalStoragePaths::new(root),
            None => GlobalStoragePaths::default(),
        };
        
        let deduplicator = VectorDeduplicator::new(Path::new(&global_paths.root))
            .map_err(|e| StorageError::new(format!("Failed to initialize deduplicator: {}", e)))?;
        
        let vector_path = global_paths.vectors.clone();
        
        let graph_path = Path::new(&project_path)
            .join(".llm-wiki/ruvector/graph")
            .to_string_lossy()
            .to_string();
        
        std::fs::create_dir_all(Path::new(&graph_path).parent().unwrap())
            .map_err(|e| StorageError::new(format!("Failed to create ruvector directory: {}", e)))?;

        println!("📁 Initializing RuVector storage:");
        println!("   Vector DB (global): {}", vector_path);
        println!("   Graph DB (local): {}", graph_path);

        let vector_options = DbOptions {
            storage_path: vector_path.clone(),
            dimensions: embedding_dim,
            distance_metric: DistanceMetric::Cosine,
            hnsw_config: None,
            quantization: None,
        };
        
        let vector_db = VectorDB::new(vector_options)
            .map_err(|e| StorageError::new(format!("Failed to create vector DB: {}", e)))?;
        
        let graph_db = GraphDB::with_storage(&graph_path)
            .map_err(|e| StorageError::new(format!("Failed to create graph DB: {}", e)))?;
        
        println!("✅ RuVector storage initialized successfully");
        
        Ok(Self {
            vector_db: Arc::new(RwLock::new(vector_db)),
            graph_db: Arc::new(RwLock::new(graph_db)),
            dimension: embedding_dim,
            deduplicator: Some(Arc::new(tokio::sync::Mutex::new(deduplicator))),
            global_paths: Some(global_paths),
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
    async fn upsert_chunks(&self, project_id: &str, page_id: &str, chunks: Vec<ChunkInput>) -> StorageResult<()> {
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
        
        let dedup = self.deduplicator.as_ref()
            .ok_or_else(|| StorageError::new("Deduplicator not initialized".to_string()))?;
        let mut deduplicator = dedup.lock().await;

        for chunk in chunks {
            let chunk_id = format!("{}:{}#{}", project_id, page_id, chunk.chunk_index);
            
            // Check for deduplication
            let dedup_result = deduplicator.check_or_create(
                &chunk.chunk_text,
                project_id,
                &chunk_id,
            ).map_err(|e| StorageError::new(format!("Deduplication failed: {}", e)))?;

            let vector_id = match &dedup_result {
                crate::types::storage::DeduplicationResult::Created { vector_id, .. } => {
                    // New vector - insert into vector DB
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
                    metadata_map.insert("project_id".to_string(), serde_json::json!(project_id));

                    let entry = ruvector_core::VectorEntry {
                        id: Some(vector_id.clone()),
                        vector: chunk.embedding,
                        metadata: Some(metadata_map.clone()),
                    };

                    vector_db.insert(entry)
                        .map_err(|e| StorageError::new(format!("Failed to insert vector: {}", e)))?;
                    
                    vector_id.clone()
                },
                crate::types::storage::DeduplicationResult::Reused { vector_id, .. } => {
                    // Reused vector - skip vector DB insertion
                    vector_id.clone()
                }
            };

            // Always create graph node (project-local, not deduplicated)
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
            metadata_map.insert("project_id".to_string(), serde_json::json!(project_id));
            metadata_map.insert("vector_id".to_string(), serde_json::json!(vector_id));

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

    async fn search(&self, query_embedding: Vec<f32>, top_k: usize, project_filter: Option<&str>) -> StorageResult<Vec<SearchResult>> {
        if query_embedding.len() != self.dimension {
            return Err(StorageError::new(format!(
                "Query embedding dimension mismatch: expected {}, got {}",
                self.dimension,
                query_embedding.len()
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
            .map_err(|e| StorageError::new(format!("Search failed: {}", e)))?;

        let mut search_results = Vec::new();
        for result in results {
            if let Some(metadata_map) = result.metadata {
                if let Some(filter_project) = project_filter {
                    if let Some(project_value) = metadata_map.get("project_id") {
                        let project_id = project_value.as_str().unwrap_or("");
                        if project_id != filter_project {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                
                if let Some(data_value) = metadata_map.get("data") {
                    let metadata: ChunkMetadata = serde_json::from_value(data_value.clone())
                        .map_err(|e| StorageError::new(format!("Failed to deserialize metadata: {}", e)))?;
                    
                    search_results.push(SearchResult {
                        chunk_id: metadata.chunk_id,
                        page_id: metadata.page_id,
                        chunk_index: metadata.chunk_index,
                        chunk_text: metadata.chunk_text,
                        heading_path: metadata.heading_path,
                        score: result.score,
                        token_ids: metadata.token_ids,
                        token_count: metadata.token_count,
                    });
                }
            }
        }

        Ok(search_results)
    }

    async fn delete_page(&self, project_id: &str, page_id: &str) -> StorageResult<()> {
        let graph_db = self.graph_db.write().await;
        
        let dedup = self.deduplicator.as_ref()
            .ok_or_else(|| StorageError::new("Deduplicator not initialized".to_string()))?;
        let mut deduplicator = dedup.lock().await;

        let prefix = format!("{}:{}#", project_id, page_id);
        let all_nodes = (0..10000)
            .map(|i| format!("{}{}", prefix, i))
            .filter_map(|chunk_id| graph_db.get_node(&chunk_id))
            .collect::<Vec<_>>();

        for node in all_nodes {
            if let Some(PropertyValue::String(data_str)) = node.properties.get("data") {
                if let Ok(data_value) = serde_json::from_str::<serde_json::Value>(data_str) {
                    if let Some(chunk_text) = data_value.get("chunk_text").and_then(|v| v.as_str()) {
                        let content_hash = crate::storage::deduplication::VectorDeduplicator::compute_content_hash(chunk_text);
                        let _ = deduplicator.remove_chunk(&content_hash, project_id, &node.id);
                    }
                }
            }
            
            let _ = graph_db.delete_node(&node.id);
        }

        Ok(())
    }

    async fn count(&self, project_filter: Option<&str>) -> StorageResult<usize> {
        // Count graph nodes (chunks), not vectors (which are deduplicated)
        let graph_db = self.graph_db.read().await;
        
        if let Some(filter_project) = project_filter {
            // Match the format used when storing: serde_json::json!(project_id).to_string()
            let stored_format = serde_json::json!(filter_project).to_string();
            let nodes = graph_db.get_nodes_by_property(
                "project_id",
                &PropertyValue::String(stored_format)
            );
            Ok(nodes.len())
        } else {
            Ok(graph_db.node_count())
        }
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
    
    fn tmp_global_root() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let p = std::env::temp_dir().join(format!("llm-wiki-global-{}-{}", ts, id));
        std::fs::create_dir_all(&p).unwrap();
        p.to_string_lossy().to_string()
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
        let global_root = tmp_global_root();
        let storage = RuVectorStorage::new_with_global_root(pp.clone(), 16, Some(global_root)).await.unwrap();

        let chunks = make_chunks("test-page", 3, 16);
        storage.upsert_chunks(&pp, "test-page", chunks).await.unwrap();

        let count = storage.count(Some(&pp)).await.unwrap();
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_vector_search() {
        let p = tmp_project();
        let pp = p.to_string_lossy().to_string();
        let global_root = tmp_global_root();
        let storage = RuVectorStorage::new_with_global_root(pp.clone(), 16, Some(global_root)).await.unwrap();

        let chunks = make_chunks("page-a", 5, 16);
        storage.upsert_chunks(&pp, "page-a", chunks).await.unwrap();

        let query = fake_embedding(2, 16);
        let results = storage.search(query, 3, Some(&pp)).await.unwrap();

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
        let global_root = tmp_global_root();
        let storage = RuVectorStorage::new_with_global_root(pp.clone(), 16, Some(global_root)).await.unwrap();

        let chunks_a = make_chunks("page-a", 2, 16);
        let chunks_b = make_chunks("page-b", 2, 16);
        
        storage.upsert_chunks(&pp, "page-a", chunks_a).await.unwrap();
        storage.upsert_chunks(&pp, "page-b", chunks_b).await.unwrap();

        // Chunk IDs now include project_id prefix: {project_id}:{page_id}#{chunk_index}
        let chunk_a0 = format!("{}:page-a#0", pp);
        let chunk_b0 = format!("{}:page-b#0", pp);
        let chunk_b1 = format!("{}:page-b#1", pp);

        storage.add_edge(&chunk_a0, &chunk_b0, "references").await.unwrap();
        storage.add_edge(&chunk_a0, &chunk_b1, "references").await.unwrap();

        let neighbors = storage.get_neighbors(&chunk_a0).await.unwrap();
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&chunk_b0));
        assert!(neighbors.contains(&chunk_b1));
    }

    #[tokio::test]
    async fn test_bfs_traversal() {
        let p = tmp_project();
        let pp = p.to_string_lossy().to_string();
        let storage = RuVectorStorage::new(pp.clone(), 16).await.unwrap();

        let chunks = make_chunks("page-a", 1, 16);
        storage.upsert_chunks(&pp, "page-a", chunks).await.unwrap();
        
        let chunks = make_chunks("page-b", 1, 16);
        storage.upsert_chunks(&pp, "page-b", chunks).await.unwrap();
        
        let chunks = make_chunks("page-c", 1, 16);
        storage.upsert_chunks(&pp, "page-c", chunks).await.unwrap();

        let chunk_a0 = format!("{}:page-a#0", pp);
        let chunk_b0 = format!("{}:page-b#0", pp);
        let chunk_c0 = format!("{}:page-c#0", pp);

        storage.add_edge(&chunk_a0, &chunk_b0, "references").await.unwrap();
        storage.add_edge(&chunk_b0, &chunk_c0, "references").await.unwrap();

        let visited = storage.bfs(&chunk_a0, 2).await.unwrap();
        
        assert!(visited.contains(&chunk_a0));
        assert!(visited.contains(&chunk_b0));
        assert!(visited.contains(&chunk_c0));
    }

    #[tokio::test]
    async fn test_delete_page() {
        let p = tmp_project();
        let pp = p.to_string_lossy().to_string();
        let global_root = tmp_global_root();
        let storage = RuVectorStorage::new_with_global_root(pp.clone(), 16, Some(global_root)).await.unwrap();

        let chunks_a = make_chunks("page-a", 3, 16);
        let chunks_b = make_chunks("page-b", 2, 16);
        
        storage.upsert_chunks(&pp, "page-a", chunks_a).await.unwrap();
        storage.upsert_chunks(&pp, "page-b", chunks_b).await.unwrap();
        
        assert_eq!(storage.count(None).await.unwrap(), 5);

        storage.delete_page(&pp, "page-a").await.unwrap();
        
        assert_eq!(storage.count(None).await.unwrap(), 2);
    }
}
