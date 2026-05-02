use arrow_array::{
    Array, ArrayRef, FixedSizeListArray, Float32Array, RecordBatch,
    StringArray, UInt32Array, new_null_array,
};
use arrow_schema::{DataType, Field, Schema};
use lancedb::{connect, query::{ExecutableQuery, QueryBase}};
use std::path::Path;
use std::sync::Arc;

use crate::utils::panic_guard::run_guarded_async;
use super::traits::{VectorStorage, ChunkInput, SearchResult, StorageResult, StorageError};

const TABLE_V2: &str = "wiki_chunks";

/// LanceDB implementation of VectorStorage trait
pub struct LanceDbStorage {
    project_path: String,
}

impl LanceDbStorage {
    pub fn new(project_path: String) -> Self {
        Self { project_path }
    }

    fn db_path(&self) -> String {
        Path::new(&self.project_path)
            .join(".llm-wiki/lancedb")
            .to_string_lossy()
            .to_string()
    }

    fn validate_page_id(&self, page_id: &str) -> Result<(), String> {
        if page_id.is_empty() || page_id.len() > 256 {
            return Err("Invalid page_id: empty or too long".to_string());
        }
        if !page_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(format!(
                "Invalid page_id: contains disallowed characters: {}",
                page_id
            ));
        }
        Ok(())
    }

    fn make_schema(&self, dim: i32) -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("chunk_id", DataType::Utf8, false),
            Field::new("page_id", DataType::Utf8, false),
            Field::new("chunk_index", DataType::UInt32, false),
            Field::new("chunk_text", DataType::Utf8, false),
            Field::new("heading_path", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    dim,
                ),
                false,
            ),
            // Token cache fields
            Field::new(
                "token_ids",
                DataType::List(Arc::new(Field::new("item", DataType::UInt32, true))),
                true,
            ),
            Field::new("token_count", DataType::UInt32, true),
        ]))
    }

    fn make_batch(
        &self,
        schema: Arc<Schema>,
        page_id: &str,
        chunks: &[ChunkInput],
        dim: i32,
    ) -> Result<RecordBatch, String> {
        let mut chunk_ids: Vec<String> = Vec::with_capacity(chunks.len());
        let mut page_ids: Vec<String> = Vec::with_capacity(chunks.len());
        let mut indexes: Vec<u32> = Vec::with_capacity(chunks.len());
        let mut texts: Vec<String> = Vec::with_capacity(chunks.len());
        let mut heading_paths: Vec<String> = Vec::with_capacity(chunks.len());
        let mut flat_vectors: Vec<f32> = Vec::with_capacity(chunks.len() * dim as usize);

        for c in chunks {
            if c.embedding.len() as i32 != dim {
                return Err(format!(
                    "Chunk #{} has embedding dim {} but batch dim is {}",
                    c.chunk_index,
                    c.embedding.len(),
                    dim
                ));
            }
            chunk_ids.push(format!("{}#{}", page_id, c.chunk_index));
            page_ids.push(page_id.to_string());
            indexes.push(c.chunk_index);
            texts.push(c.chunk_text.clone());
            heading_paths.push(c.heading_path.clone());
            flat_vectors.extend_from_slice(&c.embedding);
        }

        let chunk_ids_arr: ArrayRef = Arc::new(StringArray::from(chunk_ids));
        let page_ids_arr: ArrayRef = Arc::new(StringArray::from(page_ids));
        let indexes_arr: ArrayRef = Arc::new(UInt32Array::from(indexes));
        let texts_arr: ArrayRef = Arc::new(StringArray::from(texts));
        let heading_paths_arr: ArrayRef = Arc::new(StringArray::from(heading_paths));

        let values = Float32Array::from(flat_vectors);
        let vector_arr: ArrayRef = Arc::new(FixedSizeListArray::new(
            Arc::new(Field::new("item", DataType::Float32, true)),
            dim,
            Arc::new(values),
            None,
        ));

        let token_ids_arr: ArrayRef = Arc::new(new_null_array(&DataType::List(
            Arc::new(Field::new("item", DataType::UInt32, true))
        ), chunks.len()));
        let token_count_arr: ArrayRef = Arc::new(new_null_array(&DataType::UInt32, chunks.len()));

        RecordBatch::try_new(
            schema,
            vec![
                chunk_ids_arr,
                page_ids_arr,
                indexes_arr,
                texts_arr,
                heading_paths_arr,
                vector_arr,
                token_ids_arr,
                token_count_arr,
            ],
        )
        .map_err(|e| format!("Batch error: {e}"))
    }
}

#[async_trait::async_trait]
impl VectorStorage for LanceDbStorage {
    async fn upsert_chunks(
        &self,
        page_id: &str,
        chunks: Vec<ChunkInput>,
    ) -> StorageResult<()> {
        let page_id = page_id.to_string();
        let result = run_guarded_async("vector_upsert_chunks", async move {
            self.validate_page_id(&page_id)?;

            if chunks.is_empty() {
                return Ok(());
            }

            let dim = chunks[0].embedding.len() as i32;
            if dim == 0 {
                return Err("Chunk #0 has empty embedding".to_string());
            }

            let db = connect(&self.db_path())
                .execute()
                .await
                .map_err(|e| format!("DB connect error: {e}"))?;

            let schema = self.make_schema(dim);
            let batch = self.make_batch(schema.clone(), &page_id, &chunks, dim)?;
            let data = vec![batch];

            let tables = db
                .table_names()
                .execute()
                .await
                .map_err(|e| format!("List tables error: {e}"))?;

            if tables.contains(&TABLE_V2.to_string()) {
                let table = db
                    .open_table(TABLE_V2)
                    .execute()
                    .await
                    .map_err(|e| format!("Open table error: {e}"))?;

                if let Err(e) = table
                    .delete(&format!("page_id = '{}'", page_id))
                    .await
                {
                    eprintln!(
                        "[LanceDB] Warning: delete before upsert failed for page '{}': {}",
                        page_id, e
                    );
                }

                table
                    .add(data)
                    .execute()
                    .await
                    .map_err(|e| format!("Add error: {e}"))?;
            } else {
                db.create_table(TABLE_V2, data)
                    .execute()
                    .await
                    .map_err(|e| format!("Create table error: {e}"))?;
            }

            Ok(())
        })
        .await;
        
        result.map_err(StorageError::new)
    }

    async fn search(
        &self,
        query_embedding: Vec<f32>,
        top_k: usize,
    ) -> StorageResult<Vec<SearchResult>> {
        let result = run_guarded_async("vector_search_chunks", async move {
            let db = connect(&self.db_path())
                .execute()
                .await
                .map_err(|e| format!("DB connect error: {e}"))?;

            let tables = db
                .table_names()
                .execute()
                .await
                .map_err(|e| format!("List tables error: {e}"))?;

            if !tables.contains(&TABLE_V2.to_string()) {
                return Ok(vec![]);
            }

            let table = db
                .open_table(TABLE_V2)
                .execute()
                .await
                .map_err(|e| format!("Open table error: {e}"))?;

            let results_stream = table
                .vector_search(query_embedding)
                .map_err(|e| format!("Search error: {e}"))?
                .limit(top_k)
                .execute()
                .await
                .map_err(|e| format!("Execute search error: {e}"))?;

            use futures::TryStreamExt;
            let batches: Vec<RecordBatch> = results_stream
                .try_collect()
                .await
                .map_err(|e| format!("Collect error: {e}"))?;

            let mut out: Vec<SearchResult> = Vec::new();
            for batch in &batches {
                let chunk_ids = batch
                    .column_by_name("chunk_id")
                    .and_then(|c| c.as_any().downcast_ref::<StringArray>())
                    .ok_or("Missing chunk_id column")?;
                let page_ids = batch
                    .column_by_name("page_id")
                    .and_then(|c| c.as_any().downcast_ref::<StringArray>())
                    .ok_or("Missing page_id column")?;
                let chunk_indexes = batch
                    .column_by_name("chunk_index")
                    .and_then(|c| c.as_any().downcast_ref::<UInt32Array>())
                    .ok_or("Missing chunk_index column")?;
                let chunk_texts = batch
                    .column_by_name("chunk_text")
                    .and_then(|c| c.as_any().downcast_ref::<StringArray>())
                    .ok_or("Missing chunk_text column")?;
                let heading_paths = batch
                    .column_by_name("heading_path")
                    .and_then(|c| c.as_any().downcast_ref::<StringArray>())
                    .ok_or("Missing heading_path column")?;
                let distances = batch
                    .column_by_name("_distance")
                    .and_then(|c| c.as_any().downcast_ref::<Float32Array>())
                    .ok_or("Missing _distance column")?;

                for i in 0..batch.num_rows() {
                    let distance = distances.value(i);
                    out.push(SearchResult {
                        chunk_id: chunk_ids.value(i).to_string(),
                        page_id: page_ids.value(i).to_string(),
                        chunk_index: chunk_indexes.value(i),
                        chunk_text: chunk_texts.value(i).to_string(),
                        heading_path: heading_paths.value(i).to_string(),
                        score: 1.0 / (1.0 + distance),
                        token_ids: None,
                        token_count: None,
                    });
                }
            }

            Ok(out)
        })
        .await;
        
        result.map_err(StorageError::new)
    }

    async fn delete_page(&self, page_id: &str) -> StorageResult<()> {
        let page_id = page_id.to_string();
        let result = run_guarded_async("vector_delete_page", async move {
            self.validate_page_id(&page_id)?;

            let db = connect(&self.db_path())
                .execute()
                .await
                .map_err(|e| format!("DB connect error: {e}"))?;

            let tables = db
                .table_names()
                .execute()
                .await
                .map_err(|e| format!("List tables error: {e}"))?;

            if !tables.contains(&TABLE_V2.to_string()) {
                return Ok(());
            }

            let table = db
                .open_table(TABLE_V2)
                .execute()
                .await
                .map_err(|e| format!("Open table error: {e}"))?;

            table
                .delete(&format!("page_id = '{}'", page_id))
                .await
                .map_err(|e| format!("Delete error: {e}"))?;

            Ok(())
        })
        .await;
        
        result.map_err(StorageError::new)
    }

    async fn count(&self) -> StorageResult<usize> {
        let result = run_guarded_async("vector_count_chunks", async move {
            let db = connect(&self.db_path())
                .execute()
                .await
                .map_err(|e| format!("DB connect error: {e}"))?;

            let tables = db
                .table_names()
                .execute()
                .await
                .map_err(|e| format!("List tables error: {e}"))?;

            if !tables.contains(&TABLE_V2.to_string()) {
                return Ok(0);
            }

            let table = db
                .open_table(TABLE_V2)
                .execute()
                .await
                .map_err(|e| format!("Open table error: {e}"))?;

            let count = table
                .count_rows(None)
                .await
                .map_err(|e| format!("Count error: {e}"))?;

            Ok(count)
        })
        .await;
        
        result.map_err(StorageError::new)
    }

    fn embedding_dim(&self) -> usize {
        1536
    }
}

#[cfg(test)]
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
        let p = std::env::temp_dir().join(format!("llm-wiki-vtest-{}-{}", ts, id));
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
    async fn v2_upsert_then_count() {
        let p = tmp_project();
        let pp = p.to_string_lossy().to_string();
        let storage = LanceDbStorage::new(pp.clone());

        let chunks = make_chunks("my-page", 3, 16);
        storage.upsert_chunks("my-page".into(), chunks).await.unwrap();

        let count = storage.count().await.unwrap();
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn v2_upsert_replaces_existing_chunks_for_page() {
        let p = tmp_project();
        let pp = p.to_string_lossy().to_string();
        let storage = LanceDbStorage::new(pp.clone());

        storage.upsert_chunks("page-a".into(), make_chunks("page-a", 5, 16)).await.unwrap();
        assert_eq!(storage.count().await.unwrap(), 5);

        storage.upsert_chunks("page-a".into(), make_chunks("page-a", 2, 16)).await.unwrap();
        assert_eq!(storage.count().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn v2_different_pages_coexist() {
        let p = tmp_project();
        let pp = p.to_string_lossy().to_string();
        let storage = LanceDbStorage::new(pp.clone());

        storage.upsert_chunks("page-a".into(), make_chunks("page-a", 3, 16)).await.unwrap();
        storage.upsert_chunks("page-b".into(), make_chunks("page-b", 4, 16)).await.unwrap();

        assert_eq!(storage.count().await.unwrap(), 7);
    }

    #[tokio::test]
    async fn v2_delete_page_removes_only_its_chunks() {
        let p = tmp_project();
        let pp = p.to_string_lossy().to_string();
        let storage = LanceDbStorage::new(pp.clone());

        storage.upsert_chunks("page-a".into(), make_chunks("page-a", 3, 16)).await.unwrap();
        storage.upsert_chunks("page-b".into(), make_chunks("page-b", 2, 16)).await.unwrap();
        assert_eq!(storage.count().await.unwrap(), 5);

        storage.delete_page("page-a".into()).await.unwrap();
        assert_eq!(storage.count().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn v2_search_returns_chunks_with_metadata() {
        let p = tmp_project();
        let pp = p.to_string_lossy().to_string();
        let storage = LanceDbStorage::new(pp.clone());

        storage.upsert_chunks("page-a".into(), make_chunks("page-a", 3, 16)).await.unwrap();

        let query = fake_embedding(1, 16);
        let results = storage.search(query, 10).await.unwrap();
        assert!(!results.is_empty());
        for r in &results {
            assert_eq!(r.page_id, "page-a");
            assert!(r.chunk_id.starts_with("page-a#"));
            assert!(r.chunk_text.contains("chunk"));
            assert!(r.heading_path.starts_with("## Heading"));
        }
    }
}
