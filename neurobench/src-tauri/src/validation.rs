// Code Validation Module
// Validates generated C/C++/Rust code using external compilers

use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

/// Validation result for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub success: bool,
    pub errors: Vec<ValidationMessage>,
    pub warnings: Vec<ValidationMessage>,
    pub compiler: String,
    pub exit_code: i32,
}

/// A single error or warning message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMessage {
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub message: String,
    pub severity: String,  // "error", "warning", "note"
}

/// Language for validation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationLanguage {
    C,
    Cpp,
    Rust,
}

impl ValidationLanguage {
    fn _extension(&self) -> &str {
        match self {
            ValidationLanguage::C => "c",
            ValidationLanguage::Cpp => "cpp",
            ValidationLanguage::Rust => "rs",
        }
    }
}

/// Validate C/C++ code using gcc or clang
pub fn validate_c_code(code: &str, is_cpp: bool) -> Result<ValidationResult, String> {
    // Create temporary file with the code
    let ext = if is_cpp { "cpp" } else { "c" };
    let mut temp_file = NamedTempFile::with_suffix(&format!(".{}", ext))
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    
    temp_file.write_all(code.as_bytes())
        .map_err(|e| format!("Failed to write code to temp file: {}", e))?;
    
    let temp_path = temp_file.path().to_string_lossy().to_string();
    
    // Try gcc first, then clang
    let compilers = if is_cpp {
        vec!["g++", "clang++"]
    } else {
        vec!["gcc", "clang"]
    };
    
    for compiler in &compilers {
        if let Ok(result) = run_c_compiler(compiler, &temp_path, is_cpp) {
            return Ok(result);
        }
    }
    
    // If no compiler found, try to provide helpful message
    Err("No C/C++ compiler found. Install gcc, g++, or clang.".to_string())
}

fn run_c_compiler(compiler: &str, file_path: &str, _is_cpp: bool) -> Result<ValidationResult, String> {
    // Run syntax check only (-fsyntax-only), with all warnings (-Wall -Wextra)
    let output = Command::new(compiler)
        .args([
            "-fsyntax-only",
            "-Wall",
            "-Wextra",
            "-Wpedantic",
            "-fdiagnostics-format=json",  // JSON output for easier parsing (gcc 10+)
            file_path,
        ])
        .output()
        .map_err(|e| format!("Failed to run {}: {}", compiler, e))?;
    
    let exit_code = output.status.code().unwrap_or(-1);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Parse compiler output
    let (errors, warnings) = parse_gcc_output(&stderr);
    
    Ok(ValidationResult {
        success: exit_code == 0 && errors.is_empty(),
        errors,
        warnings,
        compiler: compiler.to_string(),
        exit_code,
    })
}

/// Parse GCC/Clang error output
fn parse_gcc_output(output: &str) -> (Vec<ValidationMessage>, Vec<ValidationMessage>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    
    for line in output.lines() {
        // Try to parse JSON format first (modern GCC)
        if line.starts_with('[') {
            if let Ok(messages) = parse_gcc_json(line) {
                for msg in messages {
                    if msg.severity == "error" {
                        errors.push(msg);
                    } else {
                        warnings.push(msg);
                    }
                }
                continue;
            }
        }
        
        // Fall back to text parsing
        // Format: file.c:10:5: error: message
        //         file.c:10:5: warning: message
        if let Some(msg) = parse_gcc_text_line(line) {
            if msg.severity == "error" {
                errors.push(msg);
            } else if msg.severity == "warning" {
                warnings.push(msg);
            }
        }
    }
    
    (errors, warnings)
}

fn parse_gcc_json(json_str: &str) -> Result<Vec<ValidationMessage>, ()> {
    // Simplified JSON parsing for GCC diagnostic format
    // Real implementation would use serde_json
    let _ = json_str;
    Err(()) // Fall back to text parsing
}

fn parse_gcc_text_line(line: &str) -> Option<ValidationMessage> {
    // Parse format: file:line:col: severity: message
    // Example: test.c:10:5: error: expected ';' before 'return'
    
    let parts: Vec<&str> = line.splitn(5, ':').collect();
    if parts.len() < 5 {
        return None;
    }
    
    let line_num = parts[1].trim().parse::<u32>().ok();
    let col_num = parts[2].trim().parse::<u32>().ok();
    let severity = parts[3].trim().to_string();
    let message = parts[4].trim().to_string();
    
    if severity == "error" || severity == "warning" || severity == "note" {
        Some(ValidationMessage {
            line: line_num,
            column: col_num,
            message,
            severity,
        })
    } else {
        None
    }
}

/// Validate Rust code using rustc
pub fn validate_rust_code(code: &str) -> Result<ValidationResult, String> {
    // Create temporary file
    let mut temp_file = NamedTempFile::with_suffix(".rs")
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    
    temp_file.write_all(code.as_bytes())
        .map_err(|e| format!("Failed to write code to temp file: {}", e))?;
    
    let temp_path = temp_file.path().to_string_lossy().to_string();
    
    // Run rustc with --emit=metadata (type check only, no codegen)
    let output = Command::new("rustc")
        .args([
            "--emit=metadata",
            "--error-format=json",
            "-o", "/dev/null",  // Don't write output
            &temp_path,
        ])
        .output()
        .map_err(|e| format!("Failed to run rustc: {}", e))?;
    
    let exit_code = output.status.code().unwrap_or(-1);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    let (errors, warnings) = parse_rustc_output(&stderr);
    
    Ok(ValidationResult {
        success: exit_code == 0 && errors.is_empty(),
        errors,
        warnings,
        compiler: "rustc".to_string(),
        exit_code,
    })
}

fn parse_rustc_output(output: &str) -> (Vec<ValidationMessage>, Vec<ValidationMessage>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    
    for line in output.lines() {
        // Rustc JSON format
        if line.starts_with('{') && line.contains("\"level\"") {
            if let Some(msg) = parse_rustc_json_line(line) {
                if msg.severity == "error" {
                    errors.push(msg);
                } else {
                    warnings.push(msg);
                }
            }
        }
    }
    
    (errors, warnings)
}

fn parse_rustc_json_line(line: &str) -> Option<ValidationMessage> {
    // Simplified parsing - extract key fields
    // Real implementation would use serde_json
    
    let level = extract_json_string(line, "level")?;
    let message = extract_json_string(line, "message")?;
    
    // Try to extract line number from spans
    let line_num = extract_json_number(line, "line_start");
    let col_num = extract_json_number(line, "column_start");
    
    Some(ValidationMessage {
        line: line_num,
        column: col_num,
        message,
        severity: level,
    })
}

fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":\"", key);
    let start = json.find(&pattern)? + pattern.len();
    let end = json[start..].find('"')? + start;
    Some(json[start..end].to_string())
}

fn extract_json_number(json: &str, key: &str) -> Option<u32> {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern)? + pattern.len();
    let num_str: String = json[start..].chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    num_str.parse().ok()
}

/// Validate embedded C code with common STM32/ARM includes stubbed
pub fn validate_embedded_c(code: &str, is_cpp: bool) -> Result<ValidationResult, String> {
    // Add stub definitions for common embedded types
    let stubbed_code = format!(r#"
// Stub definitions for embedded validation
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;
typedef int int32_t;
typedef short int16_t;
typedef signed char int8_t;
typedef float float32_t;
typedef double float64_t;
typedef _Bool bool;
#define true 1
#define false 0
#define NULL ((void*)0)
#define __IO volatile
#define __I volatile const
#define __O volatile

// CMSIS stubs
typedef struct {{ uint32_t dummy; }} IRQn_Type;
#define NVIC_EnableIRQ(x) ((void)0)
#define NVIC_DisableIRQ(x) ((void)0)
#define __enable_irq() ((void)0)
#define __disable_irq() ((void)0)

// HAL stubs
typedef enum {{ HAL_OK = 0, HAL_ERROR = 1, HAL_BUSY = 2, HAL_TIMEOUT = 3 }} HAL_StatusTypeDef;
typedef struct {{ uint32_t dummy; }} GPIO_TypeDef;
typedef struct {{ uint32_t dummy; }} GPIO_InitTypeDef;
typedef struct {{ uint32_t dummy; }} UART_HandleTypeDef;
typedef struct {{ uint32_t dummy; }} SPI_HandleTypeDef;
typedef struct {{ uint32_t dummy; }} I2C_HandleTypeDef;
typedef struct {{ uint32_t dummy; }} TIM_HandleTypeDef;
typedef struct {{ uint32_t dummy; }} ADC_HandleTypeDef;
typedef struct {{ uint32_t dummy; }} DAC_HandleTypeDef;

#define GPIOA ((GPIO_TypeDef*)0x40020000)
#define GPIOB ((GPIO_TypeDef*)0x40020400)
#define GPIOC ((GPIO_TypeDef*)0x40020800)

// ARM math stubs
typedef float arm_fir_instance_f32;
typedef float arm_biquad_casd_df1_inst_f32;
typedef float arm_cfft_instance_f32;
typedef float arm_rfft_fast_instance_f32;
#define arm_fir_init_f32(...) ((void)0)
#define arm_fir_f32(...) ((void)0)
#define arm_biquad_cascade_df1_init_f32(...) ((void)0)
#define arm_biquad_cascade_df1_f32(...) ((void)0)
#define arm_cfft_f32(...) ((void)0)
#define arm_cmplx_mag_f32(...) ((void)0)
#define arm_rfft_fast_init_f32(...) ((void)0)
#define arm_rfft_fast_f32(...) ((void)0)
#define arm_fill_f32(...) ((void)0)
#define arm_copy_f32(...) ((void)0)
#define arm_max_f32(...) ((void)0)
#define arm_cos_f32(x) (0.0f)
#define PI 3.14159265358979f

// User code follows
#line 1 "generated_code"
{code}
"#, code = code);

    validate_c_code(&stubbed_code, is_cpp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gcc_text_line() {
        let line = "test.c:10:5: error: expected ';' before 'return'";
        let msg = parse_gcc_text_line(line).unwrap();
        
        assert_eq!(msg.line, Some(10));
        assert_eq!(msg.column, Some(5));
        assert_eq!(msg.severity, "error");
        assert!(msg.message.contains("expected"));
    }

    #[test]
    fn test_validation_result_serialization() {
        let result = ValidationResult {
            success: true,
            errors: vec![],
            warnings: vec![],
            compiler: "gcc".to_string(),
            exit_code: 0,
        };
        
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":true"));
    }
}
