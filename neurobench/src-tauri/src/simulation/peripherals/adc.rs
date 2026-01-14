//! ADC Peripheral Simulation

use super::Peripheral;

/// ADC Peripheral
pub struct AdcPeripheral {
    instance: u8,
    base: u32,
    
    // Status register
    sr: u32,
    // Control register 1
    cr1: u32,
    // Control register 2
    cr2: u32,
    // Sample time registers
    smpr1: u32,
    smpr2: u32,
    // Injected channel data offset registers
    jofr: [u32; 4],
    // Watchdog high threshold
    htr: u32,
    // Watchdog low threshold
    ltr: u32,
    // Regular sequence registers
    sqr1: u32,
    sqr2: u32,
    sqr3: u32,
    // Injected sequence register
    jsqr: u32,
    // Injected data registers
    jdr: [u32; 4],
    // Regular data register
    dr: u32,
    // Common control register
    ccr: u32,
    
    // Simulated channel values (12-bit)
    channel_values: [u16; 18],
    
    // Conversion state
    converting: bool,
    current_channel: u8,
    conversion_cycles: u64,
    
    irq_pending: bool,
}

impl AdcPeripheral {
    pub fn new(instance: u8, base: u32) -> Self {
        Self {
            instance,
            base,
            sr: 0,
            cr1: 0,
            cr2: 0,
            smpr1: 0,
            smpr2: 0,
            jofr: [0; 4],
            htr: 0x0FFF,
            ltr: 0,
            sqr1: 0,
            sqr2: 0,
            sqr3: 0,
            jsqr: 0,
            jdr: [0; 4],
            dr: 0,
            ccr: 0,
            channel_values: [2048; 18], // Mid-scale default
            converting: false,
            current_channel: 0,
            conversion_cycles: 0,
            irq_pending: false,
        }
    }

    /// Set a channel value for simulation
    pub fn set_channel_value(&mut self, channel: u8, value: u16) {
        if (channel as usize) < self.channel_values.len() {
            self.channel_values[channel as usize] = value.min(4095);
        }
    }

    /// Get current channel value
    fn get_current_value(&self) -> u16 {
        if (self.current_channel as usize) < self.channel_values.len() {
            self.channel_values[self.current_channel as usize]
        } else {
            0
        }
    }
}

impl Peripheral for AdcPeripheral {
    fn name(&self) -> &str {
        match self.instance {
            1 => "ADC1",
            2 => "ADC2",
            3 => "ADC3",
            _ => "ADC?",
        }
    }

    fn read(&self, offset: u32) -> u32 {
        match offset {
            0x00 => self.sr,
            0x04 => self.cr1,
            0x08 => self.cr2,
            0x0C => self.smpr1,
            0x10 => self.smpr2,
            0x14 => self.jofr[0],
            0x18 => self.jofr[1],
            0x1C => self.jofr[2],
            0x20 => self.jofr[3],
            0x24 => self.htr,
            0x28 => self.ltr,
            0x2C => self.sqr1,
            0x30 => self.sqr2,
            0x34 => self.sqr3,
            0x38 => self.jsqr,
            0x3C => self.jdr[0],
            0x40 => self.jdr[1],
            0x44 => self.jdr[2],
            0x48 => self.jdr[3],
            0x4C => self.dr,
            _ => 0,
        }
    }

    fn write(&mut self, offset: u32, value: u32) {
        match offset {
            0x00 => self.sr &= !value, // Write 0 to clear
            0x04 => self.cr1 = value,
            0x08 => {
                self.cr2 = value;
                
                // SWSTART bit starts conversion
                if value & (1 << 30) != 0 {
                    self.converting = true;
                    self.current_channel = (self.sqr3 & 0x1F) as u8;
                    self.conversion_cycles = 15; // ~15 cycles for conversion
                    self.sr &= !(1 << 1); // Clear EOC
                }
                
                // ADON bit
                if value & 1 != 0 {
                    self.sr |= 1 << 5; // STRT
                }
            }
            0x0C => self.smpr1 = value,
            0x10 => self.smpr2 = value,
            0x14 => self.jofr[0] = value,
            0x18 => self.jofr[1] = value,
            0x1C => self.jofr[2] = value,
            0x20 => self.jofr[3] = value,
            0x24 => self.htr = value,
            0x28 => self.ltr = value,
            0x2C => self.sqr1 = value,
            0x30 => self.sqr2 = value,
            0x34 => self.sqr3 = value,
            0x38 => self.jsqr = value,
            _ => {}
        }
    }

    fn tick(&mut self, cycles: u64) {
        if self.converting {
            if cycles >= self.conversion_cycles {
                self.conversion_cycles = 0;
                self.converting = false;
                
                // Complete conversion
                self.dr = self.get_current_value() as u32;
                self.sr |= 1 << 1; // EOC
                
                // Check watchdog
                if self.dr > self.htr || self.dr < self.ltr {
                    self.sr |= 1; // AWD
                }
                
                // Check for interrupt
                if self.cr1 & (1 << 5) != 0 { // EOCIE
                    self.irq_pending = true;
                }
            } else {
                self.conversion_cycles -= cycles;
            }
        }
    }

    fn get_interrupt(&self) -> Option<u8> {
        if self.irq_pending { Some(18) } else { None } // ADC IRQ
    }

    fn clear_interrupt(&mut self) {
        self.irq_pending = false;
    }

    fn reset(&mut self) {
        self.sr = 0;
        self.cr1 = 0;
        self.cr2 = 0;
        self.dr = 0;
        self.converting = false;
        self.irq_pending = false;
    }

    fn get_state(&self) -> serde_json::Value {
        serde_json::json!({
            "instance": self.instance,
            "enabled": self.cr2 & 1 != 0,
            "dr": self.dr,
            "converting": self.converting,
        })
    }
}
