//! Texas Instruments MCU Family Support
//!
//! TI MSP430 and MSP432 series

use serde::{Deserialize, Serialize};

/// TI MCU variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TiVariant {
    // MSP430 (16-bit ultra-low-power)
    MSP430G2553,    // LaunchPad
    MSP430FR5969,   // FRAM
    MSP430FR2355,   // FRAM value line
    // MSP432 (ARM Cortex-M4F)
    MSP432P401R,    // High performance
    MSP432E401Y,    // Ethernet
    // CC series (wireless)
    CC3220SF,       // WiFi
    CC2652R,        // Thread/Zigbee
    CC1352R,        // Sub-1GHz + BLE
}

impl TiVariant {
    pub fn display_name(&self) -> &'static str {
        match self {
            TiVariant::MSP430G2553 => "MSP430G2553 (LaunchPad)",
            TiVariant::MSP430FR5969 => "MSP430FR5969 (FRAM)",
            TiVariant::MSP430FR2355 => "MSP430FR2355 (FRAM)",
            TiVariant::MSP432P401R => "MSP432P401R (Cortex-M4F)",
            TiVariant::MSP432E401Y => "MSP432E401Y (Ethernet)",
            TiVariant::CC3220SF => "CC3220SF (WiFi)",
            TiVariant::CC2652R => "CC2652R (Thread/Zigbee)",
            TiVariant::CC1352R => "CC1352R (Sub-1GHz + BLE)",
        }
    }

    pub fn architecture(&self) -> &'static str {
        match self {
            TiVariant::MSP430G2553 | TiVariant::MSP430FR5969 | 
            TiVariant::MSP430FR2355 => "MSP430",
            _ => "ARM Cortex-M4F",
        }
    }

    pub fn max_mhz(&self) -> u32 {
        match self {
            TiVariant::MSP430G2553 => 16,
            TiVariant::MSP430FR5969 | TiVariant::MSP430FR2355 => 16,
            TiVariant::MSP432P401R => 48,
            TiVariant::MSP432E401Y => 120,
            TiVariant::CC3220SF | TiVariant::CC2652R | TiVariant::CC1352R => 80,
        }
    }

    pub fn flash_kb(&self) -> u32 {
        match self {
            TiVariant::MSP430G2553 => 16,
            TiVariant::MSP430FR5969 => 64,  // FRAM
            TiVariant::MSP430FR2355 => 32,  // FRAM
            TiVariant::MSP432P401R => 256,
            TiVariant::MSP432E401Y => 1024,
            TiVariant::CC3220SF => 1024,
            TiVariant::CC2652R | TiVariant::CC1352R => 352,
        }
    }

    pub fn ram_kb(&self) -> u32 {
        match self {
            TiVariant::MSP430G2553 => 1, // 512 bytes
            TiVariant::MSP430FR5969 | TiVariant::MSP430FR2355 => 2,
            TiVariant::MSP432P401R => 64,
            TiVariant::MSP432E401Y => 256,
            TiVariant::CC3220SF => 256,
            TiVariant::CC2652R | TiVariant::CC1352R => 80,
        }
    }
    
    pub fn has_fram(&self) -> bool {
        matches!(self, TiVariant::MSP430FR5969 | TiVariant::MSP430FR2355)
    }
    
    pub fn has_wifi(&self) -> bool {
        matches!(self, TiVariant::CC3220SF)
    }
    
    pub fn has_ble(&self) -> bool {
        matches!(self, TiVariant::CC2652R | TiVariant::CC1352R)
    }
    
    pub fn has_sub_ghz(&self) -> bool {
        matches!(self, TiVariant::CC1352R)
    }
}

/// Get all supported TI variants
pub fn get_ti_variants() -> Vec<TiVariant> {
    vec![
        TiVariant::MSP430G2553,
        TiVariant::MSP430FR5969,
        TiVariant::MSP430FR2355,
        TiVariant::MSP432P401R,
        TiVariant::MSP432E401Y,
        TiVariant::CC3220SF,
        TiVariant::CC2652R,
        TiVariant::CC1352R,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ti_variants() {
        let variants = get_ti_variants();
        assert_eq!(variants.len(), 8);
    }

    #[test]
    fn test_msp430_arch() {
        let v = TiVariant::MSP430G2553;
        assert_eq!(v.architecture(), "MSP430");
    }
}
