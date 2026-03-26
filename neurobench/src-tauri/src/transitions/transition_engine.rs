//! Transition Engine
//!
//! Core engine for managing transitions between nodes

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use super::transition_types::{Transition, TransitionUpdate, TransitionInfo, ConnectionPoint};
use super::validation::{ConnectionValidator, ValidationResult, ValidationError};

/// Error types for transition operations
#[derive(Debug, Clone, Serialize)]
pub enum TransitionError {
    NotFound(String),
    AlreadyExists(String),
    InvalidConnection(String),
    ValidationFailed(Vec<ValidationError>),
    CycleDetected(Vec<String>),
    SourceNodeNotFound(String),
    TargetNodeNotFound(String),
    IncompatiblePorts { source: String, target: String },
}

impl std::fmt::Display for TransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "Transition not found: {}", id),
            Self::AlreadyExists(id) => write!(f, "Transition already exists: {}", id),
            Self::InvalidConnection(msg) => write!(f, "Invalid connection: {}", msg),
            Self::ValidationFailed(errors) => write!(f, "Validation failed: {:?}", errors),
            Self::CycleDetected(path) => write!(f, "Cycle detected: {:?}", path),
            Self::SourceNodeNotFound(id) => write!(f, "Source node not found: {}", id),
            Self::TargetNodeNotFound(id) => write!(f, "Target node not found: {}", id),
            Self::IncompatiblePorts { source, target } => {
                write!(f, "Incompatible ports: {} -> {}", source, target)
            }
        }
    }
}

impl std::error::Error for TransitionError {}

/// Transition Engine - manages all transitions in a design
#[derive(Debug, Clone, Default)]
pub struct TransitionEngine {
    /// All transitions indexed by ID
    transitions: HashMap<String, Transition>,
    
    /// Transitions indexed by source node
    by_source: HashMap<String, HashSet<String>>,
    
    /// Transitions indexed by target node
    by_target: HashMap<String, HashSet<String>>,
    
    /// Known node IDs (for validation)
    known_nodes: HashSet<String>,
    
    /// Connection validator
    validator: ConnectionValidator,
    
    /// Auto-increment ID counter
    next_id: u64,
}

impl TransitionEngine {
    /// Create a new empty transition engine
    pub fn new() -> Self {
        Self {
            transitions: HashMap::new(),
            by_source: HashMap::new(),
            by_target: HashMap::new(),
            known_nodes: HashSet::new(),
            validator: ConnectionValidator::new(),
            next_id: 1,
        }
    }

    /// Generate a unique transition ID
    fn generate_id(&mut self) -> String {
        let id = format!("t_{}", self.next_id);
        self.next_id += 1;
        id
    }

    /// Register a node as known (for validation)
    pub fn register_node(&mut self, node_id: impl Into<String>) {
        self.known_nodes.insert(node_id.into());
    }

    /// Unregister a node and remove all its transitions
    pub fn unregister_node(&mut self, node_id: &str) -> Vec<String> {
        self.known_nodes.remove(node_id);
        
        // Collect transitions to remove
        let mut to_remove = Vec::new();
        
        if let Some(outgoing) = self.by_source.get(node_id) {
            to_remove.extend(outgoing.iter().cloned());
        }
        if let Some(incoming) = self.by_target.get(node_id) {
            to_remove.extend(incoming.iter().cloned());
        }
        
        // Remove transitions
        for tid in &to_remove {
            let _ = self.delete(tid);
        }
        
        to_remove
    }

    /// Create a new transition
    pub fn create(
        &mut self,
        source_node: impl Into<String>,
        target_node: impl Into<String>,
    ) -> Result<Transition, TransitionError> {
        let source = source_node.into();
        let target = target_node.into();
        
        // Validate nodes exist
        if !self.known_nodes.contains(&source) {
            return Err(TransitionError::SourceNodeNotFound(source));
        }
        if !self.known_nodes.contains(&target) {
            return Err(TransitionError::TargetNodeNotFound(target));
        }
        
        // Create transition
        let id = self.generate_id();
        let transition = Transition::new(&id, &source, &target);
        
        // Add to indices
        self.by_source.entry(source.clone())
            .or_default()
            .insert(id.clone());
        self.by_target.entry(target.clone())
            .or_default()
            .insert(id.clone());
        
        self.transitions.insert(id.clone(), transition.clone());
        
        Ok(transition)
    }

    /// Create a transition with ports specified
    pub fn create_with_ports(
        &mut self,
        source_node: impl Into<String>,
        source_port: impl Into<String>,
        target_node: impl Into<String>,
        target_port: impl Into<String>,
    ) -> Result<Transition, TransitionError> {
        let source = source_node.into();
        let target = target_node.into();
        let src_port = source_port.into();
        let tgt_port = target_port.into();
        
        // Validate nodes exist
        if !self.known_nodes.contains(&source) {
            return Err(TransitionError::SourceNodeNotFound(source));
        }
        if !self.known_nodes.contains(&target) {
            return Err(TransitionError::TargetNodeNotFound(target));
        }
        
        // Create transition with ports
        let id = self.generate_id();
        let transition = Transition::with_ports(&id, &source, &src_port, &target, &tgt_port);
        
        // Add to indices
        self.by_source.entry(source.clone())
            .or_default()
            .insert(id.clone());
        self.by_target.entry(target.clone())
            .or_default()
            .insert(id.clone());
        
        self.transitions.insert(id.clone(), transition.clone());
        
        Ok(transition)
    }

    /// Get a transition by ID
    pub fn get(&self, id: &str) -> Option<&Transition> {
        self.transitions.get(id)
    }

    /// Get all transitions
    pub fn get_all(&self) -> Vec<&Transition> {
        self.transitions.values().collect()
    }

    /// Get all transitions as owned copies
    pub fn get_all_owned(&self) -> Vec<Transition> {
        self.transitions.values().cloned().collect()
    }

    /// Get transitions from a source node
    pub fn get_by_source(&self, node_id: &str) -> Vec<&Transition> {
        self.by_source.get(node_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.transitions.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get transitions to a target node
    pub fn get_by_target(&self, node_id: &str) -> Vec<&Transition> {
        self.by_target.get(node_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.transitions.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all transitions involving a node (as source or target)
    pub fn get_by_node(&self, node_id: &str) -> Vec<&Transition> {
        let mut result = Vec::new();
        result.extend(self.get_by_source(node_id));
        result.extend(self.get_by_target(node_id));
        result
    }

    /// Update a transition
    pub fn update(&mut self, id: &str, update: TransitionUpdate) -> Result<Transition, TransitionError> {
        let transition = self.transitions.get_mut(id)
            .ok_or_else(|| TransitionError::NotFound(id.to_string()))?;
        
        // Apply updates
        if let Some(label) = update.label {
            transition.label = Some(label);
        }
        if let Some(event) = update.event {
            transition.event = Some(event);
        }
        if let Some(guard) = update.guard {
            transition.guard = Some(guard);
        }
        if let Some(action) = update.action {
            transition.action = Some(action);
        }
        if let Some(priority) = update.priority {
            transition.priority = priority;
        }
        if let Some(is_default) = update.is_default {
            transition.is_default = is_default;
        }
        if let Some(enabled) = update.enabled {
            transition.enabled = enabled;
        }
        if let Some(waypoints) = update.waypoints {
            transition.waypoints = waypoints;
        }
        
        // Update timestamp
        transition.updated_at = Some(chrono::Utc::now().to_rfc3339());
        
        Ok(transition.clone())
    }

    /// Delete a transition
    pub fn delete(&mut self, id: &str) -> Result<Transition, TransitionError> {
        let transition = self.transitions.remove(id)
            .ok_or_else(|| TransitionError::NotFound(id.to_string()))?;
        
        // Remove from indices
        if let Some(set) = self.by_source.get_mut(&transition.source.node_id) {
            set.remove(id);
        }
        if let Some(set) = self.by_target.get_mut(&transition.target.node_id) {
            set.remove(id);
        }
        
        Ok(transition)
    }

    /// Delete all transitions
    pub fn clear(&mut self) {
        self.transitions.clear();
        self.by_source.clear();
        self.by_target.clear();
    }

    /// Get transition count
    pub fn count(&self) -> usize {
        self.transitions.len()
    }

    /// Get info for all transitions (for UI)
    pub fn get_all_info(&self) -> Vec<TransitionInfo> {
        self.transitions.values()
            .map(|t| TransitionInfo::from(t))
            .collect()
    }

    /// Check for cycles (returns path if cycle found)
    pub fn detect_cycle(&self) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();
        
        for node_id in &self.known_nodes {
            if !visited.contains(node_id) {
                if self.detect_cycle_dfs(node_id, &mut visited, &mut rec_stack, &mut path) {
                    return Some(path);
                }
            }
        }
        
        None
    }

    fn detect_cycle_dfs(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());
        
        // Check all outgoing transitions
        if let Some(outgoing) = self.by_source.get(node) {
            for tid in outgoing {
                if let Some(t) = self.transitions.get(tid) {
                    let target = &t.target.node_id;
                    
                    if !visited.contains(target) {
                        if self.detect_cycle_dfs(target, visited, rec_stack, path) {
                            return true;
                        }
                    } else if rec_stack.contains(target) {
                        path.push(target.clone());
                        return true;
                    }
                }
            }
        }
        
        path.pop();
        rec_stack.remove(node);
        false
    }

    /// Get transitions sorted by priority (for execution order)
    pub fn get_sorted_by_priority(&self, source_node: &str) -> Vec<&Transition> {
        let mut transitions = self.get_by_source(source_node);
        transitions.sort_by_key(|t| t.priority);
        transitions
    }

    /// Validate all transitions
    pub fn validate(&self) -> ValidationResult {
        self.validator.validate_all(&self.transitions.values().cloned().collect::<Vec<_>>())
    }

    /// Import transitions (for loading saved state)
    pub fn import(&mut self, transitions: Vec<Transition>) {
        for t in transitions {
            // Update next_id if needed
            if let Some(num) = t.id.strip_prefix("t_").and_then(|s| s.parse::<u64>().ok()) {
                if num >= self.next_id {
                    self.next_id = num + 1;
                }
            }
            
            // Register nodes
            self.known_nodes.insert(t.source.node_id.clone());
            self.known_nodes.insert(t.target.node_id.clone());
            
            // Add to indices
            self.by_source.entry(t.source.node_id.clone())
                .or_default()
                .insert(t.id.clone());
            self.by_target.entry(t.target.node_id.clone())
                .or_default()
                .insert(t.id.clone());
            
            self.transitions.insert(t.id.clone(), t);
        }
    }

    /// Export all transitions (for saving state)
    pub fn export(&self) -> Vec<Transition> {
        self.get_all_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_transition() {
        let mut engine = TransitionEngine::new();
        engine.register_node("node1");
        engine.register_node("node2");
        
        let t = engine.create("node1", "node2").unwrap();
        assert!(t.id.starts_with("t_"));
        assert_eq!(t.source.node_id, "node1");
        assert_eq!(t.target.node_id, "node2");
    }

    #[test]
    fn test_get_by_node() {
        let mut engine = TransitionEngine::new();
        engine.register_node("a");
        engine.register_node("b");
        engine.register_node("c");
        
        engine.create("a", "b").unwrap();
        engine.create("b", "c").unwrap();
        
        assert_eq!(engine.get_by_source("a").len(), 1);
        assert_eq!(engine.get_by_source("b").len(), 1);
        assert_eq!(engine.get_by_target("b").len(), 1);
        assert_eq!(engine.get_by_node("b").len(), 2);
    }

    #[test]
    fn test_cycle_detection() {
        let mut engine = TransitionEngine::new();
        engine.register_node("a");
        engine.register_node("b");
        engine.register_node("c");
        
        engine.create("a", "b").unwrap();
        engine.create("b", "c").unwrap();
        
        assert!(engine.detect_cycle().is_none());
        
        engine.create("c", "a").unwrap();
        assert!(engine.detect_cycle().is_some());
    }
}
