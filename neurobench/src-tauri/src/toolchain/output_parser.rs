// Compiler Output Parser
// Parse GCC/Clang output into structured diagnostics

use super::{CompilerDiagnostic, DiagnosticSeverity, SizeReport, SectionInfo, SectionType};
use regex::Regex;
use std::path::PathBuf;

/// Parse GCC/Clang compiler output into structured diagnostics
pub fn parse_compiler_output(output: &str) -> (Vec<CompilerDiagnostic>, Vec<CompilerDiagnostic>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    
    // GCC/Clang error format: file.c:10:5: error: message
    let re = Regex::new(
        r"(?m)^(.+?):(\d+):(?:(\d+):)?\s*(error|warning|note|fatal error):\s*(.+)$"
    ).unwrap();
    
    for cap in re.captures_iter(output) {
        let file = PathBuf::from(&cap[1]);
        let line: u32 = cap[2].parse().unwrap_or(0);
        let column: Option<u32> = cap.get(3).and_then(|m| m.as_str().parse().ok());
        let severity_str = &cap[4];
        let message = cap[5].to_string();
        
        let severity = match severity_str.to_lowercase().as_str() {
            "error" | "fatal error" => DiagnosticSeverity::Error,
            "warning" => DiagnosticSeverity::Warning,
            "note" => DiagnosticSeverity::Note,
            _ => DiagnosticSeverity::Info,
        };
        
        let diagnostic = CompilerDiagnostic {
            file,
            line,
            column,
            severity,
            code: None,
            message,
            suggestion: None,
            context_lines: vec![],
        };
        
        match severity {
            DiagnosticSeverity::Error => errors.push(diagnostic),
            _ => warnings.push(diagnostic),
        }
    }
    
    (errors, warnings)
}

/// Parse arm-none-eabi-size output into SizeReport
pub fn parse_size_output(output: &str, flash_total: u64, ram_total: u64) -> Option<SizeReport> {
    // Output format:
    //    text    data     bss     dec     hex filename
    //   12345    1234     567   14146    374a firmware.elf
    
    let lines: Vec<&str> = output.lines().collect();
    if lines.len() < 2 {
        return None;
    }
    
    // Parse the second line (data)
    let data_line = lines.get(1)?;
    let parts: Vec<&str> = data_line.split_whitespace().collect();
    
    if parts.len() < 4 {
        return None;
    }
    
    let text: u64 = parts.first()?.parse().ok()?;
    let data: u64 = parts.get(1)?.parse().ok()?;
    let bss: u64 = parts.get(2)?.parse().ok()?;
    let _dec: u64 = parts.get(3)?.parse().ok()?;
    
    // Flash = text + data (code + initialized data)
    let flash_used = text + data;
    // RAM = data + bss (initialized + zero-initialized)
    let ram_used = data + bss;
    
    let flash_percent = if flash_total > 0 {
        (flash_used as f32 / flash_total as f32) * 100.0
    } else {
        0.0
    };
    
    let ram_percent = if ram_total > 0 {
        (ram_used as f32 / ram_total as f32) * 100.0
    } else {
        0.0
    };
    
    Some(SizeReport {
        text,
        data,
        bss,
        total: flash_used + bss,
        flash_used,
        ram_used,
        flash_total,
        ram_total,
        flash_percent,
        ram_percent,
        sections: vec![
            SectionInfo {
                name: ".text".to_string(),
                size: text,
                address: 0x08000000, // Typical STM32 flash start
                section_type: SectionType::Code,
            },
            SectionInfo {
                name: ".data".to_string(),
                size: data,
                address: 0x20000000, // Typical STM32 RAM start
                section_type: SectionType::Data,
            },
            SectionInfo {
                name: ".bss".to_string(),
                size: bss,
                address: 0x20000000 + data,
                section_type: SectionType::Bss,
            },
        ],
    })
}

/// Generate suggested fixes for common errors
pub fn suggest_fix(diagnostic: &CompilerDiagnostic) -> Option<String> {
    let msg = diagnostic.message.to_lowercase();
    
    // Undefined reference errors
    if msg.contains("undefined reference to") {
        if msg.contains("main") {
            return Some("Add a main() function or check your linker script".to_string());
        }
        if msg.contains("_start") || msg.contains("_reset") {
            return Some("Check startup file and linker script are included".to_string());
        }
        return Some("Add the missing source file or library to your build".to_string());
    }
    
    // Include errors
    if msg.contains("no such file or directory") && msg.contains(".h") {
        return Some("Add the include path with -I flag or check header location".to_string());
    }
    
    // Type errors
    if msg.contains("implicit declaration of function") {
        return Some("Include the header file that declares this function".to_string());
    }
    
    // ARM specific
    if msg.contains("uses vfp register arguments") {
        return Some("Add -mfloat-abi=hard and -mfpu=fpv4-sp-d16 flags".to_string());
    }
    
    if msg.contains("multiple definition") {
        return Some("Check for duplicate definitions in source files".to_string());
    }
    
    None
}

/// Parse linker map file for symbol information
pub fn parse_map_file(content: &str) -> Vec<super::SymbolEntry> {
    let mut symbols = Vec::new();
    
    // Simple parser for GNU ld map file format
    let re = Regex::new(
        r"(?m)^\s*(0x[0-9a-fA-F]+)\s+(\S+)\s*$"
    ).unwrap();
    
    for cap in re.captures_iter(content) {
        if let Ok(address) = u64::from_str_radix(&cap[1].trim_start_matches("0x"), 16) {
            let name = cap[2].to_string();
            
            // Skip internal linker symbols
            if name.starts_with('.') || name.starts_with("PROVIDE") {
                continue;
            }
            
            symbols.push(super::SymbolEntry {
                name,
                address,
                size: 0, // Not easily available in basic map format
                section: String::new(),
                object_file: None,
            });
        }
    }
    
    symbols
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_compiler_output() {
        let output = r#"
main.c:10:5: error: expected ';' before 'return'
main.c:15:1: warning: unused variable 'x' [-Wunused-variable]
"#;
        
        let (errors, warnings) = parse_compiler_output(output);
        
        assert_eq!(errors.len(), 1);
        assert_eq!(warnings.len(), 1);
        assert_eq!(errors[0].line, 10);
        assert_eq!(errors[0].message, "expected ';' before 'return'");
    }
    
    #[test]
    fn test_parse_size_output() {
        let output = r#"   text    data     bss     dec     hex filename
  12345    1234     567   14146    374a firmware.elf"#;
        
        let report = parse_size_output(output, 512 * 1024, 128 * 1024).unwrap();
        
        assert_eq!(report.text, 12345);
        assert_eq!(report.data, 1234);
        assert_eq!(report.bss, 567);
        assert_eq!(report.flash_used, 13579); // text + data
    }
}
