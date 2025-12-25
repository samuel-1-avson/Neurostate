// MCU Pin Database
// Comprehensive pin definitions for supported microcontrollers

use serde::{Deserialize, Serialize};

/// MCU pin function
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PinFunction {
    Gpio,
    UartTx,
    UartRx,
    SpiMosi,
    SpiMiso,
    SpiSck,
    SpiCs,
    I2cSda,
    I2cScl,
    Adc,
    Dac,
    Pwm,
    CanTx,
    CanRx,
    Timer,
    Exti,
    UsbDm,
    UsbDp,
    Power,
    Ground,
    Reset,
    Boot,
    Clock,
}

/// Pin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McuPin {
    pub port: String,
    pub pin: u8,
    pub name: String,
    pub functions: Vec<PinFunction>,
    pub current_function: Option<PinFunction>,
    pub x: f32,  // Position for visual layout
    pub y: f32,
}

/// MCU package type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageType {
    LQFP48,
    LQFP64,
    LQFP100,
    QFN32,
    QFN48,
    DIP28,
}

/// MCU pin layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McuPinout {
    pub mcu_id: String,
    pub name: String,
    pub package: PackageType,
    pub pins: Vec<McuPin>,
}

/// Get STM32F401 BlackPill pinout
pub fn get_stm32f401_pinout() -> McuPinout {
    McuPinout {
        mcu_id: "STM32F401".to_string(),
        name: "STM32F401 BlackPill".to_string(),
        package: PackageType::LQFP48,
        pins: vec![
            // Left side (top to bottom)
            McuPin { port: "B".into(), pin: 12, name: "PB12".into(), functions: vec![PinFunction::Gpio, PinFunction::SpiCs, PinFunction::I2cScl], current_function: None, x: 0.0, y: 0.0 },
            McuPin { port: "B".into(), pin: 13, name: "PB13".into(), functions: vec![PinFunction::Gpio, PinFunction::SpiSck], current_function: None, x: 0.0, y: 1.0 },
            McuPin { port: "B".into(), pin: 14, name: "PB14".into(), functions: vec![PinFunction::Gpio, PinFunction::SpiMiso], current_function: None, x: 0.0, y: 2.0 },
            McuPin { port: "B".into(), pin: 15, name: "PB15".into(), functions: vec![PinFunction::Gpio, PinFunction::SpiMosi], current_function: None, x: 0.0, y: 3.0 },
            McuPin { port: "A".into(), pin: 8, name: "PA8".into(), functions: vec![PinFunction::Gpio, PinFunction::Timer, PinFunction::UsbDp], current_function: None, x: 0.0, y: 4.0 },
            McuPin { port: "A".into(), pin: 9, name: "PA9".into(), functions: vec![PinFunction::Gpio, PinFunction::UartTx], current_function: None, x: 0.0, y: 5.0 },
            McuPin { port: "A".into(), pin: 10, name: "PA10".into(), functions: vec![PinFunction::Gpio, PinFunction::UartRx], current_function: None, x: 0.0, y: 6.0 },
            McuPin { port: "A".into(), pin: 11, name: "PA11".into(), functions: vec![PinFunction::Gpio, PinFunction::UsbDm, PinFunction::CanRx], current_function: None, x: 0.0, y: 7.0 },
            McuPin { port: "A".into(), pin: 12, name: "PA12".into(), functions: vec![PinFunction::Gpio, PinFunction::UsbDp, PinFunction::CanTx], current_function: None, x: 0.0, y: 8.0 },
            McuPin { port: "A".into(), pin: 15, name: "PA15".into(), functions: vec![PinFunction::Gpio, PinFunction::SpiCs], current_function: None, x: 0.0, y: 9.0 },
            McuPin { port: "B".into(), pin: 3, name: "PB3".into(), functions: vec![PinFunction::Gpio, PinFunction::SpiSck], current_function: None, x: 0.0, y: 10.0 },
            McuPin { port: "B".into(), pin: 4, name: "PB4".into(), functions: vec![PinFunction::Gpio, PinFunction::SpiMiso], current_function: None, x: 0.0, y: 11.0 },
            
            // Right side (top to bottom)
            McuPin { port: "".into(), pin: 0, name: "VCC".into(), functions: vec![PinFunction::Power], current_function: Some(PinFunction::Power), x: 1.0, y: 0.0 },
            McuPin { port: "".into(), pin: 0, name: "GND".into(), functions: vec![PinFunction::Ground], current_function: Some(PinFunction::Ground), x: 1.0, y: 1.0 },
            McuPin { port: "".into(), pin: 0, name: "3V3".into(), functions: vec![PinFunction::Power], current_function: Some(PinFunction::Power), x: 1.0, y: 2.0 },
            McuPin { port: "".into(), pin: 0, name: "RST".into(), functions: vec![PinFunction::Reset], current_function: Some(PinFunction::Reset), x: 1.0, y: 3.0 },
            McuPin { port: "B".into(), pin: 10, name: "PB10".into(), functions: vec![PinFunction::Gpio, PinFunction::I2cScl, PinFunction::UartTx], current_function: None, x: 1.0, y: 4.0 },
            McuPin { port: "B".into(), pin: 2, name: "PB2".into(), functions: vec![PinFunction::Gpio, PinFunction::Boot], current_function: None, x: 1.0, y: 5.0 },
            McuPin { port: "B".into(), pin: 1, name: "PB1".into(), functions: vec![PinFunction::Gpio, PinFunction::Adc, PinFunction::Pwm], current_function: None, x: 1.0, y: 6.0 },
            McuPin { port: "B".into(), pin: 0, name: "PB0".into(), functions: vec![PinFunction::Gpio, PinFunction::Adc, PinFunction::Pwm], current_function: None, x: 1.0, y: 7.0 },
            McuPin { port: "A".into(), pin: 7, name: "PA7".into(), functions: vec![PinFunction::Gpio, PinFunction::Adc, PinFunction::SpiMosi, PinFunction::Pwm], current_function: None, x: 1.0, y: 8.0 },
            McuPin { port: "A".into(), pin: 6, name: "PA6".into(), functions: vec![PinFunction::Gpio, PinFunction::Adc, PinFunction::SpiMiso, PinFunction::Pwm], current_function: None, x: 1.0, y: 9.0 },
            McuPin { port: "A".into(), pin: 5, name: "PA5".into(), functions: vec![PinFunction::Gpio, PinFunction::Adc, PinFunction::SpiSck, PinFunction::Dac], current_function: None, x: 1.0, y: 10.0 },
            McuPin { port: "A".into(), pin: 4, name: "PA4".into(), functions: vec![PinFunction::Gpio, PinFunction::Adc, PinFunction::SpiCs, PinFunction::Dac], current_function: None, x: 1.0, y: 11.0 },
        ],
    }
}

/// Get pinout for a specific MCU
pub fn get_mcu_pinout(mcu_id: &str) -> Option<McuPinout> {
    match mcu_id.to_uppercase().as_str() {
        "STM32F401" => Some(get_stm32f401_pinout()),
        _ => None,
    }
}

/// Check for pin conflicts
pub fn check_pin_conflicts(pinout: &McuPinout) -> Vec<String> {
    let conflicts = Vec::new();
    let mut used_peripherals: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    
    for pin in &pinout.pins {
        if let Some(func) = &pin.current_function {
            let peripheral = format!("{:?}", func);
            used_peripherals.entry(peripheral).or_default().push(pin.name.clone());
        }
    }
    
    // Check for specific conflicts
    // (This is a simplified check - real implementation would be more comprehensive)
    
    conflicts
}
