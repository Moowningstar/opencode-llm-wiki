# Docker Deployment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deploy OpenCode LLM Wiki in Docker with MCP integration for Claude Desktop and Claude Code, then test by ingesting SQLAlchemy 2.0 documentation.

**Architecture:** Pre-compiled Rust binary in minimal Debian container, Docker Volumes for persistence, local MCP server via npx connecting to containerized API, environment variables for API keys.

**Tech Stack:** Docker, Docker Compose, Rust (pre-compiled), Node.js (npx), MCP Protocol

---

## File Structure

**New Files:**
- `Dockerfile` - Container image definition
- `docker-compose.yml` - Service orchestration
- `.env.example` - Environment variable template
- `.dockerignore` - Build exclusions
- `.mcp.json` - Claude Code MCP configuration

**Modified Files:**
- `.gitignore` - Add Docker and environment files

**No Code Changes:** This is pure infrastructure deployment.

---

## Task 1: Create Dockerfile

**Files:**
- Create: `Dockerfile`

- [ ] **Step 1: Create Dockerfile with base image and dependencies**

```dockerfile
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy pre-compiled binary
COPY target/release/llm-wiki-server /usr/local/bin/llm-wiki-server

# Ensure binary is executable
RUN chmod +x /usr/local/bin/llm-wiki-server

# Expose API port
EXPOSE 19828

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:19828/health || exit 1

# Run the server
CMD ["llm-wiki-server", "--host", "0.0.0.0", "--port", "19828"]
```

- [ ] **Step 2: Verify Dockerfile syntax**

Run: `docker build --dry-run -f Dockerfile .` (if supported) or just check syntax manually

Expected: No syntax errors

- [ ] **Step 3: Commit Dockerfile**

```bash
git add Dockerfile
git commit -m "feat: add Dockerfile for API server deployment

- Use debian:bookworm-slim base image
- Install minimal runtime dependencies (ca-certificates, libssl3, curl)
- Copy pre-compiled llm-wiki-server binary
- Expose port 19828
- Add health check endpoint
- Bind to 0.0.0.0 for Docker port mapping

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: Create Docker Compose Configuration

**Files:**
- Create: `docker-compose.yml`

- [ ] **Step 1: Create docker-compose.yml with service definition**

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

- [ ] **Step 2: Validate docker-compose.yml syntax**

Run: `docker-compose config`

Expected: Parsed configuration output with no errors

- [ ] **Step 3: Commit docker-compose.yml**

```bash
git add docker-compose.yml
git commit -m "feat: add Docker Compose configuration

- Define llm-wiki-api service with health checks
- Map port 19828 for API access
- Configure environment variables for API keys
- Create named volumes for data persistence
  - llm-wiki-data: Wiki pages and metadata
  - llm-wiki-vectors: Vector storage
- Set restart policy to unless-stopped

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Create Environment Configuration Files

**Files:**
- Create: `.env.example`
- Create: `.dockerignore`
- Modify: `.gitignore`

- [ ] **Step 1: Create .env.example template**

```env
# OpenRouter API Key (for embeddings)
# Get your key at: https://openrouter.ai/keys
OPENROUTER_API_KEY=sk-or-v1-your-key-here

# Anthropic API Key (optional, for context model)
# Get your key at: https://console.anthropic.com/
ANTHROPIC_API_KEY=sk-ant-your-key-here

# Logging level (debug, info, warn, error)
RUST_LOG=info
```

- [ ] **Step 2: Create .dockerignore file**

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

- [ ] **Step 3: Update .gitignore to exclude Docker files**

Add these lines to `.gitignore`:

```
# Docker environment
.env
.env.local

# Docker volumes (if mounted locally)
docker-volumes/
```

- [ ] **Step 4: Commit configuration files**

```bash
git add .env.example .dockerignore .gitignore
git commit -m "feat: add Docker environment configuration

- Add .env.example template with API key placeholders
- Add .dockerignore to exclude unnecessary files from build
- Update .gitignore to exclude .env files

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Create MCP Configuration for Claude Code

**Files:**
- Create: `.mcp.json`

- [ ] **Step 1: Create .mcp.json for Claude Code**

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

- [ ] **Step 2: Validate JSON syntax**

Run: `node -e "JSON.parse(require('fs').readFileSync('.mcp.json', 'utf8'))"`

Expected: No output (valid JSON)

- [ ] **Step 3: Commit .mcp.json**

```bash
git add .mcp.json
git commit -m "feat: add MCP configuration for Claude Code

- Configure llm-wiki MCP server via npx
- Connect to Docker API at localhost:19828
- Enable 11 MCP tools for knowledge graph operations

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: Compile Rust Binary

**Files:**
- Build: `target/release/llm-wiki-server`

- [ ] **Step 1: Clean previous builds**

Run: `cargo clean`

Expected: `target/` directory removed

- [ ] **Step 2: Build release binary**

Run: `cargo build --release`

Expected: Compilation succeeds, binary created at `target/release/llm-wiki-server` (or `target/release/llm-wiki-server.exe` on Windows)

- [ ] **Step 3: Verify binary exists and is executable**

Run: `ls -lh target/release/llm-wiki-server` (Linux/Mac) or `dir target\release\llm-wiki-server.exe` (Windows)

Expected: Binary file exists, size ~10-50MB

- [ ] **Step 4: Test binary runs**

Run: `target/release/llm-wiki-server --help`

Expected: Help text showing command-line options

---

## Task 6: Configure Environment Variables

**Files:**
- Create: `.env` (local only, not committed)

- [ ] **Step 1: Copy .env.example to .env**

Run: `cp .env.example .env`

Expected: `.env` file created

- [ ] **Step 2: Edit .env with actual API keys**

**Manual step:** Open `.env` in editor and replace placeholders:

```env
OPENROUTER_API_KEY=sk-or-v1-<your-actual-key>
ANTHROPIC_API_KEY=sk-ant-<your-actual-key>
RUST_LOG=info
```

- [ ] **Step 3: Verify .env is in .gitignore**

Run: `git check-ignore .env`

Expected: `.env` (confirms it's ignored)

---

## Task 7: Build and Start Docker Container

**Files:**
- Docker image: `opencode-llm-wiki-llm-wiki-api`
- Docker volumes: `llm-wiki-data`, `llm-wiki-vectors`

- [ ] **Step 1: Build Docker image**

Run: `docker-compose build`

Expected: Image builds successfully, shows "Successfully built" message

- [ ] **Step 2: Start container in detached mode**

Run: `docker-compose up -d`

Expected: Container starts, shows "Container llm-wiki-api  Started"

- [ ] **Step 3: Check container status**

Run: `docker-compose ps`

Expected: Container shows "Up" status with "(healthy)" after ~10 seconds

- [ ] **Step 4: View container logs**

Run: `docker-compose logs llm-wiki-api`

Expected: Logs show:
```
🚀 OpenCode LLM Wiki Server
   Version: 1.1.1
   Host: 0.0.0.0
   Port: 19828
```

- [ ] **Step 5: Test health endpoint**

Run: `curl http://localhost:19828/health`

Expected: `{"status":"ok"}`

---

## Task 8: Verify MCP Server Connection

**Files:**
- None (testing only)

- [ ] **Step 1: Test MCP server can reach API**

Run: `npx -y @opencode-llm-wiki/mcp-server@latest` (will start stdio server, press Ctrl+C after verification)

Expected: Server starts without connection errors

- [ ] **Step 2: Restart Claude Code to load MCP configuration**

**Manual step:** 
1. Close Claude Code
2. Reopen Claude Code in this project directory
3. Check MCP servers list

Expected: `llm-wiki` server appears in available MCP servers

- [ ] **Step 3: Verify MCP tools are available**

**Manual step in Claude Code:** Ask Claude to list available MCP tools

Expected: 11 tools visible:
- wiki_read
- wiki_list
- wiki_search
- wiki_query_with_context
- wiki_get_graph
- wiki_graph_insights
- wiki_deep_research
- wiki_get_index
- wiki_get_overview
- wiki_get_purpose
- wiki_ingest

---

## Task 9: Test Basic API Operations

**Files:**
- None (testing only)

- [ ] **Step 1: Test health endpoint**

Run: `curl http://localhost:19828/health`

Expected: `{"status":"ok"}`

- [ ] **Step 2: Test list pages (should be empty initially)**

Run: 
```bash
curl -X POST http://localhost:19828/api/pages \
  -H "Content-Type: application/json" \
  -d '{"scope":"global"}'
```

Expected: `{"pages":[],"total":0}` or similar empty response

- [ ] **Step 3: Test graph endpoint (should be empty initially)**

Run:
```bash
curl -X POST http://localhost:19828/api/graph \
  -H "Content-Type: application/json" \
  -d '{"scope":"global"}'
```

Expected: `{"nodes":[],"edges":[]}` or similar empty graph

---

## Task 10: Prepare SQLAlchemy Documentation

**Files:**
- External: `/tmp/sqlalchemy/` (or `C:\Temp\sqlalchemy\` on Windows)

- [ ] **Step 1: Clone SQLAlchemy repository**

Run: `git clone https://github.com/sqlalchemy/sqlalchemy.git /tmp/sqlalchemy`

Expected: Repository cloned successfully

- [ ] **Step 2: Checkout SQLAlchemy 2.0 branch**

Run: `cd /tmp/sqlalchemy && git checkout rel_2_0`

Expected: Switched to rel_2_0 branch

- [ ] **Step 3: Verify documentation files exist**

Run: `ls /tmp/sqlalchemy/doc/build/` or `find /tmp/sqlalchemy -name "*.rst" | head -10`

Expected: Documentation files (.rst or .md) found

---

## Task 11: Ingest SQLAlchemy Documentation via MCP

**Files:**
- None (data stored in Docker volumes)

- [ ] **Step 1: Use wiki_ingest tool in Claude Code**

**Manual step in Claude Code:** Ask Claude to execute:

```
Use the wiki_ingest MCP tool to ingest SQLAlchemy 2.0 documentation from /tmp/sqlalchemy/doc/build/ (or C:\Temp\sqlalchemy\doc\build\ on Windows) with recursive=true
```

Expected: Ingestion starts, shows progress, completes successfully

- [ ] **Step 2: Verify ingestion via API**

Run:
```bash
curl -X POST http://localhost:19828/api/pages \
  -H "Content-Type: application/json" \
  -d '{"scope":"global"}'
```

Expected: Response shows multiple pages with SQLAlchemy content

- [ ] **Step 3: Check Docker logs for ingestion activity**

Run: `docker-compose logs --tail=50 llm-wiki-api`

Expected: Logs show embedding requests, chunking activity, vector storage operations

---

## Task 12: Test MCP Search and Query Tools

**Files:**
- None (testing only)

- [ ] **Step 1: Test wiki_list tool**

**Manual step in Claude Code:** Ask Claude:
```
Use wiki_list to show all ingested pages
```

Expected: List of SQLAlchemy documentation pages with metadata

- [ ] **Step 2: Test wiki_search tool**

**Manual step in Claude Code:** Ask Claude:
```
Use wiki_search to find pages about "ORM session management"
```

Expected: Relevant pages ranked by keyword match

- [ ] **Step 3: Test wiki_query_with_context tool**

**Manual step in Claude Code:** Ask Claude:
```
Use wiki_query_with_context to answer: "How do I configure database connection pooling in SQLAlchemy 2.0?"
```

Expected: Context-optimized results with relevant documentation excerpts

- [ ] **Step 4: Test wiki_read tool**

**Manual step in Claude Code:** Ask Claude:
```
Use wiki_read to read the content of [pick a page from wiki_list results]
```

Expected: Full page content displayed

---

## Task 13: Test Graph Analysis Tools

**Files:**
- None (testing only)

- [ ] **Step 1: Test wiki_get_graph tool**

**Manual step in Claude Code:** Ask Claude:
```
Use wiki_get_graph to retrieve the knowledge graph
```

Expected: JSON with nodes (pages) and edges (links between pages)

- [ ] **Step 2: Test wiki_graph_insights with PageRank**

**Manual step in Claude Code:** Ask Claude:
```
Use wiki_graph_insights with analysis_type="pagerank" to find most influential pages
```

Expected: Pages ranked by PageRank score (likely index pages, core concepts)

- [ ] **Step 3: Test wiki_graph_insights with community detection**

**Manual step in Claude Code:** Ask Claude:
```
Use wiki_graph_insights with analysis_type="communities" to find topic clusters
```

Expected: Pages grouped into communities by topic

- [ ] **Step 4: Test wiki_deep_research tool**

**Manual step in Claude Code:** Ask Claude:
```
Use wiki_deep_research with query="async database operations" and max_depth=2
```

Expected: Multi-hop traversal results showing related concepts and connections

---

## Task 14: Test Data Persistence

**Files:**
- Docker volumes: `llm-wiki-data`, `llm-wiki-vectors`

- [ ] **Step 1: Stop Docker container**

Run: `docker-compose down`

Expected: Container stops, but volumes remain (do NOT use `-v` flag)

- [ ] **Step 2: Verify volumes still exist**

Run: `docker volume ls | grep llm-wiki`

Expected: Both volumes listed:
```
llm-wiki-data
llm-wiki-vectors
```

- [ ] **Step 3: Restart container**

Run: `docker-compose up -d`

Expected: Container starts successfully

- [ ] **Step 4: Verify data persisted**

Run:
```bash
curl -X POST http://localhost:19828/api/pages \
  -H "Content-Type: application/json" \
  -d '{"scope":"global"}'
```

Expected: SQLAlchemy pages still present (same count as before restart)

- [ ] **Step 5: Test via MCP**

**Manual step in Claude Code:** Ask Claude:
```
Use wiki_list to verify SQLAlchemy pages are still available
```

Expected: Same pages as before container restart

---

## Task 15: Document Claude Desktop Configuration (Optional)

**Files:**
- Create: `docs/CLAUDE_DESKTOP_SETUP.md`

- [ ] **Step 1: Create setup guide for Claude Desktop**

```markdown
# Claude Desktop MCP Setup

## Configuration

Add this to your Claude Desktop configuration file:

**Location:**
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`

**Configuration:**

\`\`\`json
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
\`\`\`

## Troubleshooting

### Connection Issues

If MCP server can't connect to API, try:

**Windows Docker Desktop:**
\`\`\`json
"LLM_WIKI_API_URL": "http://host.docker.internal:19828"
\`\`\`

**Linux:**
\`\`\`json
"LLM_WIKI_API_URL": "http://172.17.0.1:19828"
\`\`\`

### Verify API is Running

\`\`\`bash
curl http://localhost:19828/health
\`\`\`

Expected: `{"status":"ok"}`

## Restart Claude Desktop

After editing configuration, restart Claude Desktop to load the MCP server.
```

- [ ] **Step 2: Commit documentation**

```bash
git add docs/CLAUDE_DESKTOP_SETUP.md
git commit -m "docs: add Claude Desktop MCP setup guide

- Document configuration file location
- Provide MCP server configuration
- Add troubleshooting for connection issues
- Include verification steps

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 16: Create Deployment README

**Files:**
- Create: `docs/DOCKER_DEPLOYMENT.md`

- [ ] **Step 1: Create deployment guide**

```markdown
# Docker Deployment Guide

## Prerequisites

- Docker and Docker Compose installed
- Rust toolchain (for compiling binary)
- Node.js 18+ (for MCP server)
- API keys: OpenRouter and/or Anthropic

## Quick Start

### 1. Compile Rust Binary

\`\`\`bash
cargo build --release
\`\`\`

### 2. Configure Environment

\`\`\`bash
cp .env.example .env
# Edit .env and add your API keys
\`\`\`

### 3. Start Docker Container

\`\`\`bash
docker-compose up -d
\`\`\`

### 4. Verify Deployment

\`\`\`bash
curl http://localhost:19828/health
\`\`\`

Expected: `{"status":"ok"}`

### 5. Configure MCP

**Claude Code:** `.mcp.json` is already configured in project root

**Claude Desktop:** See [CLAUDE_DESKTOP_SETUP.md](CLAUDE_DESKTOP_SETUP.md)

## Management

### View Logs

\`\`\`bash
docker-compose logs -f llm-wiki-api
\`\`\`

### Stop Container

\`\`\`bash
docker-compose down
\`\`\`

### Restart Container

\`\`\`bash
docker-compose restart
\`\`\`

### Remove Everything (including data)

\`\`\`bash
docker-compose down -v
\`\`\`

**Warning:** This deletes all Wiki data and vectors!

## Data Persistence

Data is stored in Docker volumes:
- `llm-wiki-data` - Wiki pages and metadata
- `llm-wiki-vectors` - Vector embeddings

To backup:

\`\`\`bash
docker run --rm -v llm-wiki-data:/data -v $(pwd):/backup alpine tar czf /backup/llm-wiki-data-backup.tar.gz -C /data .
docker run --rm -v llm-wiki-vectors:/data -v $(pwd):/backup alpine tar czf /backup/llm-wiki-vectors-backup.tar.gz -C /data .
\`\`\`

## Troubleshooting

### Port Already in Use

If port 19828 is in use, edit `docker-compose.yml`:

\`\`\`yaml
ports:
  - "19829:19828"  # Use different host port
\`\`\`

Then update MCP configuration to use `http://localhost:19829`

### Container Unhealthy

Check logs:

\`\`\`bash
docker-compose logs llm-wiki-api
\`\`\`

Common issues:
- Missing API keys in `.env`
- Binary not compiled or wrong architecture
- Network issues reaching embedding API

### MCP Connection Failed

Verify API is accessible:

\`\`\`bash
curl http://localhost:19828/health
\`\`\`

If fails, try `http://host.docker.internal:19828` in MCP configuration (Windows Docker Desktop)
```

- [ ] **Step 2: Commit deployment guide**

```bash
git add docs/DOCKER_DEPLOYMENT.md
git commit -m "docs: add Docker deployment guide

- Document quick start steps
- Add container management commands
- Explain data persistence and backup
- Include troubleshooting section

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 17: Final Verification and Cleanup

**Files:**
- None (verification only)

- [ ] **Step 1: Run full test suite**

**Manual verification checklist:**
- [ ] Docker container is running and healthy
- [ ] API health endpoint responds
- [ ] MCP server connects successfully
- [ ] SQLAlchemy docs are ingested
- [ ] All 11 MCP tools work correctly
- [ ] Data persists after container restart
- [ ] Documentation is complete

- [ ] **Step 2: Check for uncommitted changes**

Run: `git status`

Expected: Working tree clean (or only `.env` untracked)

- [ ] **Step 3: Review all commits**

Run: `git log --oneline -20`

Expected: All tasks committed with descriptive messages

- [ ] **Step 4: Tag deployment version (optional)**

Run:
```bash
git tag -a v1.1.1-docker -m "Docker deployment with MCP integration"
git push origin v1.1.1-docker
```

---

## Success Criteria

- [x] Docker container builds and starts successfully
- [x] API health check passes
- [x] MCP server connects to Docker API
- [x] Both Claude Code and Claude Desktop configurations documented
- [x] SQLAlchemy 2.0 documentation ingested
- [x] All 11 MCP tools tested and working:
  - wiki_read, wiki_list, wiki_search
  - wiki_query_with_context
  - wiki_get_graph, wiki_graph_insights
  - wiki_deep_research
  - wiki_get_index, wiki_get_overview, wiki_get_purpose
  - wiki_ingest
- [x] Data persists across container restarts
- [x] No API keys in committed files
- [x] Complete documentation for deployment and setup

## Rollback Plan

If deployment fails:

1. Stop container: `docker-compose down`
2. Remove volumes (if needed): `docker volume rm llm-wiki-data llm-wiki-vectors`
3. Revert commits: `git reset --hard HEAD~N` (where N = number of commits to undo)
4. Remove Docker artifacts: `docker system prune -a`

## Notes

- This plan assumes Windows environment with Docker Desktop
- For Linux, adjust paths and use `http://172.17.0.1:19828` if localhost doesn't work
- MCP server runs locally via npx, not in Docker
- Pre-compiled binary approach means faster Docker builds but requires manual compilation
- All sensitive data (API keys) goes in `.env`, never committed to Git
