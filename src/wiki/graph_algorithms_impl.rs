use std::collections::{HashMap, HashSet, VecDeque};
use crate::wiki::graph::WikiGraph;

pub fn pagerank(graph: &WikiGraph, damping: f64, iterations: usize) -> Vec<(String, f64)> {
    let mut ranks: HashMap<String, f64> = HashMap::new();
    let n = graph.nodes.len();
    
    if n == 0 {
        return Vec::new();
    }
    
    for node in &graph.nodes {
        ranks.insert(node.id.clone(), 1.0 / n as f64);
    }
    
    let mut out_degree: HashMap<String, usize> = HashMap::new();
    for edge in &graph.edges {
        *out_degree.entry(edge.from.clone()).or_insert(0) += 1;
    }
    
    for _ in 0..iterations {
        let mut new_ranks: HashMap<String, f64> = HashMap::new();
        
        for node in &graph.nodes {
            let mut rank = (1.0 - damping) / n as f64;
            
            for edge in &graph.edges {
                if edge.to == node.id {
                    let source_rank = ranks.get(&edge.from).unwrap_or(&0.0);
                    let degree = out_degree.get(&edge.from).unwrap_or(&1);
                    rank += damping * source_rank / *degree as f64;
                }
            }
            
            new_ranks.insert(node.id.clone(), rank);
        }
        
        ranks = new_ranks;
    }
    
    let mut results: Vec<(String, f64)> = ranks.into_iter().collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    results
}

pub fn louvain_communities(graph: &WikiGraph) -> Vec<Vec<String>> {
    let mut communities: HashMap<String, usize> = HashMap::new();
    
    for (i, node) in graph.nodes.iter().enumerate() {
        communities.insert(node.id.clone(), i);
    }
    
    let mut improved = true;
    let mut iteration = 0;
    const MAX_ITERATIONS: usize = 10;
    
    while improved && iteration < MAX_ITERATIONS {
        improved = false;
        iteration += 1;
        
        for node in &graph.nodes {
            let current_community = *communities.get(&node.id).unwrap();
            let mut best_community = current_community;
            let mut best_gain = 0.0;
            
            let neighbors: HashSet<usize> = graph.edges.iter()
                .filter(|e| e.from == node.id)
                .filter_map(|e| communities.get(&e.to).copied())
                .collect();
            
            for &neighbor_community in &neighbors {
                if neighbor_community == current_community {
                    continue;
                }
                
                let gain = modularity_gain(&node.id, current_community, neighbor_community, &communities, graph);
                
                if gain > best_gain {
                    best_gain = gain;
                    best_community = neighbor_community;
                }
            }
            
            if best_community != current_community {
                communities.insert(node.id.clone(), best_community);
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
    
    community_map.into_values().collect()
}

fn modularity_gain(
    node: &str,
    from_community: usize,
    to_community: usize,
    communities: &HashMap<String, usize>,
    graph: &WikiGraph,
) -> f64 {
    let mut gain = 0.0;
    
    for edge in &graph.edges {
        if edge.from == node {
            if let Some(&neighbor_community) = communities.get(&edge.to) {
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

pub fn betweenness_centrality(graph: &WikiGraph) -> HashMap<String, f64> {
    let mut betweenness: HashMap<String, f64> = HashMap::new();
    
    for node in &graph.nodes {
        betweenness.insert(node.id.clone(), 0.0);
    }
    
    let n = graph.nodes.len();
    if n <= 2 {
        return betweenness;
    }
    
    for source in &graph.nodes {
        let mut stack = Vec::new();
        let mut paths: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        let mut dist: HashMap<String, usize> = HashMap::new();
        let mut queue = VecDeque::new();
        
        paths.insert(source.id.clone(), vec![vec![source.id.clone()]]);
        dist.insert(source.id.clone(), 0);
        queue.push_back(source.id.clone());
        
        while let Some(current) = queue.pop_front() {
            stack.push(current.clone());
            
            for edge in &graph.edges {
                if edge.from == current {
                    let current_dist = *dist.get(&current).unwrap();
                    
                    if !dist.contains_key(&edge.to) {
                        dist.insert(edge.to.clone(), current_dist + 1);
                        queue.push_back(edge.to.clone());
                    }
                    
                    if *dist.get(&edge.to).unwrap() == current_dist + 1 {
                        let current_paths = paths.get(&current).unwrap().clone();
                        for mut path in current_paths {
                            path.push(edge.to.clone());
                            paths.entry(edge.to.clone())
                                .or_insert_with(Vec::new)
                                .push(path);
                        }
                    }
                }
            }
        }
        
        for node in &graph.nodes {
            if node.id == source.id {
                continue;
            }
            
            if let Some(node_paths) = paths.get(&node.id) {
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
    
    betweenness
}

pub fn bfs_traversal(graph: &WikiGraph, start: &str, max_depth: usize) -> Vec<String> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut result = Vec::new();
    
    queue.push_back((start.to_string(), 0));
    visited.insert(start.to_string());
    
    while let Some((current, depth)) = queue.pop_front() {
        if depth > max_depth {
            continue;
        }
        
        result.push(current.clone());
        
        for edge in &graph.edges {
            if edge.from == current && !visited.contains(&edge.to) {
                visited.insert(edge.to.clone());
                queue.push_back((edge.to.clone(), depth + 1));
            }
        }
    }
    
    result
}
