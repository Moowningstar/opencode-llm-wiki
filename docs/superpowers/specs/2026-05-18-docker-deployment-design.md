# Docker Deployment Design for OpenCode LLM Wiki

**Date:** 2026-05-18  
**Status:** Approved  
**Author:** Claude (Opus 4.7)

## Overview

Deploy OpenCode LLM Wiki using Docker for the Rust API server, with local MCP server integration for both Claude Desktop and Claude Code environments. Test deployment by ingesting SQLAlchemy 2.0 documentation.

## Goals

1. Containerize the Rust API server using Docker with minimal configuration
2. Support both Claude Desktop (global) and Claude Code (project-local) MCP configurations
3. Persist Wiki data and vector storage using Docker Volumes
4. Configure API keys via environment variables
5. Test the complete stack by ingesting and querying SQLAlchemy 2.0 documentation

## Architecture

### Deployment Topology

```
┌─────────────────────────────────────────────────────┐
│ Host Machine (Windows)                               │
│                                                      │
│  ┌────────────────────────────────────────────┐    │
│  │ Claude Desktop / Claude Code                │    │
│  │  └─> MCP Client (stdio)                    │    │
│  └────────────────┬───────────────────────────┘    │
│                   │ stdio                           │
│  ┌────────────────▼───────────────────────────┐    │
│  │ MCP Server (Local npx)                      │    │
│  │  • Command: npx @opencode-llm-wiki/mcp-server │
│  │  • Env: LLM_WIKI_API_URL                   │    │
│  └────────────────┬───────────────────────────┘    │
│                   │ HTTP                            │
│                   │ localhost:19828                 │
│  ┌────────────────▼───────────────────────────┐    │
│  │ Docker Container: llm-wiki-api             │    │
│  │  ┌──────────────────────────────────────┐  │    │
│  │  │ Rust API Server (llm-wiki-server)    │  │    │
│  │  │  • Port: 19828                       │  │    │
│  │  │  • Env: OPENROUTER_API_KEY           │  │    │
│  │  │  • Env: ANTHROPIC_API_KEY            │  │    │
│  │  └──────────────────────────────────────┘  │    │
│  │  ┌──────────────────────────────────────┐  │    │
│  │  │ Docker Volumes                       │  │    │
│  │  │  • llm-wiki-data (Wiki pages)        │  │    │
│  │  │  • llm-wiki-vectors (Vector storage) │  │    │
│  │  └──────────────────────────────────────┘  │    │
│  └────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────┘
```

### Component Responsibilities

1. **Docker Container (llm-wiki-api)**
   - Runs pre-compiled Rust binary (`llm-wiki-server`)
   - Exposes HTTP API on port 19828
   - Receives API keys via environment variables
   - Persists data to Docker Volumes

2. **Local MCP Server**
   - Runs via `npx @opencode-llm-wiki/mcp-server@latest`
   - Connects to Docker API via HTTP (localhost:19828)
   - Communicates with Claude via stdio protocol
   - Provides 11 MCP tools for knowledge graph operations

3. **Docker Volumes**
   - `llm-wiki-data`: Wiki pages, index.json, graph.json
   - `llm-wiki-vectors`: RuVector vector storage and embeddings

### Why This Architecture

**Rationale for chosen approach:**
- **MCP on host, API in Docker**: Simplest setup, follows MCP standard patterns
- **Pre-compiled binary**: Fast startup, no build complexity in Docker
- **Docker Volumes**: Managed persistence, automatic cleanup, good performance
- **Environment variables**: Secure API key management, no hardcoded secrets
- **localhost HTTP**: Simple, reliable communication between MCP and API

**Trade-offs accepted:**
- Requires manual Rust compilation before Docker build (acceptable for testing/development)
- MCP server requires Node.js on host (already available)
- Windows-specific localhost mapping (fallback to host.docker.internal if needed)

## Implementation Details

### 1. Docker Configuration

#### Dockerfile

```dockerfile
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy pre-compiled binary
COPY target/release/llm-wiki-server /usr/local/bin/llm-wiki-server

# Expose API port
EXPOSE 19828

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:19828/health || exit 1

# Run the server
CMD ["llm-wiki-server", "--host", "0.0.0.0", "--port", "19828"]
```

**Key decisions:**
- Base image: `debian:bookworm-slim` (small, stable, good OpenSSL support)
- Runtime deps: Only `ca-certificates` and `libssl3` (minimal attack surface)
- Binary location: `/usr/local/bin/` (standard PATH location)
- Bind to `0.0.0.0`: Required for Docker port mapping
- Health check: Ensures container is ready before accepting traffic

#### docker-compose.yml

```yaml
version: '3.8'

services:
  llm-wiki-api:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: llm-wiki-api
    ports:
      - "19828:19828"
    environment:
      - OPENROUTER_API_KEY=${OPENROUTER_API_KEY}
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
      - RUST_LOG=info
    volumes:
      - llm-wiki-data:/root/.opencode-llm-wiki
      - llm-wiki-vectors:/app/data
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:19828/health"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 10s

volumes:
  llm-wiki-data:
    name: llm-wiki-data
  llm-wiki-vectors:
    name: llm-wiki-vectors
```

**Key decisions:**
- Named volumes: Easier to identify and manage
- Environment variables: Loaded from `.env` file
- Restart policy: `unless-stopped` (survives reboots, manual stop respected)
- Volume mounts:
  - `/root/.opencode-llm-wiki`: Global storage location (deduplication, registry)
  - `/app/data`: Project-local vector storage
- Health check: Duplicated from Dockerfile for docker-compose awareness

#### .env.example

```env
# OpenRouter API Key (for embeddings)
OPENROUTER_API_KEY=sk-or-v1-your-key-here

# Anthropic API Key (optional, for context model)
ANTHROPIC_API_KEY=sk-ant-your-key-here

# Logging level (debug, info, warn, error)
RUST_LOG=info
```

#### .dockerignore

```
# Build artifacts
target/
dist/
*.log

# Git
.git/
.gitignore

# Documentation
docs/
*.md
!README.md

# Development files
.vscode/
.idea/
.claude/
.gitnexus/

# Environment files
.env
.env.local

# Node modules
node_modules/
src-mcp/node_modules/

# Test data
data/
.wiki/
.llm-wiki/

# Temporary files
*.tmp
*.swp
*~
```

### 2. MCP Server Configuration

#### Claude Desktop Configuration

**File:** `~/.claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "llm-wiki": {
      "command": "npx",
      "args": [
        "-y",
        "@opencode-llm-wiki/mcp-server@latest"
      ],
      "env": {
        "LLM_WIKI_API_URL": "http://localhost:19828"
      }
    }
  }
}
```

**Configuration notes:**
- `npx -y`: Auto-confirm package installation
- `@latest`: Always use latest published version
- `LLM_WIKI_API_URL`: Points to Docker-exposed API port

#### Claude Code Configuration

**File:** `.mcp.json` (project root)

```json
{
  "mcpServers": {
    "llm-wiki": {
      "command": "npx",
      "args": [
        "-y",
        "@opencode-llm-wiki/mcp-server@latest"
      ],
      "env": {
        "LLM_WIKI_API_URL": "http://localhost:19828"
      }
    }
  }
}
```

**Same configuration as Claude Desktop, but project-scoped.**

#### Network Fallback Options

If `localhost:19828` doesn't work on Windows Docker Desktop:

```json
"LLM_WIKI_API_URL": "http://host.docker.internal:19828"
```

On Linux (if using bridge network):

```json
"LLM_WIKI_API_URL": "http://172.17.0.1:19828"
```

### 3. Build and Deployment Process

#### Step 1: Compile Rust Binary

```bash
# Build release binary
cargo build --release

# Verify binary exists
ls -lh target/release/llm-wiki-server
```

**Why pre-compile:**
- Faster Docker builds (no Rust toolchain in image)
- Smaller image size (only runtime dependencies)
- Easier to debug compilation issues on host

#### Step 2: Configure Environment

```bash
# Copy environment template
cp .env.example .env

# Edit .env and add your API keys
# OPENROUTER_API_KEY=sk-or-v1-...
# ANTHROPIC_API_KEY=sk-ant-...
```

#### Step 3: Build and Start Docker Container

```bash
# Build image and start container
docker-compose up -d

# Check container status
docker-compose ps

# View logs
docker-compose logs -f llm-wiki-api

# Verify health
curl http://localhost:19828/health
```

**Expected health response:**
```json
{"status": "ok"}
```

#### Step 4: Configure MCP Server

**For Claude Code (project-local):**
- `.mcp.json` will be created automatically
- Restart Claude Code to load MCP server

**For Claude Desktop (global):**
- Manually edit `~/.claude/claude_desktop_config.json`
- Restart Claude Desktop

#### Step 5: Verify MCP Connection

In Claude Desktop or Claude Code, check that the `llm-wiki` MCP server appears in the available tools list. You should see 11 tools:

1. `wiki_read`
2. `wiki_list`
3. `wiki_search`
4. `wiki_query_with_context`
5. `wiki_get_graph`
6. `wiki_graph_insights`
7. `wiki_deep_research`
8. `wiki_get_index`
9. `wiki_get_overview`
10. `wiki_get_purpose`
11. `wiki_ingest`

### 4. Testing Plan

#### Test 1: Basic Health Check

```bash
# API health
curl http://localhost:19828/health

# Expected: {"status":"ok"}
```

#### Test 2: Ingest SQLAlchemy 2.0 Documentation

**Preparation:**
```bash
# Clone SQLAlchemy repository
git clone https://github.com/sqlalchemy/sqlalchemy.git /tmp/sqlalchemy
cd /tmp/sqlalchemy
git checkout rel_2_0
```

**Via MCP Tool (in Claude):**
```
Use wiki_ingest tool to ingest the SQLAlchemy 2.0 documentation from /tmp/sqlalchemy/doc/build/
```

**Via HTTP API (alternative):**
```bash
curl -X POST http://localhost:19828/api/ingest \
  -H "Content-Type: application/json" \
  -d '{
    "source_path": "/tmp/sqlalchemy/doc/build/",
    "recursive": true,
    "project_name": "sqlalchemy-docs"
  }'
```

#### Test 3: List Ingested Pages

**Via MCP Tool:**
```
Use wiki_list tool to show all pages
```

**Expected:** List of SQLAlchemy documentation pages with metadata (title, path, tags, created_at)

#### Test 4: Keyword Search

**Via MCP Tool:**
```
Use wiki_search to find pages about "ORM session management"
```

**Expected:** Relevant pages ranked by keyword match

#### Test 5: Semantic Query

**Via MCP Tool:**
```
Use wiki_query_with_context to ask: "How do I configure database connection pooling in SQLAlchemy 2.0?"
```

**Expected:** Context-optimized results using vector similarity + keyword matching

#### Test 6: Knowledge Graph

**Via MCP Tool:**
```
Use wiki_get_graph to retrieve the knowledge graph
```

**Expected:** JSON with nodes (pages) and edges (links between pages)

#### Test 7: Graph Insights

**Via MCP Tool:**
```
Use wiki_graph_insights with analysis_type="pagerank" to find most influential pages
```

**Expected:** Pages ranked by PageRank score (likely index pages, core concepts)

#### Test 8: Deep Research

**Via MCP Tool:**
```
Use wiki_deep_research with query="async database operations" and max_depth=2
```

**Expected:** Multi-hop traversal results showing related concepts and their connections

#### Test 9: Data Persistence

```bash
# Stop container
docker-compose down

# Restart container
docker-compose up -d

# Verify data still exists
# Use wiki_list tool - should show previously ingested pages
```

**Expected:** All SQLAlchemy pages still present after restart

#### Test 10: Volume Inspection

```bash
# List volumes
docker volume ls | grep llm-wiki

# Inspect volume
docker volume inspect llm-wiki-data
docker volume inspect llm-wiki-vectors
```

**Expected:** Two named volumes with mount points in Docker's data directory

### 5. Troubleshooting

#### Issue: Container fails to start

**Check logs:**
```bash
docker-compose logs llm-wiki-api
```

**Common causes:**
- Missing API keys in `.env`
- Port 19828 already in use
- Binary not compiled or wrong architecture

**Solutions:**
- Verify `.env` file exists and has valid keys
- Check port: `netstat -an | grep 19828`
- Recompile: `cargo build --release`

#### Issue: MCP server can't connect to API

**Symptoms:** MCP tools fail with connection errors

**Check API accessibility:**
```bash
curl http://localhost:19828/health
```

**Solutions:**
- If fails: Try `http://host.docker.internal:19828` in MCP config
- Check Docker port mapping: `docker-compose ps`
- Verify container is healthy: `docker-compose ps` (should show "healthy")

#### Issue: Ingestion fails

**Check API logs:**
```bash
docker-compose logs -f llm-wiki-api
```

**Common causes:**
- Invalid API key (embedding service fails)
- Network issues (can't reach OpenRouter/Anthropic)
- File path not accessible from container

**Solutions:**
- Verify API keys are correct
- Check network: `docker exec llm-wiki-api curl https://openrouter.ai`
- For file ingestion: Files must be accessible from host (MCP server reads, sends to API)

#### Issue: Data not persisting

**Check volumes:**
```bash
docker volume ls
docker volume inspect llm-wiki-data
```

**Solution:**
- Ensure volumes are named (not anonymous)
- Don't use `docker-compose down -v` (removes volumes)
- Use `docker-compose down` (preserves volumes)

## File Structure

### New Files

```
opencode-llm-wiki/
├── Dockerfile                    # Docker image definition
├── docker-compose.yml            # Docker Compose orchestration
├── .env.example                  # Environment variable template
├── .dockerignore                 # Docker build exclusions
├── .mcp.json                     # Claude Code MCP configuration
└── docs/
    └── superpowers/
        └── specs/
            └── 2026-05-18-docker-deployment-design.md  # This document
```

### Modified Files

```
.gitignore                        # Add .env and Docker-related files
```

### Docker Volume Data

```
Docker Volumes (managed by Docker):
├── llm-wiki-data/
│   ├── .vectors/.store/          # Deduplicated vectors
│   ├── .hash_index.json          # Content hash index
│   ├── .ref_counter.db           # Reference counting
│   └── .projects/.registry       # Project registry
└── llm-wiki-vectors/
    └── ruvector/                 # Project-local vector storage
```

### Manual Configuration

```
~/.claude/claude_desktop_config.json  # Claude Desktop MCP config (manual)
```

## Success Criteria

1. ✅ Docker container starts successfully and passes health check
2. ✅ MCP server connects to Docker API (both Claude Desktop and Claude Code)
3. ✅ SQLAlchemy 2.0 documentation ingests successfully
4. ✅ All 11 MCP tools work correctly:
   - Basic operations (read, list, search)
   - Semantic query with context injection
   - Knowledge graph retrieval
   - Graph analytics (PageRank, communities, centrality)
   - Deep research with multi-hop traversal
5. ✅ Data persists across container restarts
6. ✅ No API keys hardcoded in files (all via environment variables)

## Non-Goals

- Multi-stage Docker build (using pre-compiled binary instead)
- Production-grade security hardening (development/testing focus)
- Kubernetes deployment (Docker Compose sufficient for single-machine)
- MCP server in Docker (local npx execution is simpler)
- Automated SQLAlchemy doc download (manual clone acceptable for testing)

## Future Enhancements

- Multi-stage Dockerfile for self-contained builds
- Docker health check improvements (check vector storage readiness)
- Automated backup scripts for Docker volumes
- Production deployment guide (reverse proxy, TLS, monitoring)
- Docker Hub image publishing
- Kubernetes manifests for scalable deployment

## References

- [OpenCode LLM Wiki README](../../README.md)
- [MCP Protocol Specification](https://modelcontextprotocol.io/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [SQLAlchemy 2.0 Documentation](https://docs.sqlalchemy.org/en/20/)
