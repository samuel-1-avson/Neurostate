// AI Service Module
// Multi-LLM integration for code generation and assistance
// Enhanced with RAG, vector memory, guardrails, and production features

pub mod gemini;
pub mod service;
pub mod providers;
pub mod agent_models;
pub mod router;
pub mod vector_store;
pub mod rag_pipeline;
pub mod guardrails;

// Phase 3: Production Hardening
pub mod observability;
pub mod semantic_cache;
pub mod prompt_manager;

// Phase 4: Advanced Capabilities
pub mod function_calling;

pub use service::*;
pub use providers::*;
pub use gemini::GeminiModel;
pub use agent_models::*;
pub use router::*;
pub use vector_store::*;
pub use rag_pipeline::*;
pub use guardrails::*;

// Phase 3 exports
pub use observability::{Tracer, Span, SpanStatus, MetricsCollector, AIMetrics, ObservabilityManager, HealthStatus};
pub use semantic_cache::{SemanticCache, CacheConfig, CacheEntry, CacheResult, CacheStats, CachedModel};
pub use prompt_manager::{PromptManager, PromptTemplate, TemplateVariable, ABTest, PromptAnalytics, create_default_templates};

// Phase 4 exports
pub use function_calling::{
    FunctionDef, FunctionParameters, ParameterDef, ParameterType,
    FunctionCall, FunctionResult, FunctionStatus,
    FunctionHandler, FunctionRegistry, ToolExecutor,
    to_gemini_tools, to_openai_tools, parse_gemini_call, parse_openai_call,
    create_embedded_functions,
};
