//! Transition Types
//!
//! Core types for transition/edge definitions

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Event that triggers a transition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum TransitionEvent {
    /// Timer-based transition after duration
    Timer { duration_ms: u64 },
    /// Signal/event name triggers transition
    Signal(String),
    /// Expression that evaluates to true
    Condition(String),
    /// Immediate transition (no trigger needed)
    Immediate,
    /// Interrupt source
    Interrupt { source: String, edge: InterruptEdge },
    /// Button/input event
    Input { name: String, state: InputState },
    /// Data received event
    DataReceived { source: String },
    /// Custom user-defined event
    Custom(String),
}

/// Interrupt edge type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InterruptEdge {
    Rising,
    Falling,
    Both,
}

/// Input state for button/switch events
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InputState {
    Pressed,
    Released,
    Held,
    DoublePress,
}

/// Connection point on a node (port)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConnectionPoint {
    pub node_id: String,
    pub port_name: Option<String>,
    pub port_index: Option<u8>,
}

impl ConnectionPoint {
    pub fn new(node_id: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            port_name: None,
            port_index: None,
        }
    }

    pub fn with_port(node_id: impl Into<String>, port: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            port_name: Some(port.into()),
            port_index: None,
        }
    }
}

/// A transition/edge between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    /// Unique identifier
    pub id: String,
    
    /// Source node and optional port
    pub source: ConnectionPoint,
    
    /// Target node and optional port
    pub target: ConnectionPoint,
    
    /// Display label
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    
    /// Event that triggers this transition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<TransitionEvent>,
    
    /// Guard condition (must be true for transition)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guard: Option<String>,
    
    /// Action code to execute during transition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    
    /// Priority (lower = higher priority, 0 = highest)
    #[serde(default)]
    pub priority: u8,
    
    /// Is this the default transition when no guard matches?
    #[serde(default)]
    pub is_default: bool,
    
    /// Transition is enabled/active
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Visual routing waypoints (for UI)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub waypoints: Vec<Waypoint>,
    
    /// Creation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    
    /// Last modified timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

fn default_true() -> bool { true }

/// Waypoint for edge routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waypoint {
    pub x: f64,
    pub y: f64,
}

impl Transition {
    /// Create a new transition between two nodes
    pub fn new(
        id: impl Into<String>,
        source_node: impl Into<String>,
        target_node: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            source: ConnectionPoint::new(source_node),
            target: ConnectionPoint::new(target_node),
            label: None,
            event: None,
            guard: None,
            action: None,
            priority: 0,
            is_default: false,
            enabled: true,
            waypoints: Vec::new(),
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            updated_at: None,
        }
    }

    /// Create with ports specified
    pub fn with_ports(
        id: impl Into<String>,
        source_node: impl Into<String>,
        source_port: impl Into<String>,
        target_node: impl Into<String>,
        target_port: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            source: ConnectionPoint::with_port(source_node, source_port),
            target: ConnectionPoint::with_port(target_node, target_port),
            label: None,
            event: None,
            guard: None,
            action: None,
            priority: 0,
            is_default: false,
            enabled: true,
            waypoints: Vec::new(),
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            updated_at: None,
        }
    }

    /// Set the event trigger
    pub fn with_event(mut self, event: TransitionEvent) -> Self {
        self.event = Some(event);
        self
    }

    /// Set the guard condition
    pub fn with_guard(mut self, guard: impl Into<String>) -> Self {
        self.guard = Some(guard.into());
        self
    }

    /// Set the action code
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Set the label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Mark as default transition
    pub fn as_default(mut self) -> Self {
        self.is_default = true;
        self
    }

    /// Get display text for the transition
    pub fn display_text(&self) -> String {
        if let Some(ref label) = self.label {
            return label.clone();
        }
        
        match &self.event {
            Some(TransitionEvent::Signal(s)) => s.clone(),
            Some(TransitionEvent::Timer { duration_ms }) => format!("{}ms", duration_ms),
            Some(TransitionEvent::Condition(c)) => format!("[{}]", c),
            Some(TransitionEvent::Immediate) => "→".to_string(),
            Some(TransitionEvent::Interrupt { source, .. }) => format!("IRQ:{}", source),
            Some(TransitionEvent::Input { name, state }) => format!("{}:{:?}", name, state),
            Some(TransitionEvent::DataReceived { source }) => format!("RX:{}", source),
            Some(TransitionEvent::Custom(c)) => c.clone(),
            None => String::new(),
        }
    }
}

/// Update fields for a transition
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransitionUpdate {
    pub label: Option<String>,
    pub event: Option<TransitionEvent>,
    pub guard: Option<String>,
    pub action: Option<String>,
    pub priority: Option<u8>,
    pub is_default: Option<bool>,
    pub enabled: Option<bool>,
    pub waypoints: Option<Vec<Waypoint>>,
}

/// Transition info for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionInfo {
    pub id: String,
    pub source_node: String,
    pub source_port: Option<String>,
    pub target_node: String,
    pub target_port: Option<String>,
    pub label: Option<String>,
    pub event_type: Option<String>,
    pub has_guard: bool,
    pub has_action: bool,
    pub priority: u8,
    pub enabled: bool,
}

impl From<&Transition> for TransitionInfo {
    fn from(t: &Transition) -> Self {
        Self {
            id: t.id.clone(),
            source_node: t.source.node_id.clone(),
            source_port: t.source.port_name.clone(),
            target_node: t.target.node_id.clone(),
            target_port: t.target.port_name.clone(),
            label: t.label.clone(),
            event_type: t.event.as_ref().map(|e| match e {
                TransitionEvent::Timer { .. } => "timer",
                TransitionEvent::Signal(_) => "signal",
                TransitionEvent::Condition(_) => "condition",
                TransitionEvent::Immediate => "immediate",
                TransitionEvent::Interrupt { .. } => "interrupt",
                TransitionEvent::Input { .. } => "input",
                TransitionEvent::DataReceived { .. } => "data_received",
                TransitionEvent::Custom(_) => "custom",
            }.to_string()),
            has_guard: t.guard.is_some(),
            has_action: t.action.is_some(),
            priority: t.priority,
            enabled: t.enabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_creation() {
        let t = Transition::new("t1", "node1", "node2")
            .with_label("Button Press")
            .with_event(TransitionEvent::Signal("BTN_PRESSED".to_string()))
            .with_guard("counter < 10")
            .with_action("counter++;");

        assert_eq!(t.id, "t1");
        assert_eq!(t.source.node_id, "node1");
        assert_eq!(t.target.node_id, "node2");
        assert!(t.guard.is_some());
        assert!(t.action.is_some());
    }

    #[test]
    fn test_display_text() {
        let t1 = Transition::new("t1", "a", "b")
            .with_label("My Label");
        assert_eq!(t1.display_text(), "My Label");

        let t2 = Transition::new("t2", "a", "b")
            .with_event(TransitionEvent::Timer { duration_ms: 1000 });
        assert_eq!(t2.display_text(), "1000ms");
    }
}
