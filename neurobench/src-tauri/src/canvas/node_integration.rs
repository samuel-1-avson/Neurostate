//! Node Engine Integration
//!
//! Bridges the shared type system with the canvas engine, enabling:
//! - Node creation with proper ports and properties
//! - Port-aware connection validation
//! - Type-safe node operations

use std::collections::HashMap;
use crate::shared::{
    NodeType, NodeCategory, NodeTypeInfo,
    PortType, PortDef, NodePorts, PortDirection,
    PropertyValue, PropertyDef, default_properties,
    default_ports,
};
use super::types::{CanvasNode, CanvasEdge, CanvasError};

// ============================================================================
// EXTENDED CANVAS NODE - With Ports and Properties
// ============================================================================

/// Extended node with full port and property support
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnhancedNode {
    /// Base canvas node
    #[serde(flatten)]
    pub base: CanvasNode,
    /// Port definitions from shared types
    pub ports: NodePorts,
    /// Property values from shared types
    pub properties: HashMap<String, PropertyValue>,
    /// Shared NodeType (for codegen integration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_type: Option<crate::shared::NodeType>,
}

impl EnhancedNode {
    /// Create from canvas node with default ports/properties
    pub fn from_canvas_node(node: CanvasNode) -> Self {
        // Try to map canvas NodeType to shared NodeType
        let shared_type = map_canvas_to_shared_type(&node.node_type);
        
        let (ports, properties) = if let Some(ref st) = shared_type {
            (
                default_ports(st),
                default_properties(st)
                    .into_iter()
                    .map(|p| (p.key, p.value))
                    .collect(),
            )
        } else {
            (NodePorts::default(), HashMap::new())
        };
        
        Self {
            base: node,
            ports,
            properties,
            shared_type,
        }
    }
}

// ============================================================================
// TYPE MAPPING - Canvas Types to Shared Types
// ============================================================================

/// Map canvas NodeType to shared NodeType
pub fn map_canvas_to_shared_type(canvas_type: &super::types::NodeType) -> Option<NodeType> {
    match canvas_type {
        // FSM
        super::types::NodeType::Initial => Some(NodeType::Initial),
        super::types::NodeType::State => Some(NodeType::State),
        super::types::NodeType::Final => Some(NodeType::Final),
        super::types::NodeType::Choice | super::types::NodeType::Decision => Some(NodeType::Decision),
        super::types::NodeType::Junction => Some(NodeType::Junction),
        super::types::NodeType::Fork => Some(NodeType::Fork),
        super::types::NodeType::Join => Some(NodeType::Join),
        super::types::NodeType::History => Some(NodeType::History),
        super::types::NodeType::DeepHistory => Some(NodeType::DeepHistory),
        super::types::NodeType::Composite => Some(NodeType::Composite),
        
        // Hardware
        super::types::NodeType::Gpio => Some(NodeType::Gpio),
        super::types::NodeType::DigitalInput => Some(NodeType::DigitalInput),
        super::types::NodeType::DigitalOutput => Some(NodeType::DigitalOutput),
        super::types::NodeType::Adc => Some(NodeType::Adc),
        super::types::NodeType::Dac => Some(NodeType::Dac),
        super::types::NodeType::Pwm => Some(NodeType::Pwm),
        super::types::NodeType::Exti => Some(NodeType::Exti),
        super::types::NodeType::Dma => Some(NodeType::Dma),
        super::types::NodeType::InputCapture => Some(NodeType::InputCapture),
        super::types::NodeType::OutputCompare => Some(NodeType::OutputCompare),
        super::types::NodeType::Comparator => Some(NodeType::Comparator),
        super::types::NodeType::OpAmp => Some(NodeType::OpAmp),
        
        // Communication
        super::types::NodeType::Uart => Some(NodeType::Uart),
        super::types::NodeType::Spi => Some(NodeType::Spi),
        super::types::NodeType::I2c => Some(NodeType::I2c),
        super::types::NodeType::Can => Some(NodeType::Can),
        super::types::NodeType::Usb => Some(NodeType::Usb),
        super::types::NodeType::Ethernet => Some(NodeType::Ethernet),
        super::types::NodeType::Rs485 => Some(NodeType::Rs485),
        super::types::NodeType::Modbus => Some(NodeType::Modbus),
        
        // Timer
        super::types::NodeType::Timer => Some(NodeType::Timer),
        super::types::NodeType::Delay => Some(NodeType::Delay),
        super::types::NodeType::Rtc => Some(NodeType::Rtc),
        super::types::NodeType::Watchdog => Some(NodeType::Watchdog),
        
        // RTOS
        super::types::NodeType::Task => Some(NodeType::Task),
        super::types::NodeType::Semaphore => Some(NodeType::Semaphore),
        super::types::NodeType::Mutex => Some(NodeType::Mutex),
        super::types::NodeType::Queue => Some(NodeType::Queue),
        super::types::NodeType::EventGroup => Some(NodeType::EventFlags),
        super::types::NodeType::SwTimer => Some(NodeType::SwTimer),
        super::types::NodeType::MemPool => Some(NodeType::MemPool),
        super::types::NodeType::Event => Some(NodeType::Event),
        super::types::NodeType::Critical => Some(NodeType::Critical),
        super::types::NodeType::MessageQueue => Some(NodeType::MessageQueue),
        super::types::NodeType::EventFlags => Some(NodeType::EventFlags),
        
        // Processing
        super::types::NodeType::Filter => Some(NodeType::Filter),
        super::types::NodeType::MathOp => Some(NodeType::MathOp),
        super::types::NodeType::Compare => Some(NodeType::Compare),
        super::types::NodeType::Mux => Some(NodeType::Mux),
        super::types::NodeType::Demux => Some(NodeType::Demux),
        super::types::NodeType::Buffer => Some(NodeType::Buffer),
        super::types::NodeType::Lut => Some(NodeType::Lut),
        super::types::NodeType::Pid => Some(NodeType::Pid),
        super::types::NodeType::SignalGen => Some(NodeType::SignalGen),
        
        // Motor
        super::types::NodeType::MotorDc => Some(NodeType::MotorDc),
        super::types::NodeType::MotorStepper => Some(NodeType::MotorStepper),
        super::types::NodeType::MotorServo => Some(NodeType::MotorServo),
        super::types::NodeType::HBridge => Some(NodeType::HBridge),
        super::types::NodeType::Encoder => Some(NodeType::Encoder),
        super::types::NodeType::Motor => Some(NodeType::Motor),
        
        // System
        super::types::NodeType::PowerMode | super::types::NodeType::PowerMgmt => Some(NodeType::PowerMode),
        super::types::NodeType::Clock | super::types::NodeType::ClockConfig => Some(NodeType::Clock),
        super::types::NodeType::Flash => Some(NodeType::Flash),
        super::types::NodeType::Reset => Some(NodeType::Reset),
        super::types::NodeType::DebugLog => Some(NodeType::DebugLog),
        
        // Sensor
        super::types::NodeType::Sensor => Some(NodeType::Sensor),
        super::types::NodeType::SensorTemp | super::types::NodeType::TempSensor => Some(NodeType::TempSensor),
        super::types::NodeType::SensorImu => Some(NodeType::Imu),
        super::types::NodeType::SensorProximity => Some(NodeType::ProximitySensor),
        
        // I/O
        super::types::NodeType::Led => Some(NodeType::Led),
        super::types::NodeType::Button => Some(NodeType::Button),
        super::types::NodeType::Relay => Some(NodeType::Relay),
        super::types::NodeType::Actuator => Some(NodeType::Actuator),
        super::types::NodeType::Display => Some(NodeType::Display),
        super::types::NodeType::Buzzer => Some(NodeType::Buzzer),
        
        // Data
        super::types::NodeType::Variable => Some(NodeType::Variable),
        super::types::NodeType::Constant => Some(NodeType::Constant),
        super::types::NodeType::Array => Some(NodeType::Array),
        super::types::NodeType::Struct => Some(NodeType::Struct),
        super::types::NodeType::RingBuffer => Some(NodeType::RingBuffer),
        super::types::NodeType::Watchpoint => Some(NodeType::Watchpoint),
        
        // Wireless
        super::types::NodeType::Wifi => Some(NodeType::Wifi),
        super::types::NodeType::Bluetooth | super::types::NodeType::Ble => Some(NodeType::Bluetooth),
        super::types::NodeType::LoRa | super::types::NodeType::Lora => Some(NodeType::Lora),
        super::types::NodeType::Zigbee => Some(NodeType::Zigbee),
        
        // Protocol
        super::types::NodeType::Mqtt => Some(NodeType::Mqtt),
        super::types::NodeType::Http => Some(NodeType::Http),
        super::types::NodeType::WebSocket => Some(NodeType::WebSocket),
        super::types::NodeType::CanOpen => Some(NodeType::CanOpen),
        
        // Legacy
        super::types::NodeType::Input => Some(NodeType::Input),
        super::types::NodeType::Output => Some(NodeType::Output),
        super::types::NodeType::Process => Some(NodeType::Process),
        super::types::NodeType::Error => Some(NodeType::Error),
        super::types::NodeType::Hardware => Some(NodeType::Hardware),
        super::types::NodeType::Interrupt => Some(NodeType::Interrupt),
        
        // Custom
        super::types::NodeType::Custom { category, icon } => {
            Some(NodeType::Custom {
                category: category.clone(),
                icon: icon.clone(),
            })
        }
    }
}

// ============================================================================
// NODE FACTORY
// ============================================================================

/// Factory for creating nodes with proper ports and properties
pub struct NodeFactory;

impl NodeFactory {
    /// Create a new node from a shared NodeType
    pub fn create_node(
        node_type: NodeType,
        x: f64,
        y: f64,
        label: Option<String>,
    ) -> (CanvasNode, NodePorts, HashMap<String, PropertyValue>) {
        let info = NodeTypeInfo::from_type(&node_type);
        let ports = default_ports(&node_type);
        let properties: HashMap<String, PropertyValue> = default_properties(&node_type)
            .into_iter()
            .map(|p| (p.key, p.value))
            .collect();
        
        // Map shared NodeType to canvas NodeType
        let canvas_type = map_shared_to_canvas_type(&node_type);
        
        let node = CanvasNode {
            id: generate_node_id(),
            label: label.unwrap_or_else(|| info.name.clone()),
            node_type: canvas_type,
            x,
            y,
            width: info.default_width,
            height: info.default_height,
            entry_action: None,
            exit_action: None,
            description: None,
        };
        
        (node, ports, properties)
    }
    
    /// Get node type info for UI palette
    pub fn get_palette() -> Vec<(NodeCategory, Vec<NodeTypeInfo>)> {
        NodeCategory::all()
            .into_iter()
            .map(|cat| {
                let types = NodeType::by_category(cat)
                    .into_iter()
                    .map(|t| NodeTypeInfo::from_type(&t))
                    .collect();
                (cat, types)
            })
            .collect()
    }
}

/// Map shared NodeType to canvas NodeType
fn map_shared_to_canvas_type(shared: &NodeType) -> super::types::NodeType {
    match shared {
        // FSM
        NodeType::Initial => super::types::NodeType::Initial,
        NodeType::State => super::types::NodeType::State,
        NodeType::Final => super::types::NodeType::Final,
        NodeType::Decision => super::types::NodeType::Decision,
        NodeType::Junction => super::types::NodeType::Choice,
        NodeType::Fork => super::types::NodeType::Fork,
        NodeType::Join => super::types::NodeType::Join,
        NodeType::History => super::types::NodeType::History,
        NodeType::DeepHistory => super::types::NodeType::DeepHistory,
        NodeType::Composite => super::types::NodeType::State, // Fallback
        
        // Hardware
        NodeType::Gpio => super::types::NodeType::Gpio,
        NodeType::DigitalInput => super::types::NodeType::DigitalInput,
        NodeType::DigitalOutput => super::types::NodeType::DigitalOutput,
        NodeType::Adc => super::types::NodeType::Adc,
        NodeType::Dac => super::types::NodeType::Dac,
        NodeType::Pwm => super::types::NodeType::Pwm,
        NodeType::Exti => super::types::NodeType::Exti,
        NodeType::Dma => super::types::NodeType::Dma,
        NodeType::InputCapture => super::types::NodeType::InputCapture,
        NodeType::OutputCompare => super::types::NodeType::OutputCompare,
        NodeType::Comparator => super::types::NodeType::Comparator,
        NodeType::OpAmp => super::types::NodeType::OpAmp,
        
        // Communication
        NodeType::Uart => super::types::NodeType::Uart,
        NodeType::Spi => super::types::NodeType::Spi,
        NodeType::I2c => super::types::NodeType::I2c,
        NodeType::Can => super::types::NodeType::Can,
        NodeType::Usb => super::types::NodeType::Usb,
        NodeType::Ethernet => super::types::NodeType::Ethernet,
        NodeType::Rs485 => super::types::NodeType::Rs485,
        
        // Timer
        NodeType::Timer => super::types::NodeType::Timer,
        NodeType::Delay => super::types::NodeType::Delay,
        NodeType::Rtc => super::types::NodeType::Rtc,
        
        // RTOS
        NodeType::Task => super::types::NodeType::Task,
        NodeType::Semaphore => super::types::NodeType::Semaphore,
        NodeType::Mutex => super::types::NodeType::Mutex,
        NodeType::MessageQueue | NodeType::Queue => super::types::NodeType::Queue,
        NodeType::EventFlags => super::types::NodeType::EventGroup,
        NodeType::SwTimer => super::types::NodeType::SwTimer,
        NodeType::MemPool => super::types::NodeType::MemPool,
        NodeType::Interrupt => super::types::NodeType::Interrupt,
        NodeType::Event => super::types::NodeType::Process, // Fallback
        NodeType::Critical => super::types::NodeType::Mutex, // Fallback
        
        // Motor
        NodeType::Motor | NodeType::MotorDc => super::types::NodeType::MotorDc,
        NodeType::MotorStepper => super::types::NodeType::MotorStepper,
        NodeType::MotorServo => super::types::NodeType::MotorServo,
        NodeType::HBridge => super::types::NodeType::HBridge,
        NodeType::Encoder => super::types::NodeType::Encoder,
        
        // Sensor
        NodeType::Sensor => super::types::NodeType::Sensor,
        NodeType::TempSensor => super::types::NodeType::SensorTemp,
        NodeType::Imu => super::types::NodeType::SensorImu,
        NodeType::ProximitySensor => super::types::NodeType::SensorProximity,
        NodeType::Actuator => super::types::NodeType::Output, // Fallback
        
        // Wireless
        NodeType::Wifi => super::types::NodeType::Wifi,
        NodeType::Ble | NodeType::Bluetooth => super::types::NodeType::Bluetooth,
        NodeType::Lora => super::types::NodeType::LoRa,
        NodeType::Zigbee => super::types::NodeType::Zigbee,
        
        // I/O
        NodeType::Led => super::types::NodeType::Led,
        NodeType::Button => super::types::NodeType::Button,
        NodeType::Relay => super::types::NodeType::Relay,
        NodeType::Display => super::types::NodeType::Output, // Fallback
        NodeType::Buzzer => super::types::NodeType::Output, // Fallback
        
        // Data
        NodeType::Variable | NodeType::Constant | NodeType::Array | 
        NodeType::Struct | NodeType::RingBuffer | NodeType::Watchpoint => {
            super::types::NodeType::Process // Fallback
        }
        
        // Protocol
        NodeType::Mqtt => super::types::NodeType::Ethernet, // Fallback
        NodeType::Http => super::types::NodeType::Ethernet, // Fallback
        NodeType::WebSocket => super::types::NodeType::Ethernet, // Fallback
        NodeType::Modbus => super::types::NodeType::Modbus,
        NodeType::CanOpen => super::types::NodeType::Can, // Fallback
        
        // System
        NodeType::Watchdog => super::types::NodeType::Watchdog,
        NodeType::PowerMode => super::types::NodeType::PowerMode,
        NodeType::Clock => super::types::NodeType::Clock,
        NodeType::DebugLog => super::types::NodeType::Process, // Fallback
        NodeType::Flash => super::types::NodeType::Flash,
        NodeType::Reset => super::types::NodeType::Reset,
        
        // Legacy/Processing
        NodeType::Filter | NodeType::MathOp | NodeType::Compare |
        NodeType::Mux | NodeType::Demux | NodeType::Buffer | 
        NodeType::Lut | NodeType::Pid | NodeType::SignalGen => {
            super::types::NodeType::Process
        }
        
        NodeType::Input => super::types::NodeType::Input,
        NodeType::Output => super::types::NodeType::Output,
        NodeType::Process => super::types::NodeType::Process,
        NodeType::Error => super::types::NodeType::Error,
        NodeType::Hardware => super::types::NodeType::Hardware,
        
        // Custom
        NodeType::Custom { category, icon } => super::types::NodeType::Custom {
            category: category.clone(),
            icon: icon.clone(),
        },
    }
}

/// Generate a unique node ID
fn generate_node_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("node_{}", timestamp)
}

// ============================================================================
// PORT VALIDATION
// ============================================================================

/// Validate that two ports can be connected
pub fn can_connect_ports(source_port: &PortDef, target_port: &PortDef) -> Result<(), CanvasError> {
    // Check direction
    if source_port.direction == PortDirection::Input {
        return Err(CanvasError::InvalidOperation(
            "Source port must be output or bidirectional".to_string(),
        ));
    }
    if target_port.direction == PortDirection::Output {
        return Err(CanvasError::InvalidOperation(
            "Target port must be input or bidirectional".to_string(),
        ));
    }
    
    // Check type compatibility
    if !source_port.port_type.is_compatible(&target_port.port_type) {
        return Err(CanvasError::InvalidOperation(format!(
            "Incompatible port types: {} -> {}",
            source_port.port_type.display_name(),
            target_port.port_type.display_name(),
        )));
    }
    
    Ok(())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_node_factory_create() {
        let (node, ports, props) = NodeFactory::create_node(
            NodeType::Gpio,
            100.0,
            200.0,
            None,
        );
        
        assert!(node.id.starts_with("node_"));
        assert_eq!(node.label, "GPIO Pin");
        assert!(!ports.inputs.is_empty() || !ports.outputs.is_empty());
        assert!(!props.is_empty());
    }
    
    #[test]
    fn test_type_mapping() {
        let shared = NodeType::State;
        let canvas = map_shared_to_canvas_type(&shared);
        assert!(matches!(canvas, super::super::types::NodeType::State));
        
        let back = map_canvas_to_shared_type(&canvas);
        assert_eq!(back, Some(NodeType::State));
    }
    
    #[test]
    fn test_port_validation() {
        use crate::shared::port_types::{PortDef, PortType, PortDirection};
        
        let output = PortDef::output("out", PortType::Digital);
        let input = PortDef::input("in", PortType::Digital);
        
        assert!(can_connect_ports(&output, &input).is_ok());
        
        // Wrong direction
        assert!(can_connect_ports(&input, &output).is_err());
        
        // Incompatible types
        let analog_output = PortDef::output("out", PortType::Data);
        assert!(can_connect_ports(&analog_output, &input).is_err());
    }
}
