// MCU Registry
// Comprehensive database of supported microcontrollers

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Architecture {
    ArmCortexM0,
    ArmCortexM0Plus,
    ArmCortexM3,
    ArmCortexM4,
    ArmCortexM7,
    ArmCortexM33,
    RiscV32,
    RiscV64,
    Xtensa,
    Avr,
    Pic,
    Msp430,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum McuFamily {
    Stm32,
    Nrf,
    Esp32,
    Rp2040,
    Samd,
    Avr,
    Pic,
    Msp430,
    Gd32,
    Ch32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DebugInterface {
    Swd,
    Jtag,
    Updi,
    Isp,
    UsbJtag,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McuSpec {
    pub flash_kb: u32,
    pub ram_kb: u32,
    pub freq_mhz: u32,
    pub voltage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McuDefinition {
    pub id: String,
    pub name: String,
    pub family: McuFamily,
    pub arch: Architecture,
    pub debug_interface: DebugInterface,
    pub description: String,
    pub specs: McuSpec,
}

/// Get all registered MCUs
pub fn get_all_mcus() -> Vec<McuDefinition> {
    vec![
        // STM32 Family
        McuDefinition {
            id: "stm32f401".to_string(),
            name: "STM32F401 BlackPill".to_string(),
            family: McuFamily::Stm32,
            arch: Architecture::ArmCortexM4,
            debug_interface: DebugInterface::Swd,
            description: "Popular Cortex-M4 development board".to_string(),
            specs: McuSpec { flash_kb: 256, ram_kb: 64, freq_mhz: 84, voltage: 3.3 },
        },
        McuDefinition {
            id: "stm32f103".to_string(),
            name: "STM32F103 BluePill".to_string(),
            family: McuFamily::Stm32,
            arch: Architecture::ArmCortexM3,
            debug_interface: DebugInterface::Swd,
            description: "Classic Cortex-M3 board".to_string(),
            specs: McuSpec { flash_kb: 64, ram_kb: 20, freq_mhz: 72, voltage: 3.3 },
        },
        McuDefinition {
            id: "stm32h743".to_string(),
            name: "STM32H743".to_string(),
            family: McuFamily::Stm32,
            arch: Architecture::ArmCortexM7,
            debug_interface: DebugInterface::Swd,
            description: "High-performance Cortex-M7".to_string(),
            specs: McuSpec { flash_kb: 2048, ram_kb: 1024, freq_mhz: 480, voltage: 3.3 },
        },
        
        // ESP32 Family
        McuDefinition {
            id: "esp32".to_string(),
            name: "ESP32-WROOM-32".to_string(),
            family: McuFamily::Esp32,
            arch: Architecture::Xtensa,
            debug_interface: DebugInterface::UsbJtag,
            description: "Dual-core WiFi+BT module".to_string(),
            specs: McuSpec { flash_kb: 4096, ram_kb: 520, freq_mhz: 240, voltage: 3.3 },
        },
        McuDefinition {
            id: "esp32c3".to_string(),
            name: "ESP32-C3".to_string(),
            family: McuFamily::Esp32,
            arch: Architecture::RiscV32,
            debug_interface: DebugInterface::UsbJtag,
            description: "RISC-V WiFi+BLE module".to_string(),
            specs: McuSpec { flash_kb: 4096, ram_kb: 400, freq_mhz: 160, voltage: 3.3 },
        },
        McuDefinition {
            id: "esp32s3".to_string(),
            name: "ESP32-S3".to_string(),
            family: McuFamily::Esp32,
            arch: Architecture::Xtensa,
            debug_interface: DebugInterface::UsbJtag,
            description: "AI-capable dual-core module".to_string(),
            specs: McuSpec { flash_kb: 8192, ram_kb: 512, freq_mhz: 240, voltage: 3.3 },
        },
        
        // RP2040
        McuDefinition {
            id: "rp2040".to_string(),
            name: "Raspberry Pi Pico".to_string(),
            family: McuFamily::Rp2040,
            arch: Architecture::ArmCortexM0Plus,
            debug_interface: DebugInterface::Swd,
            description: "Dual-core M0+ from Raspberry Pi".to_string(),
            specs: McuSpec { flash_kb: 2048, ram_kb: 264, freq_mhz: 133, voltage: 3.3 },
        },
        
        // Nordic nRF
        McuDefinition {
            id: "nrf52840".to_string(),
            name: "nRF52840".to_string(),
            family: McuFamily::Nrf,
            arch: Architecture::ArmCortexM4,
            debug_interface: DebugInterface::Swd,
            description: "BLE 5.0 + Thread/Zigbee SoC".to_string(),
            specs: McuSpec { flash_kb: 1024, ram_kb: 256, freq_mhz: 64, voltage: 3.0 },
        },
        
        // AVR
        McuDefinition {
            id: "atmega328p".to_string(),
            name: "ATmega328P (Arduino Uno)".to_string(),
            family: McuFamily::Avr,
            arch: Architecture::Avr,
            debug_interface: DebugInterface::Isp,
            description: "Classic Arduino microcontroller".to_string(),
            specs: McuSpec { flash_kb: 32, ram_kb: 2, freq_mhz: 16, voltage: 5.0 },
        },
        McuDefinition {
            id: "atmega2560".to_string(),
            name: "ATmega2560 (Arduino Mega)".to_string(),
            family: McuFamily::Avr,
            arch: Architecture::Avr,
            debug_interface: DebugInterface::Isp,
            description: "Large AVR with many I/O".to_string(),
            specs: McuSpec { flash_kb: 256, ram_kb: 8, freq_mhz: 16, voltage: 5.0 },
        },
        
        // RISC-V
        McuDefinition {
            id: "ch32v003".to_string(),
            name: "CH32V003".to_string(),
            family: McuFamily::Ch32,
            arch: Architecture::RiscV32,
            debug_interface: DebugInterface::Swd,
            description: "Ultra low-cost RISC-V MCU".to_string(),
            specs: McuSpec { flash_kb: 16, ram_kb: 2, freq_mhz: 48, voltage: 3.3 },
        },
        McuDefinition {
            id: "gd32vf103".to_string(),
            name: "GD32VF103".to_string(),
            family: McuFamily::Gd32,
            arch: Architecture::RiscV32,
            debug_interface: DebugInterface::Jtag,
            description: "RISC-V general purpose MCU".to_string(),
            specs: McuSpec { flash_kb: 128, ram_kb: 32, freq_mhz: 108, voltage: 3.3 },
        },
    ]
}

/// Get MCU by ID
pub fn get_mcu(id: &str) -> Option<McuDefinition> {
    get_all_mcus().into_iter().find(|m| m.id == id)
}

/// Get MCUs by family
pub fn get_mcus_by_family(family: McuFamily) -> Vec<McuDefinition> {
    get_all_mcus().into_iter().filter(|m| m.family == family).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_registry() {
        let all = get_all_mcus();
        assert!(!all.is_empty());
        
        let stm32 = get_mcu("stm32f401");
        assert!(stm32.is_some());
        assert_eq!(stm32.unwrap().arch, Architecture::ArmCortexM4);
    }
}
