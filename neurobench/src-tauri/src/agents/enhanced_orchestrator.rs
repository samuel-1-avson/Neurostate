// Enhanced Orchestrator
// Routes requests to agents with full AI Engine integration
// Phase 5: Integration of ModelRouter, Guardrails, and Rate Limiting
//
// Note: SemanticCache and Tracer require VectorStore initialization which needs
// embedding providers. This simplified version demonstrates the integration pattern.

use super::{AgentContext, AgentInfo, AgentResponse, AgentRegistry, ToolCall};
use crate::ai::{
    Guardrails, GuardrailsConfig, ValidationContext,
    RateLimiter, RateLimitResult,
    AIService,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use std::collections::HashMap;

// =============================================================================
// ORCHESTRATOR CONFIG
// =============================================================================

/// Configuration for the enhanced orchestrator
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Enable input/output guardrails
    pub guardrails_enabled: bool,
    /// Enable rate limiting
    pub rate_limiting_enabled: bool,
    /// Rate limit: requests per minute
    pub rate_limit_rpm: u32,
    /// Enable simple response caching (in-memory)
    pub simple_cache_enabled: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            guardrails_enabled: true,
            rate_limiting_enabled: true,
            rate_limit_rpm: 60,
            simple_cache_enabled: true,
        }
    }
}

// =============================================================================
// SIMPLE CACHE (In-memory, until VectorStore is configured)
// =============================================================================

struct SimpleCache {
    entries: RwLock<HashMap<String, CachedResponse>>,
    max_entries: usize,
}

#[derive(Clone)]
struct CachedResponse {
    response: String,
    created_at: std::time::Instant,
}

impl SimpleCache {
    fn new(max_entries: usize) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            max_entries,
        }
    }

    async fn get(&self, key: &str) -> Option<String> {
        let entries = self.entries.read().await;
        entries.get(key).map(|e| e.response.clone())
    }

    async fn put(&self, key: &str, response: &str) {
        let mut entries = self.entries.write().await;
        
        // Evict oldest if at capacity
        if entries.len() >= self.max_entries {
            // Simple eviction: remove first entry
            if let Some(first_key) = entries.keys().next().cloned() {
                entries.remove(&first_key);
            }
        }
        
        entries.insert(key.to_string(), CachedResponse {
            response: response.to_string(),
            created_at: std::time::Instant::now(),
        });
    }

    async fn stats(&self) -> SimpleCacheStats {
        let entries = self.entries.read().await;
        SimpleCacheStats {
            entry_count: entries.len(),
            max_entries: self.max_entries,
        }
    }
}

/// Simple cache statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct SimpleCacheStats {
    pub entry_count: usize,
    pub max_entries: usize,
}

// =============================================================================
// ENHANCED ORCHESTRATOR
// =============================================================================

/// Enhanced Orchestrator with AI Engine integration
/// 
/// Integrates:
/// - Guardrails for input/output safety
/// - Rate limiting per user
/// - Simple in-memory caching
/// - AIService for LLM calls
pub struct EnhancedOrchestrator {
    // Core components
    registry: AgentRegistry,
    context: Arc<RwLock<AgentContext>>,
    active_agent: Option<String>,

    // AI Engine integration
    ai_service: AIService,
    guardrails: Guardrails,
    rate_limiter: RateLimiter,
    cache: SimpleCache,

    // Configuration
    config: OrchestratorConfig,

    // Stats
    total_requests: std::sync::atomic::AtomicU64,
    cache_hits: std::sync::atomic::AtomicU64,
    guardrail_blocks: std::sync::atomic::AtomicU64,
}

impl EnhancedOrchestrator {
    /// Create new enhanced orchestrator with default config
    pub fn new() -> Self {
        Self::with_config(OrchestratorConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: OrchestratorConfig) -> Self {
        // Create agent registry
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
        registry.register(Box::new(super::super_agent::SuperAgent::new()));

        // Create AI Engine components
        let ai_service = AIService::new();

        let guardrails = Guardrails::with_config(GuardrailsConfig {
            block_on_critical: true,
            max_input_length: 10000,
            max_output_length: 50000,
            ..Default::default()
        });

        let rate_limiter = RateLimiter::new(config.rate_limit_rpm, Duration::from_secs(60));

        let cache = SimpleCache::new(1000);

        Self {
            registry,
            context: Arc::new(RwLock::new(AgentContext::default())),
            active_agent: Some("director".to_string()),
            ai_service,
            guardrails,
            rate_limiter,
            cache,
            config,
            total_requests: std::sync::atomic::AtomicU64::new(0),
            cache_hits: std::sync::atomic::AtomicU64::new(0),
            guardrail_blocks: std::sync::atomic::AtomicU64::new(0),
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
        self.active_agent
            .as_ref()
            .and_then(|id| self.registry.get(id))
            .map(|a| a.info())
    }

    /// Process a message with full AI Engine integration
    pub async fn process(&self, message: &str) -> Result<AgentResponse, String> {
        self.process_for_user(message, "anonymous").await
    }

    /// Process a message with user identification
    pub async fn process_for_user(
        &self,
        message: &str,
        user_id: &str,
    ) -> Result<AgentResponse, String> {
        use std::sync::atomic::Ordering;
        
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        // 1. Rate limiting check
        if self.config.rate_limiting_enabled {
            match self.rate_limiter.check(user_id).await {
                RateLimitResult::Limited { reset_in, .. } => {
                    return Err(format!(
                        "Rate limited. Please wait {} seconds.",
                        reset_in.as_secs()
                    ));
                }
                RateLimitResult::Allowed { .. } => {
                    self.rate_limiter.record(user_id).await;
                }
            }
        }

        // 2. Input guardrails check
        if self.config.guardrails_enabled {
            let validation = self.guardrails.validate_input(message);
            if !validation.passed {
                self.guardrail_blocks.fetch_add(1, Ordering::Relaxed);
                let issues: Vec<String> = validation
                    .issues
                    .iter()
                    .map(|i| i.description.clone())
                    .collect();
                return Err(format!(
                    "Input blocked by safety filter: {}",
                    issues.join(", ")
                ));
            }
        }

        // 3. Check simple cache
        if self.config.simple_cache_enabled {
            if let Some(cached) = self.cache.get(message).await {
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                return Ok(AgentResponse {
                    message: cached,
                    tool_calls: vec![],
                    suggestions: vec!["(cached response)".to_string()],
                });
            }
        }

        // 4. Get agent and context
        let agent_id = self
            .active_agent
            .as_ref()
            .ok_or_else(|| "No active agent".to_string())?;

        let agent = self
            .registry
            .get(agent_id)
            .ok_or_else(|| format!("Agent '{}' not found", agent_id))?;

        let mut context = self.context.write().await;
        context.add_user_message(message);

        // 5. Build prompt with system prompt and context
        let system_prompt = agent.system_prompt();
        let context_str = context.to_prompt_context();

        // 6. Call AI service
        let response = self.ai_service.chat(
            &format!("{}\n\n## User Request:\n{}", system_prompt, message),
            Some(&context_str)
        ).await?;

        // 7. Parse response
        let agent_response = self.parse_response(&response);

        // 8. Output guardrails (if enabled)
        if self.config.guardrails_enabled {
            let output_validation = self.guardrails.validate_output(
                &agent_response.message,
                &ValidationContext::default(),
            );

            if !output_validation.passed {
                self.guardrail_blocks.fetch_add(1, Ordering::Relaxed);
                return Err("Response blocked by output filter".to_string());
            }
        }

        // 9. Cache the response
        if self.config.simple_cache_enabled {
            self.cache.put(message, &agent_response.message).await;
        }

        // 10. Update conversation history
        context.add_assistant_message(&agent_response.message);

        Ok(agent_response)
    }

    /// Parse LLM response for tool calls and suggestions
    fn parse_response(&self, response: &str) -> AgentResponse {
        let mut tool_calls = Vec::new();
        let mut suggestions = Vec::new();
        let mut message = response.to_string();

        // Look for tool call patterns: [TOOL:name:params]
        if let Ok(re) = regex::Regex::new(r"\[TOOL:(\w+):([^\]]+)\]") {
            for cap in re.captures_iter(response) {
                let tool_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let params_str = cap.get(2).map(|m| m.as_str()).unwrap_or("{}");

                if let Ok(params) = serde_json::from_str(params_str) {
                    tool_calls.push(ToolCall {
                        tool: tool_name.to_string(),
                        params,
                    });
                }
            }
            message = re.replace_all(&message, "").to_string();
        }

        // Look for suggestion patterns: [SUGGEST:text]
        if let Ok(re) = regex::Regex::new(r"\[SUGGEST:([^\]]+)\]") {
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

    // === Context Management ===

    /// Update context with new FSM data
    pub async fn update_fsm(
        &self,
        nodes: Vec<super::ContextNode>,
        edges: Vec<super::ContextEdge>,
    ) {
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
        &self,
        nodes: Vec<super::context::ContextNode>,
        edges: Vec<super::context::ContextEdge>,
        selected_node: Option<String>,
    ) {
        let mut context = self.context.write().await;
        context.nodes = nodes;
        context.edges = edges;
        context.selected_node = selected_node;
    }

    // === Stats and Monitoring ===

    /// Get cache statistics
    pub async fn cache_stats(&self) -> SimpleCacheStats {
        self.cache.stats().await
    }

    /// Get rate limit usage for user
    pub async fn rate_limit_usage(&self, user_id: &str) -> crate::ai::RateLimitUsage {
        self.rate_limiter.usage(user_id).await
    }

    /// Get overall health status
    pub fn health_status(&self) -> OrchestratorHealth {
        use std::sync::atomic::Ordering;
        OrchestratorHealth {
            ready: self.ai_service.is_available(),
            agents_registered: self.registry.list().len(),
            active_agent: self.active_agent.clone(),
            guardrails_enabled: self.config.guardrails_enabled,
            rate_limiting_enabled: self.config.rate_limiting_enabled,
            total_requests: self.total_requests.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            guardrail_blocks: self.guardrail_blocks.load(Ordering::Relaxed),
        }
    }
}

impl Default for EnhancedOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Orchestrator health status
#[derive(Debug, Clone, serde::Serialize)]
pub struct OrchestratorHealth {
    pub ready: bool,
    pub agents_registered: usize,
    pub active_agent: Option<String>,
    pub guardrails_enabled: bool,
    pub rate_limiting_enabled: bool,
    pub total_requests: u64,
    pub cache_hits: u64,
    pub guardrail_blocks: u64,
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = OrchestratorConfig::default();
        assert!(config.guardrails_enabled);
        assert!(config.rate_limiting_enabled);
        assert_eq!(config.rate_limit_rpm, 60);
    }

    #[test]
    fn test_health_status() {
        let orch = EnhancedOrchestrator::new();
        let health = orch.health_status();
        assert!(health.agents_registered > 0);
        assert_eq!(health.active_agent, Some("director".to_string()));
    }

    #[tokio::test]
    async fn test_simple_cache() {
        let cache = SimpleCache::new(10);
        
        cache.put("test", "response").await;
        let result = cache.get("test").await;
        assert_eq!(result, Some("response".to_string()));

        let miss = cache.get("nonexistent").await;
        assert_eq!(miss, None);
    }
}
