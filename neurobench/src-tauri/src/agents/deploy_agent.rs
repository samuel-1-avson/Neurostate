//! Deploy Agent - Device flashing and deployment
//!
//! Handles:
//! - Firmware flashing
//! - Device verification
//! - Serial monitoring
//! - OTA updates

use super::{Agent, AgentInfo, AgentContext, AgentCapabilities, AgentResponse, ToolCall};
use crate::ai::AgentModelConfig;
use async_trait::async_trait;

/// Deploy Agent for device flashing and deployment
pub struct DeployAgent {
    model_config: AgentModelConfig,
}

impl DeployAgent {
    pub fn new() -> Self {
        Self {
            model_config: AgentModelConfig::deploy(),
        }
    }
}

impl Default for DeployAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for DeployAgent {
    fn info(&self) -> AgentInfo {
        AgentInfo {
            id: "deploy".to_string(),
            name: "Deploy Agent".to_string(),
            description: "Device flashing, deployment, and serial monitoring".to_string(),
            icon: "🚀".to_string(),
            capabilities: AgentCapabilities {
                can_edit_fsm: false,
                can_generate_code: false,
                can_execute_terminal: true,
                can_access_hardware: true,
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
        
        if message_lower.contains("flash") || message_lower.contains("upload") {
            tool_calls.push(ToolCall {
                tool: "flash_device".to_string(),
                params: serde_json::json!({
                    "target": context.mcu.target,
                    "verify": true,
                }),
            });
            
            response_message = format!(
                "Flashing firmware to {} device. I'll verify the upload when complete.",
                context.mcu.target
            );
        } else if message_lower.contains("monitor") || message_lower.contains("serial") {
            tool_calls.push(ToolCall {
                tool: "serial_monitor".to_string(),
                params: serde_json::json!({
                    "baud_rate": 115200,
                }),
            });
            
            response_message = "Opening serial monitor at 115200 baud...".to_string();
        } else if message_lower.contains("reset") || message_lower.contains("reboot") {
            tool_calls.push(ToolCall {
                tool: "reset_device".to_string(),
                params: serde_json::json!({}),
            });
            
            response_message = "Resetting device...".to_string();
        } else if message_lower.contains("verify") || message_lower.contains("check") {
            tool_calls.push(ToolCall {
                tool: "verify_flash".to_string(),
                params: serde_json::json!({}),
            });
            
            response_message = "Verifying flash memory contents...".to_string();
        } else if message_lower.contains("connect") || message_lower.contains("detect") {
            tool_calls.push(ToolCall {
                tool: "detect_devices".to_string(),
                params: serde_json::json!({}),
            });
            
            response_message = "Scanning for connected devices...".to_string();
        } else {
            response_message = format!(
                "I can help you deploy to your {} device. \
                Try asking me to:\n\
                - Flash the firmware\n\
                - Open serial monitor\n\
                - Reset the device\n\
                - Verify flash\n\
                - Detect connected devices",
                context.mcu.target
            );
        }
        
        Ok(AgentResponse {
            message: response_message,
            tool_calls,
            suggestions: vec![
                "Flash firmware".to_string(),
                "Open serial monitor".to_string(),
                "Reset device".to_string(),
            ],
        })
    }
    
    fn system_prompt(&self) -> String {
        r#"You are the Deploy Agent for NeuroBench, specializing in firmware deployment.

Your capabilities:
1. Flashing: Upload firmware to embedded devices
2. Verification: Verify flash memory contents
3. Monitoring: Serial port monitoring and logging
4. Device control: Reset, detect, and manage devices

Available tools:
- flash_device: Flash compiled firmware to device
- verify_flash: Verify flash memory matches binary
- serial_monitor: Open serial port monitor
- reset_device: Trigger device reset
- detect_devices: Scan for connected debug probes/devices

Always confirm successful operations and report any errors clearly.
"#.to_string()
    }
    
    fn can_handle(&self, request_type: &str) -> bool {
        matches!(
            request_type.to_lowercase().as_str(),
            "flash" | "upload" | "deploy" | "serial" | "monitor"
        )
    }
}
