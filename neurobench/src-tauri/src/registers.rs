// Register Viewer Module
// Low-level MCU register inspection and modification

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Register definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Register {
    pub name: String,
    pub address: u32,
    pub size: u8,  // bits: 8, 16, 32
    pub reset_value: u32,
    pub description: String,
    pub fields: Vec<RegisterField>,
}

/// Register field (bit field)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterField {
    pub name: String,
    pub bit_offset: u8,
    pub bit_width: u8,
    pub access: String,  // r, w, rw
    pub description: String,
    pub values: Vec<FieldValue>,
}

/// Possible field value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValue {
    pub value: u32,
    pub name: String,
    pub description: String,
}

/// Peripheral definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peripheral {
    pub name: String,
    pub base_address: u32,
    pub description: String,
    pub registers: Vec<Register>,
}

/// Get STM32F4 GPIO registers
pub fn get_gpio_registers(port: char) -> Peripheral {
    let base = match port {
        'A' => 0x4002_0000,
        'B' => 0x4002_0400,
        'C' => 0x4002_0800,
        'D' => 0x4002_0C00,
        'E' => 0x4002_1000,
        _ => 0x4002_0000,
    };
    
    Peripheral {
        name: format!("GPIO{}", port),
        base_address: base,
        description: format!("General Purpose I/O Port {}", port),
        registers: vec![
            Register {
                name: "MODER".to_string(),
                address: base,
                size: 32,
                reset_value: 0x0000_0000,
                description: "GPIO port mode register".to_string(),
                fields: (0..16).map(|i| RegisterField {
                    name: format!("MODER{}", i),
                    bit_offset: i * 2,
                    bit_width: 2,
                    access: "rw".to_string(),
                    description: format!("Port x configuration bits (y = {})", i),
                    values: vec![
                        FieldValue { value: 0, name: "Input".to_string(), description: "Input mode".to_string() },
                        FieldValue { value: 1, name: "Output".to_string(), description: "General purpose output".to_string() },
                        FieldValue { value: 2, name: "Alternate".to_string(), description: "Alternate function".to_string() },
                        FieldValue { value: 3, name: "Analog".to_string(), description: "Analog mode".to_string() },
                    ],
                }).collect(),
            },
            Register {
                name: "ODR".to_string(),
                address: base + 0x14,
                size: 32,
                reset_value: 0x0000_0000,
                description: "GPIO port output data register".to_string(),
                fields: (0..16).map(|i| RegisterField {
                    name: format!("ODR{}", i),
                    bit_offset: i,
                    bit_width: 1,
                    access: "rw".to_string(),
                    description: format!("Port output data bit {}", i),
                    values: vec![
                        FieldValue { value: 0, name: "Low".to_string(), description: "Output low".to_string() },
                        FieldValue { value: 1, name: "High".to_string(), description: "Output high".to_string() },
                    ],
                }).collect(),
            },
            Register {
                name: "IDR".to_string(),
                address: base + 0x10,
                size: 32,
                reset_value: 0x0000_0000,
                description: "GPIO port input data register".to_string(),
                fields: (0..16).map(|i| RegisterField {
                    name: format!("IDR{}", i),
                    bit_offset: i,
                    bit_width: 1,
                    access: "r".to_string(),
                    description: format!("Port input data bit {}", i),
                    values: vec![
                        FieldValue { value: 0, name: "Low".to_string(), description: "Input low".to_string() },
                        FieldValue { value: 1, name: "High".to_string(), description: "Input high".to_string() },
                    ],
                }).collect(),
            },
            Register {
                name: "PUPDR".to_string(),
                address: base + 0x0C,
                size: 32,
                reset_value: 0x0000_0000,
                description: "GPIO port pull-up/pull-down register".to_string(),
                fields: (0..16).map(|i| RegisterField {
                    name: format!("PUPDR{}", i),
                    bit_offset: i * 2,
                    bit_width: 2,
                    access: "rw".to_string(),
                    description: format!("Port x configuration bits (y = {})", i),
                    values: vec![
                        FieldValue { value: 0, name: "None".to_string(), description: "No pull-up/down".to_string() },
                        FieldValue { value: 1, name: "Pull-up".to_string(), description: "Pull-up enabled".to_string() },
                        FieldValue { value: 2, name: "Pull-down".to_string(), description: "Pull-down enabled".to_string() },
                    ],
                }).collect(),
            },
        ],
    }
}

/// Get RCC (Reset and Clock Control) registers
pub fn get_rcc_registers() -> Peripheral {
    let base = 0x4002_3800u32;
    
    Peripheral {
        name: "RCC".to_string(),
        base_address: base,
        description: "Reset and Clock Control".to_string(),
        registers: vec![
            Register {
                name: "CR".to_string(),
                address: base,
                size: 32,
                reset_value: 0x0000_0083,
                description: "Clock control register".to_string(),
                fields: vec![
                    RegisterField {
                        name: "HSION".to_string(),
                        bit_offset: 0,
                        bit_width: 1,
                        access: "rw".to_string(),
                        description: "Internal high-speed clock enable".to_string(),
                        values: vec![
                            FieldValue { value: 0, name: "Off".to_string(), description: "HSI disabled".to_string() },
                            FieldValue { value: 1, name: "On".to_string(), description: "HSI enabled".to_string() },
                        ],
                    },
                    RegisterField {
                        name: "HSIRDY".to_string(),
                        bit_offset: 1,
                        bit_width: 1,
                        access: "r".to_string(),
                        description: "Internal high-speed clock ready".to_string(),
                        values: vec![
                            FieldValue { value: 0, name: "NotReady".to_string(), description: "HSI not ready".to_string() },
                            FieldValue { value: 1, name: "Ready".to_string(), description: "HSI ready".to_string() },
                        ],
                    },
                    RegisterField {
                        name: "HSEON".to_string(),
                        bit_offset: 16,
                        bit_width: 1,
                        access: "rw".to_string(),
                        description: "HSE clock enable".to_string(),
                        values: vec![
                            FieldValue { value: 0, name: "Off".to_string(), description: "HSE disabled".to_string() },
                            FieldValue { value: 1, name: "On".to_string(), description: "HSE enabled".to_string() },
                        ],
                    },
                    RegisterField {
                        name: "PLLON".to_string(),
                        bit_offset: 24,
                        bit_width: 1,
                        access: "rw".to_string(),
                        description: "Main PLL enable".to_string(),
                        values: vec![
                            FieldValue { value: 0, name: "Off".to_string(), description: "PLL disabled".to_string() },
                            FieldValue { value: 1, name: "On".to_string(), description: "PLL enabled".to_string() },
                        ],
                    },
                ],
            },
            Register {
                name: "AHB1ENR".to_string(),
                address: base + 0x30,
                size: 32,
                reset_value: 0x0010_0000,
                description: "AHB1 peripheral clock enable register".to_string(),
                fields: vec![
                    RegisterField {
                        name: "GPIOAEN".to_string(),
                        bit_offset: 0,
                        bit_width: 1,
                        access: "rw".to_string(),
                        description: "IO port A clock enable".to_string(),
                        values: vec![
                            FieldValue { value: 0, name: "Disabled".to_string(), description: "Clock disabled".to_string() },
                            FieldValue { value: 1, name: "Enabled".to_string(), description: "Clock enabled".to_string() },
                        ],
                    },
                    RegisterField {
                        name: "GPIOBEN".to_string(),
                        bit_offset: 1,
                        bit_width: 1,
                        access: "rw".to_string(),
                        description: "IO port B clock enable".to_string(),
                        values: vec![
                            FieldValue { value: 0, name: "Disabled".to_string(), description: "Clock disabled".to_string() },
                            FieldValue { value: 1, name: "Enabled".to_string(), description: "Clock enabled".to_string() },
                        ],
                    },
                ],
            },
        ],
    }
}

/// Get all available peripherals
pub fn get_peripherals() -> Vec<Peripheral> {
    vec![
        get_gpio_registers('A'),
        get_gpio_registers('B'),
        get_gpio_registers('C'),
        get_rcc_registers(),
    ]
}

/// Generate register access code
pub fn generate_register_code(peripheral: &str, reg: &str, operation: &str, value: Option<u32>) -> String {
    match operation {
        "read" => format!(
            "uint32_t value = {}->{};\n// Read: 0x%08lX\\n", 
            peripheral, reg
        ),
        "write" => format!(
            "{}->{}  = 0x{:08X};  // Write value\n",
            peripheral, reg, value.unwrap_or(0)
        ),
        "set_bit" => format!(
            "{}->{}  |= (1U << {});  // Set bit\n",
            peripheral, reg, value.unwrap_or(0)
        ),
        "clear_bit" => format!(
            "{}->{}  &= ~(1U << {});  // Clear bit\n",
            peripheral, reg, value.unwrap_or(0)
        ),
        "toggle_bit" => format!(
            "{}->{}  ^= (1U << {});  // Toggle bit\n",
            peripheral, reg, value.unwrap_or(0)
        ),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_gpio_registers() {
        let gpio = get_gpio_registers('A');
        assert_eq!(gpio.name, "GPIOA");
        assert!(!gpio.registers.is_empty());
    }

    #[test]
    fn test_generate_register_code() {
        let code = generate_register_code("GPIOA", "ODR", "set_bit", Some(13));
        assert!(code.contains("GPIOA->ODR"));
        assert!(code.contains("13"));
    }
}
