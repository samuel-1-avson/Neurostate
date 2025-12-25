// FSM Executor
// Runs the state machine simulation

use super::types::*;
use super::graph::FSMGraph;
use std::sync::{Arc, Mutex};
use chrono::Utc;

/// FSM Executor for running simulations
pub struct FSMExecutor {
    graph: Arc<Mutex<FSMGraph>>,
    current_node: Option<NodeId>,
    context: SimulationContext,
    status: SimulationStatus,
    logs: Vec<LogEntry>,
    step_count: u64,
}

impl FSMExecutor {
    pub fn new(graph: FSMGraph) -> Self {
        Self {
            graph: Arc::new(Mutex::new(graph)),
            current_node: None,
            context: SimulationContext::new(),
            status: SimulationStatus::Idle,
            logs: vec![],
            step_count: 0,
        }
    }
    
    /// Start simulation from the initial node
    pub fn start(&mut self) -> Result<(), String> {
        let (start_id, entry_action) = {
            let graph = self.graph.lock().map_err(|e| e.to_string())?;
            let start_node = graph.find_start_node()
                .ok_or("No start node found in the FSM")?;
            (start_node.id, start_node.entry_action.clone())
        };
        
        self.current_node = Some(start_id);
        self.status = SimulationStatus::Running;
        self.step_count = 0;
        self.context.clear();
        
        self.log(LogLevel::Info, "SYSTEM", "Simulation started");
        
        if let Some(action) = entry_action {
            self.log(LogLevel::Debug, "EXEC", &format!("Entry: {}", action));
        }
        
        Ok(())
    }
    
    /// Stop the simulation
    pub fn stop(&mut self) {
        self.status = SimulationStatus::Idle;
        self.current_node = None;
        self.log(LogLevel::Info, "SYSTEM", "Simulation stopped");
    }
    
    /// Execute a single step
    pub fn step(&mut self) -> Result<StepResult, String> {
        if self.status != SimulationStatus::Running && self.status != SimulationStatus::Stepping {
            return Err("Simulation not running".to_string());
        }
        
        let current_id = self.current_node
            .ok_or("No current state")?;
        
        // Extract all data we need from the graph first
        let step_data = {
            let graph = self.graph.lock().map_err(|e| e.to_string())?;
            
            let edges = graph.get_outgoing(current_id);
            
            if edges.is_empty() {
                let node = graph.get_node(current_id).ok_or("Node not found")?;
                if node.node_type == NodeType::Output {
                    return Ok(StepResult::Completed);
                } else {
                    return Ok(StepResult::Deadlock);
                }
            }
            
            let edge = edges.first().ok_or("No edges available")?;
            let next_node_id = edge.target;
            let transition_label = edge.label.clone().unwrap_or_else(|| "→".to_string());
            
            let exit_action = graph.get_node(current_id)
                .and_then(|n| n.exit_action.clone());
            let from_label = graph.get_node(current_id)
                .map(|n| n.label.clone())
                .unwrap_or_default();
            let to_label = graph.get_node(next_node_id)
                .map(|n| n.label.clone())
                .unwrap_or_default();
            let entry_action = graph.get_node(next_node_id)
                .and_then(|n| n.entry_action.clone());
            
            (next_node_id, transition_label, exit_action, from_label, to_label, entry_action)
        };
        
        let (next_node_id, transition_label, exit_action, from_label, to_label, entry_action) = step_data;
        
        // Now we can log without holding the lock
        if let Some(action) = exit_action {
            self.log(LogLevel::Debug, "EXEC", &format!("Exit: {}", action));
        }
        
        self.log(LogLevel::Info, "TRANSITION", &format!("{} --[{}]--> {}", from_label, transition_label, to_label));
        
        if let Some(action) = entry_action {
            self.log(LogLevel::Debug, "EXEC", &format!("Entry: {}", action));
        }
        
        // Update state
        self.current_node = Some(next_node_id);
        self.step_count += 1;
        
        Ok(StepResult::Transitioned { 
            from: current_id, 
            to: next_node_id 
        })
    }
    
    /// Trigger an event to cause a transition
    pub fn trigger_event(&mut self, _event: &str) -> Result<(), String> {
        self.step().map(|_| ())
    }
    
    /// Get current simulation status
    pub fn status(&self) -> SimulationStatus {
        self.status
    }
    
    /// Get current node ID
    pub fn current_node(&self) -> Option<NodeId> {
        self.current_node
    }
    
    /// Get the context variables
    pub fn context(&self) -> &SimulationContext {
        &self.context
    }
    
    /// Set a context variable
    pub fn set_context(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.context.insert(key.into(), value);
    }
    
    /// Get logs
    pub fn logs(&self) -> &[LogEntry] {
        &self.logs
    }
    
    /// Clear logs
    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }
    
    /// Get step count
    pub fn step_count(&self) -> u64 {
        self.step_count
    }
    
    fn log(&mut self, level: LogLevel, source: &str, message: &str) {
        self.logs.push(LogEntry {
            timestamp: Utc::now(),
            level,
            source: source.to_string(),
            message: message.to_string(),
        });
    }
}

/// Result of a simulation step
#[derive(Debug, Clone)]
pub enum StepResult {
    Transitioned { from: NodeId, to: NodeId },
    Completed,
    Deadlock,
    Breakpoint(NodeId),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_executor_basic() {
        let mut graph = FSMGraph::new();
        
        let start = FSMNode::new("START", NodeType::Input);
        let end = FSMNode::new("END", NodeType::Output);
        
        let start_id = graph.add_node(start);
        let end_id = graph.add_node(end);
        
        graph.add_edge(FSMEdge::new(start_id, end_id).with_label("GO"));
        
        let mut executor = FSMExecutor::new(graph);
        
        assert!(executor.start().is_ok());
        assert_eq!(executor.status(), SimulationStatus::Running);
        
        let result = executor.step();
        assert!(result.is_ok());
        
        match result.unwrap() {
            StepResult::Transitioned { to, .. } => {
                assert_eq!(to, end_id);
            }
            _ => panic!("Expected transition"),
        }
    }
}
