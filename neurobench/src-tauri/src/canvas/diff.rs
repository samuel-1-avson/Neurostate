//! Diff Engine - Compare event sequences and detect changes
//!
//! Provides semantic diffing between versions of embedded designs.

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use crate::canvas::event_store::{DomainEvent, CanvasEvent, FsmEvent, EventEnvelope};

// =============================================================================
// CHANGE TYPES
// =============================================================================

/// A detected change between versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    /// Change ID
    pub id: String,
    /// Domain affected
    pub domain: String,
    /// Type of change
    pub change_type: ChangeType,
    /// Affected entity IDs
    pub entity_ids: Vec<String>,
    /// Human-readable description
    pub description: String,
    /// Is this a breaking change?
    pub breaking: bool,
    /// Related events
    pub event_ids: Vec<String>,
}

/// Type of change detected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    // Node changes
    NodeAdded,
    NodeRemoved,
    NodeMoved,
    NodePropertyChanged,
    
    // Edge changes
    EdgeAdded,
    EdgeRemoved,
    EdgeRerouted,
    
    // FSM changes
    InitialStateChanged,
    TransitionAdded,
    GuardChanged,
    ActionChanged,
    
    // Config changes
    PinAssigned,
    PinUnassigned,
    PeripheralConfigured,
    
    // Code changes
    FileGenerated,
    FileModified,
    FileDeleted,
    
    // Project changes
    TargetChanged,
    DependencyChanged,
    
    // Composite
    Multiple,
}

/// Classification of change severity
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ChangeSeverity {
    /// Cosmetic only (e.g., node moved)
    Cosmetic,
    /// Non-breaking functional change
    Minor,
    /// Significant change requiring attention
    Major,
    /// Breaking change affecting behavior
    Breaking,
}

// =============================================================================
// DIFF RESULT
// =============================================================================

/// Result of comparing two versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    /// From sequence
    pub from_sequence: u64,
    /// To sequence
    pub to_sequence: u64,
    /// All changes detected
    pub changes: Vec<Change>,
    /// Summary by domain
    pub summary: DiffSummary,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffSummary {
    pub total_changes: usize,
    pub nodes_added: usize,
    pub nodes_removed: usize,
    pub nodes_modified: usize,
    pub edges_added: usize,
    pub edges_removed: usize,
    pub fsm_changes: usize,
    pub config_changes: usize,
    pub code_changes: usize,
    pub breaking_changes: usize,
}

// =============================================================================
// DIFF ENGINE
// =============================================================================

/// Engine for comparing event sequences
#[derive(Debug, Default)]
pub struct DiffEngine {
    /// Semantic grouping rules
    grouping_enabled: bool,
}

impl DiffEngine {
    pub fn new() -> Self {
        Self { grouping_enabled: true }
    }

    /// Compare two event sequences
    pub fn diff(&self, events: &[EventEnvelope]) -> DiffResult {
        let mut changes = Vec::new();
        let mut summary = DiffSummary::default();

        let from_seq = events.first().map(|e| e.sequence).unwrap_or(0);
        let to_seq = events.last().map(|e| e.sequence).unwrap_or(0);

        // Process each event into a change
        for event in events {
            if let Some(change) = self.event_to_change(event) {
                self.update_summary(&change, &mut summary);
                changes.push(change);
            }
        }

        // Group related changes if enabled
        if self.grouping_enabled {
            changes = self.group_related_changes(changes);
        }

        DiffResult {
            from_sequence: from_seq,
            to_sequence: to_seq,
            changes,
            summary,
        }
    }

    /// Convert event to change
    fn event_to_change(&self, envelope: &EventEnvelope) -> Option<Change> {
        let (domain, change_type, entity_ids, description, breaking) = match &envelope.event {
            DomainEvent::Canvas(e) => self.canvas_event_to_change(e)?,
            DomainEvent::Fsm(e) => self.fsm_event_to_change(e)?,
            DomainEvent::Pin(e) => self.pin_event_to_change(e)?,
            DomainEvent::Peripheral(e) => self.peripheral_event_to_change(e)?,
            DomainEvent::CodeGen(e) => self.codegen_event_to_change(e)?,
            DomainEvent::Project(e) => self.project_event_to_change(e)?,
        };

        Some(Change {
            id: envelope.id.clone(),
            domain,
            change_type,
            entity_ids,
            description,
            breaking,
            event_ids: vec![envelope.id.clone()],
        })
    }

    fn canvas_event_to_change(&self, event: &CanvasEvent) -> Option<(String, ChangeType, Vec<String>, String, bool)> {
        match event {
            CanvasEvent::AddNode { node_id, label, .. } => Some((
                "canvas".into(),
                ChangeType::NodeAdded,
                vec![node_id.clone()],
                format!("Added node '{}'", label),
                false,
            )),
            CanvasEvent::RemoveNode { node_id, .. } => Some((
                "canvas".into(),
                ChangeType::NodeRemoved,
                vec![node_id.clone()],
                format!("Removed node '{}'", node_id),
                true,
            )),
            CanvasEvent::MoveNode { node_id, .. } => Some((
                "canvas".into(),
                ChangeType::NodeMoved,
                vec![node_id.clone()],
                format!("Moved node '{}'", node_id),
                false,
            )),
            CanvasEvent::UpdateNodeProperty { node_id, property, .. } => Some((
                "canvas".into(),
                ChangeType::NodePropertyChanged,
                vec![node_id.clone()],
                format!("Changed '{}' property on node '{}'", property, node_id),
                false,
            )),
            CanvasEvent::AddEdge { edge_id, source, target, .. } => Some((
                "canvas".into(),
                ChangeType::EdgeAdded,
                vec![edge_id.clone()],
                format!("Added edge {} → {}", source, target),
                false,
            )),
            CanvasEvent::RemoveEdge { edge_id, .. } => Some((
                "canvas".into(),
                ChangeType::EdgeRemoved,
                vec![edge_id.clone()],
                format!("Removed edge '{}'", edge_id),
                true,
            )),
            _ => None,
        }
    }

    fn fsm_event_to_change(&self, event: &FsmEvent) -> Option<(String, ChangeType, Vec<String>, String, bool)> {
        match event {
            FsmEvent::SetInitialState { state_id, .. } => Some((
                "fsm".into(),
                ChangeType::InitialStateChanged,
                vec![state_id.clone()],
                format!("Set initial state to '{}'", state_id),
                true,
            )),
            FsmEvent::AddTransition { edge_id, from, to, .. } => Some((
                "fsm".into(),
                ChangeType::TransitionAdded,
                vec![edge_id.clone()],
                format!("Added transition {} → {}", from, to),
                false,
            )),
            FsmEvent::SetGuard { edge_id, guard, .. } => Some((
                "fsm".into(),
                ChangeType::GuardChanged,
                vec![edge_id.clone()],
                format!("Set guard on edge '{}': {:?}", edge_id, guard),
                true,
            )),
            FsmEvent::SetAction { edge_id, action, .. } => Some((
                "fsm".into(),
                ChangeType::ActionChanged,
                vec![edge_id.clone()],
                format!("Set action on edge '{}': {:?}", edge_id, action),
                false,
            )),
            _ => None,
        }
    }

    fn pin_event_to_change(&self, event: &crate::canvas::event_store::PinEvent) -> Option<(String, ChangeType, Vec<String>, String, bool)> {
        match event {
            crate::canvas::event_store::PinEvent::AssignPin { pin_id, function, .. } => Some((
                "pin".into(),
                ChangeType::PinAssigned,
                vec![pin_id.clone()],
                format!("Assigned '{}' to {}", function, pin_id),
                false,
            )),
            crate::canvas::event_store::PinEvent::UnassignPin { pin_id, .. } => Some((
                "pin".into(),
                ChangeType::PinUnassigned,
                vec![pin_id.clone()],
                format!("Unassigned pin '{}'", pin_id),
                true,
            )),
            _ => None,
        }
    }

    fn peripheral_event_to_change(&self, event: &crate::canvas::event_store::PeripheralEvent) -> Option<(String, ChangeType, Vec<String>, String, bool)> {
        match event {
            crate::canvas::event_store::PeripheralEvent::ConfigureUart { instance, .. } => Some((
                "peripheral".into(),
                ChangeType::PeripheralConfigured,
                vec![instance.clone()],
                format!("Configured UART{}", instance),
                false,
            )),
            crate::canvas::event_store::PeripheralEvent::EnablePeripheral { peripheral, instance } => Some((
                "peripheral".into(),
                ChangeType::PeripheralConfigured,
                vec![format!("{}:{}", peripheral, instance)],
                format!("Enabled {}{}", peripheral, instance),
                false,
            )),
            _ => None,
        }
    }

    fn codegen_event_to_change(&self, event: &crate::canvas::event_store::CodeGenEvent) -> Option<(String, ChangeType, Vec<String>, String, bool)> {
        match event {
            crate::canvas::event_store::CodeGenEvent::GenerateFile { path, .. } => Some((
                "codegen".into(),
                ChangeType::FileGenerated,
                vec![path.clone()],
                format!("Generated file '{}'", path),
                false,
            )),
            crate::canvas::event_store::CodeGenEvent::UpdateFile { path, .. } => Some((
                "codegen".into(),
                ChangeType::FileModified,
                vec![path.clone()],
                format!("Modified file '{}'", path),
                false,
            )),
            crate::canvas::event_store::CodeGenEvent::DeleteFile { path, .. } => Some((
                "codegen".into(),
                ChangeType::FileDeleted,
                vec![path.clone()],
                format!("Deleted file '{}'", path),
                true,
            )),
            _ => None,
        }
    }

    fn project_event_to_change(&self, event: &crate::canvas::event_store::ProjectEvent) -> Option<(String, ChangeType, Vec<String>, String, bool)> {
        match event {
            crate::canvas::event_store::ProjectEvent::ChangeTarget { old_target, new_target } => Some((
                "project".into(),
                ChangeType::TargetChanged,
                vec![new_target.clone()],
                format!("Changed target from '{}' to '{}'", old_target, new_target),
                true,
            )),
            crate::canvas::event_store::ProjectEvent::AddDependency { name, version } => Some((
                "project".into(),
                ChangeType::DependencyChanged,
                vec![name.clone()],
                format!("Added dependency {} v{}", name, version),
                false,
            )),
            _ => None,
        }
    }

    fn update_summary(&self, change: &Change, summary: &mut DiffSummary) {
        summary.total_changes += 1;
        if change.breaking {
            summary.breaking_changes += 1;
        }

        match change.change_type {
            ChangeType::NodeAdded => summary.nodes_added += 1,
            ChangeType::NodeRemoved => summary.nodes_removed += 1,
            ChangeType::NodeMoved | ChangeType::NodePropertyChanged => summary.nodes_modified += 1,
            ChangeType::EdgeAdded => summary.edges_added += 1,
            ChangeType::EdgeRemoved => summary.edges_removed += 1,
            ChangeType::InitialStateChanged | ChangeType::TransitionAdded | 
            ChangeType::GuardChanged | ChangeType::ActionChanged => summary.fsm_changes += 1,
            ChangeType::PinAssigned | ChangeType::PinUnassigned |
            ChangeType::PeripheralConfigured => summary.config_changes += 1,
            ChangeType::FileGenerated | ChangeType::FileModified | 
            ChangeType::FileDeleted => summary.code_changes += 1,
            _ => {},
        }
    }

    /// Group related changes (e.g., multiple moves as one "rearrange")
    fn group_related_changes(&self, changes: Vec<Change>) -> Vec<Change> {
        // Group consecutive moves of same node type
        let mut grouped = Vec::new();
        let mut pending_moves: Vec<Change> = Vec::new();

        for change in changes {
            if change.change_type == ChangeType::NodeMoved {
                pending_moves.push(change);
            } else {
                // Flush pending moves
                if pending_moves.len() > 1 {
                    grouped.push(self.merge_moves(&pending_moves));
                } else {
                    grouped.extend(pending_moves.drain(..));
                }
                grouped.push(change);
            }
        }

        // Flush remaining
        if pending_moves.len() > 1 {
            grouped.push(self.merge_moves(&pending_moves));
        } else {
            grouped.extend(pending_moves);
        }

        grouped
    }

    fn merge_moves(&self, moves: &[Change]) -> Change {
        let entity_ids: Vec<String> = moves.iter()
            .flat_map(|c| c.entity_ids.clone())
            .collect();
        let event_ids: Vec<String> = moves.iter()
            .flat_map(|c| c.event_ids.clone())
            .collect();

        Change {
            id: uuid::Uuid::new_v4().to_string(),
            domain: "canvas".into(),
            change_type: ChangeType::Multiple,
            entity_ids,
            description: format!("Moved {} nodes", moves.len()),
            breaking: false,
            event_ids,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::event_store::{EventStore, DomainEvent, CanvasEvent};

    #[test]
    fn test_diff_engine() {
        let mut store = EventStore::new();
        
        store.append(DomainEvent::Canvas(CanvasEvent::AddNode {
            node_id: "n1".into(),
            node_type: "state".into(),
            x: 0.0, y: 0.0,
            label: "State1".into(),
        }), "user");

        store.append(DomainEvent::Canvas(CanvasEvent::MoveNode {
            node_id: "n1".into(),
            from_x: 0.0, from_y: 0.0,
            to_x: 100.0, to_y: 100.0,
        }), "user");

        let engine = DiffEngine::new();
        let result = engine.diff(store.all());

        assert_eq!(result.summary.nodes_added, 1);
        assert_eq!(result.summary.nodes_modified, 1);
    }
}
