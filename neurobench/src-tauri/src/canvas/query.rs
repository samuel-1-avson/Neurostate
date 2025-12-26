//! Query - Optimized query pipeline with frustum culling and LOD
//!
//! Provides efficient querying for large canvases with viewport-aware optimization.

use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use crate::canvas::types::CanvasNode;

/// Query result with visibility information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Visible node IDs (in viewport)
    pub visible: Vec<String>,
    /// Partially visible node IDs (edges in viewport)
    pub partial: Vec<String>,
    /// Hidden node IDs (outside viewport)
    pub hidden_count: usize,
    /// LOD level applied
    pub lod_level: LodLevel,
}

/// Level of Detail for rendering
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LodLevel {
    /// Full detail (all features)
    Full,
    /// Medium detail (no descriptions, simplified ports)
    Medium,
    /// Low detail (basic shapes, no text)
    Low,
    /// Minimal (dots/rectangles only)
    Minimal,
}

impl LodLevel {
    /// Get LOD level based on zoom
    pub fn from_zoom(zoom: f64) -> Self {
        match zoom {
            z if z >= 0.5 => LodLevel::Full,
            z if z >= 0.25 => LodLevel::Medium,
            z if z >= 0.1 => LodLevel::Low,
            _ => LodLevel::Minimal,
        }
    }
}

/// Frustum for viewport culling
#[derive(Debug, Clone)]
pub struct Frustum {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
    /// Expansion margin for partially visible nodes
    pub margin: f64,
}

impl Frustum {
    pub fn new(center_x: f64, center_y: f64, width: f64, height: f64, zoom: f64) -> Self {
        let half_w = (width / 2.0) / zoom;
        let half_h = (height / 2.0) / zoom;
        
        Self {
            min_x: center_x - half_w,
            min_y: center_y - half_h,
            max_x: center_x + half_w,
            max_y: center_y + half_h,
            margin: 50.0 / zoom, // Consistent margin in canvas space
        }
    }

    /// Check if node is fully visible
    pub fn contains_node(&self, node: &CanvasNode) -> bool {
        node.x >= self.min_x
            && node.y >= self.min_y
            && node.x + node.width <= self.max_x
            && node.y + node.height <= self.max_y
    }

    /// Check if node is partially visible
    pub fn intersects_node(&self, node: &CanvasNode) -> bool {
        !(node.x + node.width < self.min_x - self.margin
            || node.x > self.max_x + self.margin
            || node.y + node.height < self.min_y - self.margin
            || node.y > self.max_y + self.margin)
    }
}

/// Query optimizer for efficient canvas queries
#[derive(Debug, Default)]
pub struct QueryOptimizer {
    /// Last query frustum
    last_frustum: Option<Frustum>,
    /// Cached visible set
    cached_visible: HashSet<String>,
    /// Cache invalidation flag
    cache_dirty: bool,
}

impl QueryOptimizer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Query visible nodes with frustum culling
    pub fn query_visible(
        &mut self,
        nodes: &std::collections::HashMap<String, CanvasNode>,
        frustum: &Frustum,
        zoom: f64,
    ) -> QueryResult {
        let lod = LodLevel::from_zoom(zoom);
        let mut visible = Vec::new();
        let mut partial = Vec::new();
        let mut hidden_count = 0;

        for (id, node) in nodes {
            if frustum.contains_node(node) {
                visible.push(id.clone());
            } else if frustum.intersects_node(node) {
                partial.push(id.clone());
            } else {
                hidden_count += 1;
            }
        }

        // Update cache
        self.cached_visible = visible.iter().cloned().collect();
        self.cache_dirty = false;

        QueryResult {
            visible,
            partial,
            hidden_count,
            lod_level: lod,
        }
    }

    /// Quick check if node is in cached visible set
    pub fn is_cached_visible(&self, id: &str) -> bool {
        self.cached_visible.contains(id)
    }

    /// Invalidate cache (call when canvas changes)
    pub fn invalidate(&mut self) {
        self.cache_dirty = true;
    }

    /// Check if cache is valid
    pub fn is_cache_valid(&self) -> bool {
        !self.cache_dirty
    }
}

/// Streaming delta for incremental updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDelta {
    /// Added nodes
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub added_nodes: Vec<CanvasNode>,
    /// Removed node IDs
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed_nodes: Vec<String>,
    /// Modified nodes (partial updates)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modified_nodes: Vec<NodeDelta>,
    /// Added edges
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub added_edges: Vec<crate::canvas::types::CanvasEdge>,
    /// Removed edge IDs
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed_edges: Vec<String>,
    /// Selection changed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection: Option<Vec<String>>,
}

impl StateDelta {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_added_node(mut self, node: CanvasNode) -> Self {
        self.added_nodes.push(node);
        self
    }

    pub fn with_removed_node(mut self, id: impl Into<String>) -> Self {
        self.removed_nodes.push(id.into());
        self
    }

    pub fn with_modified_node(mut self, delta: NodeDelta) -> Self {
        self.modified_nodes.push(delta);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.added_nodes.is_empty()
            && self.removed_nodes.is_empty()
            && self.modified_nodes.is_empty()
            && self.added_edges.is_empty()
            && self.removed_edges.is_empty()
            && self.selection.is_none()
    }
}

impl Default for StateDelta {
    fn default() -> Self {
        Self {
            added_nodes: Vec::new(),
            removed_nodes: Vec::new(),
            modified_nodes: Vec::new(),
            added_edges: Vec::new(),
            removed_edges: Vec::new(),
            selection: None,
        }
    }
}

/// Partial node update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDelta {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl NodeDelta {
    pub fn position(id: impl Into<String>, x: f64, y: f64) -> Self {
        Self {
            id: id.into(),
            x: Some(x),
            y: Some(y),
            width: None,
            height: None,
            label: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::types::NodeType;

    #[test]
    fn test_frustum_culling() {
        let frustum = Frustum {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1000.0,
            max_y: 800.0,
            margin: 50.0,
        };

        let visible_node = CanvasNode::new(
            "n1".into(),
            "Test".into(),
            NodeType::State,
            100.0,
            100.0,
        );

        let hidden_node = CanvasNode::new(
            "n2".into(),
            "Test".into(),
            NodeType::State,
            2000.0,
            2000.0,
        );

        assert!(frustum.contains_node(&visible_node));
        assert!(!frustum.intersects_node(&hidden_node));
    }

    #[test]
    fn test_lod_levels() {
        assert_eq!(LodLevel::from_zoom(1.0), LodLevel::Full);
        assert_eq!(LodLevel::from_zoom(0.3), LodLevel::Medium);
        assert_eq!(LodLevel::from_zoom(0.15), LodLevel::Low);
        assert_eq!(LodLevel::from_zoom(0.05), LodLevel::Minimal);
    }
}
