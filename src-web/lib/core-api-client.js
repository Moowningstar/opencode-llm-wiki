import fetch from 'node-fetch';

const API_BASE_URL = process.env.LLM_WIKI_API_URL || 'http://127.0.0.1:19828';

export class CoreApiClient {
  constructor(baseUrl = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  async health() {
    const response = await fetch(`${this.baseUrl}/health`);
    return response.json();
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

  async ingestFile(projectPath, filePath, config) {
    const response = await fetch(`${this.baseUrl}/api/ingest`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ project_path: projectPath, file_path: filePath, config }),
    });

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
}
