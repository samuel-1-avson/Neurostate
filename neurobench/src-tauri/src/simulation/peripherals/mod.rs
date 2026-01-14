//! Peripheral Bus and Common Interfaces
//!
//! Manages all simulated peripherals and provides the common trait interface.

pub mod gpio;
pub mod uart;
pub mod spi;
pub mod i2c;
pub mod timer;
pub mod adc;
pub mod dma;
pub mod can;
pub mod modbus;
pub mod watchdog;

use super::engine::SimulationConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use gpio::GpioPort;
pub use uart::UartPeripheral;
pub use spi::SpiPeripheral;
pub use i2c::I2cPeripheral;
pub use timer::TimerPeripheral;
pub use adc::AdcPeripheral;
pub use dma::DmaController;

/// Common trait for all peripherals
pub trait Peripheral: Send + Sync {
    /// Get peripheral name
    fn name(&self) -> &str;
    
    /// Read from peripheral register
    fn read(&self, offset: u32) -> u32;
    
    /// Write to peripheral register
    fn write(&mut self, offset: u32, value: u32);
    
    /// Advance peripheral by N cycles
    fn tick(&mut self, cycles: u64);
    
    /// Get pending interrupt number (if any)
    fn get_interrupt(&self) -> Option<u8>;
    
    /// Clear interrupt
    fn clear_interrupt(&mut self);
    
    /// Reset peripheral to initial state
    fn reset(&mut self);
    
    /// Get peripheral state as JSON
    fn get_state(&self) -> serde_json::Value;
}

/// Peripheral bus manages all peripherals
pub struct PeripheralBus {
    /// GPIO ports (A-H typically)
    gpio_ports: HashMap<char, GpioPort>,
    /// UART instances
    uarts: HashMap<u8, UartPeripheral>,
    /// SPI instances
    spis: HashMap<u8, SpiPeripheral>,
    /// I2C instances
    i2cs: HashMap<u8, I2cPeripheral>,
    /// Timer instances
    timers: HashMap<u8, TimerPeripheral>,
    /// ADC instance
    adc: AdcPeripheral,
    /// DMA controller
    dma: DmaController,
    /// Base addresses for peripherals
    base_addresses: HashMap<String, u32>,
}

impl PeripheralBus {
    /// Create a new peripheral bus based on MCU config
    pub fn new(config: &SimulationConfig) -> Self {
        let mut gpio_ports = HashMap::new();
        let mut uarts = HashMap::new();
        let mut spis = HashMap::new();
        let mut i2cs = HashMap::new();
        let mut timers = HashMap::new();
        let mut base_addresses = HashMap::new();
        
        // Create GPIO ports A-H
        for (i, port) in ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'].iter().enumerate() {
            let base = 0x4002_0000 + (i as u32 * 0x400);
            gpio_ports.insert(*port, GpioPort::new(*port, base));
            base_addresses.insert(format!("GPIO{}", port), base);
        }
        
        // Create UART instances
        for i in 1..=6 {
            let base = match i {
                1 => 0x4001_1000, // USART1
                2 => 0x4000_4400, // USART2
                3 => 0x4000_4800, // USART3
                4 => 0x4000_4C00, // UART4
                5 => 0x4000_5000, // UART5
                6 => 0x4001_1400, // USART6
                _ => 0x4000_4400,
            };
            uarts.insert(i, UartPeripheral::new(i, base, config.system_clock));
            base_addresses.insert(format!("USART{}", i), base);
        }
        
        // Create SPI instances
        for i in 1..=4 {
            let base = match i {
                1 => 0x4001_3000,
                2 => 0x4000_3800,
                3 => 0x4000_3C00,
                4 => 0x4001_3400,
                _ => 0x4000_3800,
            };
            spis.insert(i, SpiPeripheral::new(i, base));
            base_addresses.insert(format!("SPI{}", i), base);
        }
        
        // Create I2C instances
        for i in 1..=3 {
            let base = match i {
                1 => 0x4000_5400,
                2 => 0x4000_5800,
                3 => 0x4000_5C00,
                _ => 0x4000_5400,
            };
            i2cs.insert(i, I2cPeripheral::new(i, base));
            base_addresses.insert(format!("I2C{}", i), base);
        }
        
        // Create timer instances
        for i in 1..=14 {
            let base = match i {
                1 => 0x4001_0000,  // TIM1
                2 => 0x4000_0000,  // TIM2
                3 => 0x4000_0400,  // TIM3
                4 => 0x4000_0800,  // TIM4
                5 => 0x4000_0C00,  // TIM5
                6 => 0x4000_1000,  // TIM6
                7 => 0x4000_1400,  // TIM7
                8 => 0x4001_0400,  // TIM8
                9 => 0x4001_4000,  // TIM9
                10 => 0x4001_4400, // TIM10
                11 => 0x4001_4800, // TIM11
                12 => 0x4000_1800, // TIM12
                13 => 0x4000_1C00, // TIM13
                14 => 0x4000_2000, // TIM14
                _ => 0x4000_0000,
            };
            timers.insert(i, TimerPeripheral::new(i, base, config.system_clock));
            base_addresses.insert(format!("TIM{}", i), base);
        }
        
        // ADC
        base_addresses.insert("ADC1".to_string(), 0x4001_2000);
        
        // DMA
        base_addresses.insert("DMA1".to_string(), 0x4002_6000);
        base_addresses.insert("DMA2".to_string(), 0x4002_6400);
        
        Self {
            gpio_ports,
            uarts,
            spis,
            i2cs,
            timers,
            adc: AdcPeripheral::new(1, 0x4001_2000),
            dma: DmaController::new(1, 0x4002_6000),
            base_addresses,
        }
    }

    /// Tick all peripherals
    pub fn tick(&mut self, cycles: u64) {
        for port in self.gpio_ports.values_mut() {
            port.tick(cycles);
        }
        for uart in self.uarts.values_mut() {
            uart.tick(cycles);
        }
        for spi in self.spis.values_mut() {
            spi.tick(cycles);
        }
        for i2c in self.i2cs.values_mut() {
            i2c.tick(cycles);
        }
        for timer in self.timers.values_mut() {
            timer.tick(cycles);
        }
        self.adc.tick(cycles);
        self.dma.tick(cycles);
    }

    /// Reset all peripherals
    pub fn reset(&mut self) {
        for port in self.gpio_ports.values_mut() {
            port.reset();
        }
        for uart in self.uarts.values_mut() {
            uart.reset();
        }
        for spi in self.spis.values_mut() {
            spi.reset();
        }
        for i2c in self.i2cs.values_mut() {
            i2c.reset();
        }
        for timer in self.timers.values_mut() {
            timer.reset();
        }
        self.adc.reset();
        self.dma.reset();
    }

    /// Inject GPIO input
    pub fn gpio_inject(&mut self, port: char, pin: u8, state: bool) {
        if let Some(gpio) = self.gpio_ports.get_mut(&port) {
            gpio.inject_input(pin, state);
        }
    }

    /// Inject UART data
    pub fn uart_inject(&mut self, instance: u8, data: &[u8]) {
        if let Some(uart) = self.uarts.get_mut(&instance) {
            uart.inject_rx(data);
        }
    }

    /// Get GPIO state for a port
    pub fn get_gpio_state(&self, port: char) -> Option<GpioState> {
        self.gpio_ports.get(&port).map(|p| p.get_state_struct())
    }

    /// Get UART state
    pub fn get_uart_state(&self, instance: u8) -> Option<UartState> {
        self.uarts.get(&instance).map(|u| u.get_state_struct())
    }

    /// Create snapshot of all peripheral states
    pub fn snapshot(&self) -> HashMap<String, serde_json::Value> {
        let mut states = HashMap::new();
        
        for (port, gpio) in &self.gpio_ports {
            states.insert(format!("GPIO{}", port), gpio.get_state());
        }
        for (instance, uart) in &self.uarts {
            states.insert(format!("USART{}", instance), uart.get_state());
        }
        for (instance, timer) in &self.timers {
            states.insert(format!("TIM{}", instance), timer.get_state());
        }
        
        states
    }

    /// Restore from snapshot
    pub fn restore(&mut self, states: &HashMap<String, serde_json::Value>) {
        for (name, state) in states {
            if let Some(port_char) = name.strip_prefix("GPIO").and_then(|s| s.chars().next()) {
                if let Some(gpio) = self.gpio_ports.get_mut(&port_char) {
                    gpio.restore_state(state);
                }
            }
        }
    }

    /// Read from peripheral address
    pub fn read(&self, addr: u32) -> Option<u32> {
        // Find which peripheral owns this address
        for (port_char, gpio) in &self.gpio_ports {
            let base = *self.base_addresses.get(&format!("GPIO{}", port_char))?;
            if addr >= base && addr < base + 0x400 {
                return Some(gpio.read(addr - base));
            }
        }
        
        for (instance, uart) in &self.uarts {
            let base = *self.base_addresses.get(&format!("USART{}", instance))?;
            if addr >= base && addr < base + 0x400 {
                return Some(uart.read(addr - base));
            }
        }
        
        None
    }

    /// Write to peripheral address
    pub fn write(&mut self, addr: u32, value: u32) -> bool {
        // Find which peripheral owns this address
        for (port_char, gpio) in &mut self.gpio_ports {
            if let Some(&base) = self.base_addresses.get(&format!("GPIO{}", port_char)) {
                if addr >= base && addr < base + 0x400 {
                    gpio.write(addr - base, value);
                    return true;
                }
            }
        }
        
        for (instance, uart) in &mut self.uarts {
            if let Some(&base) = self.base_addresses.get(&format!("USART{}", instance)) {
                if addr >= base && addr < base + 0x400 {
                    uart.write(addr - base, value);
                    return true;
                }
            }
        }
        
        false
    }
}

/// GPIO state structure for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpioState {
    pub port: char,
    pub pins: [PinState; 16],
}

/// Individual pin state
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PinState {
    pub mode: PinMode,
    pub output_type: OutputType,
    pub pull: PullConfig,
    pub value: bool,
    pub alt_function: u8,
}

impl Default for PinState {
    fn default() -> Self {
        Self {
            mode: PinMode::Input,
            output_type: OutputType::PushPull,
            pull: PullConfig::None,
            value: false,
            alt_function: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PinMode {
    Input,
    Output,
    AlternateFunction,
    Analog,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OutputType {
    PushPull,
    OpenDrain,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PullConfig {
    None,
    PullUp,
    PullDown,
}

/// UART state structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UartState {
    pub instance: u8,
    pub baud_rate: u32,
    pub enabled: bool,
    pub tx_buffer: Vec<u8>,
    pub rx_buffer: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::engine::SimulationConfig;

    #[test]
    fn test_peripheral_bus_creation() {
        let config = SimulationConfig::default();
        let bus = PeripheralBus::new(&config);
        
        assert_eq!(bus.gpio_ports.len(), 8);
        assert_eq!(bus.uarts.len(), 6);
        assert_eq!(bus.timers.len(), 14);
    }

    #[test]
    fn test_gpio_inject() {
        let config = SimulationConfig::default();
        let mut bus = PeripheralBus::new(&config);
        
        bus.gpio_inject('A', 5, true);
        let state = bus.get_gpio_state('A').unwrap();
        assert!(state.pins[5].value);
    }
}
