//! Graph Validator - Graph algorithms for validation and analysis
//!
//! Provides cycle detection, topological sorting, and connectivity analysis.

use super::types::*;
use std::collections::{HashMap, HashSet, VecDeque};

/// Graph validator for FSM canvas
pub struct GraphValidator {
    /// Adjacency list (outgoing edges)
    adjacency: HashMap<String, Vec<String>>,
    /// Reverse adjacency (incoming edges)
    reverse_adj: HashMap<String, Vec<String>>,
}

impl GraphValidator {
    pub fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
            reverse_adj: HashMap::new(),
        }
    }
    
    /// Rebuild the graph from nodes and edges
    pub fn rebuild(&mut self, nodes: &HashMap<String, CanvasNode>, edges: &HashMap<String, CanvasEdge>) {
        self.adjacency.clear();
        self.reverse_adj.clear();
        
        // Initialize with all nodes
        for id in nodes.keys() {
            self.adjacency.insert(id.clone(), Vec::new());
            self.reverse_adj.insert(id.clone(), Vec::new());
        }
        
        // Add edges
        for edge in edges.values() {
            self.add_edge(edge);
        }
    }
    
    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: &CanvasEdge) {
        self.adjacency
            .entry(edge.source.clone())
            .or_insert_with(Vec::new)
            .push(edge.target.clone());
            
        self.reverse_adj
            .entry(edge.target.clone())
            .or_insert_with(Vec::new)
            .push(edge.source.clone());
    }
    
    /// Check if adding an edge would create a cycle
    pub fn would_create_cycle(&self, source: &str, target: &str, _edges: &HashMap<String, CanvasEdge>) -> bool {
        // Can reach source from target? If yes, adding source→target creates cycle
        self.can_reach(target, source)
    }
    
    /// Check if node A can reach node B
    pub fn can_reach(&self, from: &str, to: &str) -> bool {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(from.to_string());
        
        while let Some(node) = queue.pop_front() {
            if node == to {
                return true;
            }
            
            if visited.contains(&node) {
                continue;
            }
            visited.insert(node.clone());
            
            if let Some(neighbors) = self.adjacency.get(&node) {
                for neighbor in neighbors {
                    queue.push_back(neighbor.clone());
                }
            }
        }
        
        false
    }
    
    /// Detect cycles in the graph using DFS
    pub fn find_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();
        
        for node in self.adjacency.keys() {
            if !visited.contains(node) {
                self.dfs_cycle(node, &mut visited, &mut rec_stack, &mut path, &mut cycles);
            }
        }
        
        cycles
    }
    
    fn dfs_cycle(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());
        
        if let Some(neighbors) = self.adjacency.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_cycle(neighbor, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(neighbor) {
                    // Found cycle - extract it
                    if let Some(start) = path.iter().position(|n| n == neighbor) {
                        cycles.push(path[start..].to_vec());
                    }
                }
            }
        }
        
        path.pop();
        rec_stack.remove(node);
    }
    
    /// Topological sort (returns None if graph has cycles)
    pub fn topological_sort(&self) -> Option<Vec<String>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        
        // Initialize in-degrees
        for node in self.adjacency.keys() {
            in_degree.insert(node.clone(), 0);
        }
        
        // Count in-degrees
        for neighbors in self.adjacency.values() {
            for neighbor in neighbors {
                *in_degree.entry(neighbor.clone()).or_insert(0) += 1;
            }
        }
        
        // Start with nodes that have no incoming edges
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| id.clone())
            .collect();
        
        let mut result = Vec::new();
        
        while let Some(node) = queue.pop_front() {
            result.push(node.clone());
            
            if let Some(neighbors) = self.adjacency.get(&node) {
                for neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }
        
        // If not all nodes are in result, there's a cycle
        if result.len() == self.adjacency.len() {
            Some(result)
        } else {
            None
        }
    }
    
    /// Find nodes with no incoming edges (start nodes)
    pub fn find_start_nodes(&self) -> Vec<String> {
        self.reverse_adj
            .iter()
            .filter(|(_, incoming)| incoming.is_empty())
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    /// Find nodes with no outgoing edges (end nodes)
    pub fn find_end_nodes(&self) -> Vec<String> {
        self.adjacency
            .iter()
            .filter(|(_, outgoing)| outgoing.is_empty())
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    /// Find disconnected nodes
    pub fn find_disconnected(&self) -> Vec<String> {
        self.adjacency
            .iter()
            .filter(|(id, outgoing)| {
                outgoing.is_empty() && 
                self.reverse_adj.get(*id).map_or(true, |inc| inc.is_empty())
            })
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    /// Validate the graph
    pub fn validate(&self, nodes: &HashMap<String, CanvasNode>, edges: &HashMap<String, CanvasEdge>) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Check for cycles
        let cycles = self.find_cycles();
        for cycle in &cycles {
            errors.push(ValidationError {
                code: "CYCLE_DETECTED".to_string(),
                message: format!("Cycle detected: {}", cycle.join(" → ")),
                node_ids: cycle.clone(),
            });
        }
        
        // Check for orphaned edges
        for edge in edges.values() {
            if !nodes.contains_key(&edge.source) {
                errors.push(ValidationError {
                    code: "INVALID_EDGE_SOURCE".to_string(),
                    message: format!("Edge {} references missing source node", edge.id),
                    node_ids: vec![edge.source.clone()],
                });
            }
            if !nodes.contains_key(&edge.target) {
                errors.push(ValidationError {
                    code: "INVALID_EDGE_TARGET".to_string(),
                    message: format!("Edge {} references missing target node", edge.id),
                    node_ids: vec![edge.target.clone()],
                });
            }
        }
        
        // Check for disconnected nodes
        let disconnected = self.find_disconnected();
        for id in &disconnected {
            if let Some(node) = nodes.get(id) {
                // Don't warn for input/output nodes
                if node.node_type != NodeType::Input && node.node_type != NodeType::Output {
                    warnings.push(ValidationWarning {
                        code: "DISCONNECTED_NODE".to_string(),
                        message: format!("Node '{}' is not connected to any other nodes", node.label),
                        node_ids: vec![id.clone()],
                    });
                }
            }
        }
        
        // Check for multiple start nodes (might be intentional)
        let start_nodes = self.find_start_nodes();
        if start_nodes.len() > 1 {
            let start_labels: Vec<_> = start_nodes
                .iter()
                .filter_map(|id| nodes.get(id).map(|n| n.label.clone()))
                .collect();
            warnings.push(ValidationWarning {
                code: "MULTIPLE_STARTS".to_string(),
                message: format!("Multiple start nodes detected: {}", start_labels.join(", ")),
                node_ids: start_nodes,
            });
        }
        
        ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
}

impl Default for GraphValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cycle_detection() {
        let mut validator = GraphValidator::new();
        
        // Create a simple cycle: A → B → C → A
        let edges: HashMap<String, CanvasEdge> = [
            ("e1", "A", "B"),
            ("e2", "B", "C"),
            ("e3", "C", "A"),
        ].iter().map(|(id, s, t)| {
            (id.to_string(), CanvasEdge {
                id: id.to_string(),
                source: s.to_string(),
                target: t.to_string(),
                label: None,
                condition: None,
            })
        }).collect();
        
        let nodes: HashMap<String, CanvasNode> = ["A", "B", "C"]
            .iter()
            .map(|id| (id.to_string(), CanvasNode::new(
                id.to_string(),
                id.to_string(),
                NodeType::Process,
                0.0, 0.0,
            )))
            .collect();
        
        validator.rebuild(&nodes, &edges);
        
        let cycles = validator.find_cycles();
        assert!(!cycles.is_empty());
    }
    
    #[test]
    fn test_topological_sort() {
        let mut validator = GraphValidator::new();
        
        // Create a DAG: A → B → D, A → C → D
        let edges: HashMap<String, CanvasEdge> = [
            ("e1", "A", "B"),
            ("e2", "A", "C"),
            ("e3", "B", "D"),
            ("e4", "C", "D"),
        ].iter().map(|(id, s, t)| {
            (id.to_string(), CanvasEdge {
                id: id.to_string(),
                source: s.to_string(),
                target: t.to_string(),
                label: None,
                condition: None,
            })
        }).collect();
        
        let nodes: HashMap<String, CanvasNode> = ["A", "B", "C", "D"]
            .iter()
            .map(|id| (id.to_string(), CanvasNode::new(
                id.to_string(),
                id.to_string(),
                NodeType::Process,
                0.0, 0.0,
            )))
            .collect();
        
        validator.rebuild(&nodes, &edges);
        
        let sorted = validator.topological_sort();
        assert!(sorted.is_some());
        
        let sorted = sorted.unwrap();
        assert_eq!(sorted[0], "A"); // A must be first
        assert_eq!(sorted[3], "D"); // D must be last
    }
}
