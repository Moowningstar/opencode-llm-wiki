import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/**
 * WikiBridge - Bridge to nashsu/llm_wiki data structures
 * Reads wiki files, graph data, and project config from the desktop app
 * Supports multi-project architecture: global .wiki/ and projects/name/.wiki/
 */
export class WikiBridge {
  constructor(projectRoot) {
    this.projectRoot = projectRoot;
    this.wikiDir = path.join(projectRoot, '.wiki');
    this.rawDir = path.join(projectRoot, '.raw');
    this.configDir = path.join(projectRoot, '.llm-wiki');
    this.projectsDir = path.join(projectRoot, 'projects');
    this.projectsConfigPath = path.join(this.configDir, 'projects.json');
    
    this.ensureDirectories();
  }

  async ensureDirectories() {
    try {
      await fs.mkdir(this.rawDir, { recursive: true });
      await fs.mkdir(this.projectsDir, { recursive: true });
    } catch (error) {
      console.error(`Failed to create directories: ${error.message}`);
    }
  }

  /**
   * Load projects configuration
   */
  async loadProjects() {
    try {
      const data = await fs.readFile(this.projectsConfigPath, 'utf-8');
      return JSON.parse(data);
    } catch {
      return { projects: [] };
    }
  }

  /**
   * Save projects configuration
   */
  async saveProjects(config) {
    await fs.mkdir(this.configDir, { recursive: true });
    await fs.writeFile(this.projectsConfigPath, JSON.stringify(config, null, 2));
  }

  /**
   * Resolve wiki path: supports global .wiki/ and projects/{name}/.wiki/
   * @param {string} scope - 'global', 'project:{name}', or null (auto-detect)
   * @returns {string} - resolved wiki directory path
   */
  resolveWikiPath(scope = null) {
    if (!scope || scope === 'global') {
      return this.wikiDir;
    }
    
    if (scope.startsWith('project:')) {
      const projectName = scope.substring(8);
      return path.join(this.projectsDir, projectName, '.wiki');
    }
    
    return this.wikiDir;
  }

  /**
   * Read a wiki page by relative path
   */
  async readPage(pagePath, scope = null) {
    const wikiDir = this.resolveWikiPath(scope);
    const fullPath = path.join(wikiDir, pagePath);
    try {
      return await fs.readFile(fullPath, 'utf-8');
    } catch (error) {
      throw new Error(`Failed to read page ${pagePath}: ${error.message}`);
    }
  }

  /**
   * Get index.md (content catalog)
   */
  async getIndex(scope = null) {
    return await this.readPage('index.md', scope);
  }

  /**
   * Get log.md (operation history)
   */
  async getLog(scope = null) {
    return await this.readPage('log.md', scope);
  }

  /**
   * Get overview.md (global summary)
   */
  async getOverview(scope = null) {
    try {
      return await this.readPage('overview.md', scope);
    } catch {
      return null;
    }
  }

  /**
   * Get purpose.md (wiki goals and scope)
   */
  async getPurpose() {
    try {
      return await fs.readFile(path.join(this.projectRoot, 'purpose.md'), 'utf-8');
    } catch {
      return null;
    }
  }

  /**
   * Get schema.md (wiki structure rules)
   */
  async getSchema() {
    try {
      return await fs.readFile(path.join(this.projectRoot, 'schema.md'), 'utf-8');
    } catch {
      return null;
    }
  }

  /**
   * List all wiki pages
   */
  async listPages(scope = null) {
    const wikiDir = this.resolveWikiPath(scope);
    const pages = [];
    
    async function scanDir(dir, relativePath = '') {
      try {
        const entries = await fs.readdir(dir, { withFileTypes: true });
        
        for (const entry of entries) {
          const fullPath = path.join(dir, entry.name);
          const relPath = path.join(relativePath, entry.name);
          
          if (entry.isDirectory()) {
            await scanDir(fullPath, relPath);
          } else if (entry.name.endsWith('.md')) {
            pages.push({
              path: relPath.replace(/\\/g, '/'),
              name: entry.name,
              fullPath
            });
          }
        }
      } catch (error) {
        // Directory may not exist yet
      }
    }
    
    await scanDir(wikiDir);
    return pages;
  }

  /**
   * List all pages across all scopes (global + all projects)
   */
  async listAllPages() {
    const allPages = [];
    
    // Global wiki
    const globalPages = await this.listPages('global');
    allPages.push(...globalPages.map(p => ({ ...p, scope: 'global' })));
    
    // Project wikis
    const projectsConfig = await this.loadProjects();
    for (const project of projectsConfig.projects || []) {
      const projectPages = await this.listPages(`project:${project.name}`);
      allPages.push(...projectPages.map(p => ({ ...p, scope: `project:${project.name}` })));
    }
    
    return allPages;
  }

  /**
   * Get graph data from nashsu's graph cache
   * Format: { nodes: [], edges: [] }
   */
  async getGraphData(scope = null) {
    try {
      const graphPath = path.join(this.configDir, 'graph-cache.json');
      const data = await fs.readFile(graphPath, 'utf-8');
      return JSON.parse(data);
    } catch (error) {
      return await this.buildGraphFromWiki(scope);
    }
  }

  /**
   * Build basic graph structure from wiki files
   * Extracts [[wikilinks]] and frontmatter metadata
   */
  async buildGraphFromWiki(scope = null) {
    const pages = scope === 'all' ? await this.listAllPages() : await this.listPages(scope);
    const nodes = [];
    const edges = [];
    const linkPattern = /\[\[([^\]]+)\]\]/g;

    for (const page of pages) {
      const content = await fs.readFile(page.fullPath, 'utf-8');
      
      // Extract frontmatter
      const frontmatterMatch = content.match(/^---\n([\s\S]*?)\n---/);
      let metadata = {};
      if (frontmatterMatch) {
        const yaml = frontmatterMatch[1];
        // Simple YAML parsing (type, title, sources)
        const typeMatch = yaml.match(/type:\s*(.+)/);
        const titleMatch = yaml.match(/title:\s*(.+)/);
        const sourcesMatch = yaml.match(/sources:\s*\[(.*?)\]/);
        
        metadata = {
          type: typeMatch ? typeMatch[1].trim() : 'unknown',
          title: titleMatch ? titleMatch[1].trim() : page.name.replace('.md', ''),
          sources: sourcesMatch ? sourcesMatch[1].split(',').map(s => s.trim().replace(/['"]/g, '')) : []
        };
      }

      nodes.push({
        id: page.path,
        label: metadata.title || page.name.replace('.md', ''),
        type: metadata.type,
        sources: metadata.sources,
        path: page.path
      });

      // Extract wikilinks
      let match;
      while ((match = linkPattern.exec(content)) !== null) {
        const target = match[1];
        edges.push({
          source: page.path,
          target: target.endsWith('.md') ? target : `${target}.md`,
          type: 'wikilink'
        });
      }
    }

    return { nodes, edges };
  }

  /**
   * Get project config
   */
  async getProjectConfig() {
    try {
      const configPath = path.join(this.configDir, 'project.json');
      const data = await fs.readFile(configPath, 'utf-8');
      return JSON.parse(data);
    } catch {
      return {
        name: path.basename(this.projectRoot),
        created: new Date().toISOString()
      };
    }
  }

  /**
   * Search wiki pages by keyword (simple text search)
   */
  async searchPages(query, scope = null) {
    const pages = scope === 'all' ? await this.listAllPages() : await this.listPages(scope);
    const results = [];
    const queryLower = query.toLowerCase();

    for (const page of pages) {
      const content = await fs.readFile(page.fullPath, 'utf-8');
      const contentLower = content.toLowerCase();
      
      if (contentLower.includes(queryLower)) {
        // Count occurrences
        const matches = (contentLower.match(new RegExp(queryLower, 'g')) || []).length;
        
        results.push({
          path: page.path,
          name: page.name,
          matches,
          score: matches * (page.name.toLowerCase().includes(queryLower) ? 2 : 1)
        });
      }
    }

    return results.sort((a, b) => b.score - a.score);
  }
}
