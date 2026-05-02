export class KeywordDetector {
  constructor(db) {
    this.db = db;
    this.stopwords = new Set([
      'the', 'a', 'an', 'and', 'or', 'but', 'in', 'on', 'at', 'to', 'for',
      'of', 'with', 'by', 'from', 'as', 'is', 'was', 'are', 'were', 'be',
      'been', 'being', 'have', 'has', 'had', 'do', 'does', 'did', 'will',
      'would', 'should', 'could', 'may', 'might', 'must', 'can',
      '的', '是', '在', '了', '和', '有', '我', '你', '他', '她', '它',
      '们', '这', '那', '个', '中', '上', '下', '来', '去', '说', '要'
    ]);
  }

  extractKeywords(text) {
    const words = text
      .toLowerCase()
      .match(/[\u4e00-\u9fa5]+|[a-z]+/g) || [];
    
    return words
      .filter(w => !this.stopwords.has(w) && w.length > 1)
      .reduce((acc, word) => {
        acc[word] = (acc[word] || 0) + 1;
        return acc;
      }, {});
  }

  indexPageKeywords(pageId, content) {
    const keywords = this.extractKeywords(content);
    
    this.db.prepare('DELETE FROM keywords WHERE page_id = ?').run(pageId);
    
    const insert = this.db.prepare(`
      INSERT INTO keywords (keyword, page_id, frequency)
      VALUES (?, ?, ?)
    `);
    
    for (const [keyword, frequency] of Object.entries(keywords)) {
      insert.run(keyword, pageId, frequency);
    }
  }

  findPagesByKeywords(keywords, limit = 10) {
    if (keywords.length === 0) return [];
    
    const placeholders = keywords.map(() => '?').join(',');
    const rows = this.db.prepare(`
      SELECT 
        p.id, 
        p.path, 
        p.title,
        SUM(k.frequency) as relevance
      FROM keywords k
      JOIN pages p ON k.page_id = p.id
      WHERE k.keyword IN (${placeholders})
      GROUP BY p.id
      ORDER BY relevance DESC
      LIMIT ?
    `).all(...keywords, limit);
    
    return rows;
  }

  getTopKeywords(pageId, limit = 10) {
    return this.db.prepare(`
      SELECT keyword, frequency
      FROM keywords
      WHERE page_id = ?
      ORDER BY frequency DESC
      LIMIT ?
    `).all(pageId, limit);
  }
}
