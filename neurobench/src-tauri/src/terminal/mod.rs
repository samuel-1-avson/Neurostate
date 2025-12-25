// Advanced Terminal Module
// AI-augmented embedded systems terminal with native PTY, advanced parsing, and autocomplete

pub mod parser;
pub mod executor;
pub mod commands;
pub mod autocomplete;
pub mod themes;

use serde::{Deserialize, Serialize};
pub use parser::{ParsedCommand, CommandOperator};
pub use executor::{ExecutionResult, StreamOutput};
pub use commands::process_embedded_command;
pub use autocomplete::{get_completions, CompletionItem};
pub use themes::{TerminalTheme, get_theme};

/// Terminal command result with enhanced output
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TerminalResult {
    pub success: bool,
    pub output: Vec<TerminalLine>,
    pub exit_code: Option<i32>,
    pub streaming: bool,
}

/// Single line of terminal output with ANSI support
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TerminalLine {
    pub line_type: String,
    pub content: String,
    pub ansi: Option<String>,  // Raw ANSI codes for advanced rendering
}

/// Terminal session state
#[derive(Debug, Clone)]
pub struct TerminalSession {
    pub id: String,
    pub working_dir: String,
    pub variables: std::collections::HashMap<String, String>,
    pub history: Vec<String>,
    pub history_index: usize,
}

impl TerminalResult {
    pub fn success(lines: Vec<TerminalLine>) -> Self {
        Self {
            success: true,
            output: lines,
            exit_code: Some(0),
            streaming: false,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            output: vec![TerminalLine::error(message)],
            exit_code: Some(1),
            streaming: false,
        }
    }

    pub fn info(message: &str) -> Self {
        Self {
            success: true,
            output: vec![TerminalLine::info(message)],
            exit_code: Some(0),
            streaming: false,
        }
    }

    pub fn streaming() -> Self {
        Self {
            success: true,
            output: vec![],
            exit_code: None,
            streaming: true,
        }
    }
}

impl TerminalLine {
    pub fn output(content: &str) -> Self {
        Self {
            line_type: "output".to_string(),
            content: content.to_string(),
            ansi: None,
        }
    }

    pub fn error(content: &str) -> Self {
        Self {
            line_type: "error".to_string(),
            content: content.to_string(),
            ansi: Some("\x1b[31m".to_string()), // Red
        }
    }

    pub fn success(content: &str) -> Self {
        Self {
            line_type: "success".to_string(),
            content: content.to_string(),
            ansi: Some("\x1b[32m".to_string()), // Green
        }
    }

    pub fn info(content: &str) -> Self {
        Self {
            line_type: "info".to_string(),
            content: content.to_string(),
            ansi: Some("\x1b[36m".to_string()), // Cyan
        }
    }

    pub fn system(content: &str) -> Self {
        Self {
            line_type: "system".to_string(),
            content: content.to_string(),
            ansi: Some("\x1b[35m".to_string()), // Magenta
        }
    }

    pub fn warning(content: &str) -> Self {
        Self {
            line_type: "warning".to_string(),
            content: content.to_string(),
            ansi: Some("\x1b[33m".to_string()), // Yellow
        }
    }

    pub fn with_ansi(content: &str, ansi_code: &str) -> Self {
        Self {
            line_type: "ansi".to_string(),
            content: content.to_string(),
            ansi: Some(ansi_code.to_string()),
        }
    }
}

impl Default for TerminalSession {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            working_dir: std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| ".".to_string()),
            variables: std::collections::HashMap::new(),
            history: Vec::new(),
            history_index: 0,
        }
    }
}

impl TerminalSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_variable(&mut self, name: &str, value: &str) {
        self.variables.insert(name.to_string(), value.to_string());
    }

    pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.variables.get(name)
    }

    pub fn add_to_history(&mut self, command: &str) {
        if !command.is_empty() {
            self.history.push(command.to_string());
            self.history_index = self.history.len();
        }
    }

    pub fn history_previous(&mut self) -> Option<&String> {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.history.get(self.history_index)
        } else {
            None
        }
    }

    pub fn history_next(&mut self) -> Option<&String> {
        if self.history_index < self.history.len() {
            self.history_index += 1;
            self.history.get(self.history_index)
        } else {
            None
        }
    }
}

/// Get terminal welcome message
pub fn get_welcome_message() -> Vec<TerminalLine> {
    vec![
        TerminalLine::with_ansi("╔══════════════════════════════════════════════════════════════════════╗", "\x1b[38;5;141m"),
        TerminalLine::with_ansi("║      🧠 NeuroBench Advanced Terminal v2.0 - AI-Augmented            ║", "\x1b[38;5;141m"),
        TerminalLine::with_ansi("╠══════════════════════════════════════════════════════════════════════╣", "\x1b[38;5;99m"),
        TerminalLine::output("  Type 'help' for commands, 'ai <question>' for AI assistant"),
        TerminalLine::output("  Commands: flash, monitor, gdb, power, trace, fsm, build, serial"),
        TerminalLine::with_ansi("╚══════════════════════════════════════════════════════════════════════╝", "\x1b[38;5;99m"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_session() {
        let mut session = TerminalSession::new();
        session.set_variable("MCU", "STM32F401");
        assert_eq!(session.get_variable("MCU"), Some(&"STM32F401".to_string()));
    }

    #[test]
    fn test_terminal_line_ansi() {
        let line = TerminalLine::error("test error");
        assert!(line.ansi.is_some());
        assert!(line.ansi.unwrap().contains("31")); // Red
    }
}
