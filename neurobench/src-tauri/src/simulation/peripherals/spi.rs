//! SPI Peripheral Simulation

use super::Peripheral;
use std::collections::VecDeque;

/// SPI Peripheral
pub struct SpiPeripheral {
    instance: u8,
    base: u32,
    
    // Control register 1
    cr1: u32,
    // Control register 2
    cr2: u32,
    // Status register
    sr: u32,
    // Data register
    dr: u16,
    // CRC polynomial
    crcpr: u32,
    // RX CRC
    rxcrcr: u32,
    // TX CRC
    txcrcr: u32,
    
    // TX/RX buffers
    tx_buffer: VecDeque<u16>,
    rx_buffer: VecDeque<u16>,
    
    irq_pending: bool,
}

impl SpiPeripheral {
    pub fn new(instance: u8, base: u32) -> Self {
        Self {
            instance,
            base,
            cr1: 0,
            cr2: 0,
            sr: 0x0002, // TXE set
            dr: 0,
            crcpr: 0x0007,
            rxcrcr: 0,
            txcrcr: 0,
            tx_buffer: VecDeque::new(),
            rx_buffer: VecDeque::new(),
            irq_pending: false,
        }
    }

    pub fn inject_rx(&mut self, data: &[u16]) {
        for word in data {
            self.rx_buffer.push_back(*word);
        }
        if !self.rx_buffer.is_empty() {
            self.sr |= 1; // RXNE
        }
    }
}

impl Peripheral for SpiPeripheral {
    fn name(&self) -> &str {
        match self.instance {
            1 => "SPI1",
            2 => "SPI2",
            3 => "SPI3",
            4 => "SPI4",
            _ => "SPI?",
        }
    }

    fn read(&self, offset: u32) -> u32 {
        match offset {
            0x00 => self.cr1,
            0x04 => self.cr2,
            0x08 => self.sr,
            0x0C => self.dr as u32,
            0x10 => self.crcpr,
            0x14 => self.rxcrcr,
            0x18 => self.txcrcr,
            _ => 0,
        }
    }

    fn write(&mut self, offset: u32, value: u32) {
        match offset {
            0x00 => self.cr1 = value,
            0x04 => self.cr2 = value,
            0x0C => {
                self.dr = value as u16;
                if self.cr1 & (1 << 6) != 0 { // SPE
                    self.tx_buffer.push_back(self.dr);
                    self.sr &= !(1 << 1); // Clear TXE
                }
            }
            0x10 => self.crcpr = value,
            _ => {}
        }
    }

    fn tick(&mut self, _cycles: u64) {
        // Simulate SPI transfer completion
        if !self.tx_buffer.is_empty() {
            self.tx_buffer.pop_front();
            self.sr |= 1 << 1; // TXE
            
            // Simulate loopback for testing
            if self.rx_buffer.len() < 8 {
                self.rx_buffer.push_back(0xFF);
                self.sr |= 1; // RXNE
            }
        }
        
        if !self.rx_buffer.is_empty() {
            self.dr = self.rx_buffer.front().cloned().unwrap_or(0);
        }
    }

    fn get_interrupt(&self) -> Option<u8> {
        if self.irq_pending { Some(35) } else { None }
    }

    fn clear_interrupt(&mut self) {
        self.irq_pending = false;
    }

    fn reset(&mut self) {
        self.cr1 = 0;
        self.cr2 = 0;
        self.sr = 0x0002;
        self.dr = 0;
        self.tx_buffer.clear();
        self.rx_buffer.clear();
    }

    fn get_state(&self) -> serde_json::Value {
        serde_json::json!({
            "instance": self.instance,
            "enabled": self.cr1 & (1 << 6) != 0,
            "sr": self.sr,
        })
    }
}
