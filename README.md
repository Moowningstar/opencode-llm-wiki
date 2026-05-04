# OpenCode LLM Wiki

<p align="center">
  <img src="logo.jpg" width="128" height="128" style="border-radius: 22%;" alt="OpenCode LLM Wiki Logo">
</p>

<p align="center">
  <strong>Knowledge Graph Backend with Vector Search & Graph Algorithms</strong><br>
  HTTP API • CLI Tools • MCP Protocol • Vector Search • Knowledge Graph • Graph Analytics
</p>

<p align="center">
  <a href="#what-is-this">What is this?</a> •
  <a href="#architecture">Architecture</a> •
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#license">License</a>
</p>

<p align="center">
  English | <a href="README_ZH.md">中文</a>
</p>

---

## What is this?

**OpenCode LLM Wiki** is a **knowledge graph backend** that provides persistent, queryable knowledge storage with multiple access interfaces. It maintains a structured wiki with vector search and graph algorithms, accessible via HTTP API, CLI tools, or AI agents through the Model Context Protocol (MCP).

### Core Philosophy

**Persistent Knowledge Engine for AI Agents and Developers**

This is a knowledge graph backend that provides multiple access methods for storing, indexing, and retrieving structured knowledge. Unlike ephemeral RAG systems that forget everything after each conversation, this project provides:

1. **Persistent Wiki Storage**: Markdown files in `.wiki/pages/` with metadata-driven indexing
2. **Multiple Interfaces**: HTTP API, CLI tools, and MCP protocol for different use cases
3. **Knowledge Graph**: Automatic link extraction and relationship mapping with graph algorithms
4. **Vector Search**: RuVector-powered semantic search with 2048-dimensional embeddings
5. **Cross-Session Memory**: Knowledge persists across sessions, conversations, and tools
6. **Graph Algorithms**: PageRank, community detection, shortest paths, centrality analysis

### Use Cases

- **AI Agent Memory**: Persistent context that survives across conversations (via MCP)
- **Codebase Documentation**: Living architecture docs queryable through API or CLI
- **Project Knowledge Base**: Store decisions, patterns, and tribal knowledge
- **Research Notes**: Organize papers, articles, and findings with semantic search
- **Personal Wiki**: Build a second brain accessible through multiple interfaces

---

## Architecture

### Three-Layer Knowledge Engine

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: Interface Layer (Multiple Access Points)           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │  HTTP API    │  │     CLI      │  │  MCP Server  │     │
│  │  (Axum)      │  │   (clap)     │  │  (Node.js)   │     │
│  │  Port 19828  │  │              │  │  stdio       │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│  • Request validation and routing                          │
│  • Response formatting                                     │
│  • No business logic                                       │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 2: Indexing & Retrieval (Rust Backend)                │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Wiki File System                                     │   │
│  │  • WikiFileSystem: .wiki/pages/ management          │   │
│  │  • IndexManager: index.json metadata                │   │
│  │  • GraphManager: graph.json relationships           │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Search & Retrieval                                   │   │
│  │  • Keyword search (tokenized)                        │   │
│  │  • Semantic search (vector embeddings)               │   │
│  │  • Graph traversal (BFS, shortest path)             │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Graph Algorithms                                     │   │
│  │  • PageRank (influence scoring)                      │   │
│  │  • Louvain (community detection)                     │   │
│  │  • Centrality (degree, betweenness)                  │   │
│  │  • Dijkstra (shortest paths)                         │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Document Processing                                  │   │
│  │  • Ingest pipeline (parse → chunk → embed → store)  │   │
│  │  • Markdown chunking (heading-aware)                 │   │
│  │  • Metadata extraction                               │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 3: Storage Layer (RuVector Backend)                   │
│  ┌──────────────────┐              ┌──────────────────┐    │
│  │  File System     │              │  Vector DB       │    │
│  │  .wiki/pages/    │              │  RuVector 2.2.0  │    │
│  │  index.json      │              │  (2048-dim)      │    │
│  │  graph.json      │              │                  │    │
│  └──────────────────┘              └──────────────────┘    │
│  • VectorStorage trait abstraction                         │
│  • RuVector: Vector + Graph + GNN unified storage          │
└─────────────────────────────────────────────────────────────┘
```

**Key Design Principles:**

- **Multiple Interfaces**: HTTP API for programmatic access, CLI for automation, MCP for AI agents
- **Metadata-Driven**: index.json manages page metadata, graph.json stores relationships
- **Pluggable Storage**: VectorStorage trait enables easy backend migration
- **Clean Separation**: Interface layer has no business logic, storage layer has no retrieval logic

---

## Features

### Core Backend (Production Rust)

- ✅ **3-Layer Architecture** — Clean separation: Interface → Services → Storage
- ✅ **RuVector Integration** — Unified vector + graph + GNN storage (2048-dimensional embeddings)
- ✅ **Graph Algorithms** — PageRank, Louvain community detection, Dijkstra shortest paths, centrality analysis
- ✅ **Vector Semantic Search** — Fast ANN retrieval with cosine similarity
- ✅ **Knowledge Graph** — Automatic link extraction, relationship mapping, graph traversal
- ✅ **Markdown-Aware Chunking** — Heading-path preservation, configurable overlap
- ✅ **Multi-Provider LLM** — OpenAI, Anthropic, OpenRouter, custom endpoints
- ✅ **HTTP API + CLI** — Axum server (port 19828) + standalone CLI tools
- ✅ **MCP Protocol** — 11 tools for AI agent integration
- ✅ **Async-First Design** — tokio runtime, non-blocking I/O

### Graph Analytics (Phase 2)

- ✅ **PageRank** — Identify influential pages (damping factor 0.85, 100 iterations)
- ✅ **Community Detection** — Louvain algorithm for topic clustering (modularity optimization)
- ✅ **Centrality Metrics** — Degree centrality, betweenness centrality for bridge node detection
- ✅ **Shortest Paths** — Dijkstra algorithm with path reconstruction
- ✅ **Graph Insights API** — Isolated pages, bridge nodes, graph statistics
- ✅ **Deep Research** — Multi-hop semantic search with BFS traversal
- ✅ **Cypher Queries** — Basic graph query support (MATCH, CREATE, RETURN)

---

## MCP Tools (11 Available)

| Tool | Purpose |
|------|---------|
| `wiki_read` | Read a single wiki page by path |
| `wiki_list` | List all wiki pages with metadata |
| `wiki_search` | Keyword search across wiki content |
| `wiki_query_with_context` | Intelligent context injection (keyword + vector) |
| `wiki_get_graph` | Get knowledge graph (nodes and edges) |
| `wiki_graph_insights` | Analyze graph structure (PageRank, communities, centrality) |
| `wiki_deep_research` | Multi-hop reasoning with graph traversal |
| `wiki_get_index` | Get content catalog (index.md) |
| `wiki_get_overview` | Get global summary (overview.md) |
| `wiki_get_purpose` | Get wiki goals and scope (purpose.md) |
| `wiki_ingest` | Ingest documents into knowledge base |

---

## API Endpoints (16 Available)

### Document Management
- `POST /api/ingest` - Ingest documents into wiki
- `POST /api/pages` - List all wiki pages
- `POST /api/pages/read` - Read specific page content
- `POST /api/pages/search` - Keyword search across pages

### Graph Operations
- `POST /api/graph` - Get full knowledge graph
- `POST /api/graph/insights` - Graph analysis (PageRank, communities, centrality)
- `POST /api/research` - Deep research with multi-hop traversal

### Metadata
- `POST /api/meta/index` - Wiki page catalog
- `POST /api/meta/overview` - Graph statistics
- `POST /api/meta/purpose` - Wiki purpose and structure

### Search & Query
- `POST /api/search` - Semantic vector search
- `POST /api/query` - Query with intelligent context injection

### Health & Status
- `GET /health` - Health check endpoint

---

## Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **Storage** | RuVector 2.2.0 | Vector + Graph + GNN unified storage |
| **Embedding** | OpenRouter API | 2048-dim embeddings (nvidia/llama-nemotron) |
| **Chunking** | Custom Rust | Markdown heading-aware splitting |
| **HTTP Server** | Axum + tokio | Async Rust web framework |
| **CLI** | clap | Command-line interface |
| **MCP Server** | Node.js + @modelcontextprotocol/sdk | AI agent integration |
| **Graph Algorithms** | Pure Rust | PageRank, Louvain, Dijkstra, centrality |

---

## Installation

### Prerequisites

- **Rust** 1.70+ (for backend)
- **Node.js** 20+ (for MCP server)
- **API Key** (OpenRouter, OpenAI, or Anthropic for embeddings)

### Build from Source

```bash
# Clone repository
git clone https://github.com/yourusername/opencode-llm-wiki.git
cd opencode-llm-wiki

# Build Rust backend
cargo build --release

# Install MCP server dependencies
cd src-mcp
npm install
cd ..
```

### Configuration

Create `~/.config/opencode-llm-wiki/llm-wiki.jsonc`:

```jsonc
{
  "contextModel": "anthropic/claude-opus-4.6",
  "embeddingModel": "openrouter/nvidia/llama-nemotron-embed-vl-1b-v2:free",
  "embeddingDimension": 2048,
  
  "providers": {
    "openrouter": {
      "options": {
        "apiKey": "sk-or-v1-...",
        "baseURL": "https://openrouter.ai/api/v1"
      }
    },
    "anthropic": {
      "options": {
        "apiKey": "sk-ant-...",
        "baseURL": "https://api.anthropic.com/v1"
      }
    }
  },
  
  "storage": {
    "backend": "ruvector",
    "path": "./data/ruvector"
  }
}
```

---

## Quick Start

### 1. Start HTTP API Server

```bash
cargo run --release --bin llm-wiki-server
# Server listening on http://127.0.0.1:19828
```

### 2. Use CLI Tools

```bash
# Initialize new wiki
cargo run --release --bin llm-wiki -- init my-wiki

# Ingest documents
cargo run --release --bin llm-wiki -- ingest docs/ --recursive

# Query knowledge base
cargo run --release --bin llm-wiki -- query "vector database architecture"
```

### 3. Use MCP Server (with Claude Desktop)

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "llm-wiki": {
      "command": "node",
      "args": ["/path/to/opencode-llm-wiki/src-mcp/src/index.js"],
      "env": {
        "WIKI_API_URL": "http://127.0.0.1:19828"
      }
    }
  }
}
```

### 4. Use HTTP API

```bash
# Health check
curl http://127.0.0.1:19828/health

# List pages
curl -X POST http://127.0.0.1:19828/api/pages \
  -H "Content-Type: application/json" \
  -d '{"scope":"global"}'

# Graph insights
curl -X POST http://127.0.0.1:19828/api/graph/insights \
  -H "Content-Type: application/json" \
  -d '{"analysis_type":"stats","scope":"global"}'

# Deep research
curl -X POST http://127.0.0.1:19828/api/research \
  -H "Content-Type: application/json" \
  -d '{"query":"RuVector integration","max_depth":2,"max_results":5}'
```

---

## Project Structure

### Wiki Project Structure
```
my-wiki/
├── purpose.md              # Wiki goals and scope
├── .wiki/
│   ├── pages/              # All wiki pages (markdown)
│   └── _meta/
│       ├── index.json      # Page metadata
│       └── graph.json      # Knowledge graph
└── data/
    └── ruvector/           # Vector database storage
```

### Codebase Structure
```
opencode-llm-wiki/
├── src/                    # Rust backend (3-layer architecture)
│   ├── api/                # Layer 1: HTTP API + handlers
│   │   ├── state.rs        # AppState (dependency injection)
│   │   ├── handlers.rs     # Request handlers
│   │   ├── routes.rs       # Route definitions
│   │   └── server.rs       # Axum server
│   ├── services/           # Layer 2: Business logic
│   │   ├── embedding.rs    # Embedding API client
│   │   ├── chunking.rs     # Markdown chunking
│   │   ├── ingest.rs       # Document ingestion
│   │   ├── query.rs        # Search + context optimization
│   │   └── llm_client.rs   # Multi-provider LLM client
│   ├── storage/            # Layer 3: Data abstraction
│   │   ├── traits.rs       # VectorStorage trait
│   │   └── ruvector_impl.rs # RuVector implementation
│   ├── wiki/               # Wiki-specific logic
│   │   ├── filesystem.rs   # File system operations
│   │   ├── graph.rs        # Graph management
│   │   ├── graph_algorithms.rs # PageRank, Louvain, etc.
│   │   └── cypher.rs       # Cypher query engine
│   ├── types/              # Shared types
│   ├── main.rs             # API server binary
│   └── cli.rs              # CLI tools binary
├── src-mcp/                # MCP server (Node.js)
│   └── src/
│       ├── index.js        # MCP server entry point
│       ├── server.js       # Tool definitions
│       └── lib/
│           └── core-api-client.js # HTTP API client
├── docs/                   # Documentation
│   ├── RELEASE_v1.0.1.md   # Release notes
│   ├── V1.0.1_VERIFICATION.md # Verification report
│   └── RUVECTOR_PHASE2_COMPLETE.md # Phase 2 completion
└── benches/                # Performance benchmarks
```

---

## Credits & Inspiration

**Foundational Methodology**: [Andrej Karpathy's LLM Wiki Pattern](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f) — The core three-layer architecture (Raw Sources → Wiki → Schema) and incremental compilation philosophy.

**Vector Storage**: [RuVector](https://github.com/ruvector/ruvector) — Unified vector + graph + GNN storage backend with self-organizing neural architecture.

**What We Built**: A production Rust backend implementing Karpathy's pattern with graph-native design and advanced graph algorithms (PageRank, community detection, centrality analysis).

---

## Performance

### Benchmarks (v1.0.1)

- **Vector Search** (1000 vectors): ~2ms average
- **PageRank** (62 nodes): <50ms
- **Community Detection** (62 nodes): <100ms
- **BFS Traversal** (depth=3): <20ms
- **Betweenness Centrality** (62 nodes): <150ms
- **API Response Time**: 5-300ms (depends on operation)

### Test Coverage

- **Total Tests**: 61
- **Pass Rate**: 100%
- **Categories**: Storage layer, graph algorithms, API handlers, services

---

## Roadmap

### v1.1.0 (Planned)

- [ ] Advanced Cypher query support (WHERE, ORDER BY, aggregations)
- [ ] Real-time graph updates via WebSocket
- [ ] Multi-language embedding support
- [ ] Incremental indexing (avoid full re-ingestion)
- [ ] Graph visualization UI

### Future

- [ ] Desktop client (Tauri-based, separate repository)
- [ ] Web interface (React-based)
- [ ] Browser extension (web clipper)
- [ ] Docker deployment
- [ ] Kubernetes support

---

## License

This project is licensed under the **GNU General Public License v3.0** — see [LICENSE](LICENSE) for details.

---

## Star History

<a href="https://www.star-history.com/?repos=yourusername%2Fopencode-llm-wiki&type=date&legend=top-left">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/chart?repos=yourusername/opencode-llm-wiki&type=date&theme=dark&legend=top-left" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/chart?repos=yourusername/opencode-llm-wiki&type=date&legend=top-left" />
   <img alt="Star History Chart" src="https://api.star-history.com/chart?repos=yourusername/opencode-llm-wiki&type=date&legend=top-left" />
 </picture>
</a>
