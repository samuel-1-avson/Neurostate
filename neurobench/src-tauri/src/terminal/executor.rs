// Async Command Executor
// Handles command execution with streaming output support

use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use serde::{Deserialize, Serialize};
use super::{TerminalResult, TerminalLine};
use super::parser::{ParsedCommand, CommandOperator};

/// Streaming output chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamOutput {
    pub line_type: String,
    pub content: String,
    pub complete: bool,
}

/// Execution result with optional streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub lines: Vec<TerminalLine>,
    pub exit_code: Option<i32>,
    pub pid: Option<u32>,
}

/// Execute a shell command with output capture
pub fn execute_shell_command(cmd: &str, args: &[String], working_dir: Option<&str>) -> ExecutionResult {
    let shell = if cfg!(windows) { "cmd" } else { "sh" };
    let shell_arg = if cfg!(windows) { "/C" } else { "-c" };
    
    let full_command = if args.is_empty() {
        cmd.to_string()
    } else {
        format!("{} {}", cmd, args.join(" "))
    };

    let mut command = Command::new(shell);
    command.arg(shell_arg).arg(&full_command);
    
    if let Some(dir) = working_dir {
        command.current_dir(dir);
    }
    
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    match command.spawn() {
        Ok(child) => {
            let pid = child.id();
            let mut lines = Vec::new();
            
            if let Some(stdout) = child.stdout {
                let reader = BufReader::new(stdout);
                for line in reader.lines().flatten() {
                    lines.push(TerminalLine::output(&line));
                }
            }

            if let Some(stderr) = child.stderr {
                let reader = BufReader::new(stderr);
                for line in reader.lines().flatten() {
                    lines.push(TerminalLine::error(&line));
                }
            }

            ExecutionResult {
                success: true,
                lines,
                exit_code: Some(0),
                pid: Some(pid),
            }
        }
        Err(e) => ExecutionResult {
            success: false,
            lines: vec![TerminalLine::error(&format!("Failed to execute: {}", e))],
            exit_code: Some(1),
            pid: None,
        },
    }
}

/// Execute a parsed command chain (handles &&, ||, |)
pub fn execute_command_chain(commands: &[ParsedCommand], working_dir: Option<&str>) -> Vec<ExecutionResult> {
    let mut results = Vec::new();
    let mut last_success = true;

    for (i, cmd) in commands.iter().enumerate() {
        // Check operators for conditional execution
        if i > 0 {
            let prev_op = &commands[i - 1].operator;
            match prev_op {
                CommandOperator::And => {
                    if !last_success {
                        continue; // Skip if previous failed
                    }
                }
                CommandOperator::Or => {
                    if last_success {
                        continue; // Skip if previous succeeded
                    }
                }
                _ => {}
            }
        }

        let result = execute_single_command(cmd, working_dir);
        last_success = result.success;
        results.push(result);
    }

    results
}

/// Execute a single parsed command
pub fn execute_single_command(cmd: &ParsedCommand, working_dir: Option<&str>) -> ExecutionResult {
    // Build args from flags and positional args
    let mut args: Vec<String> = Vec::new();
    
    for (key, value) in &cmd.flags {
        if key.len() == 1 {
            args.push(format!("-{}", key));
        } else {
            args.push(format!("--{}", key));
        }
        if let Some(v) = value {
            args.push(v.clone());
        }
    }
    
    args.extend(cmd.args.clone());
    
    execute_shell_command(&cmd.command, &args, working_dir)
}

/// Format command output for display
pub fn format_output_line(content: &str, line_type: &str) -> String {
    match line_type {
        "error" => format!("\x1b[31m{}\x1b[0m", content),   // Red
        "success" => format!("\x1b[32m{}\x1b[0m", content), // Green
        "warning" => format!("\x1b[33m{}\x1b[0m", content), // Yellow
        "info" => format!("\x1b[36m{}\x1b[0m", content),    // Cyan
        "system" => format!("\x1b[35m{}\x1b[0m", content),  // Magenta
        _ => content.to_string(),
    }
}

/// Parse ANSI escape codes for highlighting
pub fn parse_ansi_codes(input: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let mut current_text = String::new();
    let mut current_color = String::new();
    let mut in_escape = false;
    let mut escape_buf = String::new();

    for c in input.chars() {
        if c == '\x1b' {
            if !current_text.is_empty() {
                result.push((current_text.clone(), current_color.clone()));
                current_text.clear();
            }
            in_escape = true;
            escape_buf.clear();
        } else if in_escape {
            escape_buf.push(c);
            if c == 'm' {
                current_color = escape_buf.clone();
                in_escape = false;
            }
        } else {
            current_text.push(c);
        }
    }

    if !current_text.is_empty() {
        result.push((current_text, current_color));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_output() {
        let output = format_output_line("Error message", "error");
        assert!(output.contains("\x1b[31m"));
    }

    #[test]
    fn test_parse_ansi() {
        let input = "\x1b[31mRed text\x1b[0m normal";
        let parsed = parse_ansi_codes(input);
        assert!(!parsed.is_empty());
    }
}
