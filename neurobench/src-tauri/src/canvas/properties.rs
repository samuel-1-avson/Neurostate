//! Properties - Dynamic property system for nodes
//!
//! Provides typed, validated properties with inheritance from templates.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Property type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDef {
    /// Property name
    pub name: String,
    /// Display label
    pub label: String,
    /// Property type
    pub prop_type: PropertyType,
    /// Default value
    pub default: PropertyValue,
    /// Is required
    pub required: bool,
    /// Validation constraints
    pub constraints: Option<PropertyConstraints>,
    /// Category for grouping in UI
    pub category: Option<String>,
    /// Help text
    pub description: Option<String>,
}

/// Property types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    String,
    Integer,
    Float,
    Boolean,
    Enum(Vec<String>),
    Color,
    Pin,
    Expression,
    Code,
}

/// Property value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    None,
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<PropertyValue>),
}

impl Default for PropertyValue {
    fn default() -> Self {
        PropertyValue::None
    }
}

impl PropertyValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            PropertyValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            PropertyValue::Integer(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            PropertyValue::Float(n) => Some(*n),
            PropertyValue::Integer(n) => Some(*n as f64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PropertyValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

/// Property constraints for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyConstraints {
    /// Minimum value (for numbers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// Maximum value (for numbers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Regex pattern (for strings)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    /// Minimum length (for strings/arrays)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    /// Maximum length (for strings/arrays)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
}

/// Property set for a node
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PropertySet {
    /// Property values
    values: HashMap<String, PropertyValue>,
    /// Property definitions (schema)
    #[serde(skip)]
    schema: HashMap<String, PropertyDef>,
}

impl PropertySet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with schema
    pub fn with_schema(schema: Vec<PropertyDef>) -> Self {
        let mut set = Self::new();
        for def in schema {
            set.values.insert(def.name.clone(), def.default.clone());
            set.schema.insert(def.name.clone(), def);
        }
        set
    }

    /// Set property value
    pub fn set(&mut self, name: impl Into<String>, value: PropertyValue) -> Result<(), PropertyError> {
        let name = name.into();
        
        // Validate if schema exists
        if let Some(def) = self.schema.get(&name) {
            self.validate_value(&value, def)?;
        }
        
        self.values.insert(name, value);
        Ok(())
    }

    /// Get property value
    pub fn get(&self, name: &str) -> Option<&PropertyValue> {
        self.values.get(name)
    }

    /// Get string property
    pub fn get_string(&self, name: &str) -> Option<&str> {
        self.get(name).and_then(|v| v.as_string())
    }

    /// Get integer property
    pub fn get_int(&self, name: &str) -> Option<i64> {
        self.get(name).and_then(|v| v.as_int())
    }

    /// Get float property
    pub fn get_float(&self, name: &str) -> Option<f64> {
        self.get(name).and_then(|v| v.as_float())
    }

    /// Get boolean property
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.get(name).and_then(|v| v.as_bool())
    }

    /// Get all values
    pub fn all(&self) -> &HashMap<String, PropertyValue> {
        &self.values
    }

    /// Merge from another set (other takes precedence)
    pub fn merge(&mut self, other: &PropertySet) {
        for (k, v) in &other.values {
            self.values.insert(k.clone(), v.clone());
        }
    }

    /// Validate a value against definition
    fn validate_value(&self, value: &PropertyValue, def: &PropertyDef) -> Result<(), PropertyError> {
        // Type check
        let type_match = match (&def.prop_type, value) {
            (PropertyType::String, PropertyValue::String(_)) => true,
            (PropertyType::Integer, PropertyValue::Integer(_)) => true,
            (PropertyType::Float, PropertyValue::Float(_)) | 
            (PropertyType::Float, PropertyValue::Integer(_)) => true,
            (PropertyType::Boolean, PropertyValue::Boolean(_)) => true,
            (PropertyType::Enum(opts), PropertyValue::String(s)) => opts.contains(s),
            (PropertyType::Color, PropertyValue::String(_)) => true,
            (PropertyType::Pin, PropertyValue::String(_)) => true,
            (PropertyType::Expression, PropertyValue::String(_)) => true,
            (PropertyType::Code, PropertyValue::String(_)) => true,
            (_, PropertyValue::None) => !def.required,
            _ => false,
        };

        if !type_match {
            return Err(PropertyError::TypeMismatch {
                name: def.name.clone(),
                expected: format!("{:?}", def.prop_type),
            });
        }

        // Constraint check
        if let Some(constraints) = &def.constraints {
            if let Some(n) = value.as_float() {
                if let Some(min) = constraints.min {
                    if n < min {
                        return Err(PropertyError::OutOfRange { name: def.name.clone(), value: n, min: Some(min), max: constraints.max });
                    }
                }
                if let Some(max) = constraints.max {
                    if n > max {
                        return Err(PropertyError::OutOfRange { name: def.name.clone(), value: n, min: constraints.min, max: Some(max) });
                    }
                }
            }
        }

        Ok(())
    }
}

/// Property error
#[derive(Debug, Clone)]
pub enum PropertyError {
    TypeMismatch { name: String, expected: String },
    OutOfRange { name: String, value: f64, min: Option<f64>, max: Option<f64> },
    Required { name: String },
    PatternMismatch { name: String, pattern: String },
}

impl std::fmt::Display for PropertyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropertyError::TypeMismatch { name, expected } => 
                write!(f, "Property '{}' type mismatch, expected {}", name, expected),
            PropertyError::OutOfRange { name, value, min, max } => 
                write!(f, "Property '{}' value {} out of range [{:?}, {:?}]", name, value, min, max),
            PropertyError::Required { name } => 
                write!(f, "Property '{}' is required", name),
            PropertyError::PatternMismatch { name, pattern } => 
                write!(f, "Property '{}' doesn't match pattern '{}'", name, pattern),
        }
    }
}

impl std::error::Error for PropertyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_set() {
        let mut props = PropertySet::new();
        props.set("name", PropertyValue::String("Test".into())).unwrap();
        props.set("count", PropertyValue::Integer(42)).unwrap();
        
        assert_eq!(props.get_string("name"), Some("Test"));
        assert_eq!(props.get_int("count"), Some(42));
    }

    #[test]
    fn test_validation() {
        let schema = vec![
            PropertyDef {
                name: "priority".into(),
                label: "Priority".into(),
                prop_type: PropertyType::Integer,
                default: PropertyValue::Integer(5),
                required: true,
                constraints: Some(PropertyConstraints {
                    min: Some(1.0),
                    max: Some(10.0),
                    pattern: None,
                    min_length: None,
                    max_length: None,
                }),
                category: None,
                description: None,
            }
        ];

        let mut props = PropertySet::with_schema(schema);
        
        // Valid
        assert!(props.set("priority", PropertyValue::Integer(5)).is_ok());
        
        // Out of range
        assert!(props.set("priority", PropertyValue::Integer(100)).is_err());
    }
}
