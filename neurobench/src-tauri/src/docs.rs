// Documentation Generator Module
// Auto-generate Doxygen-style documentation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Function documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDoc {
    pub name: String,
    pub brief: String,
    pub description: String,
    pub params: Vec<ParamDoc>,
    pub returns: Option<String>,
    pub notes: Vec<String>,
    pub examples: Vec<String>,
}

/// Parameter documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDoc {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub direction: String,  // in, out, inout
}

/// File documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDoc {
    pub filename: String,
    pub brief: String,
    pub author: String,
    pub date: String,
    pub version: String,
    pub copyright: Option<String>,
    pub functions: Vec<FunctionDoc>,
}

/// Generate Doxygen header for a file
pub fn generate_file_header(doc: &FileDoc) -> String {
    let mut output = String::new();
    
    output.push_str("/**\n");
    output.push_str(&format!(" * @file {}\n", doc.filename));
    output.push_str(&format!(" * @brief {}\n", doc.brief));
    output.push_str(" *\n");
    output.push_str(&format!(" * @author {}\n", doc.author));
    output.push_str(&format!(" * @date {}\n", doc.date));
    output.push_str(&format!(" * @version {}\n", doc.version));
    if let Some(copyright) = &doc.copyright {
        output.push_str(&format!(" * @copyright {}\n", copyright));
    }
    output.push_str(" */\n\n");
    
    output
}

/// Generate Doxygen comment for a function
pub fn generate_function_doc(doc: &FunctionDoc) -> String {
    let mut output = String::new();
    
    output.push_str("/**\n");
    output.push_str(&format!(" * @brief {}\n", doc.brief));
    
    if !doc.description.is_empty() {
        output.push_str(" *\n");
        for line in doc.description.lines() {
            output.push_str(&format!(" * {}\n", line));
        }
    }
    
    if !doc.params.is_empty() {
        output.push_str(" *\n");
        for param in &doc.params {
            let dir = match param.direction.as_str() {
                "out" => "[out] ",
                "inout" => "[in,out] ",
                _ => "[in] ",
            };
            output.push_str(&format!(" * @param {} {}{}\n", 
                param.name, dir, param.description));
        }
    }
    
    if let Some(returns) = &doc.returns {
        output.push_str(&format!(" * @return {}\n", returns));
    }
    
    for note in &doc.notes {
        output.push_str(&format!(" * @note {}\n", note));
    }
    
    if !doc.examples.is_empty() {
        output.push_str(" *\n");
        output.push_str(" * @code\n");
        for example in &doc.examples {
            output.push_str(&format!(" * {}\n", example));
        }
        output.push_str(" * @endcode\n");
    }
    
    output.push_str(" */\n");
    
    output
}

/// Extract function info from C code
pub fn extract_functions(code: &str) -> Vec<FunctionDoc> {
    let mut functions = Vec::new();
    
    // Simple regex-like parsing for function declarations
    for line in code.lines() {
        let trimmed = line.trim();
        
        // Look for function definitions
        if (trimmed.starts_with("void ") || 
            trimmed.starts_with("int ") ||
            trimmed.starts_with("uint") ||
            trimmed.starts_with("char ") ||
            trimmed.starts_with("bool ") ||
            trimmed.starts_with("float ") ||
            trimmed.starts_with("double ")) && 
           trimmed.contains("(") && !trimmed.contains(";")
        {
            // Parse function name
            if let Some(paren_idx) = trimmed.find('(') {
                let before_paren = &trimmed[..paren_idx];
                let parts: Vec<&str> = before_paren.split_whitespace().collect();
                
                if parts.len() >= 2 {
                    let return_type = parts[0..parts.len()-1].join(" ");
                    let func_name = parts[parts.len()-1].trim_start_matches('*');
                    
                    // Parse parameters
                    let mut params = Vec::new();
                    if let Some(close_paren) = trimmed.find(')') {
                        let param_str = &trimmed[paren_idx+1..close_paren];
                        for param in param_str.split(',') {
                            let param = param.trim();
                            if !param.is_empty() && param != "void" {
                                let param_parts: Vec<&str> = param.split_whitespace().collect();
                                if param_parts.len() >= 2 {
                                    params.push(ParamDoc {
                                        name: param_parts.last().unwrap()
                                            .trim_start_matches('*').to_string(),
                                        param_type: param_parts[..param_parts.len()-1].join(" "),
                                        description: "Parameter description".to_string(),
                                        direction: "in".to_string(),
                                    });
                                }
                            }
                        }
                    }
                    
                    functions.push(FunctionDoc {
                        name: func_name.to_string(),
                        brief: format!("{} function", func_name),
                        description: String::new(),
                        params,
                        returns: if return_type != "void" {
                            Some(format!("{} value", return_type))
                        } else {
                            None
                        },
                        notes: vec![],
                        examples: vec![],
                    });
                }
            }
        }
    }
    
    functions
}

/// Generate complete documentation for code
pub fn generate_documentation(
    code: &str,
    filename: &str,
    author: &str,
    brief: &str,
) -> String {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    
    let functions = extract_functions(code);
    
    let file_doc = FileDoc {
        filename: filename.to_string(),
        brief: brief.to_string(),
        author: author.to_string(),
        date,
        version: "1.0.0".to_string(),
        copyright: Some("NeuroBench Generated".to_string()),
        functions: functions.clone(),
    };
    
    let mut output = generate_file_header(&file_doc);
    
    // Add include guards
    let guard = filename.replace(".", "_").to_uppercase();
    output.push_str(&format!("#ifndef {}\n", guard));
    output.push_str(&format!("#define {}\n\n", guard));
    
    // Add function declarations with docs
    for func in &functions {
        output.push_str(&generate_function_doc(func));
        
        // Add function prototype
        let params: String = func.params.iter()
            .map(|p| format!("{} {}", p.param_type, p.name))
            .collect::<Vec<_>>()
            .join(", ");
        
        let return_type = if func.returns.is_some() { "int" } else { "void" };
        output.push_str(&format!("{} {}({});\n\n", return_type, func.name, params));
    }
    
    output.push_str(&format!("#endif /* {} */\n", guard));
    
    output
}

/// Generate Doxyfile configuration
pub fn generate_doxyfile(project_name: &str, output_dir: &str) -> String {
    format!(r#"# Doxyfile generated by NeuroBench

PROJECT_NAME           = "{}"
PROJECT_BRIEF          = "Auto-generated embedded firmware documentation"
OUTPUT_DIRECTORY       = {}

# Input settings
INPUT                  = src
FILE_PATTERNS          = *.c *.h *.cpp *.hpp
RECURSIVE              = YES

# Output settings
GENERATE_HTML          = YES
GENERATE_LATEX         = NO
HTML_OUTPUT            = html

# Extraction settings
EXTRACT_ALL            = YES
EXTRACT_PRIVATE        = YES
EXTRACT_STATIC         = YES

# Documentation settings
BRIEF_MEMBER_DESC      = YES
REPEAT_BRIEF           = YES
ALWAYS_DETAILED_SEC    = YES

# Diagram settings
HAVE_DOT               = YES
CALL_GRAPH             = YES
CALLER_GRAPH           = YES
UML_LOOK               = YES

# Warning settings
WARNINGS               = YES
WARN_IF_UNDOCUMENTED   = YES
"#, project_name, output_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_functions() {
        let code = r#"
void led_init(void) {
    // init code
}

int adc_read(uint8_t channel) {
    return 0;
}
"#;
        let functions = extract_functions(code);
        assert_eq!(functions.len(), 2);
        assert_eq!(functions[0].name, "led_init");
        assert_eq!(functions[1].name, "adc_read");
    }

    #[test]
    fn test_generate_function_doc() {
        let doc = FunctionDoc {
            name: "test_func".to_string(),
            brief: "Test function".to_string(),
            description: String::new(),
            params: vec![],
            returns: None,
            notes: vec![],
            examples: vec![],
        };
        
        let output = generate_function_doc(&doc);
        assert!(output.contains("@brief Test function"));
    }
}
