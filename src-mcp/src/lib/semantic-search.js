export class SemanticSearch {
  constructor(vectorCache) {
    this.cache = vectorCache;
  }

  cosineSimilarity(a, b) {
    if (a.length !== b.length) {
      throw new Error('Vectors must have same dimension');
    }
    
    let dotProduct = 0;
    let normA = 0;
    let normB = 0;
    
    for (let i = 0; i < a.length; i++) {
      dotProduct += a[i] * b[i];
      normA += a[i] * a[i];
      normB += b[i] * b[i];
    }
    
    if (normA === 0 || normB === 0) return 0;
    
    return dotProduct / (Math.sqrt(normA) * Math.sqrt(normB));
  }

  findSimilar(queryEmbedding, topK = 5, minSimilarity = 0.3) {
    const allEmbeddings = this.cache.getAllEmbeddings();
    
    if (allEmbeddings.length === 0) {
      return [];
    }
    
    const similarities = allEmbeddings.map(({ pageId, path, title, embedding }) => ({
      pageId,
      path,
      title,
      similarity: this.cosineSimilarity(queryEmbedding, embedding)
    }));
    
    return similarities
      .filter(item => item.similarity >= minSimilarity)
      .sort((a, b) => b.similarity - a.similarity)
      .slice(0, topK);
  }

  findSimilarToPage(pageId, topK = 5, minSimilarity = 0.3) {
    const pageEmbedding = this.cache.getEmbedding(pageId);
    
    if (!pageEmbedding) {
      return [];
    }
    
    return this.findSimilar(pageEmbedding, topK + 1, minSimilarity)
      .filter(item => item.pageId !== pageId)
      .slice(0, topK);
  }

  clusterPages(numClusters = 5) {
    const allEmbeddings = this.cache.getAllEmbeddings();
    
    if (allEmbeddings.length < numClusters) {
      return allEmbeddings.map((item, idx) => ({
        cluster: idx,
        pages: [item]
      }));
    }
    
    const clusters = [];
    for (let i = 0; i < numClusters; i++) {
      clusters.push({
        cluster: i,
        centroid: allEmbeddings[Math.floor(i * allEmbeddings.length / numClusters)].embedding,
        pages: []
      });
    }
    
    for (const item of allEmbeddings) {
      let maxSim = -1;
      let bestCluster = 0;
      
      for (let i = 0; i < clusters.length; i++) {
        const sim = this.cosineSimilarity(item.embedding, clusters[i].centroid);
        if (sim > maxSim) {
          maxSim = sim;
          bestCluster = i;
        }
      }
      
      clusters[bestCluster].pages.push(item);
    }
    
    return clusters;
  }
}
