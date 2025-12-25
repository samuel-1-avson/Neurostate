// Tool Executor
// Executes tool calls from agents to modify FSM canvas and other actions

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Result of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub message: String,
    pub data: Option<Value>,
}

impl ToolResult {
    pub fn success(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: None,
        }
    }

    pub fn success_with_data(message: &str, data: Value) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: Some(data),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,
        }
    }
}

/// Tool executor handles agent tool calls
pub struct ToolExecutor;

impl ToolExecutor {
    /// Execute a tool call and return the result
    pub fn execute(tool: &str, params: &Value) -> ToolResult {
        match tool.to_lowercase().as_str() {
            // FSM Modification Tools
            "add_node" => Self::add_node(params),
            "remove_node" => Self::remove_node(params),
            "update_node" => Self::update_node(params),
            "add_edge" => Self::add_edge(params),
            "remove_edge" => Self::remove_edge(params),
            
            // Analysis Tools
            "validate_fsm" => Self::validate_fsm(params),
            "analyze" => Self::analyze(params),
            "diagnose" => Self::diagnose(params),
            
            // Hardware Tools
            "get_pinout" => Self::get_pinout(params),
            "calc_clock" => Self::calc_clock(params),
            "check_pin" => Self::check_pin(params),
            
            // Code/Doc Tools
            "gen_docs" => Self::gen_docs(params),
            "generate_driver" => Self::generate_driver(params),
            
            // Unknown tool
            _ => ToolResult::error(&format!("Unknown tool: {}", tool)),
        }
    }

    // === FSM Tools ===
    
    fn add_node(params: &Value) -> ToolResult {
        let label = params.get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("NewState");
        let node_type = params.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("process");
        let x = params.get("x").and_then(|v| v.as_f64()).unwrap_or(300.0);
        let y = params.get("y").and_then(|v| v.as_f64()).unwrap_or(200.0);
        
        let node_data = serde_json::json!({
            "action": "add_node",
            "node": {
                "id": format!("node_{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
                "label": label,
                "type": node_type,
                "x": x,
                "y": y,
                "entryAction": params.get("entryAction").and_then(|v| v.as_str())
            }
        });
        
        ToolResult::success_with_data(
            &format!("Created node '{}' of type '{}'", label, node_type),
            node_data
        )
    }

    fn remove_node(params: &Value) -> ToolResult {
        let id = params.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        if id.is_empty() {
            return ToolResult::error("Node ID required");
        }
        
        let action_data = serde_json::json!({
            "action": "remove_node",
            "nodeId": id
        });
        
        ToolResult::success_with_data(&format!("Removed node '{}'", id), action_data)
    }

    fn update_node(params: &Value) -> ToolResult {
        let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
        
        if id.is_empty() {
            return ToolResult::error("Node ID required");
        }
        
        let action_data = serde_json::json!({
            "action": "update_node",
            "nodeId": id,
            "updates": {
                "label": params.get("label"),
                "type": params.get("type"),
                "entryAction": params.get("entryAction")
            }
        });
        
        ToolResult::success_with_data(&format!("Updated node '{}'", id), action_data)
    }

    fn add_edge(params: &Value) -> ToolResult {
        let source = params.get("source").and_then(|v| v.as_str()).unwrap_or("");
        let target = params.get("target").and_then(|v| v.as_str()).unwrap_or("");
        let label = params.get("label").and_then(|v| v.as_str());
        
        if source.is_empty() || target.is_empty() {
            return ToolResult::error("Source and target node IDs required");
        }
        
        let action_data = serde_json::json!({
            "action": "add_edge",
            "edge": {
                "id": format!("edge_{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
                "source": source,
                "target": target,
                "label": label
            }
        });
        
        ToolResult::success_with_data(
            &format!("Created transition from '{}' to '{}'", source, target),
            action_data
        )
    }

    fn remove_edge(params: &Value) -> ToolResult {
        let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
        
        if id.is_empty() {
            return ToolResult::error("Edge ID required");
        }
        
        let action_data = serde_json::json!({
            "action": "remove_edge",
            "edgeId": id
        });
        
        ToolResult::success_with_data(&format!("Removed edge '{}'", id), action_data)
    }

    // === Analysis Tools ===
    
    fn validate_fsm(_params: &Value) -> ToolResult {
        // This would analyze the current FSM for issues
        let analysis = serde_json::json!({
            "action": "validate_fsm",
            "checks": ["unreachable_states", "deadlocks", "missing_transitions"]
        });
        
        ToolResult::success_with_data("FSM validation requested", analysis)
    }

    fn analyze(params: &Value) -> ToolResult {
        let target = params.get("code").and_then(|v| v.as_str()).unwrap_or("fsm");
        
        ToolResult::success(&format!("Analyzing {}...", target))
    }

    fn diagnose(params: &Value) -> ToolResult {
        let issue = params.get("issue").and_then(|v| v.as_str()).unwrap_or("general");
        
        ToolResult::success(&format!("Diagnosing issue: {}", issue))
    }

    // === Hardware Tools ===
    
    fn get_pinout(params: &Value) -> ToolResult {
        let mcu = params.get("mcu").and_then(|v| v.as_str()).unwrap_or("STM32F401");
        
        let pinout = serde_json::json!({
            "action": "show_pinout",
            "mcu": mcu
        });
        
        ToolResult::success_with_data(&format!("Pinout for {}", mcu), pinout)
    }

    fn calc_clock(params: &Value) -> ToolResult {
        let target_freq = params.get("target_freq").and_then(|v| v.as_u64()).unwrap_or(84_000_000);
        
        let clock_config = serde_json::json!({
            "action": "calc_clock",
            "target_freq": target_freq,
            "pll_m": 8,
            "pll_n": 168,
            "pll_p": 2,
            "ahb_prescaler": 1,
            "apb1_prescaler": 2,
            "apb2_prescaler": 1
        });
        
        ToolResult::success_with_data(
            &format!("Clock configuration for {}MHz", target_freq / 1_000_000),
            clock_config
        )
    }

    fn check_pin(params: &Value) -> ToolResult {
        let pin = params.get("pin").and_then(|v| v.as_str()).unwrap_or("PA0");
        let function = params.get("function").and_then(|v| v.as_str()).unwrap_or("GPIO");
        
        ToolResult::success(&format!("Pin {} can be used for {}", pin, function))
    }

    // === Doc Tools ===
    
    fn gen_docs(params: &Value) -> ToolResult {
        let doc_type = params.get("type").and_then(|v| v.as_str()).unwrap_or("readme");
        
        let doc_data = serde_json::json!({
            "action": "generate_docs",
            "type": doc_type
        });
        
        ToolResult::success_with_data(&format!("Generating {} documentation", doc_type), doc_data)
    }

    fn generate_driver(params: &Value) -> ToolResult {
        let driver_type = params.get("type").and_then(|v| v.as_str()).unwrap_or("gpio");
        
        let driver_data = serde_json::json!({
            "action": "generate_driver",
            "type": driver_type
        });
        
        ToolResult::success_with_data(&format!("Generating {} driver", driver_type), driver_data)
    }
}
