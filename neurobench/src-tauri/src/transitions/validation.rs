//! Connection Validation
//!
//! Validates transitions/connections between nodes

use serde::{Deserialize, Serialize};
use super::transition_types::Transition;

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub transition_id: Option<String>,
    pub severity: Severity,
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub code: String,
    pub message: String,
    pub transition_id: Option<String>,
}

/// Severity level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Port compatibility check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortCompatibility {
    pub compatible: bool,
    pub source_type: Option<String>,
    pub target_type: Option<String>,
    pub message: Option<String>,
}

/// Validation result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub transition_count: usize,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            transition_count: 0,
        }
    }

    pub fn add_error(&mut self, code: impl Into<String>, message: impl Into<String>, transition_id: Option<String>) {
        self.valid = false;
        self.errors.push(ValidationError {
            code: code.into(),
            message: message.into(),
            transition_id,
            severity: Severity::Error,
        });
    }

    pub fn add_warning(&mut self, code: impl Into<String>, message: impl Into<String>, transition_id: Option<String>) {
        self.warnings.push(ValidationWarning {
            code: code.into(),
            message: message.into(),
            transition_id,
        });
    }

    pub fn merge(&mut self, other: ValidationResult) {
        if !other.valid {
            self.valid = false;
        }
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.transition_count += other.transition_count;
    }
}

/// Connection validator
#[derive(Debug, Clone, Default)]
pub struct ConnectionValidator {
    /// Port type compatibility rules
    compatible_ports: Vec<(String, String)>,
}

impl ConnectionValidator {
    pub fn new() -> Self {
        let mut validator = Self {
            compatible_ports: Vec::new(),
        };
        
        // Default compatibility rules
        validator.add_compatibility("flow", "flow");
        validator.add_compatibility("digital", "digital");
        validator.add_compatibility("analog", "analog");
        validator.add_compatibility("data", "data");
        validator.add_compatibility("event", "event");
        validator.add_compatibility("output", "input");
        validator.add_compatibility("digital", "analog"); // Digital can drive analog
        
        validator
    }

    /// Add a port type compatibility rule
    pub fn add_compatibility(&mut self, source: impl Into<String>, target: impl Into<String>) {
        self.compatible_ports.push((source.into(), target.into()));
    }

    /// Check if two port types are compatible
    pub fn check_port_compatibility(&self, source_type: &str, target_type: &str) -> PortCompatibility {
        // Same types are always compatible
        if source_type == target_type {
            return PortCompatibility {
                compatible: true,
                source_type: Some(source_type.to_string()),
                target_type: Some(target_type.to_string()),
                message: None,
            };
        }

        // Check compatibility rules
        for (src, tgt) in &self.compatible_ports {
            if src == source_type && tgt == target_type {
                return PortCompatibility {
                    compatible: true,
                    source_type: Some(source_type.to_string()),
                    target_type: Some(target_type.to_string()),
                    message: None,
                };
            }
        }

        PortCompatibility {
            compatible: false,
            source_type: Some(source_type.to_string()),
            target_type: Some(target_type.to_string()),
            message: Some(format!("Port type '{}' is not compatible with '{}'", source_type, target_type)),
        }
    }

    /// Validate a single transition
    pub fn validate_transition(&self, t: &Transition) -> ValidationResult {
        let mut result = ValidationResult::new();
        result.transition_count = 1;

        // Check for self-loop
        if t.source.node_id == t.target.node_id {
            result.add_warning(
                "SELF_LOOP",
                format!("Transition {} is a self-loop", t.id),
                Some(t.id.clone()),
            );
        }

        // Check guard syntax (basic)
        if let Some(ref guard) = t.guard {
            if guard.is_empty() {
                result.add_warning(
                    "EMPTY_GUARD",
                    format!("Transition {} has an empty guard", t.id),
                    Some(t.id.clone()),
                );
            }
            if guard.contains(";;") || guard.contains("{{") {
                result.add_error(
                    "INVALID_GUARD_SYNTAX",
                    format!("Transition {} has invalid guard syntax", t.id),
                    Some(t.id.clone()),
                );
            }
        }

        // Check action syntax (basic)
        if let Some(ref action) = t.action {
            if action.is_empty() {
                result.add_warning(
                    "EMPTY_ACTION",
                    format!("Transition {} has an empty action", t.id),
                    Some(t.id.clone()),
                );
            }
        }

        // Check for missing event on non-immediate transition
        if t.event.is_none() && t.guard.is_none() && !t.is_default {
            result.add_warning(
                "NO_TRIGGER",
                format!("Transition {} has no event, guard, or default flag", t.id),
                Some(t.id.clone()),
            );
        }

        result
    }

    /// Validate all transitions
    pub fn validate_all(&self, transitions: &[Transition]) -> ValidationResult {
        let mut result = ValidationResult::new();

        for t in transitions {
            let t_result = self.validate_transition(t);
            result.merge(t_result);
        }

        // Check for duplicate transitions (same source + target)
        let mut seen: std::collections::HashSet<(&str, &str)> = std::collections::HashSet::new();
        for t in transitions {
            let key = (t.source.node_id.as_str(), t.target.node_id.as_str());
            if !seen.insert(key) {
                result.add_warning(
                    "DUPLICATE_TRANSITION",
                    format!("Multiple transitions from {} to {}", key.0, key.1),
                    Some(t.id.clone()),
                );
            }
        }

        // Check for unreachable states would require node context

        result
    }

    /// Quick check if a connection is valid
    pub fn can_connect(&self, source_node: &str, target_node: &str) -> bool {
        // Basic check - just ensure different nodes
        source_node != target_node
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transitions::transition_types::TransitionEvent;

    #[test]
    fn test_port_compatibility() {
        let validator = ConnectionValidator::new();
        
        assert!(validator.check_port_compatibility("flow", "flow").compatible);
        assert!(validator.check_port_compatibility("digital", "analog").compatible);
        assert!(!validator.check_port_compatibility("event", "flow").compatible);
    }

    #[test]
    fn test_transition_validation() {
        let validator = ConnectionValidator::new();
        
        let t = Transition::new("t1", "a", "b")
            .with_guard("x > 10");
        
        let result = validator.validate_transition(&t);
        assert!(result.valid);
    }

    #[test]
    fn test_self_loop_warning() {
        let validator = ConnectionValidator::new();
        
        let t = Transition::new("t1", "a", "a"); // Self-loop
        
        let result = validator.validate_transition(&t);
        assert!(!result.warnings.is_empty());
    }
}
