//! Renesas MCU Family Support
//!
//! Renesas RA, RX, and RL78 series

use serde::{Deserialize, Serialize};

/// Renesas MCU variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenesasVariant {
    // RA Series (ARM Cortex-M)
    RA2A1,      // Entry level
    RA4M1,      // General purpose
    RA6M3,      // High performance
    RA6M5,      // Security-focused
    // RX Series (Renesas proprietary)
    RX65N,      // Graphics
    RX72N,      // High performance
    // RL78 Series (16-bit)
    RL78G14,    // General purpose
}

impl RenesasVariant {
    pub fn display_name(&self) -> &'static str {
        match self {
            RenesasVariant::RA2A1 => "RA2A1 (Cortex-M23)",
            RenesasVariant::RA4M1 => "RA4M1 (Cortex-M4)",
            RenesasVariant::RA6M3 => "RA6M3 (Cortex-M4)",
            RenesasVariant::RA6M5 => "RA6M5 (Cortex-M33)",
            RenesasVariant::RX65N => "RX65N (RXv2)",
            RenesasVariant::RX72N => "RX72N (RXv3)",
            RenesasVariant::RL78G14 => "RL78/G14 (16-bit)",
        }
    }

    pub fn architecture(&self) -> &'static str {
        match self {
            RenesasVariant::RA2A1 => "ARM Cortex-M23",
            RenesasVariant::RA4M1 | RenesasVariant::RA6M3 => "ARM Cortex-M4",
            RenesasVariant::RA6M5 => "ARM Cortex-M33",
            RenesasVariant::RX65N | RenesasVariant::RX72N => "RX",
            RenesasVariant::RL78G14 => "RL78",
        }
    }

    pub fn max_mhz(&self) -> u32 {
        match self {
            RenesasVariant::RA2A1 => 48,
            RenesasVariant::RA4M1 => 100,
            RenesasVariant::RA6M3 => 120,
            RenesasVariant::RA6M5 => 200,
            RenesasVariant::RX65N => 120,
            RenesasVariant::RX72N => 240,
            RenesasVariant::RL78G14 => 32,
        }
    }

    pub fn flash_kb(&self) -> u32 {
        match self {
            RenesasVariant::RA2A1 => 256,
            RenesasVariant::RA4M1 => 512,
            RenesasVariant::RA6M3 => 2048,
            RenesasVariant::RA6M5 => 2048,
            RenesasVariant::RX65N => 2048,
            RenesasVariant::RX72N => 4096,
            RenesasVariant::RL78G14 => 512,
        }
    }

    pub fn ram_kb(&self) -> u32 {
        match self {
            RenesasVariant::RA2A1 => 32,
            RenesasVariant::RA4M1 => 128,
            RenesasVariant::RA6M3 => 640,
            RenesasVariant::RA6M5 => 512,
            RenesasVariant::RX65N => 640,
            RenesasVariant::RX72N => 1024,
            RenesasVariant::RL78G14 => 48,
        }
    }
    
    pub fn has_trustzone(&self) -> bool {
        matches!(self, RenesasVariant::RA6M5)
    }
    
    pub fn has_graphics(&self) -> bool {
        matches!(self, RenesasVariant::RX65N | RenesasVariant::RX72N | RenesasVariant::RA6M3)
    }
}

/// Get all supported Renesas variants
pub fn get_renesas_variants() -> Vec<RenesasVariant> {
    vec![
        RenesasVariant::RA2A1,
        RenesasVariant::RA4M1,
        RenesasVariant::RA6M3,
        RenesasVariant::RA6M5,
        RenesasVariant::RX65N,
        RenesasVariant::RX72N,
        RenesasVariant::RL78G14,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renesas_variants() {
        let variants = get_renesas_variants();
        assert_eq!(variants.len(), 7);
    }
}
