//! Port Types - Connection port types and definitions
//!
//! Defines port data types, directions, and port configurations for nodes.

use serde::{Deserialize, Serialize};

// ============================================================================
// PORT TYPES
// ============================================================================

/// Port data types for connection validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PortType {
    /// Execution flow (FSM transitions)
    Flow,
    /// Boolean/digital signal (0/1)
    Digital,
    /// Analog value (0.0-1.0 normalized or raw)
    Analog,
    /// Integer value
    Integer,
    /// Floating point value
    Float,
    /// Generic data (bytes/buffer)
    Data,
    /// Edge-triggered event
    Event,
    /// Multi-signal bus with width
    Bus { width: u8 },
    /// PWM signal
    Pwm,
    /// Clock signal
    Clock,
    /// Power/ground reference
    Power,
    /// Any type (accepts all)
    Any,
    /// Custom type
    Custom(String),
}

impl Default for PortType {
    fn default() -> Self {
        Self::Flow
    }
}

impl PortType {
    /// Check if two port types are compatible for connection
    pub fn is_compatible(&self, other: &Self) -> bool {
        match (self, other) {
            // Same type always compatible
            (a, b) if a == b => true,
            
            // Any type accepts everything
            (Self::Any, _) | (_, Self::Any) => true,
            
            // Flow compatibility
            (Self::Flow, Self::Flow) => true,
            (Self::Event, Self::Flow) | (Self::Flow, Self::Event) => true,
            
            // Numeric type compatibility
            (Self::Analog, Self::Float) | (Self::Float, Self::Analog) => true,
            (Self::Digital, Self::Integer) | (Self::Integer, Self::Digital) => true,
            (Self::Integer, Self::Float) | (Self::Float, Self::Integer) => true,
            (Self::Analog, Self::Integer) | (Self::Integer, Self::Analog) => true,
            
            // Digital can be connected to analog (0.0 or 1.0)
            (Self::Digital, Self::Analog) | (Self::Analog, Self::Digital) => true,
            
            // PWM compatible with analog/digital
            (Self::Pwm, Self::Analog) | (Self::Analog, Self::Pwm) => true,
            (Self::Pwm, Self::Digital) | (Self::Digital, Self::Pwm) => true,
            
            // Bus compatibility (must match width)
            (Self::Bus { width: w1 }, Self::Bus { width: w2 }) => w1 == w2,
            
            // Clock to digital
            (Self::Clock, Self::Digital) | (Self::Digital, Self::Clock) => true,
            
            // Everything else is incompatible
            _ => false,
        }
    }
    
    /// Get the color for this port type (for UI rendering)
    pub fn color(&self) -> &str {
        match self {
            Self::Flow => "#4CAF50",      // Green
            Self::Digital => "#2196F3",   // Blue
            Self::Analog => "#FF9800",    // Orange
            Self::Integer => "#9C27B0",   // Purple
            Self::Float => "#E91E63",     // Pink
            Self::Data => "#607D8B",      // Gray
            Self::Event => "#F44336",     // Red
            Self::Bus { .. } => "#795548", // Brown
            Self::Pwm => "#00BCD4",       // Cyan
            Self::Clock => "#FFEB3B",     // Yellow
            Self::Power => "#FF5722",     // Deep Orange
            Self::Any => "#9E9E9E",       // Gray
            Self::Custom(_) => "#673AB7", // Deep Purple
        }
    }
    
    /// Get display name for this port type
    pub fn display_name(&self) -> String {
        match self {
            Self::Flow => "Flow".into(),
            Self::Digital => "Digital".into(),
            Self::Analog => "Analog".into(),
            Self::Integer => "Integer".into(),
            Self::Float => "Float".into(),
            Self::Data => "Data".into(),
            Self::Event => "Event".into(),
            Self::Bus { width } => format!("Bus[{}]", width),
            Self::Pwm => "PWM".into(),
            Self::Clock => "Clock".into(),
            Self::Power => "Power".into(),
            Self::Any => "Any".into(),
            Self::Custom(name) => name.clone(),
        }
    }
}

// ============================================================================
// PORT DIRECTION
// ============================================================================

/// Port direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PortDirection {
    /// Input port (receives data)
    Input,
    /// Output port (sends data)
    Output,
    /// Bidirectional port (can send and receive)
    Bidirectional,
}

impl Default for PortDirection {
    fn default() -> Self {
        Self::Input
    }
}

// ============================================================================
// PORT POSITION
// ============================================================================

/// Position of port on node
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PortPosition {
    /// Top edge of node
    Top,
    /// Bottom edge of node
    #[default]
    Bottom,
    /// Left edge of node
    Left,
    /// Right edge of node
    Right,
}

// ============================================================================
// PORT DEFINITION
// ============================================================================

/// Port definition for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDef {
    /// Unique identifier within the node
    pub id: String,
    /// Display name
    pub name: String,
    /// Port data type
    pub port_type: PortType,
    /// Port direction
    pub direction: PortDirection,
    /// Position on node
    pub position: PortPosition,
    /// Offset from center on the edge (0.0 = center)
    pub offset: f64,
    /// Whether connection is required
    pub required: bool,
    /// Maximum number of connections (0 = unlimited)
    pub max_connections: u32,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl PortDef {
    /// Create a new port definition
    pub fn new(name: &str, port_type: PortType) -> Self {
        Self {
            id: name.to_lowercase().replace(' ', "_"),
            name: name.to_string(),
            port_type,
            direction: PortDirection::Input,
            position: PortPosition::Left,
            offset: 0.0,
            required: false,
            max_connections: 0,
            description: None,
        }
    }
    
    /// Create a flow port (for FSM transitions)
    pub fn flow(name: &str) -> Self {
        Self {
            id: name.to_lowercase().replace(' ', "_"),
            name: name.to_string(),
            port_type: PortType::Flow,
            direction: PortDirection::Input,
            position: PortPosition::Top,
            offset: 0.0,
            required: true,
            max_connections: 0,
            description: None,
        }
    }
    
    /// Create an input port
    pub fn input(name: &str, port_type: PortType) -> Self {
        Self {
            id: name.to_lowercase().replace(' ', "_"),
            name: name.to_string(),
            port_type,
            direction: PortDirection::Input,
            position: PortPosition::Left,
            offset: 0.0,
            required: false,
            max_connections: 1,
            description: None,
        }
    }
    
    /// Create an output port
    pub fn output(name: &str, port_type: PortType) -> Self {
        Self {
            id: name.to_lowercase().replace(' ', "_"),
            name: name.to_string(),
            port_type,
            direction: PortDirection::Output,
            position: PortPosition::Right,
            offset: 0.0,
            required: false,
            max_connections: 0, // Outputs can have unlimited connections
            description: None,
        }
    }
    
    /// Builder: set as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
    
    /// Builder: set position
    pub fn at(mut self, position: PortPosition) -> Self {
        self.position = position;
        self
    }
    
    /// Builder: set offset
    pub fn offset(mut self, offset: f64) -> Self {
        self.offset = offset;
        self
    }
    
    /// Builder: set max connections
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }
    
    /// Builder: set description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }
}

// ============================================================================
// NODE PORTS COLLECTION
// ============================================================================

/// Collection of ports for a node
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodePorts {
    /// Input ports
    pub inputs: Vec<PortDef>,
    /// Output ports
    pub outputs: Vec<PortDef>,
}

impl NodePorts {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add an input port
    pub fn add_input(mut self, port: PortDef) -> Self {
        self.inputs.push(port);
        self
    }
    
    /// Add an output port
    pub fn add_output(mut self, port: PortDef) -> Self {
        self.outputs.push(port);
        self
    }
    
    /// Get port by ID
    pub fn get_port(&self, id: &str) -> Option<&PortDef> {
        self.inputs.iter()
            .chain(self.outputs.iter())
            .find(|p| p.id == id)
    }
    
    /// Check if port exists
    pub fn has_port(&self, id: &str) -> bool {
        self.get_port(id).is_some()
    }
}

// ============================================================================
// DEFAULT PORTS FOR NODE TYPES
// ============================================================================

use super::node_types::NodeType;

/// Get default ports for a node type
pub fn default_ports(node_type: &NodeType) -> NodePorts {
    match node_type {
        // === FSM Nodes ===
        NodeType::State => NodePorts::new()
            .add_input(PortDef::flow("in").at(PortPosition::Top))
            .add_output(PortDef::flow("out").at(PortPosition::Bottom)),
            
        NodeType::Initial => NodePorts::new()
            .add_output(PortDef::flow("out").at(PortPosition::Bottom)),
            
        NodeType::Final => NodePorts::new()
            .add_input(PortDef::flow("in").at(PortPosition::Top)),
            
        NodeType::Decision => NodePorts::new()
            .add_input(PortDef::flow("in").at(PortPosition::Top))
            .add_input(PortDef::input("condition", PortType::Digital).at(PortPosition::Left))
            .add_output(PortDef::flow("true").at(PortPosition::Right).offset(-20.0))
            .add_output(PortDef::flow("false").at(PortPosition::Right).offset(20.0)),
            
        NodeType::Fork => NodePorts::new()
            .add_input(PortDef::flow("in").at(PortPosition::Top))
            .add_output(PortDef::flow("out1").at(PortPosition::Bottom).offset(-30.0))
            .add_output(PortDef::flow("out2").at(PortPosition::Bottom).offset(30.0)),
            
        NodeType::Join => NodePorts::new()
            .add_input(PortDef::flow("in1").at(PortPosition::Top).offset(-30.0))
            .add_input(PortDef::flow("in2").at(PortPosition::Top).offset(30.0))
            .add_output(PortDef::flow("out").at(PortPosition::Bottom)),
            
        // === Hardware Nodes ===
        NodeType::Gpio => NodePorts::new()
            .add_input(PortDef::input("write", PortType::Digital))
            .add_output(PortDef::output("read", PortType::Digital)),
            
        NodeType::Adc => NodePorts::new()
            .add_input(PortDef::input("trigger", PortType::Event))
            .add_output(PortDef::output("value", PortType::Analog))
            .add_output(PortDef::output("ready", PortType::Event)),
            
        NodeType::Dac => NodePorts::new()
            .add_input(PortDef::input("value", PortType::Analog))
            .add_input(PortDef::input("enable", PortType::Digital)),
            
        NodeType::Pwm => NodePorts::new()
            .add_input(PortDef::input("duty", PortType::Analog))
            .add_input(PortDef::input("enable", PortType::Digital))
            .add_output(PortDef::output("output", PortType::Pwm)),
            
        // === Communication Nodes ===
        NodeType::Uart => NodePorts::new()
            .add_input(PortDef::input("tx_data", PortType::Data))
            .add_output(PortDef::output("rx_data", PortType::Data))
            .add_output(PortDef::output("rx_ready", PortType::Event)),
            
        NodeType::Spi => NodePorts::new()
            .add_input(PortDef::input("tx_data", PortType::Data))
            .add_input(PortDef::input("cs", PortType::Digital))
            .add_output(PortDef::output("rx_data", PortType::Data))
            .add_output(PortDef::output("done", PortType::Event)),
            
        NodeType::I2c => NodePorts::new()
            .add_input(PortDef::input("address", PortType::Integer))
            .add_input(PortDef::input("tx_data", PortType::Data))
            .add_output(PortDef::output("rx_data", PortType::Data))
            .add_output(PortDef::output("done", PortType::Event)),
            
        // === Timer Nodes ===
        NodeType::Timer => NodePorts::new()
            .add_input(PortDef::input("start", PortType::Event))
            .add_input(PortDef::input("stop", PortType::Event))
            .add_output(PortDef::output("timeout", PortType::Event))
            .add_output(PortDef::output("counter", PortType::Integer)),
            
        NodeType::Delay => NodePorts::new()
            .add_input(PortDef::flow("trigger"))
            .add_output(PortDef::flow("done")),
            
        // === RTOS Nodes ===
        NodeType::Task => NodePorts::new()
            .add_input(PortDef::flow("entry"))
            .add_output(PortDef::output("signal", PortType::Event)),
            
        NodeType::Semaphore => NodePorts::new()
            .add_input(PortDef::input("give", PortType::Event))
            .add_input(PortDef::input("take", PortType::Event))
            .add_output(PortDef::output("acquired", PortType::Event)),
            
        NodeType::Mutex => NodePorts::new()
            .add_input(PortDef::input("lock", PortType::Event))
            .add_input(PortDef::input("unlock", PortType::Event))
            .add_output(PortDef::output("locked", PortType::Event)),
            
        NodeType::MessageQueue => NodePorts::new()
            .add_input(PortDef::input("send", PortType::Data))
            .add_output(PortDef::output("receive", PortType::Data))
            .add_output(PortDef::output("available", PortType::Event)),
            
        // === I/O Nodes ===
        NodeType::Led => NodePorts::new()
            .add_input(PortDef::input("state", PortType::Digital)),
            
        NodeType::Button => NodePorts::new()
            .add_output(PortDef::output("pressed", PortType::Event))
            .add_output(PortDef::output("released", PortType::Event))
            .add_output(PortDef::output("state", PortType::Digital)),
            
        NodeType::Motor | NodeType::MotorDc => NodePorts::new()
            .add_input(PortDef::input("speed", PortType::Analog))
            .add_input(PortDef::input("direction", PortType::Digital))
            .add_input(PortDef::input("enable", PortType::Digital)),
            
        NodeType::Encoder => NodePorts::new()
            .add_output(PortDef::output("position", PortType::Integer))
            .add_output(PortDef::output("velocity", PortType::Float))
            .add_output(PortDef::output("direction", PortType::Digital)),
            
        // === Wireless Nodes ===
        NodeType::Wifi => NodePorts::new()
            .add_input(PortDef::input("tx_data", PortType::Data))
            .add_output(PortDef::output("rx_data", PortType::Data))
            .add_output(PortDef::output("connected", PortType::Digital))
            .add_output(PortDef::output("rssi", PortType::Integer)),
            
        NodeType::Ble => NodePorts::new()
            .add_input(PortDef::input("tx_data", PortType::Data))
            .add_output(PortDef::output("rx_data", PortType::Data))
            .add_output(PortDef::output("connected", PortType::Digital)),
            
        // === Protocol Nodes ===
        NodeType::Mqtt => NodePorts::new()
            .add_input(PortDef::input("publish", PortType::Data))
            .add_output(PortDef::output("message", PortType::Data))
            .add_output(PortDef::output("connected", PortType::Digital)),
            
        NodeType::Http => NodePorts::new()
            .add_input(PortDef::input("request", PortType::Data))
            .add_output(PortDef::output("response", PortType::Data))
            .add_output(PortDef::output("done", PortType::Event)),
            
        // === Default for others ===
        _ => NodePorts::new()
            .add_input(PortDef::flow("in").at(PortPosition::Top))
            .add_output(PortDef::flow("out").at(PortPosition::Bottom)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_port_compatibility() {
        assert!(PortType::Flow.is_compatible(&PortType::Flow));
        assert!(PortType::Analog.is_compatible(&PortType::Float));
        assert!(PortType::Digital.is_compatible(&PortType::Integer));
        assert!(PortType::Event.is_compatible(&PortType::Flow));
        assert!(!PortType::Digital.is_compatible(&PortType::Data));
        assert!(PortType::Any.is_compatible(&PortType::Digital));
    }
    
    #[test]
    fn test_default_ports() {
        let gpio_ports = default_ports(&NodeType::Gpio);
        assert_eq!(gpio_ports.inputs.len(), 1);
        assert_eq!(gpio_ports.outputs.len(), 1);
        
        let decision_ports = default_ports(&NodeType::Decision);
        assert_eq!(decision_ports.inputs.len(), 2); // in + condition
        assert_eq!(decision_ports.outputs.len(), 2); // true + false
    }
}
