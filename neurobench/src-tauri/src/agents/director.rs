//! Director Agent - The Brain of the Multi-Agent System
//!
//! The Director Agent is responsible for:
//! - Understanding user intent
//! - Breaking down complex tasks into subtasks
//! - Delegating work to specialized agents
//! - Synthesizing results from multiple agents
//! - Maintaining conversation context

use super::{Agent, AgentInfo, AgentContext, AgentResponse, ToolCall};
use crate::ai::{AgentModelConfig, ModelManager, AgentModelRegistry};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Intent classification for routing to appropriate agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    /// FSM design and modification
    FsmDesign,
    /// Code generation
    CodeGeneration,
    /// Debugging and error analysis
    Debugging,
    /// Hardware configuration
    HardwareConfig,
    /// Build and compilation
    Build,
    /// Deployment and flashing
    Deploy,
    /// Documentation
    Documentation,
    /// Canvas manipulation
    Canvas,
    /// General conversation
    General,
    /// Multi-step complex task
    Complex,
}

/// A task that can be delegated to an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub intent: Intent,
    pub description: String,
    pub agent_id: String,
    pub context: HashMap<String, serde_json::Value>,
    pub status: TaskStatus,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Plan for executing a complex multi-step task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    pub id: String,
    pub original_request: String,
    pub tasks: Vec<Task>,
    pub current_step: usize,
    pub status: TaskStatus,
}

/// Director Agent - coordinates all other agents
pub struct DirectorAgent {
    model_config: AgentModelConfig,
}

impl DirectorAgent {
    pub fn new() -> Self {
        Self {
            model_config: AgentModelConfig::director(),
        }
    }
    
    /// Classify the intent of a user message
    pub fn classify_intent(&self, message: &str) -> Intent {
        let message_lower = message.to_lowercase();
        
        // Code generation keywords - check BEFORE FSM to handle "generate code for FSM"
        if message_lower.contains("generate code") || message_lower.contains("write code")
            || message_lower.contains("implement") || message_lower.contains("generate c ")
            || message_lower.contains("driver") || message_lower.contains("code for")
        {
            return Intent::CodeGeneration;
        }
        
        // FSM Design keywords
        if message_lower.contains("state") || message_lower.contains("fsm") 
            || message_lower.contains("transition") || message_lower.contains("machine")
            || (message_lower.contains("create") && message_lower.contains("node"))
        {
            return Intent::FsmDesign;
        }
        
        // Debug keywords
        if message_lower.contains("error") || message_lower.contains("bug")
            || message_lower.contains("fix") || message_lower.contains("debug")
            || message_lower.contains("issue") || message_lower.contains("problem")
            || message_lower.contains("not working") || message_lower.contains("crash")
        {
            return Intent::Debugging;
        }
        
        // Hardware keywords
        if message_lower.contains("gpio") || message_lower.contains("uart")
            || message_lower.contains("spi") || message_lower.contains("i2c")
            || message_lower.contains("peripheral") || message_lower.contains("pin")
            || message_lower.contains("mcu") || message_lower.contains("stm32")
            || message_lower.contains("esp32") || message_lower.contains("configure")
        {
            return Intent::HardwareConfig;
        }
        
        // Build keywords
        if message_lower.contains("build") || message_lower.contains("compile")
            || message_lower.contains("make") || message_lower.contains("cmake")
        {
            return Intent::Build;
        }
        
        // Deploy keywords - fix: use || instead of && for "download" || "device" combo
        if message_lower.contains("flash") || message_lower.contains("upload")
            || message_lower.contains("deploy") 
            || (message_lower.contains("download") && message_lower.contains("device"))
        {
            return Intent::Deploy;
        }
        
        // Canvas keywords
        if message_lower.contains("layout") || message_lower.contains("arrange")
            || message_lower.contains("canvas") || message_lower.contains("position")
            || message_lower.contains("move") && message_lower.contains("node")
        {
            return Intent::Canvas;
        }
        
        // Documentation keywords
        if message_lower.contains("document") || message_lower.contains("explain")
            || message_lower.contains("help") || message_lower.contains("how to")
            || message_lower.contains("what is")
        {
            return Intent::Documentation;
        }
        
        // Check for complex multi-step tasks
        if message_lower.contains(" and ") || message_lower.contains(" then ")
            || message_lower.contains("first") || message_lower.contains("after that")
            || (message.len() > 200 && message_lower.contains("create"))
        {
            return Intent::Complex;
        }
        
        Intent::General
    }
    
    /// Map intent to the appropriate agent
    pub fn route_to_agent(&self, intent: &Intent) -> &str {
        match intent {
            Intent::FsmDesign => "fsm",
            Intent::CodeGeneration => "code",
            Intent::Debugging => "debug",
            Intent::HardwareConfig => "hardware",
            Intent::Build => "build",
            Intent::Deploy => "deploy",
            Intent::Canvas => "canvas",
            Intent::Documentation => "docs",
            Intent::General => "fsm", // Default to FSM agent
            Intent::Complex => "director", // Handle internally
        }
    }
    
    /// Create a task plan for complex requests
    pub fn create_plan(&self, request: &str) -> TaskPlan {
        let intent = self.classify_intent(request);
        let mut tasks = Vec::new();
        
        if intent == Intent::Complex {
            // Parse the request into subtasks
            // This is a simplified version - in production, would use LLM
            let parts: Vec<&str> = request.split(&['.', ',', ';'][..])
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            
            for (i, part) in parts.iter().enumerate() {
                let sub_intent = self.classify_intent(part);
                let agent_id = self.route_to_agent(&sub_intent);
                
                tasks.push(Task {
                    id: format!("task_{}", i + 1),
                    intent: sub_intent,
                    description: part.to_string(),
                    agent_id: agent_id.to_string(),
                    context: HashMap::new(),
                    status: TaskStatus::Pending,
                    result: None,
                });
            }
        } else {
            // Single task
            let agent_id = self.route_to_agent(&intent);
            tasks.push(Task {
                id: "task_1".to_string(),
                intent,
                description: request.to_string(),
                agent_id: agent_id.to_string(),
                context: HashMap::new(),
                status: TaskStatus::Pending,
                result: None,
            });
        }
        
        TaskPlan {
            id: format!("plan_{}", chrono::Utc::now().timestamp_millis()),
            original_request: request.to_string(),
            tasks,
            current_step: 0,
            status: TaskStatus::Pending,
        }
    }
    
    /// Get model manager for this agent
    pub fn get_model_manager(&self) -> Option<ModelManager> {
        let registry = AgentModelRegistry::new();
        registry.create_model_manager("director")
    }
}

impl Default for DirectorAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for DirectorAgent {
    fn info(&self) -> AgentInfo {
        AgentInfo {
            id: "director".to_string(),
            name: "Director Agent".to_string(),
            description: "Orchestrates all agents, plans tasks, and coordinates complex operations".to_string(),
            icon: "🎯".to_string(),
            capabilities: super::AgentCapabilities {
                can_edit_fsm: true,
                can_generate_code: true,
                can_execute_terminal: false,
                can_access_hardware: false,
            },
        }
    }
    
    async fn process(
        &self,
        message: &str,
        _context: &super::AgentContext,
    ) -> Result<AgentResponse, String> {
        // Classify intent and create response
        let intent = self.classify_intent(message);
        let target_agent = self.route_to_agent(&intent);
        
        let response_message = if intent == Intent::Complex {
            let plan = self.create_plan(message);
            format!(
                "I'll help you with that. Here's my plan:\n{}",
                plan.tasks.iter()
                    .enumerate()
                    .map(|(i, t)| format!("{}. {} (via {} agent)", i + 1, t.description, t.agent_id))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            format!(
                "I'll route this request to the {} agent for best results.",
                target_agent
            )
        };
        
        Ok(AgentResponse {
            message: response_message,
            tool_calls: vec![ToolCall {
                tool: "delegate".to_string(),
                params: serde_json::json!({
                    "agent": target_agent,
                    "intent": intent,
                }),
            }],
            suggestions: vec![],
        })
    }
    
    fn system_prompt(&self) -> String {
        r#"You are the Director Agent for NeuroBench, an industrial embedded systems workbench.

Your role is to:
1. Understand what the user wants to accomplish
2. Break down complex requests into steps
3. Delegate tasks to specialized agents
4. Synthesize results into coherent responses

Available specialized agents:
- FSM Agent: State machine design and analysis
- Code Agent: Code generation, review, optimization
- Debug Agent: Error analysis, debugging, troubleshooting
- Hardware Agent: MCU/peripheral configuration
- Build Agent: Compilation, build system management
- Deploy Agent: Flashing, device deployment
- Canvas Agent: Visual layout, node arrangement
- Docs Agent: Documentation, help, explanations

When responding:
- If the request is simple, route to the appropriate agent
- If complex, create a step-by-step plan
- Always be helpful, concise, and technically accurate
- Use [DELEGATE:agent_id] to route to another agent
- Use [PLAN:step1,step2,...] to create a multi-step plan

Context about the current project will be provided.
"#.to_string()
    }
    
    fn can_handle(&self, _request_type: &str) -> bool {
        // Director can handle all requests (it routes them)
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_intent_classification() {
        let director = DirectorAgent::new();
        
        assert_eq!(
            director.classify_intent("Create a state machine for a motor controller"),
            Intent::FsmDesign
        );
        
        assert_eq!(
            director.classify_intent("Generate C code for the FSM"),
            Intent::CodeGeneration
        );
        
        assert_eq!(
            director.classify_intent("I'm getting an error when compiling"),
            Intent::Debugging
        );
        
        assert_eq!(
            director.classify_intent("Configure GPIO for the LED"),
            Intent::HardwareConfig
        );
        
        assert_eq!(
            director.classify_intent("Flash the firmware to the device"),
            Intent::Deploy
        );
    }
    
    #[test]
    fn test_routing() {
        let director = DirectorAgent::new();
        
        assert_eq!(director.route_to_agent(&Intent::FsmDesign), "fsm");
        assert_eq!(director.route_to_agent(&Intent::CodeGeneration), "code");
        assert_eq!(director.route_to_agent(&Intent::Debugging), "debug");
    }
    
    #[test]
    fn test_plan_creation() {
        let director = DirectorAgent::new();
        
        let plan = director.create_plan("Create a motor controller FSM");
        assert_eq!(plan.tasks.len(), 1);
        assert_eq!(plan.tasks[0].agent_id, "fsm");
    }
}
