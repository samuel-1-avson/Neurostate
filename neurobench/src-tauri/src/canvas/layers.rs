//! Layers - Canvas layer management for organizing content
//!
//! Provides layer system for grouping nodes/edges with visibility and lock controls.

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

/// A canvas layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    /// Unique layer ID  
    pub id: String,
    /// Display name
    pub name: String,
    /// Whether layer is visible
    pub visible: bool,
    /// Whether layer is locked (can't edit)
    pub locked: bool,
    /// Layer opacity (0.0 - 1.0)
    pub opacity: f64,
    /// Layer color (for UI indicator)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Z-index for rendering order
    pub z_index: i32,
    /// Node IDs in this layer
    #[serde(default)]
    pub nodes: HashSet<String>,
    /// Edge IDs in this layer
    #[serde(default)]
    pub edges: HashSet<String>,
}

impl Layer {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            visible: true,
            locked: false,
            opacity: 1.0,
            color: None,
            z_index: 0,
            nodes: HashSet::new(),
            edges: HashSet::new(),
        }
    }

    /// Toggle visibility
    pub fn toggle_visible(&mut self) {
        self.visible = !self.visible;
    }

    /// Toggle locked state
    pub fn toggle_locked(&mut self) {
        self.locked = !self.locked;
    }

    /// Add a node to this layer
    pub fn add_node(&mut self, id: impl Into<String>) {
        self.nodes.insert(id.into());
    }

    /// Remove a node from this layer
    pub fn remove_node(&mut self, id: &str) -> bool {
        self.nodes.remove(id)
    }

    /// Add an edge to this layer
    pub fn add_edge(&mut self, id: impl Into<String>) {
        self.edges.insert(id.into());
    }

    /// Remove an edge from this layer
    pub fn remove_edge(&mut self, id: &str) -> bool {
        self.edges.remove(id)
    }

    /// Check if layer contains node
    pub fn contains_node(&self, id: &str) -> bool {
        self.nodes.contains(id)
    }

    /// Check if layer contains edge
    pub fn contains_edge(&self, id: &str) -> bool {
        self.edges.contains(id)
    }

    /// Get item count
    pub fn item_count(&self) -> usize {
        self.nodes.len() + self.edges.len()
    }
}

/// Layer manager for the canvas
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayerManager {
    /// All layers indexed by ID
    layers: HashMap<String, Layer>,
    /// Layer order (bottom to top)
    order: Vec<String>,
    /// Active layer ID
    active_layer: Option<String>,
    /// Node to layer mapping
    node_layer: HashMap<String, String>,
    /// Edge to layer mapping
    edge_layer: HashMap<String, String>,
}

impl LayerManager {
    pub fn new() -> Self {
        let mut manager = Self::default();
        // Create default layer
        manager.create_layer("default", "Default");
        manager.active_layer = Some("default".to_string());
        manager
    }

    /// Create a new layer
    pub fn create_layer(&mut self, id: impl Into<String>, name: impl Into<String>) -> String {
        let id = id.into();
        let mut layer = Layer::new(id.clone(), name);
        layer.z_index = self.order.len() as i32;
        
        self.layers.insert(id.clone(), layer);
        self.order.push(id.clone());
        id
    }

    /// Delete a layer (moves contents to default)
    pub fn delete_layer(&mut self, id: &str) -> bool {
        if id == "default" {
            return false; // Can't delete default layer
        }

        if let Some(layer) = self.layers.remove(id) {
            // Move items to default layer
            if let Some(default) = self.layers.get_mut("default") {
                for node_id in layer.nodes {
                    default.add_node(node_id.clone());
                    self.node_layer.insert(node_id, "default".to_string());
                }
                for edge_id in layer.edges {
                    default.add_edge(edge_id.clone());
                    self.edge_layer.insert(edge_id, "default".to_string());
                }
            }

            self.order.retain(|x| x != id);
            
            // Update active if needed
            if self.active_layer.as_deref() == Some(id) {
                self.active_layer = Some("default".to_string());
            }

            true
        } else {
            false
        }
    }

    /// Get layer by ID
    pub fn get_layer(&self, id: &str) -> Option<&Layer> {
        self.layers.get(id)
    }

    /// Get mutable layer by ID
    pub fn get_layer_mut(&mut self, id: &str) -> Option<&mut Layer> {
        self.layers.get_mut(id)
    }

    /// Set active layer
    pub fn set_active(&mut self, id: impl Into<String>) {
        let id = id.into();
        if self.layers.contains_key(&id) {
            self.active_layer = Some(id);
        }
    }

    /// Get active layer
    pub fn active(&self) -> Option<&Layer> {
        self.active_layer.as_ref().and_then(|id| self.layers.get(id))
    }

    /// Get active layer ID
    pub fn active_id(&self) -> Option<&str> {
        self.active_layer.as_deref()
    }

    /// Add node to active layer
    pub fn add_node(&mut self, node_id: impl Into<String>) {
        let node_id = node_id.into();
        let layer_id = self.active_layer.clone().unwrap_or_else(|| "default".to_string());
        
        if let Some(layer) = self.layers.get_mut(&layer_id) {
            layer.add_node(node_id.clone());
            self.node_layer.insert(node_id, layer_id);
        }
    }

    /// Add node to specific layer
    pub fn add_node_to_layer(&mut self, node_id: impl Into<String>, layer_id: &str) {
        let node_id = node_id.into();
        
        // Remove from current layer if any
        if let Some(old_layer_id) = self.node_layer.remove(&node_id) {
            if let Some(old_layer) = self.layers.get_mut(&old_layer_id) {
                old_layer.remove_node(&node_id);
            }
        }

        // Add to new layer
        if let Some(layer) = self.layers.get_mut(layer_id) {
            layer.add_node(node_id.clone());
            self.node_layer.insert(node_id, layer_id.to_string());
        }
    }

    /// Remove node from all layers
    pub fn remove_node(&mut self, node_id: &str) {
        if let Some(layer_id) = self.node_layer.remove(node_id) {
            if let Some(layer) = self.layers.get_mut(&layer_id) {
                layer.remove_node(node_id);
            }
        }
    }

    /// Get layer for a node
    pub fn get_node_layer(&self, node_id: &str) -> Option<&Layer> {
        self.node_layer.get(node_id).and_then(|id| self.layers.get(id))
    }

    /// Check if node is visible (in visible layer)
    pub fn is_node_visible(&self, node_id: &str) -> bool {
        self.get_node_layer(node_id).map(|l| l.visible).unwrap_or(true)
    }

    /// Check if node is locked (in locked layer)
    pub fn is_node_locked(&self, node_id: &str) -> bool {
        self.get_node_layer(node_id).map(|l| l.locked).unwrap_or(false)
    }

    /// Get all layers in order
    pub fn layers_ordered(&self) -> Vec<&Layer> {
        self.order.iter()
            .filter_map(|id| self.layers.get(id))
            .collect()
    }

    /// Move layer up in order
    pub fn move_layer_up(&mut self, id: &str) -> bool {
        if let Some(pos) = self.order.iter().position(|x| x == id) {
            if pos < self.order.len() - 1 {
                self.order.swap(pos, pos + 1);
                self.update_z_indices();
                return true;
            }
        }
        false
    }

    /// Move layer down in order
    pub fn move_layer_down(&mut self, id: &str) -> bool {
        if let Some(pos) = self.order.iter().position(|x| x == id) {
            if pos > 0 {
                self.order.swap(pos, pos - 1);
                self.update_z_indices();
                return true;
            }
        }
        false
    }

    fn update_z_indices(&mut self) {
        for (i, id) in self.order.iter().enumerate() {
            if let Some(layer) = self.layers.get_mut(id) {
                layer.z_index = i as i32;
            }
        }
    }

    /// Get visible node IDs
    pub fn visible_nodes(&self) -> HashSet<String> {
        self.layers.values()
            .filter(|l| l.visible)
            .flat_map(|l| l.nodes.iter().cloned())
            .collect()
    }

    /// Get visible edge IDs
    pub fn visible_edges(&self) -> HashSet<String> {
        self.layers.values()
            .filter(|l| l.visible)
            .flat_map(|l| l.edges.iter().cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_manager() {
        let mut manager = LayerManager::new();
        
        assert!(manager.get_layer("default").is_some());
        
        manager.create_layer("layer1", "Layer 1");
        manager.set_active("layer1");
        
        manager.add_node("n1");
        assert!(manager.get_layer("layer1").unwrap().contains_node("n1"));
    }

    #[test]
    fn test_layer_visibility() {
        let mut manager = LayerManager::new();
        manager.add_node("n1");
        
        assert!(manager.is_node_visible("n1"));
        
        if let Some(layer) = manager.get_layer_mut("default") {
            layer.visible = false;
        }
        
        assert!(!manager.is_node_visible("n1"));
    }
}
