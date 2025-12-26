//! Enhanced Node Types for Embedded Systems Design
//!
//! Supports 40+ node types across 6 categories:
//! - FSM: State machine design
//! - Hardware: MCU peripherals (GPIO, ADC, PWM, etc.)
//! - Processing: Data transformations
//! - Control: RTOS and interrupts
//! - IO: Sensors and actuators
//! - Data: Variables and buffers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// NODE CATEGORIES
// ============================================================================

/// Categories of nodes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum NodeCategory {
    /// State machine nodes
    Fsm,
    /// Hardware peripheral nodes
    Hardware,
    /// Processing/computation nodes
    Processing,
    /// Control flow and RTOS nodes
    Control,
    /// Input/Output device nodes
    Io,
    /// Data storage nodes
    Data,
}

impl NodeCategory {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Fsm => "State Machine",
            Self::Hardware => "Hardware",
            Self::Processing => "Processing",
            Self::Control => "Control/RTOS",
            Self::Io => "I/O Devices",
            Self::Data => "Data",
        }
    }
    
    pub fn icon(&self) -> &str {
        match self {
            Self::Fsm => "🔀",
            Self::Hardware => "🔧",
            Self::Processing => "⚙️",
            Self::Control => "📋",
            Self::Io => "📡",
            Self::Data => "📦",
        }
    }
    
    pub fn all() -> Vec<Self> {
        vec![Self::Fsm, Self::Hardware, Self::Processing, Self::Control, Self::Io, Self::Data]
    }
}

// ============================================================================
// NODE TYPES
// ============================================================================

/// All available node types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    // === FSM Nodes ===
    /// Standard FSM state
    State,
    /// Initial/starting state
    Initial,
    /// Final/terminal state  
    Final,
    /// Decision/branch point
    Decision,
    /// Junction (merge paths)
    Junction,
    /// History state (shallow/deep)
    History,
    /// Composite/hierarchical state
    Composite,
    
    // === Hardware Nodes ===
    /// General Purpose I/O
    Gpio,
    /// Analog-to-Digital Converter
    Adc,
    /// Digital-to-Analog Converter
    Dac,
    /// PWM Timer Output
    Pwm,
    /// UART Serial Port
    Uart,
    /// SPI Bus
    Spi,
    /// I2C Bus
    I2c,
    /// CAN Bus
    Can,
    /// USB Interface
    Usb,
    /// External Interrupt Pin
    Exti,
    /// DMA Channel
    Dma,
    
    // === Processing Nodes ===
    /// Digital filter (LPF, HPF, etc.)
    Filter,
    /// Math operation
    MathOp,
    /// Comparison
    Compare,
    /// Multiplexer (N:1)
    Mux,
    /// Demultiplexer (1:N)
    Demux,
    /// Time delay
    Delay,
    /// Data buffer
    Buffer,
    /// Lookup table
    Lut,
    /// PID Controller
    Pid,
    /// Signal generator
    SignalGen,
    
    // === Control Nodes ===
    /// Hardware interrupt
    Interrupt,
    /// Timer peripheral
    Timer,
    /// Software event
    Event,
    /// RTOS semaphore
    Semaphore,
    /// RTOS mutex
    Mutex,
    /// RTOS task/thread
    Task,
    /// Critical section
    Critical,
    /// Message queue
    MessageQueue,
    /// Event flags/groups
    EventFlags,
    
    // === I/O Nodes ===
    /// Generic sensor
    Sensor,
    /// Generic actuator
    Actuator,
    /// LED indicator
    Led,
    /// Push button
    Button,
    /// Motor (DC/Stepper/Servo)
    Motor,
    /// LCD/OLED display
    Display,
    /// Rotary encoder
    Encoder,
    /// Buzzer/speaker
    Buzzer,
    /// Relay
    Relay,
    /// Temperature sensor
    TempSensor,
    
    // === Data Nodes ===
    /// Runtime variable
    Variable,
    /// Compile-time constant
    Constant,
    /// Fixed-size array
    Array,
    /// Struct/record
    Struct,
    /// FIFO queue
    Queue,
    /// Ring buffer
    RingBuffer,
    /// Watchpoint (debug)
    Watchpoint,
}

impl NodeType {
    /// Get the category for this node type
    pub fn category(&self) -> NodeCategory {
        match self {
            // FSM
            Self::State | Self::Initial | Self::Final | 
            Self::Decision | Self::Junction | Self::History |
            Self::Composite => NodeCategory::Fsm,
            
            // Hardware
            Self::Gpio | Self::Adc | Self::Dac | Self::Pwm |
            Self::Uart | Self::Spi | Self::I2c | Self::Can |
            Self::Usb | Self::Exti | Self::Dma => NodeCategory::Hardware,
            
            // Processing
            Self::Filter | Self::MathOp | Self::Compare |
            Self::Mux | Self::Demux | Self::Delay | Self::Buffer |
            Self::Lut | Self::Pid | Self::SignalGen => NodeCategory::Processing,
            
            // Control
            Self::Interrupt | Self::Timer | Self::Event |
            Self::Semaphore | Self::Mutex | Self::Task |
            Self::Critical | Self::MessageQueue | Self::EventFlags => NodeCategory::Control,
            
            // I/O
            Self::Sensor | Self::Actuator | Self::Led |
            Self::Button | Self::Motor | Self::Display |
            Self::Encoder | Self::Buzzer | Self::Relay |
            Self::TempSensor => NodeCategory::Io,
            
            // Data
            Self::Variable | Self::Constant | Self::Array |
            Self::Struct | Self::Queue | Self::RingBuffer |
            Self::Watchpoint => NodeCategory::Data,
        }
    }
    
    /// Get display name
    pub fn display_name(&self) -> &str {
        match self {
            Self::State => "State",
            Self::Initial => "Initial State",
            Self::Final => "Final State",
            Self::Decision => "Decision",
            Self::Junction => "Junction",
            Self::History => "History",
            Self::Composite => "Composite State",
            
            Self::Gpio => "GPIO Pin",
            Self::Adc => "ADC Channel",
            Self::Dac => "DAC Output",
            Self::Pwm => "PWM Timer",
            Self::Uart => "UART Port",
            Self::Spi => "SPI Bus",
            Self::I2c => "I2C Bus",
            Self::Can => "CAN Bus",
            Self::Usb => "USB",
            Self::Exti => "External Interrupt",
            Self::Dma => "DMA Channel",
            
            Self::Filter => "Filter",
            Self::MathOp => "Math Operation",
            Self::Compare => "Compare",
            Self::Mux => "Multiplexer",
            Self::Demux => "Demultiplexer",
            Self::Delay => "Delay",
            Self::Buffer => "Buffer",
            Self::Lut => "Lookup Table",
            Self::Pid => "PID Controller",
            Self::SignalGen => "Signal Generator",
            
            Self::Interrupt => "Interrupt",
            Self::Timer => "Timer",
            Self::Event => "Event",
            Self::Semaphore => "Semaphore",
            Self::Mutex => "Mutex",
            Self::Task => "RTOS Task",
            Self::Critical => "Critical Section",
            Self::MessageQueue => "Message Queue",
            Self::EventFlags => "Event Flags",
            
            Self::Sensor => "Sensor",
            Self::Actuator => "Actuator",
            Self::Led => "LED",
            Self::Button => "Button",
            Self::Motor => "Motor",
            Self::Display => "Display",
            Self::Encoder => "Encoder",
            Self::Buzzer => "Buzzer",
            Self::Relay => "Relay",
            Self::TempSensor => "Temperature Sensor",
            
            Self::Variable => "Variable",
            Self::Constant => "Constant",
            Self::Array => "Array",
            Self::Struct => "Struct",
            Self::Queue => "Queue",
            Self::RingBuffer => "Ring Buffer",
            Self::Watchpoint => "Watchpoint",
        }
    }
    
    /// Get icon for this node type
    pub fn icon(&self) -> &str {
        match self {
            Self::State => "●",
            Self::Initial => "◉",
            Self::Final => "◎",
            Self::Decision => "◇",
            Self::Junction => "⊕",
            Self::History => "Ⓗ",
            Self::Composite => "▣",
            
            Self::Gpio => "📍",
            Self::Adc => "📊",
            Self::Dac => "📈",
            Self::Pwm => "〜",
            Self::Uart => "↔",
            Self::Spi => "⇋",
            Self::I2c => "⇆",
            Self::Can => "🚗",
            Self::Usb => "🔌",
            Self::Exti => "⚡",
            Self::Dma => "⟹",
            
            Self::Filter => "▽",
            Self::MathOp => "±",
            Self::Compare => "≷",
            Self::Mux => "⊳",
            Self::Demux => "⊲",
            Self::Delay => "⏱",
            Self::Buffer => "▭",
            Self::Lut => "📋",
            Self::Pid => "⟲",
            Self::SignalGen => "∿",
            
            Self::Interrupt => "⚡",
            Self::Timer => "⏰",
            Self::Event => "🔔",
            Self::Semaphore => "🚦",
            Self::Mutex => "🔒",
            Self::Task => "📌",
            Self::Critical => "⛔",
            Self::MessageQueue => "📬",
            Self::EventFlags => "🚩",
            
            Self::Sensor => "📡",
            Self::Actuator => "⚙️",
            Self::Led => "💡",
            Self::Button => "🔘",
            Self::Motor => "🔄",
            Self::Display => "🖥",
            Self::Encoder => "🎚",
            Self::Buzzer => "🔊",
            Self::Relay => "⚡",
            Self::TempSensor => "🌡",
            
            Self::Variable => "𝑥",
            Self::Constant => "π",
            Self::Array => "[]",
            Self::Struct => "{}",
            Self::Queue => "⤍",
            Self::RingBuffer => "⟳",
            Self::Watchpoint => "👁",
        }
    }
    
    /// Get all node types in a category
    pub fn by_category(category: NodeCategory) -> Vec<Self> {
        match category {
            NodeCategory::Fsm => vec![
                Self::State, Self::Initial, Self::Final,
                Self::Decision, Self::Junction, Self::History, Self::Composite,
            ],
            NodeCategory::Hardware => vec![
                Self::Gpio, Self::Adc, Self::Dac, Self::Pwm,
                Self::Uart, Self::Spi, Self::I2c, Self::Can,
                Self::Usb, Self::Exti, Self::Dma,
            ],
            NodeCategory::Processing => vec![
                Self::Filter, Self::MathOp, Self::Compare,
                Self::Mux, Self::Demux, Self::Delay, Self::Buffer,
                Self::Lut, Self::Pid, Self::SignalGen,
            ],
            NodeCategory::Control => vec![
                Self::Interrupt, Self::Timer, Self::Event,
                Self::Semaphore, Self::Mutex, Self::Task,
                Self::Critical, Self::MessageQueue, Self::EventFlags,
            ],
            NodeCategory::Io => vec![
                Self::Sensor, Self::Actuator, Self::Led,
                Self::Button, Self::Motor, Self::Display,
                Self::Encoder, Self::Buzzer, Self::Relay, Self::TempSensor,
            ],
            NodeCategory::Data => vec![
                Self::Variable, Self::Constant, Self::Array,
                Self::Struct, Self::Queue, Self::RingBuffer, Self::Watchpoint,
            ],
        }
    }
    
    /// Get default port configuration for this node type
    pub fn default_ports(&self) -> NodePorts {
        match self {
            // FSM nodes
            Self::State => NodePorts {
                inputs: vec![PortDef::flow("in")],
                outputs: vec![PortDef::flow("out")],
            },
            Self::Initial => NodePorts {
                inputs: vec![],
                outputs: vec![PortDef::flow("out")],
            },
            Self::Final => NodePorts {
                inputs: vec![PortDef::flow("in")],
                outputs: vec![],
            },
            Self::Decision => NodePorts {
                inputs: vec![PortDef::flow("in"), PortDef::new("condition", PortType::Digital)],
                outputs: vec![PortDef::flow("true"), PortDef::flow("false")],
            },
            Self::Junction => NodePorts {
                inputs: vec![PortDef::flow("in1"), PortDef::flow("in2")],
                outputs: vec![PortDef::flow("out")],
            },
            
            // Hardware nodes
            Self::Gpio => NodePorts {
                inputs: vec![PortDef::new("write", PortType::Digital)],
                outputs: vec![PortDef::new("read", PortType::Digital)],
            },
            Self::Adc => NodePorts {
                inputs: vec![PortDef::new("trigger", PortType::Event)],
                outputs: vec![PortDef::new("value", PortType::Analog), PortDef::new("ready", PortType::Event)],
            },
            Self::Pwm => NodePorts {
                inputs: vec![PortDef::new("duty", PortType::Analog), PortDef::new("enable", PortType::Digital)],
                outputs: vec![PortDef::new("output", PortType::Digital)],
            },
            Self::Uart => NodePorts {
                inputs: vec![PortDef::new("tx_data", PortType::Data)],
                outputs: vec![PortDef::new("rx_data", PortType::Data), PortDef::new("rx_ready", PortType::Event)],
            },
            
            // Control nodes
            Self::Timer => NodePorts {
                inputs: vec![PortDef::new("start", PortType::Event), PortDef::new("stop", PortType::Event)],
                outputs: vec![PortDef::new("timeout", PortType::Event), PortDef::new("counter", PortType::Integer)],
            },
            Self::Task => NodePorts {
                inputs: vec![PortDef::flow("entry")],
                outputs: vec![PortDef::new("signal", PortType::Event)],
            },
            Self::Semaphore => NodePorts {
                inputs: vec![PortDef::new("give", PortType::Event), PortDef::new("take", PortType::Event)],
                outputs: vec![PortDef::new("acquired", PortType::Event)],
            },
            
            // Default for others
            _ => NodePorts {
                inputs: vec![PortDef::flow("in")],
                outputs: vec![PortDef::flow("out")],
            },
        }
    }
}

impl Default for NodeType {
    fn default() -> Self {
        Self::State
    }
}

// ============================================================================
// PORT SYSTEM
// ============================================================================

/// Port data types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PortType {
    /// Execution flow (FSM transitions)
    Flow,
    /// Boolean/digital signal
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
    /// Multi-signal bus
    Bus { width: u8 },
}

impl PortType {
    /// Check if two port types are compatible for connection
    pub fn is_compatible(&self, other: &Self) -> bool {
        match (self, other) {
            // Same type always compatible
            (a, b) if a == b => true,
            // Flow can connect to flow
            (Self::Flow, Self::Flow) => true,
            // Numeric types are somewhat compatible
            (Self::Analog, Self::Float) | (Self::Float, Self::Analog) => true,
            (Self::Digital, Self::Integer) | (Self::Integer, Self::Digital) => true,
            // Event can trigger flow
            (Self::Event, Self::Flow) | (Self::Flow, Self::Event) => true,
            _ => false,
        }
    }
    
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
        }
    }
}

/// Port direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PortDirection {
    Input,
    Output,
    Bidirectional,
}

/// Port definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDef {
    pub name: String,
    pub port_type: PortType,
    pub direction: PortDirection,
    pub required: bool,
}

impl PortDef {
    pub fn new(name: &str, port_type: PortType) -> Self {
        Self {
            name: name.to_string(),
            port_type,
            direction: PortDirection::Input, // Will be set by context
            required: false,
        }
    }
    
    pub fn flow(name: &str) -> Self {
        Self {
            name: name.to_string(),
            port_type: PortType::Flow,
            direction: PortDirection::Input,
            required: true,
        }
    }
    
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

/// Collection of ports for a node
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodePorts {
    pub inputs: Vec<PortDef>,
    pub outputs: Vec<PortDef>,
}

// ============================================================================
// NODE PROPERTIES
// ============================================================================

/// Property value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Enum { value: String, options: Vec<String> },
    Pin { port: String, pin: u8 },
}

/// Node property definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDef {
    pub key: String,
    pub label: String,
    pub value: PropertyValue,
    pub description: Option<String>,
}

/// Get default properties for a node type
pub fn default_properties(node_type: &NodeType) -> Vec<PropertyDef> {
    match node_type {
        NodeType::Gpio => vec![
            PropertyDef {
                key: "port".to_string(),
                label: "Port".to_string(),
                value: PropertyValue::Enum {
                    value: "A".to_string(),
                    options: vec!["A", "B", "C", "D", "E", "F"].into_iter().map(String::from).collect(),
                },
                description: Some("GPIO port".to_string()),
            },
            PropertyDef {
                key: "pin".to_string(),
                label: "Pin".to_string(),
                value: PropertyValue::Integer(0),
                description: Some("Pin number (0-15)".to_string()),
            },
            PropertyDef {
                key: "mode".to_string(),
                label: "Mode".to_string(),
                value: PropertyValue::Enum {
                    value: "output".to_string(),
                    options: vec!["input", "output", "alternate", "analog"].into_iter().map(String::from).collect(),
                },
                description: Some("Pin mode".to_string()),
            },
            PropertyDef {
                key: "pull".to_string(),
                label: "Pull".to_string(),
                value: PropertyValue::Enum {
                    value: "none".to_string(),
                    options: vec!["none", "up", "down"].into_iter().map(String::from).collect(),
                },
                description: Some("Internal pull resistor".to_string()),
            },
        ],
        NodeType::Pwm => vec![
            PropertyDef {
                key: "timer".to_string(),
                label: "Timer".to_string(),
                value: PropertyValue::Enum {
                    value: "TIM1".to_string(),
                    options: vec!["TIM1", "TIM2", "TIM3", "TIM4"].into_iter().map(String::from).collect(),
                },
                description: Some("Timer peripheral".to_string()),
            },
            PropertyDef {
                key: "channel".to_string(),
                label: "Channel".to_string(),
                value: PropertyValue::Integer(1),
                description: Some("Timer channel (1-4)".to_string()),
            },
            PropertyDef {
                key: "frequency".to_string(),
                label: "Frequency (Hz)".to_string(),
                value: PropertyValue::Integer(1000),
                description: Some("PWM frequency".to_string()),
            },
        ],
        NodeType::Task => vec![
            PropertyDef {
                key: "name".to_string(),
                label: "Task Name".to_string(),
                value: PropertyValue::String("task1".to_string()),
                description: Some("RTOS task name".to_string()),
            },
            PropertyDef {
                key: "priority".to_string(),
                label: "Priority".to_string(),
                value: PropertyValue::Integer(1),
                description: Some("Task priority".to_string()),
            },
            PropertyDef {
                key: "stack_size".to_string(),
                label: "Stack Size".to_string(),
                value: PropertyValue::Integer(256),
                description: Some("Stack size in words".to_string()),
            },
        ],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_node_categories() {
        assert_eq!(NodeType::State.category(), NodeCategory::Fsm);
        assert_eq!(NodeType::Gpio.category(), NodeCategory::Hardware);
        assert_eq!(NodeType::Task.category(), NodeCategory::Control);
    }
    
    #[test]
    fn test_port_compatibility() {
        assert!(PortType::Flow.is_compatible(&PortType::Flow));
        assert!(PortType::Analog.is_compatible(&PortType::Float));
        assert!(!PortType::Digital.is_compatible(&PortType::Data));
    }
    
    #[test]
    fn test_by_category() {
        let hw_nodes = NodeType::by_category(NodeCategory::Hardware);
        assert!(hw_nodes.contains(&NodeType::Gpio));
        assert!(hw_nodes.contains(&NodeType::Uart));
    }
}
