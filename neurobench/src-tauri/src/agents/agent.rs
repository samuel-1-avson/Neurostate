// Agent Trait and Types
// Core abstractions for AI agents

use serde::{Deserialize, Serialize};
use async_trait::async_trait;

/// Agent capability flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    pub can_edit_fsm: bool,
    pub can_generate_code: bool,
    pub can_execute_terminal: bool,
    pub can_access_hardware: bool,
}

/// Agent metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub capabilities: AgentCapabilities,
}

/// Tool call that an agent can request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    pub params: serde_json::Value,
}

/// Agent response with optional tool calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub message: String,
    pub tool_calls: Vec<ToolCall>,
    pub suggestions: Vec<String>,
}

/// Agent trait - all agents must implement this
#[async_trait]
pub trait Agent: Send + Sync {
    /// Get agent metadata
    fn info(&self) -> AgentInfo;
    
    /// Process a user message with context
    async fn process(
        &self,
        message: &str,
        context: &super::AgentContext,
    ) -> Result<AgentResponse, String>;
    
    /// Get system prompt for this agent
    fn system_prompt(&self) -> String;
    
    /// Check if agent can handle a specific request type
    fn can_handle(&self, request_type: &str) -> bool;
}

/// Agent registry entry
pub struct RegisteredAgent {
    pub agent: Box<dyn Agent>,
    pub enabled: bool,
}

/// Agent registry - holds all available agents
pub struct AgentRegistry {
    agents: std::collections::HashMap<String, RegisteredAgent>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: std::collections::HashMap::new(),
        }
    }
    
    pub fn register(&mut self, agent: Box<dyn Agent>) {
        let info = agent.info();
        self.agents.insert(info.id.clone(), RegisteredAgent {
            agent,
            enabled: true,
        });
    }
    
    pub fn get(&self, id: &str) -> Option<&dyn Agent> {
        self.agents.get(id).map(|r| r.agent.as_ref())
    }
    
    pub fn list(&self) -> Vec<AgentInfo> {
        self.agents.values()
            .filter(|r| r.enabled)
            .map(|r| r.agent.info())
            .collect()
    }
    
    pub fn enable(&mut self, id: &str, enabled: bool) {
        if let Some(entry) = self.agents.get_mut(id) {
            entry.enabled = enabled;
        }
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
