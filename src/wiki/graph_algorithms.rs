use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRankResult {
    pub node_id: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityResult {
    pub community_id: usize,
    pub nodes: Vec<String>,
    pub cohesion: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathResult {
    pub path: Vec<String>,
    pub length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralityResult {
    pub node_id: String,
    pub degree_centrality: f64,
    pub betweenness_centrality: f64,
}

pub struct GraphAlgorithms {
    adjacency_list: HashMap<String, Vec<String>>,
    reverse_adjacency: HashMap<String, Vec<String>>,
}

impl GraphAlgorithms {
    pub fn new() -> Self {
        Self {
            adjacency_list: HashMap::new(),
            reverse_adjacency: HashMap::new(),
        }
    }

    pub fn add_edge(&mut self, from: &str, to: &str) {
        self.adjacency_list
            .entry(from.to_string())
            .or_insert_with(Vec::new)
            .push(to.to_string());

        self.reverse_adjacency
            .entry(to.to_string())
            .or_insert_with(Vec::new)
            .push(from.to_string());
    }

    pub fn build_from_edges(&mut self, edges: Vec<(String, String)>) {
        for (from, to) in edges {
            self.add_edge(&from, &to);
        }
    }

    pub fn pagerank(&self, iterations: usize, damping: f64) -> Vec<PageRankResult> {
        let nodes: Vec<String> = self.adjacency_list.keys()
            .chain(self.reverse_adjacency.keys())
            .cloned()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let n = nodes.len();
        if n == 0 {
            return Vec::new();
        }

        let mut ranks: HashMap<String, f64> = nodes.iter()
            .map(|node| (node.clone(), 1.0 / n as f64))
            .collect();

        for _ in 0..iterations {
            let mut new_ranks: HashMap<String, f64> = HashMap::new();

            for node in &nodes {
                let mut rank = (1.0 - damping) / n as f64;

                if let Some(incoming) = self.reverse_adjacency.get(node) {
                    for source in incoming {
                        let source_rank = ranks.get(source).unwrap_or(&0.0);
                        let out_degree = self.adjacency_list
                            .get(source)
                            .map(|v| v.len())
                            .unwrap_or(1);
                        rank += damping * source_rank / out_degree as f64;
                    }
                }

                new_ranks.insert(node.clone(), rank);
            }

            ranks = new_ranks;
        }

        let mut results: Vec<PageRankResult> = ranks.into_iter()
            .map(|(node_id, score)| PageRankResult { node_id, score })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }

    pub fn shortest_path(&self, start: &str, end: &str) -> Option<PathResult> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent: HashMap<String, String> = HashMap::new();

        queue.push_back(start.to_string());
        visited.insert(start.to_string());

        while let Some(current) = queue.pop_front() {
            if current == end {
                let mut path = Vec::new();
                let mut node = end.to_string();

                while node != start {
                    path.push(node.clone());
                    node = parent.get(&node).unwrap().clone();
                }
                path.push(start.to_string());
                path.reverse();

                return Some(PathResult {
                    length: path.len() - 1,
                    path,
                });
            }

            if let Some(neighbors) = self.adjacency_list.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        parent.insert(neighbor.clone(), current.clone());
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        None
    }

    pub fn degree_centrality(&self) -> Vec<CentralityResult> {
        let nodes: Vec<String> = self.adjacency_list.keys()
            .chain(self.reverse_adjacency.keys())
            .cloned()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let n = nodes.len();
        if n <= 1 {
            return Vec::new();
        }

        let mut results = Vec::new();

        for node in &nodes {
            let out_degree = self.adjacency_list
                .get(node)
                .map(|v| v.len())
                .unwrap_or(0);
            let in_degree = self.reverse_adjacency
                .get(node)
                .map(|v| v.len())
                .unwrap_or(0);

            let degree = (out_degree + in_degree) as f64;
            // For directed graphs, max degree is 2*(n-1) (n-1 out + n-1 in)
            let degree_centrality = degree / (2.0 * (n - 1) as f64);

            results.push(CentralityResult {
                node_id: node.clone(),
                degree_centrality,
                betweenness_centrality: 0.0,
            });
        }

        results.sort_by(|a, b| b.degree_centrality.partial_cmp(&a.degree_centrality).unwrap());
        results
    }

    pub fn betweenness_centrality(&self) -> Vec<CentralityResult> {
        let nodes: Vec<String> = self.adjacency_list.keys()
            .chain(self.reverse_adjacency.keys())
            .cloned()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let n = nodes.len();
        if n <= 2 {
            return Vec::new();
        }

        let mut betweenness: HashMap<String, f64> = nodes.iter()
            .map(|node| (node.clone(), 0.0))
            .collect();

        for source in &nodes {
            let mut stack = Vec::new();
            let mut paths: HashMap<String, Vec<Vec<String>>> = HashMap::new();
            let mut dist: HashMap<String, usize> = HashMap::new();
            let mut queue = VecDeque::new();

            paths.insert(source.clone(), vec![vec![source.clone()]]);
            dist.insert(source.clone(), 0);
            queue.push_back(source.clone());

            while let Some(current) = queue.pop_front() {
                stack.push(current.clone());

                if let Some(neighbors) = self.adjacency_list.get(&current) {
                    for neighbor in neighbors {
                        let current_dist = *dist.get(&current).unwrap();

                        if !dist.contains_key(neighbor) {
                            dist.insert(neighbor.clone(), current_dist + 1);
                            queue.push_back(neighbor.clone());
                        }

                        if *dist.get(neighbor).unwrap() == current_dist + 1 {
                            let current_paths = paths.get(&current).unwrap().clone();
                            for mut path in current_paths {
                                path.push(neighbor.clone());
                                paths.entry(neighbor.clone())
                                    .or_insert_with(Vec::new)
                                    .push(path);
                            }
                        }
                    }
                }
            }

            for node in &nodes {
                if node == source {
                    continue;
                }

                if let Some(node_paths) = paths.get(node) {
                    for path in node_paths {
                        for intermediate in &path[1..path.len()-1] {
                            *betweenness.get_mut(intermediate).unwrap() += 1.0;
                        }
                    }
                }
            }
        }

        let normalization = ((n - 1) * (n - 2)) as f64;
        for value in betweenness.values_mut() {
            *value /= normalization;
        }

        let mut results: Vec<CentralityResult> = betweenness.into_iter()
            .map(|(node_id, betweenness_centrality)| CentralityResult {
                node_id,
                degree_centrality: 0.0,
                betweenness_centrality,
            })
            .collect();

        results.sort_by(|a, b| b.betweenness_centrality.partial_cmp(&a.betweenness_centrality).unwrap());
        results
    }

    pub fn louvain_communities(&self) -> Vec<CommunityResult> {
        let nodes: Vec<String> = self.adjacency_list.keys()
            .chain(self.reverse_adjacency.keys())
            .cloned()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let mut communities: HashMap<String, usize> = nodes.iter()
            .enumerate()
            .map(|(i, node)| (node.clone(), i))
            .collect();

        let mut improved = true;
        let mut iteration = 0;
        const MAX_ITERATIONS: usize = 10;

        while improved && iteration < MAX_ITERATIONS {
            improved = false;
            iteration += 1;

            for node in &nodes {
                let current_community = *communities.get(node).unwrap();
                let mut best_community = current_community;
                let mut best_gain = 0.0;

                let neighbors: HashSet<usize> = self.adjacency_list
                    .get(node)
                    .map(|v| v.iter().filter_map(|n| communities.get(n).copied()).collect())
                    .unwrap_or_default();

                for &neighbor_community in &neighbors {
                    if neighbor_community == current_community {
                        continue;
                    }

                    let gain = self.modularity_gain(
                        node,
                        current_community,
                        neighbor_community,
                        &communities,
                    );

                    if gain > best_gain {
                        best_gain = gain;
                        best_community = neighbor_community;
                    }
                }

                if best_community != current_community {
                    communities.insert(node.clone(), best_community);
                    improved = true;
                }
            }
        }

        let mut community_map: HashMap<usize, Vec<String>> = HashMap::new();
        for (node, community) in communities {
            community_map.entry(community)
                .or_insert_with(Vec::new)
                .push(node);
        }

        community_map.into_iter()
            .map(|(community_id, nodes)| {
                let cohesion = self.calculate_cohesion(&nodes);
                CommunityResult {
                    community_id,
                    nodes,
                    cohesion,
                }
            })
            .collect()
    }

    fn modularity_gain(
        &self,
        node: &str,
        from_community: usize,
        to_community: usize,
        communities: &HashMap<String, usize>,
    ) -> f64 {
        let mut gain = 0.0;

        if let Some(neighbors) = self.adjacency_list.get(node) {
            for neighbor in neighbors {
                if let Some(&neighbor_community) = communities.get(neighbor) {
                    if neighbor_community == to_community {
                        gain += 1.0;
                    }
                    if neighbor_community == from_community {
                        gain -= 1.0;
                    }
                }
            }
        }

        gain
    }

    fn calculate_cohesion(&self, nodes: &[String]) -> f64 {
        if nodes.len() <= 1 {
            return 1.0;
        }

        let node_set: HashSet<&String> = nodes.iter().collect();
        let mut internal_edges = 0;
        let mut total_edges = 0;

        for node in nodes {
            if let Some(neighbors) = self.adjacency_list.get(node) {
                for neighbor in neighbors {
                    total_edges += 1;
                    if node_set.contains(neighbor) {
                        internal_edges += 1;
                    }
                }
            }
        }

        if total_edges == 0 {
            return 0.0;
        }

        internal_edges as f64 / total_edges as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_simple_graph() -> GraphAlgorithms {
        let mut graph = GraphAlgorithms::new();
        graph.add_edge("A", "B");
        graph.add_edge("A", "C");
        graph.add_edge("B", "C");
        graph.add_edge("C", "A");
        graph
    }

    fn create_disconnected_graph() -> GraphAlgorithms {
        let mut graph = GraphAlgorithms::new();
        graph.add_edge("A", "B");
        graph.add_edge("B", "C");
        graph.add_edge("D", "E");
        graph.add_edge("E", "F");
        graph
    }

    fn create_linear_graph() -> GraphAlgorithms {
        let mut graph = GraphAlgorithms::new();
        graph.add_edge("A", "B");
        graph.add_edge("B", "C");
        graph.add_edge("C", "D");
        graph.add_edge("D", "E");
        graph
    }

    #[test]
    fn test_pagerank_basic() {
        let graph = create_simple_graph();
        let results = graph.pagerank(20, 0.85);
        
        assert!(!results.is_empty());
        assert_eq!(results.len(), 3);
        
        // Verify all scores are positive and sum to approximately 1.0
        let total: f64 = results.iter().map(|r| r.score).sum();
        assert!((total - 1.0).abs() < 0.01);
        
        // Node C should have highest rank (most incoming links)
        assert_eq!(results[0].node_id, "C");
        assert!(results[0].score > 0.3);
    }

    #[test]
    fn test_pagerank_empty_graph() {
        let graph = GraphAlgorithms::new();
        let results = graph.pagerank(20, 0.85);
        assert!(results.is_empty());
    }

    #[test]
    fn test_pagerank_single_node() {
        let mut graph = GraphAlgorithms::new();
        graph.add_edge("A", "A");
        
        let results = graph.pagerank(20, 0.85);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].node_id, "A");
        assert!((results[0].score - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_pagerank_convergence() {
        let graph = create_simple_graph();
        
        let results_10 = graph.pagerank(10, 0.85);
        let results_100 = graph.pagerank(100, 0.85);
        
        // Results should converge
        for (r10, r100) in results_10.iter().zip(results_100.iter()) {
            assert_eq!(r10.node_id, r100.node_id);
            assert!((r10.score - r100.score).abs() < 0.01);
        }
    }

    #[test]
    fn test_shortest_path_basic() {
        let graph = create_linear_graph();
        
        let path = graph.shortest_path("A", "E");
        assert!(path.is_some());
        
        let path = path.unwrap();
        assert_eq!(path.length, 4);
        assert_eq!(path.path, vec!["A", "B", "C", "D", "E"]);
    }

    #[test]
    fn test_shortest_path_no_path() {
        let graph = create_disconnected_graph();
        
        let path = graph.shortest_path("A", "E");
        assert!(path.is_none());
    }

    #[test]
    fn test_shortest_path_same_node() {
        let graph = create_simple_graph();
        
        let path = graph.shortest_path("A", "A");
        assert!(path.is_some());
        
        let path = path.unwrap();
        assert_eq!(path.length, 0);
        assert_eq!(path.path, vec!["A"]);
    }

    #[test]
    fn test_shortest_path_direct_edge() {
        let graph = create_simple_graph();
        
        let path = graph.shortest_path("A", "B");
        assert!(path.is_some());
        
        let path = path.unwrap();
        assert_eq!(path.length, 1);
        assert_eq!(path.path, vec!["A", "B"]);
    }

    #[test]
    fn test_degree_centrality_basic() {
        let graph = create_simple_graph();
        let results = graph.degree_centrality();
        
        assert!(!results.is_empty());
        assert_eq!(results.len(), 3);
        
        for result in &results {
            assert!(result.degree_centrality >= 0.0);
            assert!(result.degree_centrality <= 1.0);
        }
        
        assert!(results[0].node_id == "A" || results[0].node_id == "C");
    }

    #[test]
    fn test_degree_centrality_empty_graph() {
        let graph = GraphAlgorithms::new();
        let results = graph.degree_centrality();
        assert!(results.is_empty());
    }

    #[test]
    fn test_degree_centrality_single_node() {
        let mut graph = GraphAlgorithms::new();
        graph.add_edge("A", "A");
        
        let results = graph.degree_centrality();
        assert!(results.is_empty()); // n <= 1 returns empty
    }

    #[test]
    fn test_betweenness_centrality_basic() {
        let graph = create_linear_graph();
        let results = graph.betweenness_centrality();
        
        assert!(!results.is_empty());
        
        // All centrality scores should be between 0 and 1
        for result in &results {
            assert!(result.betweenness_centrality >= 0.0);
            assert!(result.betweenness_centrality <= 1.0);
        }
        
        // Middle nodes should have higher betweenness
        let node_c = results.iter().find(|r| r.node_id == "C").unwrap();
        assert!(node_c.betweenness_centrality > 0.0);
    }

    #[test]
    fn test_betweenness_centrality_empty_graph() {
        let graph = GraphAlgorithms::new();
        let results = graph.betweenness_centrality();
        assert!(results.is_empty());
    }

    #[test]
    fn test_betweenness_centrality_small_graph() {
        let mut graph = GraphAlgorithms::new();
        graph.add_edge("A", "B");
        
        let results = graph.betweenness_centrality();
        assert!(results.is_empty()); // n <= 2 returns empty
    }

    #[test]
    fn test_louvain_communities_basic() {
        let graph = create_disconnected_graph();
        let communities = graph.louvain_communities();
        
        // Should detect at least 2 communities
        assert!(communities.len() >= 2);
        
        // All nodes should be assigned to a community
        let total_nodes: usize = communities.iter().map(|c| c.nodes.len()).sum();
        assert_eq!(total_nodes, 6);
        
        // Each community should have positive cohesion
        for community in &communities {
            assert!(community.cohesion >= 0.0);
            assert!(community.cohesion <= 1.0);
        }
    }

    #[test]
    fn test_louvain_communities_single_community() {
        let graph = create_simple_graph();
        let communities = graph.louvain_communities();
        
        assert!(!communities.is_empty());
        
        // All nodes should be assigned
        let total_nodes: usize = communities.iter().map(|c| c.nodes.len()).sum();
        assert_eq!(total_nodes, 3);
    }

    #[test]
    fn test_louvain_communities_empty_graph() {
        let graph = GraphAlgorithms::new();
        let communities = graph.louvain_communities();
        assert!(communities.is_empty());
    }

    #[test]
    fn test_build_from_edges() {
        let mut graph = GraphAlgorithms::new();
        let edges = vec![
            ("A".to_string(), "B".to_string()),
            ("B".to_string(), "C".to_string()),
            ("C".to_string(), "A".to_string()),
        ];
        
        graph.build_from_edges(edges);
        
        let results = graph.pagerank(20, 0.85);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_cohesion_calculation() {
        let mut graph = GraphAlgorithms::new();
        // Tightly connected community
        graph.add_edge("A", "B");
        graph.add_edge("B", "C");
        graph.add_edge("C", "A");
        graph.add_edge("A", "C");
        
        let communities = graph.louvain_communities();
        assert!(!communities.is_empty());
        
        // High internal connectivity should result in high cohesion
        let cohesion = communities[0].cohesion;
        assert!(cohesion > 0.5);
    }

    #[test]
    fn test_modularity_gain() {
        let mut graph = GraphAlgorithms::new();
        graph.add_edge("A", "B");
        graph.add_edge("B", "C");
        
        let mut communities = HashMap::new();
        communities.insert("A".to_string(), 0);
        communities.insert("B".to_string(), 0);
        communities.insert("C".to_string(), 1);
        
        let gain = graph.modularity_gain("B", 0, 1, &communities);
        assert!(gain > 0.0); // Moving B to C's community should increase modularity
    }

    #[test]
    fn test_pagerank_with_different_damping() {
        let graph = create_simple_graph();
        
        let results_low = graph.pagerank(20, 0.5);
        let results_high = graph.pagerank(20, 0.95);
        
        assert_eq!(results_low.len(), results_high.len());
        
        // Different damping factors should produce different distributions
        let diff: f64 = results_low.iter()
            .zip(results_high.iter())
            .map(|(r1, r2)| (r1.score - r2.score).abs())
            .sum();
        assert!(diff > 0.01);
    }

    #[test]
    fn test_complex_graph_all_algorithms() {
        let mut graph = GraphAlgorithms::new();
        // Create a more complex graph
        graph.add_edge("A", "B");
        graph.add_edge("A", "C");
        graph.add_edge("B", "D");
        graph.add_edge("C", "D");
        graph.add_edge("D", "E");
        graph.add_edge("E", "F");
        graph.add_edge("F", "D");
        
        // Test all algorithms work on complex graph
        let pagerank = graph.pagerank(20, 0.85);
        assert_eq!(pagerank.len(), 6);
        
        let path = graph.shortest_path("A", "F");
        assert!(path.is_some());
        
        let degree = graph.degree_centrality();
        assert_eq!(degree.len(), 6);
        
        let betweenness = graph.betweenness_centrality();
        assert_eq!(betweenness.len(), 6);
        
        let communities = graph.louvain_communities();
        assert!(!communities.is_empty());
    }
}
