# LLM Wiki MCP Server & CLI

MCP (Model Context Protocol) server and CLI tool for LLM Wiki - a self-building personal knowledge base.

## Installation

### Global Installation

```bash
cd mcp-server
npm install -g .
```

After installation, the `llm-wiki` command will be available globally.

### Local Development

```bash
cd mcp-server
npm install
node cli.js --help
```

## CLI Usage

### Initialize a New Project

```bash
llm-wiki init <directory> [options]
```

**Options:**
- `-t, --template <type>` - Project template: `research`, `reading`, `personal`, `business`, or `general` (default)

**Examples:**

```bash
# Create a general knowledge base
llm-wiki init my-wiki

# Create a research-focused wiki
llm-wiki init research-notes --template research

# Create a personal growth wiki
llm-wiki init personal-kb --template personal
```

**What it creates:**
- `.raw/` - Drop files here for auto-ingest
- `raw/sources/` - Imported source documents
- `raw/assets/` - Images and media
- `.wiki/` - Generated wiki pages (entities, concepts, sources, etc.)
- `purpose.md` - Wiki goals and scope (template-specific)
- `schema.md` - Structure rules
- `.llm-wiki/` - Internal data (cache, config, database)

### Start MCP Server

```bash
llm-wiki serve [directory]
```

**Examples:**

```bash
# Serve current directory
llm-wiki serve

# Serve specific directory
llm-wiki serve ~/my-wiki

# Serve with custom port (future HTTP mode)
llm-wiki serve ~/my-wiki --port 19828
```

The server runs in stdio mode for MCP protocol communication.

### Scan for New Files

```bash
llm-wiki scan <directory>
```

Manually trigger a scan of the `.raw/` directory for new files. Note: This requires the desktop app or MCP server to be running.

## MCP Integration

### OpenCode Configuration

Add to your `.mcp.json`:

```json
{
  "mcpServers": {
    "llm-wiki": {
      "command": "llm-wiki",
      "args": ["serve", "/path/to/your/wiki"],
      "env": {
        "LLM_WIKI_PROJECT": "/path/to/your/wiki"
      }
    }
  }
}
```

Or use the default location:

```json
{
  "mcpServers": {
    "llm-wiki": {
      "command": "llm-wiki",
      "args": ["serve"],
      "env": {
        "LLM_WIKI_PROJECT": "${HOME}/.opencode-wiki"
      }
    }
  }
}
```

### Available MCP Tools

Once the server is running, these tools are available via MCP:

- `wiki_read` - Read a wiki page by path
- `wiki_list` - List all wiki pages
- `wiki_search` - Search wiki pages by keyword
- `wiki_query_with_context` - Query with intelligent context injection (keyword + optional vector search)
- `wiki_get_graph` - Get knowledge graph data (nodes and edges)
- `wiki_get_index` - Get index.md (content catalog)
- `wiki_get_overview` - Get overview.md (global summary)
- `wiki_get_purpose` - Get purpose.md (wiki goals and scope)
- `wiki_graph_insights` - Analyze knowledge graph structure (isolated pages, surprising connections, bridge nodes)
- `wiki_deep_research` - Deep research combining graph traversal, semantic search, and multi-hop reasoning

## Workflow

### 1. Initialize Project

```bash
llm-wiki init my-research --template research
cd my-research
```

### 2. Add Documents

Drop files into `.raw/` directory:

```bash
cp ~/Downloads/paper.pdf .raw/
cp ~/Documents/notes.md .raw/
```

### 3. Start Server

```bash
llm-wiki serve .
```

### 4. Use via MCP Client

The server will automatically:
- Watch `.raw/` directory for new files
- Process files through two-step ingest pipeline
- Generate wiki pages with entities, concepts, and cross-references
- Build knowledge graph
- Enable querying via MCP tools

## Project Templates

### Research
- Focus: Academic papers, research notes
- Structure: Papers, concepts, methodologies
- Purpose: Track research, identify gaps

### Reading
- Focus: Books, articles, essays
- Structure: Sources, ideas, connections
- Purpose: Organize insights, build reference library

### Personal
- Focus: Personal growth, skills, experiences
- Structure: Reflections, learnings, projects
- Purpose: Track development, document journey

### Business
- Focus: Market analysis, strategy, operations
- Structure: Companies, trends, insights
- Purpose: Strategic decision-making

### General
- Focus: Open-ended knowledge collection
- Structure: Flexible, multi-topic
- Purpose: Personal knowledge base

## Directory Structure

```
my-wiki/
├── purpose.md
├── schema.md
├── .raw/
├── .wiki/
│   ├── index.md             # Content catalog (auto-maintained)
│   ├── log.md               # Operation history (auto-maintained)
│   ├── overview.md          # Global summary (auto-generated)
│   ├── entities/            # Named things (models, companies, people)
│   ├── concepts/            # Ideas, techniques, phenomena
│   ├── sources/             # Source summaries
│   ├── queries/             # Research questions
│   ├── comparisons/         # Side-by-side analysis
│   └── synthesis/           # Cross-cutting summaries
├── purpose.md               # Wiki goals and scope
├── schema.md                # Structure rules
├── .llm-wiki/
│   ├── project.json         # Project metadata
│   ├── ingest-queue.json    # Persistent ingest queue
│   ├── ingest-cache.json    # SHA256 cache for incremental ingest
│   ├── graph-cache.json     # Knowledge graph cache
│   └── wiki.db              # SQLite database (vector embeddings, etc.)
└── README.md                # Project documentation
```

## Features

- **Auto-ingest**: Drop files into `.raw/` and they're automatically processed
- **Two-step chain-of-thought**: LLM analyzes first, then generates wiki pages
- **Incremental cache**: SHA256 hashing skips unchanged files
- **Knowledge graph**: 4-signal relevance model with Louvain community detection
- **Vector search**: Optional LanceDB integration for semantic search
- **MCP protocol**: Standard interface for LLM tool use
- **Multiple templates**: Pre-configured for different use cases

## Development

### Run Tests

```bash
cd mcp-server
npm test
```

### Debug Mode

```bash
DEBUG=* llm-wiki serve my-wiki
```

## License

GPL-3.0

## Links

- Main Project: https://github.com/nashsu/llm_wiki
- MCP Protocol: https://modelcontextprotocol.io
