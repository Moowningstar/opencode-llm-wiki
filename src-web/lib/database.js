import Database from 'better-sqlite3';
import path from 'path';
import fs from 'fs';

export class WikiDatabase {
  constructor(wikiRoot) {
    this.wikiRoot = wikiRoot;
    this.dbPath = path.join(wikiRoot, '.wiki-index', 'search.db');
    
    const dbDir = path.dirname(this.dbPath);
    if (!fs.existsSync(dbDir)) {
      fs.mkdirSync(dbDir, { recursive: true });
    }
    
    this.db = new Database(this.dbPath);
    this.initSchema();
  }

  initSchema() {
    this.db.exec(`
      CREATE TABLE IF NOT EXISTS pages (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        path TEXT UNIQUE NOT NULL,
        title TEXT NOT NULL,
        content TEXT NOT NULL,
        content_hash TEXT NOT NULL,
        tags TEXT,
        updated_at INTEGER NOT NULL,
        token_count INTEGER DEFAULT 0
      );

      CREATE TABLE IF NOT EXISTS embeddings (
        page_id INTEGER PRIMARY KEY,
        embedding BLOB NOT NULL,
        model TEXT NOT NULL,
        dimension INTEGER NOT NULL,
        created_at INTEGER NOT NULL,
        FOREIGN KEY (page_id) REFERENCES pages(id) ON DELETE CASCADE
      );

      CREATE TABLE IF NOT EXISTS keywords (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        keyword TEXT NOT NULL,
        page_id INTEGER NOT NULL,
        frequency INTEGER DEFAULT 1,
        FOREIGN KEY (page_id) REFERENCES pages(id) ON DELETE CASCADE
      );

      CREATE INDEX IF NOT EXISTS idx_keywords ON keywords(keyword);
      CREATE INDEX IF NOT EXISTS idx_pages_path ON pages(path);
      CREATE INDEX IF NOT EXISTS idx_pages_updated ON pages(updated_at);

      CREATE VIRTUAL TABLE IF NOT EXISTS pages_fts USING fts5(
        path UNINDEXED,
        title,
        content,
        tags,
        tokenize='porter unicode61 remove_diacritics 2'
      );
    `);
  }

  close() {
    this.db.close();
  }

  getDb() {
    return this.db;
  }
}
