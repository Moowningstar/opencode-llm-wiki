#!/usr/bin/env node

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';
import path from 'path';
import os from 'os';
import { CoreApiClient } from './lib/core-api-client.js';

const DEFAULT_PROJECT_ROOT = process.env.LLM_WIKI_PROJECT || path.join(os.homedir(), 'llm-wiki-projects', 'default');

let apiClient = null;

async function initApiClient() {
  if (!apiClient) {
    apiClient = new CoreApiClient();
    
    const backendAvailable = await apiClient.checkAvailability();
    if (!backendAvailable) {
      const health = await apiClient.health();
      console.error('⚠️  WARNING: Rust backend is not available');
      console.error(`   Backend URL: ${apiClient.baseUrl}`);
      console.error(`   Status: ${health.status}`);
      console.error(`   Message: ${health.message}`);
      console.error('');
      console.error('   To start the backend:');
      console.error('   cd C:\\Users\\Moow\\Projects\\opencode-llm-wiki');
      console.error('   cargo run --release');
      console.error('');
    } else {
      console.error('✓ Rust backend is available');
    }
  }
  return apiClient;
}

const server = new Server(
  {
    name: 'llm-wiki-mcp',
    version: '2.0.0',
  },
  {
    capabilities: {
      tools: {},
    },
  }
);

server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: 'wiki_read',
        description: 'Read a wiki page by path',
        inputSchema: {
          type: 'object',
          properties: {
            path: { type: 'string', description: 'Page path relative to .wiki/ directory' },
            project: { type: 'string', description: 'Project root path (optional)' },
            scope: { type: 'string', description: 'Scope: "global", "project:name", or null (default: global)' },
          },
          required: ['path'],
        },
      },
      {
        name: 'wiki_list',
        description: 'List all wiki pages',
        inputSchema: {
          type: 'object',
          properties: {
            project: { type: 'string', description: 'Project root path (optional)' },
            scope: { type: 'string', description: 'Scope: "global", "project:name", "all" (default: global)' },
          },
        },
      },
      {
        name: 'wiki_search',
        description: 'Search wiki pages by keyword',
        inputSchema: {
          type: 'object',
          properties: {
            query: { type: 'string', description: 'Search query' },
            project: { type: 'string', description: 'Project root path (optional)' },
            scope: { type: 'string', description: 'Scope: "global", "project:name", "all" (default: global)' },
          },
          required: ['query'],
        },
      },
      {
        name: 'wiki_query_with_context',
        description: 'Query wiki with intelligent context injection (keyword + optional vector search)',
        inputSchema: {
          type: 'object',
          properties: {
            query: { type: 'string', description: 'User query' },
            max_tokens: { type: 'number', description: 'Maximum context tokens (default: 4000)' },
            project: { type: 'string', description: 'Project root path (optional)' },
            scope: { type: 'string', description: 'Scope: "global", "project:name", "all" (default: global)' },
          },
          required: ['query'],
        },
      },
      {
        name: 'wiki_get_graph',
        description: 'Get knowledge graph data (nodes and edges)',
        inputSchema: {
          type: 'object',
          properties: {
            project: { type: 'string', description: 'Project root path (optional)' },
            scope: { type: 'string', description: 'Scope: "global", "project:name", "all" (default: global)' },
          },
        },
      },
      {
        name: 'wiki_get_index',
        description: 'Get index.md (content catalog)',
        inputSchema: {
          type: 'object',
          properties: {
            project: { type: 'string', description: 'Project root path (optional)' },
            scope: { type: 'string', description: 'Scope: "global", "project:name", "all" (default: global)' },
          },
        },
      },
      {
        name: 'wiki_get_overview',
        description: 'Get overview.md (global summary)',
        inputSchema: {
          type: 'object',
          properties: {
            project: { type: 'string', description: 'Project root path (optional)' },
            scope: { type: 'string', description: 'Scope: "global", "project:name", "all" (default: global)' },
          },
        },
      },
      {
        name: 'wiki_get_purpose',
        description: 'Get purpose.md (wiki goals and scope)',
        inputSchema: {
          type: 'object',
          properties: {
            project: { type: 'string', description: 'Project root path (optional)' },
            scope: { type: 'string', description: 'Scope: "global", "project:name" (default: global)' },
          },
        },
      },
      {
        name: 'wiki_graph_insights',
        description: 'Analyze knowledge graph structure and find insights (isolated pages, surprising connections, bridge nodes)',
        inputSchema: {
          type: 'object',
          properties: {
            project: { type: 'string', description: 'Project root path (optional)' },
            scope: { type: 'string', description: 'Scope: "global", "project:name", "all" (default: global)' },
            analysis_type: { 
              type: 'string', 
              description: 'Type of analysis: "isolated", "surprising", "bridges", "stats", or "all" (default)',
              enum: ['isolated', 'surprising', 'bridges', 'stats', 'all']
            },
          },
        },
      },
      {
        name: 'wiki_deep_research',
        description: 'Deep research combining graph traversal, semantic search, and multi-hop reasoning',
        inputSchema: {
          type: 'object',
          properties: {
            query: { type: 'string', description: 'Research query or topic' },
            max_depth: { type: 'number', description: 'Maximum graph traversal depth (default: 3)' },
            max_results: { type: 'number', description: 'Maximum number of pages to include (default: 10)' },
            project: { type: 'string', description: 'Project root path (optional)' },
            scope: { type: 'string', description: 'Scope: "global", "project:name", "all" (default: global)' },
          },
          required: ['query'],
        },
      },
      {
        name: 'wiki_ingest',
        description: 'Ingest documents from a path to generate knowledge graph. Calls Rust backend API to process files and create wiki pages.',
        inputSchema: {
          type: 'object',
          properties: {
            path: { type: 'string', description: 'Path to file or directory to ingest (relative to project root or absolute)' },
            extensions: { 
              type: 'array', 
              items: { type: 'string' },
              description: 'Optional: specific file extensions to include (e.g., ["md", "txt", "pdf"])' 
            },
            recursive: { 
              type: 'boolean', 
              description: 'Whether to recursively scan directories (default: true)' 
            },
          },
          required: ['path'],
        },
      },
    ],
  };
});

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;
  const projectRoot = args.project || DEFAULT_PROJECT_ROOT;

  try {
    const client = await initApiClient();

    switch (name) {
      case 'wiki_read': {
        const result = await client.readPage(args.path);
        return {
          content: [{ type: 'text', text: JSON.stringify(result) }],
        };
      }

      case 'wiki_list': {
        const result = await client.listPages();
        return {
          content: [{ type: 'text', text: JSON.stringify(result) }],
        };
      }

      case 'wiki_search': {
        const result = await client.keywordSearch(args.query);
        return {
          content: [{ type: 'text', text: JSON.stringify(result) }],
        };
      }

      case 'wiki_query_with_context': {
        // For now, use keyword search. Semantic search will be added later
        const result = await client.keywordSearch(args.query);
        return {
          content: [{ type: 'text', text: JSON.stringify(result) }],
        };
      }

      case 'wiki_get_graph': {
        const result = await client.getGraph();
        return {
          content: [{ type: 'text', text: JSON.stringify(result) }],
        };
      }

      case 'wiki_get_index': {
        const result = await client.getIndex();
        return {
          content: [{ type: 'text', text: JSON.stringify(result) }],
        };
      }

      case 'wiki_get_overview': {
        const result = await client.getOverview();
        return {
          content: [{ type: 'text', text: JSON.stringify(result) }],
        };
      }

      case 'wiki_get_purpose': {
        const result = await client.getPurpose();
        return {
          content: [{ type: 'text', text: JSON.stringify(result) }],
        };
      }

      case 'wiki_graph_insights': {
        const result = await client.getGraphInsights(args.analysis_type);
        return {
          content: [{ type: 'text', text: JSON.stringify(result) }],
        };
      }

      case 'wiki_deep_research': {
        const result = await client.deepResearch(args.query, args.max_depth, args.max_results);
        return {
          content: [{ type: 'text', text: JSON.stringify(result) }],
        };
      }

      case 'wiki_ingest': {
        const backendAvailable = await client.checkAvailability();
        if (!backendAvailable) {
          const health = await client.health();
          throw new Error(
            `Rust backend is not available: ${health.message}\n` +
            `Backend URL: ${client.baseUrl}\n` +
            `To start: cargo run --release`
          );
        }

        const ingestPath = path.isAbsolute(args.path) 
          ? args.path 
          : path.join(projectRoot, args.path);

        const recursive = args.recursive !== undefined ? args.recursive : true;
        
        console.error(`[wiki_ingest] Ingesting path: ${ingestPath}`);
        console.error(`[wiki_ingest] Extensions: ${args.extensions || 'default (md, txt)'}`);
        console.error(`[wiki_ingest] Recursive: ${recursive}`);

        const result = await client.ingestPath(ingestPath, args.extensions, recursive);
        
        return {
          content: [{ 
            type: 'text', 
            text: JSON.stringify({ 
              success: result.success,
              pages_processed: result.pages_processed,
              chunks_created: result.chunks_created,
              error: result.error,
              message: result.success 
                ? `Successfully ingested ${result.pages_processed} pages, created ${result.chunks_created} chunks`
                : `Ingest failed: ${result.error}`
            }) 
          }],
        };
      }

      default:
        throw new Error(`Unknown tool: ${name}`);
    }
  } catch (error) {
    return {
      content: [{ type: 'text', text: JSON.stringify({ success: false, error: error.message }) }],
      isError: true,
    };
  }
});

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error('LLM Wiki MCP Server v2.0 running (Rust API backend)');
}

main().catch(console.error);
