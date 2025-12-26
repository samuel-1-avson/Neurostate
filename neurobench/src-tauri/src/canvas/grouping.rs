//! Node Grouping - Hierarchy and compound nodes for complex designs
//!
//! Supports sub-FSMs, node groups, and nested hierarchies.

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

/// A group of nodes that can be collapsed/expanded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGroup {
    /// Unique group ID
    pub id: String,
    /// Display label
    pub label: String,
    /// IDs of child nodes
    pub children: HashSet<String>,
    /// IDs of child groups (nested hierarchy)
    pub child_groups: HashSet<String>,
    /// Parent group ID (if nested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Whether group is collapsed
    pub collapsed: bool,
    /// Group color for visual distinction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Position when collapsed (center of group)
    pub collapsed_x: f64,
    pub collapsed_y: f64,
    /// Size when collapsed
    pub collapsed_width: f64,
    pub collapsed_height: f64,
}

impl NodeGroup {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            children: HashSet::new(),
            child_groups: HashSet::new(),
            parent: None,
            collapsed: false,
            color: None,
            description: None,
            collapsed_x: 0.0,
            collapsed_y: 0.0,
            collapsed_width: 200.0,
            collapsed_height: 120.0,
        }
    }

    /// Add a node to this group
    pub fn add_node(&mut self, node_id: impl Into<String>) {
        self.children.insert(node_id.into());
    }

    /// Remove a node from this group
    pub fn remove_node(&mut self, node_id: &str) -> bool {
        self.children.remove(node_id)
    }

    /// Add a child group
    pub fn add_child_group(&mut self, group_id: impl Into<String>) {
        self.child_groups.insert(group_id.into());
    }

    /// Check if a node is in this group
    pub fn contains_node(&self, node_id: &str) -> bool {
        self.children.contains(node_id)
    }

    /// Get all node IDs in this group
    pub fn node_ids(&self) -> Vec<String> {
        self.children.iter().cloned().collect()
    }

    /// Toggle collapsed state
    pub fn toggle_collapse(&mut self) {
        self.collapsed = !self.collapsed;
    }

    /// Set color
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }
}

/// Manages all groups and their hierarchy
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GroupManager {
    /// All groups indexed by ID
    groups: HashMap<String, NodeGroup>,
    /// Map of node ID to group ID
    node_to_group: HashMap<String, String>,
}

impl GroupManager {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
            node_to_group: HashMap::new(),
        }
    }

    /// Create a new group from selected nodes
    pub fn create_group(&mut self, id: impl Into<String>, label: impl Into<String>, node_ids: Vec<String>) -> String {
        let id = id.into();
        let mut group = NodeGroup::new(id.clone(), label);

        for node_id in node_ids {
            group.add_node(node_id.clone());
            self.node_to_group.insert(node_id, id.clone());
        }

        self.groups.insert(id.clone(), group);
        id
    }

    /// Ungroup - dissolve a group and release its nodes
    pub fn ungroup(&mut self, group_id: &str) -> Option<Vec<String>> {
        if let Some(group) = self.groups.remove(group_id) {
            let node_ids: Vec<String> = group.children.into_iter().collect();
            
            for node_id in &node_ids {
                self.node_to_group.remove(node_id);
            }

            // Move child groups to parent (or make them root)
            for child_group_id in group.child_groups {
                if let Some(child_group) = self.groups.get_mut(&child_group_id) {
                    child_group.parent = group.parent.clone();
                }
            }

            Some(node_ids)
        } else {
            None
        }
    }

    /// Add a node to an existing group
    pub fn add_node_to_group(&mut self, node_id: impl Into<String>, group_id: &str) -> bool {
        let node_id = node_id.into();
        
        if let Some(group) = self.groups.get_mut(group_id) {
            group.add_node(node_id.clone());
            self.node_to_group.insert(node_id, group_id.to_string());
            true
        } else {
            false
        }
    }

    /// Remove a node from its group
    pub fn remove_node_from_group(&mut self, node_id: &str) -> Option<String> {
        if let Some(group_id) = self.node_to_group.remove(node_id) {
            if let Some(group) = self.groups.get_mut(&group_id) {
                group.remove_node(node_id);
            }
            Some(group_id)
        } else {
            None
        }
    }

    /// Get the group a node belongs to
    pub fn get_node_group(&self, node_id: &str) -> Option<&NodeGroup> {
        self.node_to_group.get(node_id)
            .and_then(|gid| self.groups.get(gid))
    }

    /// Get group by ID
    pub fn get_group(&self, group_id: &str) -> Option<&NodeGroup> {
        self.groups.get(group_id)
    }

    /// Get mutable group by ID
    pub fn get_group_mut(&mut self, group_id: &str) -> Option<&mut NodeGroup> {
        self.groups.get_mut(group_id)
    }

    /// Collapse a group
    pub fn collapse(&mut self, group_id: &str) -> bool {
        if let Some(group) = self.groups.get_mut(group_id) {
            group.collapsed = true;
            true
        } else {
            false
        }
    }

    /// Expand a group
    pub fn expand(&mut self, group_id: &str) -> bool {
        if let Some(group) = self.groups.get_mut(group_id) {
            group.collapsed = false;
            true
        } else {
            false
        }
    }

    /// Toggle collapse state
    pub fn toggle_collapse(&mut self, group_id: &str) -> bool {
        if let Some(group) = self.groups.get_mut(group_id) {
            group.toggle_collapse();
            true
        } else {
            false
        }
    }

    /// Get all collapsed groups
    pub fn collapsed_groups(&self) -> Vec<&NodeGroup> {
        self.groups.values().filter(|g| g.collapsed).collect()
    }

    /// Get all root groups (no parent)
    pub fn root_groups(&self) -> Vec<&NodeGroup> {
        self.groups.values().filter(|g| g.parent.is_none()).collect()
    }

    /// Get all group IDs
    pub fn group_ids(&self) -> Vec<String> {
        self.groups.keys().cloned().collect()
    }

    /// Check if a node is in any collapsed group
    pub fn is_node_hidden(&self, node_id: &str) -> bool {
        if let Some(group_id) = self.node_to_group.get(node_id) {
            if let Some(group) = self.groups.get(group_id) {
                return group.collapsed;
            }
        }
        false
    }

    /// Get visible nodes (not in collapsed groups)
    pub fn visible_nodes<'a>(&self, all_nodes: &'a [String]) -> Vec<&'a String> {
        all_nodes.iter()
            .filter(|id| !self.is_node_hidden(id))
            .collect()
    }

    /// Clear all groups
    pub fn clear(&mut self) {
        self.groups.clear();
        self.node_to_group.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_group() {
        let mut manager = GroupManager::new();
        
        let node_ids = vec!["n1".to_string(), "n2".to_string(), "n3".to_string()];
        let group_id = manager.create_group("g1", "My Group", node_ids);

        assert_eq!(group_id, "g1");
        assert!(manager.get_group("g1").is_some());
        assert!(manager.get_node_group("n1").is_some());
    }

    #[test]
    fn test_collapse_expand() {
        let mut manager = GroupManager::new();
        manager.create_group("g1", "Group", vec!["n1".to_string()]);

        assert!(!manager.is_node_hidden("n1"));

        manager.collapse("g1");
        assert!(manager.is_node_hidden("n1"));

        manager.expand("g1");
        assert!(!manager.is_node_hidden("n1"));
    }

    #[test]
    fn test_ungroup() {
        let mut manager = GroupManager::new();
        manager.create_group("g1", "Group", vec!["n1".to_string(), "n2".to_string()]);

        let released = manager.ungroup("g1");
        assert_eq!(released.map(|v| v.len()), Some(2));
        assert!(manager.get_group("g1").is_none());
    }
}
