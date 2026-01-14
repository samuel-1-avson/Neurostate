//! PIC MCU Family Support
//!
//! Microchip PIC 8-bit and 16-bit microcontrollers

use serde::{Deserialize, Serialize};

/// PIC MCU variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PicVariant {
    // PIC16 Series (8-bit)
    PIC16F877A,     // Classic
    PIC16F1619,     // Enhanced mid-range
    PIC16F18877,    // Modern
    // PIC18 Series (8-bit enhanced)
    PIC18F4550,     // USB
    PIC18F46K22,    // General purpose
    PIC18F57Q43,    // Modern Q-series
    // PIC24 Series (16-bit)
    PIC24FJ256GB110,// USB
    PIC24FJ1024GB610,// High memory
    // dsPIC (16-bit DSP)
    DSPIC33EP512GP502,// Motor control
    DSPIC33CK256MP508,// High performance
}

impl PicVariant {
    pub fn display_name(&self) -> &'static str {
        match self {
            PicVariant::PIC16F877A => "PIC16F877A (Classic)",
            PicVariant::PIC16F1619 => "PIC16F1619 (Enhanced)",
            PicVariant::PIC16F18877 => "PIC16F18877 (Modern)",
            PicVariant::PIC18F4550 => "PIC18F4550 (USB)",
            PicVariant::PIC18F46K22 => "PIC18F46K22",
            PicVariant::PIC18F57Q43 => "PIC18F57Q43 (Q-series)",
            PicVariant::PIC24FJ256GB110 => "PIC24FJ256GB110 (USB)",
            PicVariant::PIC24FJ1024GB610 => "PIC24FJ1024GB610",
            PicVariant::DSPIC33EP512GP502 => "dsPIC33EP512GP502",
            PicVariant::DSPIC33CK256MP508 => "dsPIC33CK256MP508",
        }
    }

    pub fn architecture(&self) -> &'static str {
        match self {
            PicVariant::PIC16F877A | PicVariant::PIC16F1619 | 
            PicVariant::PIC16F18877 => "PIC16 (8-bit)",
            PicVariant::PIC18F4550 | PicVariant::PIC18F46K22 | 
            PicVariant::PIC18F57Q43 => "PIC18 (8-bit)",
            PicVariant::PIC24FJ256GB110 | PicVariant::PIC24FJ1024GB610 => "PIC24 (16-bit)",
            PicVariant::DSPIC33EP512GP502 | PicVariant::DSPIC33CK256MP508 => "dsPIC (16-bit DSP)",
        }
    }

    pub fn max_mhz(&self) -> u32 {
        match self {
            PicVariant::PIC16F877A => 20,
            PicVariant::PIC16F1619 | PicVariant::PIC16F18877 => 32,
            PicVariant::PIC18F4550 | PicVariant::PIC18F46K22 => 48,
            PicVariant::PIC18F57Q43 => 64,
            PicVariant::PIC24FJ256GB110 | PicVariant::PIC24FJ1024GB610 => 32,
            PicVariant::DSPIC33EP512GP502 => 70,
            PicVariant::DSPIC33CK256MP508 => 100,
        }
    }

    pub fn flash_kb(&self) -> u32 {
        match self {
            PicVariant::PIC16F877A => 14,
            PicVariant::PIC16F1619 => 14,
            PicVariant::PIC16F18877 => 56,
            PicVariant::PIC18F4550 => 32,
            PicVariant::PIC18F46K22 => 64,
            PicVariant::PIC18F57Q43 => 128,
            PicVariant::PIC24FJ256GB110 => 256,
            PicVariant::PIC24FJ1024GB610 => 1024,
            PicVariant::DSPIC33EP512GP502 => 512,
            PicVariant::DSPIC33CK256MP508 => 256,
        }
    }

    pub fn ram_bytes(&self) -> u32 {
        match self {
            PicVariant::PIC16F877A => 368,
            PicVariant::PIC16F1619 => 1024,
            PicVariant::PIC16F18877 => 4096,
            PicVariant::PIC18F4550 => 2048,
            PicVariant::PIC18F46K22 => 3896,
            PicVariant::PIC18F57Q43 => 8192,
            PicVariant::PIC24FJ256GB110 => 16384,
            PicVariant::PIC24FJ1024GB610 => 32768,
            PicVariant::DSPIC33EP512GP502 => 49152,
            PicVariant::DSPIC33CK256MP508 => 24576,
        }
    }

    pub fn has_usb(&self) -> bool {
        matches!(self, 
            PicVariant::PIC18F4550 | 
            PicVariant::PIC24FJ256GB110 | 
            PicVariant::PIC24FJ1024GB610
        )
    }
    
    pub fn has_dsp(&self) -> bool {
        matches!(self, 
            PicVariant::DSPIC33EP512GP502 | 
            PicVariant::DSPIC33CK256MP508
        )
    }
}

/// Get all supported PIC variants
pub fn get_pic_variants() -> Vec<PicVariant> {
    vec![
        PicVariant::PIC16F877A,
        PicVariant::PIC16F1619,
        PicVariant::PIC16F18877,
        PicVariant::PIC18F4550,
        PicVariant::PIC18F46K22,
        PicVariant::PIC18F57Q43,
        PicVariant::PIC24FJ256GB110,
        PicVariant::PIC24FJ1024GB610,
        PicVariant::DSPIC33EP512GP502,
        PicVariant::DSPIC33CK256MP508,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pic_variants() {
        let variants = get_pic_variants();
        assert_eq!(variants.len(), 10);
    }

    #[test]
    fn test_dspic_has_dsp() {
        assert!(PicVariant::DSPIC33EP512GP502.has_dsp());
        assert!(!PicVariant::PIC16F877A.has_dsp());
    }
}
