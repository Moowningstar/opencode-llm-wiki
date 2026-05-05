# Knowledge Graph Setup Guide

## Overview

This guide documents the process of setting up the LLM Wiki knowledge graph from the docs directory and creating the corresponding skill file for AI agent integration.

## Date

2026-05-05

## Problem Encountered

When attempting to ingest documents from the `docs/` directory into the knowledge graph, we encountered a JSON parsing error:

```
Failed to parse index.json
```

### Root Cause

The `WikiIndex` struct required a `metadata` field, but the existing `index.json` file only contained `pages` and `version` fields. This caused deserialization to fail.

### Solution

1. Added `#[serde(default)]` attribute to the `metadata` field in `WikiIndex` struct
2. Implemented `Default` trait for `IndexMetadata` struct

**Code Changes** (`src/wiki/index.rs`):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiIndex {
    pub version: String,
    pub pages: Vec<PageMetadata>,
    #[serde(default)]  // Added this
    pub metadata: IndexMetadata,
}

impl Default for IndexMetadata {
    fn default() -> Self {
        Self {
            total_pages: 0,
            last_updated: Utc::now(),
            categories: Vec::new(),
            top_tags: Vec::new(),
        }
    }
}
```

## Setup Process

### 1. Start the Backend Server

```bash
# Kill any existing server process
taskkill //F //IM llm-wiki-server.exe

# Rebuild after code changes
cargo build --release --bin llm-wiki-server

# Start the server
cargo run --release --bin llm-wiki-server -- --port 19828 > server.log 2>&1 &

# Verify server is running
curl http://127.0.0.1:19828/health
# Expected: {"status":"ok","version":"1.1.0"}
```

### 2. Ingest Documents

```bash
# Using MCP tool
llm-wiki_wiki_ingest({
  path: "docs",
  recursive: true,
  extensions: ["md"]
})
```

### 3. Verify Import Results

```bash
# List all pages
llm-wiki_wiki_list({ scope: "global" })

# Get knowledge graph
llm-wiki_wiki_get_graph({ scope: "global" })

# Analyze graph structure
llm-wiki_wiki_graph_insights({ 
  analysis_type: "stats",
  scope: "global" 
})
```

## Import Results

### Statistics

- **Total Documents**: 23 Markdown files
- **Graph Nodes**: 23 page nodes
- **Graph Edges**: 53 reference relationships
- **Average Degree**: 2.3 (each page connects to 2-3 other pages on average)

### Document Categories

1. **API Documentation** (2 files)
   - quickstart.md
   - server.md

2. **Architecture Documentation** (9 files)
   - 3-layer-refactoring-plan.md
   - cleanup-plan.md
   - final.md
   - knowledge-graph.md
   - overview.md
   - restructure-plan.md
   - ruvector-integration.md
   - ruvector-migration-roadmap.md
   - simplified.md

3. **Guides** (3 files)
   - deployment-windows.md
   - llm-config.md
   - mcp-server.md

4. **Development** (1 file)
   - setup.md

5. **Release Documentation** (4 files)
   - RELEASE_v1.1.0.md
   - RUVECTOR_PHASE1_COMPLETE.md
   - RUVECTOR_PHASE2_COMPLETE.md
   - V1.1.0_VERIFICATION.md

6. **Other** (4 files)
   - README.md
   - LLM+RuVector.md
   - STORAGE_REFACTORING_PLAN.md
   - karpathy-original.md

## Creating the Skill File

To enable AI agents to use the LLM Wiki MCP tools effectively, we created a skill file at `.claude/skills/llm-wiki/SKILL.md`.

### Skill File Structure

```markdown
# LLM Wiki - Knowledge Graph Query & Management

## When to Use This Skill
- Trigger phrases and use cases

## Available Tools
- 11 MCP tools with detailed documentation
- Parameters, use cases, and examples for each

## Workflow Examples
- 4 practical scenarios with step-by-step code

## Best Practices
- Search strategy
- Graph analysis
- Content organization
- Document import

## Current Limitations
- Vector search status
- Fallback behaviors

## Summary
- Capabilities overview
```

### Creating the Skill

```bash
# Create skill directory
mkdir -p .claude/skills/llm-wiki

# Create SKILL.md file
# (See .claude/skills/llm-wiki/SKILL.md for full content)
```

### Verification

```bash
# Load the skill to verify it works
skill(name="llm-wiki")
```

## Available MCP Tools

The skill file documents 11 MCP tools:

1. **wiki_list** - List all pages with metadata
2. **wiki_read** - Read specific page content
3. **wiki_search** - Keyword search across pages
4. **wiki_query_with_context** - Intelligent context query
5. **wiki_get_graph** - Get complete knowledge graph
6. **wiki_graph_insights** - Graph analysis (PageRank, communities, etc.)
7. **wiki_deep_research** - Multi-hop reasoning with graph traversal
8. **wiki_get_index** - Get content catalog
9. **wiki_get_overview** - Get global summary
10. **wiki_get_purpose** - Get wiki goals and scope
11. **wiki_ingest** - Import documents into knowledge base

## Current Limitations

⚠️ **Vector Search Temporarily Unavailable**

During the import process, vector embedding failed with "Failed to store chunks" errors. This means:

- `wiki_query_with_context` falls back to keyword search
- `wiki_deep_research` falls back to graph traversal-based search
- Basic keyword search and graph features are fully functional

The vector storage failure is likely due to:
- Embedding API configuration issues
- Network connectivity to the embedding provider
- API rate limits or quota issues

## Next Steps

### For Users

1. Test the knowledge graph queries:
   ```typescript
   // Find architecture docs
   llm-wiki_wiki_search({ query: "architecture" })
   
   // Get most important pages
   llm-wiki_wiki_graph_insights({ analysis_type: "stats" })
   
   // Read specific document
   llm-wiki_wiki_read({ path: "pages/.wiki-README.md.md" })
   ```

2. Use the skill with AI agents:
   - "Find documentation about RuVector"
   - "Show me the most important pages"
   - "Search for deployment guides"

### For Developers

1. **Fix Vector Search**:
   - Check embedding API configuration in `llm-wiki.jsonc`
   - Verify API key and endpoint
   - Test embedding API connectivity
   - Review server logs for detailed error messages

2. **Enhance Knowledge Graph**:
   - Add more cross-references between documents
   - Create index.md, overview.md, and purpose.md files
   - Run graph analysis to identify isolated pages
   - Add links to connect isolated pages

3. **Improve Documentation**:
   - Add more structured metadata to documents
   - Use consistent heading structures
   - Add tags and categories
   - Create topic-based document clusters

## Troubleshooting

### Server Won't Start

```bash
# Check if port is in use
netstat -ano | findstr :19828

# Kill existing process
taskkill //F //PID <process_id>

# Rebuild and restart
cargo build --release --bin llm-wiki-server
cargo run --release --bin llm-wiki-server -- --port 19828
```

### Import Fails

```bash
# Check server logs
tail -50 server.log

# Verify file paths are correct
ls -la docs/

# Try importing a single file first
llm-wiki_wiki_ingest({
  path: "docs/README.md",
  recursive: false
})
```

### Skill Not Loading

```bash
# Verify skill file exists
ls -la .claude/skills/llm-wiki/SKILL.md

# Check file content
head -20 .claude/skills/llm-wiki/SKILL.md

# Reload skill
skill(name="llm-wiki")
```

## References

- [LLM Wiki README](../README.md)
- [API Server Documentation](../api/server.md)
- [MCP Server Guide](./mcp-server.md)
- [Architecture Overview](../architecture/overview.md)

## Conclusion

The knowledge graph has been successfully set up with 23 documents and 53 relationships. The skill file enables AI agents to effectively query and manage the knowledge base. While vector search is currently unavailable, the graph-based features provide powerful document discovery and analysis capabilities.
