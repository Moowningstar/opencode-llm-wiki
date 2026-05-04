use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::path::PathBuf;

#[cfg(feature = "ruvector")]
use llm_wiki_server::storage::ruvector_impl::RuVectorStorage;

#[cfg(feature = "lancedb-backend")]
use llm_wiki_server::storage::lancedb_impl::LanceDbStorage;

use llm_wiki_server::storage::traits::{VectorStorage, ChunkInput};

fn tmp_project() -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let p = std::env::temp_dir().join(format!("llm-wiki-bench-{}-{}", ts, id));
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
            chunk_text: format!("{} chunk {} with some longer text to simulate real content", page_id, i),
            heading_path: format!("## Heading {}", i),
            embedding: fake_embedding(i, dim),
            token_ids: None,
            token_count: None,
        })
        .collect()
}

#[cfg(feature = "ruvector")]
fn bench_ruvector_upsert(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("ruvector_upsert_10_chunks", |b| {
        b.iter(|| {
            rt.block_on(async {
                let p = tmp_project();
                let storage = RuVectorStorage::new(p.to_string_lossy().to_string(), 128).await.unwrap();
                let chunks = make_chunks("test-page", 10, 128);
                storage.upsert_chunks("test-page", black_box(chunks)).await.unwrap();
            });
        });
    });
}

#[cfg(feature = "ruvector")]
fn bench_ruvector_search(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let p = tmp_project();
    let storage = rt.block_on(async {
        let s = RuVectorStorage::new(p.to_string_lossy().to_string(), 128).await.unwrap();
        for i in 0..100 {
            let chunks = make_chunks(&format!("page-{}", i), 5, 128);
            s.upsert_chunks(&format!("page-{}", i), chunks).await.unwrap();
        }
        s
    });
    
    c.bench_function("ruvector_search_top10", |b| {
        b.iter(|| {
            rt.block_on(async {
                let query = fake_embedding(42, 128);
                storage.search(black_box(query), 10).await.unwrap();
            });
        });
    });
}

#[cfg(feature = "lancedb-backend")]
fn bench_lancedb_upsert(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("lancedb_upsert_10_chunks", |b| {
        b.iter(|| {
            rt.block_on(async {
                let p = tmp_project();
                let storage = LanceDbStorage::new(p.to_string_lossy().to_string());
                let chunks = make_chunks("test-page", 10, 128);
                storage.upsert_chunks("test-page", black_box(chunks)).await.unwrap();
            });
        });
    });
}

#[cfg(feature = "lancedb-backend")]
fn bench_lancedb_search(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let p = tmp_project();
    let storage = rt.block_on(async {
        let s = LanceDbStorage::new(p.to_string_lossy().to_string());
        for i in 0..100 {
            let chunks = make_chunks(&format!("page-{}", i), 5, 128);
            s.upsert_chunks(&format!("page-{}", i), chunks).await.unwrap();
        }
        s
    });
    
    c.bench_function("lancedb_search_top10", |b| {
        b.iter(|| {
            rt.block_on(async {
                let query = fake_embedding(42, 128);
                storage.search(black_box(query), 10).await.unwrap();
            });
        });
    });
}

#[cfg(feature = "ruvector")]
criterion_group!(ruvector_benches, bench_ruvector_upsert, bench_ruvector_search);

#[cfg(feature = "lancedb-backend")]
criterion_group!(lancedb_benches, bench_lancedb_upsert, bench_lancedb_search);

#[cfg(feature = "ruvector")]
criterion_main!(ruvector_benches);

#[cfg(all(feature = "lancedb-backend", not(feature = "ruvector")))]
criterion_main!(lancedb_benches);

#[cfg(not(any(feature = "ruvector", feature = "lancedb-backend")))]
fn main() {
    eprintln!("Error: Either 'ruvector' or 'lancedb-backend' feature must be enabled");
    std::process::exit(1);
}
