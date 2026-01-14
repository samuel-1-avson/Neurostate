//! Unified Node Types
//!
//! Single source of truth for all node types across the application.
//! Merges node types from both canvas and node engine with superset of all types.

use serde::{Deserialize, Serialize};

// ============================================================================
// NODE CATEGORIES
// ============================================================================

/// Categories of nodes - unified across all engines
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum NodeCategory {
    /// State machine nodes
    Fsm,
    /// Hardware peripheral nodes (GPIO, timers, etc.)
    Hardware,
    /// Processing/computation nodes (filters, math, etc.)
    Processing,
    /// Control flow and RTOS nodes
    Control,
    /// Input/Output device nodes (sensors, actuators)
    Io,
    /// Data storage nodes (variables, buffers)
    Data,
    /// Communication interfaces (UART, SPI, I2C, etc.)
    Communication,
    /// Wireless nodes (WiFi, BLE, LoRa)
    Wireless,
    /// Protocol nodes (MQTT, HTTP, Modbus)
    Protocol,
    /// System nodes (power, clock, watchdog)
    System,
    /// Custom user-defined nodes
    Custom,
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
            Self::Communication => "Communication",
            Self::Wireless => "Wireless",
            Self::Protocol => "Protocols",
            Self::System => "System",
            Self::Custom => "Custom",
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
            Self::Communication => "↔️",
            Self::Wireless => "📶",
            Self::Protocol => "🌐",
            Self::System => "⚡",
            Self::Custom => "🎨",
        }
    }
    
    pub fn all() -> Vec<Self> {
        vec![
            Self::Fsm, Self::Hardware, Self::Processing, Self::Control, 
            Self::Io, Self::Data, Self::Communication, Self::Wireless,
            Self::Protocol, Self::System,
        ]
    }
}

// ============================================================================
// NODE TYPES - Unified Superset
// ============================================================================

/// All available node types - unified from both canvas and node engines
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    // === FSM Core Nodes ===
    /// Initial/starting state
    Initial,
    /// Regular state
    State,
    /// Final/terminal state
    Final,
    /// Decision/choice point
    Decision,
    /// Junction (merge paths)
    Junction,
    /// Fork (parallel split)
    Fork,
    /// Join (parallel merge)
    Join,
    /// History state (shallow)
    History,
    /// Deep history state
    DeepHistory,
    /// Composite/hierarchical state
    Composite,
    
    // === Hardware Peripheral Nodes ===
    /// General Purpose I/O
    Gpio,
    /// Digital input
    DigitalInput,
    /// Digital output
    DigitalOutput,
    /// Analog-to-Digital Converter
    Adc,
    /// Digital-to-Analog Converter
    Dac,
    /// PWM Timer Output
    Pwm,
    /// External Interrupt Pin
    Exti,
    /// DMA Channel
    Dma,
    /// Input capture
    InputCapture,
    /// Output compare
    OutputCompare,
    /// Analog comparator
    Comparator,
    /// Op-amp configuration
    OpAmp,
    
    // === Communication Interfaces ===
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
    /// Ethernet
    Ethernet,
    /// RS485
    Rs485,
    
    // === Timer Nodes ===
    /// Hardware timer
    Timer,
    /// Software delay
    Delay,
    /// Real-time clock
    Rtc,
    
    // === Control/RTOS Nodes ===
    /// Hardware interrupt
    Interrupt,
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
    /// Software timer (RTOS)
    SwTimer,
    /// Memory pool
    MemPool,
    
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
    /// Data buffer
    Buffer,
    /// Lookup table
    Lut,
    /// PID Controller
    Pid,
    /// Signal generator
    SignalGen,
    
    // === I/O Device Nodes ===
    /// Generic sensor
    Sensor,
    /// Generic actuator
    Actuator,
    /// LED indicator
    Led,
    /// Push button
    Button,
    /// Motor (generic)
    Motor,
    /// DC motor
    MotorDc,
    /// Stepper motor
    MotorStepper,
    /// Servo motor
    MotorServo,
    /// H-bridge driver
    HBridge,
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
    /// Proximity sensor
    ProximitySensor,
    /// IMU/Accelerometer
    Imu,
    
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
    
    // === Wireless Nodes ===
    /// WiFi station/AP
    Wifi,
    /// Bluetooth Low Energy
    Ble,
    /// LoRa/LoRaWAN
    Lora,
    /// Zigbee/Thread
    Zigbee,
    /// Classic Bluetooth
    Bluetooth,
    
    // === Protocol Nodes ===
    /// MQTT client/broker
    Mqtt,
    /// HTTP client/server
    Http,
    /// WebSocket connection
    WebSocket,
    /// Modbus RTU/TCP
    Modbus,
    /// CANopen protocol
    CanOpen,
    
    // === System Nodes ===
    /// Watchdog timer
    Watchdog,
    /// Power management
    PowerMode,
    /// Clock configuration
    Clock,
    /// Debug/logging
    DebugLog,
    /// Flash memory operation
    Flash,
    /// Reset control
    Reset,
    
    // === Legacy/General (backward compatibility) ===
    /// Generic input (legacy)
    Input,
    /// Generic output (legacy)
    Output,
    /// Generic process (legacy)
    Process,
    /// Error state
    Error,
    /// Generic hardware (legacy)
    Hardware,
    
    /// Custom node type with user-defined category
    Custom {
        category: String,
        icon: Option<String>,
    },
}

impl Default for NodeType {
    fn default() -> Self {
        Self::State
    }
}

impl NodeType {
    /// Get the category for this node type
    pub fn category(&self) -> NodeCategory {
        match self {
            // FSM
            Self::Initial | Self::State | Self::Final | Self::Decision | 
            Self::Junction | Self::Fork | Self::Join | Self::History | 
            Self::DeepHistory | Self::Composite => NodeCategory::Fsm,
            
            // Hardware
            Self::Gpio | Self::DigitalInput | Self::DigitalOutput | 
            Self::Adc | Self::Dac | Self::Pwm | Self::Exti | Self::Dma |
            Self::InputCapture | Self::OutputCompare | Self::Comparator | 
            Self::OpAmp => NodeCategory::Hardware,
            
            // Communication
            Self::Uart | Self::Spi | Self::I2c | Self::Can | 
            Self::Usb | Self::Ethernet | Self::Rs485 => NodeCategory::Communication,
            
            // Timers (part of Hardware)
            Self::Timer | Self::Delay | Self::Rtc => NodeCategory::Hardware,
            
            // Control/RTOS
            Self::Interrupt | Self::Event | Self::Semaphore | Self::Mutex | 
            Self::Task | Self::Critical | Self::MessageQueue | Self::EventFlags |
            Self::SwTimer | Self::MemPool => NodeCategory::Control,
            
            // Processing
            Self::Filter | Self::MathOp | Self::Compare | Self::Mux | 
            Self::Demux | Self::Buffer | Self::Lut | Self::Pid | 
            Self::SignalGen => NodeCategory::Processing,
            
            // I/O
            Self::Sensor | Self::Actuator | Self::Led | Self::Button | 
            Self::Motor | Self::MotorDc | Self::MotorStepper | Self::MotorServo |
            Self::HBridge | Self::Display | Self::Encoder | Self::Buzzer | 
            Self::Relay | Self::TempSensor | Self::ProximitySensor | 
            Self::Imu => NodeCategory::Io,
            
            // Data
            Self::Variable | Self::Constant | Self::Array | Self::Struct | 
            Self::Queue | Self::RingBuffer | Self::Watchpoint => NodeCategory::Data,
            
            // Wireless
            Self::Wifi | Self::Ble | Self::Lora | Self::Zigbee | 
            Self::Bluetooth => NodeCategory::Wireless,
            
            // Protocols
            Self::Mqtt | Self::Http | Self::WebSocket | Self::Modbus | 
            Self::CanOpen => NodeCategory::Protocol,
            
            // System
            Self::Watchdog | Self::PowerMode | Self::Clock | Self::DebugLog |
            Self::Flash | Self::Reset => NodeCategory::System,
            
            // Legacy/General
            Self::Input | Self::Output | Self::Process | Self::Error | 
            Self::Hardware => NodeCategory::Io,
            
            Self::Custom { .. } => NodeCategory::Custom,
        }
    }
    
    /// Get display name
    pub fn display_name(&self) -> &str {
        match self {
            // FSM
            Self::Initial => "Initial State",
            Self::State => "State",
            Self::Final => "Final State",
            Self::Decision => "Decision",
            Self::Junction => "Junction",
            Self::Fork => "Fork",
            Self::Join => "Join",
            Self::History => "History",
            Self::DeepHistory => "Deep History",
            Self::Composite => "Composite State",
            
            // Hardware
            Self::Gpio => "GPIO Pin",
            Self::DigitalInput => "Digital Input",
            Self::DigitalOutput => "Digital Output",
            Self::Adc => "ADC Channel",
            Self::Dac => "DAC Output",
            Self::Pwm => "PWM Timer",
            Self::Exti => "External Interrupt",
            Self::Dma => "DMA Channel",
            Self::InputCapture => "Input Capture",
            Self::OutputCompare => "Output Compare",
            Self::Comparator => "Comparator",
            Self::OpAmp => "Op-Amp",
            
            // Communication
            Self::Uart => "UART Port",
            Self::Spi => "SPI Bus",
            Self::I2c => "I2C Bus",
            Self::Can => "CAN Bus",
            Self::Usb => "USB",
            Self::Ethernet => "Ethernet",
            Self::Rs485 => "RS485",
            
            // Timers
            Self::Timer => "Timer",
            Self::Delay => "Delay",
            Self::Rtc => "Real-Time Clock",
            
            // Control
            Self::Interrupt => "Interrupt",
            Self::Event => "Event",
            Self::Semaphore => "Semaphore",
            Self::Mutex => "Mutex",
            Self::Task => "RTOS Task",
            Self::Critical => "Critical Section",
            Self::MessageQueue => "Message Queue",
            Self::EventFlags => "Event Flags",
            Self::SwTimer => "Software Timer",
            Self::MemPool => "Memory Pool",
            
            // Processing
            Self::Filter => "Filter",
            Self::MathOp => "Math Operation",
            Self::Compare => "Compare",
            Self::Mux => "Multiplexer",
            Self::Demux => "Demultiplexer",
            Self::Buffer => "Buffer",
            Self::Lut => "Lookup Table",
            Self::Pid => "PID Controller",
            Self::SignalGen => "Signal Generator",
            
            // I/O
            Self::Sensor => "Sensor",
            Self::Actuator => "Actuator",
            Self::Led => "LED",
            Self::Button => "Button",
            Self::Motor => "Motor",
            Self::MotorDc => "DC Motor",
            Self::MotorStepper => "Stepper Motor",
            Self::MotorServo => "Servo Motor",
            Self::HBridge => "H-Bridge",
            Self::Display => "Display",
            Self::Encoder => "Encoder",
            Self::Buzzer => "Buzzer",
            Self::Relay => "Relay",
            Self::TempSensor => "Temperature Sensor",
            Self::ProximitySensor => "Proximity Sensor",
            Self::Imu => "IMU",
            
            // Data
            Self::Variable => "Variable",
            Self::Constant => "Constant",
            Self::Array => "Array",
            Self::Struct => "Struct",
            Self::Queue => "Queue",
            Self::RingBuffer => "Ring Buffer",
            Self::Watchpoint => "Watchpoint",
            
            // Wireless
            Self::Wifi => "WiFi",
            Self::Ble => "Bluetooth LE",
            Self::Lora => "LoRa/LoRaWAN",
            Self::Zigbee => "Zigbee/Thread",
            Self::Bluetooth => "Bluetooth",
            
            // Protocols
            Self::Mqtt => "MQTT",
            Self::Http => "HTTP",
            Self::WebSocket => "WebSocket",
            Self::Modbus => "Modbus",
            Self::CanOpen => "CANopen",
            
            // System
            Self::Watchdog => "Watchdog Timer",
            Self::PowerMode => "Power Management",
            Self::Clock => "Clock Config",
            Self::DebugLog => "Debug Log",
            Self::Flash => "Flash Memory",
            Self::Reset => "Reset",
            
            // Legacy
            Self::Input => "Input",
            Self::Output => "Output",
            Self::Process => "Process",
            Self::Error => "Error",
            Self::Hardware => "Hardware",
            
            Self::Custom { .. } => "Custom Node",
        }
    }
    
    /// Get icon for this node type
    pub fn icon(&self) -> &str {
        match self {
            // FSM
            Self::Initial => "◉",
            Self::State => "●",
            Self::Final => "◎",
            Self::Decision => "◇",
            Self::Junction => "⊕",
            Self::Fork => "⊤",
            Self::Join => "⊥",
            Self::History => "Ⓗ",
            Self::DeepHistory => "Ⓗ*",
            Self::Composite => "▣",
            
            // Hardware
            Self::Gpio => "📍",
            Self::DigitalInput => "→",
            Self::DigitalOutput => "←",
            Self::Adc => "📊",
            Self::Dac => "📈",
            Self::Pwm => "〜",
            Self::Exti => "⚡",
            Self::Dma => "⟹",
            Self::InputCapture => "⏱",
            Self::OutputCompare => "⏲",
            Self::Comparator => "≷",
            Self::OpAmp => "△",
            
            // Communication
            Self::Uart => "↔",
            Self::Spi => "⇋",
            Self::I2c => "⇆",
            Self::Can => "🚗",
            Self::Usb => "🔌",
            Self::Ethernet => "🌐",
            Self::Rs485 => "⇌",
            
            // Timers
            Self::Timer => "⏰",
            Self::Delay => "⏱",
            Self::Rtc => "🕐",
            
            // Control
            Self::Interrupt => "⚡",
            Self::Event => "🔔",
            Self::Semaphore => "🚦",
            Self::Mutex => "🔒",
            Self::Task => "📌",
            Self::Critical => "⛔",
            Self::MessageQueue => "📬",
            Self::EventFlags => "🚩",
            Self::SwTimer => "⏲",
            Self::MemPool => "💾",
            
            // Processing
            Self::Filter => "▽",
            Self::MathOp => "±",
            Self::Compare => "≷",
            Self::Mux => "⊳",
            Self::Demux => "⊲",
            Self::Buffer => "▭",
            Self::Lut => "📋",
            Self::Pid => "⟲",
            Self::SignalGen => "∿",
            
            // I/O
            Self::Sensor => "📡",
            Self::Actuator => "⚙️",
            Self::Led => "💡",
            Self::Button => "🔘",
            Self::Motor | Self::MotorDc | Self::MotorStepper | Self::MotorServo => "🔄",
            Self::HBridge => "⟺",
            Self::Display => "🖥",
            Self::Encoder => "🎚",
            Self::Buzzer => "🔊",
            Self::Relay => "⚡",
            Self::TempSensor => "🌡",
            Self::ProximitySensor => "📍",
            Self::Imu => "🧭",
            
            // Data
            Self::Variable => "𝑥",
            Self::Constant => "π",
            Self::Array => "[]",
            Self::Struct => "{}",
            Self::Queue => "⤍",
            Self::RingBuffer => "⟳",
            Self::Watchpoint => "👁",
            
            // Wireless
            Self::Wifi => "📶",
            Self::Ble => "🔵",
            Self::Lora => "📻",
            Self::Zigbee => "🕸",
            Self::Bluetooth => "🔷",
            
            // Protocols
            Self::Mqtt => "📨",
            Self::Http => "🌐",
            Self::WebSocket => "🔗",
            Self::Modbus => "🏭",
            Self::CanOpen => "🔧",
            
            // System
            Self::Watchdog => "🐕",
            Self::PowerMode => "🔋",
            Self::Clock => "🕐",
            Self::DebugLog => "📝",
            Self::Flash => "💾",
            Self::Reset => "🔄",
            
            // Legacy
            Self::Input => "→",
            Self::Output => "←",
            Self::Process => "⚙",
            Self::Error => "❌",
            Self::Hardware => "🔧",
            
            Self::Custom { icon, .. } => icon.as_deref().unwrap_or("🎨"),
        }
    }
    
    /// Get all node types in a category
    pub fn by_category(category: NodeCategory) -> Vec<Self> {
        match category {
            NodeCategory::Fsm => vec![
                Self::Initial, Self::State, Self::Final, Self::Decision,
                Self::Junction, Self::Fork, Self::Join, Self::History,
                Self::DeepHistory, Self::Composite,
            ],
            NodeCategory::Hardware => vec![
                Self::Gpio, Self::DigitalInput, Self::DigitalOutput,
                Self::Adc, Self::Dac, Self::Pwm, Self::Exti, Self::Dma,
                Self::InputCapture, Self::OutputCompare, Self::Comparator,
                Self::OpAmp, Self::Timer, Self::Delay, Self::Rtc,
            ],
            NodeCategory::Communication => vec![
                Self::Uart, Self::Spi, Self::I2c, Self::Can,
                Self::Usb, Self::Ethernet, Self::Rs485,
            ],
            NodeCategory::Control => vec![
                Self::Interrupt, Self::Event, Self::Semaphore, Self::Mutex,
                Self::Task, Self::Critical, Self::MessageQueue, Self::EventFlags,
                Self::SwTimer, Self::MemPool,
            ],
            NodeCategory::Processing => vec![
                Self::Filter, Self::MathOp, Self::Compare, Self::Mux,
                Self::Demux, Self::Buffer, Self::Lut, Self::Pid, Self::SignalGen,
            ],
            NodeCategory::Io => vec![
                Self::Sensor, Self::Actuator, Self::Led, Self::Button,
                Self::Motor, Self::MotorDc, Self::MotorStepper, Self::MotorServo,
                Self::HBridge, Self::Display, Self::Encoder, Self::Buzzer,
                Self::Relay, Self::TempSensor, Self::ProximitySensor, Self::Imu,
            ],
            NodeCategory::Data => vec![
                Self::Variable, Self::Constant, Self::Array, Self::Struct,
                Self::Queue, Self::RingBuffer, Self::Watchpoint,
            ],
            NodeCategory::Wireless => vec![
                Self::Wifi, Self::Ble, Self::Lora, Self::Zigbee, Self::Bluetooth,
            ],
            NodeCategory::Protocol => vec![
                Self::Mqtt, Self::Http, Self::WebSocket, Self::Modbus, Self::CanOpen,
            ],
            NodeCategory::System => vec![
                Self::Watchdog, Self::PowerMode, Self::Clock, Self::DebugLog,
                Self::Flash, Self::Reset,
            ],
            NodeCategory::Custom => vec![],
        }
    }
    
    /// Check if this is an FSM-specific node type
    pub fn is_fsm(&self) -> bool {
        self.category() == NodeCategory::Fsm
    }
    
    /// Check if this is a hardware peripheral node
    pub fn is_peripheral(&self) -> bool {
        matches!(
            self.category(),
            NodeCategory::Hardware | NodeCategory::Communication
        )
    }
    
    /// Check if this is an RTOS component
    pub fn is_rtos(&self) -> bool {
        self.category() == NodeCategory::Control
    }
}

/// Node type info for UI display
#[derive(Debug, Clone, Serialize)]
pub struct NodeTypeInfo {
    pub node_type: NodeType,
    pub category: NodeCategory,
    pub name: String,
    pub icon: String,
    pub default_width: f64,
    pub default_height: f64,
}

impl NodeTypeInfo {
    pub fn from_type(node_type: &NodeType) -> Self {
        let (width, height) = match node_type.category() {
            NodeCategory::Fsm => (160.0, 80.0),
            NodeCategory::Hardware => (180.0, 100.0),
            NodeCategory::Processing => (140.0, 60.0),
            NodeCategory::Control => (160.0, 90.0),
            NodeCategory::Io => (140.0, 80.0),
            NodeCategory::Data => (120.0, 60.0),
            _ => (150.0, 80.0),
        };
        
        Self {
            node_type: node_type.clone(),
            category: node_type.category(),
            name: node_type.display_name().to_string(),
            icon: node_type.icon().to_string(),
            default_width: width,
            default_height: height,
        }
    }
}

/// Get info for all node types (for palette/UI)
pub fn get_all_node_info() -> Vec<NodeTypeInfo> {
    let mut info = Vec::new();
    
    for category in NodeCategory::all() {
        for node_type in NodeType::by_category(category) {
            info.push(NodeTypeInfo::from_type(&node_type));
        }
    }
    
    info
}

/// Static node type info lookup
pub static NODE_TYPE_INFO: std::sync::LazyLock<Vec<NodeTypeInfo>> = 
    std::sync::LazyLock::new(get_all_node_info);

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_node_categories() {
        assert_eq!(NodeType::State.category(), NodeCategory::Fsm);
        assert_eq!(NodeType::Gpio.category(), NodeCategory::Hardware);
        assert_eq!(NodeType::Task.category(), NodeCategory::Control);
        assert_eq!(NodeType::Wifi.category(), NodeCategory::Wireless);
    }
    
    #[test]
    fn test_by_category() {
        let fsm_nodes = NodeType::by_category(NodeCategory::Fsm);
        assert!(fsm_nodes.contains(&NodeType::Initial));
        assert!(fsm_nodes.contains(&NodeType::State));
        assert!(fsm_nodes.contains(&NodeType::Final));
    }
    
    #[test]  
    fn test_all_types_have_display_names() {
        for category in NodeCategory::all() {
            for node_type in NodeType::by_category(category) {
                let name = node_type.display_name();
                assert!(!name.is_empty(), "{:?} should have display name", node_type);
            }
        }
    }
}
