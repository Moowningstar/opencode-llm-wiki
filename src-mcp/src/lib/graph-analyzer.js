export class GraphAnalyzer {
  constructor(graphData) {
    this.nodes = graphData.nodes || [];
    this.edges = graphData.edges || [];
    this.nodeMap = new Map(this.nodes.map(n => [n.id, n]));
  }

  computeRelevance(node1Id, node2Id) {
    const node1 = this.nodeMap.get(node1Id);
    const node2 = this.nodeMap.get(node2Id);
    if (!node1 || !node2) return 0;

    let score = 0;

    const directLink = this.edges.some(e => 
      (e.source === node1Id && e.target === node2Id) ||
      (e.source === node2Id && e.target === node1Id)
    );
    if (directLink) score += 3.0;

    const sources1 = new Set(node1.sources || []);
    const sources2 = new Set(node2.sources || []);
    const sharedSources = [...sources1].filter(s => sources2.has(s)).length;
    if (sharedSources > 0) score += 4.0 * sharedSources;

    const neighbors1 = this.getNeighbors(node1Id);
    const neighbors2 = this.getNeighbors(node2Id);
    const commonNeighbors = neighbors1.filter(n => neighbors2.includes(n));
    if (commonNeighbors.length > 0) {
      const adamicAdar = commonNeighbors.reduce((sum, neighbor) => {
        const degree = this.getNeighbors(neighbor).length;
        return sum + (degree > 0 ? 1 / Math.log(degree + 1) : 0);
      }, 0);
      score += 1.5 * adamicAdar;
    }

    if (node1.type && node2.type && node1.type === node2.type) {
      score += 1.0;
    }

    return score;
  }

  getNeighbors(nodeId) {
    return this.edges
      .filter(e => e.source === nodeId || e.target === nodeId)
      .map(e => e.source === nodeId ? e.target : e.source);
  }

  findIsolatedPages() {
    return this.nodes
      .filter(node => {
        const degree = this.getNeighbors(node.id).length;
        return degree <= 1;
      })
      .map(node => ({
        id: node.id,
        label: node.label,
        type: node.type,
        degree: this.getNeighbors(node.id).length
      }));
  }

  findSurprisingConnections(limit = 10) {
    const connections = [];

    for (const edge of this.edges) {
      const source = this.nodeMap.get(edge.source);
      const target = this.nodeMap.get(edge.target);
      if (!source || !target) continue;

      let surpriseScore = 0;

      if (source.type !== target.type) surpriseScore += 2;

      const sourceDegree = this.getNeighbors(edge.source).length;
      const targetDegree = this.getNeighbors(edge.target).length;
      if (sourceDegree <= 2 && targetDegree >= 10) surpriseScore += 3;
      if (targetDegree <= 2 && sourceDegree >= 10) surpriseScore += 3;

      if (surpriseScore > 0) {
        connections.push({
          source: { id: edge.source, label: source.label, type: source.type },
          target: { id: edge.target, label: target.label, type: target.type },
          surpriseScore,
          relevance: this.computeRelevance(edge.source, edge.target)
        });
      }
    }

    return connections
      .sort((a, b) => b.surpriseScore - a.surpriseScore)
      .slice(0, limit);
  }

  findBridgeNodes() {
    const bridges = [];

    for (const node of this.nodes) {
      const neighbors = this.getNeighbors(node.id);
      if (neighbors.length < 3) continue;

      const neighborTypes = new Set(
        neighbors.map(nId => this.nodeMap.get(nId)?.type).filter(Boolean)
      );

      if (neighborTypes.size >= 3) {
        bridges.push({
          id: node.id,
          label: node.label,
          type: node.type,
          degree: neighbors.length,
          connectedTypes: Array.from(neighborTypes)
        });
      }
    }

    return bridges.sort((a, b) => b.degree - a.degree);
  }

  getGraphStats() {
    const totalNodes = this.nodes.length;
    const totalEdges = this.edges.length;
    const avgDegree = totalNodes > 0 ? (totalEdges * 2) / totalNodes : 0;

    const typeDistribution = {};
    for (const node of this.nodes) {
      typeDistribution[node.type] = (typeDistribution[node.type] || 0) + 1;
    }

    return {
      totalNodes,
      totalEdges,
      avgDegree: avgDegree.toFixed(2),
      typeDistribution
    };
  }
}
