//! Node Engine Core
//!
//! The brain of the node system:
//! - Node factory with defaults
//! - Connection validation
//! - Simulation support
//! - Code generation hooks

use super::node_types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A node instance in the design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier
    pub id: String,
    /// Display label
    pub label: String,
    /// Node type
    pub node_type: NodeType,
    /// X position on canvas
    pub x: f64,
    /// Y position on canvas
    pub y: f64,
    /// Width
    pub width: f64,
    /// Height
    pub height: f64,
    /// Port instances
    pub ports: NodePorts,
    /// Property values
    pub properties: HashMap<String, PropertyValue>,
    /// Entry action (for FSM states)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_action: Option<String>,
    /// Exit action (for FSM states)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_action: Option<String>,
    /// Description/comments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Node {
    /// Get center point
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
    
    /// Check if point is inside node
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width &&
        y >= self.y && y <= self.y + self.height
    }
    
    /// Get input port position by name
    pub fn input_port_position(&self, port_name: &str) -> Option<(f64, f64)> {
        let idx = self.ports.inputs.iter().position(|p| p.name == port_name)?;
        let count = self.ports.inputs.len();
        let spacing = self.height / (count + 1) as f64;
        Some((self.x, self.y + spacing * (idx + 1) as f64))
    }
    
    /// Get output port position by name
    pub fn output_port_position(&self, port_name: &str) -> Option<(f64, f64)> {
        let idx = self.ports.outputs.iter().position(|p| p.name == port_name)?;
        let count = self.ports.outputs.len();
        let spacing = self.height / (count + 1) as f64;
        Some((self.x + self.width, self.y + spacing * (idx + 1) as f64))
    }
}

/// A connection between two node ports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub source_node: String,
    pub source_port: String,
    pub target_node: String,  
    pub target_port: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}

/// Node engine manages node creation and validation
pub struct NodeEngine {
    nodes: HashMap<String, Node>,
    connections: HashMap<String, Connection>,
    next_id: u64,
}

impl NodeEngine {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: HashMap::new(),
            next_id: 1,
        }
    }
    
    /// Generate a unique ID
    fn generate_id(&mut self, prefix: &str) -> String {
        let id = format!("{}_{}", prefix, self.next_id);
        self.next_id += 1;
        id
    }
    
    /// Create a new node with defaults
    pub fn create_node(&mut self, node_type: NodeType, x: f64, y: f64) -> String {
        let id = self.generate_id("node");
        let label = node_type.display_name().to_string();
        let ports = node_type.default_ports();
        let properties = default_properties(&node_type)
            .into_iter()
            .map(|p| (p.key.clone(), p.value))
            .collect();
        
        // Size varies by category
        let (width, height) = match node_type.category() {
            NodeCategory::Fsm => (160.0, 80.0),
            NodeCategory::Hardware => (180.0, 100.0),
            NodeCategory::Processing => (140.0, 60.0),
            NodeCategory::Control => (160.0, 90.0),
            NodeCategory::Io => (140.0, 80.0),
            NodeCategory::Data => (120.0, 60.0),
        };
        
        let node = Node {
            id: id.clone(),
            label,
            node_type,
            x,
            y,
            width,
            height,
            ports,
            properties,
            entry_action: None,
            exit_action: None,
            description: None,
        };
        
        self.nodes.insert(id.clone(), node);
        id
    }
    
    /// Create a connection between nodes
    pub fn connect(
        &mut self,
        source_node: &str,
        source_port: &str,
        target_node: &str,
        target_port: &str,
    ) -> Result<String, ConnectionError> {
        // Validate nodes exist
        let source = self.nodes.get(source_node)
            .ok_or(ConnectionError::NodeNotFound(source_node.to_string()))?;
        let target = self.nodes.get(target_node)
            .ok_or(ConnectionError::NodeNotFound(target_node.to_string()))?;
        
        // Validate ports exist
        let source_port_def = source.ports.outputs.iter()
            .find(|p| p.name == source_port)
            .ok_or(ConnectionError::PortNotFound(source_port.to_string()))?;
        let target_port_def = target.ports.inputs.iter()
            .find(|p| p.name == target_port)
            .ok_or(ConnectionError::PortNotFound(target_port.to_string()))?;
        
        // Check type compatibility
        if !source_port_def.port_type.is_compatible(&target_port_def.port_type) {
            return Err(ConnectionError::IncompatibleTypes {
                source: format!("{:?}", source_port_def.port_type),
                target: format!("{:?}", target_port_def.port_type),
            });
        }
        
        // Check for self-loop
        if source_node == target_node {
            return Err(ConnectionError::SelfLoop);
        }
        
        // Check for duplicate
        let duplicate = self.connections.values().any(|c| {
            c.source_node == source_node && c.source_port == source_port &&
            c.target_node == target_node && c.target_port == target_port
        });
        if duplicate {
            return Err(ConnectionError::DuplicateConnection);
        }
        
        let id = self.generate_id("conn");
        let connection = Connection {
            id: id.clone(),
            source_node: source_node.to_string(),
            source_port: source_port.to_string(),
            target_node: target_node.to_string(),
            target_port: target_port.to_string(),
            label: None,
            condition: None,
        };
        
        self.connections.insert(id.clone(), connection);
        Ok(id)
    }
    
    /// Delete a node and its connections
    pub fn delete_node(&mut self, node_id: &str) -> bool {
        if self.nodes.remove(node_id).is_some() {
            // Remove all connections involving this node
            self.connections.retain(|_, c| {
                c.source_node != node_id && c.target_node != node_id
            });
            true
        } else {
            false
        }
    }
    
    /// Delete a connection
    pub fn delete_connection(&mut self, conn_id: &str) -> bool {
        self.connections.remove(conn_id).is_some()
    }
    
    /// Get a node by ID
    pub fn get_node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }
    
    /// Get a mutable node by ID
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }
    
    /// Get all nodes
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }
    
    /// Get all connections
    pub fn connections(&self) -> impl Iterator<Item = &Connection> {
        self.connections.values()
    }
    
    /// Get nodes by category
    pub fn nodes_by_category(&self, category: NodeCategory) -> Vec<&Node> {
        self.nodes.values()
            .filter(|n| n.node_type.category() == category)
            .collect()
    }
    
    /// Validate the entire design
    pub fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Check for exactly one initial state in FSM designs
        let initial_count = self.nodes.values()
            .filter(|n| matches!(n.node_type, NodeType::Initial))
            .count();
        
        if initial_count == 0 {
            warnings.push(ValidationWarning {
                code: "no_initial".to_string(),
                message: "No initial state defined".to_string(),
                node_ids: vec![],
            });
        } else if initial_count > 1 {
            errors.push(ValidationError {
                code: "multiple_initial".to_string(),
                message: "Multiple initial states defined".to_string(),
                node_ids: self.nodes.values()
                    .filter(|n| matches!(n.node_type, NodeType::Initial))
                    .map(|n| n.id.clone())
                    .collect(),
            });
        }
        
        // Check for unconnected required ports
        for node in self.nodes.values() {
            for input in &node.ports.inputs {
                if input.required {
                    let connected = self.connections.values()
                        .any(|c| c.target_node == node.id && c.target_port == input.name);
                    if !connected {
                        warnings.push(ValidationWarning {
                            code: "unconnected_required".to_string(),
                            message: format!("Required input '{}' not connected", input.name),
                            node_ids: vec![node.id.clone()],
                        });
                    }
                }
            }
        }
        
        // Check for nodes with no connections (orphans)
        for node in self.nodes.values() {
            let has_connection = self.connections.values()
                .any(|c| c.source_node == node.id || c.target_node == node.id);
            if !has_connection && !matches!(node.node_type, NodeType::Initial | NodeType::Final) {
                warnings.push(ValidationWarning {
                    code: "orphan_node".to_string(),
                    message: format!("Node '{}' has no connections", node.label),
                    node_ids: vec![node.id.clone()],
                });
            }
        }
        
        ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
    
    /// Get node count by category
    pub fn count_by_category(&self) -> HashMap<NodeCategory, usize> {
        let mut counts = HashMap::new();
        for node in self.nodes.values() {
            *counts.entry(node.node_type.category()).or_insert(0) += 1;
        }
        counts
    }
    
    /// Export to serializable state
    pub fn export(&self) -> DesignState {
        DesignState {
            nodes: self.nodes.values().cloned().collect(),
            connections: self.connections.values().cloned().collect(),
        }
    }
    
    /// Import from serializable state
    pub fn import(&mut self, state: DesignState) {
        self.nodes.clear();
        self.connections.clear();
        
        for node in state.nodes {
            self.nodes.insert(node.id.clone(), node);
        }
        for conn in state.connections {
            self.connections.insert(conn.id.clone(), conn);
        }
        
        // Update next_id to avoid collisions
        let max_node_id = self.nodes.keys()
            .filter_map(|id| id.rsplit('_').next()?.parse::<u64>().ok())
            .max()
            .unwrap_or(0);
        let max_conn_id = self.connections.keys()
            .filter_map(|id| id.rsplit('_').next()?.parse::<u64>().ok())
            .max()
            .unwrap_or(0);
        self.next_id = max_node_id.max(max_conn_id) + 1;
    }
}

impl Default for NodeEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Connection errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionError {
    NodeNotFound(String),
    PortNotFound(String),
    IncompatibleTypes { source: String, target: String },
    SelfLoop,
    DuplicateConnection,
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NodeNotFound(id) => write!(f, "Node not found: {}", id),
            Self::PortNotFound(name) => write!(f, "Port not found: {}", name),
            Self::IncompatibleTypes { source, target } => {
                write!(f, "Incompatible port types: {} -> {}", source, target)
            }
            Self::SelfLoop => write!(f, "Cannot create self-loop"),
            Self::DuplicateConnection => write!(f, "Connection already exists"),
        }
    }
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub node_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub code: String,
    pub message: String,
    pub node_ids: Vec<String>,
}

/// Exportable design state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignState {
    pub nodes: Vec<Node>,
    pub connections: Vec<Connection>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_node() {
        let mut engine = NodeEngine::new();
        let id = engine.create_node(NodeType::Gpio, 100.0, 100.0);
        
        let node = engine.get_node(&id).unwrap();
        assert_eq!(node.node_type, NodeType::Gpio);
        assert!(!node.ports.inputs.is_empty());
    }
    
    #[test]
    fn test_connect_nodes() {
        let mut engine = NodeEngine::new();
        let gpio = engine.create_node(NodeType::Gpio, 0.0, 0.0);
        let led = engine.create_node(NodeType::Led, 200.0, 0.0);
        
        // This should work (digital -> digital)
        let result = engine.connect(&gpio, "read", &led, "in");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validation() {
        let mut engine = NodeEngine::new();
        engine.create_node(NodeType::State, 0.0, 0.0);
        
        let result = engine.validate();
        // Should warn about no initial state
        assert!(!result.warnings.is_empty());
    }
}
