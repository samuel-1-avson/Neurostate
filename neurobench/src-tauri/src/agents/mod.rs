// AI Agents Module
// Multi-agent system with specialized AI assistants

pub mod agent;
pub mod context;
pub mod orchestrator;
pub mod tools;
pub mod fsm_agent;
pub mod code_agent;
pub mod debug_agent;
pub mod hardware_agent;
pub mod docs_agent;
pub mod typed_tools;
pub mod diff_engine;

#[cfg(test)]
mod tests;

pub use agent::*;
pub use context::*;
pub use orchestrator::*;
pub use tools::*;
pub use typed_tools::{ToolDef, ToolRegistry, ToolContext, ToolPermission, ToolCategory, create_default_registry};
pub use diff_engine::{Patch, PatchTarget, PatchOperations, JsonPatchOp, DiffHunk, AuditLog};
