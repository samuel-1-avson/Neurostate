//! Node Pool - Object pooling for efficient node/edge recycling
//!
//! Reduces allocation overhead for bulk operations by reusing objects.

use std::collections::VecDeque;
use crate::canvas::types::{CanvasNode, CanvasEdge, NodeType};

/// Object pool for CanvasNode
pub struct NodePool {
    /// Available nodes for reuse
    pool: VecDeque<CanvasNode>,
    /// Maximum pool size
    max_size: usize,
    /// Statistics
    stats: PoolStats,
}

/// Object pool for CanvasEdge
pub struct EdgePool {
    /// Available edges for reuse
    pool: VecDeque<CanvasEdge>,
    /// Maximum pool size
    max_size: usize,
    /// Statistics
    stats: PoolStats,
}

/// Pool statistics
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Total allocations
    pub allocations: usize,
    /// Total deallocations
    pub deallocations: usize,
    /// Reuses from pool
    pub reuses: usize,
    /// Current pool size
    pub pool_size: usize,
}

impl NodePool {
    /// Create a new node pool
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: VecDeque::with_capacity(max_size),
            max_size,
            stats: PoolStats::default(),
        }
    }

    /// Get a node from the pool or create new
    pub fn acquire(&mut self, id: String, label: String, node_type: NodeType, x: f64, y: f64) -> CanvasNode {
        if let Some(mut node) = self.pool.pop_front() {
            // Reuse existing node
            node.id = id;
            node.label = label;
            node.node_type = node_type;
            node.x = x;
            node.y = y;
            node.entry_action = None;
            node.exit_action = None;
            node.description = None;
            
            self.stats.reuses += 1;
            self.stats.pool_size = self.pool.len();
            node
        } else {
            // Create new node
            self.stats.allocations += 1;
            CanvasNode::new(id, label, node_type, x, y)
        }
    }

    /// Return a node to the pool
    pub fn release(&mut self, node: CanvasNode) {
        self.stats.deallocations += 1;
        
        if self.pool.len() < self.max_size {
            self.pool.push_back(node);
            self.stats.pool_size = self.pool.len();
        }
        // If pool is full, node is dropped
    }

    /// Bulk acquire nodes
    pub fn acquire_batch(&mut self, count: usize, x_start: f64, y_start: f64, spacing: f64) -> Vec<CanvasNode> {
        let mut nodes = Vec::with_capacity(count);
        for i in 0..count {
            let node = self.acquire(
                format!("n{}", i),
                format!("Node {}", i),
                NodeType::State,
                x_start + (i as f64 * spacing),
                y_start,
            );
            nodes.push(node);
        }
        nodes
    }

    /// Bulk release nodes
    pub fn release_batch(&mut self, nodes: Vec<CanvasNode>) {
        for node in nodes {
            self.release(node);
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> &PoolStats {
        &self.stats
    }

    /// Clear pool
    pub fn clear(&mut self) {
        self.pool.clear();
        self.stats.pool_size = 0;
    }

    /// Preallocate nodes
    pub fn preallocate(&mut self, count: usize) {
        let to_add = count.min(self.max_size - self.pool.len());
        for i in 0..to_add {
            let node = CanvasNode::new(
                format!("_pool_{}", i),
                String::new(),
                NodeType::State,
                0.0,
                0.0,
            );
            self.pool.push_back(node);
        }
        self.stats.pool_size = self.pool.len();
    }
}

impl EdgePool {
    /// Create a new edge pool
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: VecDeque::with_capacity(max_size),
            max_size,
            stats: PoolStats::default(),
        }
    }

    /// Get an edge from the pool or create new
    pub fn acquire(&mut self, id: String, source: String, target: String, label: Option<String>) -> CanvasEdge {
        if let Some(mut edge) = self.pool.pop_front() {
            // Reuse existing edge
            edge.id = id;
            edge.source = source;
            edge.target = target;
            edge.label = label;
            edge.condition = None;
            edge.waypoints.clear();
            edge.z_order = 0;
            edge.source_port = None;
            edge.target_port = None;
            
            self.stats.reuses += 1;
            self.stats.pool_size = self.pool.len();
            edge
        } else {
            // Create new edge
            self.stats.allocations += 1;
            CanvasEdge::new(id, source, target, label)
        }
    }

    /// Return an edge to the pool
    pub fn release(&mut self, edge: CanvasEdge) {
        self.stats.deallocations += 1;
        
        if self.pool.len() < self.max_size {
            self.pool.push_back(edge);
            self.stats.pool_size = self.pool.len();
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> &PoolStats {
        &self.stats
    }

    /// Clear pool
    pub fn clear(&mut self) {
        self.pool.clear();
        self.stats.pool_size = 0;
    }
}

impl Default for NodePool {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl Default for EdgePool {
    fn default() -> Self {
        Self::new(2000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_pool_reuse() {
        let mut pool = NodePool::new(10);
        
        // Acquire a node
        let node = pool.acquire("n1".into(), "Test".into(), NodeType::State, 0.0, 0.0);
        assert_eq!(pool.stats().allocations, 1);
        
        // Release it
        pool.release(node);
        assert_eq!(pool.stats().pool_size, 1);
        
        // Acquire again - should reuse
        let _node2 = pool.acquire("n2".into(), "Test2".into(), NodeType::State, 10.0, 10.0);
        assert_eq!(pool.stats().reuses, 1);
        assert_eq!(pool.stats().allocations, 1); // No new allocation
    }

    #[test]
    fn test_edge_pool() {
        let mut pool = EdgePool::new(5);
        
        let edge = pool.acquire("e1".into(), "n1".into(), "n2".into(), None);
        pool.release(edge);
        
        assert_eq!(pool.stats().pool_size, 1);
    }
}
