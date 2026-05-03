import path from 'path';
import os from 'os';

let transformers = null;
let embedder = null;
let embeddingAvailable = null;

async function loadTransformers() {
  if (transformers === null) {
    try {
      transformers = await import('@xenova/transformers');
      transformers.env.allowLocalModels = false;
      transformers.env.useBrowserCache = false;
      transformers.env.cacheDir = process.env.TRANSFORMERS_CACHE || path.join(os.homedir(), '.cache', 'huggingface');
      transformers.env.remoteTimeout = 60000;
    } catch (error) {
      console.error('⚠️  @xenova/transformers not available - embedding features disabled');
      transformers = false;
    }
  }
  return transformers;
}

export async function checkEmbeddingAvailability() {
  if (embeddingAvailable !== null) return embeddingAvailable;
  
  const tf = await loadTransformers();
  if (!tf) {
    embeddingAvailable = false;
    return false;
  }
  
  try {
    await initEmbedder();
    embeddingAvailable = true;
  } catch (error) {
    console.error('⚠️  Embedding model unavailable:', error.message);
    embeddingAvailable = false;
  }
  
  return embeddingAvailable;
}

export async function initEmbedder() {
  const tf = await loadTransformers();
  if (!tf) throw new Error('Transformers not available');
  
  if (!embedder) {
    console.error('Loading embedding model...');
    console.error(`Cache directory: ${tf.env.cacheDir}`);
    try {
      embedder = await tf.pipeline(
        'feature-extraction',
        'Xenova/all-MiniLM-L6-v2',
        { 
          quantized: true,
          progress_callback: (progress) => {
            if (progress.status === 'downloading') {
              console.error(`Downloading: ${progress.file} - ${Math.round(progress.progress || 0)}%`);
            }
          }
        }
      );
      console.error('Embedding model loaded');
    } catch (error) {
      console.error('Failed to load embedding model:', error.message);
      throw error;
    }
  }
  return embedder;
}

export async function generateEmbedding(text) {
  const model = await initEmbedder();
  
  const output = await model(text, {
    pooling: 'mean',
    normalize: true
  });
  
  return Array.from(output.data);
}

export function estimateTokens(text) {
  return Math.ceil(text.length / 4);
}

export function extractTitle(content) {
  const match = content.match(/^#\s+(.+)$/m);
  return match ? match[1].trim() : 'Untitled';
}

export function extractTags(content) {
  const tags = content.match(/#[\u4e00-\u9fa5a-zA-Z0-9_-]+/g) || [];
  return tags.join(' ');
}

export function computeContentHash(content) {
  let hash = 0;
  for (let i = 0; i < content.length; i++) {
    const char = content.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash;
  }
  return hash.toString(36);
}
