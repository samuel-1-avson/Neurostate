//! GPIO Peripheral Simulation
//!
//! Simulates GPIO ports with input/output modes, pull resistors, and interrupts.

use super::{Peripheral, PinState, PinMode, OutputType, PullConfig, GpioState};
use serde::{Deserialize, Serialize};

/// GPIO Port (GPIOA, GPIOB, etc.)
pub struct GpioPort {
    name: char,
    base: u32,
    /// Pin states
    pins: [PinState; 16],
    /// Mode register (MODER)
    moder: u32,
    /// Output type register (OTYPER)
    otyper: u32,
    /// Output speed register (OSPEEDR)
    ospeedr: u32,
    /// Pull-up/pull-down register (PUPDR)
    pupdr: u32,
    /// Input data register (IDR)
    idr: u32,
    /// Output data register (ODR)
    odr: u32,
    /// Bit set/reset register (BSRR) - write only
    /// Alternate function low register (AFRL)
    afrl: u32,
    /// Alternate function high register (AFRH)
    afrh: u32,
    /// Lock register (LCKR)
    lckr: u32,
    /// External interrupt pending
    exti_pending: u16,
}

impl GpioPort {
    pub fn new(name: char, base: u32) -> Self {
        Self {
            name,
            base,
            pins: [PinState::default(); 16],
            moder: 0xFFFF_FFFF, // All analog by default (reset value varies by port)
            otyper: 0,
            ospeedr: 0,
            pupdr: 0,
            idr: 0,
            odr: 0,
            afrl: 0,
            afrh: 0,
            lckr: 0,
            exti_pending: 0,
        }
    }

    /// Inject an external input on a pin
    pub fn inject_input(&mut self, pin: u8, state: bool) {
        if pin < 16 {
            let old_state = self.pins[pin as usize].value;
            self.pins[pin as usize].value = state;
            
            // Update IDR based on pin mode
            if self.pins[pin as usize].mode == PinMode::Input {
                if state {
                    self.idr |= 1 << pin;
                } else {
                    self.idr &= !(1 << pin);
                }
                
                // Check for EXTI edge
                if old_state != state {
                    self.exti_pending |= 1 << pin;
                }
            }
        }
    }

    /// Get pin output state
    pub fn get_output(&self, pin: u8) -> bool {
        if pin < 16 {
            (self.odr >> pin) & 1 != 0
        } else {
            false
        }
    }

    /// Get state structure for reporting
    pub fn get_state_struct(&self) -> GpioState {
        let mut pins = [PinState::default(); 16];
        for i in 0..16 {
            pins[i] = self.pins[i];
        }
        GpioState {
            port: self.name,
            pins,
        }
    }

    /// Restore from state JSON
    pub fn restore_state(&mut self, _state: &serde_json::Value) {
        // TODO: Implement state restoration
    }

    /// Update pin configuration from registers
    fn update_pin_config(&mut self, pin: u8) {
        if pin >= 16 {
            return;
        }
        
        let i = pin as usize;
        let mode_bits = (self.moder >> (pin * 2)) & 0x3;
        let otype_bit = (self.otyper >> pin) & 0x1;
        let pupd_bits = (self.pupdr >> (pin * 2)) & 0x3;
        let af_bits = if pin < 8 {
            (self.afrl >> (pin * 4)) & 0xF
        } else {
            (self.afrh >> ((pin - 8) * 4)) & 0xF
        };
        
        self.pins[i].mode = match mode_bits {
            0 => PinMode::Input,
            1 => PinMode::Output,
            2 => PinMode::AlternateFunction,
            3 => PinMode::Analog,
            _ => PinMode::Input,
        };
        
        self.pins[i].output_type = if otype_bit == 0 {
            OutputType::PushPull
        } else {
            OutputType::OpenDrain
        };
        
        self.pins[i].pull = match pupd_bits {
            0 => PullConfig::None,
            1 => PullConfig::PullUp,
            2 => PullConfig::PullDown,
            _ => PullConfig::None,
        };
        
        self.pins[i].alt_function = af_bits as u8;
    }
}

impl Peripheral for GpioPort {
    fn name(&self) -> &str {
        // This is a bit awkward, but we need a static lifetime
        match self.name {
            'A' => "GPIOA",
            'B' => "GPIOB",
            'C' => "GPIOC",
            'D' => "GPIOD",
            'E' => "GPIOE",
            'F' => "GPIOF",
            'G' => "GPIOG",
            'H' => "GPIOH",
            _ => "GPIO?",
        }
    }

    fn read(&self, offset: u32) -> u32 {
        match offset {
            0x00 => self.moder,    // MODER
            0x04 => self.otyper,   // OTYPER
            0x08 => self.ospeedr,  // OSPEEDR
            0x0C => self.pupdr,    // PUPDR
            0x10 => self.idr,      // IDR
            0x14 => self.odr,      // ODR
            0x18 => 0,             // BSRR (write-only)
            0x1C => self.lckr,     // LCKR
            0x20 => self.afrl,     // AFRL
            0x24 => self.afrh,     // AFRH
            _ => 0,
        }
    }

    fn write(&mut self, offset: u32, value: u32) {
        match offset {
            0x00 => {
                self.moder = value;
                for i in 0..16 {
                    self.update_pin_config(i);
                }
            }
            0x04 => {
                self.otyper = value;
                for i in 0..16 {
                    self.update_pin_config(i);
                }
            }
            0x08 => self.ospeedr = value,
            0x0C => {
                self.pupdr = value;
                for i in 0..16 {
                    self.update_pin_config(i);
                }
            }
            0x14 => {
                self.odr = value;
                // Update output pin values
                for i in 0..16 {
                    if self.pins[i].mode == PinMode::Output {
                        self.pins[i].value = (value >> i) & 1 != 0;
                    }
                }
            }
            0x18 => {
                // BSRR - Bit Set/Reset Register
                // Lower 16 bits set, upper 16 bits reset
                let set_bits = value & 0xFFFF;
                let reset_bits = (value >> 16) & 0xFFFF;
                self.odr = (self.odr | set_bits) & !reset_bits;
                
                // Update output pin values
                for i in 0..16 {
                    if self.pins[i].mode == PinMode::Output {
                        self.pins[i].value = (self.odr >> i) & 1 != 0;
                    }
                }
            }
            0x1C => {
                // LCKR - Lock Register (complex locking sequence)
                self.lckr = value;
            }
            0x20 => {
                self.afrl = value;
                for i in 0..8 {
                    self.update_pin_config(i);
                }
            }
            0x24 => {
                self.afrh = value;
                for i in 8..16 {
                    self.update_pin_config(i);
                }
            }
            _ => {}
        }
    }

    fn tick(&mut self, _cycles: u64) {
        // GPIO doesn't need periodic updates
    }

    fn get_interrupt(&self) -> Option<u8> {
        // EXTI is separate, but we track pending here
        if self.exti_pending != 0 {
            // Return the lowest pending bit as interrupt
            for i in 0..16 {
                if (self.exti_pending >> i) & 1 != 0 {
                    return Some(6 + i); // EXTI0-15 are IRQ 6-22
                }
            }
        }
        None
    }

    fn clear_interrupt(&mut self) {
        self.exti_pending = 0;
    }

    fn reset(&mut self) {
        self.moder = 0xFFFF_FFFF;
        self.otyper = 0;
        self.ospeedr = 0;
        self.pupdr = 0;
        self.idr = 0;
        self.odr = 0;
        self.afrl = 0;
        self.afrh = 0;
        self.lckr = 0;
        self.exti_pending = 0;
        self.pins = [PinState::default(); 16];
    }

    fn get_state(&self) -> serde_json::Value {
        serde_json::json!({
            "port": self.name.to_string(),
            "moder": self.moder,
            "odr": self.odr,
            "idr": self.idr,
            "pins": self.pins.iter().enumerate().map(|(i, p)| {
                serde_json::json!({
                    "pin": i,
                    "mode": format!("{:?}", p.mode),
                    "value": p.value,
                })
            }).collect::<Vec<_>>(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpio_creation() {
        let gpio = GpioPort::new('A', 0x4002_0000);
        assert_eq!(gpio.name, 'A');
    }

    #[test]
    fn test_gpio_bsrr() {
        let mut gpio = GpioPort::new('A', 0x4002_0000);
        
        // Set pin 5 as output
        gpio.write(0x00, 0b01 << 10); // MODER5 = 01 (output)
        
        // Set pin 5
        gpio.write(0x18, 1 << 5);
        assert_eq!(gpio.odr & (1 << 5), 1 << 5);
        assert!(gpio.pins[5].value);
        
        // Reset pin 5
        gpio.write(0x18, 1 << (5 + 16));
        assert_eq!(gpio.odr & (1 << 5), 0);
        assert!(!gpio.pins[5].value);
    }

    #[test]
    fn test_gpio_input_inject() {
        let mut gpio = GpioPort::new('A', 0x4002_0000);
        
        // Set pin 0 as input
        gpio.write(0x00, 0); // MODER0 = 00 (input)
        gpio.update_pin_config(0);
        
        gpio.inject_input(0, true);
        assert_eq!(gpio.idr & 1, 1);
    }
}
