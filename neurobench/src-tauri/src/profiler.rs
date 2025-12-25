// Performance Profiler Module
// Code performance analysis and optimization suggestions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Code complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetrics {
    pub lines_of_code: u32,
    pub functions: u32,
    pub loops: u32,
    pub conditionals: u32,
    pub cyclomatic_complexity: u32,
    pub max_nesting_depth: u32,
    pub estimated_stack_usage: u32,
    pub memory_operations: u32,
}

/// Performance issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceIssue {
    pub severity: String,  // info, warning, critical
    pub category: String,  // timing, memory, power, size
    pub message: String,
    pub line: Option<u32>,
    pub suggestion: String,
}

/// Profiling result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingResult {
    pub metrics: CodeMetrics,
    pub issues: Vec<PerformanceIssue>,
    pub optimization_score: u32,  // 0-100
    pub suggestions: Vec<String>,
}

/// Timing estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingEstimate {
    pub function_name: String,
    pub estimated_cycles: u32,
    pub estimated_us: f32,  // At 168MHz default
}

/// Analyze code for performance
pub fn analyze_performance(code: &str, mcu_freq_mhz: u32) -> ProfilingResult {
    let lines: Vec<&str> = code.lines().collect();
    
    // Count basic metrics
    let mut functions = 0u32;
    let mut loops = 0u32;
    let mut conditionals = 0u32;
    let mut max_depth = 0u32;
    let mut current_depth = 0u32;
    let mut memory_ops = 0u32;
    
    for line in &lines {
        let trimmed = line.trim();
        
        // Count functions
        if (trimmed.starts_with("void ") || trimmed.starts_with("int ") ||
            trimmed.starts_with("uint")) && trimmed.contains("(") && !trimmed.contains(";") {
            functions += 1;
        }
        
        // Count loops
        if trimmed.starts_with("for ") || trimmed.starts_with("while ") ||
           trimmed.starts_with("do ") {
            loops += 1;
        }
        
        // Count conditionals
        if trimmed.starts_with("if ") || trimmed.starts_with("else if") ||
           trimmed.starts_with("switch ") || trimmed.contains("? ") {
            conditionals += 1;
        }
        
        // Track nesting depth
        current_depth += trimmed.matches('{').count() as u32;
        current_depth = current_depth.saturating_sub(trimmed.matches('}').count() as u32);
        if current_depth > max_depth {
            max_depth = current_depth;
        }
        
        // Count memory operations
        if trimmed.contains("malloc") || trimmed.contains("free") ||
           trimmed.contains("memcpy") || trimmed.contains("memset") {
            memory_ops += 1;
        }
    }
    
    // Calculate cyclomatic complexity
    let cyclomatic = conditionals + loops + 1;
    
    // Estimate stack usage (rough)
    let estimated_stack = functions * 64 + loops * 8 + conditionals * 4;
    
    let metrics = CodeMetrics {
        lines_of_code: lines.len() as u32,
        functions,
        loops,
        conditionals,
        cyclomatic_complexity: cyclomatic,
        max_nesting_depth: max_depth,
        estimated_stack_usage: estimated_stack,
        memory_operations: memory_ops,
    };
    
    // Detect issues
    let mut issues = Vec::new();
    
    // Check for common performance issues
    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Floating point in interrupt context
        if trimmed.contains("float") || trimmed.contains("double") {
            if code.contains("IRQHandler") || code.contains("_Handler") {
                issues.push(PerformanceIssue {
                    severity: "warning".to_string(),
                    category: "timing".to_string(),
                    message: "Floating point operations in interrupt context".to_string(),
                    line: Some(line_num as u32 + 1),
                    suggestion: "Use fixed-point arithmetic in ISRs".to_string(),
                });
            }
        }
        
        // Division in loop
        if trimmed.contains("/ ") && loops > 0 {
            issues.push(PerformanceIssue {
                severity: "info".to_string(),
                category: "timing".to_string(),
                message: "Division operation detected (slow on ARM)".to_string(),
                line: Some(line_num as u32 + 1),
                suggestion: "Consider using bit shifts for power-of-2 divisions".to_string(),
            });
        }
        
        // Volatile in tight loop
        if trimmed.contains("volatile") && loops > 0 {
            issues.push(PerformanceIssue {
                severity: "info".to_string(),
                category: "timing".to_string(),
                message: "Volatile access in loop".to_string(),
                line: Some(line_num as u32 + 1),
                suggestion: "Cache volatile value locally if safe".to_string(),
            });
        }
    }
    
    // Memory issues
    if memory_ops > 0 {
        issues.push(PerformanceIssue {
            severity: "warning".to_string(),
            category: "memory".to_string(),
            message: format!("{} dynamic memory operations detected", memory_ops),
            line: None,
            suggestion: "Consider static allocation for embedded systems".to_string(),
        });
    }
    
    // Complexity issues
    if cyclomatic > 10 {
        issues.push(PerformanceIssue {
            severity: "warning".to_string(),
            category: "size".to_string(),
            message: format!("High cyclomatic complexity: {}", cyclomatic),
            line: None,
            suggestion: "Consider breaking complex functions into smaller ones".to_string(),
        });
    }
    
    if max_depth > 4 {
        issues.push(PerformanceIssue {
            severity: "info".to_string(),
            category: "size".to_string(),
            message: format!("Deep nesting detected: {} levels", max_depth),
            line: None,
            suggestion: "Reduce nesting with early returns or helper functions".to_string(),
        });
    }
    
    // Generate suggestions
    let mut suggestions = Vec::new();
    
    if loops > 5 {
        suggestions.push("Consider loop unrolling for critical loops".to_string());
    }
    
    if functions < 3 && metrics.lines_of_code > 100 {
        suggestions.push("Consider modularizing code into more functions".to_string());
    }
    
    suggestions.push("Enable compiler optimizations (-O2 or -Os)".to_string());
    suggestions.push("Use __attribute__((optimize)) for critical functions".to_string());
    
    // Calculate optimization score
    let mut score = 100u32;
    score = score.saturating_sub(issues.iter().filter(|i| i.severity == "critical").count() as u32 * 20);
    score = score.saturating_sub(issues.iter().filter(|i| i.severity == "warning").count() as u32 * 10);
    score = score.saturating_sub(issues.iter().filter(|i| i.severity == "info").count() as u32 * 5);
    if cyclomatic > 15 { score = score.saturating_sub(15); }
    if max_depth > 5 { score = score.saturating_sub(10); }
    
    ProfilingResult {
        metrics,
        issues,
        optimization_score: score.max(0).min(100),
        suggestions,
    }
}

/// Estimate function timing
pub fn estimate_timing(code: &str, mcu_freq_mhz: u32) -> Vec<TimingEstimate> {
    let mut estimates = Vec::new();
    let cycles_per_instruction = 1.5f32;  // ARM Cortex-M average
    
    let mut current_func = String::new();
    let mut instruction_count = 0u32;
    
    for line in code.lines() {
        let trimmed = line.trim();
        
        // Detect function start
        if (trimmed.starts_with("void ") || trimmed.starts_with("int ") ||
            trimmed.starts_with("uint")) && trimmed.contains("(") && !trimmed.contains(";") {
            
            if !current_func.is_empty() {
                let cycles = (instruction_count as f32 * cycles_per_instruction) as u32;
                let us = cycles as f32 / mcu_freq_mhz as f32;
                
                estimates.push(TimingEstimate {
                    function_name: current_func.clone(),
                    estimated_cycles: cycles,
                    estimated_us: us,
                });
            }
            
            // Extract function name
            if let Some(paren_idx) = trimmed.find('(') {
                let before_paren = &trimmed[..paren_idx];
                if let Some(name) = before_paren.split_whitespace().last() {
                    current_func = name.trim_start_matches('*').to_string();
                }
            }
            instruction_count = 0;
        } else if !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("/*") {
            // Count instructions (rough estimate)
            instruction_count += 1;
            
            // Extra cycles for certain operations
            if trimmed.contains("* ") { instruction_count += 2; }  // Multiply
            if trimmed.contains("/ ") { instruction_count += 10; }  // Division
            if trimmed.contains("for ") || trimmed.contains("while ") { instruction_count += 3; }
        }
    }
    
    // Add last function
    if !current_func.is_empty() {
        let cycles = (instruction_count as f32 * cycles_per_instruction) as u32;
        let us = cycles as f32 / mcu_freq_mhz as f32;
        
        estimates.push(TimingEstimate {
            function_name: current_func,
            estimated_cycles: cycles,
            estimated_us: us,
        });
    }
    
    estimates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_performance() {
        let code = r#"
void test_func(void) {
    for (int i = 0; i < 10; i++) {
        if (i % 2 == 0) {
            // do something
        }
    }
}
"#;
        let result = analyze_performance(code, 168);
        assert_eq!(result.metrics.functions, 1);
        assert_eq!(result.metrics.loops, 1);
    }

    #[test]
    fn test_estimate_timing() {
        let code = r#"
void delay_ms(uint32_t ms) {
    for (volatile uint32_t i = 0; i < ms * 4000; i++);
}
"#;
        let timing = estimate_timing(code, 168);
        assert!(!timing.is_empty());
    }
}
