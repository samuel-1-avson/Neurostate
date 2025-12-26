//! Templates - Node template library with presets
//!
//! Provides reusable node templates for quick design creation.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::canvas::types::{NodeType, CanvasNode};
use crate::canvas::ports::{Port, PortDirection, PortDataType, PortPosition, PortTemplate};

/// A node template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTemplate {
    /// Template ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Node type this template creates
    pub node_type: NodeType,
    /// Category for organization
    pub category: String,
    /// Description
    pub description: String,
    /// Default width
    pub default_width: f64,
    /// Default height
    pub default_height: f64,
    /// Default label format (e.g., "State_{n}")
    pub label_format: String,
    /// Preset ports
    pub ports: Vec<Port>,
    /// Default properties
    pub properties: HashMap<String, PropertyValue>,
    /// Icon name
    pub icon: Option<String>,
    /// Color theme
    pub color: Option<String>,
}

/// Property value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<PropertyValue>),
}

impl NodeTemplate {
    /// Create a basic state template
    pub fn state() -> Self {
        Self {
            id: "state".into(),
            name: "State".into(),
            node_type: NodeType::State,
            category: "FSM".into(),
            description: "A basic FSM state".into(),
            default_width: 120.0,
            default_height: 60.0,
            label_format: "State_{n}".into(),
            ports: PortTemplate::fsm_state().ports,
            properties: HashMap::new(),
            icon: Some("box".into()),
            color: Some("#3b82f6".into()),
        }
    }

    /// Create a GPIO input template
    pub fn gpio_input() -> Self {
        Self {
            id: "gpio_input".into(),
            name: "GPIO Input".into(),
            node_type: NodeType::DigitalInput,
            category: "GPIO".into(),
            description: "Digital input pin".into(),
            default_width: 100.0,
            default_height: 50.0,
            label_format: "DIN_{n}".into(),
            ports: vec![
                Port::output("out", "Signal", PortDataType::Digital).at(PortPosition::Right),
            ],
            properties: [
                ("pin".to_string(), PropertyValue::String("PA0".into())),
                ("pull".to_string(), PropertyValue::String("none".into())),
            ].into_iter().collect(),
            icon: Some("arrow-right".into()),
            color: Some("#22c55e".into()),
        }
    }

    /// Create a GPIO output template
    pub fn gpio_output() -> Self {
        Self {
            id: "gpio_output".into(),
            name: "GPIO Output".into(),
            node_type: NodeType::DigitalOutput,
            category: "GPIO".into(),
            description: "Digital output pin".into(),
            default_width: 100.0,
            default_height: 50.0,
            label_format: "DOUT_{n}".into(),
            ports: vec![
                Port::input("in", "Signal", PortDataType::Digital).at(PortPosition::Left),
            ],
            properties: [
                ("pin".to_string(), PropertyValue::String("PB0".into())),
                ("initial".to_string(), PropertyValue::Boolean(false)),
            ].into_iter().collect(),
            icon: Some("arrow-left".into()),
            color: Some("#f59e0b".into()),
        }
    }

    /// Create a UART template
    pub fn uart() -> Self {
        Self {
            id: "uart".into(),
            name: "UART".into(),
            node_type: NodeType::Uart,
            category: "Communication".into(),
            description: "UART serial interface".into(),
            default_width: 140.0,
            default_height: 80.0,
            label_format: "UART{n}".into(),
            ports: vec![
                Port::output("tx", "TX", PortDataType::Serial).at(PortPosition::Right),
                Port::input("rx", "RX", PortDataType::Serial).at(PortPosition::Left),
            ],
            properties: [
                ("baud".to_string(), PropertyValue::Integer(115200)),
                ("data_bits".to_string(), PropertyValue::Integer(8)),
                ("stop_bits".to_string(), PropertyValue::Integer(1)),
                ("parity".to_string(), PropertyValue::String("none".into())),
            ].into_iter().collect(),
            icon: Some("code".into()),
            color: Some("#8b5cf6".into()),
        }
    }

    /// Create a Timer template
    pub fn timer() -> Self {
        Self {
            id: "timer".into(),
            name: "Timer".into(),
            node_type: NodeType::Timer,
            category: "Timer".into(),
            description: "Hardware timer".into(),
            default_width: 120.0,
            default_height: 60.0,
            label_format: "TIM{n}".into(),
            ports: vec![
                Port::output("overflow", "OVF", PortDataType::Event).at(PortPosition::Right),
                Port::input("enable", "EN", PortDataType::Digital).at(PortPosition::Left),
            ],
            properties: [
                ("prescaler".to_string(), PropertyValue::Integer(1)),
                ("period".to_string(), PropertyValue::Integer(1000)),
                ("mode".to_string(), PropertyValue::String("up".into())),
            ].into_iter().collect(),
            icon: Some("clock".into()),
            color: Some("#ec4899".into()),
        }
    }

    /// Create a Task template (RTOS)
    pub fn rtos_task() -> Self {
        Self {
            id: "task".into(),
            name: "Task".into(),
            node_type: NodeType::Task,
            category: "RTOS".into(),
            description: "RTOS task".into(),
            default_width: 140.0,
            default_height: 70.0,
            label_format: "Task_{n}".into(),
            ports: vec![
                Port::input("signal_in", "Signal", PortDataType::Event).at(PortPosition::Left),
                Port::output("signal_out", "Notify", PortDataType::Event).at(PortPosition::Right),
            ],
            properties: [
                ("priority".to_string(), PropertyValue::Integer(5)),
                ("stack_size".to_string(), PropertyValue::Integer(1024)),
            ].into_iter().collect(),
            icon: Some("cpu".into()),
            color: Some("#06b6d4".into()),
        }
    }

    /// Instantiate a node from this template
    pub fn instantiate(&self, id: String, x: f64, y: f64, instance_number: usize) -> CanvasNode {
        let label = self.label_format.replace("{n}", &instance_number.to_string());
        
        CanvasNode {
            id,
            label,
            node_type: self.node_type.clone(),
            x,
            y,
            width: self.default_width,
            height: self.default_height,
            entry_action: None,
            exit_action: None,
            description: Some(self.description.clone()),
        }
    }
}

/// Template library
#[derive(Debug, Clone, Default)]
pub struct TemplateLibrary {
    templates: HashMap<String, NodeTemplate>,
    categories: HashMap<String, Vec<String>>,
}

impl TemplateLibrary {
    pub fn new() -> Self {
        let mut lib = Self::default();
        lib.load_defaults();
        lib
    }

    fn load_defaults(&mut self) {
        let defaults = vec![
            NodeTemplate::state(),
            NodeTemplate::gpio_input(),
            NodeTemplate::gpio_output(),
            NodeTemplate::uart(),
            NodeTemplate::timer(),
            NodeTemplate::rtos_task(),
        ];

        for template in defaults {
            self.add(template);
        }
    }

    /// Add a template
    pub fn add(&mut self, template: NodeTemplate) {
        let id = template.id.clone();
        let category = template.category.clone();
        
        self.templates.insert(id.clone(), template);
        self.categories.entry(category).or_default().push(id);
    }

    /// Get template by ID
    pub fn get(&self, id: &str) -> Option<&NodeTemplate> {
        self.templates.get(id)
    }

    /// Get all templates
    pub fn all(&self) -> Vec<&NodeTemplate> {
        self.templates.values().collect()
    }

    /// Get templates by category
    pub fn by_category(&self, category: &str) -> Vec<&NodeTemplate> {
        self.categories.get(category)
            .map(|ids| ids.iter().filter_map(|id| self.templates.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all categories
    pub fn categories(&self) -> Vec<&str> {
        self.categories.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_instantiate() {
        let template = NodeTemplate::state();
        let node = template.instantiate("n1".into(), 100.0, 200.0, 5);
        
        assert_eq!(node.label, "State_5");
        assert_eq!(node.x, 100.0);
        assert_eq!(node.y, 200.0);
    }

    #[test]
    fn test_library() {
        let lib = TemplateLibrary::new();
        assert!(lib.get("state").is_some());
        assert!(lib.get("uart").is_some());
    }
}
