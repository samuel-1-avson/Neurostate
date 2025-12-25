// Agent Context
// Shared state available to all agents

use serde::{Deserialize, Serialize};

/// FSM Node representation for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub x: f64,
    pub y: f64,
    pub entry_action: Option<String>,
}

/// FSM Edge representation for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: Option<String>,
}

/// MCU configuration in context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McuConfig {
    pub target: String,
    pub clock_speed: u32,
    pub flash_size: u32,
    pub ram_size: u32,
}

impl Default for McuConfig {
    fn default() -> Self {
        Self {
            target: "STM32F401".to_string(),
            clock_speed: 84_000_000,
            flash_size: 256 * 1024,
            ram_size: 64 * 1024,
        }
    }
}

/// Project state in context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub name: String,
    pub language: String, // "c", "cpp", "rust"
    pub ide: String,
}

impl Default for ProjectContext {
    fn default() -> Self {
        Self {
            name: "Untitled Project".to_string(),
            language: "c".to_string(),
            ide: "stm32cubeide".to_string(),
        }
    }
}

/// Full agent context - everything agents can see
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    /// Project info
    pub project: ProjectContext,
    
    /// Target MCU
    pub mcu: McuConfig,
    
    /// Current FSM nodes
    pub nodes: Vec<ContextNode>,
    
    /// Current FSM edges
    pub edges: Vec<ContextEdge>,
    
    /// Currently selected node ID
    pub selected_node: Option<String>,
    
    /// Recent console logs
    pub recent_logs: Vec<String>,
    
    /// Conversation history for this session
    pub conversation: Vec<ConversationTurn>,
}

/// Single conversation turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub role: String, // "user" or "assistant"
    pub content: String,
}

impl Default for AgentContext {
    fn default() -> Self {
        Self {
            project: ProjectContext::default(),
            mcu: McuConfig::default(),
            nodes: Vec::new(),
            edges: Vec::new(),
            selected_node: None,
            recent_logs: Vec::new(),
            conversation: Vec::new(),
        }
    }
}

impl AgentContext {
    /// Create a new context
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Format context as a string for LLM prompts
    pub fn to_prompt_context(&self) -> String {
        let mut ctx = String::new();
        
        ctx.push_str(&format!("Project: {} ({})\n", self.project.name, self.project.language));
        ctx.push_str(&format!("Target MCU: {} @ {}MHz\n", self.mcu.target, self.mcu.clock_speed / 1_000_000));
        
        if !self.nodes.is_empty() {
            ctx.push_str(&format!("\nFSM States ({}):\n", self.nodes.len()));
            for node in &self.nodes {
                ctx.push_str(&format!("  - {} ({}){}\n", 
                    node.label, 
                    node.node_type,
                    if self.selected_node.as_ref() == Some(&node.id) { " [SELECTED]" } else { "" }
                ));
            }
        }
        
        if !self.edges.is_empty() {
            ctx.push_str(&format!("\nTransitions ({}):\n", self.edges.len()));
            for edge in &self.edges {
                let source = self.nodes.iter().find(|n| n.id == edge.source).map(|n| n.label.as_str()).unwrap_or("?");
                let target = self.nodes.iter().find(|n| n.id == edge.target).map(|n| n.label.as_str()).unwrap_or("?");
                ctx.push_str(&format!("  - {} → {}{}\n", 
                    source, 
                    target,
                    edge.label.as_ref().map(|l| format!(" [{}]", l)).unwrap_or_default()
                ));
            }
        }
        
        ctx
    }
    
    /// Add a user message to conversation
    pub fn add_user_message(&mut self, content: &str) {
        self.conversation.push(ConversationTurn {
            role: "user".to_string(),
            content: content.to_string(),
        });
    }
    
    /// Add an assistant message to conversation
    pub fn add_assistant_message(&mut self, content: &str) {
        self.conversation.push(ConversationTurn {
            role: "assistant".to_string(),
            content: content.to_string(),
        });
    }
}
