//! Unified AI Assistant
//!
//! The central entry point for all AI interactions in NeuroBench.
//! Provides a unified interface similar to Cursor/Antigravity AI assistants.
//!
//! Features:
//! - Single entry point for all AI requests
//! - Automatic complexity detection and routing
//! - Agentic loop for multi-step tasks
//! - Streaming response support
//! - Tool execution with context updates
//! - Memory persistence across sessions

use super::{
    Agent, AgentContext, AgentInfo, AgentResponse, AgentRegistry, ToolCall,
    SuperAgent,
    MemoryManager,
    HierarchicalPlanner, TaskTree,
    ConsensusProtocol, ConsensusConfig, ConsensusMethod,
    ToolExecutor, ToolResult,
    StreamChunk,
};
use crate::ai::{AIService, guardrails::Guardrails};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use async_trait::async_trait;

// =============================================================================
// CONFIGURATION
// =============================================================================

/// Configuration for the unified assistant
#[derive(Debug, Clone)]
pub struct UnifiedAssistantConfig {
    /// Enable guardrails for input/output validation
    pub enable_guardrails: bool,
    /// Complexity threshold for switching to SuperAgent orchestration
    pub complexity_threshold: u8,
    /// Maximum iterations for agentic loop
    pub max_loop_iterations: u32,
    /// Enable streaming responses
    pub enable_streaming: bool,
    /// Enable memory persistence
    pub enable_memory: bool,
    /// Default user ID for anonymous requests
    pub default_user_id: String,
}

impl Default for UnifiedAssistantConfig {
    fn default() -> Self {
        Self {
            enable_guardrails: true,
            complexity_threshold: 3,
            max_loop_iterations: 10,
            enable_streaming: true,
            enable_memory: true,
            default_user_id: "default".to_string(),
        }
    }
}

// =============================================================================
// ASSISTANT RESPONSE
// =============================================================================

/// Unified response from the assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantResponse {
    /// Main response message
    pub message: String,
    /// Tool calls to execute
    pub tool_calls: Vec<ToolCall>,
    /// Results of executed tools
    pub tool_results: Vec<ToolResult>,
    /// Follow-up suggestions
    pub suggestions: Vec<String>,
    /// Whether the task is complete
    pub is_complete: bool,
    /// Current task progress (0-100)
    pub progress: u8,
    /// Agent that handled the request
    pub agent_id: String,
    /// Execution metadata
    pub metadata: AssistantMetadata,
}

/// Metadata about the assistant's execution
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssistantMetadata {
    /// Number of loop iterations executed
    pub iterations: u32,
    /// Total tokens used
    pub tokens_used: u32,
    /// Time taken in milliseconds
    pub time_ms: u64,
    /// Agents that participated
    pub agents_used: Vec<String>,
    /// Complexity score
    pub complexity: u8,
}

// =============================================================================
// ASSISTANT CONTEXT
// =============================================================================

/// Extended context for the assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantContext {
    /// Base agent context (FSM, project, MCU)
    pub agent_context: AgentContext,
    /// User identifier
    pub user_id: String,
    /// Session identifier
    pub session_id: String,
    /// Accumulated tool results from current interaction
    pub tool_results: Vec<ToolResult>,
    /// Current task tree (if multi-step)
    pub task_tree: Option<TaskTree>,
}

impl Default for AssistantContext {
    fn default() -> Self {
        Self {
            agent_context: AgentContext::default(),
            user_id: "default".to_string(),
            session_id: uuid::Uuid::new_v4().to_string(),
            tool_results: Vec::new(),
            task_tree: None,
        }
    }
}

impl AssistantContext {
    pub fn new(user_id: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            session_id: uuid::Uuid::new_v4().to_string(),
            ..Default::default()
        }
    }
    
    /// Add a tool result to the context
    pub fn add_tool_result(&mut self, result: ToolResult) {
        self.tool_results.push(result);
    }
    
    /// Get tool results as context string
    pub fn tool_results_context(&self) -> String {
        if self.tool_results.is_empty() {
            return String::new();
        }
        
        let mut ctx = String::from("\n## Previous Tool Results:\n");
        for (i, result) in self.tool_results.iter().enumerate() {
            ctx.push_str(&format!(
                "{}. {}: {}\n",
                i + 1,
                if result.success { "✓" } else { "✗" },
                result.message
            ));
        }
        ctx
    }
}

// =============================================================================
// UNIFIED AI ASSISTANT
// =============================================================================

/// The Unified AI Assistant - single entry point for all AI interactions
pub struct UnifiedAIAssistant {
    /// Configuration
    config: UnifiedAssistantConfig,
    /// Agent registry
    registry: AgentRegistry,
    /// Super Agent for complex orchestration
    super_agent: SuperAgent,
    /// Hierarchical planner for task trees
    planner: HierarchicalPlanner,
    /// Consensus protocol for multi-agent decisions
    consensus: ConsensusProtocol,
    /// AI service for direct LLM calls
    ai_service: AIService,
    /// Guardrails for safety
    guardrails: Guardrails,
    /// Memory manager
    memory: Arc<RwLock<MemoryManager>>,
    /// Shared context (public for session memory access)
    pub context: Arc<RwLock<AssistantContext>>,
}

impl UnifiedAIAssistant {
    /// Create a new unified assistant with default config
    pub fn new() -> Self {
        Self::with_config(UnifiedAssistantConfig::default())
    }
    
    /// Create with custom configuration
    pub fn with_config(config: UnifiedAssistantConfig) -> Self {
        let mut registry = AgentRegistry::new();
        
        // Register all agents
        registry.register(Box::new(super::director::DirectorAgent::new()));
        registry.register(Box::new(super::fsm_agent::FsmAgent::new()));
        registry.register(Box::new(super::code_agent::CodeAgent::new()));
        registry.register(Box::new(super::debug_agent::DebugAgent::new()));
        registry.register(Box::new(super::hardware_agent::HardwareAgent::new()));
        registry.register(Box::new(super::docs_agent::DocsAgent::new()));
        registry.register(Box::new(super::canvas_agent::CanvasAgent::new()));
        registry.register(Box::new(super::build_agent::BuildAgent::new()));
        registry.register(Box::new(super::deploy_agent::DeployAgent::new()));
        registry.register(Box::new(super::voice_agent::VoiceAssistantAgent::new()));
        
        // Configure consensus for multi-agent decisions
        let consensus = ConsensusProtocol::new()
            .with_config(ConsensusConfig {
                method: ConsensusMethod::BestOf,
                min_opinions: 1,
                timeout_secs: 30,
                min_confidence: 0.3,
                allow_abstain: true,
            });
        
        Self {
            config,
            registry,
            super_agent: SuperAgent::new(),
            planner: HierarchicalPlanner::new(),
            consensus,
            ai_service: AIService::new(),
            guardrails: Guardrails::new(),
            memory: Arc::new(RwLock::new(MemoryManager::new())),
            context: Arc::new(RwLock::new(AssistantContext::default())),
        }
    }
    
    // =========================================================================
    // PRIMARY INTERFACE
    // =========================================================================
    
    /// Main entry point - process a user message
    /// This is the unified interface similar to Cursor/Antigravity
    pub async fn chat(&self, message: &str, user_id: Option<&str>) -> Result<AssistantResponse, String> {
        let user = user_id.unwrap_or(&self.config.default_user_id);
        let start_time = std::time::Instant::now();
        
        // Update context with user
        {
            let mut ctx = self.context.write().await;
            ctx.user_id = user.to_string();
            ctx.agent_context.add_user_message(message);
        }
        
        // Step 1: Input validation (if guardrails enabled)
        if self.config.enable_guardrails {
            let validation = self.guardrails.validate_input(message);
            if !validation.passed {
                return Err(format!("Input blocked: {:?}", validation.issues));
            }
        }
        
        // Step 2: Analyze complexity
        let context = self.context.read().await;
        let complexity = self.analyze_complexity(message, &context.agent_context).await;
        
        // Step 3: Route to appropriate processing
        let result = if complexity > self.config.complexity_threshold {
            drop(context);
            self.process_complex(message, user).await
        } else {
            drop(context);
            self.process_simple(message, user).await
        };
        
        // Step 4: Add execution time to metadata
        match result {
            Ok(mut response) => {
                response.metadata.time_ms = start_time.elapsed().as_millis() as u64;
                response.metadata.complexity = complexity;
                
                // Save to memory if enabled
                if self.config.enable_memory {
                    self.save_interaction(message, &response).await;
                }
                
                Ok(response)
            }
            Err(e) => Err(e),
        }
    }
    
    /// Process a message with streaming support
    pub async fn chat_stream(
        &self,
        message: &str,
        user_id: Option<&str>,
        callback: impl Fn(StreamChunk) + Send + Sync,
    ) -> Result<AssistantResponse, String> {
        // Send initial chunk
        callback(StreamChunk {
            stream_id: "unified".to_string(),
            sequence: 0,
            content: String::new(),
            done: false,
            metadata: None,
        });
        
        // Process normally (streaming will be enhanced later with actual token streaming)
        let response = self.chat(message, user_id).await?;
        
        // Send final chunk
        callback(StreamChunk {
            stream_id: "unified".to_string(),
            sequence: 1,
            content: response.message.clone(),
            done: true,
            metadata: None,
        });
        
        Ok(response)
    }
    
    // =========================================================================
    // PROCESSING MODES
    // =========================================================================
    
    /// Process simple, single-agent requests
    async fn process_simple(&self, message: &str, user_id: &str) -> Result<AssistantResponse, String> {
        let context = self.context.read().await;
        
        // Build context-enriched prompt
        let enriched_message = self.build_contextual_prompt(message, &context.agent_context);
        
        // Use Director to route to appropriate agent
        let director = super::director::DirectorAgent::new();
        let intent = director.classify_intent(message);
        let agent_id = director.route_to_agent(&intent);
        
        // Get the appropriate agent and process with enriched context
        let agent = self.registry.get(agent_id)
            .ok_or_else(|| format!("Agent '{}' not found", agent_id))?;
        
        let response = agent.process(&enriched_message, &context.agent_context).await?;
        
        Ok(AssistantResponse {
            message: response.message,
            tool_calls: response.tool_calls,
            tool_results: Vec::new(),
            suggestions: response.suggestions,
            is_complete: true,
            progress: 100,
            agent_id: agent_id.to_string(),
            metadata: AssistantMetadata {
                iterations: 1,
                agents_used: vec![agent_id.to_string()],
                ..Default::default()
            },
        })
    }
    
    /// Build a context-enriched prompt that injects current FSM/project/MCU state
    fn build_contextual_prompt(&self, message: &str, ctx: &AgentContext) -> String {
        let mut context_parts = Vec::new();
        
        // Add project context
        if !ctx.project.name.is_empty() {
            context_parts.push(format!("Project: {}", ctx.project.name));
        }
        
        // Add MCU target
        if !ctx.mcu.target.is_empty() {
            context_parts.push(format!("Target MCU: {}", ctx.mcu.target));
        }
        
        // Add FSM state summary
        let node_count = ctx.nodes.len();
        let edge_count = ctx.edges.len();
        if node_count > 0 {
            context_parts.push(format!("FSM: {} states, {} transitions", node_count, edge_count));
            
            // Summarize current states (truncate if too many)
            if node_count <= 8 {
                let state_names: Vec<&str> = ctx.nodes.iter()
                    .map(|s| s.label.as_str())
                    .collect();
                context_parts.push(format!("States: {}", state_names.join(", ")));
            }
        }
        
        // Add selected node context if any
        if let Some(ref selected_id) = ctx.selected_node {
            if let Some(node) = ctx.nodes.iter().find(|n| &n.id == selected_id) {
                context_parts.push(format!("Selected: {} ({})", node.label, node.node_type));
            }
        }
        
        // Build the enriched prompt
        if context_parts.is_empty() {
            message.to_string()
        } else {
            format!("[Context: {}]\n\n{}", context_parts.join(" | "), message)
        }
    }
    
    /// Process complex, multi-step requests using the agentic loop pattern
    async fn process_complex(&self, message: &str, user_id: &str) -> Result<AssistantResponse, String> {
        let context = self.context.read().await;
        
        // Step 1: Use SuperAgent to analyze and break down the task
        let breakdown = self.super_agent.analyze(message, &context.agent_context).await?;
        drop(context);
        
        // Step 2: Create a hierarchical task tree if needed
        let task_plan = self.super_agent.to_task_plan(&breakdown);
        
        // Step 3: Execute tasks with agentic loop
        let mut all_tool_results = Vec::new();
        let mut all_messages = Vec::new();
        let mut agents_used = Vec::new();
        let mut iterations = 0u32;
        
        for task in &breakdown.tasks {
            if iterations >= self.config.max_loop_iterations {
                break;
            }
            
            // Get the agent for this task
            let agent = match self.registry.get(&task.agent_id) {
                Some(a) => a,
                None => continue,
            };
            
            if !agents_used.contains(&task.agent_id) {
                agents_used.push(task.agent_id.clone());
            }
            
            // Execute the task
            let context = self.context.read().await;
            let response = agent.process(&task.description, &context.agent_context).await;
            drop(context);
            
            match response {
                Ok(resp) => {
                    all_messages.push(format!("**[{}]** {}", task.agent_id.to_uppercase(), resp.message));
                    
                    // Execute any tool calls
                    for tool in &resp.tool_calls {
                        let result = ToolExecutor::execute(&tool.tool, &tool.params);
                        all_tool_results.push(result.clone());
                        
                        // Update context with tool result
                        let mut ctx = self.context.write().await;
                        ctx.add_tool_result(result);
                    }
                }
                Err(e) => {
                    all_messages.push(format!("**[{}]** Error: {}", task.agent_id.to_uppercase(), e));
                }
            }
            
            iterations += 1;
        }
        
        // Build final response
        let combined_message = format!(
            "## 🧠 Task Complete\n\n**Intent:** {:?}\n**Complexity:** {}/10\n\n{}\n\n**Expected Outcome:** {}",
            breakdown.intent,
            breakdown.complexity,
            all_messages.join("\n\n"),
            breakdown.expected_outcome
        );
        
        Ok(AssistantResponse {
            message: combined_message,
            tool_calls: Vec::new(), // Already executed
            tool_results: all_tool_results,
            suggestions: vec![
                "Generate code for this".to_string(),
                "Add more states".to_string(),
                "Validate the design".to_string(),
            ],
            is_complete: true,
            progress: 100,
            agent_id: "nexus".to_string(),
            metadata: AssistantMetadata {
                iterations,
                agents_used,
                complexity: breakdown.complexity,
                ..Default::default()
            },
        })
    }
    
    // =========================================================================
    // HELPERS
    // =========================================================================
    
    /// Analyze message complexity to determine routing
    async fn analyze_complexity(&self, message: &str, context: &AgentContext) -> u8 {
        // Simple heuristics for complexity
        let mut score = 1u8;
        
        // Complexity indicators
        let complex_keywords = [
            "create", "design", "build", "implement", "generate",
            "and",  "then", "after that", "also",
            "traffic light", "motor controller", "state machine",
            "with", "including", "multiple", "several"
        ];
        
        let message_lower = message.to_lowercase();
        for keyword in complex_keywords {
            if message_lower.contains(keyword) {
                score = score.saturating_add(1);
            }
        }
        
        // Long messages are usually complex
        if message.len() > 100 { score = score.saturating_add(2); }
        if message.len() > 200 { score = score.saturating_add(2); }
        
        // Questions are usually simple
        if message.ends_with('?') && score > 3 {
            score = score.saturating_sub(2);
        }
        
        score.min(10)
    }
    
    /// Save interaction to memory
    async fn save_interaction(&self, message: &str, response: &AssistantResponse) {
        let mut memory = self.memory.write().await;
        let agent_memory = memory.get_or_create("unified_assistant");
        agent_memory.short_term.add(message);
        agent_memory.short_term.add(&response.message);
    }
    
    // =========================================================================
    // CONTEXT MANAGEMENT
    // =========================================================================
    
    /// Update the FSM context
    pub async fn update_canvas_context(
        &self,
        nodes: Vec<super::ContextNode>,
        edges: Vec<super::ContextEdge>,
        selected_node: Option<String>,
    ) {
        let mut ctx = self.context.write().await;
        ctx.agent_context.nodes = nodes;
        ctx.agent_context.edges = edges;
        ctx.agent_context.selected_node = selected_node;
    }
    
    /// Update MCU target
    pub async fn set_mcu(&self, target: &str) {
        let mut ctx = self.context.write().await;
        ctx.agent_context.mcu.target = target.to_string();
    }
    
    /// Update project info
    pub async fn set_project(&self, name: &str, language: &str) {
        let mut ctx = self.context.write().await;
        ctx.agent_context.project.name = name.to_string();
        ctx.agent_context.project.language = language.to_string();
    }
    
    /// Get available agents
    pub fn list_agents(&self) -> Vec<AgentInfo> {
        self.registry.list()
    }
    
    /// Get current context
    pub async fn get_context(&self) -> AssistantContext {
        self.context.read().await.clone()
    }
}

impl Default for UnifiedAIAssistant {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_defaults() {
        let config = UnifiedAssistantConfig::default();
        assert!(config.enable_guardrails);
        assert!(config.enable_streaming);
        assert_eq!(config.complexity_threshold, 3);
    }
    
    #[test]
    fn test_assistant_context() {
        let mut ctx = AssistantContext::new("test_user");
        assert_eq!(ctx.user_id, "test_user");
        
        ctx.add_tool_result(ToolResult::success("Test completed"));
        assert_eq!(ctx.tool_results.len(), 1);
    }
    
    #[tokio::test]
    async fn test_assistant_creation() {
        let assistant = UnifiedAIAssistant::new();
        let agents = assistant.list_agents();
        
        // Should have registered agents
        assert!(!agents.is_empty());
    }
}
