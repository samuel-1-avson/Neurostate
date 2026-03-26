//! Timer Peripheral Simulation

use super::Peripheral;

/// Timer Peripheral
pub struct TimerPeripheral {
    instance: u8,
    base: u32,
    system_clock: u32,
    
    // Control register 1
    cr1: u32,
    // Control register 2
    cr2: u32,
    // Slave mode control
    smcr: u32,
    // DMA/interrupt enable
    dier: u32,
    // Status register
    sr: u32,
    // Event generation register
    egr: u32,
    // Capture/compare mode registers
    ccmr1: u32,
    ccmr2: u32,
    // Capture/compare enable register
    ccer: u32,
    // Counter
    cnt: u32,
    // Prescaler
    psc: u32,
    // Auto-reload register
    arr: u32,
    // Repetition counter (advanced timers only)
    rcr: u32,
    // Capture/compare registers
    ccr: [u32; 4],
    
    // Internal state
    counter_cycles: u64,
    cycles_per_tick: u64,
    irq_pending: bool,
}

impl TimerPeripheral {
    pub fn new(instance: u8, base: u32, system_clock: u32) -> Self {
        Self {
            instance,
            base,
            system_clock,
            cr1: 0,
            cr2: 0,
            smcr: 0,
            dier: 0,
            sr: 0,
            egr: 0,
            ccmr1: 0,
            ccmr2: 0,
            ccer: 0,
            cnt: 0,
            psc: 0,
            arr: 0xFFFF,
            rcr: 0,
            ccr: [0; 4],
            counter_cycles: 0,
            cycles_per_tick: 1,
            irq_pending: false,
        }
    }

    fn update_timing(&mut self) {
        self.cycles_per_tick = (self.psc + 1) as u64;
    }

    /// Get PWM duty cycle for a channel (0-100%)
    pub fn get_pwm_duty(&self, channel: usize) -> Option<u8> {
        if channel < 4 && self.arr > 0 {
            let duty = (self.ccr[channel] * 100) / self.arr;
            Some(duty.min(100) as u8)
        } else {
            None
        }
    }
}

impl Peripheral for TimerPeripheral {
    fn name(&self) -> &str {
        match self.instance {
            1 => "TIM1",
            2 => "TIM2",
            3 => "TIM3",
            4 => "TIM4",
            5 => "TIM5",
            6 => "TIM6",
            7 => "TIM7",
            8 => "TIM8",
            9 => "TIM9",
            10 => "TIM10",
            11 => "TIM11",
            12 => "TIM12",
            13 => "TIM13",
            14 => "TIM14",
            _ => "TIM?",
        }
    }

    fn read(&self, offset: u32) -> u32 {
        match offset {
            0x00 => self.cr1,
            0x04 => self.cr2,
            0x08 => self.smcr,
            0x0C => self.dier,
            0x10 => self.sr,
            0x14 => self.egr,
            0x18 => self.ccmr1,
            0x1C => self.ccmr2,
            0x20 => self.ccer,
            0x24 => self.cnt,
            0x28 => self.psc,
            0x2C => self.arr,
            0x30 => self.rcr,
            0x34 => self.ccr[0],
            0x38 => self.ccr[1],
            0x3C => self.ccr[2],
            0x40 => self.ccr[3],
            _ => 0,
        }
    }

    fn write(&mut self, offset: u32, value: u32) {
        match offset {
            0x00 => self.cr1 = value,
            0x04 => self.cr2 = value,
            0x08 => self.smcr = value,
            0x0C => self.dier = value,
            0x10 => self.sr &= !value, // Write 0 to clear
            0x14 => {
                self.egr = value;
                // UG bit generates update event
                if value & 1 != 0 {
                    self.cnt = 0;
                    self.sr |= 1; // UIF
                }
            }
            0x18 => self.ccmr1 = value,
            0x1C => self.ccmr2 = value,
            0x20 => self.ccer = value,
            0x24 => self.cnt = value,
            0x28 => {
                self.psc = value;
                self.update_timing();
            }
            0x2C => self.arr = value,
            0x30 => self.rcr = value,
            0x34 => self.ccr[0] = value,
            0x38 => self.ccr[1] = value,
            0x3C => self.ccr[2] = value,
            0x40 => self.ccr[3] = value,
            _ => {}
        }
    }

    fn tick(&mut self, cycles: u64) {
        // Only tick if enabled
        if self.cr1 & 1 == 0 {
            return;
        }

        self.counter_cycles += cycles;
        
        while self.counter_cycles >= self.cycles_per_tick {
            self.counter_cycles -= self.cycles_per_tick;
            
            // Count direction
            let dir = (self.cr1 >> 4) & 1;
            
            if dir == 0 {
                // Count up
                self.cnt = self.cnt.wrapping_add(1);
                if self.cnt > self.arr {
                    self.cnt = 0;
                    self.sr |= 1; // UIF
                    
                    if self.dier & 1 != 0 {
                        self.irq_pending = true;
                    }
                }
            } else {
                // Count down
                if self.cnt == 0 {
                    self.cnt = self.arr;
                    self.sr |= 1; // UIF
                    
                    if self.dier & 1 != 0 {
                        self.irq_pending = true;
                    }
                } else {
                    self.cnt -= 1;
                }
            }
            
            // Check capture/compare
            for i in 0..4 {
                if self.cnt == self.ccr[i] {
                    self.sr |= 1 << (i + 1); // CCxIF
                    
                    if self.dier & (1 << (i + 1)) != 0 {
                        self.irq_pending = true;
                    }
                }
            }
        }
    }

    fn get_interrupt(&self) -> Option<u8> {
        if self.irq_pending {
            let irq = match self.instance {
                1 => 27, // TIM1_UP
                2 => 28, // TIM2
                3 => 29, // TIM3
                4 => 30, // TIM4
                5 => 50, // TIM5
                6 => 54, // TIM6
                7 => 55, // TIM7
                8 => 46, // TIM8_UP
                _ => 28,
            };
            Some(irq)
        } else {
            None
        }
    }

    fn clear_interrupt(&mut self) {
        self.irq_pending = false;
    }

    fn reset(&mut self) {
        self.cr1 = 0;
        self.cr2 = 0;
        self.dier = 0;
        self.sr = 0;
        self.cnt = 0;
        self.psc = 0;
        self.arr = 0xFFFF;
        self.ccr = [0; 4];
        self.counter_cycles = 0;
        self.cycles_per_tick = 1;
        self.irq_pending = false;
    }

    fn get_state(&self) -> serde_json::Value {
        serde_json::json!({
            "instance": self.instance,
            "enabled": self.cr1 & 1 != 0,
            "cnt": self.cnt,
            "arr": self.arr,
            "psc": self.psc,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_counting() {
        let mut timer = TimerPeripheral::new(2, 0x4000_0000, 84_000_000);
        
        // Enable timer
        timer.write(0x00, 1);
        timer.write(0x2C, 100); // ARR = 100
        
        // Tick 50 times
        timer.tick(50);
        
        assert_eq!(timer.cnt, 50);
    }

    #[test]
    fn test_timer_overflow() {
        let mut timer = TimerPeripheral::new(2, 0x4000_0000, 84_000_000);
        
        timer.write(0x00, 1); // Enable
        timer.write(0x2C, 10); // ARR = 10
        timer.write(0x0C, 1); // UIE = 1
        
        timer.tick(15);
        
        assert!(timer.sr & 1 != 0); // UIF set
        assert!(timer.irq_pending);
    }
}
