//! Debug Facilities
//!
//! Provides breakpoints, watchpoints, and debugging support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Breakpoint type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreakpointType {
    /// Software breakpoint (instruction replacement)
    Software,
    /// Hardware breakpoint
    Hardware,
}

/// Breakpoint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub id: u32,
    pub address: u32,
    pub bp_type: BreakpointType,
    pub enabled: bool,
    pub hit_count: u64,
    pub condition: Option<String>,
}

impl Breakpoint {
    pub fn new(id: u32, address: u32) -> Self {
        Self {
            id,
            address,
            bp_type: BreakpointType::Hardware,
            enabled: true,
            hit_count: 0,
            condition: None,
        }
    }
}

/// Watchpoint type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchpointType {
    /// Watch for read access
    Read,
    /// Watch for write access
    Write,
    /// Watch for read or write access
    ReadWrite,
}

/// Watchpoint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watchpoint {
    pub id: u32,
    pub address: u32,
    pub size: u8,
    pub wp_type: WatchpointType,
    pub enabled: bool,
    pub hit_count: u64,
}

impl Watchpoint {
    pub fn new(id: u32, address: u32, size: u8, wp_type: WatchpointType) -> Self {
        Self {
            id,
            address,
            size,
            wp_type,
            enabled: true,
            hit_count: 0,
        }
    }
}

/// Debugger for simulation
pub struct Debugger {
    /// Breakpoints by ID
    breakpoints: HashMap<u32, Breakpoint>,
    /// Breakpoints by address (for fast lookup)
    bp_by_addr: HashMap<u32, u32>,
    /// Watchpoints by ID
    watchpoints: HashMap<u32, Watchpoint>,
    /// Next breakpoint ID
    next_bp_id: u32,
    /// Next watchpoint ID
    next_wp_id: u32,
    /// Single-step mode
    single_step: bool,
    /// Trace buffer
    trace_buffer: Vec<TraceEntry>,
    /// Maximum trace buffer size
    max_trace_size: usize,
    /// Tracing enabled
    trace_enabled: bool,
}

/// Trace entry for execution history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEntry {
    pub pc: u32,
    pub instruction: u32,
    pub cycles: u64,
    pub timestamp_ns: u64,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
            bp_by_addr: HashMap::new(),
            watchpoints: HashMap::new(),
            next_bp_id: 1,
            next_wp_id: 1,
            single_step: false,
            trace_buffer: Vec::new(),
            max_trace_size: 10000,
            trace_enabled: false,
        }
    }

    /// Add a breakpoint
    pub fn add_breakpoint(&mut self, address: u32) -> u32 {
        let id = self.next_bp_id;
        self.next_bp_id += 1;
        
        let bp = Breakpoint::new(id, address);
        self.breakpoints.insert(id, bp);
        self.bp_by_addr.insert(address, id);
        
        id
    }

    /// Add a conditional breakpoint
    pub fn add_conditional_breakpoint(&mut self, address: u32, condition: String) -> u32 {
        let id = self.add_breakpoint(address);
        if let Some(bp) = self.breakpoints.get_mut(&id) {
            bp.condition = Some(condition);
        }
        id
    }

    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, id: u32) {
        if let Some(bp) = self.breakpoints.remove(&id) {
            self.bp_by_addr.remove(&bp.address);
        }
    }

    /// Enable/disable breakpoint
    pub fn set_breakpoint_enabled(&mut self, id: u32, enabled: bool) {
        if let Some(bp) = self.breakpoints.get_mut(&id) {
            bp.enabled = enabled;
        }
    }

    /// Check if address has a breakpoint
    pub fn check_breakpoint(&mut self, address: u32) -> Option<u32> {
        if let Some(&id) = self.bp_by_addr.get(&address) {
            if let Some(bp) = self.breakpoints.get_mut(&id) {
                if bp.enabled {
                    bp.hit_count += 1;
                    return Some(id);
                }
            }
        }
        None
    }

    /// Add a watchpoint
    pub fn add_watchpoint(&mut self, address: u32, size: u8, wp_type: WatchpointType) -> u32 {
        let id = self.next_wp_id;
        self.next_wp_id += 1;
        
        let wp = Watchpoint::new(id, address, size, wp_type);
        self.watchpoints.insert(id, wp);
        
        id
    }

    /// Remove a watchpoint
    pub fn remove_watchpoint(&mut self, id: u32) {
        self.watchpoints.remove(&id);
    }

    /// Check if address triggers a watchpoint
    pub fn check_watchpoint(&mut self, address: u32, is_write: bool) -> Option<u32> {
        for (id, wp) in self.watchpoints.iter_mut() {
            if !wp.enabled {
                continue;
            }
            
            let match_type = match wp.wp_type {
                WatchpointType::Read => !is_write,
                WatchpointType::Write => is_write,
                WatchpointType::ReadWrite => true,
            };
            
            if match_type && address >= wp.address && address < wp.address + wp.size as u32 {
                wp.hit_count += 1;
                return Some(*id);
            }
        }
        None
    }

    /// Get all breakpoints
    pub fn get_breakpoints(&self) -> Vec<&Breakpoint> {
        self.breakpoints.values().collect()
    }

    /// Get all watchpoints
    pub fn get_watchpoints(&self) -> Vec<&Watchpoint> {
        self.watchpoints.values().collect()
    }

    /// Enable single-step mode
    pub fn set_single_step(&mut self, enabled: bool) {
        self.single_step = enabled;
    }

    /// Check if in single-step mode
    pub fn is_single_step(&self) -> bool {
        self.single_step
    }

    /// Enable tracing
    pub fn enable_trace(&mut self, enabled: bool) {
        self.trace_enabled = enabled;
        if !enabled {
            self.trace_buffer.clear();
        }
    }

    /// Add trace entry
    pub fn trace(&mut self, pc: u32, instruction: u32, cycles: u64) {
        if !self.trace_enabled {
            return;
        }
        
        let entry = TraceEntry {
            pc,
            instruction,
            cycles,
            timestamp_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0),
        };
        
        self.trace_buffer.push(entry);
        
        // Limit buffer size
        if self.trace_buffer.len() > self.max_trace_size {
            self.trace_buffer.remove(0);
        }
    }

    /// Get trace buffer
    pub fn get_trace(&self) -> &[TraceEntry] {
        &self.trace_buffer
    }

    /// Clear trace buffer
    pub fn clear_trace(&mut self) {
        self.trace_buffer.clear();
    }

    /// Get debug state summary
    pub fn get_state(&self) -> DebugState {
        DebugState {
            breakpoint_count: self.breakpoints.len(),
            watchpoint_count: self.watchpoints.len(),
            single_step: self.single_step,
            trace_enabled: self.trace_enabled,
            trace_entries: self.trace_buffer.len(),
        }
    }
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}

/// Debug state summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugState {
    pub breakpoint_count: usize,
    pub watchpoint_count: usize,
    pub single_step: bool,
    pub trace_enabled: bool,
    pub trace_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_add_remove() {
        let mut dbg = Debugger::new();
        
        let id = dbg.add_breakpoint(0x0800_1000);
        assert!(dbg.check_breakpoint(0x0800_1000).is_some());
        
        dbg.remove_breakpoint(id);
        assert!(dbg.check_breakpoint(0x0800_1000).is_none());
    }

    #[test]
    fn test_watchpoint() {
        let mut dbg = Debugger::new();
        
        let id = dbg.add_watchpoint(0x2000_0100, 4, WatchpointType::Write);
        
        assert!(dbg.check_watchpoint(0x2000_0100, true).is_some());
        assert!(dbg.check_watchpoint(0x2000_0100, false).is_none());
        
        dbg.remove_watchpoint(id);
        assert!(dbg.check_watchpoint(0x2000_0100, true).is_none());
    }

    #[test]
    fn test_trace() {
        let mut dbg = Debugger::new();
        
        dbg.enable_trace(true);
        dbg.trace(0x0800_0000, 0xBF00, 1);
        dbg.trace(0x0800_0002, 0xBF00, 2);
        
        assert_eq!(dbg.get_trace().len(), 2);
    }
}
