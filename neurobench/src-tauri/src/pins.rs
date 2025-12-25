// Pin Configuration Module
// Visual MCU pin assignment and configuration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pin function type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PinFunction {
    GPIO,
    Analog,
    UART,
    SPI,
    I2C,
    PWM,
    CAN,
    USB,
    Ethernet,
    SDIO,
    Input,
    Output,
    Alternate(u8),
}

/// Pin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinConfig {
    pub pin_name: String,
    pub port: String,
    pub pin_number: u8,
    pub function: String,
    pub mode: String,         // input, output, alternate, analog
    pub pull: String,         // none, up, down
    pub speed: String,        // low, medium, high, very_high
    pub alternate_function: Option<u8>,
    pub label: Option<String>,
}

/// MCU package pin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McuPackage {
    pub name: String,
    pub package: String,
    pub pin_count: u32,
    pub pins: Vec<PackagePin>,
}

/// Package pin info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagePin {
    pub number: u32,
    pub name: String,
    pub port: Option<String>,
    pub functions: Vec<String>,
}

/// Get available MCU packages
pub fn get_mcu_packages() -> Vec<McuPackage> {
    vec![
        McuPackage {
            name: "STM32F407VG".to_string(),
            package: "LQFP100".to_string(),
            pin_count: 100,
            pins: generate_stm32f4_pins(),
        },
        McuPackage {
            name: "STM32F103C8".to_string(),
            package: "LQFP48".to_string(),
            pin_count: 48,
            pins: generate_stm32f1_pins(),
        },
    ]
}

fn generate_stm32f4_pins() -> Vec<PackagePin> {
    let mut pins = Vec::new();
    
    // GPIO Ports A-E
    for port in ['A', 'B', 'C', 'D', 'E'] {
        for pin in 0..16 {
            pins.push(PackagePin {
                number: pins.len() as u32 + 1,
                name: format!("P{}{}", port, pin),
                port: Some(format!("GPIO{}", port)),
                functions: get_pin_functions(port, pin),
            });
        }
    }
    
    // Power pins
    pins.push(PackagePin {
        number: pins.len() as u32 + 1,
        name: "VDD".to_string(),
        port: None,
        functions: vec!["Power".to_string()],
    });
    
    pins.push(PackagePin {
        number: pins.len() as u32 + 1,
        name: "VSS".to_string(),
        port: None,
        functions: vec!["Ground".to_string()],
    });
    
    pins
}

fn generate_stm32f1_pins() -> Vec<PackagePin> {
    let mut pins = Vec::new();
    
    // GPIO Ports A-C
    for port in ['A', 'B', 'C'] {
        for pin in 0..16 {
            pins.push(PackagePin {
                number: pins.len() as u32 + 1,
                name: format!("P{}{}", port, pin),
                port: Some(format!("GPIO{}", port)),
                functions: get_pin_functions(port, pin),
            });
        }
    }
    
    pins
}

fn get_pin_functions(port: char, pin: u8) -> Vec<String> {
    let mut functions = vec!["GPIO".to_string()];
    
    match (port, pin) {
        ('A', 0) => functions.extend(vec!["TIM2_CH1".to_string(), "ADC1_IN0".to_string()]),
        ('A', 1) => functions.extend(vec!["TIM2_CH2".to_string(), "ADC1_IN1".to_string()]),
        ('A', 2) => functions.extend(vec!["USART2_TX".to_string(), "TIM2_CH3".to_string()]),
        ('A', 3) => functions.extend(vec!["USART2_RX".to_string(), "TIM2_CH4".to_string()]),
        ('A', 4) => functions.extend(vec!["SPI1_NSS".to_string(), "DAC1_OUT".to_string()]),
        ('A', 5) => functions.extend(vec!["SPI1_SCK".to_string(), "DAC2_OUT".to_string()]),
        ('A', 6) => functions.extend(vec!["SPI1_MISO".to_string(), "TIM3_CH1".to_string()]),
        ('A', 7) => functions.extend(vec!["SPI1_MOSI".to_string(), "TIM3_CH2".to_string()]),
        ('A', 8) => functions.extend(vec!["MCO1".to_string(), "TIM1_CH1".to_string()]),
        ('A', 9) => functions.extend(vec!["USART1_TX".to_string(), "TIM1_CH2".to_string()]),
        ('A', 10) => functions.extend(vec!["USART1_RX".to_string(), "TIM1_CH3".to_string()]),
        ('A', 11) => functions.extend(vec!["USB_DM".to_string(), "CAN1_RX".to_string()]),
        ('A', 12) => functions.extend(vec!["USB_DP".to_string(), "CAN1_TX".to_string()]),
        ('A', 13) => functions.extend(vec!["SWDIO".to_string()]),
        ('A', 14) => functions.extend(vec!["SWCLK".to_string()]),
        ('A', 15) => functions.extend(vec!["SPI3_NSS".to_string(), "TIM2_CH1".to_string()]),
        
        ('B', 0) => functions.extend(vec!["ADC1_IN8".to_string(), "TIM3_CH3".to_string()]),
        ('B', 1) => functions.extend(vec!["ADC1_IN9".to_string(), "TIM3_CH4".to_string()]),
        ('B', 3) => functions.extend(vec!["SPI3_SCK".to_string(), "TIM2_CH2".to_string()]),
        ('B', 4) => functions.extend(vec!["SPI3_MISO".to_string(), "TIM3_CH1".to_string()]),
        ('B', 5) => functions.extend(vec!["SPI3_MOSI".to_string(), "TIM3_CH2".to_string()]),
        ('B', 6) => functions.extend(vec!["I2C1_SCL".to_string(), "TIM4_CH1".to_string()]),
        ('B', 7) => functions.extend(vec!["I2C1_SDA".to_string(), "TIM4_CH2".to_string()]),
        ('B', 8) => functions.extend(vec!["I2C1_SCL".to_string(), "TIM4_CH3".to_string()]),
        ('B', 9) => functions.extend(vec!["I2C1_SDA".to_string(), "TIM4_CH4".to_string()]),
        ('B', 10) => functions.extend(vec!["USART3_TX".to_string(), "I2C2_SCL".to_string()]),
        ('B', 11) => functions.extend(vec!["USART3_RX".to_string(), "I2C2_SDA".to_string()]),
        
        ('C', 13) => functions.extend(vec!["LED".to_string()]),
        ('C', 14) => functions.extend(vec!["OSC32_IN".to_string()]),
        ('C', 15) => functions.extend(vec!["OSC32_OUT".to_string()]),
        
        _ => {}
    }
    
    functions
}

/// Generate GPIO initialization code
pub fn generate_pin_init_code(configs: &[PinConfig]) -> String {
    let mut code = String::new();
    code.push_str("// Auto-generated pin configuration\n");
    code.push_str("#include \"stm32f4xx.h\"\n\n");
    code.push_str("void GPIO_Init(void) {\n");
    
    // Enable clocks
    let mut ports: Vec<char> = configs.iter()
        .filter_map(|c| c.port.chars().last())
        .collect();
    ports.sort();
    ports.dedup();
    
    code.push_str("    // Enable GPIO clocks\n");
    for port in &ports {
        code.push_str(&format!("    RCC->AHB1ENR |= RCC_AHB1ENR_GPIO{}EN;\n", port));
    }
    code.push_str("\n");
    
    // Configure pins
    for config in configs {
        if let Some(port_char) = config.port.chars().last() {
            code.push_str(&format!("    // {} - {}\n", config.pin_name, 
                config.label.as_ref().unwrap_or(&config.function)));
            
            let mode_val = match config.mode.as_str() {
                "input" => 0,
                "output" => 1,
                "alternate" => 2,
                "analog" => 3,
                _ => 0,
            };
            
            code.push_str(&format!(
                "    GPIO{}->MODER = (GPIO{}->MODER & ~(3U << {})) | ({}U << {});\n",
                port_char, port_char, config.pin_number * 2, mode_val, config.pin_number * 2
            ));
        }
    }
    
    code.push_str("}\n");
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_packages() {
        let packages = get_mcu_packages();
        assert!(!packages.is_empty());
    }

    #[test]
    fn test_generate_code() {
        let configs = vec![
            PinConfig {
                pin_name: "PA0".to_string(),
                port: "GPIOA".to_string(),
                pin_number: 0,
                function: "GPIO".to_string(),
                mode: "output".to_string(),
                pull: "none".to_string(),
                speed: "high".to_string(),
                alternate_function: None,
                label: Some("LED".to_string()),
            }
        ];
        
        let code = generate_pin_init_code(&configs);
        assert!(code.contains("GPIO_Init"));
    }
}
