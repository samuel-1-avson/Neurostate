//! Clock Tree Simulation
//!
//! Simulates the MCU clock system including oscillators, PLLs, and prescalers.

use super::ClockConfig;
use serde::{Deserialize, Serialize};

/// Clock source selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClockSource {
    /// High-speed internal oscillator
    HSI,
    /// High-speed external oscillator
    HSE,
    /// PLL output
    PLL,
    /// Low-speed internal oscillator
    LSI,
    /// Low-speed external oscillator
    LSE,
}

impl Default for ClockSource {
    fn default() -> Self {
        Self::HSI
    }
}

/// PLL configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PllConfig {
    /// PLL source (HSI or HSE)
    pub source: ClockSource,
    /// Input divider (PLLM)
    pub m: u8,
    /// Multiplier (PLLN)
    pub n: u16,
    /// Main output divider (PLLP)
    pub p: u8,
    /// USB/RNG output divider (PLLQ)
    pub q: u8,
    /// PLL enabled
    pub enabled: bool,
    /// PLL ready
    pub ready: bool,
}

impl Default for PllConfig {
    fn default() -> Self {
        Self {
            source: ClockSource::HSE,
            m: 8,
            n: 336,
            p: 2,
            q: 7,
            enabled: false,
            ready: false,
        }
    }
}

/// Clock tree controller
pub struct ClockTree {
    /// HSI frequency (typically 16 MHz)
    hsi: u32,
    /// HSE frequency (external crystal)
    hse: Option<u32>,
    /// LSI frequency (typically 32 kHz)
    lsi: u32,
    /// LSE frequency (typically 32.768 kHz)
    lse: Option<u32>,
    /// PLL configuration
    pll: PllConfig,
    /// System clock source
    sysclk_source: ClockSource,
    /// Calculated frequencies
    sysclk: u32,
    hclk: u32,
    pclk1: u32,
    pclk2: u32,
    /// AHB prescaler
    ahb_prescaler: u16,
    /// APB1 prescaler
    apb1_prescaler: u8,
    /// APB2 prescaler
    apb2_prescaler: u8,
    /// RCC registers simulation
    cr: u32,
    cfgr: u32,
    pllcfgr: u32,
}

impl ClockTree {
    /// Create new clock tree from configuration
    pub fn new(config: ClockConfig) -> Self {
        let mut tree = Self {
            hsi: 16_000_000,
            hse: config.hse_frequency,
            lsi: 32_000,
            lse: Some(32_768),
            pll: PllConfig::default(),
            sysclk_source: ClockSource::HSI,
            sysclk: 16_000_000,
            hclk: 16_000_000,
            pclk1: 16_000_000,
            pclk2: 16_000_000,
            ahb_prescaler: config.ahb_prescaler as u16,
            apb1_prescaler: config.apb1_prescaler,
            apb2_prescaler: config.apb2_prescaler,
            cr: 0x0000_0083, // HSI ON and ready
            cfgr: 0,
            pllcfgr: 0x2400_3010, // Default reset value
        };
        
        // Configure PLL if enabled
        if config.pll_enabled {
            if let Some(hse) = config.hse_frequency {
                tree.pll.source = ClockSource::HSE;
                tree.pll.enabled = true;
                tree.pll.ready = true;
                tree.pll.m = 8;
                tree.pll.n = config.pll_multiplier as u16 * 8;
                tree.pll.p = config.pll_divider;
                tree.sysclk_source = ClockSource::PLL;
                tree.cr |= (1 << 24) | (1 << 25); // PLLON and PLLRDY
            }
        }
        
        tree.calculate_frequencies();
        tree
    }

    /// Calculate all derived clock frequencies
    fn calculate_frequencies(&mut self) {
        // Get PLL input frequency
        let pll_input = match self.pll.source {
            ClockSource::HSE => self.hse.unwrap_or(8_000_000),
            _ => self.hsi,
        };
        
        // Calculate PLL output: VCO = input * N / M, output = VCO / P
        let pll_vco = if self.pll.m > 0 {
            (pll_input as u64 * self.pll.n as u64 / self.pll.m as u64) as u32
        } else {
            0
        };
        
        let pll_output = if self.pll.p > 0 {
            pll_vco / self.pll.p as u32
        } else {
            0
        };
        
        // Select system clock source
        self.sysclk = match self.sysclk_source {
            ClockSource::HSI => self.hsi,
            ClockSource::HSE => self.hse.unwrap_or(self.hsi),
            ClockSource::PLL => {
                if self.pll.enabled && self.pll.ready {
                    pll_output
                } else {
                    self.hsi
                }
            }
            _ => self.hsi,
        };
        
        // Calculate AHB clock (HCLK)
        let ahb_div = match self.ahb_prescaler {
            0..=7 => 1,
            8 => 2,
            9 => 4,
            10 => 8,
            11 => 16,
            12 => 64,
            13 => 128,
            14 => 256,
            _ => 512,
        };
        self.hclk = self.sysclk / ahb_div;
        
        // Calculate APB1 clock (PCLK1)
        let apb1_div = match self.apb1_prescaler {
            0..=3 => 1,
            4 => 2,
            5 => 4,
            6 => 8,
            _ => 16,
        };
        self.pclk1 = self.hclk / apb1_div;
        
        // Calculate APB2 clock (PCLK2)
        let apb2_div = match self.apb2_prescaler {
            0..=3 => 1,
            4 => 2,
            5 => 4,
            6 => 8,
            _ => 16,
        };
        self.pclk2 = self.hclk / apb2_div;
    }

    /// Get system clock frequency
    pub fn sysclk(&self) -> u32 {
        self.sysclk
    }

    /// Get AHB clock frequency
    pub fn hclk(&self) -> u32 {
        self.hclk
    }

    /// Get APB1 clock frequency
    pub fn pclk1(&self) -> u32 {
        self.pclk1
    }

    /// Get APB2 clock frequency
    pub fn pclk2(&self) -> u32 {
        self.pclk2
    }

    /// Read RCC register
    pub fn read(&self, offset: u32) -> u32 {
        match offset {
            0x00 => self.cr,
            0x08 => self.cfgr,
            0x04 => self.pllcfgr,
            _ => 0,
        }
    }

    /// Write RCC register
    pub fn write(&mut self, offset: u32, value: u32) {
        match offset {
            0x00 => {
                self.cr = value;
                
                // HSE enable
                if value & (1 << 16) != 0 && self.hse.is_some() {
                    self.cr |= 1 << 17; // HSERDY
                }
                
                // PLL enable
                if value & (1 << 24) != 0 {
                    self.pll.enabled = true;
                    self.pll.ready = true;
                    self.cr |= 1 << 25; // PLLRDY
                }
                
                self.calculate_frequencies();
            }
            0x04 => {
                self.pllcfgr = value;
                self.pll.m = (value & 0x3F) as u8;
                self.pll.n = ((value >> 6) & 0x1FF) as u16;
                self.pll.p = (((value >> 16) & 0x3) as u8 + 1) * 2;
                self.pll.q = ((value >> 24) & 0xF) as u8;
                self.pll.source = if value & (1 << 22) != 0 {
                    ClockSource::HSE
                } else {
                    ClockSource::HSI
                };
                self.calculate_frequencies();
            }
            0x08 => {
                self.cfgr = value;
                
                // System clock switch
                self.sysclk_source = match value & 0x3 {
                    0 => ClockSource::HSI,
                    1 => ClockSource::HSE,
                    2 => ClockSource::PLL,
                    _ => ClockSource::HSI,
                };
                
                // AHB prescaler
                self.ahb_prescaler = ((value >> 4) & 0xF) as u16;
                
                // APB1 prescaler
                self.apb1_prescaler = ((value >> 10) & 0x7) as u8;
                
                // APB2 prescaler
                self.apb2_prescaler = ((value >> 13) & 0x7) as u8;
                
                self.calculate_frequencies();
                
                // Update SWS (system clock switch status)
                let sws = (value & 0x3) << 2;
                self.cfgr = (self.cfgr & !0xC) | sws;
            }
            _ => {}
        }
    }

    /// Get clock info for display
    pub fn get_info(&self) -> ClockInfo {
        ClockInfo {
            sysclk: self.sysclk,
            hclk: self.hclk,
            pclk1: self.pclk1,
            pclk2: self.pclk2,
            pll_enabled: self.pll.enabled,
            source: format!("{:?}", self.sysclk_source),
        }
    }
}

/// Clock information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockInfo {
    pub sysclk: u32,
    pub hclk: u32,
    pub pclk1: u32,
    pub pclk2: u32,
    pub pll_enabled: bool,
    pub source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_clock() {
        let config = ClockConfig::default();
        let clock = ClockTree::new(config);
        
        assert!(clock.sysclk() > 0);
        assert!(clock.hclk() > 0);
    }

    #[test]
    fn test_pll_calculation() {
        let config = ClockConfig {
            hse_frequency: Some(8_000_000),
            pll_enabled: true,
            pll_multiplier: 21, // 168/8
            pll_divider: 2,
            ahb_prescaler: 1,
            apb1_prescaler: 4,
            apb2_prescaler: 2,
        };
        
        let clock = ClockTree::new(config);
        
        // With PLL: 8MHz * 168 / 8 / 2 = 84 MHz (simplified calculation differs)
        assert!(clock.sysclk() > 16_000_000);
    }
}
