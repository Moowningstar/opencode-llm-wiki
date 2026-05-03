use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::{Result, Context};

use std::sync::Arc;

use super::filesystem::WikiFileSystem;
use super::index::IndexManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub weight: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub edge_type: String,
}

pub struct GraphManager {
    fs: Arc<WikiFileSystem>,
}

impl GraphManager {
    pub fn new(fs: Arc<WikiFileSystem>) -> Self {
        Self { fs }
    }

    pub fn load(&self) -> Result<WikiGraph> {
        let graph_path = self.fs.graph_path();
        
        if !graph_path.exists() {
            return Ok(Self::empty_graph());
        }

        let content = fs::read_to_string(&graph_path)
            .context("Failed to read graph.json")?;
        
        serde_json::from_str(&content)
            .context("Failed to parse graph.json")
    }

    pub fn save(&self, graph: &WikiGraph) -> Result<()> {
        let graph_path = self.fs.graph_path();
        let content = serde_json::to_string_pretty(graph)
            .context("Failed to serialize graph")?;
        
        fs::write(&graph_path, content)
            .context("Failed to write graph.json")?;
        
        Ok(())
    }

    pub fn rebuild(&self, index_manager: &IndexManager) -> Result<()> {
        let pages = index_manager.list_pages()?;
        
        let nodes = pages.iter().map(|page| GraphNode {
            id: page.id.clone(),
            node_type: "page".to_string(),
            weight: page.importance as u32,
        }).collect();

        let edges = pages.iter().flat_map(|page| {
            page.links_to.iter().map(move |link| GraphEdge {
                from: page.id.clone(),
                to: link.clone(),
                edge_type: "references".to_string(),
            })
        }).collect();

        let graph = WikiGraph { nodes, edges };
        self.save(&graph)?;

        Ok(())
    }

    pub fn generate_from_index(&self, index: &super::index::WikiIndex) -> WikiGraph {
        let nodes = index.pages.iter().map(|page| GraphNode {
            id: page.id.clone(),
            node_type: "page".to_string(),
            weight: page.importance as u32,
        }).collect();

        let edges = index.pages.iter().flat_map(|page| {
            page.links_to.iter().map(move |link| GraphEdge {
                from: page.id.clone(),
                to: link.clone(),
                edge_type: "references".to_string(),
            })
        }).collect();

        WikiGraph { nodes, edges }
    }

    fn empty_graph() -> WikiGraph {
        WikiGraph {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}
