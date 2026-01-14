//! Selection - Advanced selection methods for canvas nodes
//!
//! Provides polygon-based lasso selection, type-based filtering, and graph traversal selection.

use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Serialize, Deserialize};

use super::types::{CanvasNode, CanvasEdge, NodeType};

/// Lasso selection state for free-form polygon selection
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LassoSelection {
    /// Points defining the lasso polygon
    points: Vec<(f64, f64)>,
    /// Whether the lasso is closed
    is_closed: bool,
}

impl LassoSelection {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Start a new lasso at the given point
    pub fn start(&mut self, x: f64, y: f64) {
        self.points.clear();
        self.points.push((x, y));
        self.is_closed = false;
    }
    
    /// Add a point to the lasso path
    pub fn add_point(&mut self, x: f64, y: f64) {
        // Only add if different from last point (avoid duplicates)
        if let Some(&(lx, ly)) = self.points.last() {
            let dist = ((x - lx).powi(2) + (y - ly).powi(2)).sqrt();
            if dist > 5.0 { // Minimum distance threshold
                self.points.push((x, y));
            }
        } else {
            self.points.push((x, y));
        }
    }
    
    /// Close the lasso polygon
    pub fn close(&mut self) {
        if self.points.len() >= 3 {
            self.is_closed = true;
        }
    }
    
    /// Check if a point is inside the lasso polygon using ray casting algorithm
    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        if !self.is_closed || self.points.len() < 3 {
            return false;
        }
        
        // Ray casting algorithm
        let mut inside = false;
        let n = self.points.len();
        
        let mut j = n - 1;
        for i in 0..n {
            let (xi, yi) = self.points[i];
            let (xj, yj) = self.points[j];
            
            if ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi) {
                inside = !inside;
            }
            j = i;
        }
        
        inside
    }
    
    /// Check if a rectangle (node bounds) intersects with or is contained by the lasso
    pub fn contains_rect(&self, x: f64, y: f64, w: f64, h: f64) -> bool {
        // Check if center is inside
        let cx = x + w / 2.0;
        let cy = y + h / 2.0;
        
        if self.contains_point(cx, cy) {
            return true;
        }
        
        // Check all corners
        if self.contains_point(x, y) { return true; }
        if self.contains_point(x + w, y) { return true; }
        if self.contains_point(x, y + h) { return true; }
        if self.contains_point(x + w, y + h) { return true; }
        
        false
    }
    
    /// Get the bounding box of the lasso
    pub fn bounds(&self) -> Option<(f64, f64, f64, f64)> {
        if self.points.is_empty() {
            return None;
        }
        
        let min_x = self.points.iter().map(|p| p.0).fold(f64::MAX, f64::min);
        let max_x = self.points.iter().map(|p| p.0).fold(f64::MIN, f64::max);
        let min_y = self.points.iter().map(|p| p.1).fold(f64::MAX, f64::min);
        let max_y = self.points.iter().map(|p| p.1).fold(f64::MIN, f64::max);
        
        Some((min_x, min_y, max_x, max_y))
    }
    
    /// Get points for rendering
    pub fn get_points(&self) -> &[(f64, f64)] {
        &self.points
    }
    
    /// Check if lasso is valid (closed with enough points)
    pub fn is_valid(&self) -> bool {
        self.is_closed && self.points.len() >= 3
    }
    
    /// Reset the lasso
    pub fn reset(&mut self) {
        self.points.clear();
        self.is_closed = false;
    }
}

/// Selection utilities for graph-based selection
pub struct SelectionUtils;

impl SelectionUtils {
    /// Select nodes of a specific type
    pub fn select_by_type(
        nodes: &HashMap<String, CanvasNode>,
        node_type: NodeType,
    ) -> Vec<String> {
        nodes.iter()
            .filter(|(_, n)| n.node_type == node_type)
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    /// Select all nodes connected to a given node (BFS traversal)
    pub fn select_connected(
        start_node: &str,
        nodes: &HashMap<String, CanvasNode>,
        edges: &HashMap<String, CanvasEdge>,
    ) -> Vec<String> {
        let mut selected = HashSet::new();
        let mut queue = VecDeque::new();
        
        if !nodes.contains_key(start_node) {
            return Vec::new();
        }
        
        queue.push_back(start_node.to_string());
        selected.insert(start_node.to_string());
        
        // Build adjacency
        let mut neighbors: HashMap<String, Vec<String>> = HashMap::new();
        for edge in edges.values() {
            neighbors.entry(edge.source.clone()).or_default().push(edge.target.clone());
            neighbors.entry(edge.target.clone()).or_default().push(edge.source.clone());
        }
        
        while let Some(node) = queue.pop_front() {
            if let Some(adj) = neighbors.get(&node) {
                for neighbor in adj {
                    if !selected.contains(neighbor) && nodes.contains_key(neighbor) {
                        selected.insert(neighbor.clone());
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
        
        selected.into_iter().collect()
    }
    
    /// Select nodes downstream from a node (follow edges forward only)
    pub fn select_downstream(
        start_node: &str,
        nodes: &HashMap<String, CanvasNode>,
        edges: &HashMap<String, CanvasEdge>,
    ) -> Vec<String> {
        let mut selected = HashSet::new();
        let mut queue = VecDeque::new();
        
        if !nodes.contains_key(start_node) {
            return Vec::new();
        }
        
        queue.push_back(start_node.to_string());
        selected.insert(start_node.to_string());
        
        // Build forward adjacency only
        let mut outgoing: HashMap<String, Vec<String>> = HashMap::new();
        for edge in edges.values() {
            outgoing.entry(edge.source.clone()).or_default().push(edge.target.clone());
        }
        
        while let Some(node) = queue.pop_front() {
            if let Some(targets) = outgoing.get(&node) {
                for target in targets {
                    if !selected.contains(target) && nodes.contains_key(target) {
                        selected.insert(target.clone());
                        queue.push_back(target.clone());
                    }
                }
            }
        }
        
        selected.into_iter().collect()
    }
    
    /// Select nodes upstream from a node (follow edges backward only)
    pub fn select_upstream(
        start_node: &str,
        nodes: &HashMap<String, CanvasNode>,
        edges: &HashMap<String, CanvasEdge>,
    ) -> Vec<String> {
        let mut selected = HashSet::new();
        let mut queue = VecDeque::new();
        
        if !nodes.contains_key(start_node) {
            return Vec::new();
        }
        
        queue.push_back(start_node.to_string());
        selected.insert(start_node.to_string());
        
        // Build reverse adjacency only
        let mut incoming: HashMap<String, Vec<String>> = HashMap::new();
        for edge in edges.values() {
            incoming.entry(edge.target.clone()).or_default().push(edge.source.clone());
        }
        
        while let Some(node) = queue.pop_front() {
            if let Some(sources) = incoming.get(&node) {
                for source in sources {
                    if !selected.contains(source) && nodes.contains_key(source) {
                        selected.insert(source.clone());
                        queue.push_back(source.clone());
                    }
                }
            }
        }
        
        selected.into_iter().collect()
    }
    
    /// Select nodes within a lasso polygon
    pub fn select_in_lasso(
        lasso: &LassoSelection,
        nodes: &HashMap<String, CanvasNode>,
    ) -> Vec<String> {
        if !lasso.is_valid() {
            return Vec::new();
        }
        
        nodes.iter()
            .filter(|(_, n)| lasso.contains_rect(n.x, n.y, n.width, n.height))
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    /// Invert selection
    pub fn invert_selection(
        current: &HashSet<String>,
        nodes: &HashMap<String, CanvasNode>,
    ) -> Vec<String> {
        nodes.keys()
            .filter(|id| !current.contains(*id))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lasso_point_in_polygon() {
        let mut lasso = LassoSelection::new();
        
        // Create a square polygon
        lasso.start(0.0, 0.0);
        lasso.add_point(100.0, 0.0);
        lasso.add_point(100.0, 100.0);
        lasso.add_point(0.0, 100.0);
        lasso.close();
        
        // Point inside
        assert!(lasso.contains_point(50.0, 50.0));
        
        // Point outside
        assert!(!lasso.contains_point(150.0, 50.0));
    }
    
    #[test]
    fn test_lasso_rect_containment() {
        let mut lasso = LassoSelection::new();
        
        // Create a large polygon
        lasso.start(0.0, 0.0);
        lasso.add_point(200.0, 0.0);
        lasso.add_point(200.0, 200.0);
        lasso.add_point(0.0, 200.0);
        lasso.close();
        
        // Rect inside
        assert!(lasso.contains_rect(50.0, 50.0, 40.0, 30.0));
        
        // Rect outside
        assert!(!lasso.contains_rect(250.0, 50.0, 40.0, 30.0));
    }
    
    #[test]
    fn test_select_by_type() {
        let mut nodes = HashMap::new();
        nodes.insert("n1".to_string(), CanvasNode::new("n1".to_string(), "A".to_string(), NodeType::State, 0.0, 0.0));
        nodes.insert("n2".to_string(), CanvasNode::new("n2".to_string(), "B".to_string(), NodeType::Initial, 0.0, 0.0));
        nodes.insert("n3".to_string(), CanvasNode::new("n3".to_string(), "C".to_string(), NodeType::State, 0.0, 0.0));
        
        let selected = SelectionUtils::select_by_type(&nodes, NodeType::State);
        assert_eq!(selected.len(), 2);
    }
    
    #[test]
    fn test_select_connected() {
        let mut nodes = HashMap::new();
        nodes.insert("a".to_string(), CanvasNode::new("a".to_string(), "A".to_string(), NodeType::State, 0.0, 0.0));
        nodes.insert("b".to_string(), CanvasNode::new("b".to_string(), "B".to_string(), NodeType::State, 0.0, 0.0));
        nodes.insert("c".to_string(), CanvasNode::new("c".to_string(), "C".to_string(), NodeType::State, 0.0, 0.0));
        nodes.insert("d".to_string(), CanvasNode::new("d".to_string(), "D".to_string(), NodeType::State, 0.0, 0.0));
        
        let mut edges = HashMap::new();
        edges.insert("e1".to_string(), super::super::types::CanvasEdge::new("e1".to_string(), "a".to_string(), "b".to_string(), None));
        edges.insert("e2".to_string(), super::super::types::CanvasEdge::new("e2".to_string(), "b".to_string(), "c".to_string(), None));
        
        let selected = SelectionUtils::select_connected("a", &nodes, &edges);
        assert_eq!(selected.len(), 3); // a, b, c connected; d isolated
    }
}
