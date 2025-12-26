//! Native Function Calling Support
//!
//! Provides structured function/tool calling for LLMs that support it.
//! Compatible with OpenAI function calling and Gemini tool use.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JSON Schema type for function parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JsonSchemaType {
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
    Null,
}

/// Property definition for function parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDef {
    #[serde(rename = "type")]
    pub prop_type: JsonSchemaType,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<PropertyDef>>,
}

impl PropertyDef {
    pub fn string(description: &str) -> Self {
        Self {
            prop_type: JsonSchemaType::String,
            description: description.to_string(),
            enum_values: None,
            items: None,
        }
    }
    
    pub fn number(description: &str) -> Self {
        Self {
            prop_type: JsonSchemaType::Number,
            description: description.to_string(),
            enum_values: None,
            items: None,
        }
    }
    
    pub fn boolean(description: &str) -> Self {
        Self {
            prop_type: JsonSchemaType::Boolean,
            description: description.to_string(),
            enum_values: None,
            items: None,
        }
    }
    
    pub fn string_enum(description: &str, values: Vec<&str>) -> Self {
        Self {
            prop_type: JsonSchemaType::String,
            description: description.to_string(),
            enum_values: Some(values.into_iter().map(|s| s.to_string()).collect()),
            items: None,
        }
    }
    
    pub fn array(description: &str, item_type: PropertyDef) -> Self {
        Self {
            prop_type: JsonSchemaType::Array,
            description: description.to_string(),
            enum_values: None,
            items: Some(Box::new(item_type)),
        }
    }
}

/// Function parameter schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    pub properties: HashMap<String, PropertyDef>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
}

impl ParameterSchema {
    pub fn new() -> Self {
        Self {
            schema_type: "object".to_string(),
            properties: HashMap::new(),
            required: Vec::new(),
        }
    }
    
    pub fn with_property(mut self, name: &str, prop: PropertyDef, required: bool) -> Self {
        self.properties.insert(name.to_string(), prop);
        if required {
            self.required.push(name.to_string());
        }
        self
    }
}

impl Default for ParameterSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Function definition for LLM tool calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDef {
    pub name: String,
    pub description: String,
    pub parameters: ParameterSchema,
}

impl FunctionDef {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            parameters: ParameterSchema::new(),
        }
    }
    
    pub fn with_param(mut self, name: &str, prop: PropertyDef, required: bool) -> Self {
        self.parameters = self.parameters.with_property(name, prop, required);
        self
    }
}

/// Function call from LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

impl FunctionCall {
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.arguments.get(key)?.as_str().map(|s| s.to_string())
    }
    
    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.arguments.get(key)?.as_f64()
    }
    
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.arguments.get(key)?.as_bool()
    }
    
    pub fn get_array(&self, key: &str) -> Option<&Vec<serde_json::Value>> {
        self.arguments.get(key)?.as_array()
    }
}

/// Result of executing a function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionResult {
    pub name: String,
    pub result: serde_json::Value,
    pub success: bool,
    pub error: Option<String>,
}

impl FunctionResult {
    pub fn success(name: &str, result: serde_json::Value) -> Self {
        Self {
            name: name.to_string(),
            result,
            success: true,
            error: None,
        }
    }
    
    pub fn error(name: &str, error: &str) -> Self {
        Self {
            name: name.to_string(),
            result: serde_json::Value::Null,
            success: false,
            error: Some(error.to_string()),
        }
    }
}

/// OpenAI-format tool wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAITool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDef,
}

impl From<FunctionDef> for OpenAITool {
    fn from(func: FunctionDef) -> Self {
        Self {
            tool_type: "function".to_string(),
            function: func,
        }
    }
}

/// Gemini-format tool wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiTool {
    pub function_declarations: Vec<FunctionDef>,
}

impl GeminiTool {
    pub fn new(functions: Vec<FunctionDef>) -> Self {
        Self {
            function_declarations: functions,
        }
    }
}

/// Standard agent functions available to all agents
pub fn get_standard_functions() -> Vec<FunctionDef> {
    vec![
        // Add node to canvas
        FunctionDef::new("add_node", "Add a new node to the FSM canvas")
            .with_param("name", PropertyDef::string("Name of the node"), true)
            .with_param("node_type", PropertyDef::string_enum(
                "Type of node",
                vec!["state", "input", "output", "decision", "process", "error"]
            ), true)
            .with_param("x", PropertyDef::number("X position on canvas"), false)
            .with_param("y", PropertyDef::number("Y position on canvas"), false),
        
        // Connect nodes
        FunctionDef::new("connect_nodes", "Create a connection between two nodes")
            .with_param("from_node", PropertyDef::string("Source node name"), true)
            .with_param("to_node", PropertyDef::string("Target node name"), true)
            .with_param("condition", PropertyDef::string("Transition condition"), false),
        
        // Auto-layout
        FunctionDef::new("auto_layout", "Arrange nodes using automatic layout")
            .with_param("algorithm", PropertyDef::string_enum(
                "Layout algorithm to use",
                vec!["hierarchical", "force_directed", "grid", "tree"]
            ), true),
        
        // Generate code
        FunctionDef::new("generate_code", "Generate code for the FSM")
            .with_param("language", PropertyDef::string_enum(
                "Programming language",
                vec!["c", "cpp", "rust", "python"]
            ), true)
            .with_param("style", PropertyDef::string_enum(
                "Code style",
                vec!["switch_case", "state_pattern", "table_driven"]
            ), false),
        
        // Build project
        FunctionDef::new("build_project", "Compile the project")
            .with_param("target", PropertyDef::string("MCU target"), false)
            .with_param("release", PropertyDef::boolean("Build in release mode"), false),
        
        // Flash device
        FunctionDef::new("flash_device", "Flash firmware to device")
            .with_param("verify", PropertyDef::boolean("Verify after flash"), false),
        
        // Validate FSM
        FunctionDef::new("validate_fsm", "Validate the FSM for errors")
            .with_param("check_cycles", PropertyDef::boolean("Check for cycles"), false)
            .with_param("check_unreachable", PropertyDef::boolean("Check for unreachable states"), false),
        
        // Delegate to agent
        FunctionDef::new("delegate", "Delegate task to another agent")
            .with_param("agent_id", PropertyDef::string_enum(
                "Agent to delegate to",
                vec!["code", "debug", "hardware", "canvas", "build", "deploy", "docs", "fsm"]
            ), true)
            .with_param("task", PropertyDef::string("Task description"), true),
    ]
}

/// Get functions specific to an agent
pub fn get_agent_functions(agent_id: &str) -> Vec<FunctionDef> {
    let mut functions = get_standard_functions();
    
    match agent_id {
        "canvas" => {
            functions.push(
                FunctionDef::new("align_nodes", "Align selected nodes")
                    .with_param("alignment", PropertyDef::string_enum(
                        "Alignment direction",
                        vec!["left", "right", "top", "bottom", "center_horizontal", "center_vertical"]
                    ), true)
            );
            functions.push(
                FunctionDef::new("snap_to_grid", "Snap nodes to grid")
                    .with_param("grid_size", PropertyDef::number("Grid size in pixels"), false)
            );
        }
        "code" => {
            functions.push(
                FunctionDef::new("optimize_code", "Optimize generated code")
                    .with_param("focus", PropertyDef::string_enum(
                        "Optimization focus",
                        vec!["size", "speed", "readability"]
                    ), true)
            );
        }
        "debug" => {
            functions.push(
                FunctionDef::new("analyze_error", "Analyze an error message")
                    .with_param("error", PropertyDef::string("Error message to analyze"), true)
            );
        }
        "hardware" => {
            functions.push(
                FunctionDef::new("configure_gpio", "Configure GPIO pin")
                    .with_param("pin", PropertyDef::string("Pin name (e.g., PA0)"), true)
                    .with_param("mode", PropertyDef::string_enum(
                        "Pin mode",
                        vec!["input", "output", "analog", "alternate"]
                    ), true)
            );
        }
        _ => {}
    }
    
    functions
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_def() {
        let func = FunctionDef::new("test", "A test function")
            .with_param("name", PropertyDef::string("The name"), true)
            .with_param("count", PropertyDef::number("The count"), false);
        
        assert_eq!(func.name, "test");
        assert_eq!(func.parameters.properties.len(), 2);
        assert_eq!(func.parameters.required.len(), 1);
    }
    
    #[test]
    fn test_openai_format() {
        let func = FunctionDef::new("add_node", "Add a node");
        let tool: OpenAITool = func.into();
        
        assert_eq!(tool.tool_type, "function");
        assert_eq!(tool.function.name, "add_node");
    }
    
    #[test]
    fn test_function_call_parsing() {
        let call = FunctionCall {
            name: "test".to_string(),
            arguments: serde_json::json!({
                "name": "hello",
                "count": 42,
                "enabled": true
            }),
        };
        
        assert_eq!(call.get_string("name"), Some("hello".to_string()));
        assert_eq!(call.get_number("count"), Some(42.0));
        assert_eq!(call.get_bool("enabled"), Some(true));
    }
}
