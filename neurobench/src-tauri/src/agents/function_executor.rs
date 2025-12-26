//! Function Execution Engine
//!
//! Executes function calls from agents and returns results.
//! Bridges between AI function calls and actual system actions.

use super::function_calling::{FunctionCall, FunctionResult};
use serde_json::json;

/// Executes function calls from agents
/// Note: This is a simplified executor that returns instructions
/// for the frontend or other systems to execute.
pub struct FunctionExecutor;

impl FunctionExecutor {
    pub fn new() -> Self {
        Self
    }
    
    /// Execute a function call and return the result
    pub fn execute(&self, call: &FunctionCall) -> FunctionResult {
        match call.name.as_str() {
            "add_node" => self.add_node(call),
            "connect_nodes" => self.connect_nodes(call),
            "auto_layout" => self.auto_layout(call),
            "align_nodes" => self.align_nodes(call),
            "validate_fsm" => self.validate_fsm(call),
            "generate_code" => self.generate_code(call),
            "build_project" => self.build_project(call),
            "flash_device" => self.flash_device(call),
            "delegate" => self.delegate(call),
            "snap_to_grid" => self.snap_to_grid(call),
            _ => FunctionResult::error(&call.name, &format!("Unknown function: {}", call.name)),
        }
    }
    
    /// Add a node to the canvas
    fn add_node(&self, call: &FunctionCall) -> FunctionResult {
        let name = match call.get_string("name") {
            Some(n) => n,
            None => return FunctionResult::error("add_node", "Missing 'name' parameter"),
        };
        
        let node_type = call.get_string("node_type").unwrap_or_else(|| "state".to_string());
        let x = call.get_number("x").unwrap_or(100.0);
        let y = call.get_number("y").unwrap_or(100.0);
        
        // Return instruction for canvas command
        FunctionResult::success("add_node", json!({
            "action": "canvas_add_node",
            "params": {
                "name": name,
                "node_type": node_type,
                "x": x,
                "y": y
            }
        }))
    }
    
    /// Connect two nodes
    fn connect_nodes(&self, call: &FunctionCall) -> FunctionResult {
        let from_node = match call.get_string("from_node") {
            Some(n) => n,
            None => return FunctionResult::error("connect_nodes", "Missing 'from_node' parameter"),
        };
        
        let to_node = match call.get_string("to_node") {
            Some(n) => n,
            None => return FunctionResult::error("connect_nodes", "Missing 'to_node' parameter"),
        };
        
        let condition = call.get_string("condition");
        
        FunctionResult::success("connect_nodes", json!({
            "action": "canvas_connect",
            "params": {
                "from_node": from_node,
                "to_node": to_node,
                "condition": condition
            }
        }))
    }
    
    /// Apply automatic layout
    fn auto_layout(&self, call: &FunctionCall) -> FunctionResult {
        let algorithm = call.get_string("algorithm").unwrap_or_else(|| "hierarchical".to_string());
        
        FunctionResult::success("auto_layout", json!({
            "action": "canvas_auto_layout",
            "params": {
                "algorithm": algorithm
            }
        }))
    }
    
    /// Align selected nodes
    fn align_nodes(&self, call: &FunctionCall) -> FunctionResult {
        let alignment = call.get_string("alignment").unwrap_or_else(|| "left".to_string());
        
        FunctionResult::success("align_nodes", json!({
            "action": "canvas_align",
            "params": {
                "alignment": alignment
            }
        }))
    }
    
    /// Snap nodes to grid
    fn snap_to_grid(&self, call: &FunctionCall) -> FunctionResult {
        let grid_size = call.get_number("grid_size").unwrap_or(20.0);
        
        FunctionResult::success("snap_to_grid", json!({
            "action": "canvas_snap_grid",
            "params": {
                "grid_size": grid_size
            }
        }))
    }
    
    /// Validate FSM for errors
    fn validate_fsm(&self, call: &FunctionCall) -> FunctionResult {
        let check_cycles = call.get_bool("check_cycles").unwrap_or(true);
        let check_unreachable = call.get_bool("check_unreachable").unwrap_or(true);
        
        FunctionResult::success("validate_fsm", json!({
            "action": "canvas_validate",
            "params": {
                "check_cycles": check_cycles,
                "check_unreachable": check_unreachable
            }
        }))
    }
    
    /// Generate code (returns instruction to code agent)
    fn generate_code(&self, call: &FunctionCall) -> FunctionResult {
        let language = call.get_string("language").unwrap_or_else(|| "c".to_string());
        let style = call.get_string("style").unwrap_or_else(|| "switch_case".to_string());
        
        FunctionResult::success("generate_code", json!({
            "action": "delegate",
            "target_agent": "code",
            "params": {
                "language": language,
                "style": style
            }
        }))
    }
    
    /// Build project (triggers build agent)
    fn build_project(&self, call: &FunctionCall) -> FunctionResult {
        let target = call.get_string("target");
        let release = call.get_bool("release").unwrap_or(false);
        
        FunctionResult::success("build_project", json!({
            "action": "delegate",
            "target_agent": "build",
            "params": {
                "target": target,
                "release": release
            }
        }))
    }
    
    /// Flash device (triggers deploy agent)
    fn flash_device(&self, call: &FunctionCall) -> FunctionResult {
        let verify = call.get_bool("verify").unwrap_or(true);
        
        FunctionResult::success("flash_device", json!({
            "action": "delegate",
            "target_agent": "deploy",
            "params": {
                "verify": verify
            }
        }))
    }
    
    /// Delegate to another agent
    fn delegate(&self, call: &FunctionCall) -> FunctionResult {
        let agent_id = match call.get_string("agent_id") {
            Some(id) => id,
            None => return FunctionResult::error("delegate", "Missing 'agent_id' parameter"),
        };
        
        let task = match call.get_string("task") {
            Some(t) => t,
            None => return FunctionResult::error("delegate", "Missing 'task' parameter"),
        };
        
        FunctionResult::success("delegate", json!({
            "action": "delegate",
            "target_agent": agent_id,
            "task": task
        }))
    }
}

impl Default for FunctionExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_node() {
        let executor = FunctionExecutor::new();
        let call = FunctionCall {
            name: "add_node".to_string(),
            arguments: serde_json::json!({
                "name": "Start",
                "node_type": "state"
            }),
        };
        
        let result = executor.execute(&call);
        assert!(result.success);
    }
    
    #[test]
    fn test_unknown_function() {
        let executor = FunctionExecutor::new();
        let call = FunctionCall {
            name: "unknown".to_string(),
            arguments: serde_json::json!({}),
        };
        
        let result = executor.execute(&call);
        assert!(!result.success);
    }
}
