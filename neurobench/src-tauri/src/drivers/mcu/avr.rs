//! AVR MCU Family Support
//!
//! Microchip/Atmel AVR 8-bit microcontrollers

use serde::{Deserialize, Serialize};

/// AVR MCU variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AvrVariant {
    // ATmega series
    ATmega328P,     // Arduino Uno
    ATmega2560,     // Arduino Mega
    ATmega32U4,     // Arduino Leonardo (USB)
    ATmega1284P,    // High memory
    // ATtiny series
    ATtiny85,       // Small 8-pin
    ATtiny1614,     // Modern ATtiny
    ATtiny3227,     // Latest ATtiny
    // AVR DA/DB series
    AVR128DA48,     // Modern AVR
    AVR128DB48,     // With MVIO
}

impl AvrVariant {
    pub fn display_name(&self) -> &'static str {
        match self {
            AvrVariant::ATmega328P => "ATmega328P (Arduino Uno)",
            AvrVariant::ATmega2560 => "ATmega2560 (Arduino Mega)",
            AvrVariant::ATmega32U4 => "ATmega32U4 (USB)",
            AvrVariant::ATmega1284P => "ATmega1284P",
            AvrVariant::ATtiny85 => "ATtiny85 (8-pin)",
            AvrVariant::ATtiny1614 => "ATtiny1614",
            AvrVariant::ATtiny3227 => "ATtiny3227",
            AvrVariant::AVR128DA48 => "AVR128DA48",
            AvrVariant::AVR128DB48 => "AVR128DB48 (MVIO)",
        }
    }

    pub fn architecture(&self) -> &'static str {
        match self {
            AvrVariant::AVR128DA48 | AvrVariant::AVR128DB48 => "AVR Dx",
            AvrVariant::ATtiny1614 | AvrVariant::ATtiny3227 => "AVR-0/1",
            _ => "AVR",
        }
    }

    pub fn max_mhz(&self) -> u32 {
        match self {
            AvrVariant::ATmega328P | AvrVariant::ATmega2560 | 
            AvrVariant::ATmega32U4 | AvrVariant::ATmega1284P => 20,
            AvrVariant::ATtiny85 => 20,
            AvrVariant::ATtiny1614 | AvrVariant::ATtiny3227 => 20,
            AvrVariant::AVR128DA48 | AvrVariant::AVR128DB48 => 24,
        }
    }

    pub fn flash_kb(&self) -> u32 {
        match self {
            AvrVariant::ATmega328P => 32,
            AvrVariant::ATmega2560 => 256,
            AvrVariant::ATmega32U4 => 32,
            AvrVariant::ATmega1284P => 128,
            AvrVariant::ATtiny85 => 8,
            AvrVariant::ATtiny1614 => 16,
            AvrVariant::ATtiny3227 => 32,
            AvrVariant::AVR128DA48 | AvrVariant::AVR128DB48 => 128,
        }
    }

    pub fn ram_bytes(&self) -> u32 {
        match self {
            AvrVariant::ATmega328P => 2048,
            AvrVariant::ATmega2560 => 8192,
            AvrVariant::ATmega32U4 => 2560,
            AvrVariant::ATmega1284P => 16384,
            AvrVariant::ATtiny85 => 512,
            AvrVariant::ATtiny1614 => 2048,
            AvrVariant::ATtiny3227 => 3072,
            AvrVariant::AVR128DA48 | AvrVariant::AVR128DB48 => 16384,
        }
    }
    
    pub fn eeprom_bytes(&self) -> u32 {
        match self {
            AvrVariant::ATmega328P => 1024,
            AvrVariant::ATmega2560 => 4096,
            AvrVariant::ATmega32U4 => 1024,
            AvrVariant::ATmega1284P => 4096,
            AvrVariant::ATtiny85 => 512,
            AvrVariant::ATtiny1614 => 256,
            AvrVariant::ATtiny3227 => 256,
            AvrVariant::AVR128DA48 | AvrVariant::AVR128DB48 => 512,
        }
    }

    pub fn has_usb(&self) -> bool {
        matches!(self, AvrVariant::ATmega32U4)
    }
    
    pub fn pin_count(&self) -> u8 {
        match self {
            AvrVariant::ATtiny85 => 8,
            AvrVariant::ATtiny1614 => 14,
            AvrVariant::ATtiny3227 => 24,
            AvrVariant::ATmega328P => 28,
            AvrVariant::ATmega32U4 => 44,
            AvrVariant::AVR128DA48 | AvrVariant::AVR128DB48 => 48,
            AvrVariant::ATmega2560 | AvrVariant::ATmega1284P => 100,
        }
    }
}

/// Get all supported AVR variants
pub fn get_avr_variants() -> Vec<AvrVariant> {
    vec![
        AvrVariant::ATmega328P,
        AvrVariant::ATmega2560,
        AvrVariant::ATmega32U4,
        AvrVariant::ATmega1284P,
        AvrVariant::ATtiny85,
        AvrVariant::ATtiny1614,
        AvrVariant::ATtiny3227,
        AvrVariant::AVR128DA48,
        AvrVariant::AVR128DB48,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avr_variants() {
        let variants = get_avr_variants();
        assert_eq!(variants.len(), 9);
    }

    #[test]
    fn test_atmega328p() {
        let v = AvrVariant::ATmega328P;
        assert_eq!(v.flash_kb(), 32);
        assert_eq!(v.ram_bytes(), 2048);
    }
}
