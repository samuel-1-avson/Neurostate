// Agent Tools - Object-Safe, Type-Erased Tool System
// 
// This implements a tool registry where:
// - Tools define JSON schemas for input/output (via schemars)
// - Execution is type-erased to serde_json::Value
// - Tools can be dynamically registered and discovered
// - Permissions are enforced at runtime

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Tool execution result
pub type ToolResult = Result<Value, ToolError>;

/// Tool errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

impl ToolError {
    pub fn validation(msg: impl Into<String>) -> Self {
        Self {
            code: "VALIDATION_ERROR".to_string(),
            message: msg.into(),
            recoverable: true,
        }
    }
    
    pub fn execution(msg: impl Into<String>) -> Self {
        Self {
            code: "EXECUTION_ERROR".to_string(),
            message: msg.into(),
            recoverable: false,
        }
    }
    
    pub fn permission(msg: impl Into<String>) -> Self {
        Self {
            code: "PERMISSION_DENIED".to_string(),
            message: msg.into(),
            recoverable: false,
        }
    }
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for ToolError {}

/// JSON Schema representation (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, JsonSchema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<JsonSchema>>,
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<Value>>,
}

impl JsonSchema {
    pub fn string() -> Self {
        Self {
            schema_type: "string".to_string(),
            description: None,
            properties: None,
            required: None,
            items: None,
            enum_values: None,
        }
    }
    
    pub fn number() -> Self {
        Self {
            schema_type: "number".to_string(),
            ..Self::string()
        }
    }
    
    pub fn boolean() -> Self {
        Self {
            schema_type: "boolean".to_string(),
            ..Self::string()
        }
    }
    
    pub fn object() -> Self {
        Self {
            schema_type: "object".to_string(),
            properties: Some(HashMap::new()),
            required: Some(Vec::new()),
            ..Self::string()
        }
    }
    
    pub fn array(items: JsonSchema) -> Self {
        Self {
            schema_type: "array".to_string(),
            items: Some(Box::new(items)),
            ..Self::string()
        }
    }
    
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
    
    pub fn with_property(mut self, name: impl Into<String>, schema: JsonSchema, required: bool) -> Self {
        let name = name.into();
        if let Some(ref mut props) = self.properties {
            props.insert(name.clone(), schema);
        }
        if required {
            if let Some(ref mut req) = self.required {
                req.push(name);
            }
        }
        self
    }
}

/// Tool definition - the core unit of the tool system
#[derive(Clone)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub category: ToolCategory,
    pub input_schema: JsonSchema,
    pub output_schema: JsonSchema,
    pub required_permissions: Vec<ToolPermission>,
    handler: Arc<dyn Fn(Value, &ToolContext) -> ToolResult + Send + Sync>,
}

impl ToolDef {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: JsonSchema,
        output_schema: JsonSchema,
        handler: impl Fn(Value, &ToolContext) -> ToolResult + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            category: ToolCategory::General,
            input_schema,
            output_schema,
            required_permissions: vec![],
            handler: Arc::new(handler),
        }
    }
    
    pub fn with_category(mut self, category: ToolCategory) -> Self {
        self.category = category;
        self
    }
    
    pub fn with_permissions(mut self, permissions: Vec<ToolPermission>) -> Self {
        self.required_permissions = permissions;
        self
    }
    
    /// Execute the tool with the given input
    pub fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        // Check permissions
        for perm in &self.required_permissions {
            if !ctx.has_permission(perm) {
                return Err(ToolError::permission(format!(
                    "Tool '{}' requires permission: {:?}", self.name, perm
                )));
            }
        }
        
        // Execute handler
        (self.handler)(input, ctx)
    }
}

// Manual Debug implementation since handler is not Debug
impl std::fmt::Debug for ToolDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolDef")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("category", &self.category)
            .finish()
    }
}

/// Tool categories for organization
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    General,
    FSM,
    Peripheral,
    Build,
    Debug,
    Documentation,
}

/// Permissions for tool execution
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ToolPermission {
    ReadFSM,
    WriteFSM,
    ReadConfig,
    WriteConfig,
    ReadCode,
    WriteCode,
    RunBuild,
    RunFlash,
    RunTerminal,
    AccessNetwork,
}

/// Context provided to tool execution
#[derive(Debug, Clone)]
pub struct ToolContext {
    pub agent_id: String,
    pub permissions: std::collections::HashSet<ToolPermission>,
    pub project_path: Option<std::path::PathBuf>,
    pub variables: HashMap<String, Value>,
}

impl ToolContext {
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            permissions: std::collections::HashSet::new(),
            project_path: None,
            variables: HashMap::new(),
        }
    }
    
    pub fn with_permissions(mut self, perms: impl IntoIterator<Item = ToolPermission>) -> Self {
        self.permissions = perms.into_iter().collect();
        self
    }
    
    pub fn has_permission(&self, perm: &ToolPermission) -> bool {
        self.permissions.contains(perm)
    }
}

/// Tool registry - holds all registered tools
pub struct ToolRegistry {
    tools: HashMap<String, ToolDef>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }
    
    /// Register a tool
    pub fn register(&mut self, tool: ToolDef) {
        self.tools.insert(tool.name.clone(), tool);
    }
    
    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&ToolDef> {
        self.tools.get(name)
    }
    
    /// List all tools
    pub fn list(&self) -> Vec<&ToolDef> {
        self.tools.values().collect()
    }
    
    /// List tools by category
    pub fn list_by_category(&self, category: ToolCategory) -> Vec<&ToolDef> {
        self.tools.values()
            .filter(|t| t.category == category)
            .collect()
    }
    
    /// Execute a tool by name
    pub fn execute(&self, name: &str, input: Value, ctx: &ToolContext) -> ToolResult {
        let tool = self.get(name)
            .ok_or_else(|| ToolError::validation(format!("Unknown tool: {}", name)))?;
        tool.execute(input, ctx)
    }
    
    /// Get schemas for all tools (for AI function calling)
    pub fn get_schemas(&self) -> Vec<Value> {
        self.tools.values().map(|t| {
            serde_json::json!({
                "name": t.name,
                "description": t.description,
                "parameters": t.input_schema,
            })
        }).collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== Built-in Tools ====================

/// Create the default tool registry with built-in tools
pub fn create_default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    
    // FSM Tools
    registry.register(create_add_state_tool());
    registry.register(create_remove_state_tool());
    registry.register(create_add_transition_tool());
    registry.register(create_update_state_tool());
    
    // Analysis Tools
    registry.register(create_validate_fsm_tool());
    registry.register(create_analyze_fsm_tool());
    
    // Hardware Tools
    registry.register(create_get_pinout_tool());
    registry.register(create_check_pin_conflict_tool());
    
    // Build Tools
    registry.register(create_run_build_tool());
    
    registry
}

fn create_add_state_tool() -> ToolDef {
    ToolDef::new(
        "add_state",
        "Add a new state to the FSM",
        JsonSchema::object()
            .with_property("label", JsonSchema::string().with_description("State name"), true)
            .with_property("state_type", JsonSchema::string().with_description("State type: input, process, output, decision"), true)
            .with_property("entry_action", JsonSchema::string().with_description("Code to run on state entry"), false)
            .with_property("exit_action", JsonSchema::string().with_description("Code to run on state exit"), false)
            .with_property("x", JsonSchema::number().with_description("X position"), false)
            .with_property("y", JsonSchema::number().with_description("Y position"), false),
        JsonSchema::object()
            .with_property("state_id", JsonSchema::string(), true)
            .with_property("success", JsonSchema::boolean(), true),
        |input, _ctx| {
            let label = input.get("label")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::validation("Missing 'label' field"))?;
            
            // In real implementation, this would modify the FSM graph
            Ok(serde_json::json!({
                "state_id": format!("state_{}", uuid::Uuid::new_v4()),
                "success": true,
                "message": format!("Added state '{}'", label)
            }))
        },
    )
    .with_category(ToolCategory::FSM)
    .with_permissions(vec![ToolPermission::WriteFSM])
}

fn create_remove_state_tool() -> ToolDef {
    ToolDef::new(
        "remove_state",
        "Remove a state from the FSM",
        JsonSchema::object()
            .with_property("state_id", JsonSchema::string().with_description("ID of state to remove"), true),
        JsonSchema::object()
            .with_property("success", JsonSchema::boolean(), true),
        |input, _ctx| {
            let state_id = input.get("state_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::validation("Missing 'state_id' field"))?;
            
            Ok(serde_json::json!({
                "success": true,
                "message": format!("Removed state '{}'", state_id)
            }))
        },
    )
    .with_category(ToolCategory::FSM)
    .with_permissions(vec![ToolPermission::WriteFSM])
}

fn create_add_transition_tool() -> ToolDef {
    ToolDef::new(
        "add_transition",
        "Add a transition between states",
        JsonSchema::object()
            .with_property("from_state", JsonSchema::string().with_description("Source state ID"), true)
            .with_property("to_state", JsonSchema::string().with_description("Target state ID"), true)
            .with_property("event", JsonSchema::string().with_description("Trigger event name"), false)
            .with_property("guard", JsonSchema::string().with_description("Guard condition expression"), false)
            .with_property("action", JsonSchema::string().with_description("Code to execute on transition"), false),
        JsonSchema::object()
            .with_property("transition_id", JsonSchema::string(), true)
            .with_property("success", JsonSchema::boolean(), true),
        |input, _ctx| {
            let from = input.get("from_state").and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::validation("Missing 'from_state'"))?;
            let to = input.get("to_state").and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::validation("Missing 'to_state'"))?;
            
            Ok(serde_json::json!({
                "transition_id": format!("edge_{}", uuid::Uuid::new_v4()),
                "success": true,
                "message": format!("Added transition from '{}' to '{}'", from, to)
            }))
        },
    )
    .with_category(ToolCategory::FSM)
    .with_permissions(vec![ToolPermission::WriteFSM])
}

fn create_update_state_tool() -> ToolDef {
    ToolDef::new(
        "update_state",
        "Update properties of an existing state",
        JsonSchema::object()
            .with_property("state_id", JsonSchema::string().with_description("State ID to update"), true)
            .with_property("label", JsonSchema::string().with_description("New state name"), false)
            .with_property("entry_action", JsonSchema::string().with_description("New entry action code"), false)
            .with_property("exit_action", JsonSchema::string().with_description("New exit action code"), false),
        JsonSchema::object()
            .with_property("success", JsonSchema::boolean(), true),
        |input, _ctx| {
            let state_id = input.get("state_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::validation("Missing 'state_id'"))?;
            
            Ok(serde_json::json!({
                "success": true,
                "message": format!("Updated state '{}'", state_id)
            }))
        },
    )
    .with_category(ToolCategory::FSM)
    .with_permissions(vec![ToolPermission::WriteFSM])
}

fn create_validate_fsm_tool() -> ToolDef {
    ToolDef::new(
        "validate_fsm",
        "Validate the FSM for errors and warnings",
        JsonSchema::object(),
        JsonSchema::object()
            .with_property("valid", JsonSchema::boolean(), true)
            .with_property("errors", JsonSchema::array(JsonSchema::string()), true)
            .with_property("warnings", JsonSchema::array(JsonSchema::string()), true),
        |_input, _ctx| {
            // In real implementation, this would run validation checks
            Ok(serde_json::json!({
                "valid": true,
                "errors": [],
                "warnings": [],
                "checks_run": [
                    "unreachable_states",
                    "dead_ends",
                    "nondeterministic_transitions",
                    "missing_initial_state"
                ]
            }))
        },
    )
    .with_category(ToolCategory::FSM)
    .with_permissions(vec![ToolPermission::ReadFSM])
}

fn create_analyze_fsm_tool() -> ToolDef {
    ToolDef::new(
        "analyze_fsm",
        "Perform deep analysis of the FSM structure",
        JsonSchema::object()
            .with_property("analysis_type", JsonSchema::string().with_description("Type: complexity, coverage, timing"), false),
        JsonSchema::object()
            .with_property("analysis", JsonSchema::object(), true),
        |_input, _ctx| {
            Ok(serde_json::json!({
                "analysis": {
                    "state_count": 5,
                    "transition_count": 7,
                    "cyclomatic_complexity": 3,
                    "max_depth": 4,
                    "has_initial_state": true,
                    "has_final_state": true
                }
            }))
        },
    )
    .with_category(ToolCategory::FSM)
    .with_permissions(vec![ToolPermission::ReadFSM])
}

fn create_get_pinout_tool() -> ToolDef {
    ToolDef::new(
        "get_pinout",
        "Get the pinout for the selected MCU",
        JsonSchema::object()
            .with_property("mcu_id", JsonSchema::string().with_description("MCU identifier"), false),
        JsonSchema::object()
            .with_property("pins", JsonSchema::array(JsonSchema::object()), true),
        |_input, _ctx| {
            Ok(serde_json::json!({
                "mcu": "STM32F407VG",
                "pins": [
                    {"port": "A", "pin": 0, "functions": ["GPIO", "TIM2_CH1", "USART2_CTS"]},
                    {"port": "A", "pin": 1, "functions": ["GPIO", "TIM2_CH2", "USART2_RTS"]}
                ]
            }))
        },
    )
    .with_category(ToolCategory::Peripheral)
    .with_permissions(vec![ToolPermission::ReadConfig])
}

fn create_check_pin_conflict_tool() -> ToolDef {
    ToolDef::new(
        "check_pin_conflict",
        "Check for pin conflicts in peripheral configuration",
        JsonSchema::object()
            .with_property("pin", JsonSchema::string().with_description("Pin to check (e.g., PA5)"), true)
            .with_property("function", JsonSchema::string().with_description("Desired function"), true),
        JsonSchema::object()
            .with_property("conflict", JsonSchema::boolean(), true)
            .with_property("conflicting_peripheral", JsonSchema::string(), false),
        |input, _ctx| {
            let pin = input.get("pin").and_then(|v| v.as_str()).unwrap_or("PA0");
            let func = input.get("function").and_then(|v| v.as_str()).unwrap_or("");
            
            Ok(serde_json::json!({
                "pin": pin,
                "requested_function": func,
                "conflict": false,
                "message": "No conflicts detected"
            }))
        },
    )
    .with_category(ToolCategory::Peripheral)
    .with_permissions(vec![ToolPermission::ReadConfig])
}

fn create_run_build_tool() -> ToolDef {
    ToolDef::new(
        "run_build",
        "Trigger a firmware build",
        JsonSchema::object()
            .with_property("clean", JsonSchema::boolean().with_description("Clean before build"), false)
            .with_property("target", JsonSchema::string().with_description("Build target"), false),
        JsonSchema::object()
            .with_property("success", JsonSchema::boolean(), true)
            .with_property("build_id", JsonSchema::string(), true),
        |_input, _ctx| {
            // This would trigger an async build and return immediately
            Ok(serde_json::json!({
                "success": true,
                "build_id": format!("build_{}", uuid::Uuid::new_v4()),
                "message": "Build started. Monitor 'build:output' events for progress."
            }))
        },
    )
    .with_category(ToolCategory::Build)
    .with_permissions(vec![ToolPermission::RunBuild])
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tool_registry() {
        let registry = create_default_registry();
        
        assert!(registry.get("add_state").is_some());
        assert!(registry.get("validate_fsm").is_some());
        assert!(registry.get("unknown_tool").is_none());
    }
    
    #[test]
    fn test_tool_execution() {
        let registry = create_default_registry();
        let ctx = ToolContext::new("test_agent")
            .with_permissions(vec![ToolPermission::WriteFSM]);
        
        let input = serde_json::json!({
            "label": "INIT",
            "state_type": "input"
        });
        
        let result = registry.execute("add_state", input, &ctx);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.get("success").and_then(|v| v.as_bool()).unwrap_or(false));
    }
    
    #[test]
    fn test_permission_denied() {
        let registry = create_default_registry();
        let ctx = ToolContext::new("test_agent"); // No permissions
        
        let input = serde_json::json!({"label": "TEST", "state_type": "process"});
        let result = registry.execute("add_state", input, &ctx);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "PERMISSION_DENIED");
    }
    
    #[test]
    fn test_get_schemas() {
        let registry = create_default_registry();
        let schemas = registry.get_schemas();
        
        assert!(!schemas.is_empty());
        assert!(schemas.iter().any(|s| s.get("name").and_then(|v| v.as_str()) == Some("add_state")));
    }
}
