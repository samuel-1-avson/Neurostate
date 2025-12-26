//! Embedded Resources - MCU resource management and conflict detection
//!
//! Provides hardware resource tracking for embedded design validation.

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

/// MCU pin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McuPin {
    /// Pin number/name (e.g., "PA0", "PB3")
    pub id: String,
    /// Alternate function selected (e.g., "UART1_TX", "TIM2_CH1")
    pub function: Option<String>,
    /// Available alternate functions
    pub alt_functions: Vec<String>,
    /// Pin mode
    pub mode: PinMode,
    /// Pull-up/down configuration
    pub pull: PullConfig,
    /// Node ID that owns this pin
    pub owner_node: Option<String>,
}

/// Pin mode
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PinMode {
    #[default]
    Input,
    Output,
    Analog,
    AlternateFunction,
}

/// Pull resistor configuration
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PullConfig {
    #[default]
    None,
    PullUp,
    PullDown,
}

/// Hardware resource types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    /// GPIO pin
    Pin(String),
    /// Timer peripheral
    Timer(String),
    /// DMA channel
    Dma { controller: u8, stream: u8, channel: u8 },
    /// UART peripheral
    Uart(u8),
    /// SPI peripheral
    Spi(u8),
    /// I2C peripheral
    I2c(u8),
    /// ADC channel
    Adc { unit: u8, channel: u8 },
    /// DAC channel
    Dac(u8),
    /// Interrupt line
    Interrupt(u8),
    /// CAN bus
    Can(u8),
    /// USB endpoint
    Usb { endpoint: u8 },
    /// Custom resource
    Custom(String),
}

/// Resource allocation entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    /// Resource being allocated
    pub resource: ResourceType,
    /// Node ID that owns this resource
    pub owner_node: String,
    /// Configuration details
    pub config: HashMap<String, String>,
}

/// Resource conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConflict {
    /// The contested resource
    pub resource: ResourceType,
    /// Node IDs competing for the resource
    pub competing_nodes: Vec<String>,
    /// Severity level
    pub severity: ConflictSeverity,
    /// Human-readable description
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictSeverity {
    Error,
    Warning,
    Info,
}

/// Resource manager for tracking hardware allocations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceManager {
    /// All allocated resources
    allocations: HashMap<String, ResourceAllocation>,
    /// Node to resources mapping
    node_resources: HashMap<String, HashSet<String>>,
    /// Pin configurations
    pins: HashMap<String, McuPin>,
    /// Target MCU
    target_mcu: Option<String>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set target MCU (loads pin definitions)
    pub fn set_target_mcu(&mut self, mcu: impl Into<String>) {
        self.target_mcu = Some(mcu.into());
        // In real impl, would load MCU-specific pin definitions
    }

    /// Allocate a resource to a node
    pub fn allocate(
        &mut self,
        node_id: impl Into<String>,
        resource: ResourceType,
        config: HashMap<String, String>,
    ) -> Result<(), ResourceConflict> {
        let node_id = node_id.into();
        let resource_key = Self::resource_key(&resource);

        // Check for existing allocation
        if let Some(existing) = self.allocations.get(&resource_key) {
            if existing.owner_node != node_id {
                return Err(ResourceConflict {
                    resource: resource.clone(),
                    competing_nodes: vec![existing.owner_node.clone(), node_id],
                    severity: ConflictSeverity::Error,
                    message: format!(
                        "Resource {:?} already allocated to node {}",
                        resource, existing.owner_node
                    ),
                });
            }
        }

        // Create allocation
        let allocation = ResourceAllocation {
            resource,
            owner_node: node_id.clone(),
            config,
        };

        self.allocations.insert(resource_key.clone(), allocation);
        self.node_resources
            .entry(node_id)
            .or_default()
            .insert(resource_key);

        Ok(())
    }

    /// Deallocate a resource
    pub fn deallocate(&mut self, resource: &ResourceType) -> bool {
        let key = Self::resource_key(resource);
        
        if let Some(allocation) = self.allocations.remove(&key) {
            if let Some(resources) = self.node_resources.get_mut(&allocation.owner_node) {
                resources.remove(&key);
            }
            true
        } else {
            false
        }
    }

    /// Deallocate all resources for a node
    pub fn deallocate_node(&mut self, node_id: &str) {
        if let Some(resources) = self.node_resources.remove(node_id) {
            for key in resources {
                self.allocations.remove(&key);
            }
        }
    }

    /// Get all resources for a node
    pub fn get_node_resources(&self, node_id: &str) -> Vec<&ResourceAllocation> {
        self.node_resources
            .get(node_id)
            .map(|keys| {
                keys.iter()
                    .filter_map(|k| self.allocations.get(k))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check for conflicts in current allocations
    pub fn find_conflicts(&self) -> Vec<ResourceConflict> {
        // In this simplified model, conflicts are prevented at allocation time
        // This method would detect more complex conflicts like:
        // - DMA channels on same stream
        // - Interrupt priority conflicts
        // - Timing conflicts
        Vec::new()
    }

    /// Configure a pin
    pub fn configure_pin(
        &mut self,
        pin_id: impl Into<String>,
        mode: PinMode,
        function: Option<String>,
        owner: Option<String>,
    ) {
        let pin_id = pin_id.into();
        
        let pin = self.pins.entry(pin_id.clone()).or_insert_with(|| McuPin {
            id: pin_id,
            function: None,
            alt_functions: Vec::new(),
            mode: PinMode::Input,
            pull: PullConfig::None,
            owner_node: None,
        });

        pin.mode = mode;
        pin.function = function;
        pin.owner_node = owner;
    }

    /// Get pin configuration
    pub fn get_pin(&self, pin_id: &str) -> Option<&McuPin> {
        self.pins.get(pin_id)
    }

    /// Get all pin configurations
    pub fn get_all_pins(&self) -> Vec<&McuPin> {
        self.pins.values().collect()
    }

    /// Get pins for a specific node
    pub fn get_node_pins(&self, node_id: &str) -> Vec<&McuPin> {
        self.pins.values()
            .filter(|p| p.owner_node.as_deref() == Some(node_id))
            .collect()
    }

    /// Generate unique key for resource
    fn resource_key(resource: &ResourceType) -> String {
        match resource {
            ResourceType::Pin(id) => format!("pin:{}", id),
            ResourceType::Timer(id) => format!("tim:{}", id),
            ResourceType::Dma { controller, stream, channel } => {
                format!("dma:{}:{}:{}", controller, stream, channel)
            }
            ResourceType::Uart(n) => format!("uart:{}", n),
            ResourceType::Spi(n) => format!("spi:{}", n),
            ResourceType::I2c(n) => format!("i2c:{}", n),
            ResourceType::Adc { unit, channel } => format!("adc:{}:{}", unit, channel),
            ResourceType::Dac(n) => format!("dac:{}", n),
            ResourceType::Interrupt(n) => format!("irq:{}", n),
            ResourceType::Can(n) => format!("can:{}", n),
            ResourceType::Usb { endpoint } => format!("usb:{}", endpoint),
            ResourceType::Custom(id) => format!("custom:{}", id),
        }
    }

    /// Get resource utilization summary
    pub fn get_utilization(&self) -> ResourceUtilization {
        let mut util = ResourceUtilization::default();
        
        for alloc in self.allocations.values() {
            match &alloc.resource {
                ResourceType::Pin(_) => util.pins_used += 1,
                ResourceType::Timer(_) => util.timers_used += 1,
                ResourceType::Dma { .. } => util.dma_channels_used += 1,
                ResourceType::Uart(_) => util.uarts_used += 1,
                ResourceType::Spi(_) => util.spi_used += 1,
                ResourceType::I2c(_) => util.i2c_used += 1,
                ResourceType::Interrupt(_) => util.interrupts_used += 1,
                _ => {}
            }
        }
        
        util
    }
}

/// Resource utilization summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub pins_used: usize,
    pub timers_used: usize,
    pub dma_channels_used: usize,
    pub uarts_used: usize,
    pub spi_used: usize,
    pub i2c_used: usize,
    pub interrupts_used: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_allocation() {
        let mut manager = ResourceManager::new();
        
        let result = manager.allocate(
            "node1",
            ResourceType::Uart(1),
            HashMap::new(),
        );
        assert!(result.is_ok());

        // Conflict detection
        let result = manager.allocate(
            "node2",
            ResourceType::Uart(1),
            HashMap::new(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_pin_config() {
        let mut manager = ResourceManager::new();
        
        manager.configure_pin("PA0", PinMode::AlternateFunction, Some("UART1_TX".into()), Some("node1".into()));
        
        let pin = manager.get_pin("PA0").unwrap();
        assert_eq!(pin.function, Some("UART1_TX".to_string()));
        assert_eq!(pin.owner_node, Some("node1".to_string()));
    }
}
