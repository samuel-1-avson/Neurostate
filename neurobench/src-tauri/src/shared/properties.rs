//! Property Types - Node configuration properties
//!
//! Defines property values and default properties for node types.

use serde::{Deserialize, Serialize};
use super::node_types::NodeType;

// ============================================================================
// PROPERTY VALUES
// ============================================================================

/// Property value types for node configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PropertyValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Floating point value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Enum value with options
    Enum { value: String, options: Vec<String> },
    /// Pin configuration
    Pin { port: String, pin: u8 },
    /// Array of values
    Array(Vec<PropertyValue>),
}

impl Default for PropertyValue {
    fn default() -> Self {
        Self::String(String::new())
    }
}

impl PropertyValue {
    /// Get as string if possible
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            Self::Enum { value, .. } => Some(value),
            _ => None,
        }
    }
    
    /// Get as integer if possible
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(n) => Some(*n),
            Self::Float(f) => Some(*f as i64),
            Self::Boolean(b) => Some(if *b { 1 } else { 0 }),
            _ => None,
        }
    }
    
    /// Get as float if possible
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            Self::Integer(n) => Some(*n as f64),
            _ => None,
        }
    }
    
    /// Get as boolean if possible
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            Self::Integer(n) => Some(*n != 0),
            _ => None,
        }
    }
}

// ============================================================================
// PROPERTY DEFINITION
// ============================================================================

/// Property definition for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDef {
    /// Property key (identifier)
    pub key: String,
    /// Display label
    pub label: String,
    /// Current/default value
    pub value: PropertyValue,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether property is read-only
    #[serde(default)]
    pub readonly: bool,
    /// Whether property is required
    #[serde(default)]
    pub required: bool,
    /// Validation constraints (min, max, pattern, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<PropertyValidation>,
}

/// Property validation constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyValidation {
    /// Minimum value (for numeric)
    pub min: Option<f64>,
    /// Maximum value (for numeric)
    pub max: Option<f64>,
    /// Regex pattern (for string)
    pub pattern: Option<String>,
}

impl PropertyDef {
    /// Create a string property
    pub fn string(key: &str, label: &str, default: &str) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            value: PropertyValue::String(default.to_string()),
            description: None,
            readonly: false,
            required: false,
            validation: None,
        }
    }
    
    /// Create an integer property
    pub fn integer(key: &str, label: &str, default: i64) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            value: PropertyValue::Integer(default),
            description: None,
            readonly: false,
            required: false,
            validation: None,
        }
    }
    
    /// Create a float property
    pub fn float(key: &str, label: &str, default: f64) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            value: PropertyValue::Float(default),
            description: None,
            readonly: false,
            required: false,
            validation: None,
        }
    }
    
    /// Create a boolean property
    pub fn boolean(key: &str, label: &str, default: bool) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            value: PropertyValue::Boolean(default),
            description: None,
            readonly: false,
            required: false,
            validation: None,
        }
    }
    
    /// Create an enum property
    pub fn enum_prop(key: &str, label: &str, default: &str, options: &[&str]) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            value: PropertyValue::Enum {
                value: default.to_string(),
                options: options.iter().map(|s| s.to_string()).collect(),
            },
            description: None,
            readonly: false,
            required: false,
            validation: None,
        }
    }
    
    /// Create a pin property
    pub fn pin(key: &str, label: &str, default_port: &str, default_pin: u8) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            value: PropertyValue::Pin {
                port: default_port.to_string(),
                pin: default_pin,
            },
            description: None,
            readonly: false,
            required: false,
            validation: None,
        }
    }
    
    /// Builder: set description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }
    
    /// Builder: set as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
    
    /// Builder: set as readonly
    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }
    
    /// Builder: set min/max validation
    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.validation = Some(PropertyValidation {
            min: Some(min),
            max: Some(max),
            pattern: None,
        });
        self
    }
}

// ============================================================================
// DEFAULT PROPERTIES FOR NODE TYPES
// ============================================================================

/// Get default properties for a node type
pub fn default_properties(node_type: &NodeType) -> Vec<PropertyDef> {
    match node_type {
        // === GPIO ===
        NodeType::Gpio | NodeType::DigitalInput | NodeType::DigitalOutput => vec![
            PropertyDef::enum_prop("port", "Port", "A", &["A", "B", "C", "D", "E", "F", "G", "H"])
                .with_description("GPIO port"),
            PropertyDef::integer("pin", "Pin", 0)
                .with_description("Pin number (0-15)")
                .with_range(0.0, 15.0),
            PropertyDef::enum_prop("mode", "Mode", "output", &["input", "output", "alternate", "analog"])
                .with_description("Pin mode"),
            PropertyDef::enum_prop("pull", "Pull", "none", &["none", "up", "down"])
                .with_description("Internal pull resistor"),
            PropertyDef::enum_prop("speed", "Speed", "high", &["low", "medium", "high", "very_high"])
                .with_description("Output speed"),
        ],
        
        // === PWM ===
        NodeType::Pwm => vec![
            PropertyDef::enum_prop("timer", "Timer", "TIM1", &["TIM1", "TIM2", "TIM3", "TIM4", "TIM8"])
                .with_description("Timer peripheral"),
            PropertyDef::integer("channel", "Channel", 1)
                .with_description("Timer channel (1-4)")
                .with_range(1.0, 4.0),
            PropertyDef::integer("frequency", "Frequency (Hz)", 1000)
                .with_description("PWM frequency")
                .with_range(1.0, 1000000.0),
            PropertyDef::integer("duty_default", "Default Duty (%)", 50)
                .with_description("Default duty cycle")
                .with_range(0.0, 100.0),
        ],
        
        // === ADC ===
        NodeType::Adc => vec![
            PropertyDef::enum_prop("adc", "ADC", "ADC1", &["ADC1", "ADC2", "ADC3"])
                .with_description("ADC peripheral"),
            PropertyDef::integer("channel", "Channel", 0)
                .with_description("ADC channel")
                .with_range(0.0, 18.0),
            PropertyDef::enum_prop("resolution", "Resolution", "12bit", &["6bit", "8bit", "10bit", "12bit"])
                .with_description("ADC resolution"),
            PropertyDef::enum_prop("sample_time", "Sample Time", "15_cycles", 
                &["3_cycles", "15_cycles", "28_cycles", "56_cycles", "84_cycles", "112_cycles", "144_cycles", "480_cycles"])
                .with_description("Sampling time"),
        ],
        
        // === UART ===
        NodeType::Uart => vec![
            PropertyDef::enum_prop("instance", "Instance", "USART1", &["USART1", "USART2", "USART3", "UART4", "UART5", "USART6"])
                .with_description("UART peripheral"),
            PropertyDef::integer("baud_rate", "Baud Rate", 115200)
                .with_description("Baud rate")
                .with_range(300.0, 4500000.0),
            PropertyDef::enum_prop("data_bits", "Data Bits", "8", &["7", "8", "9"])
                .with_description("Data bits"),
            PropertyDef::enum_prop("parity", "Parity", "none", &["none", "odd", "even"])
                .with_description("Parity"),
            PropertyDef::enum_prop("stop_bits", "Stop Bits", "1", &["0.5", "1", "1.5", "2"])
                .with_description("Stop bits"),
            PropertyDef::boolean("hardware_flow", "Hardware Flow Control", false)
                .with_description("Enable RTS/CTS flow control"),
        ],
        
        // === SPI ===
        NodeType::Spi => vec![
            PropertyDef::enum_prop("instance", "Instance", "SPI1", &["SPI1", "SPI2", "SPI3", "SPI4"])
                .with_description("SPI peripheral"),
            PropertyDef::enum_prop("mode", "Mode", "master", &["master", "slave"])
                .with_description("SPI mode"),
            PropertyDef::enum_prop("clock_polarity", "Clock Polarity", "low", &["low", "high"])
                .with_description("CPOL"),
            PropertyDef::enum_prop("clock_phase", "Clock Phase", "1edge", &["1edge", "2edge"])
                .with_description("CPHA"),
            PropertyDef::integer("clock_div", "Clock Divider", 8)
                .with_description("Clock prescaler")
                .with_range(2.0, 256.0),
        ],
        
        // === I2C ===
        NodeType::I2c => vec![
            PropertyDef::enum_prop("instance", "Instance", "I2C1", &["I2C1", "I2C2", "I2C3"])
                .with_description("I2C peripheral"),
            PropertyDef::integer("clock_speed", "Clock Speed (Hz)", 100000)
                .with_description("I2C clock speed")
                .with_range(10000.0, 400000.0),
            PropertyDef::integer("own_address", "Own Address", 0)
                .with_description("Device address (7-bit)")
                .with_range(0.0, 127.0),
        ],
        
        // === Timer ===
        NodeType::Timer => vec![
            PropertyDef::enum_prop("timer", "Timer", "TIM1", &["TIM1", "TIM2", "TIM3", "TIM4", "TIM5", "TIM6", "TIM7", "TIM8"])
                .with_description("Timer peripheral"),
            PropertyDef::integer("period_ms", "Period (ms)", 1000)
                .with_description("Timer period in milliseconds")
                .with_range(1.0, 60000.0),
            PropertyDef::boolean("auto_reload", "Auto Reload", true)
                .with_description("Enable auto-reload"),
            PropertyDef::boolean("interrupt", "Enable Interrupt", true)
                .with_description("Enable timer interrupt"),
        ],
        
        // === RTOS Task ===
        NodeType::Task => vec![
            PropertyDef::string("name", "Task Name", "Task1")
                .with_description("RTOS task name"),
            PropertyDef::integer("priority", "Priority", 1)
                .with_description("Task priority (0 = lowest)")
                .with_range(0.0, 56.0),
            PropertyDef::integer("stack_size", "Stack Size", 256)
                .with_description("Stack size in words")
                .with_range(64.0, 65536.0),
            PropertyDef::integer("period_ms", "Period (ms)", 100)
                .with_description("Task execution period")
                .with_range(1.0, 60000.0),
        ],
        
        // === Semaphore ===
        NodeType::Semaphore => vec![
            PropertyDef::string("name", "Name", "sem1")
                .with_description("Semaphore name"),
            PropertyDef::enum_prop("type", "Type", "binary", &["binary", "counting"])
                .with_description("Semaphore type"),
            PropertyDef::integer("initial", "Initial Count", 0)
                .with_description("Initial count")
                .with_range(0.0, 255.0),
            PropertyDef::integer("max_count", "Max Count", 1)
                .with_description("Maximum count (for counting semaphore)")
                .with_range(1.0, 255.0),
        ],
        
        // === WiFi ===
        NodeType::Wifi => vec![
            PropertyDef::string("ssid", "SSID", "")
                .with_description("WiFi network name")
                .required(),
            PropertyDef::string("password", "Password", "")
                .with_description("WiFi password"),
            PropertyDef::enum_prop("mode", "Mode", "station", &["station", "ap", "ap_sta"])
                .with_description("WiFi mode"),
            PropertyDef::enum_prop("security", "Security", "wpa2", &["open", "wep", "wpa", "wpa2", "wpa3"])
                .with_description("Security mode"),
        ],
        
        // === MQTT ===
        NodeType::Mqtt => vec![
            PropertyDef::string("broker", "Broker URL", "mqtt://localhost:1883")
                .with_description("MQTT broker URL")
                .required(),
            PropertyDef::string("client_id", "Client ID", "neurostate_device")
                .with_description("MQTT client identifier"),
            PropertyDef::string("username", "Username", "")
                .with_description("MQTT username"),
            PropertyDef::string("password", "Password", "")
                .with_description("MQTT password"),
            PropertyDef::integer("keep_alive", "Keep Alive (s)", 60)
                .with_description("Keep-alive interval")
                .with_range(0.0, 3600.0),
            PropertyDef::enum_prop("qos", "QoS", "1", &["0", "1", "2"])
                .with_description("Quality of Service"),
        ],
        
        // === LED ===
        NodeType::Led => vec![
            PropertyDef::enum_prop("port", "Port", "A", &["A", "B", "C", "D", "E", "F"])
                .with_description("GPIO port"),
            PropertyDef::integer("pin", "Pin", 5)
                .with_description("Pin number")
                .with_range(0.0, 15.0),
            PropertyDef::boolean("active_low", "Active Low", false)
                .with_description("LED is on when pin is low"),
        ],
        
        // === Button ===
        NodeType::Button => vec![
            PropertyDef::enum_prop("port", "Port", "C", &["A", "B", "C", "D", "E", "F"])
                .with_description("GPIO port"),
            PropertyDef::integer("pin", "Pin", 13)
                .with_description("Pin number")
                .with_range(0.0, 15.0),
            PropertyDef::boolean("active_low", "Active Low", true)
                .with_description("Button is pressed when pin is low"),
            PropertyDef::enum_prop("pull", "Pull", "up", &["none", "up", "down"])
                .with_description("Internal pull resistor"),
            PropertyDef::integer("debounce_ms", "Debounce (ms)", 50)
                .with_description("Debounce time")
                .with_range(0.0, 500.0),
        ],
        
        // === State (FSM) ===
        NodeType::State => vec![
            PropertyDef::string("name", "State Name", "State")
                .with_description("Name of the state"),
            PropertyDef::string("entry_action", "Entry Action", "")
                .with_description("Code to run on state entry"),
            PropertyDef::string("exit_action", "Exit Action", "")
                .with_description("Code to run on state exit"),
            PropertyDef::string("during_action", "During Action", "")
                .with_description("Code to run while in state"),
        ],
        
        // === Delay ===
        NodeType::Delay => vec![
            PropertyDef::integer("delay_ms", "Delay (ms)", 1000)
                .with_description("Delay duration in milliseconds")
                .with_range(1.0, 3600000.0),
            PropertyDef::enum_prop("type", "Type", "blocking", &["blocking", "non_blocking"])
                .with_description("Delay type"),
        ],
        
        // === Default: empty properties ===
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_property_value_accessors() {
        let int_val = PropertyValue::Integer(42);
        assert_eq!(int_val.as_integer(), Some(42));
        assert_eq!(int_val.as_float(), Some(42.0));
        
        let bool_val = PropertyValue::Boolean(true);
        assert_eq!(bool_val.as_boolean(), Some(true));
        assert_eq!(bool_val.as_integer(), Some(1));
    }
    
    #[test]
    fn test_default_properties() {
        let gpio_props = default_properties(&NodeType::Gpio);
        assert!(!gpio_props.is_empty());
        assert!(gpio_props.iter().any(|p| p.key == "port"));
        assert!(gpio_props.iter().any(|p| p.key == "pin"));
        
        let task_props = default_properties(&NodeType::Task);
        assert!(task_props.iter().any(|p| p.key == "priority"));
        assert!(task_props.iter().any(|p| p.key == "stack_size"));
    }
}
