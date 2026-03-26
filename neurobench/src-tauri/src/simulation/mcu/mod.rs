//! MCU Simulation Configurations
//!
//! Pre-configured simulation setups for different MCU families.

use super::engine::SimulationConfig;
use super::cpu::CpuArchitecture;
use super::ClockConfig;
use serde::{Deserialize, Serialize};

/// Trait for MCU-specific simulation setup
pub trait McuSimulation {
    /// Get MCU identifier
    fn id(&self) -> &str;
    /// Get display name
    fn name(&self) -> &str;
    /// Get CPU architecture
    fn architecture(&self) -> CpuArchitecture;
    /// Get simulation configuration
    fn config(&self) -> SimulationConfig;
    /// Get memory map description
    fn memory_map(&self) -> Vec<MemoryMapEntry>;
    /// Get available peripherals
    fn peripherals(&self) -> Vec<String>;
}

/// Memory map entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMapEntry {
    pub name: String,
    pub base: u32,
    pub size: usize,
    pub region_type: String,
}

/// STM32F4 MCU configuration
pub struct Stm32F4Sim {
    pub variant: Stm32F4Variant,
}

#[derive(Debug, Clone, Copy)]
pub enum Stm32F4Variant {
    Stm32F401,
    Stm32F407,
    Stm32F411,
    Stm32F429,
}

impl Default for Stm32F4Sim {
    fn default() -> Self {
        Self { variant: Stm32F4Variant::Stm32F401 }
    }
}

impl McuSimulation for Stm32F4Sim {
    fn id(&self) -> &str {
        match self.variant {
            Stm32F4Variant::Stm32F401 => "STM32F401",
            Stm32F4Variant::Stm32F407 => "STM32F407",
            Stm32F4Variant::Stm32F411 => "STM32F411",
            Stm32F4Variant::Stm32F429 => "STM32F429",
        }
    }

    fn name(&self) -> &str {
        match self.variant {
            Stm32F4Variant::Stm32F401 => "STM32F401 (84 MHz, 256KB Flash)",
            Stm32F4Variant::Stm32F407 => "STM32F407 (168 MHz, 1MB Flash)",
            Stm32F4Variant::Stm32F411 => "STM32F411 (100 MHz, 512KB Flash)",
            Stm32F4Variant::Stm32F429 => "STM32F429 (180 MHz, 2MB Flash)",
        }
    }

    fn architecture(&self) -> CpuArchitecture {
        CpuArchitecture::CortexM4
    }

    fn config(&self) -> SimulationConfig {
        match self.variant {
            Stm32F4Variant::Stm32F401 => SimulationConfig {
                mcu: "STM32F401".to_string(),
                architecture: CpuArchitecture::CortexM4,
                flash_size: 256 * 1024,
                ram_size: 64 * 1024,
                flash_base: 0x0800_0000,
                ram_base: 0x2000_0000,
                system_clock: 84_000_000,
                cycle_accurate: true,
                clock_config: Some(ClockConfig::default()),
                use_qemu: false,
            },
            Stm32F4Variant::Stm32F407 => SimulationConfig {
                mcu: "STM32F407".to_string(),
                architecture: CpuArchitecture::CortexM4,
                flash_size: 1024 * 1024,
                ram_size: 192 * 1024,
                flash_base: 0x0800_0000,
                ram_base: 0x2000_0000,
                system_clock: 168_000_000,
                cycle_accurate: true,
                clock_config: Some(ClockConfig {
                    pll_multiplier: 168 / 8,
                    ..ClockConfig::default()
                }),
                use_qemu: false,
            },
            Stm32F4Variant::Stm32F411 => SimulationConfig {
                mcu: "STM32F411".to_string(),
                architecture: CpuArchitecture::CortexM4,
                flash_size: 512 * 1024,
                ram_size: 128 * 1024,
                flash_base: 0x0800_0000,
                ram_base: 0x2000_0000,
                system_clock: 100_000_000,
                cycle_accurate: true,
                clock_config: Some(ClockConfig::default()),
                use_qemu: false,
            },
            Stm32F4Variant::Stm32F429 => SimulationConfig {
                mcu: "STM32F429".to_string(),
                architecture: CpuArchitecture::CortexM4,
                flash_size: 2048 * 1024,
                ram_size: 256 * 1024,
                flash_base: 0x0800_0000,
                ram_base: 0x2000_0000,
                system_clock: 180_000_000,
                cycle_accurate: true,
                clock_config: Some(ClockConfig {
                    pll_multiplier: 180 / 8,
                    ..ClockConfig::default()
                }),
                use_qemu: false,
            },
        }
    }

    fn memory_map(&self) -> Vec<MemoryMapEntry> {
        vec![
            MemoryMapEntry {
                name: "Flash".to_string(),
                base: 0x0800_0000,
                size: self.config().flash_size,
                region_type: "Flash".to_string(),
            },
            MemoryMapEntry {
                name: "SRAM".to_string(),
                base: 0x2000_0000,
                size: self.config().ram_size,
                region_type: "RAM".to_string(),
            },
            MemoryMapEntry {
                name: "Peripherals".to_string(),
                base: 0x4000_0000,
                size: 0x2000_0000,
                region_type: "Peripheral".to_string(),
            },
            MemoryMapEntry {
                name: "PPB".to_string(),
                base: 0xE000_0000,
                size: 0x0010_0000,
                region_type: "System".to_string(),
            },
        ]
    }

    fn peripherals(&self) -> Vec<String> {
        vec![
            "GPIOA-H".to_string(),
            "USART1-3,6".to_string(),
            "UART4-5".to_string(),
            "SPI1-4".to_string(),
            "I2C1-3".to_string(),
            "TIM1-14".to_string(),
            "ADC1-3".to_string(),
            "DMA1-2".to_string(),
            "CAN1-2".to_string(),
            "USB_OTG".to_string(),
        ]
    }
}

/// ESP32 MCU configuration
pub struct Esp32Sim {
    pub variant: Esp32Variant,
}

#[derive(Debug, Clone, Copy)]
pub enum Esp32Variant {
    Esp32,
    Esp32S3,
    Esp32C3,
}

impl Default for Esp32Sim {
    fn default() -> Self {
        Self { variant: Esp32Variant::Esp32 }
    }
}

impl McuSimulation for Esp32Sim {
    fn id(&self) -> &str {
        match self.variant {
            Esp32Variant::Esp32 => "ESP32",
            Esp32Variant::Esp32S3 => "ESP32-S3",
            Esp32Variant::Esp32C3 => "ESP32-C3",
        }
    }

    fn name(&self) -> &str {
        match self.variant {
            Esp32Variant::Esp32 => "ESP32 (240 MHz Xtensa, 520KB SRAM)",
            Esp32Variant::Esp32S3 => "ESP32-S3 (240 MHz Xtensa, 512KB SRAM)",
            Esp32Variant::Esp32C3 => "ESP32-C3 (160 MHz RISC-V, 400KB SRAM)",
        }
    }

    fn architecture(&self) -> CpuArchitecture {
        match self.variant {
            Esp32Variant::Esp32 | Esp32Variant::Esp32S3 => CpuArchitecture::Xtensa,
            Esp32Variant::Esp32C3 => CpuArchitecture::RiscV32,
        }
    }

    fn config(&self) -> SimulationConfig {
        match self.variant {
            Esp32Variant::Esp32 => SimulationConfig {
                mcu: "ESP32".to_string(),
                architecture: CpuArchitecture::Xtensa,
                flash_size: 4 * 1024 * 1024,
                ram_size: 520 * 1024,
                flash_base: 0x4008_0000,
                ram_base: 0x3FFB_0000,
                system_clock: 240_000_000,
                cycle_accurate: false,
                clock_config: None,
                use_qemu: true,
            },
            Esp32Variant::Esp32S3 => SimulationConfig {
                mcu: "ESP32-S3".to_string(),
                architecture: CpuArchitecture::Xtensa,
                flash_size: 8 * 1024 * 1024,
                ram_size: 512 * 1024,
                flash_base: 0x4200_0000,
                ram_base: 0x3FC8_0000,
                system_clock: 240_000_000,
                cycle_accurate: false,
                clock_config: None,
                use_qemu: true,
            },
            Esp32Variant::Esp32C3 => SimulationConfig {
                mcu: "ESP32-C3".to_string(),
                architecture: CpuArchitecture::RiscV32,
                flash_size: 4 * 1024 * 1024,
                ram_size: 400 * 1024,
                flash_base: 0x4200_0000,
                ram_base: 0x3FC8_0000,
                system_clock: 160_000_000,
                cycle_accurate: false,
                clock_config: None,
                use_qemu: true,
            },
        }
    }

    fn memory_map(&self) -> Vec<MemoryMapEntry> {
        vec![
            MemoryMapEntry {
                name: "Flash".to_string(),
                base: self.config().flash_base,
                size: self.config().flash_size,
                region_type: "Flash".to_string(),
            },
            MemoryMapEntry {
                name: "SRAM".to_string(),
                base: self.config().ram_base,
                size: self.config().ram_size,
                region_type: "RAM".to_string(),
            },
        ]
    }

    fn peripherals(&self) -> Vec<String> {
        vec![
            "GPIO".to_string(),
            "UART0-2".to_string(),
            "SPI0-3".to_string(),
            "I2C0-1".to_string(),
            "WiFi".to_string(),
            "Bluetooth/BLE".to_string(),
            "ADC1-2".to_string(),
            "DAC1-2".to_string(),
            "PWM".to_string(),
        ]
    }
}

/// RP2040 MCU configuration
pub struct Rp2040Sim;

impl McuSimulation for Rp2040Sim {
    fn id(&self) -> &str { "RP2040" }

    fn name(&self) -> &str { "RP2040 (133 MHz Cortex-M0+, 264KB SRAM)" }

    fn architecture(&self) -> CpuArchitecture { CpuArchitecture::CortexM0 }

    fn config(&self) -> SimulationConfig {
        SimulationConfig {
            mcu: "RP2040".to_string(),
            architecture: CpuArchitecture::CortexM0,
            flash_size: 2 * 1024 * 1024,
            ram_size: 264 * 1024,
            flash_base: 0x1000_0000,
            ram_base: 0x2000_0000,
            system_clock: 133_000_000,
            cycle_accurate: true,
            clock_config: None,
            use_qemu: false,
        }
    }

    fn memory_map(&self) -> Vec<MemoryMapEntry> {
        vec![
            MemoryMapEntry {
                name: "Flash (XIP)".to_string(),
                base: 0x1000_0000,
                size: 2 * 1024 * 1024,
                region_type: "Flash".to_string(),
            },
            MemoryMapEntry {
                name: "SRAM".to_string(),
                base: 0x2000_0000,
                size: 264 * 1024,
                region_type: "RAM".to_string(),
            },
        ]
    }

    fn peripherals(&self) -> Vec<String> {
        vec![
            "GPIO (30 pins)".to_string(),
            "UART0-1".to_string(),
            "SPI0-1".to_string(),
            "I2C0-1".to_string(),
            "PWM (16 channels)".to_string(),
            "ADC (4 channels)".to_string(),
            "PIO (2 blocks)".to_string(),
            "USB 1.1".to_string(),
        ]
    }
}

/// Nordic nRF52 configuration
pub struct Nrf52Sim {
    pub variant: Nrf52Variant,
}

#[derive(Debug, Clone, Copy)]
pub enum Nrf52Variant {
    Nrf52832,
    Nrf52840,
}

impl Default for Nrf52Sim {
    fn default() -> Self {
        Self { variant: Nrf52Variant::Nrf52840 }
    }
}

impl McuSimulation for Nrf52Sim {
    fn id(&self) -> &str {
        match self.variant {
            Nrf52Variant::Nrf52832 => "nRF52832",
            Nrf52Variant::Nrf52840 => "nRF52840",
        }
    }

    fn name(&self) -> &str {
        match self.variant {
            Nrf52Variant::Nrf52832 => "nRF52832 (64 MHz Cortex-M4F, 64KB RAM)",
            Nrf52Variant::Nrf52840 => "nRF52840 (64 MHz Cortex-M4F, 256KB RAM)",
        }
    }

    fn architecture(&self) -> CpuArchitecture { CpuArchitecture::CortexM4 }

    fn config(&self) -> SimulationConfig {
        match self.variant {
            Nrf52Variant::Nrf52832 => SimulationConfig {
                mcu: "nRF52832".to_string(),
                architecture: CpuArchitecture::CortexM4,
                flash_size: 512 * 1024,
                ram_size: 64 * 1024,
                flash_base: 0x0000_0000,
                ram_base: 0x2000_0000,
                system_clock: 64_000_000,
                cycle_accurate: true,
                clock_config: None,
                use_qemu: false,
            },
            Nrf52Variant::Nrf52840 => SimulationConfig {
                mcu: "nRF52840".to_string(),
                architecture: CpuArchitecture::CortexM4,
                flash_size: 1024 * 1024,
                ram_size: 256 * 1024,
                flash_base: 0x0000_0000,
                ram_base: 0x2000_0000,
                system_clock: 64_000_000,
                cycle_accurate: true,
                clock_config: None,
                use_qemu: false,
            },
        }
    }

    fn memory_map(&self) -> Vec<MemoryMapEntry> {
        vec![
            MemoryMapEntry {
                name: "Flash".to_string(),
                base: 0x0000_0000,
                size: self.config().flash_size,
                region_type: "Flash".to_string(),
            },
            MemoryMapEntry {
                name: "RAM".to_string(),
                base: 0x2000_0000,
                size: self.config().ram_size,
                region_type: "RAM".to_string(),
            },
        ]
    }

    fn peripherals(&self) -> Vec<String> {
        vec![
            "GPIO (32 pins)".to_string(),
            "UARTE".to_string(),
            "SPIM/SPIS".to_string(),
            "TWIM/TWIS".to_string(),
            "BLE 5.0".to_string(),
            "802.15.4".to_string(),
            "USB".to_string(),
            "ADC (SAADC)".to_string(),
            "PWM".to_string(),
        ]
    }
}

/// NXP LPC55 configuration
pub struct Lpc55Sim {
    pub variant: Lpc55Variant,
}

#[derive(Debug, Clone, Copy)]
pub enum Lpc55Variant {
    Lpc55S69,
    Lpc55S28,
    Lpc55S16,
}

impl Default for Lpc55Sim {
    fn default() -> Self {
        Self { variant: Lpc55Variant::Lpc55S69 }
    }
}

impl McuSimulation for Lpc55Sim {
    fn id(&self) -> &str {
        match self.variant {
            Lpc55Variant::Lpc55S69 => "LPC55S69",
            Lpc55Variant::Lpc55S28 => "LPC55S28",
            Lpc55Variant::Lpc55S16 => "LPC55S16",
        }
    }

    fn name(&self) -> &str {
        match self.variant {
            Lpc55Variant::Lpc55S69 => "LPC55S69 (150 MHz Cortex-M33, 320KB RAM)",
            Lpc55Variant::Lpc55S28 => "LPC55S28 (150 MHz Cortex-M33, 256KB RAM)",
            Lpc55Variant::Lpc55S16 => "LPC55S16 (150 MHz Cortex-M33, 96KB RAM)",
        }
    }

    fn architecture(&self) -> CpuArchitecture { CpuArchitecture::CortexM4 } // M33 uses similar instruction set

    fn config(&self) -> SimulationConfig {
        match self.variant {
            Lpc55Variant::Lpc55S69 => SimulationConfig {
                mcu: "LPC55S69".to_string(),
                architecture: CpuArchitecture::CortexM4,
                flash_size: 640 * 1024,
                ram_size: 320 * 1024,
                flash_base: 0x0000_0000,
                ram_base: 0x2000_0000,
                system_clock: 150_000_000,
                cycle_accurate: true,
                clock_config: None,
                use_qemu: false,
            },
            Lpc55Variant::Lpc55S28 => SimulationConfig {
                mcu: "LPC55S28".to_string(),
                architecture: CpuArchitecture::CortexM4,
                flash_size: 512 * 1024,
                ram_size: 256 * 1024,
                flash_base: 0x0000_0000,
                ram_base: 0x2000_0000,
                system_clock: 150_000_000,
                cycle_accurate: true,
                clock_config: None,
                use_qemu: false,
            },
            Lpc55Variant::Lpc55S16 => SimulationConfig {
                mcu: "LPC55S16".to_string(),
                architecture: CpuArchitecture::CortexM4,
                flash_size: 256 * 1024,
                ram_size: 96 * 1024,
                flash_base: 0x0000_0000,
                ram_base: 0x2000_0000,
                system_clock: 150_000_000,
                cycle_accurate: true,
                clock_config: None,
                use_qemu: false,
            },
        }
    }

    fn memory_map(&self) -> Vec<MemoryMapEntry> {
        vec![
            MemoryMapEntry {
                name: "Flash".to_string(),
                base: 0x0000_0000,
                size: self.config().flash_size,
                region_type: "Flash".to_string(),
            },
            MemoryMapEntry {
                name: "SRAM".to_string(),
                base: 0x2000_0000,
                size: self.config().ram_size,
                region_type: "RAM".to_string(),
            },
        ]
    }

    fn peripherals(&self) -> Vec<String> {
        vec![
            "GPIO (64 pins)".to_string(),
            "FLEXCOMM (UART/SPI/I2C)".to_string(),
            "USB HS".to_string(),
            "ADC (16-bit)".to_string(),
            "CTimer".to_string(),
            "SCTimer/PWM".to_string(),
            "CAN-FD".to_string(),
            "SDIO".to_string(),
            "Crypto (AES/SHA)".to_string(),
            "PUF".to_string(),
        ]
    }
}

/// Get list of all supported MCUs
pub fn get_supported_mcus() -> Vec<McuInfo> {
    vec![
        McuInfo { id: "STM32F401".to_string(), name: "STM32F401".to_string(), family: "STM32".to_string(), arch: "Cortex-M4".to_string() },
        McuInfo { id: "STM32F407".to_string(), name: "STM32F407".to_string(), family: "STM32".to_string(), arch: "Cortex-M4".to_string() },
        McuInfo { id: "STM32F411".to_string(), name: "STM32F411".to_string(), family: "STM32".to_string(), arch: "Cortex-M4".to_string() },
        McuInfo { id: "STM32F429".to_string(), name: "STM32F429".to_string(), family: "STM32".to_string(), arch: "Cortex-M4".to_string() },
        McuInfo { id: "ESP32".to_string(), name: "ESP32".to_string(), family: "ESP".to_string(), arch: "Xtensa".to_string() },
        McuInfo { id: "ESP32-S3".to_string(), name: "ESP32-S3".to_string(), family: "ESP".to_string(), arch: "Xtensa".to_string() },
        McuInfo { id: "ESP32-C3".to_string(), name: "ESP32-C3".to_string(), family: "ESP".to_string(), arch: "RISC-V".to_string() },
        McuInfo { id: "RP2040".to_string(), name: "RP2040".to_string(), family: "Raspberry Pi".to_string(), arch: "Cortex-M0+".to_string() },
        McuInfo { id: "nRF52832".to_string(), name: "nRF52832".to_string(), family: "Nordic".to_string(), arch: "Cortex-M4F".to_string() },
        McuInfo { id: "nRF52840".to_string(), name: "nRF52840".to_string(), family: "Nordic".to_string(), arch: "Cortex-M4F".to_string() },
        McuInfo { id: "LPC55S69".to_string(), name: "LPC55S69".to_string(), family: "NXP LPC".to_string(), arch: "Cortex-M33".to_string() },
        McuInfo { id: "LPC55S28".to_string(), name: "LPC55S28".to_string(), family: "NXP LPC".to_string(), arch: "Cortex-M33".to_string() },
        McuInfo { id: "LPC55S16".to_string(), name: "LPC55S16".to_string(), family: "NXP LPC".to_string(), arch: "Cortex-M33".to_string() },
    ]
}

/// MCU information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McuInfo {
    pub id: String,
    pub name: String,
    pub family: String,
    pub arch: String,
}

/// Create simulation config from MCU ID
pub fn create_config_for_mcu(mcu_id: &str) -> Option<SimulationConfig> {
    match mcu_id {
        "STM32F401" => Some(Stm32F4Sim { variant: Stm32F4Variant::Stm32F401 }.config()),
        "STM32F407" => Some(Stm32F4Sim { variant: Stm32F4Variant::Stm32F407 }.config()),
        "STM32F411" => Some(Stm32F4Sim { variant: Stm32F4Variant::Stm32F411 }.config()),
        "STM32F429" => Some(Stm32F4Sim { variant: Stm32F4Variant::Stm32F429 }.config()),
        "ESP32" => Some(Esp32Sim { variant: Esp32Variant::Esp32 }.config()),
        "ESP32-S3" => Some(Esp32Sim { variant: Esp32Variant::Esp32S3 }.config()),
        "ESP32-C3" => Some(Esp32Sim { variant: Esp32Variant::Esp32C3 }.config()),
        "RP2040" => Some(Rp2040Sim.config()),
        "nRF52832" => Some(Nrf52Sim { variant: Nrf52Variant::Nrf52832 }.config()),
        "nRF52840" => Some(Nrf52Sim { variant: Nrf52Variant::Nrf52840 }.config()),
        "LPC55S69" => Some(Lpc55Sim { variant: Lpc55Variant::Lpc55S69 }.config()),
        "LPC55S28" => Some(Lpc55Sim { variant: Lpc55Variant::Lpc55S28 }.config()),
        "LPC55S16" => Some(Lpc55Sim { variant: Lpc55Variant::Lpc55S16 }.config()),
        _ => None,
    }
}
