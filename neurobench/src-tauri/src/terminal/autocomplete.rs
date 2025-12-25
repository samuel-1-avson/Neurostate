// Tab Completion Engine
// Dynamic autocomplete for commands, paths, pins, and peripherals

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Completion item with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub text: String,
    pub display: String,
    pub description: String,
    pub kind: CompletionKind,
    pub insert_text: Option<String>,
}

/// Types of completions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompletionKind {
    Command,
    Flag,
    Argument,
    Pin,
    Peripheral,
    Path,
    Variable,
    McuTarget,
}

/// Get completions for the current input
pub fn get_completions(input: &str, cursor_pos: usize) -> Vec<CompletionItem> {
    let before_cursor = &input[..cursor_pos.min(input.len())];
    let parts: Vec<&str> = before_cursor.split_whitespace().collect();
    
    if parts.is_empty() || (parts.len() == 1 && !before_cursor.ends_with(' ')) {
        // Completing command name
        let prefix = parts.first().map(|s| *s).unwrap_or("");
        return complete_commands(prefix);
    }
    
    let command = parts[0];
    let last_part = parts.last().map(|s| *s).unwrap_or("");
    
    // Check if we're completing a flag value
    if parts.len() >= 2 {
        let prev = parts[parts.len() - 2];
        if prev.starts_with('-') {
            return complete_flag_value(command, prev, last_part);
        }
    }
    
    // Check if completing a flag
    if last_part.starts_with('-') {
        return complete_flags(command, last_part);
    }
    
    // Check for pin completion (PA, PB, etc.)
    if last_part.len() >= 1 && last_part.chars().next().map(|c| c == 'P').unwrap_or(false) {
        return complete_pins(last_part);
    }
    
    // Complete based on command context
    complete_command_args(command, last_part)
}

/// Complete command names
fn complete_commands(prefix: &str) -> Vec<CompletionItem> {
    let commands = vec![
        ("flash", "Flash firmware to MCU", "flash [file] --probe <type>"),
        ("monitor", "Real-time peripheral monitoring", "monitor uart|can|gpio"),
        ("gdb", "GDB debugger control", "gdb connect|disconnect"),
        ("debug", "Launch debug session", "debug launch"),
        ("trace", "ITM/SWO tracing", "trace start|stop swo <freq>"),
        ("power", "Power measurement", "power measure|report"),
        ("build", "Build project", "build [--release] --target <mcu>"),
        ("clean", "Clean build artifacts", "clean"),
        ("rebuild", "Clean and rebuild", "rebuild"),
        ("serial", "Serial port commands", "serial list|open <port>"),
        ("fsm", "FSM operations", "fsm simulate|step|validate"),
        ("driver", "Generate drivers", "driver gpio|uart|spi|i2c"),
        ("mcu", "Set target MCU", "mcu <target>"),
        ("target", "Set target MCU", "target <mcu>"),
        ("gpio", "GPIO control", "gpio config|set|read <pin>"),
        ("ai", "AI assistant", "ai \"your question\""),
        ("help", "Show help", "help [command]"),
        ("version", "Show version", "version"),
        ("clear", "Clear terminal", "clear"),
        ("history", "Command history", "history"),
        ("pwd", "Print directory", "pwd"),
        ("ls", "List directory", "ls [path]"),
        ("echo", "Echo text", "echo <text>"),
        ("export", "Set variable", "export VAR=value"),
        ("env", "Show environment", "env"),
        ("dfu", "DFU bootloader", "dfu enter|exit"),
        ("verify", "Verify flash", "verify"),
        ("erase", "Erase flash", "erase [--full]"),
        ("bp", "Breakpoints", "bp add|remove|list"),
        ("log", "Log commands", "log dump|show"),
        ("info", "System info", "info"),
    ];
    
    commands
        .into_iter()
        .filter(|(name, _, _)| name.starts_with(&prefix.to_lowercase()))
        .map(|(name, desc, usage)| CompletionItem {
            text: name.to_string(),
            display: name.to_string(),
            description: desc.to_string(),
            kind: CompletionKind::Command,
            insert_text: Some(usage.to_string()),
        })
        .collect()
}

/// Complete flags for a command
fn complete_flags(command: &str, prefix: &str) -> Vec<CompletionItem> {
    let flags = match command.to_lowercase().as_str() {
        "flash" => vec![
            ("--probe", "-p", "Debug probe type (stlink, jlink)"),
            ("--speed", "-s", "Flash speed in kHz"),
            ("--target", "-t", "Target MCU"),
            ("--verify", "-v", "Verify after flash"),
            ("--reset", "-r", "Reset after flash"),
        ],
        "monitor" => vec![
            ("--baud", "-b", "Baud rate"),
            ("--filter", "-f", "Filter pattern"),
            ("--hex", "-x", "Hex output"),
        ],
        "build" => vec![
            ("--release", "-r", "Release build"),
            ("--target", "-t", "Target MCU"),
            ("--verbose", "-v", "Verbose output"),
        ],
        "gdb" => vec![
            ("--port", "-p", "GDB server port"),
        ],
        "power" => vec![
            ("--interval", "-i", "Measurement interval"),
        ],
        "erase" => vec![
            ("--full", "", "Full chip erase"),
            ("--sector", "", "Erase specific sector"),
        ],
        _ => vec![],
    };
    
    flags
        .into_iter()
        .filter(|(long, short, _)| {
            long.starts_with(prefix) || (short.starts_with(prefix) && !short.is_empty())
        })
        .map(|(long, short, desc)| CompletionItem {
            text: long.to_string(),
            display: if short.is_empty() {
                long.to_string()
            } else {
                format!("{}, {}", long, short)
            },
            description: desc.to_string(),
            kind: CompletionKind::Flag,
            insert_text: None,
        })
        .collect()
}

/// Complete flag values
fn complete_flag_value(command: &str, flag: &str, prefix: &str) -> Vec<CompletionItem> {
    let flag_name = flag.trim_start_matches('-');
    
    match (command.to_lowercase().as_str(), flag_name) {
        ("flash", "probe" | "p") => {
            vec!["stlink", "jlink", "cmsis-dap", "blackmagic"]
                .into_iter()
                .filter(|p| p.starts_with(prefix))
                .map(|p| CompletionItem {
                    text: p.to_string(),
                    display: p.to_string(),
                    description: format!("{} debug probe", p.to_uppercase()),
                    kind: CompletionKind::Argument,
                    insert_text: None,
                })
                .collect()
        }
        ("flash" | "build", "target" | "t") | ("mcu", _) => complete_mcu_targets(prefix),
        ("flash", "speed" | "s") => {
            vec!["1000", "4000", "8000", "12000"]
                .into_iter()
                .filter(|s| s.starts_with(prefix))
                .map(|s| CompletionItem {
                    text: s.to_string(),
                    display: format!("{} kHz", s),
                    description: "Flash speed".to_string(),
                    kind: CompletionKind::Argument,
                    insert_text: None,
                })
                .collect()
        }
        ("monitor", "baud" | "b") => {
            vec!["9600", "38400", "57600", "115200", "230400", "460800", "921600"]
                .into_iter()
                .filter(|b| b.starts_with(prefix))
                .map(|b| CompletionItem {
                    text: b.to_string(),
                    display: format!("{} baud", b),
                    description: "Baud rate".to_string(),
                    kind: CompletionKind::Argument,
                    insert_text: None,
                })
                .collect()
        }
        _ => vec![],
    }
}

/// Complete MCU targets
fn complete_mcu_targets(prefix: &str) -> Vec<CompletionItem> {
    let targets = vec![
        ("stm32f401", "STM32F401 (Cortex-M4, 84MHz, 256KB Flash)"),
        ("stm32f103", "STM32F103 (Cortex-M3, 72MHz, 64KB Flash)"),
        ("stm32f407", "STM32F407 (Cortex-M4, 168MHz, 1MB Flash)"),
        ("stm32f746", "STM32F746 (Cortex-M7, 216MHz, 1MB Flash)"),
        ("stm32h7", "STM32H7 (Cortex-M7, 480MHz)"),
        ("esp32", "ESP32 (Dual Xtensa, 240MHz, WiFi+BT)"),
        ("esp32-s3", "ESP32-S3 (Dual Xtensa, 240MHz, AI)"),
        ("esp8266", "ESP8266 (Tensilica, 80MHz, WiFi)"),
        ("nrf52832", "nRF52832 (Cortex-M4, 64MHz, BLE)"),
        ("nrf52840", "nRF52840 (Cortex-M4, 64MHz, BLE+USB)"),
        ("rp2040", "RP2040 (Dual Cortex-M0+, 133MHz)"),
        ("atmega328p", "ATMega328P (AVR, 16MHz)"),
        ("atmega2560", "ATMega2560 (AVR, 16MHz)"),
    ];
    
    targets
        .into_iter()
        .filter(|(name, _)| name.starts_with(&prefix.to_lowercase()))
        .map(|(name, desc)| CompletionItem {
            text: name.to_string(),
            display: name.to_uppercase(),
            description: desc.to_string(),
            kind: CompletionKind::McuTarget,
            insert_text: None,
        })
        .collect()
}

/// Complete GPIO pins
fn complete_pins(prefix: &str) -> Vec<CompletionItem> {
    let mut completions = Vec::new();
    
    let ports = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'];
    
    for port in ports {
        for pin in 0..16 {
            let pin_name = format!("P{}{}", port, pin);
            if pin_name.to_lowercase().starts_with(&prefix.to_lowercase()) {
                completions.push(CompletionItem {
                    text: pin_name.clone(),
                    display: pin_name.clone(),
                    description: format!("GPIO Port {} Pin {}", port, pin),
                    kind: CompletionKind::Pin,
                    insert_text: None,
                });
            }
        }
    }
    
    completions.truncate(10); // Limit results
    completions
}

/// Complete command-specific arguments
fn complete_command_args(command: &str, prefix: &str) -> Vec<CompletionItem> {
    match command.to_lowercase().as_str() {
        "monitor" => {
            vec!["uart", "can", "gpio", "adc", "spi", "i2c"]
                .into_iter()
                .filter(|t| t.starts_with(prefix))
                .map(|t| CompletionItem {
                    text: t.to_string(),
                    display: t.to_uppercase(),
                    description: format!("{} monitoring", t.to_uppercase()),
                    kind: CompletionKind::Argument,
                    insert_text: None,
                })
                .collect()
        }
        "driver" => {
            vec!["gpio", "uart", "spi", "i2c", "can", "adc", "dac", "timer", "pwm"]
                .into_iter()
                .filter(|d| d.starts_with(prefix))
                .map(|d| CompletionItem {
                    text: d.to_string(),
                    display: d.to_uppercase(),
                    description: format!("{} driver", d.to_uppercase()),
                    kind: CompletionKind::Peripheral,
                    insert_text: None,
                })
                .collect()
        }
        "fsm" => {
            vec!["simulate", "step", "validate", "stats", "generate"]
                .into_iter()
                .filter(|a| a.starts_with(prefix))
                .map(|a| CompletionItem {
                    text: a.to_string(),
                    display: a.to_string(),
                    description: format!("FSM {}", a),
                    kind: CompletionKind::Argument,
                    insert_text: None,
                })
                .collect()
        }
        "gdb" => {
            vec!["connect", "disconnect"]
                .into_iter()
                .filter(|a| a.starts_with(prefix))
                .map(|a| CompletionItem {
                    text: a.to_string(),
                    display: a.to_string(),
                    description: format!("GDB {}", a),
                    kind: CompletionKind::Argument,
                    insert_text: None,
                })
                .collect()
        }
        "dfu" => {
            vec!["enter", "exit"]
                .into_iter()
                .filter(|a| a.starts_with(prefix))
                .map(|a| CompletionItem {
                    text: a.to_string(),
                    display: a.to_string(),
                    description: format!("DFU {}", a),
                    kind: CompletionKind::Argument,
                    insert_text: None,
                })
                .collect()
        }
        "gpio" => {
            vec!["config", "set", "clear", "toggle", "read", "high", "low"]
                .into_iter()
                .filter(|a| a.starts_with(prefix))
                .map(|a| CompletionItem {
                    text: a.to_string(),
                    display: a.to_string(),
                    description: format!("GPIO {}", a),
                    kind: CompletionKind::Argument,
                    insert_text: None,
                })
                .collect()
        }
        "serial" => {
            vec!["list", "open", "close"]
                .into_iter()
                .filter(|a| a.starts_with(prefix))
                .map(|a| CompletionItem {
                    text: a.to_string(),
                    display: a.to_string(),
                    description: format!("Serial {}", a),
                    kind: CompletionKind::Argument,
                    insert_text: None,
                })
                .collect()
        }
        "trace" => {
            vec!["start", "stop"]
                .into_iter()
                .filter(|a| a.starts_with(prefix))
                .map(|a| CompletionItem {
                    text: a.to_string(),
                    display: a.to_string(),
                    description: format!("Trace {}", a),
                    kind: CompletionKind::Argument,
                    insert_text: None,
                })
                .collect()
        }
        "power" => {
            vec!["measure", "report"]
                .into_iter()
                .filter(|a| a.starts_with(prefix))
                .map(|a| CompletionItem {
                    text: a.to_string(),
                    display: a.to_string(),
                    description: format!("Power {}", a),
                    kind: CompletionKind::Argument,
                    insert_text: None,
                })
                .collect()
        }
        "bp" | "breakpoint" => {
            vec!["add", "remove", "list", "clear"]
                .into_iter()
                .filter(|a| a.starts_with(prefix))
                .map(|a| CompletionItem {
                    text: a.to_string(),
                    display: a.to_string(),
                    description: format!("Breakpoint {}", a),
                    kind: CompletionKind::Argument,
                    insert_text: None,
                })
                .collect()
        }
        "help" => complete_commands(prefix),
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_completion() {
        let completions = get_completions("fl", 2);
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.text == "flash"));
    }

    #[test]
    fn test_flag_completion() {
        let completions = get_completions("flash --p", 9);
        assert!(completions.iter().any(|c| c.text == "--probe"));
    }

    #[test]
    fn test_pin_completion() {
        let completions = get_completions("gpio config PA", 14);
        assert!(completions.iter().any(|c| c.text.starts_with("PA")));
    }
}
