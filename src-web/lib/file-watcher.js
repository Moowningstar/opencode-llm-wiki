import chokidar from 'chokidar';
import path from 'path';

export class FileWatcher {
  constructor(wikiRoot, indexer) {
    this.wikiRoot = wikiRoot;
    this.indexer = indexer;
    this.watcher = null;
  }

  start() {
    console.error('Starting file watcher...');
    
    this.watcher = chokidar.watch('**/*.md', {
      cwd: this.wikiRoot,
      ignored: /(^|[\/\\])\.|node_modules|\.wiki-index/,
      persistent: true,
      ignoreInitial: true,
      awaitWriteFinish: {
        stabilityThreshold: 2000,
        pollInterval: 100
      }
    });

    this.watcher
      .on('add', async (relativePath) => {
        const fullPath = path.join(this.wikiRoot, relativePath);
        console.error(`File added: ${relativePath}`);
        try {
          await this.indexer.indexFile(fullPath);
        } catch (error) {
          console.error(`Failed to index ${relativePath}:`, error.message);
        }
      })
      .on('change', async (relativePath) => {
        const fullPath = path.join(this.wikiRoot, relativePath);
        console.error(`File changed: ${relativePath}`);
        try {
          await this.indexer.indexFile(fullPath);
        } catch (error) {
          console.error(`Failed to reindex ${relativePath}:`, error.message);
        }
      })
      .on('unlink', async (relativePath) => {
        const fullPath = path.join(this.wikiRoot, relativePath);
        console.error(`File removed: ${relativePath}`);
        try {
          await this.indexer.removeFile(fullPath);
        } catch (error) {
          console.error(`Failed to remove ${relativePath}:`, error.message);
        }
      })
      .on('error', (error) => {
        console.error('Watcher error:', error);
      });

    console.error('File watcher started');
  }

  stop() {
    if (this.watcher) {
      this.watcher.close();
      console.error('File watcher stopped');
    }
  }
}
