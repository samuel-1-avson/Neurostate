//! CAN Bus Peripheral Simulation
//!
//! Simulates CAN 2.0A/B with TX/RX mailboxes, filters, and interrupts.

use super::Peripheral;
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

/// CAN message frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanFrame {
    /// Standard (11-bit) or Extended (29-bit) ID
    pub id: u32,
    /// Extended ID flag
    pub extended: bool,
    /// Remote transmission request
    pub rtr: bool,
    /// Data length (0-8)
    pub dlc: u8,
    /// Data bytes
    pub data: [u8; 8],
    /// Timestamp (in cycles)
    pub timestamp: u64,
}

impl CanFrame {
    pub fn new_standard(id: u16, data: &[u8]) -> Self {
        let mut frame_data = [0u8; 8];
        let len = data.len().min(8);
        frame_data[..len].copy_from_slice(&data[..len]);
        
        Self {
            id: id as u32,
            extended: false,
            rtr: false,
            dlc: len as u8,
            data: frame_data,
            timestamp: 0,
        }
    }

    pub fn new_extended(id: u32, data: &[u8]) -> Self {
        let mut frame_data = [0u8; 8];
        let len = data.len().min(8);
        frame_data[..len].copy_from_slice(&data[..len]);
        
        Self {
            id: id & 0x1FFF_FFFF,
            extended: true,
            rtr: false,
            dlc: len as u8,
            data: frame_data,
            timestamp: 0,
        }
    }
}

/// CAN filter configuration
#[derive(Debug, Clone, Default)]
pub struct CanFilter {
    pub id: u32,
    pub mask: u32,
    pub extended: bool,
    pub enabled: bool,
    pub fifo: u8, // 0 or 1
}

impl CanFilter {
    pub fn matches(&self, frame: &CanFrame) -> bool {
        if !self.enabled {
            return false;
        }
        if self.extended != frame.extended {
            return false;
        }
        (frame.id & self.mask) == (self.id & self.mask)
    }
}

/// CAN peripheral state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CanState {
    Initialization,
    Normal,
    Sleep,
    Loopback,
    Silent,
    SilentLoopback,
    BusOff,
}

/// CAN Peripheral
pub struct CanPeripheral {
    instance: u8,
    base: u32,
    
    // Master control register
    mcr: u32,
    // Master status register
    msr: u32,
    // Transmit status register
    tsr: u32,
    // Receive FIFO 0 register
    rf0r: u32,
    // Receive FIFO 1 register
    rf1r: u32,
    // Interrupt enable register
    ier: u32,
    // Error status register
    esr: u32,
    // Bit timing register
    btr: u32,
    
    // TX mailboxes (3)
    tx_mailboxes: [Option<CanFrame>; 3],
    // RX FIFOs (2 x 3 deep)
    rx_fifo0: VecDeque<CanFrame>,
    rx_fifo1: VecDeque<CanFrame>,
    // Filters (14 available)
    filters: [CanFilter; 14],
    
    // State
    state: CanState,
    tx_error_count: u8,
    rx_error_count: u8,
    
    // Timing
    cycles_per_bit: u64,
    current_cycles: u64,
    
    irq_pending: bool,
}

impl CanPeripheral {
    pub fn new(instance: u8, base: u32, system_clock: u32) -> Self {
        // Default to 500kbps
        let bitrate = 500_000;
        let cycles_per_bit = (system_clock / bitrate) as u64;
        
        Self {
            instance,
            base,
            mcr: 0x0001_0002, // INRQ + SLEEP set by default
            msr: 0x0000_0C02, // SLAK + INAK set
            tsr: 0x1C00_0000, // All TX mailboxes empty
            rf0r: 0,
            rf1r: 0,
            ier: 0,
            esr: 0,
            btr: 0x0123_0000, // Default bit timing
            tx_mailboxes: [None, None, None],
            rx_fifo0: VecDeque::new(),
            rx_fifo1: VecDeque::new(),
            filters: Default::default(),
            state: CanState::Initialization,
            tx_error_count: 0,
            rx_error_count: 0,
            cycles_per_bit,
            current_cycles: 0,
            irq_pending: false,
        }
    }

    /// Inject a received frame
    pub fn inject_rx(&mut self, frame: CanFrame) {
        // Check filters
        for filter in &self.filters {
            if filter.matches(&frame) {
                if filter.fifo == 0 {
                    if self.rx_fifo0.len() < 3 {
                        self.rx_fifo0.push_back(frame.clone());
                        self.rf0r = (self.rf0r & !0x3) | (self.rx_fifo0.len() as u32);
                        
                        if self.ier & (1 << 1) != 0 { // FMPIE0
                            self.irq_pending = true;
                        }
                    } else {
                        self.rf0r |= 1 << 4; // FOVR0 (overflow)
                    }
                } else {
                    if self.rx_fifo1.len() < 3 {
                        self.rx_fifo1.push_back(frame.clone());
                        self.rf1r = (self.rf1r & !0x3) | (self.rx_fifo1.len() as u32);
                        
                        if self.ier & (1 << 4) != 0 { // FMPIE1
                            self.irq_pending = true;
                        }
                    } else {
                        self.rf1r |= 1 << 4; // FOVR1
                    }
                }
                return;
            }
        }
    }

    /// Get transmitted frames
    pub fn get_tx_frames(&mut self) -> Vec<CanFrame> {
        let mut frames = Vec::new();
        for mailbox in &mut self.tx_mailboxes {
            if let Some(frame) = mailbox.take() {
                frames.push(frame);
            }
        }
        // Mark mailboxes empty
        self.tsr |= 0x1C00_0000;
        frames
    }

    /// Configure a filter
    pub fn set_filter(&mut self, index: usize, filter: CanFilter) {
        if index < 14 {
            self.filters[index] = filter;
        }
    }
}

impl Peripheral for CanPeripheral {
    fn name(&self) -> &str {
        match self.instance {
            1 => "CAN1",
            2 => "CAN2",
            _ => "CAN?",
        }
    }

    fn read(&self, offset: u32) -> u32 {
        match offset {
            0x00 => self.mcr,
            0x04 => self.msr,
            0x08 => self.tsr,
            0x0C => self.rf0r,
            0x10 => self.rf1r,
            0x14 => self.ier,
            0x18 => self.esr,
            0x1C => self.btr,
            // TX mailboxes
            0x180 => {
                if let Some(frame) = &self.tx_mailboxes[0] {
                    (frame.id << 21) | if frame.extended { 4 } else { 0 } | if frame.rtr { 2 } else { 0 }
                } else { 0 }
            }
            // RX FIFO 0
            0x1B0 => {
                if let Some(frame) = self.rx_fifo0.front() {
                    (frame.id << 21) | if frame.extended { 4 } else { 0 } | if frame.rtr { 2 } else { 0 }
                } else { 0 }
            }
            0x1B4 => {
                if let Some(frame) = self.rx_fifo0.front() {
                    (frame.dlc as u32) | ((frame.timestamp as u32) << 16)
                } else { 0 }
            }
            0x1B8 => {
                if let Some(frame) = self.rx_fifo0.front() {
                    u32::from_le_bytes([frame.data[0], frame.data[1], frame.data[2], frame.data[3]])
                } else { 0 }
            }
            0x1BC => {
                if let Some(frame) = self.rx_fifo0.front() {
                    u32::from_le_bytes([frame.data[4], frame.data[5], frame.data[6], frame.data[7]])
                } else { 0 }
            }
            _ => 0,
        }
    }

    fn write(&mut self, offset: u32, value: u32) {
        match offset {
            0x00 => {
                let old_inrq = self.mcr & 1;
                self.mcr = value;
                
                // INRQ bit - enter/leave initialization
                if value & 1 != 0 && old_inrq == 0 {
                    self.state = CanState::Initialization;
                    self.msr |= 1; // INAK
                } else if value & 1 == 0 && old_inrq != 0 {
                    self.state = CanState::Normal;
                    self.msr &= !1;
                }
                
                // SLEEP
                if value & (1 << 1) != 0 {
                    self.state = CanState::Sleep;
                    self.msr |= 1 << 1; // SLAK
                } else {
                    self.msr &= !(1 << 1);
                }
            }
            0x14 => self.ier = value,
            0x1C => self.btr = value,
            // TX mailbox request
            0x180 => {
                let id = (value >> 21) & 0x7FF;
                let extended = value & 4 != 0;
                let rtr = value & 2 != 0;
                let txrq = value & 1 != 0;
                
                if txrq {
                    self.tx_mailboxes[0] = Some(CanFrame {
                        id: if extended { (value >> 3) & 0x1FFF_FFFF } else { id },
                        extended,
                        rtr,
                        dlc: 0,
                        data: [0; 8],
                        timestamp: self.current_cycles,
                    });
                    self.tsr &= !(1 << 26); // TME0 = 0
                }
            }
            // RX FIFO release
            0x0C => {
                if value & (1 << 5) != 0 { // RFOM0
                    self.rx_fifo0.pop_front();
                    self.rf0r = (self.rf0r & !0x3) | (self.rx_fifo0.len() as u32);
                }
            }
            0x10 => {
                if value & (1 << 5) != 0 { // RFOM1
                    self.rx_fifo1.pop_front();
                    self.rf1r = (self.rf1r & !0x3) | (self.rx_fifo1.len() as u32);
                }
            }
            _ => {}
        }
    }

    fn tick(&mut self, cycles: u64) {
        self.current_cycles += cycles;
        
        // Process pending TX in loopback mode
        if self.state == CanState::Loopback || self.state == CanState::SilentLoopback {
            for i in 0..3 {
                if let Some(frame) = self.tx_mailboxes[i].take() {
                    self.inject_rx(frame);
                    self.tsr |= 1 << (26 + i); // TMEx = 1
                }
            }
        }
    }

    fn get_interrupt(&self) -> Option<u8> {
        if self.irq_pending {
            Some(match self.instance {
                1 => 19, // CAN1_RX0
                2 => 63, // CAN2_RX0
                _ => 19,
            })
        } else {
            None
        }
    }

    fn clear_interrupt(&mut self) {
        self.irq_pending = false;
    }

    fn reset(&mut self) {
        self.mcr = 0x0001_0002;
        self.msr = 0x0000_0C02;
        self.tsr = 0x1C00_0000;
        self.rf0r = 0;
        self.rf1r = 0;
        self.rx_fifo0.clear();
        self.rx_fifo1.clear();
        self.tx_mailboxes = [None, None, None];
        self.state = CanState::Initialization;
    }

    fn get_state(&self) -> serde_json::Value {
        serde_json::json!({
            "instance": self.instance,
            "state": format!("{:?}", self.state),
            "rx_fifo0_count": self.rx_fifo0.len(),
            "rx_fifo1_count": self.rx_fifo1.len(),
            "tx_error_count": self.tx_error_count,
            "rx_error_count": self.rx_error_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_frame() {
        let frame = CanFrame::new_standard(0x123, &[0x11, 0x22, 0x33]);
        assert_eq!(frame.id, 0x123);
        assert_eq!(frame.dlc, 3);
        assert!(!frame.extended);
    }

    #[test]
    fn test_can_filter() {
        let filter = CanFilter {
            id: 0x100,
            mask: 0x700,
            extended: false,
            enabled: true,
            fifo: 0,
        };
        
        let frame1 = CanFrame::new_standard(0x123, &[]);
        let frame2 = CanFrame::new_standard(0x234, &[]);
        
        assert!(filter.matches(&frame1)); // 0x123 & 0x700 = 0x100
        assert!(!filter.matches(&frame2)); // 0x234 & 0x700 = 0x200
    }
}
