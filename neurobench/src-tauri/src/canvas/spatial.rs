//! Spatial Index - R-tree based spatial indexing for fast node lookups
//!
//! Provides O(log n) point and rectangle queries for canvas nodes using rstar R-tree.

use super::types::CanvasNode;
use rstar::{RTree, RTreeObject, AABB, PointDistance};
use std::collections::HashMap;

/// R-tree envelope wrapper for nodes
#[derive(Debug, Clone)]
struct NodeEnvelope {
    id: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl RTreeObject for NodeEnvelope {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            [self.x, self.y],
            [self.x + self.width, self.y + self.height],
        )
    }
}

impl PointDistance for NodeEnvelope {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let dx = (point[0] - (self.x + self.width / 2.0)).abs();
        let dy = (point[1] - (self.y + self.height / 2.0)).abs();
        dx * dx + dy * dy
    }
}

/// High-performance R-tree spatial index for O(log n) queries
pub struct SpatialIndex {
    /// R-tree for fast spatial queries
    tree: RTree<NodeEnvelope>,
    /// Quick lookup from ID to envelope data
    id_map: HashMap<String, NodeEnvelope>,
}

impl SpatialIndex {
    pub fn new() -> Self {
        Self {
            tree: RTree::new(),
            id_map: HashMap::new(),
        }
    }

    /// Clear the index
    pub fn clear(&mut self) {
        self.tree = RTree::new();
        self.id_map.clear();
    }

    /// Insert a node into the index
    pub fn insert(&mut self, node: &CanvasNode) {
        let envelope = NodeEnvelope {
            id: node.id.clone(),
            x: node.x,
            y: node.y,
            width: node.width,
            height: node.height,
        };

        self.id_map.insert(node.id.clone(), envelope.clone());
        self.tree.insert(envelope);
    }

    /// Remove a node from the index
    pub fn remove(&mut self, node: &CanvasNode) {
        if let Some(envelope) = self.id_map.remove(&node.id) {
            // Rebuild tree without this envelope (rstar doesn't have efficient remove)
            let envelopes: Vec<_> = self.id_map.values().cloned().collect();
            self.tree = RTree::bulk_load(envelopes);
        }
    }

    /// Bulk insert for initialization (much faster than individual inserts)
    pub fn bulk_insert(&mut self, nodes: &[&CanvasNode]) {
        let envelopes: Vec<NodeEnvelope> = nodes
            .iter()
            .map(|node| {
                let env = NodeEnvelope {
                    id: node.id.clone(),
                    x: node.x,
                    y: node.y,
                    width: node.width,
                    height: node.height,
                };
                self.id_map.insert(node.id.clone(), env.clone());
                env
            })
            .collect();

        self.tree = RTree::bulk_load(envelopes);
    }

    /// Query for node at point - O(log n)
    pub fn query_point(&self, x: f64, y: f64) -> Option<String> {
        // Create a tiny envelope around the point for intersection query
        let point_envelope = AABB::from_corners([x, y], [x, y]);
        
        // Find all nodes whose envelope contains the point
        for envelope in self.tree.locate_in_envelope_intersecting(&point_envelope) {
            // Double-check the point is actually inside the node bounds
            if x >= envelope.x && x <= envelope.x + envelope.width &&
               y >= envelope.y && y <= envelope.y + envelope.height {
                return Some(envelope.id.clone());
            }
        }

        None
    }

    /// Query for nodes in rectangle - O(log n + k)
    pub fn query_rect(&self, x: f64, y: f64, w: f64, h: f64) -> Vec<String> {
        let query_rect = AABB::from_corners([x, y], [x + w, y + h]);
        
        self.tree
            .locate_in_envelope_intersecting(&query_rect)
            .map(|env| env.id.clone())
            .collect()
    }

    /// Query for nodes near a point (within radius) - O(log n)
    pub fn query_radius(&self, x: f64, y: f64, radius: f64) -> Vec<String> {
        self.query_rect(x - radius, y - radius, radius * 2.0, radius * 2.0)
    }

    /// Find k nearest nodes to a point - O(log n)
    pub fn nearest_k(&self, x: f64, y: f64, k: usize) -> Vec<String> {
        self.tree
            .nearest_neighbor_iter(&[x, y])
            .take(k)
            .map(|env| env.id.clone())
            .collect()
    }

    /// Find the single nearest node to a point - O(log n)
    pub fn nearest(&self, x: f64, y: f64) -> Option<String> {
        self.tree
            .nearest_neighbor(&[x, y])
            .map(|env| env.id.clone())
    }

    /// Get total count of indexed nodes
    pub fn len(&self) -> usize {
        self.id_map.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.id_map.is_empty()
    }

    /// Update a node's position (remove + insert)
    pub fn update(&mut self, node: &CanvasNode) {
        self.remove(node);
        self.insert(node);
    }

    /// Batch update positions (more efficient than individual updates)
    pub fn batch_update(&mut self, nodes: &[&CanvasNode]) {
        // Remove all updated nodes from id_map
        for node in nodes {
            self.id_map.remove(&node.id);
        }

        // Re-insert updated nodes
        for node in nodes {
            let env = NodeEnvelope {
                id: node.id.clone(),
                x: node.x,
                y: node.y,
                width: node.width,
                height: node.height,
            };
            self.id_map.insert(node.id.clone(), env);
        }

        // Rebuild tree with all current envelopes
        let envelopes: Vec<_> = self.id_map.values().cloned().collect();
        self.tree = RTree::bulk_load(envelopes);
    }
}

impl Default for SpatialIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::types::NodeType;

    fn create_node(id: &str, x: f64, y: f64) -> CanvasNode {
        CanvasNode::new(
            id.to_string(),
            format!("Node {}", id),
            NodeType::Process,
            x,
            y,
        )
    }

    #[test]
    fn test_insert_and_query() {
        let mut index = SpatialIndex::new();
        let node = create_node("n1", 100.0, 100.0);
        index.insert(&node);

        // Point inside node (default size is 160x80)
        assert_eq!(index.query_point(150.0, 130.0), Some("n1".to_string()));

        // Point outside node
        assert_eq!(index.query_point(50.0, 50.0), None);
    }

    #[test]
    fn test_rect_query() {
        let mut index = SpatialIndex::new();

        for i in 0..5 {
            let node = create_node(&format!("n{}", i), i as f64 * 200.0, 100.0);
            index.insert(&node);
        }

        // Query rect that covers first 3 nodes
        let result = index.query_rect(0.0, 0.0, 500.0, 200.0);
        assert!(result.len() >= 3);
    }

    #[test]
    fn test_bulk_insert() {
        let mut index = SpatialIndex::new();
        let nodes: Vec<CanvasNode> = (0..100)
            .map(|i| create_node(&format!("n{}", i), (i % 10) as f64 * 200.0, (i / 10) as f64 * 150.0))
            .collect();

        let node_refs: Vec<_> = nodes.iter().collect();
        index.bulk_insert(&node_refs);

        assert_eq!(index.len(), 100);
    }

    #[test]
    fn test_nearest() {
        let mut index = SpatialIndex::new();

        // Create a grid of nodes
        for i in 0..9 {
            let x = (i % 3) as f64 * 300.0;
            let y = (i / 3) as f64 * 200.0;
            let node = create_node(&format!("n{}", i), x, y);
            index.insert(&node);
        }

        // Find nearest to center of canvas
        let nearest = index.nearest(450.0, 300.0);
        assert!(nearest.is_some());
    }

    #[test]
    fn test_nearest_k() {
        let mut index = SpatialIndex::new();

        for i in 0..20 {
            let node = create_node(&format!("n{}", i), i as f64 * 50.0, 0.0);
            index.insert(&node);
        }

        let nearest_5 = index.nearest_k(0.0, 0.0, 5);
        assert_eq!(nearest_5.len(), 5);
    }
}
