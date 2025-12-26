//! Port System - Typed input/output ports for nodes
//!
//! Provides a structured port system with data types for proper connection validation.

use serde::{Serialize, Deserialize};

/// Direction of a port
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PortDirection {
    /// Input port (receives data)
    Input,
    /// Output port (sends data)
    Output,
    /// Bidirectional port
    Bidirectional,
}

/// Data type that flows through a port
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PortDataType {
    /// Digital signal (high/low)
    Digital,
    /// Analog signal (voltage/current)
    Analog,
    /// Serial data stream (UART, SPI, I2C)
    Serial,
    /// Parallel data bus
    Parallel,
    /// Event/trigger signal
    Event,
    /// Integer value
    Integer,
    /// Floating point value
    Float,
    /// Boolean value
    Boolean,
    /// String/text data
    String,
    /// Binary data buffer
    Buffer,
    /// PWM signal (duty cycle)
    Pwm,
    /// Clock signal
    Clock,
    /// Power rail
    Power,
    /// Ground reference
    Ground,
    /// Generic/any type
    Any,
    /// Custom data type
    Custom(String),
}

/// Position of a port on a node
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PortPosition {
    Top,
    Bottom,
    Left,
    Right,
}

/// A port definition for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    /// Unique ID within the node
    pub id: String,
    /// Display name
    pub name: String,
    /// Port direction
    pub direction: PortDirection,
    /// Data type
    pub data_type: PortDataType,
    /// Position on the node
    pub position: PortPosition,
    /// Offset from edge (0.0 to 1.0)
    pub offset: f64,
    /// Whether this port is required
    pub required: bool,
    /// Maximum number of connections (0 = unlimited)
    pub max_connections: u32,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Port {
    /// Create a new input port
    pub fn input(id: impl Into<String>, name: impl Into<String>, data_type: PortDataType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            direction: PortDirection::Input,
            data_type,
            position: PortPosition::Left,
            offset: 0.5,
            required: false,
            max_connections: 1,
            description: None,
        }
    }

    /// Create a new output port
    pub fn output(id: impl Into<String>, name: impl Into<String>, data_type: PortDataType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            direction: PortDirection::Output,
            data_type,
            position: PortPosition::Right,
            offset: 0.5,
            required: false,
            max_connections: 0, // Unlimited
            description: None,
        }
    }

    /// Create a flow input (for FSM transitions)
    pub fn flow_in(id: impl Into<String>) -> Self {
        Self::input(id, "In", PortDataType::Event)
            .at(PortPosition::Top)
    }

    /// Create a flow output (for FSM transitions)
    pub fn flow_out(id: impl Into<String>) -> Self {
        Self::output(id, "Out", PortDataType::Event)
            .at(PortPosition::Bottom)
    }

    /// Set position
    pub fn at(mut self, position: PortPosition) -> Self {
        self.position = position;
        self
    }

    /// Set offset
    pub fn offset(mut self, offset: f64) -> Self {
        self.offset = offset.clamp(0.0, 1.0);
        self
    }

    /// Mark as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set max connections
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Add description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Check if this port can connect to another
    pub fn can_connect_to(&self, other: &Port) -> bool {
        // Direction check: output -> input or bidirectional
        let direction_ok = match (&self.direction, &other.direction) {
            (PortDirection::Output, PortDirection::Input) => true,
            (PortDirection::Input, PortDirection::Output) => true,
            (PortDirection::Bidirectional, _) => true,
            (_, PortDirection::Bidirectional) => true,
            _ => false,
        };

        if !direction_ok {
            return false;
        }

        // Data type compatibility
        self.data_type_compatible(&other.data_type)
    }

    /// Check if data types are compatible
    fn data_type_compatible(&self, other: &PortDataType) -> bool {
        if self.data_type == *other {
            return true;
        }

        // Any type matches everything
        if matches!(&self.data_type, PortDataType::Any) || matches!(other, PortDataType::Any) {
            return true;
        }

        // Digital/Boolean are compatible
        if matches!((&self.data_type, other), 
            (PortDataType::Digital, PortDataType::Boolean) |
            (PortDataType::Boolean, PortDataType::Digital)) {
            return true;
        }

        // Integer/Float are compatible
        if matches!((&self.data_type, other),
            (PortDataType::Integer, PortDataType::Float) |
            (PortDataType::Float, PortDataType::Integer)) {
            return true;
        }

        false
    }

    /// Get world position given node bounds
    pub fn world_position(&self, node_x: f64, node_y: f64, node_w: f64, node_h: f64) -> (f64, f64) {
        match self.position {
            PortPosition::Top => (node_x + node_w * self.offset, node_y),
            PortPosition::Bottom => (node_x + node_w * self.offset, node_y + node_h),
            PortPosition::Left => (node_x, node_y + node_h * self.offset),
            PortPosition::Right => (node_x + node_w, node_y + node_h * self.offset),
        }
    }
}

/// Port template for generating default ports for a node type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortTemplate {
    pub ports: Vec<Port>,
}

impl PortTemplate {
    pub fn new() -> Self {
        Self { ports: Vec::new() }
    }

    pub fn with_port(mut self, port: Port) -> Self {
        self.ports.push(port);
        self
    }

    /// Standard FSM state template (flow in/out)
    pub fn fsm_state() -> Self {
        Self::new()
            .with_port(Port::flow_in("in"))
            .with_port(Port::flow_out("out"))
    }

    /// GPIO input template
    pub fn gpio_input() -> Self {
        Self::new()
            .with_port(Port::output("signal", "Signal", PortDataType::Digital))
            .with_port(Port::flow_in("trigger"))
    }

    /// GPIO output template
    pub fn gpio_output() -> Self {
        Self::new()
            .with_port(Port::input("signal", "Signal", PortDataType::Digital))
            .with_port(Port::flow_in("trigger"))
    }

    /// UART template
    pub fn uart() -> Self {
        Self::new()
            .with_port(Port::input("tx_data", "TX", PortDataType::Buffer))
            .with_port(Port::output("rx_data", "RX", PortDataType::Buffer))
            .with_port(Port::flow_in("trigger"))
    }

    /// ADC template
    pub fn adc() -> Self {
        Self::new()
            .with_port(Port::output("value", "Value", PortDataType::Integer))
            .with_port(Port::flow_in("trigger"))
            .with_port(Port::flow_out("complete"))
    }

    /// Timer template
    pub fn timer() -> Self {
        Self::new()
            .with_port(Port::input("period", "Period", PortDataType::Integer))
            .with_port(Port::flow_in("start"))
            .with_port(Port::flow_in("stop"))
            .with_port(Port::flow_out("timeout"))
    }

    /// Task template
    pub fn rtos_task() -> Self {
        Self::new()
            .with_port(Port::flow_in("signal"))
            .with_port(Port::flow_out("complete"))
            .with_port(Port::input("priority", "Priority", PortDataType::Integer))
    }
}

impl Default for PortTemplate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_creation() {
        let port = Port::input("p1", "Test", PortDataType::Digital)
            .at(PortPosition::Left)
            .required();

        assert_eq!(port.id, "p1");
        assert!(port.required);
        assert_eq!(port.position, PortPosition::Left);
    }

    #[test]
    fn test_connection_compatibility() {
        let out_port = Port::output("out", "Out", PortDataType::Digital);
        let in_port = Port::input("in", "In", PortDataType::Digital);

        assert!(out_port.can_connect_to(&in_port));
        assert!(!in_port.can_connect_to(&out_port)); // Input can't initiate
    }

    #[test]
    fn test_type_compatibility() {
        let digital = Port::output("d", "D", PortDataType::Digital);
        let boolean = Port::input("b", "B", PortDataType::Boolean);

        assert!(digital.can_connect_to(&boolean));
    }
}
