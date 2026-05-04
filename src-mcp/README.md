# OpenCode LLM Wiki - MCP Server

[![npm version](https://badge.fury.io/js/@opencode-llm-wiki%2Fmcp-server.svg)](https://www.npmjs.com/package/@opencode-llm-wiki/mcp-server)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

MCP (Model Context Protocol) server for OpenCode LLM Wiki - Knowledge graph backend with vector search.

## 📦 Installation

### From npm

```bash
npm install -g @opencode-llm-wiki/mcp-server
```

### From source

```bash
git clone https://github.com/Moowningstar/opencode-llm-wiki.git
cd opencode-llm-wiki/src-mcp
npm install
```

## 🚀 Quick Start

### Usage in Claude Desktop

Add to your Claude Desktop configuration (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "llm-wiki": {
      "command": "node",
      "args": [
        "/path/to/node_modules/@opencode-llm-wiki/mcp-server/src/server.js"
      ],
      "env": {
        "LLM_WIKI_API_URL": "http://localhost:19828"
      }
    }
  }
}
```

### Usage in OpenCode

The MCP server is automatically configured via `opencode.jsonc` in the project root:

```jsonc
{
  "mcp": {
    "servers": {
      "llm-wiki": {
        "command": "node",
        "args": ["node_modules/@opencode-llm-wiki/mcp-server/src/server.js"],
        "env": {
          "LLM_WIKI_API_URL": "http://localhost:19828"
        }
      }
    }
  }
}
```

### Start the Backend Server

Before using the MCP server, make sure the OpenCode LLM Wiki backend is running:

```bash
# Clone the main repository
git clone https://github.com/Moowningstar/opencode-llm-wiki.git
cd opencode-llm-wiki

# Build and run the Rust backend
cargo run --bin llm-wiki-server -- --port 19828
```

The MCP server communicates with the backend via HTTP API.

## 🛠️ Available MCP Tools

The server exposes **11 tools** for interacting with your knowledge base:

### Core Tools

| Tool | Description |
|------|-------------|
| `wiki_read` | Read a wiki page by path |
| `wiki_list` | List all wiki pages |
| `wiki_search` | Search pages by keyword |
| `wiki_query_with_context` | Intelligent context injection (keyword + vector search) |
| `wiki_ingest` | Ingest documents into the knowledge graph |

### Graph Tools

| Tool | Description |
|------|-------------|
| `wiki_get_graph` | Get knowledge graph data (nodes and edges) |
| `wiki_graph_insights` | Analyze graph structure (isolated pages, bridges, statistics) |
| `wiki_deep_research` | Multi-hop reasoning with graph traversal and semantic search |

### Metadata Tools

| Tool | Description |
|------|-------------|
| `wiki_get_index` | Get index.md (content catalog) |
| `wiki_get_overview` | Get overview.md (global summary with graph statistics) |
| `wiki_get_purpose` | Get purpose.md (wiki goals and scope) |

## 📖 Tool Examples

### Read a Page

```typescript
{
  "tool": "wiki_read",
  "arguments": {
    "path": "entities/gpt-4.md"
  }
}
```

### Search with Context

```typescript
{
  "tool": "wiki_query_with_context",
  "arguments": {
    "query": "transformer architecture",
    "max_tokens": 4000
  }
}
```

### Deep Research

```typescript
{
  "tool": "wiki_deep_research",
  "arguments": {
    "query": "attention mechanisms in neural networks",
    "max_depth": 3,
    "max_results": 10
  }
}
```

### Graph Insights

```typescript
{
  "tool": "wiki_graph_insights",
  "arguments": {
    "analysis_type": "all"  // Options: "isolated", "bridges", "stats", "all"
  }
}
```

### Ingest Documents

```typescript
{
  "tool": "wiki_ingest",
  "arguments": {
    "path": "docs/architecture.md",
    "recursive": false
  }
}
```

## 🏗️ Architecture

The MCP server acts as a bridge between AI agents (like Claude Desktop) and the OpenCode LLM Wiki backend:

```
┌─────────────────┐
│  Claude Desktop │
│   (AI Agent)    │
└────────┬────────┘
         │ MCP Protocol (stdio)
         ↓
┌─────────────────┐
│   MCP Server    │
│  (Node.js)      │
└────────┬────────┘
         │ HTTP API
         ↓
┌─────────────────┐
│  Rust Backend   │
│  (Port 19828)   │
│                 │
│  • RuVector DB  │
│  • Graph Algos  │
│  • Vector Search│
└─────────────────┘
```

### Components

- **MCP Server** (`src/server.js`): Implements Model Context Protocol, exposes 11 tools
- **Core API Client** (`src/lib/core-api-client.js`): HTTP client for backend communication
- **Backend Server**: Rust service providing vector search, graph algorithms, and storage

## 🔧 Development

### Run Server Manually

```bash
cd src-mcp
LLM_WIKI_API_URL=http://localhost:19828 node src/server.js
```

The server runs in stdio mode for MCP protocol communication.

### Debug Mode

Set the `DEBUG` environment variable to see detailed logs:

```bash
DEBUG=llm-wiki:* LLM_WIKI_API_URL=http://localhost:19828 node src/server.js
```

### Testing Tools

You can test individual tools using the MCP inspector or by sending JSON-RPC requests:

```bash
# Example: Test wiki_list tool
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"wiki_list","arguments":{}}}' | node src/server.js
```

## 📝 Configuration

### Environment Variables

- `LLM_WIKI_PROJECT` - Project root path (default: `~/llm-wiki-projects/default`)
- `LLM_WIKI_API_URL` - API server URL (optional, for future HTTP mode)

### Project Structure

```
my-wiki/
├── .raw/              # Drop files here for auto-ingest
├── .wiki/
│   ├── entities/      # Named things (models, companies, people)
│   ├── concepts/      # Ideas, techniques, phenomena
│   ├── sources/       # Source summaries
│   ├── queries/       # Research questions
│   ├── index.md       # Content catalog (auto-maintained)
│   └── overview.md    # Global summary (auto-generated)
├── purpose.md         # Wiki goals and scope
├── schema.md          # Structure rules
└── .llm-wiki/         # Internal data (database, cache)
```

## 🔗 Integration with OpenCode

### How It Works

1. **OpenCode starts** → Reads `opencode.jsonc`
2. **MCP server launches** → `node src-mcp/src/server.js`
3. **Server initializes** → Connects to wiki project at `${workspaceFolder}`
4. **Tools available** → OpenCode can call any of the 10 MCP tools
5. **File watching** → Server monitors `.raw/` and `.wiki/` for changes

### Usage in OpenCode

When you ask OpenCode a question about your knowledge base:

```
You: "What do I know about transformer architecture?"

OpenCode internally calls:
1. wiki_query_with_context(query="transformer architecture")
2. Receives relevant wiki pages with context
3. Answers your question with citations
```

## 🚨 Troubleshooting

### Server Won't Start

```bash
# Check Node.js version (requires >= 18)
node --version

# Reinstall dependencies
cd src-mcp
rm -rf node_modules package-lock.json
npm install
```

### No Wiki Pages Found

```bash
# Verify project structure
ls .wiki/

# Check if pages exist
ls .wiki/entities/
ls .wiki/concepts/
```

### Vector Search Not Working

Vector search requires embeddings. If you see warnings about embedding generation:

1. Check if OpenAI API key is configured (for embedding generation)
2. Or use keyword-only search (works without embeddings)

## 📚 Related Documentation

- [MCP Server Guide](../../docs/guides/mcp-server.md) - Detailed architecture and usage
- [Project README](../../README.md) - Main project documentation
- [Model Context Protocol](https://modelcontextprotocol.io) - MCP specification

## 📄 License

GPL-3.0
