import fs from 'fs/promises';
import path from 'path';
import { WikiDatabase } from './database.js';
import { VectorCache } from './vector-cache.js';
import { KeywordDetector } from './keyword-detector.js';
import { ContextManager } from './context-manager.js';
import { generateEmbedding, extractTitle, extractTags, computeContentHash, estimateTokens } from './utils.js';

let embeddingAvailable = null;

async function checkEmbeddingAvailability() {
  if (embeddingAvailable !== null) return embeddingAvailable;
  
  try {
    await generateEmbedding('test');
    embeddingAvailable = true;
    console.log('✓ Embedding model available');
  } catch (error) {
    embeddingAvailable = false;
    console.warn('⚠ Embedding model unavailable, running in keyword-only mode');
    console.warn('  To enable semantic search, ensure transformers model is downloaded');
  }
  
  return embeddingAvailable;
}

export class WikiIndexer {
  constructor(wikiRoot) {
    this.wikiRoot = wikiRoot;
    this.wikiDb = new WikiDatabase(wikiRoot);
    this.db = this.wikiDb.getDb();
    this.vectorCache = new VectorCache(this.db);
    this.keywordDetector = new KeywordDetector(this.db);
    this.contextManager = new ContextManager(this.db, this.vectorCache);
  }

  async indexFile(filePath) {
    const relativePath = path.relative(this.wikiRoot, filePath);
    
    if (!relativePath.endsWith('.md')) {
      return;
    }
    
    const content = await fs.readFile(filePath, 'utf-8');
    const contentHash = computeContentHash(content);
    
    const existing = this.db.prepare('SELECT id, content_hash FROM pages WHERE path = ?').get(relativePath);
    
    if (existing && existing.content_hash === contentHash) {
      return;
    }
    
    const title = extractTitle(content);
    const tags = extractTags(content);
    const tokenCount = estimateTokens(content);
    
    this.db.prepare(`
      INSERT OR REPLACE INTO pages (path, title, content, content_hash, token_count, tags, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, datetime('now'))
    `).run(relativePath, title, content, contentHash, tokenCount, tags);
    
    const page = this.db.prepare('SELECT id FROM pages WHERE path = ?').get(relativePath);
    
    const canEmbed = await checkEmbeddingAvailability();
    if (canEmbed) {
      try {
        const embedding = await generateEmbedding(content.substring(0, 5000));
        this.vectorCache.storeEmbedding(page.id, embedding);
      } catch (error) {
        console.warn(`Failed to generate embedding for ${relativePath}:`, error.message);
      }
    }
    
    const keywords = this.extractKeywords(content);
    this.db.prepare('DELETE FROM keywords WHERE page_id = ?').run(page.id);
    
    const insertKeyword = this.db.prepare('INSERT INTO keywords (keyword, page_id, frequency) VALUES (?, ?, ?)');
    for (const [keyword, frequency] of Object.entries(keywords)) {
      insertKeyword.run(keyword, page.id, frequency);
    }
    
    console.error(`Indexed: ${relativePath} (${tokenCount} tokens)`);
  }

  async indexAllFiles() {
    const files = await this.findMarkdownFiles(this.wikiRoot);
    console.error(`Found ${files.length} markdown files to index`);
    
    let indexed = 0;
    for (const file of files) {
      try {
        await this.indexFile(file);
        indexed++;
      } catch (error) {
        console.error(`Failed to index ${file}:`, error.message);
      }
    }
    
    console.error(`Indexing complete: ${indexed}/${files.length} files`);
    return { total: files.length, indexed };
  }

  async findMarkdownFiles(dir) {
    const files = [];
    
    async function scan(currentDir) {
      const entries = await fs.readdir(currentDir, { withFileTypes: true });
      
      for (const entry of entries) {
        const fullPath = path.join(currentDir, entry.name);
        
        if (entry.isDirectory()) {
          if (entry.name !== '.wiki-index' && entry.name !== 'node_modules' && !entry.name.startsWith('.')) {
            await scan(fullPath);
          }
        } else if (entry.name.endsWith('.md')) {
          files.push(fullPath);
        }
      }
    }
    
    await scan(dir);
    return files;
  }

  async removeFile(filePath) {
    const relativePath = path.relative(this.wikiRoot, filePath).replace(/\\/g, '/');
    
    const page = this.db.prepare('SELECT id FROM pages WHERE path = ?').get(relativePath);
    if (page) {
      this.db.prepare('DELETE FROM pages WHERE id = ?').run(page.id);
      this.db.prepare('DELETE FROM keywords WHERE page_id = ?').run(page.id);
      console.error(`Removed from index: ${relativePath}`);
    }
  }

  extractKeywords(content) {
    const words = content
      .toLowerCase()
      .replace(/[^\w\s\u4e00-\u9fa5]/g, ' ')
      .split(/\s+/)
      .filter(w => w.length > 2);
    
    const freq = {};
    for (const word of words) {
      freq[word] = (freq[word] || 0) + 1;
    }
    
    return freq;
  }

  getStats() {
    const pageStats = this.db.prepare(`
      SELECT 
        COUNT(*) as total_pages,
        SUM(token_count) as total_tokens,
        AVG(token_count) as avg_tokens
      FROM pages
    `).get();
    
    const embeddingStats = this.vectorCache.getStats();
    
    const keywordStats = this.db.prepare(`
      SELECT COUNT(DISTINCT keyword) as unique_keywords
      FROM keywords
    `).get();
    
    return {
      pages: pageStats,
      embeddings: embeddingStats,
      keywords: keywordStats
    };
  }

  close() {
    this.wikiDb.close();
  }
}
