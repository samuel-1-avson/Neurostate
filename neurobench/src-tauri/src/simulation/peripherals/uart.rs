//! UART Peripheral Simulation
//!
//! Simulates USART/UART with baud rate timing, TX/RX buffers, and interrupts.

use super::{Peripheral, UartState};
use std::collections::VecDeque;

/// UART Peripheral
pub struct UartPeripheral {
    instance: u8,
    base: u32,
    system_clock: u32,
    
    // Status register (SR/ISR)
    sr: u32,
    // Data register (DR/RDR/TDR)
    dr: u8,
    // Baud rate register (BRR)
    brr: u32,
    // Control register 1 (CR1)
    cr1: u32,
    // Control register 2 (CR2)
    cr2: u32,
    // Control register 3 (CR3)
    cr3: u32,
    // Guard time and prescaler (GTPR)
    gtpr: u32,
    
    // TX buffer
    tx_buffer: VecDeque<u8>,
    // RX buffer
    rx_buffer: VecDeque<u8>,
    
    // Timing
    tx_cycles_remaining: u64,
    rx_cycles_remaining: u64,
    cycles_per_byte: u64,
    
    // Interrupt pending
    irq_pending: bool,
}

impl UartPeripheral {
    pub fn new(instance: u8, base: u32, system_clock: u32) -> Self {
        Self {
            instance,
            base,
            system_clock,
            sr: 0x00C0, // TXE and TC set (transmitter empty)
            dr: 0,
            brr: 0,
            cr1: 0,
            cr2: 0,
            cr3: 0,
            gtpr: 0,
            tx_buffer: VecDeque::new(),
            rx_buffer: VecDeque::new(),
            tx_cycles_remaining: 0,
            rx_cycles_remaining: 0,
            cycles_per_byte: 0,
            irq_pending: false,
        }
    }

    /// Calculate cycles per byte based on baud rate
    fn calculate_timing(&mut self) {
        if self.brr > 0 {
            let mantissa = (self.brr >> 4) & 0xFFF;
            let fraction = self.brr & 0xF;
            let divisor = mantissa * 16 + fraction;
            
            if divisor > 0 {
                let baud = self.system_clock / divisor;
                // 10 bits per byte (start + 8 data + stop)
                self.cycles_per_byte = (self.system_clock as u64) / (baud as u64);
            }
        }
    }

    /// Inject data into RX buffer (simulate received data)
    pub fn inject_rx(&mut self, data: &[u8]) {
        for byte in data {
            self.rx_buffer.push_back(*byte);
        }
        
        // Set RXNE if data available
        if !self.rx_buffer.is_empty() {
            self.sr |= 1 << 5; // RXNE
            
            // Check if RXNEIE is enabled
            if self.cr1 & (1 << 5) != 0 {
                self.irq_pending = true;
            }
        }
    }

    /// Get transmitted data
    pub fn get_tx_data(&mut self) -> Vec<u8> {
        self.tx_buffer.drain(..).collect()
    }

    /// Get state structure
    pub fn get_state_struct(&self) -> UartState {
        let enabled = self.cr1 & (1 << 13) != 0;
        let baud = if self.brr > 0 {
            let mantissa = (self.brr >> 4) & 0xFFF;
            let fraction = self.brr & 0xF;
            let divisor = mantissa * 16 + fraction;
            if divisor > 0 {
                self.system_clock / divisor
            } else {
                0
            }
        } else {
            0
        };
        
        UartState {
            instance: self.instance,
            baud_rate: baud,
            enabled,
            tx_buffer: self.tx_buffer.iter().cloned().collect(),
            rx_buffer: self.rx_buffer.iter().cloned().collect(),
        }
    }
}

impl Peripheral for UartPeripheral {
    fn name(&self) -> &str {
        match self.instance {
            1 => "USART1",
            2 => "USART2",
            3 => "USART3",
            4 => "UART4",
            5 => "UART5",
            6 => "USART6",
            _ => "USART?",
        }
    }

    fn read(&self, offset: u32) -> u32 {
        match offset {
            0x00 => self.sr,      // SR
            0x04 => {
                // DR - reading clears RXNE
                self.dr as u32
            }
            0x08 => self.brr,     // BRR
            0x0C => self.cr1,     // CR1
            0x10 => self.cr2,     // CR2
            0x14 => self.cr3,     // CR3
            0x18 => self.gtpr,    // GTPR
            _ => 0,
        }
    }

    fn write(&mut self, offset: u32, value: u32) {
        match offset {
            0x00 => {
                // SR - write to clear some flags
                // Only specific flags can be cleared by writing 0
                let clearable = 0x3FF;
                self.sr &= value | !clearable;
            }
            0x04 => {
                // DR - data register
                self.dr = value as u8;
                
                // If transmitter enabled, queue byte
                if self.cr1 & (1 << 3) != 0 { // TE
                    self.tx_buffer.push_back(self.dr);
                    self.sr &= !(1 << 7); // Clear TXE temporarily
                    self.sr &= !(1 << 6); // Clear TC
                    self.tx_cycles_remaining = self.cycles_per_byte;
                }
            }
            0x08 => {
                self.brr = value;
                self.calculate_timing();
            }
            0x0C => self.cr1 = value,
            0x10 => self.cr2 = value,
            0x14 => self.cr3 = value,
            0x18 => self.gtpr = value,
            _ => {}
        }
    }

    fn tick(&mut self, cycles: u64) {
        // Handle TX timing
        if self.tx_cycles_remaining > 0 {
            if cycles >= self.tx_cycles_remaining {
                self.tx_cycles_remaining = 0;
                self.sr |= 1 << 7; // TXE
                self.sr |= 1 << 6; // TC
                
                // Check for TXEIE or TCIE
                if (self.cr1 & (1 << 7) != 0) || (self.cr1 & (1 << 6) != 0) {
                    self.irq_pending = true;
                }
            } else {
                self.tx_cycles_remaining -= cycles;
            }
        }
        
        // Handle RX - check if data available and update status
        if !self.rx_buffer.is_empty() && (self.sr & (1 << 5)) == 0 {
            self.dr = self.rx_buffer.pop_front().unwrap_or(0);
            self.sr |= 1 << 5; // RXNE
            
            if self.cr1 & (1 << 5) != 0 { // RXNEIE
                self.irq_pending = true;
            }
        }
    }

    fn get_interrupt(&self) -> Option<u8> {
        if self.irq_pending {
            // USART IRQ numbers (STM32F4)
            let irq = match self.instance {
                1 => 37,
                2 => 38,
                3 => 39,
                4 => 52,
                5 => 53,
                6 => 71,
                _ => 37,
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
        self.sr = 0x00C0;
        self.dr = 0;
        self.brr = 0;
        self.cr1 = 0;
        self.cr2 = 0;
        self.cr3 = 0;
        self.gtpr = 0;
        self.tx_buffer.clear();
        self.rx_buffer.clear();
        self.tx_cycles_remaining = 0;
        self.rx_cycles_remaining = 0;
        self.cycles_per_byte = 0;
        self.irq_pending = false;
    }

    fn get_state(&self) -> serde_json::Value {
        serde_json::json!({
            "instance": self.instance,
            "sr": self.sr,
            "cr1": self.cr1,
            "brr": self.brr,
            "tx_pending": self.tx_buffer.len(),
            "rx_pending": self.rx_buffer.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uart_creation() {
        let uart = UartPeripheral::new(1, 0x4001_1000, 84_000_000);
        assert_eq!(uart.instance, 1);
        assert_eq!(uart.sr & (1 << 7), 1 << 7); // TXE set
    }

    #[test]
    fn test_uart_rx_inject() {
        let mut uart = UartPeripheral::new(1, 0x4001_1000, 84_000_000);
        uart.inject_rx(&[0x41, 0x42, 0x43]); // "ABC"
        
        assert_eq!(uart.rx_buffer.len(), 3);
        assert_eq!(uart.sr & (1 << 5), 1 << 5); // RXNE set
    }

    #[test]
    fn test_uart_baud_calculation() {
        let mut uart = UartPeripheral::new(1, 0x4001_1000, 84_000_000);
        
        // Set BRR for ~115200 baud
        // 84MHz / 115200 = 729.17 -> 729 = 0x2D9
        // Mantissa = 45 (0x2D), Fraction = 9
        uart.write(0x08, (45 << 4) | 9);
        
        assert!(uart.cycles_per_byte > 0);
    }
}
