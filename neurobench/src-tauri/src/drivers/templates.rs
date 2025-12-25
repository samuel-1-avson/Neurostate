// Driver Templates and Common Types
// Shared types for driver generation

use serde::{Deserialize, Serialize};

// ============================================================================
// Common Types for Driver Generation
// ============================================================================

/// MCU Architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McuArch {
    ArmCortexM0,
    ArmCortexM3,
    ArmCortexM4,
    ArmCortexM7,
    Stm32,
    Esp32,
    Avr,
    RiscV,
}

impl Default for McuArch {
    fn default() -> Self {
        McuArch::ArmCortexM4
    }
}

/// Driver output language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriverLanguage {
    C,
    Cpp,
    Rust,
}

impl Default for DriverLanguage {
    fn default() -> Self {
        DriverLanguage::C
    }
}

/// Peripheral type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeripheralType {
    GPIO,
    SPI,
    I2C,
    UART,
    Timer,
    ADC,
    DAC,
    PWM,
    CAN,
    USB,
    Ethernet,
    DMA,
    Modbus,
}

/// Driver output structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverOutput {
    pub header_file: Option<String>,
    pub source_file: String,
    pub example_file: Option<String>,
    pub peripheral_type: PeripheralType,
}

/// Bit order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BitOrder {
    MsbFirst,
    LsbFirst,
}

/// Stop bits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StopBits {
    One,
    OnePointFive,
    Two,
}

impl Default for StopBits {
    fn default() -> Self {
        StopBits::One
    }
}

/// Driver request for generator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverRequest {
    pub peripheral_type: PeripheralType,
    pub arch: McuArch,
    pub mcu_arch: McuArch,
    pub language: DriverLanguage,
    pub gpio_config: Option<GpioConfig>,
    pub uart_config: Option<UartConfig>,
    pub spi_config: Option<SpiConfig>,
    pub i2c_config: Option<I2cConfig>,
}

// ============================================================================
// SPI Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiConfig {
    pub instance: String,
    pub clock_hz: u32,
    pub mode: SpiMode,
    pub data_bits: u8,
    pub msb_first: bool,
    pub bit_order: BitOrder,
    pub mosi_pin: Option<String>,
    pub miso_pin: Option<String>,
    pub sck_pin: Option<String>,
    pub cs_pin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpiMode {
    Mode0,  // CPOL=0, CPHA=0
    Mode1,  // CPOL=0, CPHA=1
    Mode2,  // CPOL=1, CPHA=0
    Mode3,  // CPOL=1, CPHA=1
}

impl Default for SpiConfig {
    fn default() -> Self {
        Self {
            instance: "SPI1".to_string(),
            clock_hz: 1_000_000,
            mode: SpiMode::Mode0,
            data_bits: 8,
            msb_first: true,
            bit_order: BitOrder::MsbFirst,
            mosi_pin: None,
            miso_pin: None,
            sck_pin: None,
            cs_pin: None,
        }
    }
}

// ============================================================================
// I2C Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I2cConfig {
    pub instance: String,
    pub speed: I2cSpeed,
    pub address_bits: u8,
    pub address: Option<u8>,
    pub sda_pin: Option<String>,
    pub scl_pin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum I2cSpeed {
    Standard,   // 100 kHz
    Fast,       // 400 kHz
    FastPlus,   // 1 MHz
}

impl Default for I2cConfig {
    fn default() -> Self {
        Self {
            instance: "I2C1".to_string(),
            speed: I2cSpeed::Fast,
            address_bits: 7,
            address: None,
            sda_pin: None,
            scl_pin: None,
        }
    }
}

// ============================================================================
// UART Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UartConfig {
    pub instance: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: StopBits,
    pub parity: UartParity,
    pub flow_control: bool,
    pub tx_pin: Option<String>,
    pub rx_pin: Option<String>,
    pub use_dma: bool,
    pub use_interrupt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UartParity {
    None,
    Even,
    Odd,
}

/// Parity alias for backwards compatibility
pub type Parity = UartParity;

impl Default for UartConfig {
    fn default() -> Self {
        Self {
            instance: "USART1".to_string(),
            baud_rate: 115200,
            data_bits: 8,
            stop_bits: StopBits::One,
            parity: UartParity::None,
            flow_control: false,
            tx_pin: None,
            rx_pin: None,
            use_dma: false,
            use_interrupt: false,
        }
    }
}

// ============================================================================
// Timer Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerConfig {
    pub instance: String,
    pub frequency_hz: u32,
    pub mode: TimerMode,
    pub auto_reload: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimerMode {
    Basic,
    PWM,
    InputCapture,
    OutputCompare,
    OnePulse,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            instance: "TIM1".to_string(),
            frequency_hz: 1000,
            mode: TimerMode::Basic,
            auto_reload: true,
        }
    }
}

// ============================================================================
// ADC Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdcConfig {
    pub instance: String,
    pub resolution_bits: u8,
    pub channels: Vec<u8>,
    pub continuous: bool,
    pub dma: bool,
}

impl Default for AdcConfig {
    fn default() -> Self {
        Self {
            instance: "ADC1".to_string(),
            resolution_bits: 12,
            channels: vec![0],
            continuous: false,
            dma: false,
        }
    }
}

// ============================================================================
// GPIO Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpioConfig {
    pub port: String,
    pub pin: u8,
    pub mode: GpioMode,
    pub pull: GpioPull,
    pub speed: GpioSpeed,
    pub initial_state: Option<bool>,
    pub alternate_function: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpioMode {
    Input,
    Output,
    AlternateFunction,
    Analog,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpioPull {
    None,
    Up,
    Down,
    PullUp,    // Alias
    PullDown,  // Alias
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpioSpeed {
    Low,
    Medium,
    High,
    VeryHigh,
}

impl Default for GpioConfig {
    fn default() -> Self {
        Self {
            port: "A".to_string(),
            pin: 0,
            mode: GpioMode::Input,
            pull: GpioPull::None,
            speed: GpioSpeed::Low,
            initial_state: None,
            alternate_function: None,
        }
    }
}

// ============================================================================
// PWM Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PwmConfig {
    pub timer_instance: String,
    pub channel: u8,
    pub frequency_hz: u32,
    pub duty_cycle_percent: f32,
}

impl Default for PwmConfig {
    fn default() -> Self {
        Self {
            timer_instance: "TIM1".to_string(),
            channel: 1,
            frequency_hz: 1000,
            duty_cycle_percent: 50.0,
        }
    }
}

// ============================================================================
// DMA Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmaConfig {
    pub stream: u8,
    pub channel: u8,
    pub direction: DmaDirection,
    pub circular: bool,
    pub mem_inc: bool,
    pub periph_inc: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DmaDirection {
    PeriphToMem,
    MemToPeriph,
    MemToMem,
}

impl Default for DmaConfig {
    fn default() -> Self {
        Self {
            stream: 0,
            channel: 0,
            direction: DmaDirection::PeriphToMem,
            circular: false,
            mem_inc: true,
            periph_inc: false,
        }
    }
}

// ============================================================================
// Interrupt Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterruptConfig {
    pub name: String,
    pub priority: u8,
    pub subpriority: u8,
    pub handler_name: String,
}

impl Default for InterruptConfig {
    fn default() -> Self {
        Self {
            name: "EXTI0".to_string(),
            priority: 5,
            subpriority: 0,
            handler_name: "EXTI0_IRQHandler".to_string(),
        }
    }
}

// ============================================================================
// Modbus Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModbusConfig {
    pub mode: ModbusMode,
    pub slave_address: u8,
    pub uart_instance: String,
    pub baud_rate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModbusMode {
    RTU,
    ASCII,
    TCP,
}

impl Default for ModbusConfig {
    fn default() -> Self {
        Self {
            mode: ModbusMode::RTU,
            slave_address: 1,
            uart_instance: "USART1".to_string(),
            baud_rate: 9600,
        }
    }
}

// ============================================================================
// Project Templates (for code export)
// ============================================================================

/// Project template type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateType {
    Stm32FreeRTOS,
    Stm32BareMetal,
    Esp32WiFiOta,
    Esp32BLE,
    Nrf52Zephyr,
    Nrf52BareMetal,
    Rp2040Basic,
}

/// Project template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTemplate {
    pub name: String,
    pub template_type: TemplateType,
    pub description: String,
    pub mcu: String,
    pub files: Vec<TemplateFile>,
}

/// Template file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
}

/// Get STM32 + FreeRTOS template
pub fn get_stm32_freertos_template(project_name: &str) -> ProjectTemplate {
    ProjectTemplate {
        name: project_name.to_string(),
        template_type: TemplateType::Stm32FreeRTOS,
        description: "STM32F4 with FreeRTOS".to_string(),
        mcu: "STM32F407".to_string(),
        files: vec![
            TemplateFile {
                path: "Src/main.c".to_string(),
                content: format!("// {} - STM32F4 + FreeRTOS\n#include \"main.h\"\n", project_name),
            },
        ],
    }
}

/// Get ESP32 + WiFi + OTA template
pub fn get_esp32_wifi_ota_template(project_name: &str) -> ProjectTemplate {
    ProjectTemplate {
        name: project_name.to_string(),
        template_type: TemplateType::Esp32WiFiOta,
        description: "ESP32 with WiFi and OTA".to_string(),
        mcu: "ESP32".to_string(),
        files: vec![
            TemplateFile {
                path: "main/main.c".to_string(),
                content: format!("// {} - ESP32 WiFi + OTA\n#include \"esp_wifi.h\"\n", project_name),
            },
        ],
    }
}

/// Get nRF52 + Zephyr template
pub fn get_nrf52_zephyr_template(project_name: &str) -> ProjectTemplate {
    ProjectTemplate {
        name: project_name.to_string(),
        template_type: TemplateType::Nrf52Zephyr,
        description: "nRF52840 with Zephyr RTOS".to_string(),
        mcu: "nRF52840".to_string(),
        files: vec![
            TemplateFile {
                path: "src/main.c".to_string(),
                content: format!("// {} - nRF52 Zephyr\n#include <zephyr/kernel.h>\n", project_name),
            },
        ],
    }
}

/// Get available templates list
pub fn get_available_templates() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("stm32_freertos", "STM32 + FreeRTOS", "STM32F4 with FreeRTOS"),
        ("esp32_wifi_ota", "ESP32 WiFi + OTA", "ESP32 with WiFi and OTA"),
        ("nrf52_zephyr", "nRF52 + Zephyr", "nRF52840 with Zephyr RTOS"),
    ]
}
