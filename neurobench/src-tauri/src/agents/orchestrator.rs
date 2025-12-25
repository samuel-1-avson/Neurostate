// Agent Orchestrator
// Routes requests to agents and manages execution

use super::{AgentContext, AgentInfo, AgentResponse, AgentRegistry};
use crate::ai::AIService;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Orchestrator manages agent routing and execution
pub struct Orchestrator {
    registry: AgentRegistry,
    context: Arc<RwLock<AgentContext>>,
    ai_service: AIService,
    active_agent: Option<String>,
}

impl Orchestrator {
    pub fn new() -> Self {
        let mut registry = AgentRegistry::new();
        
        // Register built-in agents
        registry.register(Box::new(super::fsm_agent::FsmAgent::new()));
        registry.register(Box::new(super::code_agent::CodeAgent::new()));
        registry.register(Box::new(super::debug_agent::DebugAgent::new()));
        registry.register(Box::new(super::hardware_agent::HardwareAgent::new()));
        registry.register(Box::new(super::docs_agent::DocsAgent::new()));
        
        Self {
            registry,
            context: Arc::new(RwLock::new(AgentContext::default())),
            ai_service: AIService::new(),
            active_agent: Some("fsm".to_string()),
        }
    }
    
    /// Get list of available agents
    pub fn list_agents(&self) -> Vec<AgentInfo> {
        self.registry.list()
    }
    
    /// Set active agent
    pub fn set_active_agent(&mut self, agent_id: &str) -> Result<(), String> {
        if self.registry.get(agent_id).is_some() {
            self.active_agent = Some(agent_id.to_string());
            Ok(())
        } else {
            Err(format!("Agent '{}' not found", agent_id))
        }
    }
    
    /// Get active agent info
    pub fn get_active_agent(&self) -> Option<AgentInfo> {
        self.active_agent.as_ref()
            .and_then(|id| self.registry.get(id))
            .map(|a| a.info())
    }
    
    /// Process a message with the active agent
    pub async fn process(&self, message: &str) -> Result<AgentResponse, String> {
        let agent_id = self.active_agent.as_ref()
            .ok_or_else(|| "No active agent".to_string())?;
        
        let agent = self.registry.get(agent_id)
            .ok_or_else(|| format!("Agent '{}' not found", agent_id))?;
        
        // Get context
        let mut context = self.context.write().await;
        context.add_user_message(message);
        
        // Build prompt with system prompt and context
        let system_prompt = agent.system_prompt();
        let context_str = context.to_prompt_context();
        
        let full_prompt = format!(
            "{}\n\n## User Request:\n{}",
            system_prompt,
            message
        );
        
        // Call AI service with context
        let response = self.ai_service.chat(&full_prompt, Some(&context_str)).await?;
        
        // Parse response for tool calls
        let agent_response = self.parse_response(&response);
        
        // Add to conversation history
        context.add_assistant_message(&agent_response.message);
        
        Ok(agent_response)
    }
    
    /// Parse LLM response for tool calls and suggestions
    fn parse_response(&self, response: &str) -> AgentResponse {
        let mut tool_calls = Vec::new();
        let mut suggestions = Vec::new();
        let mut message = response.to_string();
        
        // Look for tool call patterns: [TOOL:name:params]
        let tool_regex = regex::Regex::new(r"\[TOOL:(\w+):([^\]]+)\]").ok();
        if let Some(re) = tool_regex {
            for cap in re.captures_iter(response) {
                let tool_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let params_str = cap.get(2).map(|m| m.as_str()).unwrap_or("{}");
                
                if let Ok(params) = serde_json::from_str(params_str) {
                    tool_calls.push(super::ToolCall {
                        tool: tool_name.to_string(),
                        params,
                    });
                }
            }
            // Remove tool calls from message
            message = re.replace_all(&message, "").to_string();
        }
        
        // Look for suggestion patterns: [SUGGEST:text]
        let suggest_regex = regex::Regex::new(r"\[SUGGEST:([^\]]+)\]").ok();
        if let Some(re) = suggest_regex {
            for cap in re.captures_iter(response) {
                if let Some(suggestion) = cap.get(1) {
                    suggestions.push(suggestion.as_str().to_string());
                }
            }
            message = re.replace_all(&message, "").to_string();
        }
        
        AgentResponse {
            message: message.trim().to_string(),
            tool_calls,
            suggestions,
        }
    }
    
    /// Update context with new FSM data
    pub async fn update_fsm(&self, nodes: Vec<super::ContextNode>, edges: Vec<super::ContextEdge>) {
        let mut context = self.context.write().await;
        context.nodes = nodes;
        context.edges = edges;
    }
    
    /// Update selected node
    pub async fn set_selected_node(&self, node_id: Option<String>) {
        let mut context = self.context.write().await;
        context.selected_node = node_id;
    }
    
    /// Update MCU target
    pub async fn set_mcu(&self, target: &str) {
        let mut context = self.context.write().await;
        context.mcu.target = target.to_string();
    }
    
    /// Update FSM context with current canvas state
    pub async fn update_context(
        &mut self,
        nodes: Vec<super::context::ContextNode>,
        edges: Vec<super::context::ContextEdge>,
        selected_node: Option<String>,
    ) {
        let mut context = self.context.write().await;
        context.nodes = nodes;
        context.edges = edges;
        context.selected_node = selected_node;
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}
