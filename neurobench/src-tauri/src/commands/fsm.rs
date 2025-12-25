// FSM Operation Commands

use crate::core::*;
use serde::{Deserialize, Serialize};

/// Add a node to the FSM
#[tauri::command]
pub fn add_node(
    label: String,
    node_type: NodeType,
    x: f64,
    y: f64,
) -> Result<FSMNode, String> {
    let node = FSMNode::new(label, node_type).with_position(x, y);
    log::debug!("Created node: {} ({:?})", node.label, node.node_type);
    Ok(node)
}

/// Remove a node from the FSM
#[tauri::command]
pub fn remove_node(node_id: String) -> Result<bool, String> {
    log::debug!("Remove node requested: {}", node_id);
    Ok(true)
}

/// Update a node's properties
#[tauri::command]
pub fn update_node(
    node_id: String,
    _label: Option<String>,
    _entry_action: Option<String>,
    _exit_action: Option<String>,
    _description: Option<String>,
) -> Result<bool, String> {
    log::debug!("Update node: {} with new properties", node_id);
    Ok(true)
}

/// Add an edge between nodes
#[tauri::command]
pub fn add_edge(
    source_id: String,
    target_id: String,
    label: Option<String>,
    guard: Option<String>,
) -> Result<FSMEdge, String> {
    let source = source_id.parse().map_err(|_| "Invalid source ID")?;
    let target = target_id.parse().map_err(|_| "Invalid target ID")?;
    
    let mut edge = FSMEdge::new(source, target);
    if let Some(l) = label {
        edge.label = Some(l);
    }
    if let Some(g) = guard {
        edge.guard = Some(g);
    }
    
    log::debug!("Created edge from {} to {}", source_id, target_id);
    Ok(edge)
}

/// Remove an edge
#[tauri::command]
pub fn remove_edge(edge_id: String) -> Result<bool, String> {
    log::debug!("Remove edge requested: {}", edge_id);
    Ok(true)
}

/// Update an edge's properties
#[tauri::command]
pub fn update_edge(
    edge_id: String,
    label: Option<String>,
    guard: Option<String>,
) -> Result<bool, String> {
    log::debug!("Update edge: {} with label={:?}, guard={:?}", edge_id, label, guard);
    Ok(true)
}

/// Execute a single simulation step
#[tauri::command]
pub fn simulate_step() -> Result<SimulationStepResult, String> {
    log::debug!("Simulation step requested");
    Ok(SimulationStepResult {
        status: SimulationStatus::Stepping,
        current_node: None,
        step_count: 0,
        logs: vec![],
    })
}

/// Start continuous simulation
#[tauri::command]
pub fn simulate_run() -> Result<SimulationStatus, String> {
    log::info!("Simulation run started");
    Ok(SimulationStatus::Running)
}

/// Stop simulation
#[tauri::command]
pub fn simulate_stop() -> Result<SimulationStatus, String> {
    log::info!("Simulation stopped");
    Ok(SimulationStatus::Idle)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationStepResult {
    pub status: SimulationStatus,
    pub current_node: Option<String>,
    pub step_count: u64,
    pub logs: Vec<LogEntry>,
}
