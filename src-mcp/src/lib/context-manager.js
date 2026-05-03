import { KeywordDetector } from './keyword-detector.js';
import { SemanticSearch } from './semantic-search.js';

export class ContextManager {
  constructor(db, vectorCache, maxTokens = 4000) {
    this.db = db;
    this.cache = vectorCache;
    this.maxTokens = maxTokens;
    this.keywordDetector = new KeywordDetector(db);
    this.semanticSearch = new SemanticSearch(vectorCache);
  }

  extractKeywordsFromQuery(query) {
    const keywords = this.keywordDetector.extractKeywords(query);
    return Object.keys(keywords).slice(0, 10);
  }

  async injectContext(userQuery, queryEmbedding) {
    const keywords = this.extractKeywordsFromQuery(userQuery);
    
    const keywordPages = this.keywordDetector.findPagesByKeywords(keywords, 10);
    
    let similarPages = [];
    try {
      if (queryEmbedding && this.semanticSearch) {
        similarPages = this.semanticSearch.findSimilar(queryEmbedding, 5);
      }
    } catch (error) {
      console.warn('Semantic search unavailable, falling back to keyword-only:', error.message);
    }
    
    const pageIds = new Set([
      ...keywordPages.map(p => p.id),
      ...similarPages.map(p => p.pageId)
    ]);
    
    const pages = Array.from(pageIds).map(id => 
      this.db.prepare('SELECT * FROM pages WHERE id = ?').get(id)
    ).filter(p => p);
    
    const selectedPages = this.selectWithinBudget(pages, this.maxTokens);
    
    return {
      context: this.buildContext(selectedPages),
      pages: selectedPages.map(p => ({
        id: p.id,
        path: p.path,
        title: p.title,
        tokens: p.token_count
      })),
      totalTokens: selectedPages.reduce((sum, p) => sum + (p.token_count || 0), 0),
      keywords,
      similarityScores: similarPages.map(p => ({
        path: p.path,
        similarity: p.similarity
      })),
      mode: similarPages.length > 0 ? 'hybrid' : 'keyword-only'
    };
  }

  selectWithinBudget(pages, maxTokens) {
    const selected = [];
    let totalTokens = 0;
    
    const sorted = pages.sort((a, b) => (b.token_count || 0) - (a.token_count || 0));
    
    for (const page of sorted) {
      const pageTokens = page.token_count || this.estimateTokens(page.content);
      
      if (totalTokens + pageTokens <= maxTokens) {
        selected.push(page);
        totalTokens += pageTokens;
      }
    }
    
    return selected;
  }

  estimateTokens(text) {
    return Math.ceil(text.length / 4);
  }

  buildContext(pages) {
    if (pages.length === 0) {
      return '';
    }
    
    return pages.map(page => `
## [[${page.path}]]

**Title**: ${page.title}
${page.tags ? `**Tags**: ${page.tags}` : ''}

${page.content}
    `.trim()).join('\n\n---\n\n');
  }

  async findRelatedPages(pagePath, limit = 5) {
    const page = this.db.prepare('SELECT * FROM pages WHERE path = ?').get(pagePath);
    
    if (!page) {
      return [];
    }
    
    try {
      if (!this.semanticSearch) {
        return [];
      }
      const related = this.semanticSearch.findSimilarToPage(page.id, limit);
      
      return related.map(item => ({
        path: item.path,
        title: item.title,
        similarity: item.similarity
      }));
    } catch (error) {
      console.warn('Semantic search unavailable:', error.message);
      return [];
    }
  }
}
