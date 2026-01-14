//! Power Consumption Estimation
//!
//! Estimates power consumption based on MCU state, clock frequencies, and peripherals.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Power profile for a peripheral
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeripheralPowerProfile {
    /// Name of the peripheral
    pub name: String,
    /// Active current in microamps
    pub active_current_ua: u32,
    /// Idle current in microamps
    pub idle_current_ua: u32,
    /// Currently enabled
    pub enabled: bool,
    /// Currently active (processing)
    pub active: bool,
}

/// MCU power profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McuPowerProfile {
    /// MCU name
    pub name: String,
    /// Supply voltage (mV)
    pub vdd_mv: u32,
    /// Core current per MHz (uA/MHz)
    pub core_current_per_mhz: f32,
    /// Static leakage current (uA)
    pub leakage_current_ua: u32,
    /// Flash read current (uA)
    pub flash_read_current_ua: u32,
    /// RAM access current (uA)
    pub ram_access_current_ua: u32,
    /// Sleep mode current (uA)
    pub sleep_current_ua: u32,
    /// Deep sleep current (uA)
    pub deep_sleep_current_ua: u32,
    /// Standby current (uA)
    pub standby_current_ua: u32,
}

impl Default for McuPowerProfile {
    fn default() -> Self {
        // STM32F4 typical values
        Self {
            name: "STM32F4".to_string(),
            vdd_mv: 3300,
            core_current_per_mhz: 250.0, // ~250uA per MHz
            leakage_current_ua: 50,
            flash_read_current_ua: 500,
            ram_access_current_ua: 200,
            sleep_current_ua: 2000,
            deep_sleep_current_ua: 100,
            standby_current_ua: 3,
        }
    }
}

/// Power estimation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerEstimate {
    /// Total current in microamps
    pub total_current_ua: u32,
    /// Total power in microwatts
    pub total_power_uw: u32,
    /// Breakdown by component
    pub breakdown: HashMap<String, u32>,
    /// Estimated battery life (hours) for given capacity
    pub battery_life_hours: Option<f32>,
}

/// Power estimator
pub struct PowerEstimator {
    /// MCU power profile
    mcu_profile: McuPowerProfile,
    /// Peripheral power profiles
    peripherals: Vec<PeripheralPowerProfile>,
    /// Current clock frequency (Hz)
    clock_hz: u32,
    /// Current power mode
    power_mode: PowerMode,
    /// Battery capacity (mAh)
    battery_capacity_mah: Option<f32>,
    /// Accumulated energy (nanojoules)
    energy_nj: u64,
    /// Total time tracked (microseconds)
    time_us: u64,
}

/// Power mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerMode {
    Run,
    Sleep,
    DeepSleep,
    Standby,
}

impl PowerEstimator {
    pub fn new(mcu_profile: McuPowerProfile) -> Self {
        Self {
            mcu_profile,
            peripherals: Vec::new(),
            clock_hz: 16_000_000, // Default 16MHz
            power_mode: PowerMode::Run,
            battery_capacity_mah: None,
            energy_nj: 0,
            time_us: 0,
        }
    }

    /// Set clock frequency
    pub fn set_clock(&mut self, hz: u32) {
        self.clock_hz = hz;
    }

    /// Set power mode
    pub fn set_power_mode(&mut self, mode: PowerMode) {
        self.power_mode = mode;
    }

    /// Set battery capacity
    pub fn set_battery(&mut self, capacity_mah: f32) {
        self.battery_capacity_mah = Some(capacity_mah);
    }

    /// Add peripheral
    pub fn add_peripheral(&mut self, profile: PeripheralPowerProfile) {
        self.peripherals.push(profile);
    }

    /// Update peripheral state
    pub fn set_peripheral_state(&mut self, name: &str, enabled: bool, active: bool) {
        if let Some(p) = self.peripherals.iter_mut().find(|p| p.name == name) {
            p.enabled = enabled;
            p.active = active;
        }
    }

    /// Calculate current power consumption
    pub fn estimate(&self) -> PowerEstimate {
        let mut breakdown = HashMap::new();
        let mut total_current_ua = 0u32;

        match self.power_mode {
            PowerMode::Run => {
                // Core consumption
                let mhz = self.clock_hz as f32 / 1_000_000.0;
                let core_current = (mhz * self.mcu_profile.core_current_per_mhz) as u32;
                breakdown.insert("Core".to_string(), core_current);
                total_current_ua += core_current;

                // Leakage
                breakdown.insert("Leakage".to_string(), self.mcu_profile.leakage_current_ua);
                total_current_ua += self.mcu_profile.leakage_current_ua;

                // Flash/RAM (assume always accessed in run mode)
                breakdown.insert("Flash".to_string(), self.mcu_profile.flash_read_current_ua);
                total_current_ua += self.mcu_profile.flash_read_current_ua;

                // Peripherals
                for p in &self.peripherals {
                    if p.enabled {
                        let current = if p.active { p.active_current_ua } else { p.idle_current_ua };
                        breakdown.insert(p.name.clone(), current);
                        total_current_ua += current;
                    }
                }
            }
            PowerMode::Sleep => {
                breakdown.insert("Sleep".to_string(), self.mcu_profile.sleep_current_ua);
                total_current_ua = self.mcu_profile.sleep_current_ua;

                // Some peripherals may still be active
                for p in &self.peripherals {
                    if p.enabled && p.active {
                        breakdown.insert(p.name.clone(), p.active_current_ua);
                        total_current_ua += p.active_current_ua;
                    }
                }
            }
            PowerMode::DeepSleep => {
                breakdown.insert("DeepSleep".to_string(), self.mcu_profile.deep_sleep_current_ua);
                total_current_ua = self.mcu_profile.deep_sleep_current_ua;
            }
            PowerMode::Standby => {
                breakdown.insert("Standby".to_string(), self.mcu_profile.standby_current_ua);
                total_current_ua = self.mcu_profile.standby_current_ua;
            }
        }

        // Calculate power
        let total_power_uw = total_current_ua * self.mcu_profile.vdd_mv / 1000;

        // Estimate battery life
        let battery_life_hours = self.battery_capacity_mah.map(|cap| {
            let current_ma = total_current_ua as f32 / 1000.0;
            if current_ma > 0.0 {
                cap / current_ma
            } else {
                f32::INFINITY
            }
        });

        PowerEstimate {
            total_current_ua,
            total_power_uw,
            breakdown,
            battery_life_hours,
        }
    }

    /// Update energy accumulator
    pub fn track(&mut self, dt_us: u64) {
        let estimate = self.estimate();
        
        // E = P * t (power in uW, time in us -> energy in pJ)
        // Convert to nJ: pJ / 1000
        let energy_pj = (estimate.total_power_uw as u64) * dt_us;
        self.energy_nj += energy_pj / 1000;
        self.time_us += dt_us;
    }

    /// Get total energy consumed
    pub fn get_total_energy_nj(&self) -> u64 {
        self.energy_nj
    }

    /// Get total energy in millijoules
    pub fn get_total_energy_mj(&self) -> f64 {
        self.energy_nj as f64 / 1_000_000.0
    }

    /// Get average power in milliwatts
    pub fn get_average_power_mw(&self) -> f64 {
        if self.time_us == 0 {
            return 0.0;
        }
        // P = E / t
        // energy_nj / time_us = nW -> divide by 1000 for uW -> 1000000 for mW
        (self.energy_nj as f64 / self.time_us as f64) * 1000.0
    }

    /// Reset tracking
    pub fn reset(&mut self) {
        self.energy_nj = 0;
        self.time_us = 0;
    }
}

/// Pre-configured power profiles
pub mod profiles {
    use super::*;

    pub fn stm32f4() -> McuPowerProfile {
        McuPowerProfile {
            name: "STM32F4".to_string(),
            vdd_mv: 3300,
            core_current_per_mhz: 250.0,
            leakage_current_ua: 50,
            flash_read_current_ua: 500,
            ram_access_current_ua: 200,
            sleep_current_ua: 2000,
            deep_sleep_current_ua: 100,
            standby_current_ua: 3,
        }
    }

    pub fn stm32l4() -> McuPowerProfile {
        McuPowerProfile {
            name: "STM32L4".to_string(),
            vdd_mv: 3300,
            core_current_per_mhz: 100.0, // Ultra low power
            leakage_current_ua: 10,
            flash_read_current_ua: 200,
            ram_access_current_ua: 80,
            sleep_current_ua: 500,
            deep_sleep_current_ua: 30,
            standby_current_ua: 0, // <1uA
        }
    }

    pub fn esp32() -> McuPowerProfile {
        McuPowerProfile {
            name: "ESP32".to_string(),
            vdd_mv: 3300,
            core_current_per_mhz: 300.0,
            leakage_current_ua: 100,
            flash_read_current_ua: 1000,
            ram_access_current_ua: 500,
            sleep_current_ua: 10000, // Light sleep
            deep_sleep_current_ua: 10,
            standby_current_ua: 5,
        }
    }

    pub fn nrf52() -> McuPowerProfile {
        McuPowerProfile {
            name: "nRF52".to_string(),
            vdd_mv: 3000,
            core_current_per_mhz: 50.0, // Very low power
            leakage_current_ua: 5,
            flash_read_current_ua: 150,
            ram_access_current_ua: 50,
            sleep_current_ua: 1000,
            deep_sleep_current_ua: 3,
            standby_current_ua: 0,
        }
    }

    pub fn gpio_peripheral() -> PeripheralPowerProfile {
        PeripheralPowerProfile {
            name: "GPIO".to_string(),
            active_current_ua: 100,
            idle_current_ua: 10,
            enabled: true,
            active: false,
        }
    }

    pub fn uart_peripheral() -> PeripheralPowerProfile {
        PeripheralPowerProfile {
            name: "UART".to_string(),
            active_current_ua: 500,
            idle_current_ua: 50,
            enabled: false,
            active: false,
        }
    }

    pub fn spi_peripheral() -> PeripheralPowerProfile {
        PeripheralPowerProfile {
            name: "SPI".to_string(),
            active_current_ua: 800,
            idle_current_ua: 20,
            enabled: false,
            active: false,
        }
    }

    pub fn adc_peripheral() -> PeripheralPowerProfile {
        PeripheralPowerProfile {
            name: "ADC".to_string(),
            active_current_ua: 1000,
            idle_current_ua: 100,
            enabled: false,
            active: false,
        }
    }

    pub fn wifi_peripheral() -> PeripheralPowerProfile {
        PeripheralPowerProfile {
            name: "WiFi".to_string(),
            active_current_ua: 120000, // 120mA TX
            idle_current_ua: 20000,    // 20mA listen
            enabled: false,
            active: false,
        }
    }

    pub fn ble_peripheral() -> PeripheralPowerProfile {
        PeripheralPowerProfile {
            name: "BLE".to_string(),
            active_current_ua: 8000, // 8mA TX
            idle_current_ua: 1000,   // 1mA idle
            enabled: false,
            active: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_estimate() {
        let mut estimator = PowerEstimator::new(profiles::stm32f4());
        estimator.set_clock(168_000_000); // 168MHz
        
        let estimate = estimator.estimate();
        
        // Should be roughly 168 * 250 + leakage + flash = ~42mA
        assert!(estimate.total_current_ua > 40000);
        assert!(estimate.total_current_ua < 50000);
    }

    #[test]
    fn test_battery_life() {
        let mut estimator = PowerEstimator::new(profiles::stm32l4());
        estimator.set_clock(1_000_000); // 1MHz low power
        estimator.set_power_mode(PowerMode::Run);
        estimator.set_battery(1000.0); // 1000mAh
        
        let estimate = estimator.estimate();
        
        // Should have significant battery life at low power
        assert!(estimate.battery_life_hours.unwrap() > 1000.0);
    }

    #[test]
    fn test_energy_tracking() {
        let mut estimator = PowerEstimator::new(profiles::stm32f4());
        estimator.set_clock(16_000_000);
        
        estimator.track(1_000_000); // 1 second
        
        assert!(estimator.get_total_energy_mj() > 0.0);
    }
}
