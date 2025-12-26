// Debug Module
//
// Provides GDB-like debugging capabilities via probe-rs:
// - Session management (start/stop)
// - Execution control (step, continue, pause)
// - Breakpoint management
// - Variable inspection
// - Call stack reading

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

// ==================== Types ====================

/// Debug session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DebugState {
    Stopped,     // Not running
    Running,     // Target executing
    Halted,      // Target halted (breakpoint, step, pause)
    Disconnected,
}

/// Breakpoint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub id: u32,
    pub file: String,
    pub line: u32,
    pub address: Option<u64>,
    pub enabled: bool,
    pub hit_count: u32,
    pub condition: Option<String>,
}

/// Stack frame info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub index: u32,
    pub name: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub address: u64,
}

/// Variable/watch info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub value: String,
    pub var_type: String,
    pub address: Option<u64>,
    pub children: Option<Vec<Variable>>,
}

/// CPU registers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registers {
    pub pc: u64,
    pub sp: u64,
    pub lr: u64,
    pub general: HashMap<String, u64>,
}

/// Debug session status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugStatus {
    pub state: DebugState,
    pub chip: String,
    pub elf_path: Option<String>,
    pub current_address: Option<u64>,
    pub current_file: Option<String>,
    pub current_line: Option<u32>,
    pub breakpoints_hit: Vec<u32>,
}

/// Step type for execution control
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    Into,   // Step into functions
    Over,   // Step over function calls
    Out,    // Step out of current function
}

// ==================== Mock Debug Session ====================

/// Debug session manager (mock for now, would use probe-rs in real impl)
pub struct DebugSession {
    state: Arc<RwLock<DebugState>>,
    chip: String,
    elf_path: Option<String>,
    breakpoints: Arc<RwLock<Vec<Breakpoint>>>,
    next_bp_id: Arc<RwLock<u32>>,
    status_tx: Option<mpsc::Sender<DebugStatus>>,
}

impl DebugSession {
    pub fn new(chip: &str, elf_path: Option<&str>) -> Self {
        Self {
            state: Arc::new(RwLock::new(DebugState::Stopped)),
            chip: chip.to_string(),
            elf_path: elf_path.map(|s| s.to_string()),
            breakpoints: Arc::new(RwLock::new(Vec::new())),
            next_bp_id: Arc::new(RwLock::new(1)),
            status_tx: None,
        }
    }
    
    /// Start debug session
    pub async fn start(&self) -> Result<(), String> {
        // In real impl: connect to probe, load ELF symbols
        let mut state = self.state.write().await;
        *state = DebugState::Halted;  // Start halted at reset
        Ok(())
    }
    
    /// Stop debug session
    pub async fn stop(&self) -> Result<(), String> {
        let mut state = self.state.write().await;
        *state = DebugState::Disconnected;
        Ok(())
    }
    
    /// Continue execution
    pub async fn resume(&self) -> Result<(), String> {
        let mut state = self.state.write().await;
        if *state != DebugState::Halted {
            return Err("Target not halted".to_string());
        }
        *state = DebugState::Running;
        Ok(())
    }
    
    /// Pause execution
    pub async fn pause(&self) -> Result<(), String> {
        let mut state = self.state.write().await;
        if *state != DebugState::Running {
            return Err("Target not running".to_string());
        }
        *state = DebugState::Halted;
        Ok(())
    }
    
    /// Step execution
    pub async fn step(&self, step_type: StepType) -> Result<DebugStatus, String> {
        let state = self.state.read().await;
        if *state != DebugState::Halted {
            return Err("Target not halted".to_string());
        }
        drop(state);
        
        // In real impl: perform step instruction
        // For now, just return current status
        self.get_status().await
    }
    
    /// Add breakpoint
    pub async fn add_breakpoint(&self, file: &str, line: u32) -> Result<Breakpoint, String> {
        let mut bps = self.breakpoints.write().await;
        let mut next_id = self.next_bp_id.write().await;
        
        let bp = Breakpoint {
            id: *next_id,
            file: file.to_string(),
            line,
            address: None,  // Would resolve from ELF symbols
            enabled: true,
            hit_count: 0,
            condition: None,
        };
        
        *next_id += 1;
        bps.push(bp.clone());
        Ok(bp)
    }
    
    /// Remove breakpoint
    pub async fn remove_breakpoint(&self, id: u32) -> Result<bool, String> {
        let mut bps = self.breakpoints.write().await;
        let len_before = bps.len();
        bps.retain(|bp| bp.id != id);
        Ok(bps.len() != len_before)
    }
    
    /// List breakpoints
    pub async fn list_breakpoints(&self) -> Vec<Breakpoint> {
        self.breakpoints.read().await.clone()
    }
    
    /// Get call stack
    pub async fn get_stack(&self) -> Result<Vec<StackFrame>, String> {
        // Mock stack for now
        Ok(vec![
            StackFrame {
                index: 0,
                name: "main".to_string(),
                file: Some("src/main.c".to_string()),
                line: Some(42),
                address: 0x0800_0100,
            },
        ])
    }
    
    /// Get registers
    pub async fn get_registers(&self) -> Result<Registers, String> {
        // Mock registers
        let mut general = HashMap::new();
        for i in 0..13 {
            general.insert(format!("r{}", i), 0);
        }
        
        Ok(Registers {
            pc: 0x0800_0100,
            sp: 0x2001_FFF0,
            lr: 0x0800_0050,
            general,
        })
    }
    
    /// Read variable
    pub async fn read_variable(&self, name: &str) -> Result<Variable, String> {
        // Mock variable reading
        Ok(Variable {
            name: name.to_string(),
            value: "0".to_string(),
            var_type: "int".to_string(),
            address: Some(0x2000_0000),
            children: None,
        })
    }
    
    /// Get current status
    pub async fn get_status(&self) -> Result<DebugStatus, String> {
        let state = *self.state.read().await;
        
        Ok(DebugStatus {
            state,
            chip: self.chip.clone(),
            elf_path: self.elf_path.clone(),
            current_address: Some(0x0800_0100),
            current_file: Some("src/main.c".to_string()),
            current_line: Some(42),
            breakpoints_hit: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_debug_session_lifecycle() {
        let session = DebugSession::new("STM32F407VG", Some("firmware.elf"));
        
        // Start session
        session.start().await.unwrap();
        let status = session.get_status().await.unwrap();
        assert_eq!(status.state, DebugState::Halted);
        
        // Resume
        session.resume().await.unwrap();
        let status = session.get_status().await.unwrap();
        assert_eq!(status.state, DebugState::Running);
        
        // Pause
        session.pause().await.unwrap();
        let status = session.get_status().await.unwrap();
        assert_eq!(status.state, DebugState::Halted);
        
        // Stop
        session.stop().await.unwrap();
        let status = session.get_status().await.unwrap();
        assert_eq!(status.state, DebugState::Disconnected);
    }
    
    #[tokio::test]
    async fn test_breakpoints() {
        let session = DebugSession::new("STM32F407VG", None);
        
        // Add breakpoint
        let bp = session.add_breakpoint("main.c", 10).await.unwrap();
        assert_eq!(bp.id, 1);
        assert_eq!(bp.line, 10);
        
        // List breakpoints
        let bps = session.list_breakpoints().await;
        assert_eq!(bps.len(), 1);
        
        // Remove breakpoint
        let removed = session.remove_breakpoint(1).await.unwrap();
        assert!(removed);
        
        let bps = session.list_breakpoints().await;
        assert_eq!(bps.len(), 0);
    }
}
