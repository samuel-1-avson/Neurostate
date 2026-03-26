//! Interrupt Controller (NVIC) Simulation
//!
//! Simulates the Nested Vectored Interrupt Controller for ARM Cortex-M.

use serde::{Deserialize, Serialize};
use std::collections::BinaryHeap;
use std::cmp::Ordering;

/// Interrupt priority (lower = higher priority)
pub type InterruptPriority = u8;

/// Pending interrupt with priority
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct PendingInterrupt {
    vector: u8,
    priority: u8,
}

impl Ord for PendingInterrupt {
    fn cmp(&self, other: &Self) -> Ordering {
        // Lower priority value = higher priority (should come first)
        other.priority.cmp(&self.priority)
            .then_with(|| self.vector.cmp(&other.vector))
    }
}

impl PartialOrd for PendingInterrupt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Interrupt state
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct InterruptState {
    pub enabled: bool,
    pub pending: bool,
    pub active: bool,
    pub priority: u8,
}

/// NVIC Interrupt Controller
pub struct InterruptController {
    /// Interrupt enable registers (ISER)
    iser: [u32; 8],
    /// Interrupt clear-enable registers (ICER)
    icer: [u32; 8],
    /// Interrupt set-pending registers (ISPR)
    ispr: [u32; 8],
    /// Interrupt clear-pending registers (ICPR)
    icpr: [u32; 8],
    /// Interrupt active bit registers (IABR)
    iabr: [u32; 8],
    /// Interrupt priority registers (IPR)
    ipr: [u8; 240],
    /// Current active priority
    current_priority: u8,
    /// Priority grouping (for preemption)
    priority_grouping: u8,
    /// Pending interrupt heap (sorted by priority)
    pending_heap: BinaryHeap<PendingInterrupt>,
    /// Currently active interrupt
    active_vector: Option<u8>,
}

impl InterruptController {
    pub fn new() -> Self {
        Self {
            iser: [0; 8],
            icer: [0; 8],
            ispr: [0; 8],
            icpr: [0; 8],
            iabr: [0; 8],
            ipr: [0; 240],
            current_priority: 0xFF,
            priority_grouping: 0,
            pending_heap: BinaryHeap::new(),
            active_vector: None,
        }
    }

    /// Reset the interrupt controller
    pub fn reset(&mut self) {
        self.iser = [0; 8];
        self.icer = [0; 8];
        self.ispr = [0; 8];
        self.icpr = [0; 8];
        self.iabr = [0; 8];
        self.ipr = [0; 240];
        self.current_priority = 0xFF;
        self.pending_heap.clear();
        self.active_vector = None;
    }

    /// Set interrupt as pending
    pub fn set_pending(&mut self, vector: u8) {
        let reg = (vector / 32) as usize;
        let bit = vector % 32;
        
        if reg < 8 {
            self.ispr[reg] |= 1 << bit;
            
            // Check if enabled
            if self.iser[reg] & (1 << bit) != 0 {
                let priority = if (vector as usize) < self.ipr.len() {
                    self.ipr[vector as usize]
                } else {
                    0xFF
                };
                
                self.pending_heap.push(PendingInterrupt { vector, priority });
            }
        }
    }

    /// Clear interrupt pending
    pub fn clear_pending(&mut self, vector: u8) {
        let reg = (vector / 32) as usize;
        let bit = vector % 32;
        
        if reg < 8 {
            self.ispr[reg] &= !(1 << bit);
        }
    }

    /// Enable an interrupt
    pub fn enable(&mut self, vector: u8) {
        let reg = (vector / 32) as usize;
        let bit = vector % 32;
        
        if reg < 8 {
            self.iser[reg] |= 1 << bit;
        }
    }

    /// Disable an interrupt
    pub fn disable(&mut self, vector: u8) {
        let reg = (vector / 32) as usize;
        let bit = vector % 32;
        
        if reg < 8 {
            self.iser[reg] &= !(1 << bit);
        }
    }

    /// Set interrupt priority
    pub fn set_priority(&mut self, vector: u8, priority: u8) {
        if (vector as usize) < self.ipr.len() {
            self.ipr[vector as usize] = priority;
        }
    }

    /// Get interrupt priority
    pub fn get_priority(&self, vector: u8) -> u8 {
        if (vector as usize) < self.ipr.len() {
            self.ipr[vector as usize]
        } else {
            0xFF
        }
    }

    /// Check if interrupt is enabled
    pub fn is_enabled(&self, vector: u8) -> bool {
        let reg = (vector / 32) as usize;
        let bit = vector % 32;
        
        if reg < 8 {
            self.iser[reg] & (1 << bit) != 0
        } else {
            false
        }
    }

    /// Get highest priority pending interrupt
    pub fn get_pending(&mut self) -> Option<u8> {
        // Rebuild heap from pending flags (simplified approach)
        self.rebuild_pending_heap();
        
        while let Some(irq) = self.pending_heap.pop() {
            let reg = (irq.vector / 32) as usize;
            let bit = irq.vector % 32;
            
            // Check if still pending and enabled
            if reg < 8 && 
               self.ispr[reg] & (1 << bit) != 0 &&
               self.iser[reg] & (1 << bit) != 0 &&
               irq.priority < self.current_priority {
                return Some(irq.vector);
            }
        }
        
        None
    }

    /// Acknowledge interrupt (start handling)
    pub fn acknowledge(&mut self, vector: u8) {
        let reg = (vector / 32) as usize;
        let bit = vector % 32;
        
        if reg < 8 {
            // Clear pending, set active
            self.ispr[reg] &= !(1 << bit);
            self.iabr[reg] |= 1 << bit;
            
            // Update current priority
            self.current_priority = self.get_priority(vector);
            self.active_vector = Some(vector);
        }
    }

    /// Complete interrupt handling
    pub fn complete(&mut self, vector: u8) {
        let reg = (vector / 32) as usize;
        let bit = vector % 32;
        
        if reg < 8 {
            self.iabr[reg] &= !(1 << bit);
            
            if self.active_vector == Some(vector) {
                self.active_vector = None;
                self.current_priority = 0xFF;
            }
        }
    }

    /// Rebuild pending interrupt heap
    fn rebuild_pending_heap(&mut self) {
        self.pending_heap.clear();
        
        for reg in 0..8 {
            let pending = self.ispr[reg] & self.iser[reg];
            
            for bit in 0..32 {
                if pending & (1 << bit) != 0 {
                    let vector = (reg * 32 + bit) as u8;
                    let priority = self.get_priority(vector);
                    self.pending_heap.push(PendingInterrupt { vector, priority });
                }
            }
        }
    }

    /// Read NVIC register
    pub fn read(&self, offset: u32) -> u32 {
        match offset {
            0x100..=0x11F => {
                let idx = ((offset - 0x100) / 4) as usize;
                if idx < 8 { self.iser[idx] } else { 0 }
            }
            0x180..=0x19F => {
                let idx = ((offset - 0x180) / 4) as usize;
                if idx < 8 { self.iser[idx] } else { 0 } // ICER reads same as ISER
            }
            0x200..=0x21F => {
                let idx = ((offset - 0x200) / 4) as usize;
                if idx < 8 { self.ispr[idx] } else { 0 }
            }
            0x280..=0x29F => {
                let idx = ((offset - 0x280) / 4) as usize;
                if idx < 8 { self.ispr[idx] } else { 0 }
            }
            0x300..=0x31F => {
                let idx = ((offset - 0x300) / 4) as usize;
                if idx < 8 { self.iabr[idx] } else { 0 }
            }
            0x400..=0x4EF => {
                let idx = (offset - 0x400) as usize;
                if idx + 3 < self.ipr.len() {
                    u32::from_le_bytes([
                        self.ipr[idx],
                        self.ipr[idx + 1],
                        self.ipr[idx + 2],
                        self.ipr[idx + 3],
                    ])
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    /// Write NVIC register
    pub fn write(&mut self, offset: u32, value: u32) {
        match offset {
            0x100..=0x11F => {
                let idx = ((offset - 0x100) / 4) as usize;
                if idx < 8 {
                    self.iser[idx] |= value; // ISER: write 1 to enable
                }
            }
            0x180..=0x19F => {
                let idx = ((offset - 0x180) / 4) as usize;
                if idx < 8 {
                    self.iser[idx] &= !value; // ICER: write 1 to disable
                }
            }
            0x200..=0x21F => {
                let idx = ((offset - 0x200) / 4) as usize;
                if idx < 8 {
                    self.ispr[idx] |= value; // ISPR: write 1 to set pending
                }
            }
            0x280..=0x29F => {
                let idx = ((offset - 0x280) / 4) as usize;
                if idx < 8 {
                    self.ispr[idx] &= !value; // ICPR: write 1 to clear pending
                }
            }
            0x400..=0x4EF => {
                let idx = (offset - 0x400) as usize;
                let bytes = value.to_le_bytes();
                for (i, byte) in bytes.iter().enumerate() {
                    if idx + i < self.ipr.len() {
                        self.ipr[idx + i] = *byte;
                    }
                }
            }
            _ => {}
        }
    }

    /// Get interrupt state for display
    pub fn get_state(&self, vector: u8) -> InterruptState {
        let reg = (vector / 32) as usize;
        let bit = vector % 32;
        
        if reg < 8 {
            InterruptState {
                enabled: self.iser[reg] & (1 << bit) != 0,
                pending: self.ispr[reg] & (1 << bit) != 0,
                active: self.iabr[reg] & (1 << bit) != 0,
                priority: self.get_priority(vector),
            }
        } else {
            InterruptState::default()
        }
    }
}

impl Default for InterruptController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interrupt_enable() {
        let mut nvic = InterruptController::new();
        
        nvic.enable(25);
        assert!(nvic.is_enabled(25));
        
        nvic.disable(25);
        assert!(!nvic.is_enabled(25));
    }

    #[test]
    fn test_interrupt_pending() {
        let mut nvic = InterruptController::new();
        
        nvic.enable(10);
        nvic.set_priority(10, 0x40);
        nvic.set_pending(10);
        
        let pending = nvic.get_pending();
        assert_eq!(pending, Some(10));
    }

    #[test]
    fn test_interrupt_priority() {
        let mut nvic = InterruptController::new();
        
        nvic.enable(10);
        nvic.enable(20);
        
        nvic.set_priority(10, 0x80); // Lower priority
        nvic.set_priority(20, 0x40); // Higher priority
        
        nvic.set_pending(10);
        nvic.set_pending(20);
        
        // Should get higher priority (lower value) first
        assert_eq!(nvic.get_pending(), Some(20));
    }
}
