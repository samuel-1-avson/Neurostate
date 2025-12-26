//! Build Agent - Intelligent build system management
//!
//! Handles:
//! - Build triggers and monitoring
//! - Error analysis and fixing
//! - Optimization suggestions
//! - Toolchain configuration

use super::{Agent, AgentInfo, AgentContext, AgentCapabilities, AgentResponse, ToolCall};
use crate::ai::AgentModelConfig;
use async_trait::async_trait;

/// Build Agent for compilation management
pub struct BuildAgent {
    model_config: AgentModelConfig,
}

impl BuildAgent {
    pub fn new() -> Self {
        Self {
            model_config: AgentModelConfig::build(),
        }
    }
}

impl Default for BuildAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for BuildAgent {
    fn info(&self) -> AgentInfo {
        AgentInfo {
            id: "build".to_string(),
            name: "Build Agent".to_string(),
            description: "Intelligent build system management, error analysis, and optimization".to_string(),
            icon: "🔨".to_string(),
            capabilities: AgentCapabilities {
                can_edit_fsm: false,
                can_generate_code: false,
                can_execute_terminal: true,
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
        let mut tool_calls = Vec::new();
        let response_message: String;
        
        if message_lower.contains("build") || message_lower.contains("compile") {
            tool_calls.push(ToolCall {
                tool: "build_project".to_string(),
                params: serde_json::json!({
                    "target": context.mcu.target,
                    "release": message_lower.contains("release"),
                }),
            });
            
            response_message = format!(
                "Starting build for {} target. I'll monitor the output and help with any errors.",
                context.mcu.target
            );
        } else if message_lower.contains("error") || message_lower.contains("fix") {
            tool_calls.push(ToolCall {
                tool: "analyze_build_error".to_string(),
                params: serde_json::json!({}),
            });
            
            response_message = "I'll analyze the build errors and suggest fixes.".to_string();
        } else if message_lower.contains("clean") {
            tool_calls.push(ToolCall {
                tool: "clean_build".to_string(),
                params: serde_json::json!({}),
            });
            
            response_message = "Cleaning build artifacts...".to_string();
        } else if message_lower.contains("optimize") || message_lower.contains("size") {
            tool_calls.push(ToolCall {
                tool: "analyze_binary".to_string(),
                params: serde_json::json!({
                    "focus": if message_lower.contains("size") { "size" } else { "speed" },
                }),
            });
            
            response_message = "I'll analyze the binary and suggest optimizations.".to_string();
        } else if message_lower.contains("toolchain") || message_lower.contains("configure") {
            tool_calls.push(ToolCall {
                tool: "configure_toolchain".to_string(),
                params: serde_json::json!({
                    "target": context.mcu.target,
                }),
            });
            
            response_message = format!(
                "I'll help configure the toolchain for {} target.",
                context.mcu.target
            );
        } else {
            response_message = format!(
                "I can help you with builds. Current target: {}. \
                Try asking me to:\n\
                - Build the project\n\
                - Fix build errors\n\
                - Clean build artifacts\n\
                - Optimize binary size\n\
                - Configure toolchain",
                context.mcu.target
            );
        }
        
        Ok(AgentResponse {
            message: response_message,
            tool_calls,
            suggestions: vec![
                "Build project".to_string(),
                "Clean and rebuild".to_string(),
                "Optimize for size".to_string(),
            ],
        })
    }
    
    fn system_prompt(&self) -> String {
        r#"You are the Build Agent for NeuroBench, specializing in embedded systems compilation.

Your capabilities:
1. Build management: Trigger, monitor, and manage builds
2. Error analysis: Parse compiler errors and suggest fixes
3. Optimization: Analyze binary and suggest size/speed optimizations
4. Toolchain: Configure and verify compiler toolchains

Available tools:
- build_project: Trigger a build for the target MCU
- analyze_build_error: Parse and explain compiler errors
- clean_build: Remove build artifacts
- analyze_binary: Analyze compiled binary for optimizations
- configure_toolchain: Set up compiler toolchain

Always provide clear build status and actionable error fixes.
"#.to_string()
    }
    
    fn can_handle(&self, request_type: &str) -> bool {
        matches!(
            request_type.to_lowercase().as_str(),
            "build" | "compile" | "make" | "cmake" | "toolchain"
        )
    }
}
