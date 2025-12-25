// FSM Core Types
// Defines nodes, edges, and state machine structures

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Unique identifier for FSM elements
pub type NodeId = Uuid;
pub type EdgeId = Uuid;
pub type ProjectId = Uuid;

/// FSM Node Types for different embedded system domains
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    // Core FSM types
    Input,
    Process,
    Output,
    Decision,
    Error,
    
    // Embedded-specific
    Hardware,
    Uart,
    Interrupt,
    Timer,
    Peripheral,
    Listener,
    
    // Concurrency
    Queue,
    Mutex,
    Critical,
    
    // I/O
    Sensor,
    Display,
    Storage,
    Network,
    Wireless,
    
    // Utility
    Math,
    Logger,
    
    // Grouping
    Group,
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::Process
    }
}

/// Position in the canvas
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// FSM Node representing a state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FSMNode {
    pub id: NodeId,
    pub label: String,
    pub node_type: NodeType,
    pub position: Position,
    
    // Code actions
    pub entry_action: Option<String>,
    pub exit_action: Option<String>,
    
    // Metadata
    pub description: Option<String>,
    pub tags: Vec<String>,
    
    // Runtime state
    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub is_breakpoint: bool,
    #[serde(default)]
    pub has_error: bool,
}

impl FSMNode {
    pub fn new(label: impl Into<String>, node_type: NodeType) -> Self {
        Self {
            id: Uuid::new_v4(),
            label: label.into(),
            node_type,
            position: Position::default(),
            entry_action: None,
            exit_action: None,
            description: None,
            tags: vec![],
            is_active: false,
            is_breakpoint: false,
            has_error: false,
        }
    }
    
    pub fn with_position(mut self, x: f64, y: f64) -> Self {
        self.position = Position { x, y };
        self
    }
    
    pub fn with_entry_action(mut self, code: impl Into<String>) -> Self {
        self.entry_action = Some(code.into());
        self
    }
}

/// FSM Edge representing a transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FSMEdge {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub label: Option<String>,
    
    // Guard condition (JavaScript expression)
    pub guard: Option<String>,
    
    // Runtime state
    #[serde(default)]
    pub is_traversing: bool,
}

impl FSMEdge {
    pub fn new(source: NodeId, target: NodeId) -> Self {
        Self {
            id: Uuid::new_v4(),
            source,
            target,
            label: None,
            guard: None,
            is_traversing: false,
        }
    }
    
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    
    pub fn with_guard(mut self, guard: impl Into<String>) -> Self {
        self.guard = Some(guard.into());
        self
    }
}

/// Simulation state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SimulationStatus {
    Idle,
    Running,
    Paused,
    Stepping,
    Error,
}

impl Default for SimulationStatus {
    fn default() -> Self {
        SimulationStatus::Idle
    }
}

/// Project containing an FSM design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FSMProject {
    pub id: ProjectId,
    pub name: String,
    pub description: Option<String>,
    pub nodes: Vec<FSMNode>,
    pub edges: Vec<FSMEdge>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub target_mcu: Option<String>,
}

impl FSMProject {
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            nodes: vec![],
            edges: vec![],
            created_at: now,
            updated_at: now,
            target_mcu: None,
        }
    }
}

/// Context variables during simulation
pub type SimulationContext = std::collections::HashMap<String, serde_json::Value>;

/// Log entry from simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub source: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Success,
    Debug,
}
