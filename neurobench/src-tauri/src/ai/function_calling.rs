//! Native Function Calling - Structured Tool Use
//!
//! Provides native function calling support for AI models:
//! - Gemini tools API integration
//! - OpenAI function_call support
//! - Multi-turn execution with tool results
//! - Type-safe function definitions

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// =============================================================================
// FUNCTION DEFINITIONS
// =============================================================================

/// A callable function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDef {
    /// Function name (must be unique)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Parameters schema (JSON Schema format)
    pub parameters: FunctionParameters,
    /// Whether the function requires confirmation
    pub requires_confirmation: bool,
    /// Category for organization
    pub category: Option<String>,
}

impl FunctionDef {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            parameters: FunctionParameters::default(),
            requires_confirmation: false,
            category: None,
        }
    }

    pub fn with_param(mut self, name: &str, param: ParameterDef) -> Self {
        self.parameters.properties.insert(name.to_string(), param);
        self
    }

    pub fn with_required(mut self, name: &str) -> Self {
        self.parameters.required.push(name.to_string());
        self
    }

    pub fn with_confirmation(mut self) -> Self {
        self.requires_confirmation = true;
        self
    }

    pub fn with_category(mut self, category: &str) -> Self {
        self.category = Some(category.to_string());
        self
    }
}

/// Function parameters schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FunctionParameters {
    #[serde(default = "default_object_type")]
    pub r#type: String,
    pub properties: HashMap<String, ParameterDef>,
    #[serde(default)]
    pub required: Vec<String>,
}

fn default_object_type() -> String {
    "object".to_string()
}

/// Parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDef {
    pub r#type: ParameterType,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
}

impl ParameterDef {
    pub fn string(description: &str) -> Self {
        Self {
            r#type: ParameterType::String,
            description: description.to_string(),
            r#enum: None,
            default: None,
            minimum: None,
            maximum: None,
        }
    }

    pub fn number(description: &str) -> Self {
        Self {
            r#type: ParameterType::Number,
            description: description.to_string(),
            r#enum: None,
            default: None,
            minimum: None,
            maximum: None,
        }
    }

    pub fn integer(description: &str) -> Self {
        Self {
            r#type: ParameterType::Integer,
            description: description.to_string(),
            r#enum: None,
            default: None,
            minimum: None,
            maximum: None,
        }
    }

    pub fn boolean(description: &str) -> Self {
        Self {
            r#type: ParameterType::Boolean,
            description: description.to_string(),
            r#enum: None,
            default: None,
            minimum: None,
            maximum: None,
        }
    }

    pub fn array(description: &str) -> Self {
        Self {
            r#type: ParameterType::Array,
            description: description.to_string(),
            r#enum: None,
            default: None,
            minimum: None,
            maximum: None,
        }
    }

    pub fn with_enum(mut self, values: Vec<&str>) -> Self {
        self.r#enum = Some(values.into_iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn with_default(mut self, value: Value) -> Self {
        self.default = Some(value);
        self
    }

    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.minimum = Some(min);
        self.maximum = Some(max);
        self
    }
}

/// Parameter types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
}

// =============================================================================
// FUNCTION CALLS
// =============================================================================

/// A function call requested by the model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Call ID (for tracking)
    pub id: String,
    /// Function name
    pub name: String,
    /// Arguments (parsed JSON)
    pub arguments: Value,
}

impl FunctionCall {
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.arguments.get(key)?.as_str().map(|s| s.to_string())
    }

    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.arguments.get(key)?.as_f64()
    }

    pub fn get_integer(&self, key: &str) -> Option<i64> {
        self.arguments.get(key)?.as_i64()
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.arguments.get(key)?.as_bool()
    }

    pub fn get_array(&self, key: &str) -> Option<&Vec<Value>> {
        self.arguments.get(key)?.as_array()
    }
}

/// Result of a function execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionResult {
    /// Call ID this result corresponds to
    pub call_id: String,
    /// Function name
    pub name: String,
    /// Result status
    pub status: FunctionStatus,
    /// Result value (if successful)
    pub value: Option<Value>,
    /// Error message (if failed)
    pub error: Option<String>,
}

impl FunctionResult {
    pub fn success(call: &FunctionCall, value: Value) -> Self {
        Self {
            call_id: call.id.clone(),
            name: call.name.clone(),
            status: FunctionStatus::Success,
            value: Some(value),
            error: None,
        }
    }

    pub fn error(call: &FunctionCall, error: &str) -> Self {
        Self {
            call_id: call.id.clone(),
            name: call.name.clone(),
            status: FunctionStatus::Error,
            value: None,
            error: Some(error.to_string()),
        }
    }

    pub fn pending(call: &FunctionCall) -> Self {
        Self {
            call_id: call.id.clone(),
            name: call.name.clone(),
            status: FunctionStatus::Pending,
            value: None,
            error: None,
        }
    }

    pub fn requires_confirmation(call: &FunctionCall) -> Self {
        Self {
            call_id: call.id.clone(),
            name: call.name.clone(),
            status: FunctionStatus::RequiresConfirmation,
            value: None,
            error: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FunctionStatus {
    Success,
    Error,
    Pending,
    RequiresConfirmation,
    Cancelled,
}

// =============================================================================
// FUNCTION HANDLER
// =============================================================================

/// Trait for implementing function handlers
#[async_trait]
pub trait FunctionHandler: Send + Sync {
    /// Get the function definition
    fn definition(&self) -> FunctionDef;
    
    /// Execute the function with given arguments
    async fn execute(&self, call: &FunctionCall) -> FunctionResult;
    
    /// Validate arguments before execution
    fn validate(&self, call: &FunctionCall) -> Result<(), String> {
        // Default: no extra validation
        let _ = call;
        Ok(())
    }
}

/// Dynamic function handler using closures
pub struct DynamicHandler {
    definition: FunctionDef,
    handler: Box<dyn Fn(&FunctionCall) -> FunctionResult + Send + Sync>,
}

impl DynamicHandler {
    pub fn new<F>(definition: FunctionDef, handler: F) -> Self
    where
        F: Fn(&FunctionCall) -> FunctionResult + Send + Sync + 'static,
    {
        Self {
            definition,
            handler: Box::new(handler),
        }
    }
}

#[async_trait]
impl FunctionHandler for DynamicHandler {
    fn definition(&self) -> FunctionDef {
        self.definition.clone()
    }

    async fn execute(&self, call: &FunctionCall) -> FunctionResult {
        (self.handler)(call)
    }
}

// =============================================================================
// FUNCTION REGISTRY
// =============================================================================

/// Registry of available functions
pub struct FunctionRegistry {
    /// Registered functions by name
    functions: RwLock<HashMap<String, Arc<dyn FunctionHandler>>>,
    /// Function categories
    categories: RwLock<HashMap<String, Vec<String>>>,
}

impl FunctionRegistry {
    pub fn new() -> Self {
        Self {
            functions: RwLock::new(HashMap::new()),
            categories: RwLock::new(HashMap::new()),
        }
    }

    /// Register a function handler
    pub async fn register(&self, handler: Arc<dyn FunctionHandler>) {
        let def = handler.definition();
        let name = def.name.clone();
        
        // Add to functions
        {
            let mut funcs = self.functions.write().await;
            funcs.insert(name.clone(), handler);
        }

        // Add to category
        if let Some(category) = def.category {
            let mut cats = self.categories.write().await;
            cats.entry(category).or_default().push(name);
        }
    }

    /// Register a simple function with closure
    pub async fn register_simple<F>(&self, definition: FunctionDef, handler: F)
    where
        F: Fn(&FunctionCall) -> FunctionResult + Send + Sync + 'static,
    {
        let dynamic = Arc::new(DynamicHandler::new(definition, handler));
        self.register(dynamic).await;
    }

    /// Get a function handler by name
    pub async fn get(&self, name: &str) -> Option<Arc<dyn FunctionHandler>> {
        let funcs = self.functions.read().await;
        funcs.get(name).cloned()
    }

    /// Get all function definitions
    pub async fn definitions(&self) -> Vec<FunctionDef> {
        let funcs = self.functions.read().await;
        funcs.values().map(|h| h.definition()).collect()
    }

    /// Get functions by category
    pub async fn by_category(&self, category: &str) -> Vec<FunctionDef> {
        let cats = self.categories.read().await;
        let funcs = self.functions.read().await;
        
        cats.get(category)
            .map(|names| {
                names.iter()
                    .filter_map(|n| funcs.get(n).map(|h| h.definition()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Execute a function call
    pub async fn execute(&self, call: &FunctionCall) -> FunctionResult {
        let handler = {
            let funcs = self.functions.read().await;
            funcs.get(&call.name).cloned()
        };

        match handler {
            Some(h) => {
                // Validate first
                if let Err(e) = h.validate(call) {
                    return FunctionResult::error(call, &e);
                }

                // Check if confirmation required
                let def = h.definition();
                if def.requires_confirmation {
                    return FunctionResult::requires_confirmation(call);
                }

                // Execute
                h.execute(call).await
            }
            None => FunctionResult::error(call, &format!("Unknown function: {}", call.name)),
        }
    }

    /// List all function names
    pub async fn list(&self) -> Vec<String> {
        let funcs = self.functions.read().await;
        funcs.keys().cloned().collect()
    }

    /// List all categories
    pub async fn list_categories(&self) -> Vec<String> {
        let cats = self.categories.read().await;
        cats.keys().cloned().collect()
    }

    /// Unregister a function
    pub async fn unregister(&self, name: &str) {
        let mut funcs = self.functions.write().await;
        funcs.remove(name);
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// TOOL USE EXECUTOR
// =============================================================================

/// Multi-turn tool use executor
pub struct ToolExecutor {
    /// Function registry
    registry: Arc<FunctionRegistry>,
    /// Maximum tool calls per turn
    max_calls_per_turn: usize,
    /// Maximum total turns
    max_turns: usize,
    /// Pending confirmations
    pending_confirmations: RwLock<HashMap<String, FunctionCall>>,
}

impl ToolExecutor {
    pub fn new(registry: Arc<FunctionRegistry>) -> Self {
        Self {
            registry,
            max_calls_per_turn: 10,
            max_turns: 5,
            pending_confirmations: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_limits(mut self, max_calls: usize, max_turns: usize) -> Self {
        self.max_calls_per_turn = max_calls;
        self.max_turns = max_turns;
        self
    }

    /// Execute multiple function calls
    pub async fn execute_batch(&self, calls: Vec<FunctionCall>) -> Vec<FunctionResult> {
        let mut results = Vec::with_capacity(calls.len());
        
        // Limit number of calls
        let calls: Vec<_> = calls.into_iter().take(self.max_calls_per_turn).collect();

        for call in calls {
            let result = self.registry.execute(&call).await;
            
            // Track pending confirmations
            if result.status == FunctionStatus::RequiresConfirmation {
                let mut pending = self.pending_confirmations.write().await;
                pending.insert(call.id.clone(), call);
            }
            
            results.push(result);
        }

        results
    }

    /// Confirm a pending function call
    pub async fn confirm(&self, call_id: &str) -> Option<FunctionResult> {
        let call = {
            let mut pending = self.pending_confirmations.write().await;
            pending.remove(call_id)?
        };

        // Get handler and execute (bypass confirmation check)
        let handler = self.registry.get(&call.name).await?;
        Some(handler.execute(&call).await)
    }

    /// Cancel a pending function call
    pub async fn cancel(&self, call_id: &str) -> Option<FunctionResult> {
        let call = {
            let mut pending = self.pending_confirmations.write().await;
            pending.remove(call_id)?
        };

        Some(FunctionResult {
            call_id: call.id,
            name: call.name,
            status: FunctionStatus::Cancelled,
            value: None,
            error: Some("Cancelled by user".to_string()),
        })
    }

    /// Get pending confirmations
    pub async fn pending(&self) -> Vec<FunctionCall> {
        let pending = self.pending_confirmations.read().await;
        pending.values().cloned().collect()
    }
}

// =============================================================================
// GEMINI FORMAT CONVERTER
// =============================================================================

/// Convert to Gemini tools format
pub fn to_gemini_tools(functions: &[FunctionDef]) -> Value {
    serde_json::json!({
        "function_declarations": functions.iter().map(|f| {
            serde_json::json!({
                "name": f.name,
                "description": f.description,
                "parameters": {
                    "type": "object",
                    "properties": f.parameters.properties.iter().map(|(k, v)| {
                        (k.clone(), serde_json::json!({
                            "type": format!("{:?}", v.r#type).to_lowercase(),
                            "description": v.description,
                            "enum": v.r#enum,
                        }))
                    }).collect::<HashMap<_, _>>(),
                    "required": f.parameters.required,
                }
            })
        }).collect::<Vec<_>>()
    })
}

/// Parse Gemini function call response
pub fn parse_gemini_call(response: &Value) -> Option<FunctionCall> {
    let parts = response.get("candidates")?
        .get(0)?
        .get("content")?
        .get("parts")?;
    
    let function_call = parts.as_array()?
        .iter()
        .find_map(|p| p.get("functionCall"))?;
    
    Some(FunctionCall {
        id: generate_call_id(),
        name: function_call.get("name")?.as_str()?.to_string(),
        arguments: function_call.get("args")?.clone(),
    })
}

// =============================================================================
// OPENAI FORMAT CONVERTER
// =============================================================================

/// Convert to OpenAI tools format
pub fn to_openai_tools(functions: &[FunctionDef]) -> Vec<Value> {
    functions.iter().map(|f| {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": f.name,
                "description": f.description,
                "parameters": {
                    "type": "object",
                    "properties": f.parameters.properties.iter().map(|(k, v)| {
                        (k.clone(), serde_json::json!({
                            "type": format!("{:?}", v.r#type).to_lowercase(),
                            "description": v.description,
                            "enum": v.r#enum,
                        }))
                    }).collect::<HashMap<_, _>>(),
                    "required": f.parameters.required,
                }
            }
        })
    }).collect()
}

/// Parse OpenAI function call response
pub fn parse_openai_call(response: &Value) -> Option<FunctionCall> {
    let tool_calls = response.get("choices")?
        .get(0)?
        .get("message")?
        .get("tool_calls")?
        .as_array()?;
    
    let tool_call = tool_calls.first()?;
    let function = tool_call.get("function")?;
    
    let arguments: Value = function.get("arguments")?
        .as_str()
        .and_then(|s| serde_json::from_str(s).ok())?;
    
    Some(FunctionCall {
        id: tool_call.get("id")?.as_str()?.to_string(),
        name: function.get("name")?.as_str()?.to_string(),
        arguments,
    })
}

// =============================================================================
// BUILT-IN FUNCTIONS
// =============================================================================

/// Create built-in functions for embedded systems
pub fn create_embedded_functions() -> Vec<(FunctionDef, Box<dyn Fn(&FunctionCall) -> FunctionResult + Send + Sync>)> {
    vec![
        // Read file
        (
            FunctionDef::new("read_file", "Read the contents of a file")
                .with_param("path", ParameterDef::string("Path to the file to read"))
                .with_required("path")
                .with_category("filesystem"),
            Box::new(|call: &FunctionCall| {
                let path = call.get_string("path").unwrap_or_default();
                match std::fs::read_to_string(&path) {
                    Ok(content) => FunctionResult::success(call, serde_json::json!({ "content": content })),
                    Err(e) => FunctionResult::error(call, &e.to_string()),
                }
            }),
        ),
        // Write file
        (
            FunctionDef::new("write_file", "Write content to a file")
                .with_param("path", ParameterDef::string("Path to the file to write"))
                .with_param("content", ParameterDef::string("Content to write"))
                .with_required("path")
                .with_required("content")
                .with_category("filesystem")
                .with_confirmation(),
            Box::new(|call: &FunctionCall| {
                let path = call.get_string("path").unwrap_or_default();
                let content = call.get_string("content").unwrap_or_default();
                match std::fs::write(&path, &content) {
                    Ok(_) => FunctionResult::success(call, serde_json::json!({ "written": true, "path": path })),
                    Err(e) => FunctionResult::error(call, &e.to_string()),
                }
            }),
        ),
        // List directory
        (
            FunctionDef::new("list_directory", "List files in a directory")
                .with_param("path", ParameterDef::string("Path to the directory"))
                .with_required("path")
                .with_category("filesystem"),
            Box::new(|call: &FunctionCall| {
                let path = call.get_string("path").unwrap_or_default();
                match std::fs::read_dir(&path) {
                    Ok(entries) => {
                        let files: Vec<String> = entries
                            .filter_map(|e| e.ok())
                            .map(|e| e.file_name().to_string_lossy().to_string())
                            .collect();
                        FunctionResult::success(call, serde_json::json!({ "files": files }))
                    }
                    Err(e) => FunctionResult::error(call, &e.to_string()),
                }
            }),
        ),
        // Generate FSM code
        (
            FunctionDef::new("generate_fsm", "Generate a finite state machine")
                .with_param("name", ParameterDef::string("Name of the FSM"))
                .with_param("states", ParameterDef::array("List of state names"))
                .with_param("initial_state", ParameterDef::string("Initial state name"))
                .with_required("name")
                .with_required("states")
                .with_category("code_generation"),
            Box::new(|call: &FunctionCall| {
                let name = call.get_string("name").unwrap_or("StateMachine".to_string());
                let states = call.get_array("states")
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                    .unwrap_or_default();
                let initial = call.get_string("initial_state").unwrap_or_else(|| {
                    states.first().map(|s| s.to_string()).unwrap_or("Idle".to_string())
                });
                
                let code = generate_fsm_code(&name, &states, &initial);
                FunctionResult::success(call, serde_json::json!({ "code": code, "language": "rust" }))
            }),
        ),
        // Analyze code
        (
            FunctionDef::new("analyze_code", "Analyze code for issues and suggestions")
                .with_param("code", ParameterDef::string("Code to analyze"))
                .with_param("language", ParameterDef::string("Programming language").with_enum(vec!["rust", "c", "cpp", "python"]))
                .with_required("code")
                .with_category("code_analysis"),
            Box::new(|call: &FunctionCall| {
                let code = call.get_string("code").unwrap_or_default();
                let lang = call.get_string("language").unwrap_or("rust".to_string());
                
                // Simple analysis (in practice, this would use proper tools)
                let issues = analyze_code_simple(&code, &lang);
                FunctionResult::success(call, serde_json::json!({ 
                    "issues": issues,
                    "line_count": code.lines().count(),
                    "language": lang
                }))
            }),
        ),
    ]
}

fn generate_fsm_code(name: &str, states: &[&str], initial: &str) -> String {
    let state_enum = states.iter()
        .map(|s| format!("    {},", s))
        .collect::<Vec<_>>()
        .join("\n");
    
    format!(r#"#[derive(Debug, Clone, Copy, PartialEq)]
pub enum {}State {{
{}
}}

pub struct {} {{
    state: {}State,
}}

impl {} {{
    pub fn new() -> Self {{
        Self {{
            state: {}State::{},
        }}
    }}

    pub fn current_state(&self) -> {}State {{
        self.state
    }}

    pub fn transition(&mut self, next: {}State) {{
        self.state = next;
    }}
}}
"#, name, state_enum, name, name, name, name, initial, name, name)
}

fn analyze_code_simple(code: &str, _lang: &str) -> Vec<String> {
    let mut issues = Vec::new();
    
    if code.contains("unwrap()") {
        issues.push("Consider handling errors instead of using unwrap()".to_string());
    }
    if code.contains("unsafe") {
        issues.push("Contains unsafe code blocks - ensure memory safety".to_string());
    }
    if code.lines().any(|l| l.len() > 100) {
        issues.push("Some lines exceed 100 characters".to_string());
    }
    if !code.contains("///") && !code.contains("//!") {
        issues.push("Consider adding documentation comments".to_string());
    }
    
    if issues.is_empty() {
        issues.push("No obvious issues found".to_string());
    }
    
    issues
}

// =============================================================================
// HELPERS
// =============================================================================

fn generate_call_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("call_{:x}", nanos)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_def() {
        let func = FunctionDef::new("test_func", "A test function")
            .with_param("name", ParameterDef::string("The name"))
            .with_param("count", ParameterDef::integer("The count").with_range(0.0, 100.0))
            .with_required("name")
            .with_category("testing");

        assert_eq!(func.name, "test_func");
        assert_eq!(func.parameters.properties.len(), 2);
        assert_eq!(func.parameters.required.len(), 1);
        assert_eq!(func.category, Some("testing".to_string()));
    }

    #[test]
    fn test_function_call() {
        let call = FunctionCall {
            id: "test_id".to_string(),
            name: "test_func".to_string(),
            arguments: serde_json::json!({
                "name": "hello",
                "count": 42,
                "flag": true
            }),
        };

        assert_eq!(call.get_string("name"), Some("hello".to_string()));
        assert_eq!(call.get_integer("count"), Some(42));
        assert_eq!(call.get_bool("flag"), Some(true));
        assert_eq!(call.get_string("missing"), None);
    }

    #[test]
    fn test_function_result() {
        let call = FunctionCall {
            id: "test".to_string(),
            name: "func".to_string(),
            arguments: serde_json::json!({}),
        };

        let success = FunctionResult::success(&call, serde_json::json!({"result": 42}));
        assert_eq!(success.status, FunctionStatus::Success);
        assert!(success.value.is_some());

        let error = FunctionResult::error(&call, "Something went wrong");
        assert_eq!(error.status, FunctionStatus::Error);
        assert!(error.error.is_some());
    }

    #[tokio::test]
    async fn test_registry() {
        let registry = FunctionRegistry::new();
        
        let def = FunctionDef::new("echo", "Echo the input")
            .with_param("message", ParameterDef::string("Message to echo"))
            .with_required("message");

        registry.register_simple(def, |call| {
            let msg = call.get_string("message").unwrap_or_default();
            FunctionResult::success(call, serde_json::json!({ "echoed": msg }))
        }).await;

        assert!(registry.get("echo").await.is_some());
        assert!(registry.get("nonexistent").await.is_none());

        let call = FunctionCall {
            id: "test".to_string(),
            name: "echo".to_string(),
            arguments: serde_json::json!({ "message": "hello" }),
        };

        let result = registry.execute(&call).await;
        assert_eq!(result.status, FunctionStatus::Success);
    }

    #[test]
    fn test_gemini_format() {
        let funcs = vec![
            FunctionDef::new("get_weather", "Get current weather")
                .with_param("location", ParameterDef::string("City name"))
                .with_required("location"),
        ];

        let tools = to_gemini_tools(&funcs);
        assert!(tools.get("function_declarations").is_some());
    }

    #[test]
    fn test_openai_format() {
        let funcs = vec![
            FunctionDef::new("get_weather", "Get current weather")
                .with_param("location", ParameterDef::string("City name"))
                .with_required("location"),
        ];

        let tools = to_openai_tools(&funcs);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["type"], "function");
    }
}
