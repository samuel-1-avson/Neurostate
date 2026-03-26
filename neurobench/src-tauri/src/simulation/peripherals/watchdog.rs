//! Watchdog Timer Simulation
//!
//! Simulates Independent Watchdog (IWDG) and Window Watchdog (WWDG).

use super::Peripheral;
use serde::{Deserialize, Serialize};

/// Independent Watchdog (IWDG)
pub struct IwdgPeripheral {
    /// Key register
    kr: u32,
    /// Prescaler register
    pr: u32,
    /// Reload register
    rlr: u32,
    /// Status register
    sr: u32,
    /// Window register
    winr: u32,
    
    /// Counter value
    counter: u32,
    /// Enabled flag
    enabled: bool,
    /// Cycles per decrement
    cycles_per_tick: u64,
    /// Accumulated cycles
    accumulated_cycles: u64,
    /// Reset triggered
    reset_pending: bool,
}

impl IwdgPeripheral {
    pub fn new(lsi_clock: u32) -> Self {
        // Default: /4 prescaler, 4096 reload = ~1 second timeout at 32kHz LSI
        let default_prescaler = 4;
        let cycles_per_tick = (lsi_clock / default_prescaler) as u64;
        
        Self {
            kr: 0,
            pr: 0, // /4
            rlr: 0x0FFF, // 4095
            sr: 0,
            winr: 0x0FFF,
            counter: 0x0FFF,
            enabled: false,
            cycles_per_tick,
            accumulated_cycles: 0,
            reset_pending: false,
        }
    }

    /// Check if reset was triggered
    pub fn is_reset_pending(&self) -> bool {
        self.reset_pending
    }

    /// Clear reset flag
    pub fn clear_reset(&mut self) {
        self.reset_pending = false;
    }

    fn update_prescaler(&mut self, lsi_clock: u32) {
        let div = match self.pr & 0x7 {
            0 => 4,
            1 => 8,
            2 => 16,
            3 => 32,
            4 => 64,
            5 => 128,
            6 => 256,
            7 => 256,
            _ => 4,
        };
        self.cycles_per_tick = (lsi_clock as u64) / div;
    }
}

impl Peripheral for IwdgPeripheral {
    fn name(&self) -> &str { "IWDG" }

    fn read(&self, offset: u32) -> u32 {
        match offset {
            0x00 => self.kr,
            0x04 => self.pr,
            0x08 => self.rlr,
            0x0C => self.sr,
            0x10 => self.winr,
            _ => 0,
        }
    }

    fn write(&mut self, offset: u32, value: u32) {
        match offset {
            0x00 => {
                self.kr = value;
                match value {
                    0xCCCC => {
                        // Start watchdog
                        self.enabled = true;
                        self.counter = self.rlr;
                    }
                    0xAAAA => {
                        // Reload (feed the dog)
                        if self.enabled {
                            self.counter = self.rlr;
                        }
                    }
                    0x5555 => {
                        // Enable write access to PR, RLR, WINR
                        self.sr |= 1 << 0; // PVU
                    }
                    _ => {}
                }
            }
            0x04 => {
                self.pr = value & 0x7;
                self.update_prescaler(32_000); // Assuming 32kHz LSI
            }
            0x08 => {
                self.rlr = value & 0x0FFF;
            }
            0x10 => {
                self.winr = value & 0x0FFF;
            }
            _ => {}
        }
    }

    fn tick(&mut self, cycles: u64) {
        if !self.enabled {
            return;
        }

        self.accumulated_cycles += cycles;
        
        while self.accumulated_cycles >= self.cycles_per_tick {
            self.accumulated_cycles -= self.cycles_per_tick;
            
            if self.counter > 0 {
                self.counter -= 1;
            } else {
                // Watchdog timeout - trigger reset!
                self.reset_pending = true;
            }
        }
    }

    fn get_interrupt(&self) -> Option<u8> { None }
    fn clear_interrupt(&mut self) {}

    fn reset(&mut self) {
        // Note: IWDG cannot be disabled once started (hardware behavior)
        self.counter = self.rlr;
        self.reset_pending = false;
    }

    fn get_state(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "counter": self.counter,
            "reload": self.rlr,
            "prescaler_div": match self.pr { 0 => 4, 1 => 8, 2 => 16, 3 => 32, 4 => 64, 5 => 128, _ => 256 },
        })
    }
}

/// Window Watchdog (WWDG)
pub struct WwdgPeripheral {
    /// Control register
    cr: u32,
    /// Configuration register
    cfr: u32,
    /// Status register
    sr: u32,
    
    /// Counter
    counter: u8,
    /// Enabled
    enabled: bool,
    /// Cycles per decrement
    cycles_per_tick: u64,
    accumulated_cycles: u64,
    /// Early wakeup interrupt pending
    ewi_pending: bool,
    /// Reset pending
    reset_pending: bool,
}

impl WwdgPeripheral {
    pub fn new(pclk1: u32) -> Self {
        // WWDG clock = PCLK1 / 4096 / prescaler
        let cycles_per_tick = (pclk1 as u64) / 4096;
        
        Self {
            cr: 0x7F,
            cfr: 0x7F,
            sr: 0,
            counter: 0x7F,
            enabled: false,
            cycles_per_tick,
            accumulated_cycles: 0,
            ewi_pending: false,
            reset_pending: false,
        }
    }

    pub fn is_reset_pending(&self) -> bool {
        self.reset_pending
    }
}

impl Peripheral for WwdgPeripheral {
    fn name(&self) -> &str { "WWDG" }

    fn read(&self, offset: u32) -> u32 {
        match offset {
            0x00 => self.cr,
            0x04 => self.cfr,
            0x08 => self.sr,
            _ => 0,
        }
    }

    fn write(&mut self, offset: u32, value: u32) {
        match offset {
            0x00 => {
                self.cr = value & 0xFF;
                self.counter = (value & 0x7F) as u8;
                if value & 0x80 != 0 {
                    self.enabled = true;
                }
            }
            0x04 => {
                self.cfr = value & 0x3FF;
            }
            0x08 => {
                // Write 0 to clear EWIF
                if value & 1 == 0 {
                    self.sr &= !1;
                }
            }
            _ => {}
        }
    }

    fn tick(&mut self, cycles: u64) {
        if !self.enabled {
            return;
        }

        let prescaler = 1 << ((self.cfr >> 7) & 0x3);
        let adjusted_cycles = self.cycles_per_tick * prescaler as u64;
        
        self.accumulated_cycles += cycles;
        
        while self.accumulated_cycles >= adjusted_cycles {
            self.accumulated_cycles -= adjusted_cycles;
            
            if self.counter > 0x3F {
                self.counter -= 1;
                
                // Check for early wakeup (counter reaches window value)
                if self.counter == 0x40 && (self.cfr & (1 << 9)) != 0 {
                    self.ewi_pending = true;
                    self.sr |= 1; // EWIF
                }
            } else {
                // Counter went below 0x40 - reset!
                self.reset_pending = true;
            }
        }
    }

    fn get_interrupt(&self) -> Option<u8> {
        if self.ewi_pending { Some(0) } else { None } // WWDG IRQ = 0
    }

    fn clear_interrupt(&mut self) {
        self.ewi_pending = false;
    }

    fn reset(&mut self) {
        self.cr = 0x7F;
        self.cfr = 0x7F;
        self.sr = 0;
        self.counter = 0x7F;
        self.enabled = false;
        self.reset_pending = false;
        self.ewi_pending = false;
    }

    fn get_state(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "counter": self.counter,
            "window": (self.cfr & 0x7F),
            "reset_pending": self.reset_pending,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iwdg_enable() {
        let mut iwdg = IwdgPeripheral::new(32_000);
        
        iwdg.write(0x00, 0xCCCC); // Start
        assert!(iwdg.enabled);
    }

    #[test]
    fn test_iwdg_timeout() {
        let mut iwdg = IwdgPeripheral::new(32_000);
        iwdg.write(0x08, 10); // Short reload
        iwdg.write(0x00, 0xCCCC); // Start
        
        // Tick enough to timeout
        for _ in 0..20 {
            iwdg.tick(iwdg.cycles_per_tick);
        }
        
        assert!(iwdg.is_reset_pending());
    }

    #[test]
    fn test_iwdg_feed() {
        let mut iwdg = IwdgPeripheral::new(32_000);
        iwdg.write(0x08, 100);
        iwdg.write(0x00, 0xCCCC);
        
        // Tick halfway
        for _ in 0..50 {
            iwdg.tick(iwdg.cycles_per_tick);
        }
        
        // Feed the dog
        iwdg.write(0x00, 0xAAAA);
        assert_eq!(iwdg.counter, 100);
    }
}
