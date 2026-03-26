//! CH32 MCU Family Support
//!
//! WCH's CH32 series - RISC-V microcontrollers

use serde::{Deserialize, Serialize};

/// CH32 MCU variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ch32Variant {
    // RISC-V (RISC-V4A core)
    CH32V003,   // Ultra low-cost
    CH32V103,   // General purpose
    CH32V203,   // High performance
    CH32V303,   // With USB
    CH32V307,   // Ethernet + USB
}

impl Ch32Variant {
    pub fn display_name(&self) -> &'static str {
        match self {
            Ch32Variant::CH32V003 => "CH32V003 (RISC-V, Ultra Low-Cost)",
            Ch32Variant::CH32V103 => "CH32V103 (RISC-V)",
            Ch32Variant::CH32V203 => "CH32V203 (RISC-V, USB)",
            Ch32Variant::CH32V303 => "CH32V303 (RISC-V, USB)",
            Ch32Variant::CH32V307 => "CH32V307 (RISC-V, USB+ETH)",
        }
    }

    pub fn architecture(&self) -> &'static str {
        "RISC-V"
    }

    pub fn max_mhz(&self) -> u32 {
        match self {
            Ch32Variant::CH32V003 => 48,
            Ch32Variant::CH32V103 => 80,
            Ch32Variant::CH32V203 => 144,
            Ch32Variant::CH32V303 => 144,
            Ch32Variant::CH32V307 => 144,
        }
    }

    pub fn flash_kb(&self) -> u32 {
        match self {
            Ch32Variant::CH32V003 => 16,
            Ch32Variant::CH32V103 => 64,
            Ch32Variant::CH32V203 => 256,
            Ch32Variant::CH32V303 => 256,
            Ch32Variant::CH32V307 => 256,
        }
    }

    pub fn ram_kb(&self) -> u32 {
        match self {
            Ch32Variant::CH32V003 => 2,
            Ch32Variant::CH32V103 => 20,
            Ch32Variant::CH32V203 => 64,
            Ch32Variant::CH32V303 => 64,
            Ch32Variant::CH32V307 => 64,
        }
    }
    
    pub fn has_usb(&self) -> bool {
        match self {
            Ch32Variant::CH32V003 | Ch32Variant::CH32V103 => false,
            _ => true,
        }
    }
    
    pub fn has_ethernet(&self) -> bool {
        matches!(self, Ch32Variant::CH32V307)
    }
    
    pub fn package_pins(&self) -> &'static [u8] {
        match self {
            Ch32Variant::CH32V003 => &[8, 10, 16, 20],
            Ch32Variant::CH32V103 => &[32, 48, 64],
            Ch32Variant::CH32V203 => &[32, 48, 64],
            Ch32Variant::CH32V303 => &[48, 64, 100],
            Ch32Variant::CH32V307 => &[64, 100],
        }
    }
}

/// Get all supported CH32 variants
pub fn get_ch32_variants() -> Vec<Ch32Variant> {
    vec![
        Ch32Variant::CH32V003,
        Ch32Variant::CH32V103,
        Ch32Variant::CH32V203,
        Ch32Variant::CH32V303,
        Ch32Variant::CH32V307,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ch32_variants() {
        let variants = get_ch32_variants();
        assert_eq!(variants.len(), 5);
        
        for v in variants {
            assert_eq!(v.architecture(), "RISC-V");
        }
    }
}
