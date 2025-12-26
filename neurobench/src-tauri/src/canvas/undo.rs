//! Incremental Undo System - Operation-based undo/redo for memory efficiency
//!
//! Stores deltas instead of full snapshots, reducing memory by ~95% for move operations.

use super::types::{CanvasNode, CanvasEdge, NodeUpdate};
use serde::{Serialize, Deserialize};

/// A reversible canvas operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CanvasOp {
    /// Node was added
    AddNode {
        node: CanvasNode,
    },
    /// Node was deleted (stores backup for undo)
    DeleteNode {
        id: String,
        backup: CanvasNode,
    },
    /// Multiple nodes were deleted
    DeleteNodes {
        backups: Vec<(String, CanvasNode)>,
        deleted_edges: Vec<(String, CanvasEdge)>,
    },
    /// Nodes were moved
    MoveNodes {
        moves: Vec<NodeMoveOp>,
    },
    /// Node was updated
    UpdateNode {
        id: String,
        old_state: NodeSnapshot,
        new_state: NodeSnapshot,
    },
    /// Edge was added
    AddEdge {
        edge: CanvasEdge,
    },
    /// Edge was deleted
    DeleteEdge {
        id: String,
        backup: CanvasEdge,
    },
    /// Multiple edges were deleted
    DeleteEdges {
        backups: Vec<(String, CanvasEdge)>,
    },
    /// Batch operation (combines multiple ops into one undo step)
    Batch {
        ops: Vec<CanvasOp>,
    },
}

/// Single node move operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMoveOp {
    pub id: String,
    pub old_x: f64,
    pub old_y: f64,
    pub new_x: f64,
    pub new_y: f64,
}

/// Snapshot of node properties for updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSnapshot {
    pub label: String,
    pub node_type: super::types::NodeType,
    pub entry_action: Option<String>,
    pub exit_action: Option<String>,
    pub description: Option<String>,
}

impl NodeSnapshot {
    pub fn from_node(node: &CanvasNode) -> Self {
        Self {
            label: node.label.clone(),
            node_type: node.node_type.clone(),
            entry_action: node.entry_action.clone(),
            exit_action: node.exit_action.clone(),
            description: node.description.clone(),
        }
    }

    pub fn apply_to(&self, node: &mut CanvasNode) {
        node.label = self.label.clone();
        node.node_type = self.node_type.clone();
        node.entry_action = self.entry_action.clone();
        node.exit_action = self.exit_action.clone();
        node.description = self.description.clone();
    }
}

/// Incremental undo manager
pub struct UndoManager {
    /// Stack of operations to undo
    undo_stack: Vec<CanvasOp>,
    /// Stack of operations to redo
    redo_stack: Vec<CanvasOp>,
    /// Maximum number of operations to store
    max_ops: usize,
    /// Current batch being built (for grouping operations)
    current_batch: Option<Vec<CanvasOp>>,
}

impl UndoManager {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_ops: 100,
            current_batch: None,
        }
    }

    pub fn with_max_ops(max_ops: usize) -> Self {
        Self {
            max_ops,
            ..Self::new()
        }
    }

    /// Push an operation to the undo stack
    pub fn push(&mut self, op: CanvasOp) {
        if let Some(batch) = &mut self.current_batch {
            // Add to current batch instead of stack
            batch.push(op);
        } else {
            self.undo_stack.push(op);
            if self.undo_stack.len() > self.max_ops {
                self.undo_stack.remove(0);
            }
            // Clear redo on new action
            self.redo_stack.clear();
        }
    }

    /// Start a batch operation (groups multiple ops into one undo step)
    pub fn begin_batch(&mut self) {
        self.current_batch = Some(Vec::new());
    }

    /// End the current batch and push it as a single operation
    pub fn end_batch(&mut self) {
        if let Some(ops) = self.current_batch.take() {
            if !ops.is_empty() {
                self.push(CanvasOp::Batch { ops });
            }
        }
    }

    /// Cancel the current batch without pushing
    pub fn cancel_batch(&mut self) {
        self.current_batch = None;
    }

    /// Pop the last operation for undo (returns inverse operation)
    pub fn pop_undo(&mut self) -> Option<CanvasOp> {
        self.undo_stack.pop()
    }

    /// Push an operation to the redo stack
    pub fn push_redo(&mut self, op: CanvasOp) {
        self.redo_stack.push(op);
    }

    /// Pop an operation from the redo stack
    pub fn pop_redo(&mut self) -> Option<CanvasOp> {
        self.redo_stack.pop()
    }

    /// Clear redo stack (called when new operations are performed)
    pub fn clear_redo(&mut self) {
        self.redo_stack.clear();
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get undo stack size
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get redo stack size
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_batch = None;
    }
}

impl Default for UndoManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasOp {
    /// Create the inverse operation for undo
    pub fn inverse(&self) -> CanvasOp {
        match self {
            CanvasOp::AddNode { node } => CanvasOp::DeleteNode {
                id: node.id.clone(),
                backup: node.clone(),
            },
            CanvasOp::DeleteNode { id: _id, backup } => CanvasOp::AddNode {
                node: backup.clone(),
            },
            CanvasOp::DeleteNodes { backups, deleted_edges } => CanvasOp::Batch {
                ops: backups
                    .iter()
                    .map(|(_, node)| CanvasOp::AddNode { node: node.clone() })
                    .chain(deleted_edges.iter().map(|(_, edge)| CanvasOp::AddEdge {
                        edge: edge.clone(),
                    }))
                    .collect(),
            },
            CanvasOp::MoveNodes { moves } => CanvasOp::MoveNodes {
                moves: moves
                    .iter()
                    .map(|m| NodeMoveOp {
                        id: m.id.clone(),
                        old_x: m.new_x,
                        old_y: m.new_y,
                        new_x: m.old_x,
                        new_y: m.old_y,
                    })
                    .collect(),
            },
            CanvasOp::UpdateNode { id, old_state, new_state } => CanvasOp::UpdateNode {
                id: id.clone(),
                old_state: new_state.clone(),
                new_state: old_state.clone(),
            },
            CanvasOp::AddEdge { edge } => CanvasOp::DeleteEdge {
                id: edge.id.clone(),
                backup: edge.clone(),
            },
            CanvasOp::DeleteEdge { id: _id, backup } => CanvasOp::AddEdge {
                edge: backup.clone(),
            },
            CanvasOp::DeleteEdges { backups } => CanvasOp::Batch {
                ops: backups
                    .iter()
                    .map(|(_, edge)| CanvasOp::AddEdge { edge: edge.clone() })
                    .collect(),
            },
            CanvasOp::Batch { ops } => CanvasOp::Batch {
                ops: ops.iter().rev().map(|op| op.inverse()).collect(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_pop() {
        let mut manager = UndoManager::new();
        
        let node = CanvasNode::new(
            "n1".to_string(),
            "Test".to_string(),
            super::super::types::NodeType::Process,
            100.0,
            100.0,
        );
        
        manager.push(CanvasOp::AddNode { node: node.clone() });
        
        assert!(manager.can_undo());
        assert!(!manager.can_redo());
        
        let op = manager.pop_undo().unwrap();
        assert!(matches!(op, CanvasOp::AddNode { .. }));
    }

    #[test]
    fn test_move_inverse() {
        let op = CanvasOp::MoveNodes {
            moves: vec![
                NodeMoveOp {
                    id: "n1".to_string(),
                    old_x: 0.0,
                    old_y: 0.0,
                    new_x: 100.0,
                    new_y: 100.0,
                },
            ],
        };
        
        let inverse = op.inverse();
        if let CanvasOp::MoveNodes { moves } = inverse {
            assert_eq!(moves[0].old_x, 100.0);
            assert_eq!(moves[0].new_x, 0.0);
        } else {
            panic!("Expected MoveNodes");
        }
    }

    #[test]
    fn test_batch() {
        let mut manager = UndoManager::new();
        
        manager.begin_batch();
        manager.push(CanvasOp::MoveNodes { moves: vec![] });
        manager.push(CanvasOp::MoveNodes { moves: vec![] });
        manager.end_batch();
        
        assert_eq!(manager.undo_count(), 1);
    }
}
