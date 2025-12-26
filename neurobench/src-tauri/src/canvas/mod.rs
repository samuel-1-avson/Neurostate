//! Canvas Engine - High-Performance Graph Canvas for FSM Design
//! 
//! This module provides a Rust-based canvas engine with:
//! - O(log n) spatial lookups via R-tree indexing
//! - Incremental undo/redo with operation deltas
//! - Efficient graph algorithms for validation
//! - Bezier curve calculations for edge routing
//! - Batch operations for bulk manipulation

pub mod types;
pub mod spatial;
pub mod graph;
pub mod path;
pub mod path_cache;
pub mod layout;
pub mod undo;
pub mod batch;
pub mod ports;
pub mod grouping;
pub mod routing;
pub mod viewport;
pub mod layers;
pub mod embedded;
pub mod signal_flow;
pub mod validation;
pub mod analysis;
pub mod node_pool;
pub mod query;
pub mod templates;
pub mod properties;
pub mod connections;
pub mod fsm;
pub mod event_store;
pub mod diff;

#[cfg(test)]
mod benchmarks;
#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

pub use types::*;
pub use spatial::SpatialIndex;
pub use graph::GraphValidator;
pub use path::PathCalculator;
pub use path_cache::PathCache;
pub use layout::LayoutEngine;
pub use undo::{UndoManager, CanvasOp, NodeMoveOp, NodeSnapshot};
pub use batch::{BatchOperation, BatchResult};

/// Canvas Engine - Core state manager for the design canvas
pub struct CanvasEngine {
    /// All nodes indexed by ID
    nodes: HashMap<String, CanvasNode>,
    /// All edges indexed by ID
    edges: HashMap<String, CanvasEdge>,
    /// Spatial index for fast lookups (R-tree)
    spatial: SpatialIndex,
    /// Graph validator
    graph: GraphValidator,
    /// Path calculator
    path: PathCalculator,
    /// Edge path cache for lazy calculation
    path_cache: PathCache,
    /// Layout engine
    layout: LayoutEngine,
    /// Incremental undo manager
    undo_manager: UndoManager,
    /// Current selection
    selection: HashSet<String>,
}

impl CanvasEngine {
    /// Create a new canvas engine
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            spatial: SpatialIndex::new(),
            graph: GraphValidator::new(),
            path: PathCalculator::new(),
            path_cache: PathCache::new(),
            layout: LayoutEngine::new(),
            undo_manager: UndoManager::new(),
            selection: HashSet::new(),
        }
    }
    
    /// Initialize engine with existing state
    pub fn init(&mut self, nodes: Vec<CanvasNode>, edges: Vec<CanvasEdge>) {
        self.nodes.clear();
        self.edges.clear();
        self.spatial.clear();
        
        for node in nodes {
            self.spatial.insert(&node);
            self.nodes.insert(node.id.clone(), node);
        }
        
        for edge in edges {
            self.edges.insert(edge.id.clone(), edge);
        }
        
        self.graph.rebuild(&self.nodes, &self.edges);
    }
    
    /// Undo last action using incremental operations
    pub fn undo(&mut self) -> bool {
        if let Some(op) = self.undo_manager.pop_undo() {
            let inverse = op.inverse();
            self.apply_op(&inverse);
            self.undo_manager.push_redo(op);
            true
        } else {
            false
        }
    }
    
    /// Redo last undone action
    pub fn redo(&mut self) -> bool {
        if let Some(op) = self.undo_manager.pop_redo() {
            self.apply_op(&op);
            self.undo_manager.push(op);
            true
        } else {
            false
        }
    }
    
    /// Apply an operation (used for undo/redo)
    fn apply_op(&mut self, op: &CanvasOp) {
        match op {
            CanvasOp::AddNode { node } => {
                self.spatial.insert(node);
                self.nodes.insert(node.id.clone(), node.clone());
            }
            CanvasOp::DeleteNode { id, .. } => {
                if let Some(node) = self.nodes.remove(id) {
                    self.spatial.remove(&node);
                }
                self.selection.remove(id);
            }
            CanvasOp::DeleteNodes { backups, deleted_edges } => {
                for (id, _) in backups {
                    if let Some(node) = self.nodes.remove(id) {
                        self.spatial.remove(&node);
                    }
                    self.selection.remove(id);
                }
                for (id, _) in deleted_edges {
                    self.edges.remove(id);
                }
                self.graph.rebuild(&self.nodes, &self.edges);
            }
            CanvasOp::MoveNodes { moves } => {
                for m in moves {
                    if let Some(node) = self.nodes.get_mut(&m.id) {
                        self.spatial.remove(node);
                        node.x = m.new_x;
                        node.y = m.new_y;
                        self.spatial.insert(node);
                    }
                }
            }
            CanvasOp::UpdateNode { id, new_state, .. } => {
                if let Some(node) = self.nodes.get_mut(id) {
                    new_state.apply_to(node);
                }
            }
            CanvasOp::AddEdge { edge } => {
                self.edges.insert(edge.id.clone(), edge.clone());
                self.graph.add_edge(edge);
            }
            CanvasOp::DeleteEdge { id, .. } => {
                self.edges.remove(id);
                self.graph.rebuild(&self.nodes, &self.edges);
            }
            CanvasOp::DeleteEdges { backups } => {
                for (id, _) in backups {
                    self.edges.remove(id);
                }
                self.graph.rebuild(&self.nodes, &self.edges);
            }
            CanvasOp::Batch { ops } => {
                for batch_op in ops {
                    self.apply_op(batch_op);
                }
            }
        }
    }
    
    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.undo_manager.can_undo()
    }
    
    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.undo_manager.can_redo()
    }
    
    /// Begin a batch operation (groups multiple ops into one undo step)
    pub fn begin_batch(&mut self) {
        self.undo_manager.begin_batch();
    }
    
    /// End the current batch operation
    pub fn end_batch(&mut self) {
        self.undo_manager.end_batch();
    }
    
    // === Node Operations ===
    
    /// Add a new node
    pub fn add_node(&mut self, node: CanvasNode) -> Result<(), CanvasError> {
        // Record operation for undo
        self.undo_manager.push(CanvasOp::AddNode { node: node.clone() });
        
        self.spatial.insert(&node);
        self.nodes.insert(node.id.clone(), node);
        
        Ok(())
    }
    
    /// Move multiple nodes
    pub fn move_nodes(&mut self, moves: Vec<NodeMove>) -> Result<(), CanvasError> {
        // Build operation with old positions
        let move_ops: Vec<NodeMoveOp> = moves.iter()
            .filter_map(|m| {
                self.nodes.get(&m.id).map(|node| NodeMoveOp {
                    id: m.id.clone(),
                    old_x: node.x,
                    old_y: node.y,
                    new_x: m.x,
                    new_y: m.y,
                })
            })
            .collect();
        
        if !move_ops.is_empty() {
            self.undo_manager.push(CanvasOp::MoveNodes { moves: move_ops });
        }
        
        for m in moves {
            if let Some(node) = self.nodes.get_mut(&m.id) {
                self.spatial.remove(node);
                node.x = m.x;
                node.y = m.y;
                self.spatial.insert(node);
            }
        }
        
        Ok(())
    }
    
    /// Delete nodes and connected edges
    pub fn delete_nodes(&mut self, ids: Vec<String>) -> Result<DeleteResult, CanvasError> {
        let id_set: HashSet<_> = ids.iter().collect();
        
        // Collect backups for undo
        let node_backups: Vec<(String, CanvasNode)> = ids.iter()
            .filter_map(|id| self.nodes.get(id).map(|n| (id.clone(), n.clone())))
            .collect();
        
        let edge_backups: Vec<(String, CanvasEdge)> = self.edges.iter()
            .filter(|(_, edge)| id_set.contains(&edge.source) || id_set.contains(&edge.target))
            .map(|(id, edge)| (id.clone(), edge.clone()))
            .collect();
        
        let deleted_edge_ids: Vec<String> = edge_backups.iter().map(|(id, _)| id.clone()).collect();
        
        // Record operation for undo
        self.undo_manager.push(CanvasOp::DeleteNodes {
            backups: node_backups,
            deleted_edges: edge_backups,
        });
        
        // Remove connected edges
        self.edges.retain(|_, edge| {
            !id_set.contains(&edge.source) && !id_set.contains(&edge.target)
        });
        
        // Remove nodes
        for id in &ids {
            if let Some(node) = self.nodes.remove(id) {
                self.spatial.remove(&node);
            }
            self.selection.remove(id);
        }
        
        self.graph.rebuild(&self.nodes, &self.edges);
        
        Ok(DeleteResult {
            deleted_nodes: ids,
            deleted_edges: deleted_edge_ids,
        })
    }
    
    /// Update node properties
    pub fn update_node(&mut self, id: &str, update: NodeUpdate) -> Result<(), CanvasError> {
        if let Some(node) = self.nodes.get_mut(id) {
            // Record old state for undo
            let old_state = NodeSnapshot::from_node(node);
            
            // Apply updates
            if let Some(label) = update.label {
                node.label = label;
            }
            if let Some(node_type) = update.node_type {
                node.node_type = node_type;
            }
            if let Some(entry_action) = update.entry_action {
                node.entry_action = entry_action;
            }
            if let Some(exit_action) = update.exit_action {
                node.exit_action = exit_action;
            }
            
            let new_state = NodeSnapshot::from_node(node);
            
            // Record operation for undo
            self.undo_manager.push(CanvasOp::UpdateNode {
                id: id.to_string(),
                old_state,
                new_state,
            });
            
            Ok(())
        } else {
            Err(CanvasError::NodeNotFound(id.to_string()))
        }
    }
    
    // === Edge Operations ===
    
    /// Connect two nodes with validation
    pub fn connect(
        &mut self,
        source: &str,
        target: &str,
        label: Option<String>,
    ) -> Result<CanvasEdge, CanvasError> {
        // Validate nodes exist
        if !self.nodes.contains_key(source) {
            return Err(CanvasError::NodeNotFound(source.to_string()));
        }
        if !self.nodes.contains_key(target) {
            return Err(CanvasError::NodeNotFound(target.to_string()));
        }
        
        // Prevent self-loops
        if source == target {
            return Err(CanvasError::SelfLoop);
        }
        
        // Check for duplicate
        for edge in self.edges.values() {
            if edge.source == source && edge.target == target {
                return Err(CanvasError::DuplicateEdge);
            }
        }
        
        // Check if would create cycle (optional, for DAG mode)
        if self.graph.would_create_cycle(source, target, &self.edges) {
            return Err(CanvasError::WouldCreateCycle);
        }
        
        let edge = CanvasEdge::new(
            format!("e{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()),
            source.to_string(),
            target.to_string(),
            label,
        );
        
        // Record operation for undo
        self.undo_manager.push(CanvasOp::AddEdge { edge: edge.clone() });
        
        self.edges.insert(edge.id.clone(), edge.clone());
        self.graph.add_edge(&edge);
        
        Ok(edge)
    }
    
    /// Delete edges
    pub fn delete_edges(&mut self, ids: Vec<String>) -> Result<(), CanvasError> {
        // Collect backups for undo
        let backups: Vec<(String, CanvasEdge)> = ids.iter()
            .filter_map(|id| self.edges.get(id).map(|e| (id.clone(), e.clone())))
            .collect();
        
        if !backups.is_empty() {
            self.undo_manager.push(CanvasOp::DeleteEdges { backups });
        }
        
        for id in ids {
            self.edges.remove(&id);
        }
        
        self.graph.rebuild(&self.nodes, &self.edges);
        
        Ok(())
    }
    
    // === Spatial Queries ===
    
    /// Find node at point
    pub fn query_at(&self, x: f64, y: f64) -> Option<&CanvasNode> {
        self.spatial.query_point(x, y)
            .and_then(|id| self.nodes.get(&id))
    }
    
    /// Find nodes in rectangle (marquee selection)
    pub fn query_rect(&self, x: f64, y: f64, width: f64, height: f64) -> Vec<String> {
        self.spatial.query_rect(x, y, width, height)
    }
    
    /// Find nearest edge to point
    pub fn nearest_edge(&self, x: f64, y: f64, threshold: f64) -> Option<&CanvasEdge> {
        let mut best: Option<(&CanvasEdge, f64)> = None;
        
        for edge in self.edges.values() {
            if let (Some(source), Some(target)) = (
                self.nodes.get(&edge.source),
                self.nodes.get(&edge.target)
            ) {
                let dist = self.path.distance_to_edge(
                    x, y,
                    source.x, source.y,
                    target.x, target.y,
                );
                
                if dist < threshold {
                    if best.is_none() || dist < best.unwrap().1 {
                        best = Some((edge, dist));
                    }
                }
            }
        }
        
        best.map(|(e, _)| e)
    }
    
    // === Selection ===
    
    pub fn select(&mut self, ids: Vec<String>) {
        self.selection = ids.into_iter().collect();
    }
    
    pub fn select_all(&mut self) {
        self.selection = self.nodes.keys().cloned().collect();
    }
    
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }
    
    pub fn get_selection(&self) -> Vec<String> {
        self.selection.iter().cloned().collect()
    }
    
    // === Path Calculations ===
    
    /// Get edge path as SVG path string
    pub fn get_edge_path(&self, edge_id: &str) -> Option<String> {
        let edge = self.edges.get(edge_id)?;
        let source = self.nodes.get(&edge.source)?;
        let target = self.nodes.get(&edge.target)?;
        
        Some(self.path.calculate(source, target))
    }
    
    /// Get all edge paths
    pub fn get_all_edge_paths(&self) -> HashMap<String, String> {
        self.edges.iter()
            .filter_map(|(id, edge)| {
                let source = self.nodes.get(&edge.source)?;
                let target = self.nodes.get(&edge.target)?;
                Some((id.clone(), self.path.calculate(source, target)))
            })
            .collect()
    }
    
    // === Validation ===
    
    /// Validate the graph
    pub fn validate(&self) -> ValidationResult {
        self.graph.validate(&self.nodes, &self.edges)
    }
    
    // === Layout ===
    
    /// Apply auto-layout to all nodes
    pub fn auto_layout(&mut self, algorithm: LayoutAlgorithm) -> Result<(), CanvasError> {
        let positions = self.layout.calculate(
            &self.nodes,
            &self.edges,
            algorithm,
        );
        
        // Build move operations with old positions for undo
        let move_ops: Vec<NodeMoveOp> = positions.iter()
            .filter_map(|(id, (new_x, new_y))| {
                self.nodes.get(id).map(|node| NodeMoveOp {
                    id: id.clone(),
                    old_x: node.x,
                    old_y: node.y,
                    new_x: *new_x,
                    new_y: *new_y,
                })
            })
            .collect();
        
        if !move_ops.is_empty() {
            self.undo_manager.push(CanvasOp::MoveNodes { moves: move_ops });
        }
        
        // Apply new positions
        for (id, (x, y)) in positions {
            if let Some(node) = self.nodes.get_mut(&id) {
                self.spatial.remove(node);
                node.x = x;
                node.y = y;
                self.spatial.insert(node);
            }
        }
        
        Ok(())
    }
    
    /// Align selected nodes
    pub fn align_nodes(&mut self, alignment: Alignment) -> Result<(), CanvasError> {
        if self.selection.is_empty() {
            return Ok(());
        }
        
        // Capture old positions for undo
        let old_positions: Vec<(String, f64, f64)> = self.selection.iter()
            .filter_map(|id| self.nodes.get(id).map(|n| (id.clone(), n.x, n.y)))
            .collect();
        
        self.layout.align(&mut self.nodes, &self.selection, alignment);
        
        // Build move operations
        let move_ops: Vec<NodeMoveOp> = old_positions.iter()
            .filter_map(|(id, old_x, old_y)| {
                self.nodes.get(id).map(|node| NodeMoveOp {
                    id: id.clone(),
                    old_x: *old_x,
                    old_y: *old_y,
                    new_x: node.x,
                    new_y: node.y,
                })
            })
            .filter(|m| m.old_x != m.new_x || m.old_y != m.new_y)
            .collect();
        
        if !move_ops.is_empty() {
            self.undo_manager.push(CanvasOp::MoveNodes { moves: move_ops });
        }
        
        // Rebuild spatial index for moved nodes
        self.spatial.clear();
        for node in self.nodes.values() {
            self.spatial.insert(node);
        }
        
        Ok(())
    }
    
    // === State Export ===
    
    /// Get current state
    pub fn get_state(&self) -> CanvasState {
        CanvasState {
            nodes: self.nodes.values().cloned().collect(),
            edges: self.edges.values().cloned().collect(),
            selection: self.selection.iter().cloned().collect(),
        }
    }
    
    /// Get node by ID
    pub fn get_node(&self, id: &str) -> Option<&CanvasNode> {
        self.nodes.get(id)
    }
    
    /// Get edge by ID
    pub fn get_edge(&self, id: &str) -> Option<&CanvasEdge> {
        self.edges.get(id)
    }
    
    // === Batch Operations ===
    
    /// Execute multiple operations as a single batch (one undo step)
    pub fn execute_batch(&mut self, batch: BatchOperation) -> Result<BatchResult, CanvasError> {
        if batch.is_empty() {
            return Ok(BatchResult::new());
        }
        
        let mut result = BatchResult::new();
        
        // Start batch for undo grouping
        self.undo_manager.begin_batch();
        
        // 1. Delete nodes first (to avoid conflicts)
        if !batch.delete_nodes.is_empty() {
            match self.delete_nodes(batch.delete_nodes) {
                Ok(del_result) => {
                    result.nodes_deleted = del_result.deleted_nodes.len();
                    result.edges_deleted = del_result.deleted_edges.len();
                }
                Err(e) => result.errors.push(format!("Delete nodes: {}", e)),
            }
        }
        
        // 2. Delete edges
        if !batch.delete_edges.is_empty() {
            let count = batch.delete_edges.len();
            if self.delete_edges(batch.delete_edges).is_ok() {
                result.edges_deleted += count;
            }
        }
        
        // 3. Add nodes
        for node in batch.add_nodes {
            if self.add_node(node).is_ok() {
                result.nodes_added += 1;
            }
        }
        
        // 4. Move nodes
        if !batch.move_nodes.is_empty() {
            let count = batch.move_nodes.len();
            if self.move_nodes(batch.move_nodes).is_ok() {
                result.nodes_moved = count;
            }
        }
        
        // 5. Update nodes
        for (id, update) in batch.update_nodes {
            if self.update_node(&id, update).is_ok() {
                result.nodes_updated += 1;
            }
        }
        
        // 6. Add edges
        for (source, target, label) in batch.add_edges {
            match self.connect(&source, &target, label) {
                Ok(edge) => result.edges_added.push(edge.id),
                Err(e) => result.errors.push(format!("Add edge: {}", e)),
            }
        }
        
        // End batch
        self.undo_manager.end_batch();
        
        // Invalidate path cache for any moved nodes
        self.path_cache.invalidate_all();
        
        Ok(result)
    }
    
    // === Path Caching ===
    
    /// Get edge path with caching
    pub fn get_edge_path_cached(&mut self, edge_id: &str) -> Option<String> {
        // Check cache first
        if let Some(cached) = self.path_cache.get(edge_id) {
            return Some(cached.to_string());
        }
        
        // Calculate and cache
        let edge = self.edges.get(edge_id)?;
        let source = self.nodes.get(&edge.source)?;
        let target = self.nodes.get(&edge.target)?;
        
        let path = self.path.calculate(source, target);
        
        self.path_cache.insert(
            edge_id.to_string(),
            path.clone(),
            &edge.source,
            (source.x, source.y),
            &edge.target,
            (target.x, target.y),
        );
        
        Some(path)
    }
    
    /// Get all edge paths with caching
    pub fn get_all_edge_paths_cached(&mut self) -> HashMap<String, String> {
        let edge_ids: Vec<String> = self.edges.keys().cloned().collect();
        let mut paths = HashMap::new();
        
        for edge_id in edge_ids {
            if let Some(path) = self.get_edge_path_cached(&edge_id) {
                paths.insert(edge_id, path);
            }
        }
        
        paths
    }
    
    /// Get path cache statistics
    pub fn path_cache_stats(&self) -> path_cache::CacheStats {
        self.path_cache.stats()
    }
}

impl Default for CanvasEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe canvas engine wrapper
pub type SharedCanvasEngine = Arc<RwLock<CanvasEngine>>;

/// Create a new shared canvas engine
pub fn create_engine() -> SharedCanvasEngine {
    Arc::new(RwLock::new(CanvasEngine::new()))
}
