// QEMU Simulation Module
// Provides ARM firmware emulation using QEMU

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use serde::{Deserialize, Serialize};

/// Supported QEMU machine types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QemuMachine {
    Stm32VldDiscovery,    // STM32F100 Discovery
    Stm32F4Discovery,     // STM32F407 Discovery (needs special build)
    Lm3s6965evb,          // TI Stellaris LM3S6965
    Lm3s811evb,           // TI Stellaris LM3S811
    MicrobitV1,           // BBC micro:bit v1
    Netduino2,            // Netduino 2 (STM32F205)
    Olimex_Stm32_H405,    // Olimex STM32-H405
    NucleoF411re,         // Nucleo-F411RE
    Custom(String),       // Custom machine string
}

impl QemuMachine {
    pub fn to_qemu_arg(&self) -> &str {
        match self {
            QemuMachine::Stm32VldDiscovery => "stm32vldiscovery",
            QemuMachine::Stm32F4Discovery => "stm32f4-discovery",
            QemuMachine::Lm3s6965evb => "lm3s6965evb",
            QemuMachine::Lm3s811evb => "lm3s811evb",
            QemuMachine::MicrobitV1 => "microbit",
            QemuMachine::Netduino2 => "netduino2",
            QemuMachine::Olimex_Stm32_H405 => "olimex-stm32-h405",
            QemuMachine::NucleoF411re => "nucleo-f411re",
            QemuMachine::Custom(s) => s,
        }
    }

    pub fn cpu(&self) -> &str {
        match self {
            QemuMachine::Lm3s6965evb | QemuMachine::Lm3s811evb => "cortex-m3",
            QemuMachine::Stm32VldDiscovery => "cortex-m3",
            QemuMachine::Stm32F4Discovery | QemuMachine::Netduino2 => "cortex-m4",
            QemuMachine::MicrobitV1 => "cortex-m0",
            QemuMachine::NucleoF411re => "cortex-m4",
            _ => "cortex-m3",
        }
    }
}

/// QEMU run configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QemuConfig {
    pub machine: String,
    pub cpu: Option<String>,
    pub firmware_path: String,
    pub gdb_port: Option<u16>,
    pub serial_output: bool,
    pub nographic: bool,
    pub memory_mb: Option<u32>,
    pub extra_args: Vec<String>,
}

impl Default for QemuConfig {
    fn default() -> Self {
        Self {
            machine: "lm3s6965evb".to_string(),
            cpu: None,
            firmware_path: String::new(),
            gdb_port: None,
            serial_output: true,
            nographic: true,
            memory_mb: None,
            extra_args: vec![],
        }
    }
}

/// QEMU process status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QemuStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub gdb_port: Option<u16>,
    pub exit_code: Option<i32>,
}

/// Running QEMU instance
pub struct QemuInstance {
    process: Child,
    output_buffer: Arc<Mutex<Vec<String>>>,
    config: QemuConfig,
}

impl QemuInstance {
    pub fn status(&mut self) -> QemuStatus {
        let exit_status = self.process.try_wait().ok().flatten();
        
        QemuStatus {
            running: exit_status.is_none(),
            pid: Some(self.process.id()),
            gdb_port: self.config.gdb_port,
            exit_code: exit_status.map(|s| s.code().unwrap_or(-1)),
        }
    }

    pub fn get_output(&self) -> Vec<String> {
        self.output_buffer.lock().unwrap().clone()
    }

    pub fn stop(&mut self) -> Result<(), String> {
        self.process.kill().map_err(|e| e.to_string())?;
        self.process.wait().map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Find QEMU ARM executable
pub fn find_qemu() -> Option<String> {
    // Check common locations
    let candidates = [
        "qemu-system-arm",
        "qemu-system-arm.exe",
        "C:\\Program Files\\qemu\\qemu-system-arm.exe",
        "C:\\Program Files (x86)\\qemu\\qemu-system-arm.exe",
        "/usr/bin/qemu-system-arm",
        "/usr/local/bin/qemu-system-arm",
        "/opt/homebrew/bin/qemu-system-arm",
    ];

    for candidate in candidates {
        if Command::new(candidate)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok()
        {
            return Some(candidate.to_string());
        }
    }
    None
}

/// Get QEMU version
pub fn get_qemu_version() -> Result<String, String> {
    let qemu = find_qemu().ok_or("QEMU not found")?;
    
    let output = Command::new(&qemu)
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to run QEMU: {}", e))?;

    let version = String::from_utf8_lossy(&output.stdout);
    let first_line = version.lines().next().unwrap_or("Unknown version");
    Ok(first_line.to_string())
}

/// List supported machines
pub fn list_machines() -> Result<Vec<String>, String> {
    let qemu = find_qemu().ok_or("QEMU not found")?;
    
    let output = Command::new(&qemu)
        .args(["-machine", "help"])
        .output()
        .map_err(|e| format!("Failed to list machines: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let machines: Vec<String> = stdout
        .lines()
        .skip(1) // Skip header
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            parts.first().map(|s| s.to_string())
        })
        .filter(|m| !m.is_empty())
        .collect();

    Ok(machines)
}

/// Start QEMU simulation
pub fn start_simulation(config: QemuConfig) -> Result<QemuInstance, String> {
    let qemu = find_qemu().ok_or("QEMU not found. Please install QEMU and add it to PATH.")?;

    let mut cmd = Command::new(&qemu);
    
    // Basic config
    cmd.arg("-machine").arg(&config.machine);
    
    if let Some(cpu) = &config.cpu {
        cmd.arg("-cpu").arg(cpu);
    }
    
    // Firmware
    cmd.arg("-kernel").arg(&config.firmware_path);
    
    // Graphics
    if config.nographic {
        cmd.arg("-nographic");
    }
    
    // Memory
    if let Some(mb) = config.memory_mb {
        cmd.arg("-m").arg(format!("{}M", mb));
    }
    
    // GDB server
    if let Some(port) = config.gdb_port {
        cmd.arg("-gdb").arg(format!("tcp::{}", port));
        cmd.arg("-S"); // Wait for debugger
    }
    
    // Serial output
    if config.serial_output {
        cmd.arg("-serial").arg("stdio");
    }
    
    // Extra args
    for arg in &config.extra_args {
        cmd.arg(arg);
    }

    // Spawn process
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to start QEMU: {}", e))?;

    // Capture output in background
    let output_buffer = Arc::new(Mutex::new(Vec::new()));
    let buffer_clone = output_buffer.clone();

    if let Some(stdout) = child.stdout.take() {
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let mut buf = buffer_clone.lock().unwrap();
                    buf.push(line);
                    // Keep last 1000 lines
                    if buf.len() > 1000 {
                        buf.remove(0);
                    }
                }
            }
        });
    }

    Ok(QemuInstance {
        process: child,
        output_buffer,
        config,
    })
}

/// Check if QEMU is available
pub fn is_qemu_available() -> bool {
    find_qemu().is_some()
}

/// Get predefined machine configurations
pub fn get_machine_presets() -> HashMap<String, QemuConfig> {
    let mut presets = HashMap::new();

    presets.insert("STM32 VL Discovery".to_string(), QemuConfig {
        machine: "stm32vldiscovery".to_string(),
        cpu: Some("cortex-m3".to_string()),
        ..Default::default()
    });

    presets.insert("LM3S6965 EVB".to_string(), QemuConfig {
        machine: "lm3s6965evb".to_string(),
        cpu: Some("cortex-m3".to_string()),
        ..Default::default()
    });

    presets.insert("LM3S811 EVB".to_string(), QemuConfig {
        machine: "lm3s811evb".to_string(),
        cpu: Some("cortex-m3".to_string()),
        ..Default::default()
    });

    presets.insert("Netduino 2".to_string(), QemuConfig {
        machine: "netduino2".to_string(),
        cpu: Some("cortex-m4".to_string()),
        ..Default::default()
    });

    presets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qemu_machine_arg() {
        assert_eq!(QemuMachine::Lm3s6965evb.to_qemu_arg(), "lm3s6965evb");
        assert_eq!(QemuMachine::Stm32VldDiscovery.to_qemu_arg(), "stm32vldiscovery");
    }

    #[test]
    fn test_default_config() {
        let config = QemuConfig::default();
        assert_eq!(config.machine, "lm3s6965evb");
        assert!(config.nographic);
    }

    #[test]
    fn test_machine_presets() {
        let presets = get_machine_presets();
        assert!(presets.contains_key("LM3S6965 EVB"));
        assert!(presets.contains_key("STM32 VL Discovery"));
    }
}
