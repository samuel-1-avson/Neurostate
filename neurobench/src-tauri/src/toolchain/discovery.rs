// Toolchain Discovery
// Auto-detect installed toolchains (ARM GCC, Clang, Rust embedded)

use super::{ToolchainInfo, ToolchainType, ToolchainError};
use std::path::PathBuf;
use std::process::Command;

/// Discover all available toolchains on the system
pub fn discover_all() -> Vec<ToolchainInfo> {
    let mut toolchains = Vec::new();
    
    // Try to find ARM GCC
    if let Some(info) = discover_arm_gcc() {
        toolchains.push(info);
    }
    
    // Try to find ARM Clang
    if let Some(info) = discover_arm_clang() {
        toolchains.push(info);
    }
    
    // Try to find Rust embedded toolchain
    if let Some(info) = discover_rust_embedded() {
        toolchains.push(info);
    }
    
    toolchains
}

/// Discover ARM GCC toolchain
pub fn discover_arm_gcc() -> Option<ToolchainInfo> {
    // Common ARM GCC executable names
    let executables = [
        "arm-none-eabi-gcc",
        "arm-none-eabi-gcc.exe",
    ];
    
    for exe in executables {
        if let Ok(output) = Command::new(exe).arg("--version").output() {
            if output.status.success() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                let version = parse_gcc_version(&version_str);
                
                // Try to find the path
                let path = which_path(exe).unwrap_or_else(|| PathBuf::from(exe));
                
                return Some(ToolchainInfo {
                    id: "arm-gcc".to_string(),
                    name: "ARM GNU Toolchain".to_string(),
                    version,
                    path,
                    toolchain_type: ToolchainType::ArmGcc,
                    targets: vec![
                        "cortex-m0".to_string(),
                        "cortex-m0+".to_string(),
                        "cortex-m3".to_string(),
                        "cortex-m4".to_string(),
                        "cortex-m4f".to_string(),
                        "cortex-m7".to_string(),
                        "cortex-m33".to_string(),
                    ],
                });
            }
        }
    }
    
    // Check common installation paths on Windows
    #[cfg(target_os = "windows")]
    {
        let common_paths = [
            r"C:\Program Files\GNU Arm Embedded Toolchain",
            r"C:\Program Files (x86)\GNU Arm Embedded Toolchain",
            r"C:\Program Files\arm-gnu-toolchain",
        ];
        
        for base in common_paths {
            if let Ok(entries) = std::fs::read_dir(base) {
                for entry in entries.flatten() {
                    let bin_path = entry.path().join("bin").join("arm-none-eabi-gcc.exe");
                    if bin_path.exists() {
                        if let Ok(output) = Command::new(&bin_path).arg("--version").output() {
                            if output.status.success() {
                                let version_str = String::from_utf8_lossy(&output.stdout);
                                let version = parse_gcc_version(&version_str);
                                
                                return Some(ToolchainInfo {
                                    id: "arm-gcc".to_string(),
                                    name: "ARM GNU Toolchain".to_string(),
                                    version,
                                    path: entry.path().join("bin"),
                                    toolchain_type: ToolchainType::ArmGcc,
                                    targets: vec![
                                        "cortex-m0".to_string(),
                                        "cortex-m0+".to_string(),
                                        "cortex-m3".to_string(),
                                        "cortex-m4".to_string(),
                                        "cortex-m4f".to_string(),
                                        "cortex-m7".to_string(),
                                        "cortex-m33".to_string(),
                                    ],
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    None
}

/// Discover ARM Clang toolchain
fn discover_arm_clang() -> Option<ToolchainInfo> {
    let executables = [
        "armclang",
        "armclang.exe",
    ];
    
    for exe in executables {
        if let Ok(output) = Command::new(exe).arg("--version").output() {
            if output.status.success() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                let version = parse_clang_version(&version_str);
                let path = which_path(exe).unwrap_or_else(|| PathBuf::from(exe));
                
                return Some(ToolchainInfo {
                    id: "arm-clang".to_string(),
                    name: "ARM Compiler".to_string(),
                    version,
                    path,
                    toolchain_type: ToolchainType::Clang,
                    targets: vec![
                        "cortex-m0".to_string(),
                        "cortex-m3".to_string(),
                        "cortex-m4".to_string(),
                        "cortex-m7".to_string(),
                    ],
                });
            }
        }
    }
    
    None
}

/// Discover Rust embedded toolchain
fn discover_rust_embedded() -> Option<ToolchainInfo> {
    // Check if cargo is available
    if let Ok(output) = Command::new("cargo").arg("--version").output() {
        if output.status.success() {
            let version_str = String::from_utf8_lossy(&output.stdout);
            let version = version_str.trim().to_string();
            
            // Check if thumbv7em target is installed
            if let Ok(targets_output) = Command::new("rustup")
                .args(["target", "list", "--installed"])
                .output() 
            {
                let installed = String::from_utf8_lossy(&targets_output.stdout);
                let has_thumb = installed.contains("thumbv");
                
                if has_thumb {
                    let path = which_path("cargo").unwrap_or_else(|| PathBuf::from("cargo"));
                    
                    // Parse installed targets
                    let targets: Vec<String> = installed
                        .lines()
                        .filter(|l| l.contains("thumb"))
                        .map(|s| s.trim().to_string())
                        .collect();
                    
                    return Some(ToolchainInfo {
                        id: "rust-embedded".to_string(),
                        name: "Rust Embedded".to_string(),
                        version,
                        path,
                        toolchain_type: ToolchainType::RustEmbedded,
                        targets,
                    });
                }
            }
        }
    }
    
    None
}

/// Parse GCC version from --version output
fn parse_gcc_version(output: &str) -> String {
    // Example: "arm-none-eabi-gcc (GNU Arm Embedded Toolchain 10.3-2021.10) 10.3.1 20210824"
    if let Some(line) = output.lines().next() {
        // Try to extract version number
        if let Some(start) = line.rfind(')') {
            let after_paren = &line[start + 1..];
            if let Some(version) = after_paren.split_whitespace().next() {
                return version.to_string();
            }
        }
        // Fallback: return first line
        return line.trim().to_string();
    }
    "unknown".to_string()
}

/// Parse Clang version from --version output
fn parse_clang_version(output: &str) -> String {
    if let Some(line) = output.lines().next() {
        return line.trim().to_string();
    }
    "unknown".to_string()
}

/// Find the full path to an executable
fn which_path(exe: &str) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = Command::new("where").arg(exe).output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = path_str.lines().next() {
                    return Some(PathBuf::from(first_line.trim()));
                }
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(output) = Command::new("which").arg(exe).output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                return Some(PathBuf::from(path_str.trim()));
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_gcc_version() {
        let output = "arm-none-eabi-gcc (GNU Arm Embedded Toolchain 10.3-2021.10) 10.3.1 20210824";
        assert_eq!(parse_gcc_version(output), "10.3.1");
    }
    
    #[test]
    fn test_discover_all() {
        // This will just run without panicking
        let toolchains = discover_all();
        println!("Found {} toolchains", toolchains.len());
        for tc in &toolchains {
            println!("  - {} v{}", tc.name, tc.version);
        }
    }
}
