# 3-Layer Architecture Refactoring Plan

**Status:** Design Phase  
**Created:** 2026-05-02  
**Purpose:** Prepare codebase for future RuVector migration by establishing clean layer boundaries

---

## Executive Summary

Current codebase has implicit layering but lacks clear boundaries. This refactoring establishes explicit 3-layer architecture:

- **Layer 1 (Interface):** HTTP API + CLI entry points
- **Layer 2 (Business Logic):** Token cache, file parsing, embedding generation, ingest orchestration
- **Layer 3 (Data):** Vector storage abstraction (currently LanceDB, future RuVector)

**Key Goals:**
1. Zero functional changes - pure refactoring
2. Trait-based storage abstraction for easy backend swapping
3. Dependency injection for testability
4. Clear module boundaries with explicit contracts

---

## Current State Analysis

### Layer 1: Interface (Entry Points)

**HTTP API (5 endpoints):**
- `GET /health` → Health check
- `POST /api/llm/stream` → LLM chat streaming
- `POST /api/ingest` → Document ingestion
- `POST /api/config/get` → Load project config
- `POST /api/config/save` → Save project config

**CLI (4 commands):**
- `serve --host --port` → Start API server
- `init <path> --template` → Initialize wiki project
- `ingest <file> --project` → Ingest document
- `query <query> --project` → Query knowledge base

**Status:** ⚠️ All handlers are placeholders, no service wiring exists

**Critical Gap:** Handlers have no access to Layer 2 services (no dependency injection)

### Layer 2: Business Logic (Services)

**Existing Services:**
- `token_cache.rs` (270 lines) - Token pre-computation with tiktoken-rs
  - `TokenCacheService::tokenize_chunk()` → Vec<usize>
  - `QueryOptimizer::optimize_context()` → Budget-aware selection
- `ingest_engine.rs` (236 lines) - FILE block parser
  - `parse_file_blocks()` → ParsedFileBlock[]
  - `is_safe_ingest_path()` → Security validation
- `llm_client.rs` (116 lines) - LLM streaming client
  - `stream_chat()` → SSE stream

**Missing Services:**
- ❌ Embedding generation (no Rust implementation)
- ❌ Chunk extraction (markdown heading-based splitting)
- ❌ Ingest orchestration (end-to-end pipeline)
- ❌ Query orchestration (search + rerank + context assembly)

**Status:** Services exist but isolated, no integration

### Layer 3: Data (Storage)

**Current Implementation:** `lancedb.rs` (489 lines)

**Public API (4 methods):**
```rust
pub async fn vector_upsert_chunks(project_path, page_id, chunks) -> Result<(), String>
pub async fn vector_search_chunks(project_path, query_embedding, top_k) -> Result<Vec<ChunkSearchResult>, String>
pub async fn vector_delete_page(project_path, page_id) -> Result<(), String>
pub async fn vector_count_chunks(project_path) -> Result<usize, String>
```

**Schema V2 (8 fields):**
- Core: `chunk_id`, `page_id`, `chunk_index`, `chunk_text`, `heading_path`
- Vector: `vector` (FixedSizeList<Float32>)
- Token cache: `token_ids` (List<UInt32>), `token_count` (UInt32) - **currently NULL**

**Status:** ✅ Functional but hardcoded to LanceDB, no abstraction

---

## Target Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    LAYER 1: INTERFACE                           │
├─────────────────────────────────────────────────────────────────┤
│  HTTP API (Axum)              CLI (Clap)                        │
│  ├─ GET  /health              ├─ serve                          │
│  ├─ POST /api/llm/stream      ├─ init                           │
│  ├─ POST /api/ingest          ├─ ingest                         │
│  ├─ POST /api/config/get      └─ query                          │
│  └─ POST /api/config/save                                       │
│                                                                 │
│  Responsibilities:                                              │
│  • Parse requests, validate inputs                             │
│  • Call Layer 2 services                                       │
│  • Format responses, handle errors                             │
│  • NO business logic                                           │
└─────────────────────────────────────────────────────────────────┘
                             ↓ (Service calls)
┌─────────────────────────────────────────────────────────────────┐
│                 LAYER 2: BUSINESS LOGIC                         │
├─────────────────────────────────────────────────────────────────┤
│  Services (Orchestration)                                       │
│  ├─ IngestService                                              │
│  │  └─ parse → chunk → embed → tokenize → store               │
│  ├─ QueryService                                               │
│  │  └─ embed query → search → optimize context → format       │
│  ├─ TokenCacheService (existing)                               │
│  │  └─ tokenize_chunk(), optimize_context()                   │
│  ├─ EmbeddingService (NEW)                                     │
│  │  └─ generate_embeddings() via OpenAI-compatible API        │
│  ├─ ChunkingService (NEW)                                      │
│  │  └─ split_by_headings() for markdown                       │
│  └─ LLMService (existing llm_client.rs)                        │
│     └─ stream_chat()                                           │
│                                                                 │
│  Responsibilities:                                              │
│  • Orchestrate multi-step workflows                            │
│  • Business logic (chunking, token optimization)               │
│  • Call Layer 3 for persistence                                │
│  • NO direct storage access                                    │
└─────────────────────────────────────────────────────────────────┘
                             ↓ (Trait calls)
┌─────────────────────────────────────────────────────────────────┐
│                    LAYER 3: DATA                                │
├─────────────────────────────────────────────────────────────────┤
│  VectorStorage Trait (abstraction)                              │
│  ├─ upsert_chunks()                                            │
│  ├─ search()                                                   │
│  ├─ delete_page()                                              │
│  └─ count()                                                    │
│                                                                 │
│  Implementations:                                               │
│  ├─ LanceDBStorage (current)                                   │
│  └─ RuVectorStorage (future)                                   │
│                                                                 │
│  Responsibilities:                                              │
│  • CRUD operations                                             │
│  • Index management                                            │
│  • Query execution                                             │
│  • NO business logic                                           │
└─────────────────────────────────────────────────────────────────┘
```

---

## Refactoring Phases

### Phase 1: Extract Storage Trait (Week 1, Day 1-2)

**Goal:** Abstract LanceDB behind trait for future RuVector swap

**Tasks:**
1. Create `src/storage/traits.rs`:
   ```rust
   #[async_trait]
   pub trait VectorStorage: Send + Sync {
       async fn upsert_chunks(&self, page_id: &str, chunks: Vec<ChunkInput>) -> Result<()>;
       async fn search(&self, query: Vec<f32>, top_k: usize) -> Result<Vec<SearchResult>>;
       async fn delete_page(&self, page_id: &str) -> Result<()>;
       async fn count(&self) -> Result<usize>;
   }
   ```

2. Refactor `lancedb.rs` → `lancedb_impl.rs`:
   ```rust
   pub struct LanceDBStorage {
       project_path: String,
   }
   
   #[async_trait]
   impl VectorStorage for LanceDBStorage {
       // Implement trait methods
   }
   ```

3. Update `src/storage/mod.rs`:
   ```rust
   pub mod traits;
   pub mod lancedb_impl;
   
   pub use traits::VectorStorage;
   pub use lancedb_impl::LanceDBStorage;
   ```

**Verification:**
- All existing tests pass
- No functional changes
- `cargo build --release` succeeds

### Phase 2: Add Missing Services (Week 1, Day 3-5)

**Goal:** Implement missing Layer 2 services

**2.1 EmbeddingService (NEW)**
```rust
// src/services/embedding.rs
pub struct EmbeddingService {
    client: reqwest::Client,
    api_base: String,
    api_key: String,
    model: String,
}

impl EmbeddingService {
    pub async fn generate_embeddings(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        // Call OpenAI-compatible /v1/embeddings endpoint
    }
}
```

**2.2 ChunkingService (NEW)**
```rust
// src/services/chunking.rs
pub struct ChunkingService {
    max_chunk_size: usize,
    overlap: usize,
}

impl ChunkingService {
    pub fn split_by_headings(&self, markdown: &str) -> Vec<Chunk> {
        // Parse markdown, split by ## headings
        // Track heading_path breadcrumb
    }
}
```

**2.3 IngestService (Orchestrator)**
```rust
// src/services/ingest.rs
pub struct IngestService {
    storage: Arc<dyn VectorStorage>,
    chunking: ChunkingService,
    embedding: EmbeddingService,
    token_cache: TokenCacheService,
}

impl IngestService {
    pub async fn ingest_file(&self, page_id: &str, content: &str) -> Result<IngestStats> {
        let chunks = self.chunking.split_by_headings(content);
        let texts: Vec<_> = chunks.iter().map(|c| c.text.clone()).collect();
        let embeddings = self.embedding.generate_embeddings(texts).await?;
        let tokenized = self.token_cache.tokenize_chunks_batch(&chunks);
        
        let inputs: Vec<_> = chunks.into_iter()
            .zip(embeddings)
            .zip(tokenized)
            .map(|((chunk, emb), tok)| ChunkInput {
                chunk_index: chunk.index,
                chunk_text: chunk.text,
                heading_path: chunk.heading_path,
                embedding: emb,
                token_ids: Some(tok.token_ids),
                token_count: Some(tok.token_count),
            })
            .collect();
        
        self.storage.upsert_chunks(page_id, inputs).await?;
        Ok(IngestStats { chunks_processed: inputs.len() })
    }
}
```

**2.4 QueryService (Orchestrator)**
```rust
// src/services/query.rs
pub struct QueryService {
    storage: Arc<dyn VectorStorage>,
    embedding: EmbeddingService,
    token_cache: TokenCacheService,
}

impl QueryService {
    pub async fn query(&self, query_text: &str, top_k: usize, token_budget: usize) -> Result<QueryResult> {
        let query_emb = self.embedding.generate_embeddings(vec![query_text.to_string()]).await?;
        let results = self.storage.search(query_emb[0].clone(), top_k * 3).await?; // 3x for reranking
        
        let tokenized: Vec<_> = results.iter()
            .map(|r| TokenizedChunk {
                chunk_id: r.chunk_id.clone(),
                page_id: r.page_id.clone(),
                chunk_text: r.chunk_text.clone(),
                token_ids: vec![], // Load from storage if available
                token_count: self.token_cache.count_tokens(&r.chunk_text),
            })
            .collect();
        
        let optimized = self.token_cache.optimize_context(tokenized, token_budget);
        
        Ok(QueryResult {
            chunks: optimized.into_iter().take(top_k).collect(),
            total_tokens: optimized.iter().map(|c| c.token_count).sum(),
        })
    }
}
```

**Verification:**
- Unit tests for each service
- Integration test: ingest → query roundtrip
- Token cache fields populated in LanceDB

### Phase 3: Wire Services to Handlers (Week 2, Day 1-2)

**Goal:** Connect Layer 1 (handlers) to Layer 2 (services)

**3.1 Create AppState**
```rust
// src/api/state.rs
pub struct AppState {
    pub ingest_service: Arc<IngestService>,
    pub query_service: Arc<QueryService>,
    pub llm_service: Arc<LLMService>,
    pub config_service: Arc<ConfigService>,
}

impl AppState {
    pub fn new(config: Config) -> Result<Self> {
        let storage: Arc<dyn VectorStorage> = Arc::new(LanceDBStorage::new(config.project_path)?);
        let embedding = EmbeddingService::new(config.embedding_api);
        let token_cache = TokenCacheService::new();
        let chunking = ChunkingService::new(config.chunk_size, config.overlap);
        
        Ok(Self {
            ingest_service: Arc::new(IngestService::new(storage.clone(), chunking, embedding.clone(), token_cache.clone())),
            query_service: Arc::new(QueryService::new(storage, embedding, token_cache)),
            llm_service: Arc::new(LLMService::new(config.llm_api)),
            config_service: Arc::new(ConfigService::new()),
        })
    }
}
```

**3.2 Update Handlers**
```rust
// src/api/handlers.rs
pub async fn ingest_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<IngestRequest>,
) -> Result<Json<IngestResponse>, StatusCode> {
    let stats = state.ingest_service
        .ingest_file(&req.page_id, &req.content)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(IngestResponse {
        success: true,
        chunks_processed: stats.chunks_processed,
    }))
}

pub async fn query_wiki(
    State(state): State<Arc<AppState>>,
    Json(req): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, StatusCode> {
    let result = state.query_service
        .query(&req.query, req.top_k, req.token_budget)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(QueryResponse {
        chunks: result.chunks,
        total_tokens: result.total_tokens,
    }))
}
```

**3.3 Update Server Initialization**
```rust
// src/api/server.rs
pub async fn start_api_server(port: u16) -> Result<()> {
    let config = Config::load()?;
    let state = Arc::new(AppState::new(config)?);
    
    let app = create_router()
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());
    
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}
```

**Verification:**
- `curl http://localhost:19828/health` → 200 OK
- `curl -X POST http://localhost:19828/api/ingest -d '{"page_id":"test","content":"# Test\nContent"}'` → Success
- `curl -X POST http://localhost:19828/api/query -d '{"query":"test","top_k":5,"token_budget":2000}'` → Results

### Phase 4: Update CLI (Week 2, Day 3)

**Goal:** Wire CLI commands to services (reuse same services as HTTP API)

```rust
// src/cli.rs
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::load()?;
    let state = AppState::new(config)?;
    
    match args.command {
        Command::Serve { host, port } => {
            start_api_server(port).await?;
        }
        Command::Init { path, template } => {
            state.config_service.init_project(&path, &template)?;
        }
        Command::Ingest { file, project } => {
            let content = fs::read_to_string(&file)?;
            let stats = state.ingest_service.ingest_file(&file, &content).await?;
            println!("✅ Ingested {} chunks", stats.chunks_processed);
        }
        Command::Query { query, project } => {
            let result = state.query_service.query(&query, 5, 2000).await?;
            for chunk in result.chunks {
                println!("📄 {} (score: {:.2})", chunk.page_id, chunk.score);
                println!("   {}", chunk.chunk_text.lines().next().unwrap_or(""));
            }
        }
    }
    
    Ok(())
}
```

**Verification:**
- `llm-wiki init .wiki/test` → Creates project structure
- `llm-wiki ingest test.md` → Ingests file
- `llm-wiki query "test query"` → Returns results
- `llm-wiki serve --port 19828` → Starts server

---

## Migration Path to RuVector

Once 3-layer architecture is in place, RuVector migration becomes trivial:

```rust
// src/storage/ruvector_impl.rs
pub struct RuVectorStorage {
    db: RuVectorDB,
}

#[async_trait]
impl VectorStorage for RuVectorStorage {
    async fn upsert_chunks(&self, page_id: &str, chunks: Vec<ChunkInput>) -> Result<()> {
        // Use RuVector API
    }
    
    async fn search(&self, query: Vec<f32>, top_k: usize) -> Result<Vec<SearchResult>> {
        // Use RuVector HNSW + SONA
    }
    
    // ... other methods
}
```

**Switch storage backend:**
```rust
// src/api/state.rs
let storage: Arc<dyn VectorStorage> = if config.use_ruvector {
    Arc::new(RuVectorStorage::new(config.project_path)?)
} else {
    Arc::new(LanceDBStorage::new(config.project_path)?)
};
```

**Zero changes to Layer 1 (handlers) or Layer 2 (services).**

---

## Success Metrics

### Functional Requirements
- ✅ All 5 HTTP endpoints functional (not placeholders)
- ✅ All 4 CLI commands functional
- ✅ End-to-end ingest → query pipeline works
- ✅ Token cache fields populated in LanceDB
- ✅ Embedding generation integrated

### Non-Functional Requirements
- ✅ Query latency ≤ 150ms (current target)
- ✅ Token cache hit rate = 100%
- ✅ Zero breaking changes to existing API contracts
- ✅ All tests pass (unit + integration)

### Architecture Quality
- ✅ Clear layer boundaries (no cross-layer dependencies)
- ✅ Storage abstraction allows backend swap in <1 day
- ✅ Services testable in isolation (dependency injection)
- ✅ No circular dependencies

---

## Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Breaking existing API contracts | Low | High | Preserve all request/response types, add integration tests |
| Performance regression | Medium | High | Benchmark before/after, keep LanceDB optimizations |
| Embedding API failures | Medium | Medium | Add retry logic, fallback to cached embeddings |
| Token cache memory bloat | Low | Medium | Add LRU eviction policy, configurable cache size |
| RuVector migration complexity | Low | High | Trait abstraction ensures clean swap, prototype first |

---

## Timeline

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| Phase 1: Storage Trait | 2 days | `VectorStorage` trait, `LanceDBStorage` impl |
| Phase 2: Missing Services | 3 days | `EmbeddingService`, `ChunkingService`, `IngestService`, `QueryService` |
| Phase 3: Wire Handlers | 2 days | `AppState`, updated handlers, working HTTP API |
| Phase 4: Update CLI | 1 day | Functional CLI commands |
| **Total** | **8 days** | **Production-ready 3-layer architecture** |

---

## Next Steps

1. **Review this plan** - Confirm architecture aligns with vision
2. **Approve Phase 1** - Start with storage trait extraction (lowest risk)
3. **Prototype embedding service** - Validate OpenAI-compatible API integration
4. **Create tracking issues** - Break down into GitHub issues for progress tracking

**Decision Point:** Proceed with Phase 1, or adjust architecture first?
