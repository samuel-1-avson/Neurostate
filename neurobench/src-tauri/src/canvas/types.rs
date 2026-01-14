//! Canvas Types - Core data structures for the canvas engine

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Node type for FSM and embedded system nodes
/// Organized by category for industrial-grade embedded design
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    // === FSM Core ===
    /// Initial state (entry point)
    Initial,
    /// Regular state
    State,
    /// Final state (termination)
    Final,
    /// Choice/decision point (legacy name)
    Choice,
    /// Decision point (same as choice, for compatibility with nodes module)
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
    
    // === GPIO & Digital I/O ===
    /// GPIO pin configuration
    Gpio,
    /// Digital input
    DigitalInput,
    /// Digital output
    DigitalOutput,
    /// External interrupt (EXTI)
    Exti,
    /// Button/switch input
    Button,
    /// LED output
    Led,
    /// Relay control
    Relay,
    
    // === Communication Interfaces ===
    /// UART serial communication
    Uart,
    /// SPI bus
    Spi,
    /// I2C bus
    I2c,
    /// CAN bus
    Can,
    /// USB interface
    Usb,
    /// Ethernet
    Ethernet,
    /// RS485
    Rs485,
    /// Modbus protocol
    Modbus,
    
    // === Timers & Delays ===
    /// Hardware timer
    Timer,
    /// Software delay
    Delay,
    /// PWM output
    Pwm,
    /// Input capture
    InputCapture,
    /// Output compare
    OutputCompare,
    /// Real-time clock
    Rtc,
    /// Watchdog timer
    Watchdog,
    
    // === RTOS Components ===
    /// RTOS task
    Task,
    /// Semaphore
    Semaphore,
    /// Mutex
    Mutex,
    /// Message queue
    Queue,
    /// Event group
    EventGroup,
    /// Software timer (RTOS)
    SwTimer,
    /// Memory pool
    MemPool,
    /// Software event
    Event,
    /// Critical section
    Critical,
    /// Message queue (alternative name)
    MessageQueue,
    /// Event flags
    EventFlags,
    
    // === Analog ===
    /// ADC input
    Adc,
    /// DAC output
    Dac,
    /// Analog comparator
    Comparator,
    /// Op-amp configuration
    OpAmp,
    
    // === Processing ===
    /// Digital filter
    Filter,
    /// Math operation
    MathOp,
    /// Comparison
    Compare,
    /// Multiplexer
    Mux,
    /// Demultiplexer
    Demux,
    /// Data buffer
    Buffer,
    /// Lookup table
    Lut,
    /// PID Controller
    Pid,
    /// Signal generator
    SignalGen,
    
    // === Motor Control ===
    /// DC motor
    MotorDc,
    /// Stepper motor
    MotorStepper,
    /// Servo motor
    MotorServo,
    /// H-bridge driver
    HBridge,
    /// Encoder input
    Encoder,
    /// Generic motor
    Motor,
    
    // === Power & System ===
    /// Power mode control
    PowerMode,
    /// Power management
    PowerMgmt,
    /// Clock configuration
    Clock,
    /// Clock config (alternative)
    ClockConfig,
    /// DMA transfer
    Dma,
    /// Flash memory operation
    Flash,
    /// Reset control
    Reset,
    /// Debug logging
    DebugLog,
    
    // === Sensors ===
    /// Temperature sensor
    SensorTemp,
    /// Temperature sensor (alternative name)
    TempSensor,
    /// Accelerometer/IMU
    SensorImu,
    /// Proximity sensor
    SensorProximity,
    /// Generic sensor
    Sensor,
    
    // === I/O Devices ===
    /// Generic actuator
    Actuator,
    /// Display (LCD/OLED)
    Display,
    /// Buzzer/speaker
    Buzzer,
    
    // === Data Storage ===
    /// Runtime variable
    Variable,
    /// Compile-time constant
    Constant,
    /// Fixed-size array
    Array,
    /// Struct/record
    Struct,
    /// Ring buffer
    RingBuffer,
    /// Watchpoint (debug)
    Watchpoint,
    
    // === Wireless ===
    /// WiFi module
    Wifi,
    /// Bluetooth
    Bluetooth,
    /// Bluetooth Low Energy
    Ble,
    /// LoRa radio
    LoRa,
    /// LoRa (lowercase variant)
    Lora,
    /// Zigbee
    Zigbee,
    
    // === Protocols ===
    /// MQTT client
    Mqtt,
    /// HTTP client/server
    Http,
    /// WebSocket
    WebSocket,
    /// CANopen
    CanOpen,
    
    // === Legacy Compatibility ===
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
    /// Generic interrupt
    Interrupt,
    
    // === Custom ===
    /// Custom node type with user-defined category
    Custom {
        category: String,
        icon: Option<String>,
    },
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::State
    }
}

impl NodeType {
    /// Get the category for this node type
    pub fn category(&self) -> &'static str {
        match self {
            NodeType::Initial | NodeType::State | NodeType::Final | 
            NodeType::Choice | NodeType::Decision | NodeType::Junction |
            NodeType::Fork | NodeType::Join |
            NodeType::History | NodeType::DeepHistory | NodeType::Composite => "FSM",
            
            NodeType::Gpio | NodeType::DigitalInput | NodeType::DigitalOutput |
            NodeType::Exti | NodeType::Button | NodeType::Led |
            NodeType::Relay => "GPIO",
            
            NodeType::Uart | NodeType::Spi | NodeType::I2c | NodeType::Can |
            NodeType::Usb | NodeType::Ethernet | NodeType::Rs485 |
            NodeType::Modbus => "Communication",
            
            NodeType::Timer | NodeType::Delay | NodeType::Pwm |
            NodeType::InputCapture | NodeType::OutputCompare |
            NodeType::Rtc | NodeType::Watchdog => "Timer",
            
            NodeType::Task | NodeType::Semaphore | NodeType::Mutex |
            NodeType::Queue | NodeType::EventGroup | NodeType::SwTimer |
            NodeType::MemPool | NodeType::Event | NodeType::Critical |
            NodeType::MessageQueue | NodeType::EventFlags => "RTOS",
            
            NodeType::Adc | NodeType::Dac | NodeType::Comparator |
            NodeType::OpAmp => "Analog",
            
            NodeType::Filter | NodeType::MathOp | NodeType::Compare |
            NodeType::Mux | NodeType::Demux | NodeType::Buffer |
            NodeType::Lut | NodeType::Pid | NodeType::SignalGen => "Processing",
            
            NodeType::MotorDc | NodeType::MotorStepper | NodeType::MotorServo |
            NodeType::HBridge | NodeType::Encoder | NodeType::Motor => "Motor",
            
            NodeType::PowerMode | NodeType::PowerMgmt | NodeType::Clock | 
            NodeType::ClockConfig | NodeType::Dma |
            NodeType::Flash | NodeType::Reset | NodeType::DebugLog => "System",
            
            NodeType::SensorTemp | NodeType::TempSensor | NodeType::SensorImu | 
            NodeType::SensorProximity | NodeType::Sensor => "Sensor",
            
            NodeType::Actuator | NodeType::Display | NodeType::Buzzer => "IO",
            
            NodeType::Variable | NodeType::Constant | NodeType::Array |
            NodeType::Struct | NodeType::RingBuffer | NodeType::Watchpoint => "Data",
            
            NodeType::Wifi | NodeType::Bluetooth | NodeType::Ble |
            NodeType::LoRa | NodeType::Lora | NodeType::Zigbee => "Wireless",
            
            NodeType::Mqtt | NodeType::Http | NodeType::WebSocket |
            NodeType::CanOpen => "Protocol",
            
            NodeType::Input | NodeType::Output | NodeType::Process |
            NodeType::Error | NodeType::Hardware |
            NodeType::Interrupt => "General",
            
            NodeType::Custom { .. } => "Custom",
        }
    }
    
    /// Check if this is an FSM-specific node type
    pub fn is_fsm(&self) -> bool {
        self.category() == "FSM"
    }
    
    /// Check if this is a hardware peripheral node
    pub fn is_peripheral(&self) -> bool {
        matches!(self.category(), "GPIO" | "Communication" | "Timer" | "Analog" | "Motor" | "Sensor" | "Wireless")
    }
    
    /// Check if this is an RTOS component
    pub fn is_rtos(&self) -> bool {
        self.category() == "RTOS"
    }
}

/// A node in the canvas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasNode {
    pub id: String,
    pub label: String,
    pub node_type: NodeType,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl CanvasNode {
    pub fn new(id: String, label: String, node_type: NodeType, x: f64, y: f64) -> Self {
        Self {
            id,
            label,
            node_type,
            x,
            y,
            width: 160.0,
            height: 80.0,
            entry_action: None,
            exit_action: None,
            description: None,
        }
    }
    
    /// Get center point
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
    
    /// Get output port position (bottom center)
    pub fn output_port(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height)
    }
    
    /// Get input port position (top center)
    pub fn input_port(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y)
    }
    
    /// Check if point is inside node
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width &&
        y >= self.y && y <= self.y + self.height
    }
    
    /// Check if rectangle intersects node
    pub fn intersects(&self, rx: f64, ry: f64, rw: f64, rh: f64) -> bool {
        !(self.x + self.width < rx ||
          rx + rw < self.x ||
          self.y + self.height < ry ||
          ry + rh < self.y)
    }
}

/// An edge connecting two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// Guard condition expression (e.g., "x > 10 && ready")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guard: Option<String>,
    /// Action to execute on transition (e.g., "counter = 0; led_on()")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    /// Priority for multiple valid transitions (lower = higher priority, 0 = highest)
    #[serde(default)]
    pub priority: u8,
    /// Manual waypoints for edge routing
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub waypoints: Vec<EdgeWaypoint>,
    /// Routing style for this edge
    #[serde(default)]
    pub routing: EdgeRoutingStyle,
    /// Z-order for rendering (higher = on top)
    #[serde(default)]
    pub z_order: i32,
    /// Source port ID (if using typed ports)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_port: Option<String>,
    /// Target port ID (if using typed ports)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_port: Option<String>,
}

/// A waypoint on an edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeWaypoint {
    pub x: f64,
    pub y: f64,
}

/// Edge routing style
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EdgeRoutingStyle {
    #[default]
    Bezier,
    Orthogonal,
    OrthogonalRounded,
    Straight,
}

impl CanvasEdge {
    pub fn new(id: String, source: String, target: String, label: Option<String>) -> Self {
        Self {
            id,
            source,
            target,
            label,
            condition: None,
            guard: None,
            action: None,
            priority: 0,
            waypoints: Vec::new(),
            routing: EdgeRoutingStyle::default(),
            z_order: 0,
            source_port: None,
            target_port: None,
        }
    }

    /// Add a waypoint
    pub fn add_waypoint(&mut self, x: f64, y: f64) {
        self.waypoints.push(EdgeWaypoint { x, y });
    }

    /// Clear all waypoints
    pub fn clear_waypoints(&mut self) {
        self.waypoints.clear();
    }

    /// Set routing style
    pub fn with_routing(mut self, style: EdgeRoutingStyle) -> Self {
        self.routing = style;
        self
    }

    /// Check if edge has manual waypoints
    pub fn has_waypoints(&self) -> bool {
        !self.waypoints.is_empty()
    }
}

/// Node position update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMove {
    pub id: String,
    pub x: f64,
    pub y: f64,
}

/// Node property update
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_type: Option<NodeType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_action: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_action: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,
}

/// Canvas state snapshot for undo/redo
#[derive(Debug, Clone)]
pub struct CanvasSnapshot {
    pub nodes: HashMap<String, CanvasNode>,
    pub edges: HashMap<String, CanvasEdge>,
}

/// Current canvas state for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasState {
    pub nodes: Vec<CanvasNode>,
    pub edges: Vec<CanvasEdge>,
    pub selection: Vec<String>,
}

/// Result of delete operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteResult {
    pub deleted_nodes: Vec<String>,
    pub deleted_edges: Vec<String>,
}

/// Canvas operation errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CanvasError {
    NodeNotFound(String),
    EdgeNotFound(String),
    SelfLoop,
    DuplicateEdge,
    WouldCreateCycle,
    InvalidOperation(String),
}

impl std::fmt::Display for CanvasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CanvasError::NodeNotFound(id) => write!(f, "Node not found: {}", id),
            CanvasError::EdgeNotFound(id) => write!(f, "Edge not found: {}", id),
            CanvasError::SelfLoop => write!(f, "Cannot create self-loop"),
            CanvasError::DuplicateEdge => write!(f, "Edge already exists"),
            CanvasError::WouldCreateCycle => write!(f, "Would create cycle in DAG"),
            CanvasError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
        }
    }
}

impl std::error::Error for CanvasError {}

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

/// Layout algorithm types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutAlgorithm {
    /// Force-directed spring layout
    ForceDirected,
    /// Hierarchical top-to-bottom
    Hierarchical,
    /// Tree layout
    Tree,
    /// Grid layout
    Grid,
}

/// Alignment options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Alignment {
    Left,
    Right,
    Top,
    Bottom,
    CenterHorizontal,
    CenterVertical,
    DistributeHorizontal,
    DistributeVertical,
}

/// Edge routing style
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EdgeRouting {
    #[default]
    Bezier,
    Orthogonal,
    Straight,
}
