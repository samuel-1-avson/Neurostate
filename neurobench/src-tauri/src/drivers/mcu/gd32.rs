//! GD32 MCU Family Support
//!
//! GigaDevice's GD32 series - ARM Cortex-M and RISC-V variants
//! Compatible with STM32 peripherals in many cases

use serde::{Deserialize, Serialize};

/// GD32 MCU variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gd32Variant {
    // ARM Cortex-M3
    GD32F103,   // STM32F103 compatible
    GD32F303,   // High performance
    // ARM Cortex-M4
    GD32F403,   // STM32F4 compatible
    GD32F450,   // High performance
    // RISC-V
    GD32VF103,  // RISC-V 32-bit
}

impl Gd32Variant {
    pub fn display_name(&self) -> &'static str {
        match self {
            Gd32Variant::GD32F103 => "GD32F103 (Cortex-M3)",
            Gd32Variant::GD32F303 => "GD32F303 (Cortex-M3)",
            Gd32Variant::GD32F403 => "GD32F403 (Cortex-M4)",
            Gd32Variant::GD32F450 => "GD32F450 (Cortex-M4)",
            Gd32Variant::GD32VF103 => "GD32VF103 (RISC-V)",
        }
    }

    pub fn architecture(&self) -> &'static str {
        match self {
            Gd32Variant::GD32VF103 => "RISC-V",
            _ => "ARM Cortex-M",
        }
    }

    pub fn max_mhz(&self) -> u32 {
        match self {
            Gd32Variant::GD32F103 => 108,
            Gd32Variant::GD32F303 => 120,
            Gd32Variant::GD32F403 => 168,
            Gd32Variant::GD32F450 => 200,
            Gd32Variant::GD32VF103 => 108,
        }
    }

    pub fn flash_kb(&self) -> u32 {
        match self {
            Gd32Variant::GD32F103 => 512,
            Gd32Variant::GD32F303 => 512,
            Gd32Variant::GD32F403 => 1024,
            Gd32Variant::GD32F450 => 3072,
            Gd32Variant::GD32VF103 => 128,
        }
    }

    pub fn ram_kb(&self) -> u32 {
        match self {
            Gd32Variant::GD32F103 => 96,
            Gd32Variant::GD32F303 => 96,
            Gd32Variant::GD32F403 => 256,
            Gd32Variant::GD32F450 => 256,
            Gd32Variant::GD32VF103 => 32,
        }
    }
    
    pub fn has_usb(&self) -> bool {
        true
    }
    
    pub fn has_can(&self) -> bool {
        true
    }
}

/// GD32 peripheral availability
pub struct Gd32Peripherals {
    pub gpio_ports: u8,
    pub uart_count: u8,
    pub spi_count: u8,
    pub i2c_count: u8,
    pub timer_count: u8,
    pub adc_count: u8,
    pub dac_count: u8,
}

impl Gd32Variant {
    pub fn peripherals(&self) -> Gd32Peripherals {
        match self {
            Gd32Variant::GD32F103 | Gd32Variant::GD32VF103 => Gd32Peripherals {
                gpio_ports: 5,
                uart_count: 5,
                spi_count: 3,
                i2c_count: 2,
                timer_count: 8,
                adc_count: 3,
                dac_count: 2,
            },
            Gd32Variant::GD32F303 => Gd32Peripherals {
                gpio_ports: 6,
                uart_count: 5,
                spi_count: 4,
                i2c_count: 3,
                timer_count: 14,
                adc_count: 4,
                dac_count: 2,
            },
            Gd32Variant::GD32F403 | Gd32Variant::GD32F450 => Gd32Peripherals {
                gpio_ports: 9,
                uart_count: 8,
                spi_count: 6,
                i2c_count: 3,
                timer_count: 17,
                adc_count: 3,
                dac_count: 2,
            },
        }
    }
}

/// Get all supported GD32 variants
pub fn get_gd32_variants() -> Vec<Gd32Variant> {
    vec![
        Gd32Variant::GD32F103,
        Gd32Variant::GD32F303,
        Gd32Variant::GD32F403,
        Gd32Variant::GD32F450,
        Gd32Variant::GD32VF103,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gd32_variants() {
        let variants = get_gd32_variants();
        assert_eq!(variants.len(), 5);
    }

    #[test]
    fn test_gd32vf_riscv() {
        let v = Gd32Variant::GD32VF103;
        assert_eq!(v.architecture(), "RISC-V");
    }
}
