// Enhanced Agent Context
// Extended agent context with RAG pipeline, function calling, and observability support

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ai::{
    RAGPipeline, RetrievedChunk,
    SemanticCache, CacheResult,
    Tracer, Span,
};

// Use the agents function_calling module (not ai)
use super::function_calling::{FunctionDef, FunctionCall, PropertyDef, get_standard_functions};

// =============================================================================
// ENHANCED CONTEXT CONFIG
// =============================================================================

/// Configuration for the enhanced agent context
#[derive(Debug, Clone)]
pub struct EnhancedContextConfig {
    /// Enable RAG for context enhancement
    pub rag_enabled: bool,
    /// Enable semantic caching
    pub cache_enabled: bool,
    /// Enable distributed tracing
    pub tracing_enabled: bool,
    /// Enable function calling
    pub function_calling_enabled: bool,
    /// Maximum RAG chunks to include
    pub max_rag_chunks: usize,
}

impl Default for EnhancedContextConfig {
    fn default() -> Self {
        Self {
            rag_enabled: true,
            cache_enabled: true,
            tracing_enabled: true,
            function_calling_enabled: true,
            max_rag_chunks: 5,
        }
    }
}

// =============================================================================
// FUNCTION REGISTRY FOR AGENTS
// =============================================================================

/// Simple function registry for agent function calling
pub struct AgentFunctionRegistry {
    functions: Vec<FunctionDef>,
}

impl AgentFunctionRegistry {
    pub fn new() -> Self {
        Self { functions: Vec::new() }
    }

    /// Register a function definition
    pub fn register(&mut self, func: FunctionDef) {
        self.functions.push(func);
    }

    /// Get all function definitions
    pub fn list(&self) -> &[FunctionDef] {
        &self.functions
    }

    /// Get a function by name
    pub fn get(&self, name: &str) -> Option<&FunctionDef> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// Format functions for prompt injection
    pub fn to_prompt_format(&self) -> String {
        if self.functions.is_empty() {
            return String::new();
        }

        let mut output = String::from("\n## Available Functions:\n");
        output.push_str("You can call these functions using the format: [TOOL:function_name:{\"param\": \"value\"}]\n\n");

        for func in &self.functions {
            output.push_str(&format!("### {}\n", func.name));
            output.push_str(&format!("{}\n", func.description));
            
            if !func.parameters.properties.is_empty() {
                output.push_str("Parameters:\n");
                for (name, prop) in &func.parameters.properties {
                    let required = if func.parameters.required.contains(name) { " (required)" } else { "" };
                    output.push_str(&format!("  - {}: {}{}\n", name, prop.description, required));
                }
            }
            output.push_str("\n");
        }

        output
    }

    /// Create default embedded systems functions using the builder pattern
    pub fn with_embedded_functions() -> Self {
        let mut registry = Self::new();

        // Add all standard functions from function_calling module
        for func in get_standard_functions() {
            registry.register(func);
        }

        // Add some additional functions
        registry.register(
            FunctionDef::new("read_file", "Read the contents of a file")
                .with_param("path", PropertyDef::string("Path to the file to read"), true)
        );

        registry.register(
            FunctionDef::new("write_file", "Write content to a file")
                .with_param("path", PropertyDef::string("Path to the file"), true)
                .with_param("content", PropertyDef::string("Content to write"), true)
        );

        registry.register(
            FunctionDef::new("analyze_code", "Analyze code for issues and improvements")
                .with_param("code", PropertyDef::string("Code to analyze"), true)
                .with_param("language", PropertyDef::string("Programming language"), false)
        );

        registry
    }
}

impl Default for AgentFunctionRegistry {
    fn default() -> Self {
        Self::with_embedded_functions()
    }
}

// =============================================================================
// ENHANCED AGENT CONTEXT
// =============================================================================

/// Enhanced agent context with RAG, caching, tracing, and function calling
pub struct EnhancedAgentContext {
    // Base context
    base: super::context::AgentContext,
    
    // AI Engine components
    rag_pipeline: Arc<RwLock<RAGPipeline>>,
    cache: Arc<SemanticCache>,
    tracer: Tracer,
    function_registry: Arc<RwLock<AgentFunctionRegistry>>,
    
    // Pending function calls
    pending_calls: Arc<RwLock<Vec<FunctionCall>>>,
    
    // Configuration
    config: EnhancedContextConfig,
}

impl EnhancedAgentContext {
    /// Create a new enhanced context
    pub fn new() -> Self {
        Self::with_config(EnhancedContextConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: EnhancedContextConfig) -> Self {
        Self {
            base: super::context::AgentContext::default(),
            rag_pipeline: Arc::new(RwLock::new(RAGPipeline::new())),
            cache: Arc::new(SemanticCache::new()),
            tracer: Tracer::new("agent-context"),
            function_registry: Arc::new(RwLock::new(AgentFunctionRegistry::with_embedded_functions())),
            pending_calls: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    // === Base Context Access ===

    /// Get base context
    pub fn base(&self) -> &super::context::AgentContext {
        &self.base
    }

    /// Get mutable base context
    pub fn base_mut(&mut self) -> &mut super::context::AgentContext {
        &mut self.base
    }

    /// Add user message
    pub fn add_user_message(&mut self, content: &str) {
        self.base.add_user_message(content);
    }

    /// Add assistant message
    pub fn add_assistant_message(&mut self, content: &str) {
        self.base.add_assistant_message(content);
    }

    // === RAG Pipeline ===

    /// Add a document to the RAG pipeline
    pub async fn add_document(&self, id: &str, content: &str, title: Option<&str>) -> Result<usize, String> {
        use crate::ai::Document;
        
        let document = Document {
            id: id.to_string(),
            title: title.unwrap_or(id).to_string(),
            content: content.to_string(),
            doc_type: crate::ai::DocumentType::Text,
            metadata: std::collections::HashMap::new(),
            indexed_at: None,
        };
        
        let pipeline = self.rag_pipeline.read().await;
        pipeline.index_document(document)
            .await
            .map_err(|e| format!("Failed to add document: {}", e))
    }

    /// Query the RAG pipeline for relevant context
    pub async fn query_rag(&self, query: &str) -> Result<Vec<RetrievedChunk>, String> {
        if !self.config.rag_enabled {
            return Ok(Vec::new());
        }

        let pipeline = self.rag_pipeline.read().await;
        pipeline.query(query)
            .await
            .map(|chunks| chunks.into_iter().take(self.config.max_rag_chunks).collect())
            .map_err(|e| format!("RAG query failed: {}", e))
    }

    /// Get augmented context with RAG results
    pub async fn get_augmented_context(&self, query: &str) -> String {
        let mut context = self.base.to_prompt_context();

        // Add RAG context if enabled
        if self.config.rag_enabled {
            if let Ok(chunks) = self.query_rag(query).await {
                if !chunks.is_empty() {
                    context.push_str("\n## Relevant Documentation:\n");
                    for chunk in chunks {
                        context.push_str(&format!("---\n{}\n", chunk.chunk.content));
                    }
                }
            }
        }

        // Add available functions if enabled
        if self.config.function_calling_enabled {
            let registry = self.function_registry.read().await;
            context.push_str(&registry.to_prompt_format());
        }

        context
    }

    // === Semantic Cache ===

    /// Check cache for a query
    pub async fn check_cache(&self, query: &str) -> Option<String> {
        if !self.config.cache_enabled {
            return None;
        }

        match self.cache.get(query).await {
            Ok(CacheResult::ExactHit(entry)) => Some(entry.response.clone()),
            Ok(CacheResult::SemanticHit { entry, similarity }) if similarity >= 0.92 => {
                Some(entry.response.clone())
            }
            _ => None,
        }
    }

    /// Store response in cache
    pub async fn cache_response(&self, query: &str, response: &str) {
        if self.config.cache_enabled {
            let _ = self.cache.put(query, response, "agent").await;
        }
    }

    // === Tracing ===

    /// Start a trace for an operation
    pub async fn start_trace(&self, operation: &str) -> Option<Span> {
        if self.config.tracing_enabled {
            Some(self.tracer.start_trace(operation).await)
        } else {
            None
        }
    }

    /// End a trace
    pub async fn end_trace(&self, span: Span) {
        if self.config.tracing_enabled {
            self.tracer.end_span(span).await;
        }
    }

    // === Function Calling ===

    /// Get the function registry
    pub async fn get_functions(&self) -> Vec<FunctionDef> {
        let registry = self.function_registry.read().await;
        registry.list().to_vec()
    }

    /// Register a custom function
    pub async fn register_function(&self, func: FunctionDef) {
        let mut registry = self.function_registry.write().await;
        registry.register(func);
    }

    /// Queue a function call for execution
    pub async fn queue_function_call(&self, call: FunctionCall) {
        let mut pending = self.pending_calls.write().await;
        pending.push(call);
    }

    /// Get pending function calls
    pub async fn get_pending_calls(&self) -> Vec<FunctionCall> {
        let pending = self.pending_calls.read().await;
        pending.clone()
    }

    /// Clear pending function calls
    pub async fn clear_pending_calls(&self) {
        let mut pending = self.pending_calls.write().await;
        pending.clear();
    }

    /// Parse tool calls from response and queue them
    pub async fn parse_and_queue_calls(&self, response: &str) -> Vec<FunctionCall> {
        let mut calls = Vec::new();

        if let Ok(re) = regex::Regex::new(r"\[TOOL:(\w+):([^\]]+)\]") {
            for cap in re.captures_iter(response) {
                let tool_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let params_str = cap.get(2).map(|m| m.as_str()).unwrap_or("{}");

                if let Ok(params) = serde_json::from_str(params_str) {
                    let call = FunctionCall {
                        name: tool_name.to_string(),
                        arguments: params,
                    };
                    calls.push(call.clone());
                    self.queue_function_call(call).await;
                }
            }
        }

        calls
    }

    // === Cache Stats ===

    /// Get cache statistics
    pub async fn cache_stats(&self) -> crate::ai::CacheStats {
        self.cache.stats().await
    }
}

impl Default for EnhancedAgentContext {
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
    fn test_function_registry_default() {
        let registry = AgentFunctionRegistry::with_embedded_functions();
        assert!(registry.list().len() >= 4);
        assert!(registry.get("read_file").is_some());
    }

    #[test]
    fn test_function_registry_prompt_format() {
        let registry = AgentFunctionRegistry::with_embedded_functions();
        let prompt = registry.to_prompt_format();
        assert!(prompt.contains("Available Functions"));
        assert!(prompt.contains("read_file"));
    }

    #[test]
    fn test_config_default() {
        let config = EnhancedContextConfig::default();
        assert!(config.rag_enabled);
        assert!(config.cache_enabled);
        assert!(config.tracing_enabled);
        assert!(config.function_calling_enabled);
    }
}
