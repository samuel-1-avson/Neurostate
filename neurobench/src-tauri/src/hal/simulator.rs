// HAL Simulator
// Simulates GPIO, ADC, PWM, UART for virtual testing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// Simulated Hardware Abstraction Layer
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct HalSimulator {
    gpio: RwLock<HashMap<u8, bool>>,
    pwm: RwLock<HashMap<u8, u8>>,
    adc: RwLock<HashMap<u8, u16>>,
    uart_tx: RwLock<Vec<String>>,
    uart_rx: RwLock<Vec<String>>,
}

impl HalSimulator {
    pub fn new() -> Self {
        Self::default()
    }
    
    // GPIO Operations
    pub fn gpio_write(&self, pin: u8, value: bool) {
        self.gpio.write().unwrap().insert(pin, value);
    }
    
    pub fn gpio_read(&self, pin: u8) -> bool {
        *self.gpio.read().unwrap().get(&pin).unwrap_or(&false)
    }
    
    // PWM Operations  
    pub fn pwm_set(&self, channel: u8, duty: u8) {
        let duty = duty.min(100);
        self.pwm.write().unwrap().insert(channel, duty);
    }
    
    pub fn pwm_get(&self, channel: u8) -> u8 {
        *self.pwm.read().unwrap().get(&channel).unwrap_or(&0)
    }
    
    // ADC Operations (simulated values)
    pub fn adc_read(&self, channel: u8) -> u16 {
        // Generate simulated sensor data
        let base = 2048u16;
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as f64)
            .unwrap_or(0.0);
        
        let noise = ((time / 1000.0 + channel as f64).sin() * 500.0) as i32;
        (base as i32 + noise).clamp(0, 4095) as u16
    }
    
    // UART Operations
    pub fn uart_transmit(&self, data: &str) {
        self.uart_tx.write().unwrap().push(data.to_string());
    }
    
    pub fn uart_receive(&self) -> Option<String> {
        self.uart_rx.write().unwrap().pop()
    }
    
    pub fn uart_inject_rx(&self, data: &str) {
        self.uart_rx.write().unwrap().push(data.to_string());
    }
    
    // Get snapshot for UI
    pub fn get_snapshot(&self) -> HalSnapshot {
        HalSnapshot {
            gpio: self.gpio.read().unwrap().clone(),
            pwm: self.pwm.read().unwrap().clone(),
            uart_tx: self.uart_tx.read().unwrap().clone(),
        }
    }
    
    // Reset all state
    pub fn reset(&self) {
        self.gpio.write().unwrap().clear();
        self.pwm.write().unwrap().clear();
        self.uart_tx.write().unwrap().clear();
        self.uart_rx.write().unwrap().clear();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalSnapshot {
    pub gpio: HashMap<u8, bool>,
    pub pwm: HashMap<u8, u8>,
    pub uart_tx: Vec<String>,
}

// Global simulator instance
lazy_static::lazy_static! {
    pub static ref HAL: HalSimulator = HalSimulator::new();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gpio() {
        let hal = HalSimulator::new();
        hal.gpio_write(13, true);
        assert!(hal.gpio_read(13));
        assert!(!hal.gpio_read(14));
    }
    
    #[test]
    fn test_adc() {
        let hal = HalSimulator::new();
        let value = hal.adc_read(0);
        assert!(value <= 4095);
    }
}
