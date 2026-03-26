//! Connections - Smart connection management and auto-routing
//!
//! Provides intelligent connection handling with port compatibility.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::canvas::types::{CanvasEdge, CanvasNode};
use crate::canvas::ports::{Port, PortDataType};

/// Connection rule for validating connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionRule {
    /// Source port types allowed
    pub source_types: Vec<PortDataType>,
    /// Target port types allowed
    pub target_types: Vec<PortDataType>,
    /// Maximum connections from source
    pub max_source_connections: u32,
    /// Maximum connections to target
    pub max_target_connections: u32,
}

/// Connection validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionValidation {
    pub valid: bool,
    pub reason: Option<String>,
    pub suggested_port: Option<String>,
}

/// Smart connection manager
#[derive(Debug, Clone, Default)]
pub struct ConnectionManager {
    /// Connections by edge ID
    connections: HashMap<String, EdgeConnection>,
    /// Port connection counts
    port_counts: HashMap<String, u32>,
    /// Rules for connection validation
    rules: Vec<ConnectionRule>,
}

#[derive(Debug, Clone)]
pub struct EdgeConnection {
    pub edge_id: String,
    pub source_node: String,
    pub source_port: Option<String>,
    pub target_node: String,
    pub target_port: Option<String>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate if connection is allowed
    pub fn validate_connection(
        &self,
        source_node: &CanvasNode,
        source_port: Option<&Port>,
        target_node: &CanvasNode,
        target_port: Option<&Port>,
    ) -> ConnectionValidation {
        // Check self-connection
        if source_node.id == target_node.id {
            return ConnectionValidation {
                valid: false,
                reason: Some("Cannot connect node to itself".into()),
                suggested_port: None,
            };
        }

        // Check port compatibility if ports are specified
        if let (Some(sp), Some(tp)) = (source_port, target_port) {
            if !sp.can_connect_to(tp) {
                return ConnectionValidation {
                    valid: false,
                    reason: Some(format!(
                        "Port types incompatible: {:?} -> {:?}",
                        sp.data_type, tp.data_type
                    )),
                    suggested_port: None,
                };
            }

            // Check connection limits
            let source_key = format!("{}:{}", source_node.id, sp.id);
            let target_key = format!("{}:{}", target_node.id, tp.id);

            if sp.max_connections > 0 {
                let count = self.port_counts.get(&source_key).unwrap_or(&0);
                if *count >= sp.max_connections {
                    return ConnectionValidation {
                        valid: false,
                        reason: Some("Source port connection limit reached".into()),
                        suggested_port: None,
                    };
                }
            }

            if tp.max_connections > 0 {
                let count = self.port_counts.get(&target_key).unwrap_or(&0);
                if *count >= tp.max_connections {
                    return ConnectionValidation {
                        valid: false,
                        reason: Some("Target port connection limit reached".into()),
                        suggested_port: None,
                    };
                }
            }
        }

        ConnectionValidation {
            valid: true,
            reason: None,
            suggested_port: None,
        }
    }

    /// Register a connection
    pub fn add_connection(&mut self, edge: &CanvasEdge) {
        let conn = EdgeConnection {
            edge_id: edge.id.clone(),
            source_node: edge.source.clone(),
            source_port: edge.source_port.clone(),
            target_node: edge.target.clone(),
            target_port: edge.target_port.clone(),
        };

        // Update counts
        if let Some(ref sp) = edge.source_port {
            let key = format!("{}:{}", edge.source, sp);
            *self.port_counts.entry(key).or_default() += 1;
        }
        if let Some(ref tp) = edge.target_port {
            let key = format!("{}:{}", edge.target, tp);
            *self.port_counts.entry(key).or_default() += 1;
        }

        self.connections.insert(edge.id.clone(), conn);
    }

    /// Remove a connection
    pub fn remove_connection(&mut self, edge_id: &str) {
        if let Some(conn) = self.connections.remove(edge_id) {
            if let Some(ref sp) = conn.source_port {
                let key = format!("{}:{}", conn.source_node, sp);
                if let Some(count) = self.port_counts.get_mut(&key) {
                    *count = count.saturating_sub(1);
                }
            }
            if let Some(ref tp) = conn.target_port {
                let key = format!("{}:{}", conn.target_node, tp);
                if let Some(count) = self.port_counts.get_mut(&key) {
                    *count = count.saturating_sub(1);
                }
            }
        }
    }

    /// Get connections for a node
    pub fn get_node_connections(&self, node_id: &str) -> Vec<&EdgeConnection> {
        self.connections.values()
            .filter(|c| c.source_node == node_id || c.target_node == node_id)
            .collect()
    }

    /// Find compatible port on target node
    pub fn find_compatible_port(
        &self,
        source_port: &Port,
        target_node: &CanvasNode,
        target_ports: &[Port],
    ) -> Option<String> {
        for port in target_ports {
            if source_port.can_connect_to(port) {
                let key = format!("{}:{}", target_node.id, port.id);
                let count = self.port_counts.get(&key).unwrap_or(&0);
                if port.max_connections == 0 || *count < port.max_connections {
                    return Some(port.id.clone());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::types::NodeType;

    #[test]
    fn test_self_connection_blocked() {
        let manager = ConnectionManager::new();
        let node = CanvasNode::new("n1".into(), "Test".into(), NodeType::State, 0.0, 0.0);
        
        let result = manager.validate_connection(&node, None, &node, None);
        assert!(!result.valid);
    }
}
