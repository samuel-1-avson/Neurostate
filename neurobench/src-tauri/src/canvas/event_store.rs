//! Event Store - Persistent event sourcing for all embedded design operations
//!
//! Core architecture for tracking, replaying, and diffing all changes.

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// =============================================================================
// DOMAIN EVENTS
// =============================================================================

/// Unique event ID
pub type EventId = String;

/// Domain event covering ALL embedded design operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "domain", content = "event")]
pub enum DomainEvent {
    /// Canvas/node operations
    Canvas(CanvasEvent),
    /// FSM-specific operations
    Fsm(FsmEvent),
    /// Pin configuration
    Pin(PinEvent),
    /// Peripheral configuration
    Peripheral(PeripheralEvent),
    /// Code generation
    CodeGen(CodeGenEvent),
    /// Project settings
    Project(ProjectEvent),
}

/// Canvas events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CanvasEvent {
    // Node operations
    AddNode { node_id: String, node_type: String, x: f64, y: f64, label: String },
    RemoveNode { node_id: String, snapshot: serde_json::Value },
    MoveNode { node_id: String, from_x: f64, from_y: f64, to_x: f64, to_y: f64 },
    UpdateNodeProperty { node_id: String, property: String, old_value: serde_json::Value, new_value: serde_json::Value },
    
    // Edge operations
    AddEdge { edge_id: String, source: String, target: String, label: Option<String> },
    RemoveEdge { edge_id: String, snapshot: serde_json::Value },
    UpdateEdge { edge_id: String, property: String, old_value: serde_json::Value, new_value: serde_json::Value },
    
    // Group operations
    CreateGroup { group_id: String, node_ids: Vec<String>, label: String },
    Ungroup { group_id: String, node_ids: Vec<String> },
    
    // Layer operations
    CreateLayer { layer_id: String, name: String },
    DeleteLayer { layer_id: String, snapshot: serde_json::Value },
    MoveToLayer { node_id: String, from_layer: Option<String>, to_layer: String },
}

/// FSM-specific events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FsmEvent {
    SetInitialState { state_id: String, previous: Option<String> },
    SetFinalState { state_id: String, is_final: bool },
    AddTransition { edge_id: String, from: String, to: String, trigger: Option<String> },
    SetGuard { edge_id: String, guard: Option<String>, previous: Option<String> },
    SetAction { edge_id: String, action: Option<String>, previous: Option<String> },
    SetEntryAction { state_id: String, action: Option<String>, previous: Option<String> },
    SetExitAction { state_id: String, action: Option<String>, previous: Option<String> },
}

/// Pin configuration events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PinEvent {
    AssignPin { pin_id: String, function: String, previous: Option<String> },
    ConfigurePin { pin_id: String, config: serde_json::Value, previous: serde_json::Value },
    UnassignPin { pin_id: String, previous_function: String },
    ResolveConflict { pin_id: String, resolution: String },
}

/// Peripheral configuration events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PeripheralEvent {
    ConfigureUart { instance: String, config: serde_json::Value, previous: Option<serde_json::Value> },
    ConfigureSpi { instance: String, config: serde_json::Value, previous: Option<serde_json::Value> },
    ConfigureI2c { instance: String, config: serde_json::Value, previous: Option<serde_json::Value> },
    ConfigureTimer { instance: String, config: serde_json::Value, previous: Option<serde_json::Value> },
    ConfigureDma { channel: String, config: serde_json::Value, previous: Option<serde_json::Value> },
    ConfigureAdc { instance: String, config: serde_json::Value, previous: Option<serde_json::Value> },
    EnablePeripheral { peripheral: String, instance: String },
    DisablePeripheral { peripheral: String, instance: String },
}

/// Code generation events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CodeGenEvent {
    GenerateFile { path: String, content_hash: String },
    UpdateFile { path: String, old_hash: String, new_hash: String },
    DeleteFile { path: String, content_hash: String },
    ModifySnippet { file: String, snippet_id: String, old_content: String, new_content: String },
}

/// Project events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProjectEvent {
    Create { project_id: String, name: String, target: String },
    Rename { old_name: String, new_name: String },
    ChangeTarget { old_target: String, new_target: String },
    AddDependency { name: String, version: String },
    RemoveDependency { name: String, version: String },
    UpdateSetting { key: String, old_value: serde_json::Value, new_value: serde_json::Value },
}

// =============================================================================
// EVENT ENVELOPE
// =============================================================================

/// Complete event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Unique event ID
    pub id: EventId,
    /// Sequence number (monotonic)
    pub sequence: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Author/source
    pub author: String,
    /// The domain event
    pub event: DomainEvent,
    /// Optional correlation ID (for grouping related events)
    pub correlation_id: Option<String>,
    /// Tags for filtering
    pub tags: Vec<String>,
}

impl EventEnvelope {
    pub fn new(event: DomainEvent, author: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            sequence: 0, // Set by store
            timestamp: Utc::now(),
            author: author.into(),
            event,
            correlation_id: None,
            tags: Vec::new(),
        }
    }

    pub fn with_correlation(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

// =============================================================================
// SNAPSHOT
// =============================================================================

/// State snapshot for fast replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Snapshot ID
    pub id: String,
    /// At which event sequence
    pub at_sequence: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Name/tag
    pub name: Option<String>,
    /// Serialized state
    pub state: serde_json::Value,
}

// =============================================================================
// EVENT STORE
// =============================================================================

/// In-memory event store with optional persistence
#[derive(Debug, Default)]
pub struct EventStore {
    /// All events
    events: Vec<EventEnvelope>,
    /// Snapshots
    snapshots: Vec<Snapshot>,
    /// Current sequence number
    sequence: u64,
    /// Index by correlation ID
    correlation_index: HashMap<String, Vec<usize>>,
}

impl EventStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an event
    pub fn append(&mut self, event: DomainEvent, author: impl Into<String>) -> EventId {
        let mut envelope = EventEnvelope::new(event, author);
        self.sequence += 1;
        envelope.sequence = self.sequence;
        
        let id = envelope.id.clone();
        let idx = self.events.len();
        
        if let Some(ref corr) = envelope.correlation_id {
            self.correlation_index.entry(corr.clone())
                .or_default()
                .push(idx);
        }
        
        self.events.push(envelope);
        id
    }

    /// Append with correlation
    pub fn append_correlated(
        &mut self, 
        event: DomainEvent, 
        author: impl Into<String>,
        correlation_id: impl Into<String>,
    ) -> EventId {
        let mut envelope = EventEnvelope::new(event, author)
            .with_correlation(correlation_id);
        self.sequence += 1;
        envelope.sequence = self.sequence;
        
        let id = envelope.id.clone();
        let idx = self.events.len();
        
        if let Some(ref corr) = envelope.correlation_id {
            self.correlation_index.entry(corr.clone())
                .or_default()
                .push(idx);
        }
        
        self.events.push(envelope);
        id
    }

    /// Get all events
    pub fn all(&self) -> &[EventEnvelope] {
        &self.events
    }

    /// Get events since sequence
    pub fn since(&self, sequence: u64) -> Vec<&EventEnvelope> {
        self.events.iter()
            .filter(|e| e.sequence > sequence)
            .collect()
    }

    /// Get events by correlation ID
    pub fn by_correlation(&self, correlation_id: &str) -> Vec<&EventEnvelope> {
        self.correlation_index.get(correlation_id)
            .map(|indices| indices.iter().filter_map(|&i| self.events.get(i)).collect())
            .unwrap_or_default()
    }

    /// Get events by domain
    pub fn by_domain(&self, domain: &str) -> Vec<&EventEnvelope> {
        self.events.iter()
            .filter(|e| matches!(
                (&e.event, domain),
                (DomainEvent::Canvas(_), "canvas") |
                (DomainEvent::Fsm(_), "fsm") |
                (DomainEvent::Pin(_), "pin") |
                (DomainEvent::Peripheral(_), "peripheral") |
                (DomainEvent::CodeGen(_), "codegen") |
                (DomainEvent::Project(_), "project")
            ))
            .collect()
    }

    /// Get current sequence
    pub fn current_sequence(&self) -> u64 {
        self.sequence
    }

    /// Get event count
    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Create a snapshot
    pub fn create_snapshot(&mut self, state: serde_json::Value, name: Option<String>) -> String {
        let snapshot = Snapshot {
            id: uuid::Uuid::new_v4().to_string(),
            at_sequence: self.sequence,
            timestamp: Utc::now(),
            name,
            state,
        };
        let id = snapshot.id.clone();
        self.snapshots.push(snapshot);
        id
    }

    /// Get latest snapshot before sequence
    pub fn snapshot_before(&self, sequence: u64) -> Option<&Snapshot> {
        self.snapshots.iter()
            .filter(|s| s.at_sequence <= sequence)
            .max_by_key(|s| s.at_sequence)
    }

    /// Get events between sequences
    pub fn range(&self, from: u64, to: u64) -> Vec<&EventEnvelope> {
        self.events.iter()
            .filter(|e| e.sequence >= from && e.sequence <= to)
            .collect()
    }

    /// Clear all events (careful!)
    pub fn clear(&mut self) {
        self.events.clear();
        self.snapshots.clear();
        self.correlation_index.clear();
        self.sequence = 0;
    }
}

// =============================================================================
// INVERSE OPERATIONS
// =============================================================================

impl DomainEvent {
    /// Generate inverse event for undo
    pub fn inverse(&self) -> Option<DomainEvent> {
        match self {
            DomainEvent::Canvas(e) => e.inverse().map(DomainEvent::Canvas),
            DomainEvent::Fsm(e) => e.inverse().map(DomainEvent::Fsm),
            DomainEvent::Pin(e) => e.inverse().map(DomainEvent::Pin),
            DomainEvent::Peripheral(e) => e.inverse().map(DomainEvent::Peripheral),
            DomainEvent::CodeGen(e) => e.inverse().map(DomainEvent::CodeGen),
            DomainEvent::Project(e) => e.inverse().map(DomainEvent::Project),
        }
    }
}

impl CanvasEvent {
    pub fn inverse(&self) -> Option<CanvasEvent> {
        match self {
            CanvasEvent::AddNode { node_id, .. } => Some(CanvasEvent::RemoveNode {
                node_id: node_id.clone(),
                snapshot: serde_json::Value::Null,
            }),
            CanvasEvent::RemoveNode { node_id, snapshot } => Some(CanvasEvent::AddNode {
                node_id: node_id.clone(),
                node_type: snapshot.get("node_type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                x: snapshot.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
                y: snapshot.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
                label: snapshot.get("label").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            }),
            CanvasEvent::MoveNode { node_id, from_x, from_y, to_x, to_y } => Some(CanvasEvent::MoveNode {
                node_id: node_id.clone(),
                from_x: *to_x,
                from_y: *to_y,
                to_x: *from_x,
                to_y: *from_y,
            }),
            CanvasEvent::UpdateNodeProperty { node_id, property, old_value, new_value } => Some(CanvasEvent::UpdateNodeProperty {
                node_id: node_id.clone(),
                property: property.clone(),
                old_value: new_value.clone(),
                new_value: old_value.clone(),
            }),
            CanvasEvent::AddEdge { edge_id, .. } => Some(CanvasEvent::RemoveEdge {
                edge_id: edge_id.clone(),
                snapshot: serde_json::Value::Null,
            }),
            CanvasEvent::RemoveEdge { edge_id, snapshot } => Some(CanvasEvent::AddEdge {
                edge_id: edge_id.clone(),
                source: snapshot.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                target: snapshot.get("target").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                label: snapshot.get("label").and_then(|v| v.as_str()).map(|s| s.to_string()),
            }),
            _ => None,
        }
    }
}

impl FsmEvent {
    pub fn inverse(&self) -> Option<FsmEvent> {
        match self {
            FsmEvent::SetInitialState { state_id, previous } => Some(FsmEvent::SetInitialState {
                state_id: previous.clone().unwrap_or_default(),
                previous: Some(state_id.clone()),
            }),
            FsmEvent::SetGuard { edge_id, guard, previous } => Some(FsmEvent::SetGuard {
                edge_id: edge_id.clone(),
                guard: previous.clone(),
                previous: guard.clone(),
            }),
            FsmEvent::SetAction { edge_id, action, previous } => Some(FsmEvent::SetAction {
                edge_id: edge_id.clone(),
                action: previous.clone(),
                previous: action.clone(),
            }),
            _ => None,
        }
    }
}

impl PinEvent {
    pub fn inverse(&self) -> Option<PinEvent> {
        match self {
            PinEvent::AssignPin { pin_id, function, previous } => Some(match previous {
                Some(prev) => PinEvent::AssignPin {
                    pin_id: pin_id.clone(),
                    function: prev.clone(),
                    previous: Some(function.clone()),
                },
                None => PinEvent::UnassignPin {
                    pin_id: pin_id.clone(),
                    previous_function: function.clone(),
                },
            }),
            PinEvent::UnassignPin { pin_id, previous_function } => Some(PinEvent::AssignPin {
                pin_id: pin_id.clone(),
                function: previous_function.clone(),
                previous: None,
            }),
            _ => None,
        }
    }
}

impl PeripheralEvent {
    pub fn inverse(&self) -> Option<PeripheralEvent> {
        match self {
            PeripheralEvent::ConfigureUart { instance, config, previous } => Some(PeripheralEvent::ConfigureUart {
                instance: instance.clone(),
                config: previous.clone().unwrap_or_default(),
                previous: Some(config.clone()),
            }),
            PeripheralEvent::EnablePeripheral { peripheral, instance } => Some(PeripheralEvent::DisablePeripheral {
                peripheral: peripheral.clone(),
                instance: instance.clone(),
            }),
            PeripheralEvent::DisablePeripheral { peripheral, instance } => Some(PeripheralEvent::EnablePeripheral {
                peripheral: peripheral.clone(),
                instance: instance.clone(),
            }),
            _ => None,
        }
    }
}

impl CodeGenEvent {
    pub fn inverse(&self) -> Option<CodeGenEvent> {
        match self {
            CodeGenEvent::UpdateFile { path, old_hash, new_hash } => Some(CodeGenEvent::UpdateFile {
                path: path.clone(),
                old_hash: new_hash.clone(),
                new_hash: old_hash.clone(),
            }),
            _ => None,
        }
    }
}

impl ProjectEvent {
    pub fn inverse(&self) -> Option<ProjectEvent> {
        match self {
            ProjectEvent::Rename { old_name, new_name } => Some(ProjectEvent::Rename {
                old_name: new_name.clone(),
                new_name: old_name.clone(),
            }),
            ProjectEvent::ChangeTarget { old_target, new_target } => Some(ProjectEvent::ChangeTarget {
                old_target: new_target.clone(),
                new_target: old_target.clone(),
            }),
            ProjectEvent::UpdateSetting { key, old_value, new_value } => Some(ProjectEvent::UpdateSetting {
                key: key.clone(),
                old_value: new_value.clone(),
                new_value: old_value.clone(),
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_store() {
        let mut store = EventStore::new();
        
        let id = store.append(
            DomainEvent::Canvas(CanvasEvent::AddNode {
                node_id: "n1".into(),
                node_type: "state".into(),
                x: 100.0,
                y: 100.0,
                label: "State1".into(),
            }),
            "user",
        );
        
        assert!(!id.is_empty());
        assert_eq!(store.len(), 1);
        assert_eq!(store.current_sequence(), 1);
    }

    #[test]
    fn test_inverse_operations() {
        let event = CanvasEvent::MoveNode {
            node_id: "n1".into(),
            from_x: 0.0,
            from_y: 0.0,
            to_x: 100.0,
            to_y: 100.0,
        };
        
        let inverse = event.inverse().unwrap();
        if let CanvasEvent::MoveNode { to_x, to_y, .. } = inverse {
            assert_eq!(to_x, 0.0);
            assert_eq!(to_y, 0.0);
        }
    }

    #[test]
    fn test_correlation() {
        let mut store = EventStore::new();
        let corr_id = "batch_001";
        
        store.append_correlated(
            DomainEvent::Canvas(CanvasEvent::AddNode {
                node_id: "n1".into(),
                node_type: "state".into(),
                x: 0.0, y: 0.0,
                label: "S1".into(),
            }),
            "user",
            corr_id,
        );
        
        store.append_correlated(
            DomainEvent::Canvas(CanvasEvent::AddNode {
                node_id: "n2".into(),
                node_type: "state".into(),
                x: 100.0, y: 0.0,
                label: "S2".into(),
            }),
            "user",
            corr_id,
        );
        
        let batch = store.by_correlation(corr_id);
        assert_eq!(batch.len(), 2);
    }
}
