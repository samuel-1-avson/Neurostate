// AI Agents Module
// Multi-agent system with specialized AI assistants

pub mod agent;
pub mod context;
pub mod orchestrator;
pub mod tools;
pub mod director;
pub mod fsm_agent;
pub mod code_agent;
pub mod debug_agent;
pub mod hardware_agent;
pub mod docs_agent;
pub mod canvas_agent;
pub mod build_agent;
pub mod deploy_agent;
pub mod typed_tools;
pub mod diff_engine;
pub mod function_calling;
pub mod function_executor;
pub mod memory;
pub mod memory_storage;
pub mod task_queue;
pub mod plan_executor;
pub mod streaming;
pub mod collaboration;
pub mod voice_agent;
pub mod super_agent;

#[cfg(test)]
mod tests;

pub use agent::*;
pub use context::*;
pub use orchestrator::*;
pub use tools::*;
pub use director::{DirectorAgent, Intent, Task, TaskPlan, TaskStatus};
pub use canvas_agent::CanvasAgent;
pub use build_agent::BuildAgent;
pub use deploy_agent::DeployAgent;
pub use typed_tools::{ToolDef, ToolRegistry, ToolContext, ToolPermission, ToolCategory, create_default_registry};
pub use diff_engine::{Patch, PatchTarget, PatchOperations, JsonPatchOp, DiffHunk, AuditLog};
pub use function_calling::{FunctionDef, FunctionCall, FunctionResult, PropertyDef, OpenAITool, GeminiTool};
pub use function_executor::FunctionExecutor;
pub use memory::{AgentMemory, MemoryManager, MemoryEntry, MemoryCategory, ShortTermMemory, WorkingMemory, LongTermMemory};
pub use memory_storage::{MemoryStorage, PersistedMemoryManager, MemoryStats};
pub use task_queue::{TaskQueue, BackgroundTask, TaskProgress, TaskSummary, TaskType, TaskStatus as QueueTaskStatus, TaskPriority};
pub use plan_executor::{PlanExecutor, PlanResult, StepResult};
pub use streaming::{StreamManager, StreamChunk, StreamEvent, StreamMetadata, StreamingResponse};
pub use collaboration::{CollaborationManager, CollaborationSession, AgentMessage, MessageType, CollaborationProtocol};
pub use voice_agent::{VoiceAssistantAgent, VoiceCommand};
pub use super_agent::{SuperAgent, TaskBreakdown, AgentTask, TaskInput, IntentCategory, ExecutionStrategy};
