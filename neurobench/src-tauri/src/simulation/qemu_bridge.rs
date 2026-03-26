//! QEMU Bridge
//!
//! Integration with QEMU for advanced emulation of ESP32 and other complex MCUs.

use super::engine::{SimulationConfig, SimulationState};
use super::{SimulationEvent, SimulationCommand};

use serde::{Deserialize, Serialize};
use std::process::{Child, Command, Stdio};
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// QEMU machine type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QemuMachine {
    /// STM32 LM3S6965 (Cortex-M3)
    Lm3s6965evb,
    /// Netduino 2 (STM32F205)
    Netduino2,
    /// MPS2 AN385 (Cortex-M3)
    Mps2An385,
    /// ESP32 (requires qemu-system-xtensa)
    Esp32,
    /// Custom machine string
    Custom(String),
}

impl QemuMachine {
    pub fn to_qemu_arg(&self) -> &str {
        match self {
            Self::Lm3s6965evb => "lm3s6965evb",
            Self::Netduino2 => "netduino2",
            Self::Mps2An385 => "mps2-an385",
            Self::Esp32 => "esp32",
            Self::Custom(s) => s,
        }
    }

    pub fn qemu_binary(&self) -> &str {
        match self {
            Self::Esp32 => "qemu-system-xtensa",
            _ => "qemu-system-arm",
        }
    }

    pub fn cpu(&self) -> Option<&str> {
        match self {
            Self::Mps2An385 => Some("cortex-m3"),
            Self::Esp32 => Some("esp32"),
            _ => None,
        }
    }
}

/// QEMU bridge configuration
#[derive(Debug, Clone)]
pub struct QemuBridgeConfig {
    pub machine: QemuMachine,
    pub firmware_path: Option<String>,
    pub kernel_path: Option<String>,
    pub gdb_port: Option<u16>,
    pub serial_port: Option<u16>,
    pub extra_args: Vec<String>,
}

impl Default for QemuBridgeConfig {
    fn default() -> Self {
        Self {
            machine: QemuMachine::Lm3s6965evb,
            firmware_path: None,
            kernel_path: None,
            gdb_port: Some(1234),
            serial_port: Some(4444),
            extra_args: Vec::new(),
        }
    }
}

/// QEMU Bridge for advanced emulation
pub struct QemuBridge {
    config: QemuBridgeConfig,
    process: Option<Child>,
    state: Arc<Mutex<QemuState>>,
    event_tx: Option<mpsc::Sender<SimulationEvent>>,
}

#[derive(Debug, Clone, Default)]
struct QemuState {
    running: bool,
    output_lines: Vec<String>,
    error: Option<String>,
}

impl QemuBridge {
    pub fn new(config: QemuBridgeConfig) -> Self {
        Self {
            config,
            process: None,
            state: Arc::new(Mutex::new(QemuState::default())),
            event_tx: None,
        }
    }

    /// Create from simulation config
    pub fn from_simulation_config(sim_config: &SimulationConfig) -> Self {
        let machine = match sim_config.mcu.to_uppercase().as_str() {
            s if s.contains("ESP32") => QemuMachine::Esp32,
            s if s.contains("LM3S") => QemuMachine::Lm3s6965evb,
            _ => QemuMachine::Mps2An385,
        };

        Self::new(QemuBridgeConfig {
            machine,
            gdb_port: Some(1234),
            ..Default::default()
        })
    }

    /// Set event sender for async communication
    pub fn set_event_tx(&mut self, tx: mpsc::Sender<SimulationEvent>) {
        self.event_tx = Some(tx);
    }

    /// Check if QEMU is available
    pub fn is_available(&self) -> bool {
        let binary = self.config.machine.qemu_binary();
        Command::new(binary)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Get QEMU version
    pub fn get_version(&self) -> Option<String> {
        let binary = self.config.machine.qemu_binary();
        Command::new(binary)
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.lines().next().unwrap_or("").to_string())
    }

    /// Start QEMU
    pub fn start(&mut self) -> Result<(), String> {
        if self.process.is_some() {
            return Err("QEMU already running".to_string());
        }

        let binary = self.config.machine.qemu_binary();
        let mut cmd = Command::new(binary);

        // Machine type
        cmd.arg("-machine").arg(self.config.machine.to_qemu_arg());

        // CPU type if specified
        if let Some(cpu) = self.config.machine.cpu() {
            cmd.arg("-cpu").arg(cpu);
        }

        // Kernel/firmware
        if let Some(kernel) = &self.config.kernel_path {
            cmd.arg("-kernel").arg(kernel);
        }

        // GDB server
        if let Some(port) = self.config.gdb_port {
            cmd.arg("-gdb").arg(format!("tcp::{}", port));
            cmd.arg("-S"); // Start paused, waiting for GDB
        }

        // Serial
        if let Some(port) = self.config.serial_port {
            cmd.arg("-serial").arg(format!("tcp::{},server,nowait", port));
        }

        // Extra args
        for arg in &self.config.extra_args {
            cmd.arg(arg);
        }

        // No graphics
        cmd.arg("-nographic");

        // Start process
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let child = cmd.spawn().map_err(|e| format!("Failed to start QEMU: {}", e))?;

        self.process = Some(child);

        if let Ok(mut state) = self.state.lock() {
            state.running = true;
        }

        Ok(())
    }

    /// Stop QEMU
    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.process.take() {
            child.kill().map_err(|e| format!("Failed to stop QEMU: {}", e))?;
            
            if let Ok(mut state) = self.state.lock() {
                state.running = false;
            }
        }
        Ok(())
    }

    /// Check if QEMU is running
    pub fn is_running(&self) -> bool {
        if let Some(ref mut child) = self.process.as_ref() {
            // TODO: Check process status
            true
        } else {
            false
        }
    }

    /// Get QEMU command line for display
    pub fn get_command_line(&self) -> String {
        let binary = self.config.machine.qemu_binary();
        let mut args = vec![
            binary.to_string(),
            "-machine".to_string(),
            self.config.machine.to_qemu_arg().to_string(),
        ];

        if let Some(cpu) = self.config.machine.cpu() {
            args.push("-cpu".to_string());
            args.push(cpu.to_string());
        }

        if let Some(kernel) = &self.config.kernel_path {
            args.push("-kernel".to_string());
            args.push(kernel.clone());
        }

        if let Some(port) = self.config.gdb_port {
            args.push("-gdb".to_string());
            args.push(format!("tcp::{}", port));
        }

        args.push("-nographic".to_string());

        args.join(" ")
    }

    /// Set firmware/kernel path
    pub fn set_firmware(&mut self, path: &str) {
        self.config.kernel_path = Some(path.to_string());
    }

    /// Get GDB connection string
    pub fn get_gdb_connection(&self) -> Option<String> {
        self.config.gdb_port.map(|p| format!("localhost:{}", p))
    }

    /// Get serial connection string
    pub fn get_serial_connection(&self) -> Option<String> {
        self.config.serial_port.map(|p| format!("localhost:{}", p))
    }
}

impl Drop for QemuBridge {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// QEMU status info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QemuStatus {
    pub available: bool,
    pub running: bool,
    pub version: Option<String>,
    pub machine: String,
    pub gdb_port: Option<u16>,
    pub serial_port: Option<u16>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qemu_config() {
        let config = QemuBridgeConfig::default();
        assert_eq!(config.machine.to_qemu_arg(), "lm3s6965evb");
    }

    #[test]
    fn test_command_line() {
        let bridge = QemuBridge::new(QemuBridgeConfig::default());
        let cmd = bridge.get_command_line();
        assert!(cmd.contains("qemu-system-arm"));
        assert!(cmd.contains("lm3s6965evb"));
    }
}
