//! Path Cache - Lazy edge path calculation with caching
//!
//! Caches calculated SVG paths for edges to avoid recalculating on every render.
//! Invalidates cache entries when nodes move.

use std::collections::{HashMap, HashSet};

/// Cached edge path with invalidation tracking
pub struct PathCache {
    /// Cached paths indexed by edge ID
    cache: HashMap<String, CachedPath>,
    /// Dirty edges that need recalculation
    dirty: HashSet<String>,
    /// Node positions at time of caching (for invalidation)
    node_positions: HashMap<String, (f64, f64)>,
    /// Cache hit counter
    hits: u64,
    /// Cache miss counter
    misses: u64,
}

#[derive(Debug, Clone)]
struct CachedPath {
    /// The SVG path string
    path: String,
    /// Source node position when cached
    source_pos: (f64, f64),
    /// Target node position when cached
    target_pos: (f64, f64),
}

impl PathCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            dirty: HashSet::new(),
            node_positions: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    /// Clear all cached paths
    pub fn clear(&mut self) {
        self.cache.clear();
        self.dirty.clear();
        self.node_positions.clear();
    }

    /// Get cached path for an edge, or None if not cached/invalid
    pub fn get(&mut self, edge_id: &str) -> Option<&str> {
        if self.dirty.contains(edge_id) {
            self.misses += 1;
            return None;
        }
        match self.cache.get(edge_id) {
            Some(cached) => {
                self.hits += 1;
                Some(cached.path.as_str())
            }
            None => {
                self.misses += 1;
                None
            }
        }
    }

    /// Store a computed path in the cache
    pub fn insert(
        &mut self,
        edge_id: String,
        path: String,
        source_id: &str,
        source_pos: (f64, f64),
        target_id: &str,
        target_pos: (f64, f64),
    ) {
        self.cache.insert(edge_id.clone(), CachedPath {
            path,
            source_pos,
            target_pos,
        });
        self.dirty.remove(&edge_id);
        
        // Track node positions
        self.node_positions.insert(source_id.to_string(), source_pos);
        self.node_positions.insert(target_id.to_string(), target_pos);
    }

    /// Mark edges connected to a node as dirty (node moved)
    pub fn invalidate_node(&mut self, node_id: &str, new_pos: (f64, f64)) {
        // Update position
        let old_pos = self.node_positions.insert(node_id.to_string(), new_pos);
        
        // Only invalidate if position actually changed
        if old_pos != Some(new_pos) {
            // Mark all edges involving this node as dirty
            // This requires knowing which edges connect to this node
            // For now, we track this by checking source/target positions
            for (edge_id, cached) in &self.cache {
                let source_matches = self.positions_match(&cached.source_pos, node_id);
                let target_matches = self.positions_match(&cached.target_pos, node_id);
                if source_matches || target_matches {
                    self.dirty.insert(edge_id.clone());
                }
            }
        }
    }

    /// Mark specific edges as dirty
    pub fn invalidate_edges(&mut self, edge_ids: &[String]) {
        for id in edge_ids {
            self.dirty.insert(id.clone());
        }
    }

    /// Mark all edges as dirty (full invalidation)
    pub fn invalidate_all(&mut self) {
        self.dirty = self.cache.keys().cloned().collect();
    }

    /// Remove an edge from the cache
    pub fn remove(&mut self, edge_id: &str) {
        self.cache.remove(edge_id);
        self.dirty.remove(edge_id);
    }

    /// Check if cache has valid entry for edge
    pub fn is_valid(&self, edge_id: &str) -> bool {
        self.cache.contains_key(edge_id) && !self.dirty.contains(edge_id)
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total = self.hits + self.misses;
        let hit_rate = if total > 0 { self.hits as f64 / total as f64 } else { 0.0 };
        CacheStats {
            total_entries: self.cache.len(),
            dirty_entries: self.dirty.len(),
            valid_entries: self.cache.len().saturating_sub(self.dirty.len()),
            hits: self.hits,
            misses: self.misses,
            hit_rate,
        }
    }
    
    /// Reset hit/miss counters
    pub fn reset_stats(&mut self) {
        self.hits = 0;
        self.misses = 0;
    }

    fn positions_match(&self, cached_pos: &(f64, f64), node_id: &str) -> bool {
        if let Some(current_pos) = self.node_positions.get(node_id) {
            (cached_pos.0 - current_pos.0).abs() < 0.1 && 
            (cached_pos.1 - current_pos.1).abs() < 0.1
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub dirty_entries: usize,
    pub valid_entries: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

impl Default for PathCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = PathCache::new();
        
        cache.insert(
            "e1".to_string(),
            "M0,0 C50,50 50,150 100,200".to_string(),
            "n1",
            (0.0, 0.0),
            "n2",
            (100.0, 200.0),
        );
        
        assert!(cache.is_valid("e1"));
        assert_eq!(cache.get("e1"), Some("M0,0 C50,50 50,150 100,200"));
    }

    #[test]
    fn test_invalidation() {
        let mut cache = PathCache::new();
        
        cache.insert(
            "e1".to_string(),
            "M0,0 L100,100".to_string(),
            "n1",
            (0.0, 0.0),
            "n2",
            (100.0, 100.0),
        );
        
        assert!(cache.is_valid("e1"));
        
        // Invalidate
        cache.invalidate_edges(&["e1".to_string()]);
        
        assert!(!cache.is_valid("e1"));
        assert_eq!(cache.get("e1"), None);
    }
}
