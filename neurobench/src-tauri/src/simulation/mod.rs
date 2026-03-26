//! Embedded Systems Simulation Engine
//! 
//! A comprehensive, high-performance simulation engine for embedded systems.
//! Supports cycle-accurate CPU emulation, peripheral simulation, and debugging.
//!
//! ## Features
//! - Multi-architecture CPU emulation (ARM Cortex-M, RISC-V, Xtensa)
//! - Complete peripheral simulation (GPIO, UART, SPI, I2C, Timers, ADC, DMA)
//! - Clock tree and power management
//! - NVIC interrupt controller
//! - QEMU integration for advanced emulation
//! - Real-time debugging facilities

pub mod engine;
pub mod cpu;
pub mod memory;
pub mod clock;
pub mod interrupts;
pub mod debug;
pub mod peripherals;
pub mod mcu;
pub mod qemu_bridge;
pub mod power;
pub mod rtos;
pub mod sensors;
pub mod devices;
pub mod power_estimation;
pub mod bus_timing;
pub mod fsm_behavior;

// Re-exports for convenience
pub use engine::{SimulationEngine, SimulationConfig, SimulationState, StepResult, RunResult};
pub use cpu::{CpuCore, CpuArchitecture, RegisterFile};
pub use memory::{MemoryController, MemoryRegion, MemoryAccess};
pub use clock::{ClockTree, ClockSource, PllConfig};
pub use interrupts::{InterruptController, InterruptPriority};
pub use debug::{Debugger, Breakpoint, Watchpoint};
pub use peripherals::PeripheralBus;
pub use mcu::McuSimulation;
pub use power::{PowerController, PowerMode};
pub use rtos::RtosScheduler;
pub use sensors::SensorSimulator;
pub use devices::{CharacterLcd, ServoMotor, DcMotor, RgbLed};
pub use power_estimation::PowerEstimator;
pub use bus_timing::BusTimingAnalyzer;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Simulation event for async communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimulationEvent {
    /// CPU executed an instruction
    CpuStep { pc: u32, cycles: u64 },
    /// Peripheral state changed
    PeripheralUpdate { name: String, data: serde_json::Value },
    /// Interrupt triggered
    InterruptTriggered { vector: u8, priority: u8 },
    /// Breakpoint hit
    BreakpointHit { address: u32, id: u32 },
    /// Watchpoint triggered
    WatchpointTriggered { address: u32, value: u32 },
    /// Simulation state changed
    StateChanged(SimulationState),
    /// Error occurred
    Error(String),
    /// UART data received
    UartData { instance: u8, data: Vec<u8> },
    /// GPIO state changed
    GpioChange { port: char, pin: u8, state: bool },
}

/// Command to control simulation from frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimulationCommand {
    /// Start simulation
    Start,
    /// Stop simulation
    Stop,
    /// Pause simulation
    Pause,
    /// Resume simulation
    Resume,
    /// Step single instruction
    Step,
    /// Run for N cycles
    RunCycles(u64),
    /// Reset simulation
    Reset,
    /// Load firmware
    LoadFirmware(Vec<u8>),
    /// Set breakpoint
    SetBreakpoint { address: u32 },
    /// Remove breakpoint
    RemoveBreakpoint { id: u32 },
    /// Inject GPIO input
    InjectGpio { port: char, pin: u8, state: bool },
    /// Inject UART data
    InjectUart { instance: u8, data: Vec<u8> },
    /// Get current state
    GetState,
    /// Configure clock
    ConfigureClock(ClockConfig),
}

/// Clock configuration for simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockConfig {
    pub hse_frequency: Option<u32>,
    pub pll_enabled: bool,
    pub pll_multiplier: u8,
    pub pll_divider: u8,
    pub ahb_prescaler: u8,
    pub apb1_prescaler: u8,
    pub apb2_prescaler: u8,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            hse_frequency: Some(8_000_000), // 8 MHz external crystal
            pll_enabled: true,
            pll_multiplier: 168 / 8,
            pll_divider: 2,
            ahb_prescaler: 1,
            apb1_prescaler: 4,
            apb2_prescaler: 2,
        }
    }
}

/// Simulation handle for async control
pub struct SimulationHandle {
    command_tx: mpsc::Sender<SimulationCommand>,
    event_rx: Arc<RwLock<mpsc::Receiver<SimulationEvent>>>,
}

impl SimulationHandle {
    /// Send a command to the simulation
    pub async fn send_command(&self, cmd: SimulationCommand) -> Result<(), String> {
        self.command_tx.send(cmd).await
            .map_err(|e| format!("Failed to send command: {}", e))
    }

    /// Receive next event from simulation
    pub async fn recv_event(&self) -> Option<SimulationEvent> {
        self.event_rx.write().await.recv().await
    }
}

/// Create a new simulation with async channels
pub fn create_simulation(config: SimulationConfig) -> (SimulationEngine, SimulationHandle) {
    let (cmd_tx, cmd_rx) = mpsc::channel(256);
    let (evt_tx, evt_rx) = mpsc::channel(1024);

    let engine = SimulationEngine::new(config, cmd_rx, evt_tx);
    let handle = SimulationHandle {
        command_tx: cmd_tx,
        event_rx: Arc::new(RwLock::new(evt_rx)),
    };

    (engine, handle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // Verify module compiles correctly
        assert!(true);
    }
}
