//! Batch Operations - Execute multiple canvas operations in a single call
//!
//! Reduces IPC round-trips by grouping operations together.
//! All operations in a batch are treated as a single undo step.

use serde::{Serialize, Deserialize};
use super::types::{CanvasNode, CanvasEdge, NodeMove, NodeUpdate};

/// A batch of canvas operations to execute together
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BatchOperation {
    /// Nodes to add
    #[serde(default)]
    pub add_nodes: Vec<CanvasNode>,
    
    /// Node IDs to delete
    #[serde(default)]
    pub delete_nodes: Vec<String>,
    
    /// Nodes to move
    #[serde(default)]
    pub move_nodes: Vec<NodeMove>,
    
    /// Node updates (id, update)
    #[serde(default)]
    pub update_nodes: Vec<(String, NodeUpdate)>,
    
    /// Edges to add (source_id, target_id, label)
    #[serde(default)]
    pub add_edges: Vec<(String, String, Option<String>)>,
    
    /// Edge IDs to delete
    #[serde(default)]
    pub delete_edges: Vec<String>,
}

/// Result of executing a batch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    /// Number of nodes added
    pub nodes_added: usize,
    /// Number of nodes deleted
    pub nodes_deleted: usize,
    /// Number of nodes moved
    pub nodes_moved: usize,
    /// Number of nodes updated
    pub nodes_updated: usize,
    /// IDs of edges added
    pub edges_added: Vec<String>,
    /// Number of edges deleted
    pub edges_deleted: usize,
    /// Errors encountered (non-fatal)
    pub errors: Vec<String>,
}

impl BatchOperation {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the batch
    pub fn add_node(mut self, node: CanvasNode) -> Self {
        self.add_nodes.push(node);
        self
    }

    /// Delete a node in the batch
    pub fn delete_node(mut self, id: impl Into<String>) -> Self {
        self.delete_nodes.push(id.into());
        self
    }

    /// Move a node in the batch
    pub fn move_node(mut self, id: impl Into<String>, x: f64, y: f64) -> Self {
        self.move_nodes.push(NodeMove {
            id: id.into(),
            x,
            y,
        });
        self
    }

    /// Update a node in the batch
    pub fn update_node(mut self, id: impl Into<String>, update: NodeUpdate) -> Self {
        self.update_nodes.push((id.into(), update));
        self
    }

    /// Add an edge in the batch
    pub fn add_edge(
        mut self,
        source: impl Into<String>,
        target: impl Into<String>,
        label: Option<String>,
    ) -> Self {
        self.add_edges.push((source.into(), target.into(), label));
        self
    }

    /// Delete an edge in the batch
    pub fn delete_edge(mut self, id: impl Into<String>) -> Self {
        self.delete_edges.push(id.into());
        self
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.add_nodes.is_empty()
            && self.delete_nodes.is_empty()
            && self.move_nodes.is_empty()
            && self.update_nodes.is_empty()
            && self.add_edges.is_empty()
            && self.delete_edges.is_empty()
    }

    /// Get total number of operations
    pub fn op_count(&self) -> usize {
        self.add_nodes.len()
            + self.delete_nodes.len()
            + self.move_nodes.len()
            + self.update_nodes.len()
            + self.add_edges.len()
            + self.delete_edges.len()
    }
}

impl BatchResult {
    pub fn new() -> Self {
        Self {
            nodes_added: 0,
            nodes_deleted: 0,
            nodes_moved: 0,
            nodes_updated: 0,
            edges_added: Vec::new(),
            edges_deleted: 0,
            errors: Vec::new(),
        }
    }

    /// Check if batch completed without errors
    pub fn success(&self) -> bool {
        self.errors.is_empty()
    }
}

impl Default for BatchResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_builder() {
        let batch = BatchOperation::new()
            .delete_node("n1")
            .delete_node("n2")
            .add_edge("n3", "n4", Some("transition".to_string()));

        assert_eq!(batch.op_count(), 3);
        assert!(!batch.is_empty());
    }
}
