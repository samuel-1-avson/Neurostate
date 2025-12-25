// Advanced Command Parser
// Parses complex command lines with operators, variables, and flags

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Command operator for chaining
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommandOperator {
    None,           // Single command
    And,            // &&
    Or,             // ||
    Pipe,           // |
    Background,     // &
}

/// Parsed command with flags and arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedCommand {
    pub command: String,
    pub args: Vec<String>,
    pub flags: HashMap<String, Option<String>>,
    pub operator: CommandOperator,
    pub next: Option<Box<ParsedCommand>>,
}

/// Command flag definition for autocomplete
#[derive(Debug, Clone)]
pub struct FlagDef {
    pub short: Option<char>,
    pub long: String,
    pub takes_value: bool,
    pub description: String,
    pub suggestions: Vec<String>,
}

/// Parse a full command line (may contain multiple commands)
pub fn parse_command_line(input: &str, variables: &HashMap<String, String>) -> Vec<ParsedCommand> {
    let expanded = expand_variables(input, variables);
    let mut commands = Vec::new();
    let mut remaining = expanded.as_str().trim();

    while !remaining.is_empty() {
        let (cmd, rest, operator) = parse_single_command(remaining);
        if !cmd.command.is_empty() {
            commands.push(cmd);
        }
        remaining = rest.trim();
        
        // If we hit a terminator, break
        if operator == CommandOperator::None && remaining.is_empty() {
            break;
        }
    }

    commands
}

/// Parse a single command with its flags and arguments
fn parse_single_command(input: &str) -> (ParsedCommand, &str, CommandOperator) {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '"';
    let mut chars = input.chars().peekable();
    let mut operator = CommandOperator::None;
    let mut consumed = 0;

    while let Some(c) = chars.next() {
        consumed += c.len_utf8();
        
        match c {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = c;
            }
            c if c == quote_char && in_quotes => {
                in_quotes = false;
            }
            '&' if !in_quotes => {
                if chars.peek() == Some(&'&') {
                    chars.next();
                    consumed += 1;
                    operator = CommandOperator::And;
                    break;
                } else {
                    operator = CommandOperator::Background;
                    break;
                }
            }
            '|' if !in_quotes => {
                if chars.peek() == Some(&'|') {
                    chars.next();
                    consumed += 1;
                    operator = CommandOperator::Or;
                    break;
                } else {
                    operator = CommandOperator::Pipe;
                    break;
                }
            }
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    let (command, args, flags) = if tokens.is_empty() {
        (String::new(), Vec::new(), HashMap::new())
    } else {
        let cmd = tokens[0].clone();
        let (args, flags) = parse_args_and_flags(&tokens[1..]);
        (cmd, args, flags)
    };

    let remaining = &input[consumed..];

    (
        ParsedCommand {
            command,
            args,
            flags,
            operator: operator.clone(),
            next: None,
        },
        remaining,
        operator,
    )
}

/// Parse arguments and flags from tokens
fn parse_args_and_flags(tokens: &[String]) -> (Vec<String>, HashMap<String, Option<String>>) {
    let mut args = Vec::new();
    let mut flags = HashMap::new();
    let mut i = 0;

    while i < tokens.len() {
        let token = &tokens[i];
        
        if token.starts_with("--") {
            // Long flag
            let flag_part = &token[2..];
            if let Some(eq_pos) = flag_part.find('=') {
                // --flag=value
                let key = flag_part[..eq_pos].to_string();
                let value = flag_part[eq_pos + 1..].to_string();
                flags.insert(key, Some(value));
            } else {
                // --flag or --flag value
                let key = flag_part.to_string();
                if i + 1 < tokens.len() && !tokens[i + 1].starts_with('-') {
                    flags.insert(key, Some(tokens[i + 1].clone()));
                    i += 1;
                } else {
                    flags.insert(key, None);
                }
            }
        } else if token.starts_with('-') && token.len() > 1 {
            // Short flag(s)
            for c in token[1..].chars() {
                flags.insert(c.to_string(), None);
            }
        } else {
            args.push(token.clone());
        }
        
        i += 1;
    }

    (args, flags)
}

/// Expand variables in a command string
pub fn expand_variables(input: &str, variables: &HashMap<String, String>) -> String {
    let mut result = input.to_string();
    
    // Expand $VAR and ${VAR} style variables
    for (key, value) in variables {
        result = result.replace(&format!("${{{}}}", key), value);
        result = result.replace(&format!("${}", key), value);
    }
    
    // Expand environment variables
    for (key, value) in std::env::vars() {
        result = result.replace(&format!("${{{}}}", key), &value);
        // Only replace $VAR if not already replaced
        if !variables.contains_key(&key) {
            result = result.replace(&format!("${}", key), &value);
        }
    }
    
    result
}

/// Known command definitions for autocomplete
pub fn get_command_defs() -> HashMap<String, Vec<FlagDef>> {
    let mut defs = HashMap::new();

    // flash command
    defs.insert("flash".to_string(), vec![
        FlagDef {
            short: Some('p'),
            long: "probe".to_string(),
            takes_value: true,
            description: "Debug probe type".to_string(),
            suggestions: vec!["stlink".into(), "jlink".into(), "cmsis-dap".into(), "blackmagic".into()],
        },
        FlagDef {
            short: Some('s'),
            long: "speed".to_string(),
            takes_value: true,
            description: "Flash speed in kHz".to_string(),
            suggestions: vec!["1000".into(), "4000".into(), "8000".into()],
        },
        FlagDef {
            short: Some('t'),
            long: "target".to_string(),
            takes_value: true,
            description: "Target MCU".to_string(),
            suggestions: vec!["stm32f401".into(), "stm32f407".into(), "stm32f103".into()],
        },
        FlagDef {
            short: Some('v'),
            long: "verify".to_string(),
            takes_value: false,
            description: "Verify after flash".to_string(),
            suggestions: vec![],
        },
        FlagDef {
            short: Some('r'),
            long: "reset".to_string(),
            takes_value: false,
            description: "Reset after flash".to_string(),
            suggestions: vec![],
        },
    ]);

    // monitor command
    defs.insert("monitor".to_string(), vec![
        FlagDef {
            short: Some('b'),
            long: "baud".to_string(),
            takes_value: true,
            description: "Baud rate".to_string(),
            suggestions: vec!["9600".into(), "115200".into(), "921600".into()],
        },
        FlagDef {
            short: Some('f'),
            long: "filter".to_string(),
            takes_value: true,
            description: "Filter pattern".to_string(),
            suggestions: vec![],
        },
    ]);

    // gdb command
    defs.insert("gdb".to_string(), vec![
        FlagDef {
            short: Some('p'),
            long: "port".to_string(),
            takes_value: true,
            description: "GDB server port".to_string(),
            suggestions: vec!["3333".into(), "4242".into()],
        },
    ]);

    // power command
    defs.insert("power".to_string(), vec![
        FlagDef {
            short: Some('i'),
            long: "interval".to_string(),
            takes_value: true,
            description: "Measurement interval (ms)".to_string(),
            suggestions: vec!["10".into(), "100".into(), "1000".into()],
        },
    ]);

    // build command
    defs.insert("build".to_string(), vec![
        FlagDef {
            short: Some('r'),
            long: "release".to_string(),
            takes_value: false,
            description: "Release build".to_string(),
            suggestions: vec![],
        },
        FlagDef {
            short: Some('t'),
            long: "target".to_string(),
            takes_value: true,
            description: "Target MCU".to_string(),
            suggestions: vec!["stm32f401".into(), "stm32f407".into(), "esp32".into()],
        },
    ]);

    defs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_command() {
        let vars = HashMap::new();
        let cmds = parse_command_line("flash firmware.elf", &vars);
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].command, "flash");
        assert_eq!(cmds[0].args, vec!["firmware.elf"]);
    }

    #[test]
    fn test_command_with_flags() {
        let vars = HashMap::new();
        let cmds = parse_command_line("flash --probe stlink --speed 8000 firmware.elf", &vars);
        assert_eq!(cmds[0].flags.get("probe"), Some(&Some("stlink".to_string())));
        assert_eq!(cmds[0].flags.get("speed"), Some(&Some("8000".to_string())));
    }

    #[test]
    fn test_chained_commands() {
        let vars = HashMap::new();
        let cmds = parse_command_line("build && flash && monitor uart", &vars);
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0].command, "build");
        assert_eq!(cmds[1].command, "flash");
        assert_eq!(cmds[2].command, "monitor");
    }

    #[test]
    fn test_variable_expansion() {
        let mut vars = HashMap::new();
        vars.insert("MCU".to_string(), "STM32F401".to_string());
        let cmds = parse_command_line("flash --target $MCU", &vars);
        assert_eq!(cmds[0].flags.get("target"), Some(&Some("STM32F401".to_string())));
    }
}
