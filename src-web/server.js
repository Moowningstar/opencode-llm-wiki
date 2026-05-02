#!/usr/bin/env node

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';
import path from 'path';
import os from 'os';
import { WikiBridge } from './lib/wiki-bridge.js';
import { WikiDatabase } from './lib/database.js';
import { VectorCache } from './lib/vector-cache.js';
import { ContextManager } from './lib/context-manager.js';
import { GraphAnalyzer } from './lib/graph-analyzer.js';
import { SemanticSearch } from './lib/semantic-search.js';
import { generateEmbedding } from './lib/utils.js';

const DEFAULT_PROJECT_ROOT = process.env.LLM_WIKI_PROJECT || path.join(os.homedir(), 'llm-wiki-projects', 'default');

let bridge = null;
let db = null;
let vectorCache = null;
let contextManager = null;
let semanticSearch = null;

async function initBridge(projectRoot = DEFAULT_PROJECT_ROOT) {
  if (!bridge || bridge.projectRoot !== projectRoot) {
    bridge = new WikiBridge(projectRoot);
    
    const wikiDb = new WikiDatabase(path.join(projectRoot, '.llm-wiki'));
    db = wikiDb.getDb();
    vectorCache = new VectorCache(db);
    contextManager = new ContextManager(db, vectorCache, 4000);
    semanticSearch = new SemanticSearch(db, vectorCache);
  }
  return bridge;
}

const server = new Server(
  {
    name: 'llm-wiki-mcp',
    version: '1.0.0',
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
    ],
  };
});

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;
  const projectRoot = args.project || DEFAULT_PROJECT_ROOT;

  try {
    const b = await initBridge(projectRoot);

    switch (name) {
      case 'wiki_read': {
        const content = await b.readPage(args.path, args.scope);
        return {
          content: [{ type: 'text', text: JSON.stringify({ success: true, content }) }],
        };
      }

      case 'wiki_list': {
        const pages = await b.listPages(args.scope);
        return {
          content: [{ type: 'text', text: JSON.stringify({ success: true, pages }) }],
        };
      }

      case 'wiki_search': {
        const results = await b.searchPages(args.query, args.scope);
        return {
          content: [{ type: 'text', text: JSON.stringify({ success: true, results }) }],
        };
      }

      case 'wiki_query_with_context': {
        let queryEmbedding = null;
        try {
          queryEmbedding = await generateEmbedding(args.query);
        } catch (error) {
          console.warn('Embedding generation failed, using keyword-only search:', error.message);
        }

        const maxTokens = args.max_tokens || 4000;
        const result = await contextManager.injectContext(args.query, queryEmbedding);
        
        return {
          content: [{ type: 'text', text: JSON.stringify({ success: true, ...result }) }],
        };
      }

      case 'wiki_get_graph': {
        const graph = await b.getGraphData(args.scope);
        return {
          content: [{ type: 'text', text: JSON.stringify({ success: true, graph }) }],
        };
      }

      case 'wiki_get_index': {
        const index = await b.getIndex(args.scope);
        return {
          content: [{ type: 'text', text: JSON.stringify({ success: true, content: index }) }],
        };
      }

      case 'wiki_get_overview': {
        const overview = await b.getOverview(args.scope);
        return {
          content: [{ type: 'text', text: JSON.stringify({ success: true, content: overview }) }],
        };
      }

      case 'wiki_get_purpose': {
        const purpose = await b.getPurpose();
        return {
          content: [{ type: 'text', text: JSON.stringify({ success: true, content: purpose }) }],
        };
      }

      case 'wiki_graph_insights': {
        const analyzer = new GraphAnalyzer(b);
        const analysisType = args.analysis_type || 'all';
        const insights = await analyzer.analyze(analysisType);
        return {
          content: [{ type: 'text', text: JSON.stringify({ success: true, insights }) }],
        };
      }

      case 'wiki_deep_research': {
        const maxDepth = args.max_depth || 3;
        const maxResults = args.max_results || 10;
        
        let queryEmbedding = null;
        try {
          queryEmbedding = await generateEmbedding(args.query);
        } catch (error) {
          console.warn('Embedding generation failed, using keyword-only search:', error.message);
        }

        const searchResults = await b.searchPages(args.query, args.scope);
        const semanticResults = queryEmbedding && semanticSearch
          ? await semanticSearch.findSimilar(queryEmbedding, maxResults * 2)
          : [];

        const seedPages = new Set();
        searchResults.slice(0, 5).forEach(r => seedPages.add(r.path));
        semanticResults.slice(0, 5).forEach(r => seedPages.add(r.pageId));

        const graph = await b.getGraphData(args.scope);
        const visited = new Set();
        const researchPages = [];
        
        const traverse = async (pagePath, depth) => {
          if (depth > maxDepth || visited.has(pagePath) || researchPages.length >= maxResults) {
            return;
          }
          
          visited.add(pagePath);
          
          try {
            const content = await b.readPage(pagePath, args.scope);
            researchPages.push({ path: pagePath, content, depth });
            
            if (depth < maxDepth) {
              const outgoingLinks = graph.edges
                .filter(e => e.source === pagePath)
                .map(e => e.target);
              
              for (const link of outgoingLinks.slice(0, 3)) {
                await traverse(link, depth + 1);
              }
            }
          } catch (error) {
            console.warn(`Failed to read page ${pagePath}:`, error.message);
          }
        };

        for (const seedPage of Array.from(seedPages).slice(0, 3)) {
          await traverse(seedPage, 0);
        }

        return {
          content: [{ 
            type: 'text', 
            text: JSON.stringify({ 
              success: true, 
              query: args.query,
              pages: researchPages,
              total_pages: researchPages.length,
              max_depth_reached: Math.max(...researchPages.map(p => p.depth)),
            }) 
          }],
        };
      }

      case 'wiki_get_graph': {
        const graph = await b.getGraph(args.scope);
        return {
          content: [{ type: 'text', text: JSON.stringify({ success: true, graph }) }],
        };
      }

      case 'wiki_get_index': {
        const index = await b.getIndex(args.scope);
        return {
          content: [{ type: 'text', text: JSON.stringify({ success: true, index }) }],
        };
      }

      case 'wiki_get_overview': {
        const overview = await b.getOverview(args.scope);
        return {
          content: [{ type: 'text', text: JSON.stringify({ success: true, overview }) }],
        };
      }

      case 'wiki_get_purpose': {
        const purpose = await b.getPurpose(args.scope);
        return {
          content: [{ type: 'text', text: JSON.stringify({ success: true, purpose }) }],
        };
      }

      case 'wiki_get_graph': {
        const graph = await b.getGraphData();
        return {
          content: [{ type: 'text', text: JSON.stringify({ success: true, graph }) }],
        };
      }

      case 'wiki_get_index': {
        const index = await b.getIndex();
        return {
          content: [{ type: 'text', text: JSON.stringify({ success: true, content: index }) }],
        };
      }

      case 'wiki_get_overview': {
        const overview = await b.getOverview();
        return {
          content: [{ type: 'text', text: JSON.stringify({ success: true, content: overview }) }],
        };
      }

      case 'wiki_get_purpose': {
        const purpose = await b.getPurpose();
        return {
          content: [{ type: 'text', text: JSON.stringify({ success: true, content: purpose }) }],
        };
      }

      case 'wiki_graph_insights': {
        const insights = await b.getGraphInsights(args.analysis_type, args.scope);
        return {
          content: [{ type: 'text', text: JSON.stringify({ success: true, insights }) }],
        };
      }

      case 'wiki_deep_research': {
        const research = await b.deepResearch(args.query, args.max_depth, args.max_results, args.scope);
        return {
          content: [{ type: 'text', text: JSON.stringify({ success: true, research }) }],
        };
      }

      case 'wiki_deep_research': {
        const maxDepth = args.max_depth || 3;
        const maxResults = args.max_results || 10;
        
        // Step 1: Get query embedding for semantic search
        let queryEmbedding = null;
        try {
          queryEmbedding = await generateEmbedding(args.query);
        } catch (error) {
          console.warn('Embedding generation failed, using keyword-only search:', error.message);
        }

        // Step 2: Find initial relevant pages
        const searchResults = await b.searchPages(args.query);
        const semanticResults = queryEmbedding && semanticSearch
          ? await semanticSearch.findSimilar(queryEmbedding, maxResults * 2)
          : [];

        // Combine and deduplicate results
        const seedPages = new Set();
        searchResults.slice(0, 5).forEach(r => seedPages.add(r.path));
        semanticResults.slice(0, 5).forEach(r => seedPages.add(r.pageId));

        // Step 3: Graph traversal to find connected knowledge
        const graph = await b.getGraphData();
        const visited = new Set();
        const researchPages = [];
        
        const traverse = async (pagePath, depth) => {
          if (depth > maxDepth || visited.has(pagePath) || researchPages.length >= maxResults) {
            return;
          }
          
          visited.add(pagePath);
          
          try {
            const content = await b.readPage(pagePath);
            researchPages.push({ path: pagePath, content, depth });
            
            // Find linked pages in graph
            if (depth < maxDepth) {
              const outgoingLinks = graph.edges
                .filter(e => e.source === pagePath)
                .map(e => e.target);
              
              for (const link of outgoingLinks.slice(0, 3)) {
                await traverse(link, depth + 1);
              }
            }
          } catch (error) {
            console.warn(`Failed to read page ${pagePath}:`, error.message);
          }
        };

        // Start traversal from seed pages
        for (const seedPage of Array.from(seedPages).slice(0, 3)) {
          await traverse(seedPage, 0);
        }

        return {
          content: [{ 
            type: 'text', 
            text: JSON.stringify({ 
              success: true, 
              query: args.query,
              pages: researchPages,
              total_pages: researchPages.length,
              max_depth_reached: Math.max(...researchPages.map(p => p.depth)),
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
  console.error('LLM Wiki MCP Server running');
}

main().catch(console.error);
