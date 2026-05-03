# OpenCode LLM Wiki - MCP Server

MCP (Model Context Protocol) server integration for OpenCode LLM Wiki.

## 🚀 Quick Start

### Installation

```bash
cd src-mcp
npm install
```

### Usage in OpenCode

The MCP server is automatically configured via `opencode.jsonc` in the project root:

```jsonc
{
  "mcp": {
    "servers": {
      "llm-wiki": {
        "command": "node",
        "args": ["src-mcp/src/server.js"],
        "env": {
          "LLM_WIKI_PROJECT": "${workspaceFolder}"
        }
      }
    }
  }
}
```

OpenCode will automatically start the MCP server when you open this project.

## 🛠️ Available MCP Tools

The server exposes **10 tools** for interacting with your knowledge base:

### Core Tools

| Tool | Description |
|------|-------------|
| `wiki_read` | Read a wiki page by path |
| `wiki_list` | List all wiki pages |
| `wiki_search` | Search pages by keyword |
| `wiki_query_with_context` | Intelligent context injection (keyword + vector search) |

### Graph Tools

| Tool | Description |
|------|-------------|
| `wiki_get_graph` | Get knowledge graph data (nodes and edges) |
| `wiki_graph_insights` | Analyze graph structure (isolated pages, surprising connections, bridges) |
| `wiki_deep_research` | Multi-hop reasoning with graph traversal |

### Metadata Tools

| Tool | Description |
|------|-------------|
| `wiki_get_index` | Get index.md (content catalog) |
| `wiki_get_overview` | Get overview.md (global summary) |
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

## 🏗️ Architecture

```
src-mcp/
├── src/
│   ├── server.js           # MCP protocol handler
│   ├── cli.js              # CLI tool (llm-wiki command)
│   └── lib/
│       ├── wiki-bridge.js      # Wiki file system bridge
│       ├── database.js         # SQLite database
│       ├── vector-cache.js     # Vector embedding cache
│       ├── context-manager.js  # Smart context injection
│       ├── semantic-search.js  # Vector similarity search
│       ├── graph-analyzer.js   # Knowledge graph analysis
│       ├── indexer.js          # Full-text indexing
│       ├── keyword-detector.js # Keyword extraction
│       └── file-watcher.js     # File change monitoring
└── package.json
```

## 🔧 Development

### Run Server Manually

```bash
cd src-mcp
node src/server.js
```

The server runs in stdio mode for MCP protocol communication.

### Debug Mode

```bash
DEBUG=* node src/server.js
```

### CLI Tool

```bash
# Initialize a new wiki project
node src/cli.js init my-wiki

# Start MCP server
node src/cli.js serve my-wiki
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
