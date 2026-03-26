//! Shared Types Module
//!
//! Single source of truth for types shared across engines:
//! - NodeType: All node types across all categories
//! - NodeCategory: Category classification
//! - PortType: Port data types for connections
//! - PropertyValue: Node property values

pub mod node_types;
pub mod port_types;
pub mod properties;

// Re-export main types for convenience
pub use node_types::{NodeType, NodeCategory, NodeTypeInfo, get_all_node_info, NODE_TYPE_INFO};
pub use port_types::{PortType, PortDef, PortDirection, PortPosition, NodePorts, default_ports};
pub use properties::{PropertyValue, PropertyDef, PropertyValidation, default_properties};
