// ARM GCC Toolchain Implementation
// Wrapper for arm-none-eabi-gcc build operations

use super::{
    Toolchain, ToolchainInfo, ToolchainType, ToolchainError,
    BuildConfig, BuildResult, SizeReport, MapFileInfo, MemoryRegion,
    output_parser,
};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// ARM GCC Toolchain implementation
pub struct ArmGcc {
    info: ToolchainInfo,
    gcc_path: PathBuf,
    objcopy_path: PathBuf,
    size_path: PathBuf,
}

impl ArmGcc {
    /// Create a new ARM GCC toolchain from discovered info
    pub fn new(info: ToolchainInfo) -> Self {
        let bin_dir = info.path.clone();
        
        Self {
            gcc_path: bin_dir.join("arm-none-eabi-gcc"),
            objcopy_path: bin_dir.join("arm-none-eabi-objcopy"),
            size_path: bin_dir.join("arm-none-eabi-size"),
            info,
        }
    }
    
    /// Create from a specific path
    pub fn from_path(path: PathBuf) -> Result<Self, ToolchainError> {
        let gcc_path = path.join("arm-none-eabi-gcc");
        
        #[cfg(target_os = "windows")]
        let gcc_path = path.join("arm-none-eabi-gcc.exe");
        
        if !gcc_path.exists() {
            return Err(ToolchainError::NotFound(
                format!("arm-none-eabi-gcc not found at {:?}", path)
            ));
        }
        
        // Get version
        let output = Command::new(&gcc_path)
            .arg("--version")
            .output()
            .map_err(|e| ToolchainError::IoError(e))?;
        
        let version = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("unknown")
            .to_string();
        
        let info = ToolchainInfo {
            id: "arm-gcc".to_string(),
            name: "ARM GNU Toolchain".to_string(),
            version,
            path: path.clone(),
            toolchain_type: ToolchainType::ArmGcc,
            targets: vec![
                "cortex-m0".to_string(),
                "cortex-m3".to_string(),
                "cortex-m4".to_string(),
                "cortex-m7".to_string(),
            ],
        };
        
        Ok(Self::new(info))
    }
    
    /// Get CPU flags for target
    fn cpu_flags(&self, target: &str) -> Vec<String> {
        match target.to_lowercase().as_str() {
            "cortex-m0" | "cortex-m0+" => vec![
                "-mcpu=cortex-m0".to_string(),
                "-mthumb".to_string(),
            ],
            "cortex-m3" => vec![
                "-mcpu=cortex-m3".to_string(),
                "-mthumb".to_string(),
            ],
            "cortex-m4" => vec![
                "-mcpu=cortex-m4".to_string(),
                "-mthumb".to_string(),
                "-mfloat-abi=hard".to_string(),
                "-mfpu=fpv4-sp-d16".to_string(),
            ],
            "cortex-m4f" => vec![
                "-mcpu=cortex-m4".to_string(),
                "-mthumb".to_string(),
                "-mfloat-abi=hard".to_string(),
                "-mfpu=fpv4-sp-d16".to_string(),
            ],
            "cortex-m7" => vec![
                "-mcpu=cortex-m7".to_string(),
                "-mthumb".to_string(),
                "-mfloat-abi=hard".to_string(),
                "-mfpu=fpv5-sp-d16".to_string(),
            ],
            "cortex-m33" => vec![
                "-mcpu=cortex-m33".to_string(),
                "-mthumb".to_string(),
                "-mfloat-abi=hard".to_string(),
                "-mfpu=fpv5-sp-d16".to_string(),
            ],
            _ => vec![
                "-mcpu=cortex-m4".to_string(),
                "-mthumb".to_string(),
            ],
        }
    }
    
    /// Build a single source file to object
    fn compile_file(
        &self,
        source: &Path,
        output: &Path,
        config: &BuildConfig,
    ) -> Result<String, ToolchainError> {
        let mut cmd = Command::new(&self.gcc_path);
        
        // CPU flags
        for flag in self.cpu_flags(&config.mcu_target) {
            cmd.arg(&flag);
        }
        
        // Optimization
        cmd.arg(config.optimization.as_gcc_flag());
        
        // Common embedded flags
        cmd.args(["-Wall", "-Wextra", "-ffunction-sections", "-fdata-sections"]);
        cmd.args(["-ffreestanding", "-nostdlib"]);
        
        // Debug info
        cmd.arg("-g3");
        
        // Include paths
        for inc in &config.include_paths {
            cmd.arg("-I").arg(inc);
        }
        
        // Defines
        for (key, value) in &config.defines {
            if value.is_empty() {
                cmd.arg(format!("-D{}", key));
            } else {
                cmd.arg(format!("-D{}={}", key, value));
            }
        }
        
        // Compile only, output object
        cmd.arg("-c")
           .arg(source)
           .arg("-o")
           .arg(output);
        
        let output = cmd.output().map_err(|e| ToolchainError::IoError(e))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        Ok(format!("{}{}", stdout, stderr))
    }
    
    /// Link object files into ELF
    fn link_objects(
        &self,
        objects: &[PathBuf],
        output: &Path,
        config: &BuildConfig,
    ) -> Result<String, ToolchainError> {
        let mut cmd = Command::new(&self.gcc_path);
        
        // CPU flags
        for flag in self.cpu_flags(&config.mcu_target) {
            cmd.arg(&flag);
        }
        
        // Linker flags
        cmd.args(["-Wl,--gc-sections", "-Wl,-Map=output.map"]);
        cmd.arg("--specs=nosys.specs");
        cmd.arg("--specs=nano.specs");
        
        // Linker script
        if let Some(ref ld_script) = config.linker_script {
            cmd.arg("-T").arg(ld_script);
        }
        
        // Input objects
        for obj in objects {
            cmd.arg(obj);
        }
        
        // Output ELF
        cmd.arg("-o").arg(output);
        
        let output = cmd.output().map_err(|e| ToolchainError::IoError(e))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        Ok(format!("{}{}", stdout, stderr))
    }
    
    /// Convert ELF to binary
    pub fn objcopy(&self, elf: &Path, output: &Path) -> Result<(), ToolchainError> {
        let status = Command::new(&self.objcopy_path)
            .args(["-O", "binary"])
            .arg(elf)
            .arg(output)
            .status()
            .map_err(|e| ToolchainError::IoError(e))?;
        
        if !status.success() {
            return Err(ToolchainError::BuildFailed("objcopy failed".to_string()));
        }
        
        Ok(())
    }
}

impl Toolchain for ArmGcc {
    fn info(&self) -> &ToolchainInfo {
        &self.info
    }
    
    fn build(&self, config: &BuildConfig) -> Result<BuildResult, ToolchainError> {
        let start = Instant::now();
        let mut all_output = String::new();
        let mut objects = Vec::new();
        
        // Create build directory
        let build_dir = config.output_dir.clone()
            .unwrap_or_else(|| config.project_path.join("build"));
        std::fs::create_dir_all(&build_dir)?;
        
        // Compile each source file
        for source in &config.source_files {
            let obj_name = source
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();
            let obj_path = build_dir.join(format!("{}.o", obj_name));
            
            let output = self.compile_file(source, &obj_path, config)?;
            all_output.push_str(&output);
            
            if obj_path.exists() {
                objects.push(obj_path);
            }
        }
        
        // Parse compilation output for errors
        let (errors, warnings) = output_parser::parse_compiler_output(&all_output);
        
        // If there are errors, don't link
        if !errors.is_empty() {
            return Ok(BuildResult {
                success: false,
                elf_path: None,
                binary_path: None,
                errors,
                warnings,
                duration_ms: start.elapsed().as_millis() as u64,
                output: all_output,
            });
        }
        
        // Link
        let elf_path = build_dir.join("firmware.elf");
        let link_output = self.link_objects(&objects, &elf_path, config)?;
        all_output.push_str(&link_output);
        
        let (link_errors, link_warnings) = output_parser::parse_compiler_output(&link_output);
        let mut all_errors = errors;
        let mut all_warnings = warnings;
        all_errors.extend(link_errors);
        all_warnings.extend(link_warnings);
        
        let success = elf_path.exists() && all_errors.is_empty();
        
        // Generate binary if successful
        let binary_path = if success {
            let bin_path = build_dir.join("firmware.bin");
            self.objcopy(&elf_path, &bin_path).ok();
            if bin_path.exists() {
                Some(bin_path)
            } else {
                None
            }
        } else {
            None
        };
        
        Ok(BuildResult {
            success,
            elf_path: if success { Some(elf_path) } else { None },
            binary_path,
            errors: all_errors,
            warnings: all_warnings,
            duration_ms: start.elapsed().as_millis() as u64,
            output: all_output,
        })
    }
    
    fn clean(&self, project_path: &Path) -> Result<(), ToolchainError> {
        let build_dir = project_path.join("build");
        if build_dir.exists() {
            std::fs::remove_dir_all(&build_dir)?;
        }
        Ok(())
    }
    
    fn size(&self, elf_path: &Path) -> Result<SizeReport, ToolchainError> {
        let output = Command::new(&self.size_path)
            .arg(elf_path)
            .output()
            .map_err(|e| ToolchainError::IoError(e))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Default STM32F4 sizes (can be overridden based on MCU)
        let flash_total = 512 * 1024; // 512KB
        let ram_total = 128 * 1024;   // 128KB
        
        output_parser::parse_size_output(&stdout, flash_total, ram_total)
            .ok_or_else(|| ToolchainError::ParseError("Failed to parse size output".to_string()))
    }
    
    fn parse_map(&self, map_path: &Path) -> Result<MapFileInfo, ToolchainError> {
        let content = std::fs::read_to_string(map_path)?;
        let symbols = output_parser::parse_map_file(&content);
        
        // Parse memory regions from map file
        let memory_regions = parse_memory_regions(&content);
        
        Ok(MapFileInfo {
            memory_regions,
            symbols,
            sections: vec![],
        })
    }
}

/// Parse memory regions from GNU ld map file
fn parse_memory_regions(content: &str) -> Vec<MemoryRegion> {
    let mut regions = Vec::new();
    
    // Look for MEMORY section
    let re = regex::Regex::new(
        r"(?m)^\s*(\w+)\s+\(([rwx]+)\)\s*:\s*ORIGIN\s*=\s*(0x[0-9a-fA-F]+)\s*,\s*LENGTH\s*=\s*(0x[0-9a-fA-F]+|\d+[KkMm]?)"
    ).ok();
    
    if let Some(re) = re {
        for cap in re.captures_iter(content) {
            let name = cap[1].to_string();
            let attributes = cap[2].to_string();
            let origin = u64::from_str_radix(cap[3].trim_start_matches("0x"), 16).unwrap_or(0);
            let length_str = &cap[4];
            
            let length = if length_str.starts_with("0x") {
                u64::from_str_radix(length_str.trim_start_matches("0x"), 16).unwrap_or(0)
            } else if length_str.to_lowercase().ends_with('k') {
                let num: u64 = length_str[..length_str.len()-1].parse().unwrap_or(0);
                num * 1024
            } else if length_str.to_lowercase().ends_with('m') {
                let num: u64 = length_str[..length_str.len()-1].parse().unwrap_or(0);
                num * 1024 * 1024
            } else {
                length_str.parse().unwrap_or(0)
            };
            
            regions.push(MemoryRegion {
                name,
                origin,
                length,
                used: 0, // Would need to parse section placement
                attributes,
            });
        }
    }
    
    // Default regions if none found
    if regions.is_empty() {
        regions.push(MemoryRegion {
            name: "FLASH".to_string(),
            origin: 0x08000000,
            length: 512 * 1024,
            used: 0,
            attributes: "rx".to_string(),
        });
        regions.push(MemoryRegion {
            name: "RAM".to_string(),
            origin: 0x20000000,
            length: 128 * 1024,
            used: 0,
            attributes: "rwx".to_string(),
        });
    }
    
    regions
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cpu_flags() {
        let info = ToolchainInfo {
            id: "arm-gcc".to_string(),
            name: "ARM GCC".to_string(),
            version: "10.3".to_string(),
            path: PathBuf::from("/usr/bin"),
            toolchain_type: ToolchainType::ArmGcc,
            targets: vec![],
        };
        let gcc = ArmGcc::new(info);
        
        let flags = gcc.cpu_flags("cortex-m4");
        assert!(flags.contains(&"-mcpu=cortex-m4".to_string()));
        assert!(flags.contains(&"-mfloat-abi=hard".to_string()));
    }
}
