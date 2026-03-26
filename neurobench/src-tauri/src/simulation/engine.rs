//! Simulation Engine Core
//!
//! The main simulation controller that orchestrates CPU, memory, peripherals,
//! and timing. Runs in a separate async task for non-blocking operation.

use super::cpu::{CpuCore, CpuArchitecture};
use super::memory::MemoryController;
use super::peripherals::PeripheralBus;
use super::clock::ClockTree;
use super::interrupts::InterruptController;
use super::debug::Debugger;
use super::{SimulationCommand, SimulationEvent, ClockConfig};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;

/// Simulation state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimulationState {
    /// Not started
    Idle,
    /// Currently running
    Running,
    /// Paused (can resume)
    Paused,
    /// Stopped (must reset to continue)
    Stopped,
    /// Breakpoint hit
    AtBreakpoint,
    /// Error state
    Error,
}

impl Default for SimulationState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Result of a single step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepResult {
    /// Normal execution
    Ok { cycles: u64, pc: u32 },
    /// Hit a breakpoint
    Breakpoint { address: u32, id: u32 },
    /// Watchpoint triggered
    Watchpoint { address: u32, id: u32 },
    /// CPU halted (WFI/WFE)
    Halted,
    /// Error during execution
    Error(String),
}

/// Result of running multiple cycles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    pub cycles_executed: u64,
    pub instructions_executed: u64,
    pub stop_reason: StopReason,
}

/// Reason for stopping execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StopReason {
    /// Ran requested number of cycles
    CycleLimit,
    /// User requested stop
    UserStop,
    /// Hit breakpoint
    Breakpoint { address: u32, id: u32 },
    /// CPU fault
    Fault(String),
    /// CPU halted
    Halted,
}

/// Simulation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Target MCU identifier
    pub mcu: String,
    /// CPU architecture
    pub architecture: CpuArchitecture,
    /// Flash memory size in bytes
    pub flash_size: usize,
    /// RAM size in bytes
    pub ram_size: usize,
    /// Flash base address
    pub flash_base: u32,
    /// RAM base address
    pub ram_base: u32,
    /// System clock frequency
    pub system_clock: u32,
    /// Enable cycle-accurate timing
    pub cycle_accurate: bool,
    /// Clock configuration
    pub clock_config: Option<ClockConfig>,
    /// QEMU integration mode
    pub use_qemu: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            mcu: "STM32F401".to_string(),
            architecture: CpuArchitecture::CortexM4,
            flash_size: 512 * 1024,  // 512KB
            ram_size: 96 * 1024,     // 96KB
            flash_base: 0x0800_0000,
            ram_base: 0x2000_0000,
            system_clock: 84_000_000, // 84 MHz
            cycle_accurate: true,
            clock_config: Some(ClockConfig::default()),
            use_qemu: false,
        }
    }
}

/// CPU and peripheral snapshot for save/restore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub cpu_state: CpuSnapshot,
    pub memory_hash: u64,
    pub peripheral_states: HashMap<String, serde_json::Value>,
    pub cycle_count: u64,
}

/// CPU state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuSnapshot {
    pub registers: [u32; 16],
    pub pc: u32,
    pub sp: u32,
    pub lr: u32,
    pub xpsr: u32,
    pub primask: bool,
    pub basepri: u8,
    pub faultmask: bool,
    pub control: u8,
}

/// The main simulation engine
pub struct SimulationEngine {
    /// CPU core
    cpu: CpuCore,
    /// Memory controller
    memory: MemoryController,
    /// Peripheral bus
    peripherals: PeripheralBus,
    /// Clock tree
    clock: ClockTree,
    /// Interrupt controller
    interrupts: InterruptController,
    /// Debugger
    debugger: Debugger,
    /// Configuration
    config: SimulationConfig,
    /// Current state
    state: SimulationState,
    /// Total cycles executed
    cycle_count: u64,
    /// Total instructions executed
    instruction_count: u64,
    /// Command receiver
    command_rx: mpsc::Receiver<SimulationCommand>,
    /// Event sender
    event_tx: mpsc::Sender<SimulationEvent>,
}

impl SimulationEngine {
    /// Create a new simulation engine
    pub fn new(
        config: SimulationConfig,
        command_rx: mpsc::Receiver<SimulationCommand>,
        event_tx: mpsc::Sender<SimulationEvent>,
    ) -> Self {
        let cpu = CpuCore::new(config.architecture);
        let memory = MemoryController::new(&config);
        let peripherals = PeripheralBus::new(&config);
        let clock = ClockTree::new(config.clock_config.clone().unwrap_or_default());
        let interrupts = InterruptController::new();
        let debugger = Debugger::new();

        Self {
            cpu,
            memory,
            peripherals,
            clock,
            interrupts,
            debugger,
            config,
            state: SimulationState::Idle,
            cycle_count: 0,
            instruction_count: 0,
            command_rx,
            event_tx,
        }
    }

    /// Run the simulation loop (async)
    pub async fn run_loop(&mut self) {
        loop {
            // Process commands
            match self.command_rx.try_recv() {
                Ok(cmd) => {
                    if !self.handle_command(cmd).await {
                        break; // Exit requested
                    }
                }
                Err(mpsc::error::TryRecvError::Empty) => {}
                Err(mpsc::error::TryRecvError::Disconnected) => break,
            }

            // Execute simulation if running
            if self.state == SimulationState::Running {
                match self.step_internal() {
                    StepResult::Ok { cycles, pc } => {
                        self.cycle_count += cycles;
                        self.instruction_count += 1;

                        // Emit event periodically (every 1000 instructions)
                        if self.instruction_count % 1000 == 0 {
                            let _ = self.event_tx.send(SimulationEvent::CpuStep {
                                pc,
                                cycles: self.cycle_count,
                            }).await;
                        }
                    }
                    StepResult::Breakpoint { address, id } => {
                        self.state = SimulationState::AtBreakpoint;
                        let _ = self.event_tx.send(SimulationEvent::BreakpointHit { address, id }).await;
                    }
                    StepResult::Halted => {
                        // CPU in WFI/WFE, check for interrupts
                        if let Some(irq) = self.interrupts.get_pending() {
                            self.handle_interrupt(irq);
                        } else {
                            // Yield to allow other tasks
                            tokio::task::yield_now().await;
                        }
                    }
                    StepResult::Error(e) => {
                        self.state = SimulationState::Error;
                        let _ = self.event_tx.send(SimulationEvent::Error(e)).await;
                    }
                    StepResult::Watchpoint { address, id } => {
                        self.state = SimulationState::AtBreakpoint;
                        let _ = self.event_tx.send(SimulationEvent::WatchpointTriggered { address, value: id }).await;
                    }
                }
            } else {
                // Not running, wait for commands
                tokio::task::yield_now().await;
            }
        }
    }

    /// Handle a simulation command
    async fn handle_command(&mut self, cmd: SimulationCommand) -> bool {
        match cmd {
            SimulationCommand::Start => {
                self.state = SimulationState::Running;
                let _ = self.event_tx.send(SimulationEvent::StateChanged(self.state)).await;
            }
            SimulationCommand::Stop => {
                self.state = SimulationState::Stopped;
                let _ = self.event_tx.send(SimulationEvent::StateChanged(self.state)).await;
                return false; // Exit loop
            }
            SimulationCommand::Pause => {
                self.state = SimulationState::Paused;
                let _ = self.event_tx.send(SimulationEvent::StateChanged(self.state)).await;
            }
            SimulationCommand::Resume => {
                if self.state == SimulationState::Paused || self.state == SimulationState::AtBreakpoint {
                    self.state = SimulationState::Running;
                    let _ = self.event_tx.send(SimulationEvent::StateChanged(self.state)).await;
                }
            }
            SimulationCommand::Step => {
                let result = self.step();
                match &result {
                    StepResult::Ok { pc, cycles } => {
                        let _ = self.event_tx.send(SimulationEvent::CpuStep {
                            pc: *pc,
                            cycles: *cycles,
                        }).await;
                    }
                    StepResult::Breakpoint { address, id } => {
                        let _ = self.event_tx.send(SimulationEvent::BreakpointHit {
                            address: *address,
                            id: *id,
                        }).await;
                    }
                    _ => {}
                }
            }
            SimulationCommand::RunCycles(cycles) => {
                let result = self.run(cycles);
                let _ = self.event_tx.send(SimulationEvent::CpuStep {
                    pc: self.cpu.pc(),
                    cycles: result.cycles_executed,
                }).await;
            }
            SimulationCommand::Reset => {
                self.reset();
                let _ = self.event_tx.send(SimulationEvent::StateChanged(SimulationState::Idle)).await;
            }
            SimulationCommand::LoadFirmware(data) => {
                if let Err(e) = self.load_firmware(&data) {
                    let _ = self.event_tx.send(SimulationEvent::Error(e)).await;
                }
            }
            SimulationCommand::SetBreakpoint { address } => {
                let id = self.debugger.add_breakpoint(address);
                let _ = self.event_tx.send(SimulationEvent::BreakpointHit { address, id }).await;
            }
            SimulationCommand::RemoveBreakpoint { id } => {
                self.debugger.remove_breakpoint(id);
            }
            SimulationCommand::InjectGpio { port, pin, state } => {
                self.peripherals.gpio_inject(port, pin, state);
                let _ = self.event_tx.send(SimulationEvent::GpioChange { port, pin, state }).await;
            }
            SimulationCommand::InjectUart { instance, data } => {
                self.peripherals.uart_inject(instance, &data);
                let _ = self.event_tx.send(SimulationEvent::UartData { instance, data }).await;
            }
            SimulationCommand::GetState => {
                let _ = self.event_tx.send(SimulationEvent::StateChanged(self.state)).await;
            }
            SimulationCommand::ConfigureClock(config) => {
                self.clock = ClockTree::new(config);
            }
        }
        true
    }

    /// Execute a single instruction
    pub fn step(&mut self) -> StepResult {
        self.step_internal()
    }

    /// Internal step implementation
    fn step_internal(&mut self) -> StepResult {
        // Check for breakpoints
        let pc = self.cpu.pc();
        if let Some(bp_id) = self.debugger.check_breakpoint(pc) {
            return StepResult::Breakpoint { address: pc, id: bp_id };
        }

        // Fetch instruction
        let instruction = match self.memory.read32(pc) {
            Ok(instr) => instr,
            Err(e) => return StepResult::Error(format!("Fetch error at 0x{:08X}: {}", pc, e)),
        };

        // Decode and execute
        let cycles = match self.cpu.execute(instruction, &mut self.memory, &mut self.peripherals) {
            Ok(c) => c,
            Err(e) => return StepResult::Error(format!("Execution error: {}", e)),
        };

        // Tick peripherals
        self.peripherals.tick(cycles);

        // Check for interrupts
        if let Some(irq) = self.interrupts.get_pending() {
            self.handle_interrupt(irq);
        }

        StepResult::Ok { cycles, pc: self.cpu.pc() }
    }

    /// Run for specified number of cycles
    pub fn run(&mut self, max_cycles: u64) -> RunResult {
        let start_cycles = self.cycle_count;
        let start_instructions = self.instruction_count;

        while self.cycle_count - start_cycles < max_cycles {
            match self.step_internal() {
                StepResult::Ok { cycles, .. } => {
                    self.cycle_count += cycles;
                    self.instruction_count += 1;
                }
                StepResult::Breakpoint { address, id } => {
                    return RunResult {
                        cycles_executed: self.cycle_count - start_cycles,
                        instructions_executed: self.instruction_count - start_instructions,
                        stop_reason: StopReason::Breakpoint { address, id },
                    };
                }
                StepResult::Halted => {
                    return RunResult {
                        cycles_executed: self.cycle_count - start_cycles,
                        instructions_executed: self.instruction_count - start_instructions,
                        stop_reason: StopReason::Halted,
                    };
                }
                StepResult::Error(e) => {
                    return RunResult {
                        cycles_executed: self.cycle_count - start_cycles,
                        instructions_executed: self.instruction_count - start_instructions,
                        stop_reason: StopReason::Fault(e),
                    };
                }
                StepResult::Watchpoint { address, id } => {
                    return RunResult {
                        cycles_executed: self.cycle_count - start_cycles,
                        instructions_executed: self.instruction_count - start_instructions,
                        stop_reason: StopReason::Breakpoint { address, id },
                    };
                }
            }
        }

        RunResult {
            cycles_executed: self.cycle_count - start_cycles,
            instructions_executed: self.instruction_count - start_instructions,
            stop_reason: StopReason::CycleLimit,
        }
    }

    /// Pause simulation
    pub fn pause(&mut self) {
        self.state = SimulationState::Paused;
    }

    /// Reset simulation
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.memory.reset();
        self.peripherals.reset();
        self.interrupts.reset();
        self.cycle_count = 0;
        self.instruction_count = 0;
        self.state = SimulationState::Idle;
    }

    /// Load firmware into flash
    pub fn load_firmware(&mut self, data: &[u8]) -> Result<(), String> {
        self.memory.load_firmware(data, self.config.flash_base)
    }

    /// Create snapshot of current state
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            cpu_state: self.cpu.snapshot(),
            memory_hash: self.memory.hash(),
            peripheral_states: self.peripherals.snapshot(),
            cycle_count: self.cycle_count,
        }
    }

    /// Restore from snapshot
    pub fn restore(&mut self, snapshot: Snapshot) {
        self.cpu.restore(&snapshot.cpu_state);
        self.peripherals.restore(&snapshot.peripheral_states);
        self.cycle_count = snapshot.cycle_count;
    }

    /// Handle interrupt
    fn handle_interrupt(&mut self, irq: u8) {
        // Save context and jump to handler
        self.cpu.enter_exception(irq as u32);
        self.interrupts.acknowledge(irq);
    }

    /// Get current state
    pub fn state(&self) -> SimulationState {
        self.state
    }

    /// Get cycle count
    pub fn cycle_count(&self) -> u64 {
        self.cycle_count
    }

    /// Get CPU reference
    pub fn cpu(&self) -> &CpuCore {
        &self.cpu
    }

    /// Get memory reference
    pub fn memory(&self) -> &MemoryController {
        &self.memory
    }

    /// Get peripheral reference
    pub fn peripherals(&self) -> &PeripheralBus {
        &self.peripherals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_config_default() {
        let config = SimulationConfig::default();
        assert_eq!(config.mcu, "STM32F401");
        assert_eq!(config.flash_size, 512 * 1024);
        assert_eq!(config.ram_size, 96 * 1024);
    }

    #[test]
    fn test_simulation_state_default() {
        let state = SimulationState::default();
        assert_eq!(state, SimulationState::Idle);
    }
}
