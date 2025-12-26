//! Enhanced Node Engine Module
//!
//! A powerful node system for embedded systems design:
//! - 40+ node types across 6 categories (FSM, Hardware, Processing, Control, I/O, Data)
//! - Typed port system with connection validation
//! - RTOS support (Tasks, Semaphores, Mutexes)
//! - Property system for node configuration

pub mod node_types;
pub mod node_engine;
pub mod codegen;

// Re-export main types
pub use node_types::{
    NodeCategory,
    NodeType,
    PortType,
    PortDirection,
    PortDef,
    NodePorts,
    PropertyValue,
    PropertyDef,
    default_properties,
};

pub use node_engine::{
    Node,
    Connection,
    NodeEngine,
    ConnectionError,
    ValidationResult,
    ValidationError,
    ValidationWarning,
    DesignState,
};

/// Get all node types organized by category
pub fn get_node_palette() -> Vec<(NodeCategory, Vec<NodeType>)> {
    NodeCategory::all()
        .into_iter()
        .map(|cat| (cat, NodeType::by_category(cat)))
        .collect()
}

/// Node type info for UI display
#[derive(Debug, Clone, serde::Serialize)]
pub struct NodeTypeInfo {
    pub node_type: NodeType,
    pub category: NodeCategory,
    pub name: String,
    pub icon: String,
    pub ports: NodePorts,
}

/// Get info for all node types
pub fn get_all_node_info() -> Vec<NodeTypeInfo> {
    let mut info = Vec::new();
    
    for category in NodeCategory::all() {
        for node_type in NodeType::by_category(category) {
            info.push(NodeTypeInfo {
                node_type: node_type.clone(),
                category,
                name: node_type.display_name().to_string(),
                icon: node_type.icon().to_string(),
                ports: node_type.default_ports(),
            });
        }
    }
    
    info
}
