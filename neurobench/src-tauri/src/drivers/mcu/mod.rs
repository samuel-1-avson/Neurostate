// Multi-MCU Abstraction Layer
// Provides a unified interface for code generation across different MCU families

use serde::{Deserialize, Serialize};

/// Supported MCU Families
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum McuFamily {
    STM32F1,
    STM32F4,
    STM32H7,
    STM32L4,
    STM32G4,
    ESP32,
    ESP32S3,
    ESP32C3,
    RP2040,
    NRF52832,
    NRF52840,
    LPC1768,
    LPC5500,
}

impl McuFamily {
    pub fn display_name(&self) -> &'static str {
        match self {
            McuFamily::STM32F1 => "STM32F1 (Cortex-M3)",
            McuFamily::STM32F4 => "STM32F4 (Cortex-M4F)",
            McuFamily::STM32H7 => "STM32H7 (Cortex-M7)",
            McuFamily::STM32L4 => "STM32L4 (Low Power)",
            McuFamily::STM32G4 => "STM32G4 (Mixed Signal)",
            McuFamily::ESP32 => "ESP32 (Xtensa LX6)",
            McuFamily::ESP32S3 => "ESP32-S3 (Xtensa LX7)",
            McuFamily::ESP32C3 => "ESP32-C3 (RISC-V)",
            McuFamily::RP2040 => "RP2040 (Dual Cortex-M0+)",
            McuFamily::NRF52832 => "nRF52832 (BLE)",
            McuFamily::NRF52840 => "nRF52840 (BLE5/Thread)",
            McuFamily::LPC1768 => "LPC1768 (Cortex-M3)",
            McuFamily::LPC5500 => "LPC5500 (Cortex-M33)",
        }
    }

    pub fn vendor(&self) -> &'static str {
        match self {
            McuFamily::STM32F1 | McuFamily::STM32F4 | McuFamily::STM32H7 |
            McuFamily::STM32L4 | McuFamily::STM32G4 => "STMicroelectronics",
            McuFamily::ESP32 | McuFamily::ESP32S3 | McuFamily::ESP32C3 => "Espressif",
            McuFamily::RP2040 => "Raspberry Pi",
            McuFamily::NRF52832 | McuFamily::NRF52840 => "Nordic Semiconductor",
            McuFamily::LPC1768 | McuFamily::LPC5500 => "NXP",
        }
    }

    pub fn architecture(&self) -> &'static str {
        match self {
            McuFamily::STM32F1 | McuFamily::LPC1768 => "ARM Cortex-M3",
            McuFamily::STM32F4 | McuFamily::STM32G4 | McuFamily::NRF52832 | McuFamily::NRF52840 => "ARM Cortex-M4F",
            McuFamily::STM32H7 => "ARM Cortex-M7",
            McuFamily::STM32L4 => "ARM Cortex-M4",
            McuFamily::LPC5500 => "ARM Cortex-M33",
            McuFamily::RP2040 => "ARM Cortex-M0+",
            McuFamily::ESP32 | McuFamily::ESP32S3 => "Xtensa LX6/LX7",
            McuFamily::ESP32C3 => "RISC-V",
        }
    }

    pub fn has_fpu(&self) -> bool {
        matches!(self, 
            McuFamily::STM32F4 | McuFamily::STM32H7 | McuFamily::STM32L4 | 
            McuFamily::STM32G4 | McuFamily::NRF52832 | McuFamily::NRF52840 |
            McuFamily::ESP32 | McuFamily::ESP32S3
        )
    }

    pub fn has_dsp(&self) -> bool {
        matches!(self,
            McuFamily::STM32F4 | McuFamily::STM32H7 | McuFamily::STM32G4 |
            McuFamily::ESP32 | McuFamily::ESP32S3
        )
    }

    pub fn has_ble(&self) -> bool {
        matches!(self,
            McuFamily::NRF52832 | McuFamily::NRF52840 | McuFamily::ESP32 | McuFamily::ESP32S3
        )
    }

    pub fn has_wifi(&self) -> bool {
        matches!(self, McuFamily::ESP32 | McuFamily::ESP32S3 | McuFamily::ESP32C3)
    }

    pub fn max_frequency_mhz(&self) -> u32 {
        match self {
            McuFamily::STM32F1 => 72,
            McuFamily::STM32F4 => 168,
            McuFamily::STM32H7 => 480,
            McuFamily::STM32L4 => 80,
            McuFamily::STM32G4 => 170,
            McuFamily::ESP32 => 240,
            McuFamily::ESP32S3 => 240,
            McuFamily::ESP32C3 => 160,
            McuFamily::RP2040 => 133,
            McuFamily::NRF52832 => 64,
            McuFamily::NRF52840 => 64,
            McuFamily::LPC1768 => 100,
            McuFamily::LPC5500 => 150,
        }
    }

    pub fn flash_kb(&self) -> u32 {
        match self {
            McuFamily::STM32F1 => 128,
            McuFamily::STM32F4 => 1024,
            McuFamily::STM32H7 => 2048,
            McuFamily::STM32L4 => 512,
            McuFamily::STM32G4 => 512,
            McuFamily::ESP32 => 4096,
            McuFamily::ESP32S3 => 8192,
            McuFamily::ESP32C3 => 4096,
            McuFamily::RP2040 => 2048,
            McuFamily::NRF52832 => 512,
            McuFamily::NRF52840 => 1024,
            McuFamily::LPC1768 => 512,
            McuFamily::LPC5500 => 640,
        }
    }

    pub fn ram_kb(&self) -> u32 {
        match self {
            McuFamily::STM32F1 => 20,
            McuFamily::STM32F4 => 192,
            McuFamily::STM32H7 => 1024,
            McuFamily::STM32L4 => 128,
            McuFamily::STM32G4 => 128,
            McuFamily::ESP32 => 520,
            McuFamily::ESP32S3 => 512,
            McuFamily::ESP32C3 => 400,
            McuFamily::RP2040 => 264,
            McuFamily::NRF52832 => 64,
            McuFamily::NRF52840 => 256,
            McuFamily::LPC1768 => 64,
            McuFamily::LPC5500 => 320,
        }
    }
}

/// MCU Information for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McuInfo {
    pub family: McuFamily,
    pub display_name: String,
    pub vendor: String,
    pub architecture: String,
    pub max_freq_mhz: u32,
    pub flash_kb: u32,
    pub ram_kb: u32,
    pub has_fpu: bool,
    pub has_dsp: bool,
    pub has_ble: bool,
    pub has_wifi: bool,
}

impl From<McuFamily> for McuInfo {
    fn from(family: McuFamily) -> Self {
        Self {
            family,
            display_name: family.display_name().to_string(),
            vendor: family.vendor().to_string(),
            architecture: family.architecture().to_string(),
            max_freq_mhz: family.max_frequency_mhz(),
            flash_kb: family.flash_kb(),
            ram_kb: family.ram_kb(),
            has_fpu: family.has_fpu(),
            has_dsp: family.has_dsp(),
            has_ble: family.has_ble(),
            has_wifi: family.has_wifi(),
        }
    }
}

/// Get all supported MCUs
pub fn get_all_mcus() -> Vec<McuInfo> {
    vec![
        McuFamily::STM32F1.into(),
        McuFamily::STM32F4.into(),
        McuFamily::STM32H7.into(),
        McuFamily::STM32L4.into(),
        McuFamily::STM32G4.into(),
        McuFamily::ESP32.into(),
        McuFamily::ESP32S3.into(),
        McuFamily::ESP32C3.into(),
        McuFamily::RP2040.into(),
        McuFamily::NRF52832.into(),
        McuFamily::NRF52840.into(),
        McuFamily::LPC1768.into(),
        McuFamily::LPC5500.into(),
    ]
}

/// GPIO Configuration (platform-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpioConfig {
    pub pin: String,
    pub mode: GpioMode,
    pub pull: GpioPull,
    pub speed: GpioSpeed,
    pub initial_state: Option<bool>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GpioMode {
    Input,
    Output,
    AlternateFunction(u8),
    Analog,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GpioPull {
    None,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GpioSpeed {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// SPI Configuration (platform-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiConfigAbstract {
    pub instance: u8,
    pub mode: u8,          // 0-3
    pub clock_hz: u32,
    pub data_bits: u8,     // 8 or 16
    pub msb_first: bool,
    pub dma: bool,
}

/// I2C Configuration (platform-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I2cConfigAbstract {
    pub instance: u8,
    pub speed: I2cSpeedAbstract,
    pub address_bits: u8,  // 7 or 10
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum I2cSpeedAbstract {
    Standard100k,
    Fast400k,
    FastPlus1m,
}

/// UART Configuration (platform-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UartConfigAbstract {
    pub instance: u8,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: UartParity,
    pub stop_bits: u8,
    pub flow_control: bool,
    pub dma: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UartParity {
    None,
    Even,
    Odd,
}

/// Timer Configuration (platform-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerConfigAbstract {
    pub instance: u8,
    pub frequency_hz: u32,
    pub period_us: u32,
    pub pwm_channels: Vec<PwmChannelAbstract>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PwmChannelAbstract {
    pub channel: u8,
    pub duty_percent: f32,
}

/// ADC Configuration (platform-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdcConfigAbstract {
    pub instance: u8,
    pub resolution_bits: u8,
    pub channels: Vec<u8>,
    pub continuous: bool,
    pub dma: bool,
}

/// MCU HAL Trait - All MCU implementations must provide these
pub trait McuHal {
    fn family(&self) -> McuFamily;
    
    fn generate_gpio(&self, config: &GpioConfig) -> String;
    fn generate_spi(&self, config: &SpiConfigAbstract) -> String;
    fn generate_i2c(&self, config: &I2cConfigAbstract) -> String;
    fn generate_uart(&self, config: &UartConfigAbstract) -> String;
    fn generate_timer(&self, config: &TimerConfigAbstract) -> String;
    fn generate_adc(&self, config: &AdcConfigAbstract) -> String;
    fn generate_clock_init(&self, freq_mhz: u32) -> String;
    fn generate_system_init(&self) -> String;
    
    fn include_headers(&self) -> Vec<&'static str>;
    fn linker_script(&self) -> &'static str;
    fn startup_file(&self) -> &'static str;
}

pub mod stm32;
pub mod esp32;
pub mod rp2040;
pub mod nordic;
pub mod nxp;
