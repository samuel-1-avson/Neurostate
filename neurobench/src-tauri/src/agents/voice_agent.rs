//! Voice Assistant Agent
//!
//! A conversational AI agent powered by Grok that:
//! - Has natural conversations about what you're building
//! - Listens to voice commands and ideas
//! - Delegates work to specialized agents
//! - Provides friendly, helpful responses

use super::{Agent, AgentContext, AgentResponse, ToolCall};
use crate::ai::providers::{ModelConfig, ModelProvider, OpenAIModel, AIModel};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Voice Assistant Agent - Conversational interface powered by Grok
pub struct VoiceAssistantAgent {
    model: OpenAIModel,  // Grok uses OpenAI-compatible API
}

impl VoiceAssistantAgent {
    pub fn new() -> Self {
        let api_key = std::env::var("GROK_API_KEY").ok();
        
        Self {
            model: OpenAIModel::new(ModelConfig {
                provider: ModelProvider::Custom,
                model_name: "grok-2".to_string(),
                api_key,
                base_url: Some("https://api.x.ai/v1".to_string()),
                temperature: 0.8,  // More creative for conversation
                max_tokens: 4096,
                timeout_secs: 60,
            }),
        }
    }
    
    /// Build the conversational prompt
    fn build_prompt(&self, message: &str, context: &AgentContext) -> String {
        let selected = context.selected_node.as_deref().unwrap_or("None");
        
        format!(r#"You are the Voice Assistant for NeuroBench, an embedded systems development workbench.

PERSONALITY:
- Friendly, helpful, and conversational
- Enthusiastic about embedded systems and state machines
- Explain technical concepts in simple terms when asked
- Remember context from the conversation

YOUR CAPABILITIES:
- Have natural conversations about the user's project
- Listen to ideas and help refine them
- Delegate tasks to specialized agents when needed
- Provide guidance on FSM design, code generation, and hardware

AVAILABLE AGENTS TO DELEGATE TO:
- director: Orchestrates complex multi-step tasks
- code: Generates and optimizes code
- debug: Analyzes errors and suggests fixes
- hardware: Configures MCU peripherals
- fsm: Designs state machines
- canvas: Manipulates the FSM canvas
- build: Compiles projects
- deploy: Flashes firmware to devices

CURRENT PROJECT CONTEXT:
- Project: {} ({})
- MCU Target: {}
- States: {}
- Selected Node: {}

When the user wants something done, respond conversationally AND include a DELEGATE block:
```delegate
agent: <agent_id>
task: <clear description of what to do>
```

USER MESSAGE: {}

Respond conversationally and helpfully. If the user wants action taken, include the delegate block."#,
            context.project.name,
            context.project.language,
            context.mcu.target,
            context.nodes.len(),
            selected,
            message
        )
    }
    
    /// Parse delegate blocks from response
    fn parse_delegates(&self, response: &str) -> Vec<ToolCall> {
        let mut tool_calls = Vec::new();
        
        // Find delegate blocks
        let re = regex::Regex::new(r"```delegate\s*\n([\s\S]*?)\n```").unwrap();
        
        for cap in re.captures_iter(response) {
            if let Some(block) = cap.get(1) {
                let block_text = block.as_str();
                
                let mut agent = String::new();
                let mut task = String::new();
                
                for line in block_text.lines() {
                    if line.starts_with("agent:") {
                        agent = line.trim_start_matches("agent:").trim().to_string();
                    } else if line.starts_with("task:") {
                        task = line.trim_start_matches("task:").trim().to_string();
                    }
                }
                
                if !agent.is_empty() && !task.is_empty() {
                    tool_calls.push(ToolCall {
                        tool: "delegate".to_string(),
                        params: serde_json::json!({
                            "target_agent": agent,
                            "task": task
                        }),
                    });
                }
            }
        }
        
        tool_calls
    }
    
    /// Clean response (remove delegate blocks for display)
    fn clean_response(&self, response: &str) -> String {
        let re = regex::Regex::new(r"```delegate[\s\S]*?```").unwrap();
        re.replace_all(response, "").trim().to_string()
    }
}

impl Default for VoiceAssistantAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for VoiceAssistantAgent {
    fn info(&self) -> super::AgentInfo {
        super::AgentInfo {
            id: "voice".to_string(),
            name: "Voice Assistant".to_string(),
            description: "Conversational AI assistant powered by Grok for voice commands and idea refinement".to_string(),
            icon: "🎙️".to_string(),
            capabilities: super::AgentCapabilities {
                can_edit_fsm: false,
                can_generate_code: false,
                can_execute_terminal: false,
                can_access_hardware: false,
            },
        }
    }
    
    fn system_prompt(&self) -> String {
        r#"You are the Voice Assistant for NeuroBench, an embedded systems development workbench.

PERSONALITY:
- Friendly, helpful, and conversational
- Enthusiastic about embedded systems and state machines
- Explain technical concepts in simple terms when asked

YOUR CAPABILITIES:
- Have natural conversations about the user's project
- Listen to ideas and help refine them
- Delegate tasks to specialized agents
- Provide guidance on FSM design, code generation, and hardware

AVAILABLE AGENTS TO DELEGATE TO:
- director: Orchestrates complex multi-step tasks
- code: Generates and optimizes code
- debug: Analyzes errors and suggests fixes
- hardware: Configures MCU peripherals
- fsm: Designs state machines
- canvas: Manipulates the FSM canvas
- build: Compiles projects
- deploy: Flashes firmware to devices

When the user wants something done, respond conversationally AND include a DELEGATE block:
```delegate
agent: <agent_id>
task: <clear description>
```"#.to_string()
    }
    
    fn can_handle(&self, request_type: &str) -> bool {
        matches!(request_type, 
            "conversation" | "voice_command" | "question" | 
            "idea" | "help" | "delegate"
        )
    }
    
    async fn process(&self, message: &str, context: &AgentContext) -> Result<AgentResponse, String> {
        let prompt = self.build_prompt(message, context);
        
        let model_response = self.model.generate(&prompt).await
            .map_err(|e| format!("Voice Assistant error: {}", e))?;
        
        let response_text = &model_response.content;
        
        let tool_calls = self.parse_delegates(response_text);
        let clean_message = self.clean_response(response_text);
        
        // Generate contextual suggestions
        let suggestions = if tool_calls.is_empty() {
            vec![
                "Tell me more about your idea".to_string(),
                "Would you like me to create that?".to_string(),
                "Let me know when you're ready to build".to_string(),
            ]
        } else {
            vec![
                "That's in progress!".to_string(),
                "What else would you like to do?".to_string(),
            ]
        };
        
        Ok(AgentResponse {
            message: clean_message,
            tool_calls,
            suggestions,
        })
    }
}

/// Voice command types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoiceCommand {
    /// Create something (node, FSM, code)
    Create { what: String, details: Option<String> },
    /// Modify existing element
    Modify { target: String, how: String },
    /// Ask a question
    Question { question: String },
    /// Execute an action
    Execute { action: String },
    /// Navigate the UI
    Navigate { destination: String },
    /// General conversation
    Conversation { message: String },
}

impl VoiceCommand {
    /// Parse a voice command from natural language
    pub fn parse(input: &str) -> Self {
        let input_lower = input.to_lowercase();
        
        // Create patterns
        if input_lower.starts_with("create") || 
           input_lower.starts_with("add") ||
           input_lower.starts_with("make") ||
           input_lower.starts_with("new") {
            return Self::Create {
                what: input.to_string(),
                details: None,
            };
        }
        
        // Modify patterns
        if input_lower.starts_with("change") ||
           input_lower.starts_with("update") ||
           input_lower.starts_with("modify") ||
           input_lower.starts_with("rename") {
            return Self::Modify {
                target: input.to_string(),
                how: String::new(),
            };
        }
        
        // Question patterns
        if input_lower.starts_with("what") ||
           input_lower.starts_with("how") ||
           input_lower.starts_with("why") ||
           input_lower.starts_with("can you") ||
           input_lower.ends_with("?") {
            return Self::Question {
                question: input.to_string(),
            };
        }
        
        // Execute patterns
        if input_lower.starts_with("run") ||
           input_lower.starts_with("build") ||
           input_lower.starts_with("flash") ||
           input_lower.starts_with("compile") ||
           input_lower.starts_with("generate") {
            return Self::Execute {
                action: input.to_string(),
            };
        }
        
        // Navigate patterns
        if input_lower.starts_with("go to") ||
           input_lower.starts_with("show") ||
           input_lower.starts_with("open") ||
           input_lower.starts_with("switch to") {
            return Self::Navigate {
                destination: input.to_string(),
            };
        }
        
        // Default to conversation
        Self::Conversation {
            message: input.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_voice_command_parsing() {
        let cmd = VoiceCommand::parse("Create a motor controller FSM");
        assert!(matches!(cmd, VoiceCommand::Create { .. }));
        
        let cmd = VoiceCommand::parse("What is the best way to handle errors?");
        assert!(matches!(cmd, VoiceCommand::Question { .. }));
        
        let cmd = VoiceCommand::parse("Build the project");
        assert!(matches!(cmd, VoiceCommand::Execute { .. }));
    }
}
