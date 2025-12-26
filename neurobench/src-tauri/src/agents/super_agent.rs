//! Super AI Agent (Nexus)
//!
//! The master orchestrator that:
//! - Breaks down complex user inputs into structured tasks
//! - Converts natural language into agent-compatible formats
//! - Coordinates multiple agents for comprehensive results
//! - Synthesizes outputs into cohesive responses

use super::{Agent, AgentContext, AgentResponse, ToolCall, Task, TaskPlan, TaskStatus};
use crate::ai::providers::{ModelConfig, ModelProvider, OpenAIModel, AIModel};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Super AI Agent - The Nexus that coordinates everything
pub struct SuperAgent {
    model: OpenAIModel,
}

/// Structured task breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBreakdown {
    /// Original user request
    pub original_request: String,
    /// Overall intent classification
    pub intent: IntentCategory,
    /// Complexity level (1-10)
    pub complexity: u8,
    /// Individual tasks to execute
    pub tasks: Vec<AgentTask>,
    /// Execution strategy
    pub strategy: ExecutionStrategy,
    /// Expected outcome description
    pub expected_outcome: String,
}

/// Categories of user intents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentCategory {
    /// Design-related (FSM design, architecture)
    Design,
    /// Implementation (code generation, configuration)
    Implementation,
    /// Build and deployment
    BuildDeploy,
    /// Debugging and troubleshooting
    Debug,
    /// Information and learning
    Information,
    /// Mixed/complex requiring multiple categories
    Mixed,
}

/// Execution strategy for tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStrategy {
    /// Execute tasks one after another
    Sequential,
    /// Execute independent tasks in parallel
    Parallel,
    /// Mix of parallel and sequential
    Mixed,
    /// Requires user confirmation between steps
    Interactive,
}

/// A single task for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub agent_id: String,
    pub description: String,
    pub input_format: TaskInput,
    pub dependencies: Vec<String>,
    pub priority: u8,
}

/// Structured input format for agents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskInput {
    /// Natural language description
    Text { content: String },
    /// Structured FSM definition
    FsmDefinition {
        states: Vec<String>,
        transitions: Vec<(String, String, Option<String>)>,
        initial_state: Option<String>,
    },
    /// Code generation request
    CodeRequest {
        language: String,
        style: Option<String>,
        constraints: Vec<String>,
    },
    /// Build configuration
    BuildConfig {
        target: Option<String>,
        release: bool,
        flags: Vec<String>,
    },
    /// Hardware configuration
    HardwareConfig {
        mcu: String,
        peripherals: Vec<String>,
        pins: Vec<(String, String)>,
    },
    /// Error analysis request
    ErrorAnalysis {
        error_message: String,
        context: Option<String>,
    },
}

impl SuperAgent {
    pub fn new() -> Self {
        // Use GPT-4o for maximum reasoning capability
        let api_key = std::env::var("OPENAI_API_KEY").ok();
        
        Self {
            model: OpenAIModel::new(ModelConfig {
                provider: ModelProvider::OpenAI,
                model_name: "gpt-4o".to_string(),
                api_key,
                base_url: Some("https://api.openai.com/v1".to_string()),
                temperature: 0.3,  // Lower for precise parsing
                max_tokens: 8192,
                timeout_secs: 120,
            }),
        }
    }
    
    /// Build the parsing prompt
    fn build_prompt(&self, message: &str, context: &AgentContext) -> String {
        format!(r#"You are the NEXUS - the Super AI that orchestrates all other AI agents in NeuroBench.

YOUR MISSION:
Parse the user's natural language input and break it down into structured tasks that specialized agents can execute.

AVAILABLE AGENTS:
1. director - High-level task planning and coordination
2. code - Code generation, optimization, review
3. debug - Error analysis, troubleshooting
4. hardware - MCU configuration, peripheral setup
5. fsm - State machine design and analysis
6. canvas - Visual canvas manipulation
7. build - Project compilation
8. deploy - Firmware flashing
9. docs - Documentation generation
10. voice - Conversational interface

CURRENT CONTEXT:
- Project: {} ({})
- MCU: {}
- States: {}
- Selected Node: {}

USER INPUT: "{}"

Analyze this request and respond with a structured JSON breakdown:
```json
{{
  "intent": "design|implementation|build_deploy|debug|information|mixed",
  "complexity": 1-10,
  "strategy": "sequential|parallel|mixed|interactive",
  "expected_outcome": "Brief description of what will be accomplished",
  "tasks": [
    {{
      "id": "task_1",
      "agent_id": "agent_name",
      "description": "Clear task description for the agent",
      "input_format": {{ "type": "text", "content": "..." }},
      "dependencies": [],
      "priority": 1
    }}
  ]
}}
```

IMPORTANT:
- Break complex requests into atomic tasks
- Identify dependencies between tasks
- Choose the most appropriate agent for each task
- Order tasks logically for execution
- Use parallel execution when tasks are independent"#,
            context.project.name,
            context.project.language,
            context.mcu.target,
            context.nodes.len(),
            context.selected_node.as_deref().unwrap_or("None"),
            message
        )
    }
    
    /// Parse the structured response
    fn parse_breakdown(&self, response: &str) -> Result<TaskBreakdown, String> {
        // Extract JSON from response
        let json_re = regex::Regex::new(r"```json\s*([\s\S]*?)\s*```").unwrap();
        
        let json_str = if let Some(cap) = json_re.captures(response) {
            cap.get(1).map(|m| m.as_str()).unwrap_or(response)
        } else {
            response
        };
        
        // Parse the JSON
        #[derive(Deserialize)]
        struct RawBreakdown {
            intent: String,
            complexity: u8,
            strategy: String,
            expected_outcome: String,
            tasks: Vec<RawTask>,
        }
        
        #[derive(Deserialize)]
        struct RawTask {
            id: String,
            agent_id: String,
            description: String,
            input_format: serde_json::Value,
            dependencies: Vec<String>,
            priority: u8,
        }
        
        let raw: RawBreakdown = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse breakdown: {}", e))?;
        
        // Convert to typed structures
        let intent = match raw.intent.as_str() {
            "design" => IntentCategory::Design,
            "implementation" => IntentCategory::Implementation,
            "build_deploy" => IntentCategory::BuildDeploy,
            "debug" => IntentCategory::Debug,
            "information" => IntentCategory::Information,
            _ => IntentCategory::Mixed,
        };
        
        let strategy = match raw.strategy.as_str() {
            "sequential" => ExecutionStrategy::Sequential,
            "parallel" => ExecutionStrategy::Parallel,
            "interactive" => ExecutionStrategy::Interactive,
            _ => ExecutionStrategy::Mixed,
        };
        
        let tasks: Vec<AgentTask> = raw.tasks.into_iter().map(|t| {
            let input_format = match t.input_format.get("type").and_then(|v| v.as_str()) {
                Some("text") => TaskInput::Text {
                    content: t.input_format.get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&t.description)
                        .to_string(),
                },
                Some("fsm_definition") => TaskInput::FsmDefinition {
                    states: t.input_format.get("states")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                        .unwrap_or_default(),
                    transitions: Vec::new(),
                    initial_state: t.input_format.get("initial_state")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                },
                Some("code_request") => TaskInput::CodeRequest {
                    language: t.input_format.get("language")
                        .and_then(|v| v.as_str())
                        .unwrap_or("c")
                        .to_string(),
                    style: t.input_format.get("style")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    constraints: Vec::new(),
                },
                _ => TaskInput::Text {
                    content: t.description.clone(),
                },
            };
            
            AgentTask {
                id: t.id,
                agent_id: t.agent_id,
                description: t.description,
                input_format,
                dependencies: t.dependencies,
                priority: t.priority,
            }
        }).collect();
        
        Ok(TaskBreakdown {
            original_request: String::new(),
            intent,
            complexity: raw.complexity,
            tasks,
            strategy,
            expected_outcome: raw.expected_outcome,
        })
    }
    
    /// Convert breakdown to TaskPlan format
    pub fn to_task_plan(&self, breakdown: &TaskBreakdown) -> TaskPlan {
        let tasks: Vec<Task> = breakdown.tasks.iter().map(|t| {
            Task {
                id: t.id.clone(),
                description: t.description.clone(),
                agent_id: t.agent_id.clone(),
                status: TaskStatus::Pending,
                intent: super::Intent::General,
                context: std::collections::HashMap::new(),
                result: None,
            }
        }).collect();
        
        TaskPlan {
            id: format!("plan_{}", chrono::Utc::now().timestamp_millis()),
            original_request: breakdown.original_request.clone(),
            tasks,
            current_step: 0,
            status: TaskStatus::Pending,
        }
    }
    
    /// Analyze user input and return structured breakdown
    pub async fn analyze(&self, message: &str, context: &AgentContext) -> Result<TaskBreakdown, String> {
        let prompt = self.build_prompt(message, context);
        
        let model_response = self.model.generate(&prompt).await
            .map_err(|e| format!("Super Agent analysis error: {}", e))?;
        
        let mut breakdown = self.parse_breakdown(&model_response.content)?;
        breakdown.original_request = message.to_string();
        
        Ok(breakdown)
    }
}

impl Default for SuperAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for SuperAgent {
    fn info(&self) -> super::AgentInfo {
        super::AgentInfo {
            id: "nexus".to_string(),
            name: "Nexus (Super AI)".to_string(),
            description: "Master orchestrator that breaks down complex requests and coordinates all agents".to_string(),
            icon: "🧠".to_string(),
            capabilities: super::AgentCapabilities {
                can_edit_fsm: true,
                can_generate_code: true,
                can_execute_terminal: true,
                can_access_hardware: true,
            },
        }
    }
    
    fn system_prompt(&self) -> String {
        r#"You are the NEXUS - the Super AI that orchestrates all other AI agents in NeuroBench.

YOUR MISSION:
Parse the user's natural language input and break it down into structured tasks that specialized agents can execute.

AVAILABLE AGENTS:
1. director - High-level task planning
2. code - Code generation, optimization
3. debug - Error analysis, troubleshooting
4. hardware - MCU configuration
5. fsm - State machine design
6. canvas - Visual canvas manipulation
7. build - Project compilation
8. deploy - Firmware flashing
9. docs - Documentation
10. voice - Conversational interface

Analyze requests and respond with structured JSON task breakdowns."#.to_string()
    }
    
    fn can_handle(&self, request_type: &str) -> bool {
        matches!(request_type, 
            "complex" | "multi_step" | "parse" | 
            "analyze" | "orchestrate" | "breakdown"
        )
    }
    
    async fn process(&self, message: &str, context: &AgentContext) -> Result<AgentResponse, String> {
        // Analyze the message
        let breakdown = self.analyze(message, context).await?;
        
        // Build response message
        let mut response_parts = Vec::new();
        
        response_parts.push(format!("## 🧠 Analysis Complete\n"));
        response_parts.push(format!("**Intent:** {:?}", breakdown.intent));
        response_parts.push(format!("**Complexity:** {}/10", breakdown.complexity));
        response_parts.push(format!("**Strategy:** {:?}\n", breakdown.strategy));
        response_parts.push(format!("**Expected Outcome:** {}\n", breakdown.expected_outcome));
        
        response_parts.push("### Task Breakdown:".to_string());
        for task in &breakdown.tasks {
            let deps = if task.dependencies.is_empty() {
                "none".to_string()
            } else {
                task.dependencies.join(", ")
            };
            response_parts.push(format!(
                "{}. **[{}]** {} (deps: {})",
                task.priority, task.agent_id.to_uppercase(), task.description, deps
            ));
        }
        
        // Create tool calls for task execution
        let tool_calls: Vec<ToolCall> = breakdown.tasks.iter().map(|task| {
            ToolCall {
                tool: "execute_task".to_string(),
                params: serde_json::json!({
                    "task_id": task.id,
                    "agent_id": task.agent_id,
                    "description": task.description,
                    "priority": task.priority
                }),
            }
        }).collect();
        
        let suggestions = vec![
            "Execute this plan".to_string(),
            "Modify the breakdown".to_string(),
            "Add more details".to_string(),
        ];
        
        Ok(AgentResponse {
            message: response_parts.join("\n"),
            tool_calls,
            suggestions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_intent_category() {
        let intent = IntentCategory::Design;
        let serialized = serde_json::to_string(&intent).unwrap();
        assert!(serialized.contains("design"));
    }
    
    #[test]
    fn test_task_input_formats() {
        let text = TaskInput::Text {
            content: "Create a motor controller".to_string(),
        };
        
        let code = TaskInput::CodeRequest {
            language: "c".to_string(),
            style: Some("switch_case".to_string()),
            constraints: vec!["minimal memory".to_string()],
        };
        
        assert!(serde_json::to_string(&text).is_ok());
        assert!(serde_json::to_string(&code).is_ok());
    }
}
