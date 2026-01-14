//! I2C Peripheral Simulation

use super::Peripheral;
use std::collections::VecDeque;

/// I2C Peripheral
pub struct I2cPeripheral {
    instance: u8,
    base: u32,
    
    // Control register 1
    cr1: u32,
    // Control register 2
    cr2: u32,
    // Own address register 1
    oar1: u32,
    // Own address register 2
    oar2: u32,
    // Data register
    dr: u8,
    // Status register 1
    sr1: u32,
    // Status register 2
    sr2: u32,
    // Clock control register
    ccr: u32,
    // Rise time register
    trise: u32,
    
    // TX/RX buffers
    tx_buffer: VecDeque<u8>,
    rx_buffer: VecDeque<u8>,
    
    // State machine
    state: I2cState,
    target_address: u8,
    
    irq_pending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum I2cState {
    Idle,
    StartSent,
    AddressSent,
    Transmitting,
    Receiving,
}

impl I2cPeripheral {
    pub fn new(instance: u8, base: u32) -> Self {
        Self {
            instance,
            base,
            cr1: 0,
            cr2: 0,
            oar1: 0,
            oar2: 0,
            dr: 0,
            sr1: 0,
            sr2: 0,
            ccr: 0,
            trise: 0,
            tx_buffer: VecDeque::new(),
            rx_buffer: VecDeque::new(),
            state: I2cState::Idle,
            target_address: 0,
            irq_pending: false,
        }
    }

    pub fn inject_rx(&mut self, data: &[u8]) {
        for byte in data {
            self.rx_buffer.push_back(*byte);
        }
        if !self.rx_buffer.is_empty() {
            self.sr1 |= 1 << 6; // RXNE
        }
    }
}

impl Peripheral for I2cPeripheral {
    fn name(&self) -> &str {
        match self.instance {
            1 => "I2C1",
            2 => "I2C2",
            3 => "I2C3",
            _ => "I2C?",
        }
    }

    fn read(&self, offset: u32) -> u32 {
        match offset {
            0x00 => self.cr1,
            0x04 => self.cr2,
            0x08 => self.oar1,
            0x0C => self.oar2,
            0x10 => self.dr as u32,
            0x14 => self.sr1,
            0x18 => self.sr2,
            0x1C => self.ccr,
            0x20 => self.trise,
            _ => 0,
        }
    }

    fn write(&mut self, offset: u32, value: u32) {
        match offset {
            0x00 => {
                self.cr1 = value;
                
                // START condition
                if value & (1 << 8) != 0 {
                    self.state = I2cState::StartSent;
                    self.sr1 |= 1; // SB (Start bit)
                }
                
                // STOP condition
                if value & (1 << 9) != 0 {
                    self.state = I2cState::Idle;
                    self.sr1 = 0;
                }
            }
            0x04 => self.cr2 = value,
            0x08 => self.oar1 = value,
            0x0C => self.oar2 = value,
            0x10 => {
                self.dr = value as u8;
                
                if self.state == I2cState::StartSent {
                    // Address byte
                    self.target_address = (value >> 1) as u8;
                    let is_read = value & 1 != 0;
                    self.state = I2cState::AddressSent;
                    self.sr1 |= 1 << 1; // ADDR
                    self.sr2 |= if is_read { 0 } else { 1 << 2 }; // TRA
                } else if self.state == I2cState::AddressSent || self.state == I2cState::Transmitting {
                    // Data byte
                    self.tx_buffer.push_back(self.dr);
                    self.state = I2cState::Transmitting;
                    self.sr1 |= 1 << 7; // TXE
                    self.sr1 |= 1 << 2; // BTF
                }
            }
            0x1C => self.ccr = value,
            0x20 => self.trise = value,
            _ => {}
        }
    }

    fn tick(&mut self, _cycles: u64) {
        // Handle RX
        if !self.rx_buffer.is_empty() && (self.sr1 & (1 << 6)) == 0 {
            self.dr = self.rx_buffer.pop_front().unwrap_or(0);
            self.sr1 |= 1 << 6; // RXNE
        }
    }

    fn get_interrupt(&self) -> Option<u8> {
        if self.irq_pending {
            Some(match self.instance {
                1 => 31, // I2C1_EV
                2 => 33, // I2C2_EV
                3 => 72, // I2C3_EV
                _ => 31,
            })
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
        self.sr1 = 0;
        self.sr2 = 0;
        self.state = I2cState::Idle;
        self.tx_buffer.clear();
        self.rx_buffer.clear();
    }

    fn get_state(&self) -> serde_json::Value {
        serde_json::json!({
            "instance": self.instance,
            "enabled": self.cr1 & 1 != 0,
            "state": format!("{:?}", self.state),
        })
    }
}
