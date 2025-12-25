// FSM Graph Operations
// Provides graph manipulation utilities

use super::types::*;
use std::collections::HashMap;

/// FSM Graph structure for efficient lookups
#[derive(Debug, Clone, Default)]
pub struct FSMGraph {
    nodes: HashMap<NodeId, FSMNode>,
    edges: HashMap<EdgeId, FSMEdge>,
    
    // Adjacency lists for fast traversal
    outgoing: HashMap<NodeId, Vec<EdgeId>>,
    incoming: HashMap<NodeId, Vec<EdgeId>>,
}

impl FSMGraph {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Load from a project
    pub fn from_project(project: &FSMProject) -> Self {
        let mut graph = Self::new();
        for node in &project.nodes {
            graph.add_node(node.clone());
        }
        for edge in &project.edges {
            graph.add_edge(edge.clone());
        }
        graph
    }
    
    /// Add a node to the graph
    pub fn add_node(&mut self, node: FSMNode) -> NodeId {
        let id = node.id;
        self.nodes.insert(id, node);
        self.outgoing.entry(id).or_default();
        self.incoming.entry(id).or_default();
        id
    }
    
    /// Remove a node and its connected edges
    pub fn remove_node(&mut self, id: NodeId) -> Option<FSMNode> {
        // Remove connected edges first
        let edges_to_remove: Vec<EdgeId> = self.edges
            .values()
            .filter(|e| e.source == id || e.target == id)
            .map(|e| e.id)
            .collect();
        
        for edge_id in edges_to_remove {
            self.remove_edge(edge_id);
        }
        
        self.outgoing.remove(&id);
        self.incoming.remove(&id);
        self.nodes.remove(&id)
    }
    
    /// Get a node by ID
    pub fn get_node(&self, id: NodeId) -> Option<&FSMNode> {
        self.nodes.get(&id)
    }
    
    /// Get a mutable node by ID
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut FSMNode> {
        self.nodes.get_mut(&id)
    }
    
    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: FSMEdge) -> EdgeId {
        let id = edge.id;
        self.outgoing.entry(edge.source).or_default().push(id);
        self.incoming.entry(edge.target).or_default().push(id);
        self.edges.insert(id, edge);
        id
    }
    
    /// Remove an edge
    pub fn remove_edge(&mut self, id: EdgeId) -> Option<FSMEdge> {
        if let Some(edge) = self.edges.remove(&id) {
            if let Some(out) = self.outgoing.get_mut(&edge.source) {
                out.retain(|e| *e != id);
            }
            if let Some(inc) = self.incoming.get_mut(&edge.target) {
                inc.retain(|e| *e != id);
            }
            Some(edge)
        } else {
            None
        }
    }
    
    /// Get outgoing edges from a node
    pub fn get_outgoing(&self, node_id: NodeId) -> Vec<&FSMEdge> {
        self.outgoing
            .get(&node_id)
            .map(|ids| ids.iter().filter_map(|id| self.edges.get(id)).collect())
            .unwrap_or_default()
    }
    
    /// Get incoming edges to a node
    pub fn get_incoming(&self, node_id: NodeId) -> Vec<&FSMEdge> {
        self.incoming
            .get(&node_id)
            .map(|ids| ids.iter().filter_map(|id| self.edges.get(id)).collect())
            .unwrap_or_default()
    }
    
    /// Find the initial/start node
    pub fn find_start_node(&self) -> Option<&FSMNode> {
        self.nodes.values().find(|n| {
            n.node_type == NodeType::Input || n.label.to_uppercase() == "START"
        })
    }
    
    /// Get all nodes
    pub fn nodes(&self) -> impl Iterator<Item = &FSMNode> {
        self.nodes.values()
    }
    
    /// Get all edges
    pub fn edges(&self) -> impl Iterator<Item = &FSMEdge> {
        self.edges.values()
    }
    
    /// Node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    
    /// Edge count
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
    
    /// Find unreachable nodes (for static analysis)
    pub fn find_unreachable(&self) -> Vec<NodeId> {
        let start = match self.find_start_node() {
            Some(n) => n.id,
            None => return self.nodes.keys().copied().collect(),
        };
        
        let mut visited = std::collections::HashSet::new();
        let mut queue = vec![start];
        
        while let Some(node_id) = queue.pop() {
            if visited.insert(node_id) {
                for edge in self.get_outgoing(node_id) {
                    queue.push(edge.target);
                }
            }
        }
        
        self.nodes
            .keys()
            .filter(|id| !visited.contains(id))
            .copied()
            .collect()
    }
    
    /// Find deadlock states (non-final states with no outgoing transitions)
    pub fn find_deadlocks(&self) -> Vec<NodeId> {
        self.nodes
            .values()
            .filter(|n| {
                n.node_type != NodeType::Output 
                    && n.node_type != NodeType::Error
                    && self.get_outgoing(n.id).is_empty()
            })
            .map(|n| n.id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_graph_operations() {
        let mut graph = FSMGraph::new();
        
        let start = FSMNode::new("START", NodeType::Input);
        let process = FSMNode::new("PROCESS", NodeType::Process);
        let end = FSMNode::new("END", NodeType::Output);
        
        let start_id = graph.add_node(start);
        let process_id = graph.add_node(process);
        let end_id = graph.add_node(end);
        
        graph.add_edge(FSMEdge::new(start_id, process_id).with_label("BEGIN"));
        graph.add_edge(FSMEdge::new(process_id, end_id).with_label("COMPLETE"));
        
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 2);
        assert!(graph.find_unreachable().is_empty());
        assert!(graph.find_deadlocks().is_empty());
    }
}
