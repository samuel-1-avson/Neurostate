//! Canvas Agent - AI-powered canvas manipulation
//!
//! Handles intelligent canvas operations:
//! - Auto-layout algorithms
//! - Node suggestions
//! - FSM generation from description
//! - Visual organization

use super::{Agent, AgentInfo, AgentContext, AgentCapabilities, AgentResponse, ToolCall};
use crate::ai::{AgentModelConfig, AgentModelRegistry};
use async_trait::async_trait;

/// Canvas Agent for intelligent canvas operations
pub struct CanvasAgent {
    model_config: AgentModelConfig,
}

impl CanvasAgent {
    pub fn new() -> Self {
        Self {
            model_config: AgentModelConfig::canvas(),
        }
    }
}

impl Default for CanvasAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for CanvasAgent {
    fn info(&self) -> AgentInfo {
        AgentInfo {
            id: "canvas".to_string(),
            name: "Canvas Agent".to_string(),
            description: "AI-powered canvas manipulation, layout, and visual organization".to_string(),
            icon: "🎨".to_string(),
            capabilities: AgentCapabilities {
                can_edit_fsm: true,
                can_generate_code: false,
                can_execute_terminal: false,
                can_access_hardware: false,
            },
        }
    }
    
    async fn process(
        &self,
        message: &str,
        context: &AgentContext,
    ) -> Result<AgentResponse, String> {
        let message_lower = message.to_lowercase();
        
        // Determine what canvas action to take
        let mut tool_calls = Vec::new();
        let mut response_message = String::new();
        
        if message_lower.contains("layout") || message_lower.contains("arrange") {
            // Determine best layout algorithm based on context
            let algorithm = if context.nodes.len() > 10 {
                "hierarchical"
            } else if context.edges.len() > context.nodes.len() * 2 {
                "force_directed"
            } else {
                "grid"
            };
            
            tool_calls.push(ToolCall {
                tool: "auto_layout".to_string(),
                params: serde_json::json!({
                    "algorithm": algorithm,
                }),
            });
            
            response_message = format!(
                "I'll apply a {} layout to organize your {} nodes optimally.",
                algorithm, context.nodes.len()
            );
        } else if message_lower.contains("align") {
            let alignment = if message_lower.contains("left") {
                "left"
            } else if message_lower.contains("right") {
                "right"
            } else if message_lower.contains("top") {
                "top"
            } else if message_lower.contains("bottom") {
                "bottom"
            } else {
                "center_horizontal"
            };
            
            tool_calls.push(ToolCall {
                tool: "align_nodes".to_string(),
                params: serde_json::json!({
                    "alignment": alignment,
                }),
            });
            
            response_message = format!("Aligning nodes to {}.", alignment);
        } else if message_lower.contains("add") || message_lower.contains("create") {
            // Suggest node creation
            let node_type = if message_lower.contains("input") {
                "input"
            } else if message_lower.contains("output") {
                "output"
            } else if message_lower.contains("error") {
                "error"
            } else if message_lower.contains("decision") || message_lower.contains("check") {
                "decision"
            } else {
                "process"
            };
            
            tool_calls.push(ToolCall {
                tool: "add_node".to_string(),
                params: serde_json::json!({
                    "node_type": node_type,
                    "suggested": true,
                }),
            });
            
            response_message = format!("I'll add a {} node. You can position it on the canvas.", node_type);
        } else if message_lower.contains("connect") || message_lower.contains("link") {
            tool_calls.push(ToolCall {
                tool: "suggest_connections".to_string(),
                params: serde_json::json!({}),
            });
            
            response_message = "I'll analyze the FSM and suggest logical connections.".to_string();
        } else if message_lower.contains("clean") || message_lower.contains("organize") {
            tool_calls.push(ToolCall {
                tool: "auto_layout".to_string(),
                params: serde_json::json!({
                    "algorithm": "hierarchical",
                }),
            });
            tool_calls.push(ToolCall {
                tool: "validate".to_string(),
                params: serde_json::json!({}),
            });
            
            response_message = "I'll clean up the canvas with an optimal layout and validate the FSM.".to_string();
        } else {
            response_message = format!(
                "I can help you with canvas operations. Your current FSM has {} nodes and {} edges. \
                Try asking me to:\n\
                - Layout the canvas\n\
                - Align nodes\n\
                - Add a new node\n\
                - Suggest connections\n\
                - Clean up the design",
                context.nodes.len(),
                context.edges.len()
            );
        }
        
        Ok(AgentResponse {
            message: response_message,
            tool_calls,
            suggestions: vec![
                "Apply hierarchical layout".to_string(),
                "Align selected nodes".to_string(),
                "Validate connections".to_string(),
            ],
        })
    }
    
    fn system_prompt(&self) -> String {
        r#"You are the Canvas Agent for NeuroBench, specializing in visual FSM manipulation.

Your capabilities:
1. Auto-layout: Arrange nodes using hierarchical, force-directed, or grid algorithms
2. Alignment: Align nodes left, right, top, bottom, or center
3. Node operations: Add, position, and organize nodes
4. Connection suggestions: Analyze FSM and suggest logical transitions
5. Visual organization: Clean up and optimize canvas layout

Available tools:
- auto_layout: Apply layout algorithm (hierarchical, force_directed, grid)
- align_nodes: Align selected nodes (left, right, top, bottom, center)
- add_node: Add a new node of specified type
- suggest_connections: Analyze and suggest logical connections
- validate: Check FSM for errors

Always provide clear, actionable responses about canvas operations.
"#.to_string()
    }
    
    fn can_handle(&self, request_type: &str) -> bool {
        matches!(
            request_type.to_lowercase().as_str(),
            "layout" | "align" | "canvas" | "arrange" | "organize" | "position" | "visual"
        )
    }
}
