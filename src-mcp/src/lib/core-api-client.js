import fetch from 'node-fetch';

const API_BASE_URL = process.env.LLM_WIKI_API_URL || 'http://127.0.0.1:19828';

export class CoreApiClient {
  constructor(baseUrl = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  async health() {
    try {
      const response = await fetch(`${this.baseUrl}/health`, { 
        signal: AbortSignal.timeout(3000) 
      });
      if (!response.ok) {
        return { status: 'error', message: `HTTP ${response.status}` };
      }
      return await response.json();
    } catch (error) {
      if (error.name === 'AbortError' || error.code === 'ECONNREFUSED') {
        return { 
          status: 'unavailable', 
          message: `Cannot connect to backend at ${this.baseUrl}` 
        };
      }
      return { status: 'error', message: error.message };
    }
  }

  async checkAvailability() {
    const health = await this.health();
    return health.status === 'ok' || health.status === 'healthy';
  }

  async streamChat(config, messages) {
    const response = await fetch(`${this.baseUrl}/api/llm/stream`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ config, messages, stream: true }),
    });

    if (!response.ok) {
      throw new Error(`API error: ${response.statusText}`);
    }

    return response.body;
  }

  async ingestPath(path, extensions = null, recursive = true) {
    const payload = {
      path,
      recursive
    };
    
    if (extensions) {
      payload.extensions = extensions;
    }

    const response = await fetch(`${this.baseUrl}/api/ingest`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Ingest API error: ${response.statusText} - ${error}`);
    }

    return response.json();
  }

  async getConfig(projectPath) {
    const response = await fetch(`${this.baseUrl}/api/config/get`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ project_path: projectPath }),
    });

    return response.json();
  }

  async saveConfig(projectPath, config) {
    const response = await fetch(`${this.baseUrl}/api/config/save`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ project_path: projectPath, config }),
    });

    return response.json();
  }

  async listPages() {
    const response = await fetch(`${this.baseUrl}/api/pages`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({}),
    });

    if (!response.ok) {
      throw new Error(`List pages API error: ${response.statusText}`);
    }

    return response.json();
  }

  async readPage(pagePath) {
    const response = await fetch(`${this.baseUrl}/api/pages/read`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ path: pagePath }),
    });

    if (!response.ok) {
      throw new Error(`Read page API error: ${response.statusText}`);
    }

    return response.json();
  }

  async keywordSearch(query) {
    const response = await fetch(`${this.baseUrl}/api/search/keyword`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ query }),
    });

    if (!response.ok) {
      throw new Error(`Keyword search API error: ${response.statusText}`);
    }

    return response.json();
  }

  async semanticSearch(query, limit = 10) {
    const response = await fetch(`${this.baseUrl}/api/search/semantic`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ query, limit }),
    });

    if (!response.ok) {
      throw new Error(`Semantic search API error: ${response.statusText}`);
    }

    return response.json();
  }

  async getGraph() {
    const response = await fetch(`${this.baseUrl}/api/graph`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({}),
    });

    if (!response.ok) {
      throw new Error(`Get graph API error: ${response.statusText}`);
    }

    return response.json();
  }

  async getGraphInsights(analysisType = 'all') {
    const response = await fetch(`${this.baseUrl}/api/graph/insights`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ analysis_type: analysisType }),
    });

    if (!response.ok) {
      throw new Error(`Graph insights API error: ${response.statusText}`);
    }

    return response.json();
  }

  async deepResearch(query, maxDepth = 3, maxResults = 10) {
    const response = await fetch(`${this.baseUrl}/api/research`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ 
        query, 
        max_depth: maxDepth, 
        max_results: maxResults 
      }),
    });

    if (!response.ok) {
      throw new Error(`Deep research API error: ${response.statusText}`);
    }

    return response.json();
  }

  async getIndex() {
    const response = await fetch(`${this.baseUrl}/api/meta/index`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({}),
    });

    if (!response.ok) {
      throw new Error(`Get index API error: ${response.statusText}`);
    }

    return response.json();
  }

  async getOverview() {
    const response = await fetch(`${this.baseUrl}/api/meta/overview`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({}),
    });

    if (!response.ok) {
      throw new Error(`Get overview API error: ${response.statusText}`);
    }

    return response.json();
  }

  async getPurpose() {
    const response = await fetch(`${this.baseUrl}/api/meta/purpose`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({}),
    });

    if (!response.ok) {
      throw new Error(`Get purpose API error: ${response.statusText}`);
    }

    return response.json();
  }
}
