//! Layout Engine - Automatic graph layout algorithms
//!
//! Provides force-directed, hierarchical, and grid layouts.

use super::types::*;
use std::collections::{HashMap, HashSet};

/// Layout engine for automatic node arrangement
pub struct LayoutEngine {
    /// Grid cell size for snapping
    grid_size: f64,
    /// Node spacing for layouts
    spacing: f64,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            grid_size: 24.0,
            spacing: 80.0,
        }
    }
    
    /// Calculate new positions for all nodes
    pub fn calculate(
        &self,
        nodes: &HashMap<String, CanvasNode>,
        edges: &HashMap<String, CanvasEdge>,
        algorithm: LayoutAlgorithm,
    ) -> HashMap<String, (f64, f64)> {
        match algorithm {
            LayoutAlgorithm::ForceDirected => self.force_directed(nodes, edges),
            LayoutAlgorithm::Hierarchical => self.hierarchical(nodes, edges),
            LayoutAlgorithm::Tree => self.tree_layout(nodes, edges),
            LayoutAlgorithm::Grid => self.grid_layout(nodes),
        }
    }
    
    /// Force-directed spring layout
    fn force_directed(
        &self,
        nodes: &HashMap<String, CanvasNode>,
        edges: &HashMap<String, CanvasEdge>,
    ) -> HashMap<String, (f64, f64)> {
        let mut positions: HashMap<String, (f64, f64)> = nodes
            .iter()
            .map(|(id, n)| (id.clone(), (n.x, n.y)))
            .collect();
        
        let ids: Vec<_> = positions.keys().cloned().collect();
        let iterations = 100;
        let cooling = 0.95;
        let mut temp = 100.0;
        
        let ideal_dist = 200.0;
        let k_spring = 0.1;
        let k_repel = 5000.0;
        
        // Build adjacency
        let mut neighbors: HashMap<String, HashSet<String>> = HashMap::new();
        for edge in edges.values() {
            neighbors.entry(edge.source.clone()).or_default().insert(edge.target.clone());
            neighbors.entry(edge.target.clone()).or_default().insert(edge.source.clone());
        }
        
        for _ in 0..iterations {
            let mut forces: HashMap<String, (f64, f64)> = ids
                .iter()
                .map(|id| (id.clone(), (0.0, 0.0)))
                .collect();
            
            // Repulsion between all nodes
            for i in 0..ids.len() {
                for j in (i + 1)..ids.len() {
                    let id1 = &ids[i];
                    let id2 = &ids[j];
                    
                    let (x1, y1) = positions[id1];
                    let (x2, y2) = positions[id2];
                    
                    let dx = x2 - x1;
                    let dy = y2 - y1;
                    let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                    
                    let force = k_repel / (dist * dist);
                    let fx = force * dx / dist;
                    let fy = force * dy / dist;
                    
                    forces.get_mut(id1).unwrap().0 -= fx;
                    forces.get_mut(id1).unwrap().1 -= fy;
                    forces.get_mut(id2).unwrap().0 += fx;
                    forces.get_mut(id2).unwrap().1 += fy;
                }
            }
            
            // Attraction along edges
            for edge in edges.values() {
                if let (Some(&(x1, y1)), Some(&(x2, y2))) = 
                    (positions.get(&edge.source), positions.get(&edge.target)) 
                {
                    let dx = x2 - x1;
                    let dy = y2 - y1;
                    let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                    
                    let force = k_spring * (dist - ideal_dist);
                    let fx = force * dx / dist;
                    let fy = force * dy / dist;
                    
                    forces.get_mut(&edge.source).unwrap().0 += fx;
                    forces.get_mut(&edge.source).unwrap().1 += fy;
                    forces.get_mut(&edge.target).unwrap().0 -= fx;
                    forces.get_mut(&edge.target).unwrap().1 -= fy;
                }
            }
            
            // Apply forces with temperature
            for id in &ids {
                let (fx, fy) = forces[id];
                let (x, y) = positions.get_mut(id).unwrap();
                
                *x += fx.clamp(-temp, temp);
                *y += fy.clamp(-temp, temp);
            }
            
            temp *= cooling;
        }
        
        // Snap to grid
        self.snap_positions(&mut positions);
        
        positions
    }
    
    /// Hierarchical top-to-bottom layout
    fn hierarchical(
        &self,
        nodes: &HashMap<String, CanvasNode>,
        edges: &HashMap<String, CanvasEdge>,
    ) -> HashMap<String, (f64, f64)> {
        let mut positions = HashMap::new();
        
        // Build adjacency
        let mut outgoing: HashMap<String, Vec<String>> = HashMap::new();
        let mut incoming: HashMap<String, usize> = HashMap::new();
        
        for id in nodes.keys() {
            outgoing.insert(id.clone(), Vec::new());
            incoming.insert(id.clone(), 0);
        }
        
        for edge in edges.values() {
            outgoing.get_mut(&edge.source).unwrap().push(edge.target.clone());
            *incoming.get_mut(&edge.target).unwrap() += 1;
        }
        
        // Find roots (no incoming)
        let mut roots: Vec<String> = incoming
            .iter()
            .filter(|(_, &count)| count == 0)
            .map(|(id, _)| id.clone())
            .collect();
        
        if roots.is_empty() {
            // If no roots, pick first node
            if let Some(id) = nodes.keys().next() {
                roots.push(id.clone());
            }
        }
        
        // Assign levels via BFS
        let mut levels: HashMap<String, usize> = HashMap::new();
        let mut queue = std::collections::VecDeque::new();
        
        for root in &roots {
            levels.insert(root.clone(), 0);
            queue.push_back(root.clone());
        }
        
        while let Some(node) = queue.pop_front() {
            let level = levels[&node];
            if let Some(children) = outgoing.get(&node) {
                for child in children {
                    if !levels.contains_key(child) {
                        levels.insert(child.clone(), level + 1);
                        queue.push_back(child.clone());
                    }
                }
            }
        }
        
        // Group by level
        let mut level_nodes: HashMap<usize, Vec<String>> = HashMap::new();
        for (id, level) in &levels {
            level_nodes.entry(*level).or_default().push(id.clone());
        }
        
        // Position nodes
        let y_spacing = 150.0;
        let x_spacing = 200.0;
        
        for (level, node_ids) in &level_nodes {
            let y = 100.0 + (*level as f64) * y_spacing;
            let total_width = (node_ids.len() as f64 - 1.0) * x_spacing;
            let start_x = 400.0 - total_width / 2.0;
            
            for (i, id) in node_ids.iter().enumerate() {
                let x = start_x + (i as f64) * x_spacing;
                positions.insert(id.clone(), (self.snap(x), self.snap(y)));
            }
        }
        
        // Handle unleveled nodes
        let mut y_offset = 100.0 + (level_nodes.len() as f64) * y_spacing;
        for id in nodes.keys() {
            if !positions.contains_key(id) {
                positions.insert(id.clone(), (self.snap(100.0), self.snap(y_offset)));
                y_offset += 100.0;
            }
        }
        
        positions
    }
    
    /// Tree layout
    fn tree_layout(
        &self,
        nodes: &HashMap<String, CanvasNode>,
        edges: &HashMap<String, CanvasEdge>,
    ) -> HashMap<String, (f64, f64)> {
        // Similar to hierarchical but with tree-specific spacing
        self.hierarchical(nodes, edges)
    }
    
    /// Simple grid layout
    fn grid_layout(&self, nodes: &HashMap<String, CanvasNode>) -> HashMap<String, (f64, f64)> {
        let mut positions = HashMap::new();
        
        let cols = (nodes.len() as f64).sqrt().ceil() as usize;
        let x_spacing = 200.0;
        let y_spacing = 120.0;
        
        for (i, id) in nodes.keys().enumerate() {
            let col = i % cols;
            let row = i / cols;
            
            let x = 100.0 + (col as f64) * x_spacing;
            let y = 100.0 + (row as f64) * y_spacing;
            
            positions.insert(id.clone(), (self.snap(x), self.snap(y)));
        }
        
        positions
    }
    
    /// Align selected nodes
    pub fn align(
        &self,
        nodes: &mut HashMap<String, CanvasNode>,
        selection: &HashSet<String>,
        alignment: Alignment,
    ) {
        if selection.is_empty() {
            return;
        }
        
        let selected_nodes: Vec<_> = selection
            .iter()
            .filter_map(|id| nodes.get(id))
            .collect();
        
        match alignment {
            Alignment::Left => {
                let min_x = selected_nodes.iter().map(|n| n.x).fold(f64::MAX, f64::min);
                for id in selection {
                    if let Some(node) = nodes.get_mut(id) {
                        node.x = min_x;
                    }
                }
            }
            Alignment::Right => {
                let max_x = selected_nodes.iter().map(|n| n.x + n.width).fold(f64::MIN, f64::max);
                for id in selection {
                    if let Some(node) = nodes.get_mut(id) {
                        node.x = max_x - node.width;
                    }
                }
            }
            Alignment::Top => {
                let min_y = selected_nodes.iter().map(|n| n.y).fold(f64::MAX, f64::min);
                for id in selection {
                    if let Some(node) = nodes.get_mut(id) {
                        node.y = min_y;
                    }
                }
            }
            Alignment::Bottom => {
                let max_y = selected_nodes.iter().map(|n| n.y + n.height).fold(f64::MIN, f64::max);
                for id in selection {
                    if let Some(node) = nodes.get_mut(id) {
                        node.y = max_y - node.height;
                    }
                }
            }
            Alignment::CenterHorizontal => {
                let center_x = selected_nodes.iter().map(|n| n.x + n.width / 2.0).sum::<f64>() 
                    / selected_nodes.len() as f64;
                for id in selection {
                    if let Some(node) = nodes.get_mut(id) {
                        node.x = center_x - node.width / 2.0;
                    }
                }
            }
            Alignment::CenterVertical => {
                let center_y = selected_nodes.iter().map(|n| n.y + n.height / 2.0).sum::<f64>() 
                    / selected_nodes.len() as f64;
                for id in selection {
                    if let Some(node) = nodes.get_mut(id) {
                        node.y = center_y - node.height / 2.0;
                    }
                }
            }
            Alignment::DistributeHorizontal => {
                let mut sorted: Vec<_> = selection.iter()
                    .filter_map(|id| nodes.get(id).map(|n| (id.clone(), n.x)))
                    .collect();
                sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                
                if sorted.len() >= 2 {
                    let first_x = sorted[0].1;
                    let last_x = sorted.last().unwrap().1;
                    let spacing = (last_x - first_x) / (sorted.len() - 1) as f64;
                    
                    for (i, (id, _)) in sorted.iter().enumerate() {
                        if let Some(node) = nodes.get_mut(id) {
                            node.x = first_x + (i as f64) * spacing;
                        }
                    }
                }
            }
            Alignment::DistributeVertical => {
                let mut sorted: Vec<_> = selection.iter()
                    .filter_map(|id| nodes.get(id).map(|n| (id.clone(), n.y)))
                    .collect();
                sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                
                if sorted.len() >= 2 {
                    let first_y = sorted[0].1;
                    let last_y = sorted.last().unwrap().1;
                    let spacing = (last_y - first_y) / (sorted.len() - 1) as f64;
                    
                    for (i, (id, _)) in sorted.iter().enumerate() {
                        if let Some(node) = nodes.get_mut(id) {
                            node.y = first_y + (i as f64) * spacing;
                        }
                    }
                }
            }
        }
    }
    
    /// Snap value to grid
    fn snap(&self, value: f64) -> f64 {
        (value / self.grid_size).round() * self.grid_size
    }
    
    /// Snap all positions to grid
    fn snap_positions(&self, positions: &mut HashMap<String, (f64, f64)>) {
        for (_, (x, y)) in positions.iter_mut() {
            *x = self.snap(*x);
            *y = self.snap(*y);
        }
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// LAYOUT CACHE
// =============================================================================

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Cache key for layout results
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct LayoutCacheKey {
    algorithm: String,
    node_ids_hash: u64,
    edge_ids_hash: u64,
}

impl LayoutCacheKey {
    fn new(
        algorithm: LayoutAlgorithm,
        nodes: &HashMap<String, CanvasNode>,
        edges: &HashMap<String, CanvasEdge>,
    ) -> Self {
        // Hash node IDs and positions
        let mut node_hasher = DefaultHasher::new();
        let mut node_ids: Vec<_> = nodes.keys().collect();
        node_ids.sort();
        for id in node_ids {
            id.hash(&mut node_hasher);
        }
        
        // Hash edge IDs
        let mut edge_hasher = DefaultHasher::new();
        let mut edge_ids: Vec<_> = edges.keys().collect();
        edge_ids.sort();
        for id in edge_ids {
            id.hash(&mut edge_hasher);
            if let Some(edge) = edges.get(id) {
                edge.source.hash(&mut edge_hasher);
                edge.target.hash(&mut edge_hasher);
            }
        }
        
        Self {
            algorithm: format!("{:?}", algorithm),
            node_ids_hash: node_hasher.finish(),
            edge_ids_hash: edge_hasher.finish(),
        }
    }
}

/// Cached layout entry with timestamp
#[derive(Debug, Clone)]
struct CachedLayoutEntry {
    positions: HashMap<String, (f64, f64)>,
    created_at: std::time::Instant,
}

/// Cache for computed layout results
/// 
/// Avoids expensive recalculation of force-directed layouts for unchanged graphs.
pub struct LayoutCache {
    cache: HashMap<LayoutCacheKey, CachedLayoutEntry>,
    max_entries: usize,
    ttl_secs: u64,
    hits: u64,
    misses: u64,
}

impl LayoutCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_entries: 10,
            ttl_secs: 300, // 5 minutes
            hits: 0,
            misses: 0,
        }
    }
    
    /// Get cached layout if available and fresh
    pub fn get(
        &mut self,
        algorithm: LayoutAlgorithm,
        nodes: &HashMap<String, CanvasNode>,
        edges: &HashMap<String, CanvasEdge>,
    ) -> Option<HashMap<String, (f64, f64)>> {
        let key = LayoutCacheKey::new(algorithm, nodes, edges);
        
        if let Some(entry) = self.cache.get(&key) {
            // Check TTL
            if entry.created_at.elapsed().as_secs() < self.ttl_secs {
                self.hits += 1;
                return Some(entry.positions.clone());
            } else {
                // Expired - remove it
                self.cache.remove(&key);
            }
        }
        
        self.misses += 1;
        None
    }
    
    /// Store computed layout in cache
    pub fn insert(
        &mut self,
        algorithm: LayoutAlgorithm,
        nodes: &HashMap<String, CanvasNode>,
        edges: &HashMap<String, CanvasEdge>,
        positions: HashMap<String, (f64, f64)>,
    ) {
        let key = LayoutCacheKey::new(algorithm, nodes, edges);
        
        // Evict oldest if at capacity
        if self.cache.len() >= self.max_entries {
            self.evict_oldest();
        }
        
        self.cache.insert(key, CachedLayoutEntry {
            positions,
            created_at: std::time::Instant::now(),
        });
    }
    
    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> LayoutCacheStats {
        let total = self.hits + self.misses;
        LayoutCacheStats {
            entries: self.cache.len(),
            hits: self.hits,
            misses: self.misses,
            hit_rate: if total > 0 { self.hits as f64 / total as f64 } else { 0.0 },
        }
    }
    
    /// Evict oldest entry
    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self.cache.iter()
            .min_by_key(|(_, v)| v.created_at)
            .map(|(k, _)| k.clone())
        {
            self.cache.remove(&oldest_key);
        }
    }
}

impl Default for LayoutCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Layout cache statistics
#[derive(Debug, Clone)]
pub struct LayoutCacheStats {
    pub entries: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}
