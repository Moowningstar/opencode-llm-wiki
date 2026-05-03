export class VectorCache {
  constructor(db) {
    this.db = db;
  }

  storeEmbedding(pageId, embedding, model = 'MiniLM-L6-v2') {
    const buffer = Buffer.from(new Float32Array(embedding).buffer);
    
    this.db.prepare(`
      INSERT OR REPLACE INTO embeddings (page_id, embedding, model, dimension, created_at)
      VALUES (?, ?, ?, ?, ?)
    `).run(pageId, buffer, model, embedding.length, Date.now());
  }

  getEmbedding(pageId) {
    const row = this.db.prepare(`
      SELECT embedding, dimension FROM embeddings WHERE page_id = ?
    `).get(pageId);
    
    if (!row) return null;
    
    const buffer = Buffer.from(row.embedding);
    return new Float32Array(buffer.buffer, buffer.byteOffset, row.dimension);
  }

  getAllEmbeddings() {
    const rows = this.db.prepare(`
      SELECT e.page_id, e.embedding, e.dimension, p.path, p.title
      FROM embeddings e
      JOIN pages p ON e.page_id = p.id
    `).all();
    
    return rows.map(row => ({
      pageId: row.page_id,
      path: row.path,
      title: row.title,
      embedding: new Float32Array(
        Buffer.from(row.embedding).buffer,
        0,
        row.dimension
      )
    }));
  }

  hasEmbedding(pageId) {
    const row = this.db.prepare(`
      SELECT 1 FROM embeddings WHERE page_id = ?
    `).get(pageId);
    return !!row;
  }

  deleteEmbedding(pageId) {
    this.db.prepare(`
      DELETE FROM embeddings WHERE page_id = ?
    `).run(pageId);
  }

  getStats() {
    const row = this.db.prepare(`
      SELECT 
        COUNT(*) as total,
        AVG(dimension) as avg_dimension,
        model
      FROM embeddings
      GROUP BY model
    `).get();
    
    return row || { total: 0, avg_dimension: 0, model: null };
  }
}
