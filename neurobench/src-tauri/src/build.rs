// Build System Module
// Make/CMake integration for project building

use serde::{Deserialize, Serialize};
use std::process::Command;

/// Build system type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildSystem {
    Make,
    CMake,
    Cargo,
    PlatformIO,
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub system: String,
    pub target: String,
    pub optimization: String,  // O0, O1, O2, O3, Os, Og
    pub debug_symbols: bool,
    pub defines: Vec<String>,
    pub include_paths: Vec<String>,
    pub source_files: Vec<String>,
    pub linker_script: Option<String>,
}

/// Build result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub success: bool,
    pub output: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub binary_path: Option<String>,
    pub binary_size: Option<u32>,
}

/// Generate Makefile
pub fn generate_makefile(config: &BuildConfig) -> String {
    let mut makefile = String::new();
    
    makefile.push_str("# Auto-generated Makefile for NeuroBench project\n\n");
    
    // Toolchain
    makefile.push_str("# Toolchain\n");
    makefile.push_str("PREFIX = arm-none-eabi-\n");
    makefile.push_str("CC = $(PREFIX)gcc\n");
    makefile.push_str("AS = $(PREFIX)as\n");
    makefile.push_str("LD = $(PREFIX)ld\n");
    makefile.push_str("OBJCOPY = $(PREFIX)objcopy\n");
    makefile.push_str("SIZE = $(PREFIX)size\n\n");
    
    // Project
    makefile.push_str("# Project\n");
    makefile.push_str(&format!("TARGET = {}\n", config.target));
    makefile.push_str("BUILD_DIR = build\n\n");
    
    // Sources
    makefile.push_str("# Sources\n");
    makefile.push_str("SRCS = \\\n");
    for (i, src) in config.source_files.iter().enumerate() {
        if i < config.source_files.len() - 1 {
            makefile.push_str(&format!("    {} \\\n", src));
        } else {
            makefile.push_str(&format!("    {}\n", src));
        }
    }
    makefile.push_str("\n");
    
    // Flags
    makefile.push_str("# Compiler flags\n");
    makefile.push_str(&format!("CFLAGS = -{} -Wall -Wextra\n", config.optimization));
    
    if config.debug_symbols {
        makefile.push_str("CFLAGS += -g3\n");
    }
    
    makefile.push_str("CFLAGS += -mcpu=cortex-m4 -mthumb -mfloat-abi=hard -mfpu=fpv4-sp-d16\n");
    
    for define in &config.defines {
        makefile.push_str(&format!("CFLAGS += -D{}\n", define));
    }
    
    for include in &config.include_paths {
        makefile.push_str(&format!("CFLAGS += -I{}\n", include));
    }
    makefile.push_str("\n");
    
    // Linker
    makefile.push_str("# Linker flags\n");
    if let Some(script) = &config.linker_script {
        makefile.push_str(&format!("LDFLAGS = -T{}\n", script));
    }
    makefile.push_str("LDFLAGS += -Wl,--gc-sections\n\n");
    
    // Rules
    makefile.push_str("# Objects\n");
    makefile.push_str("OBJS = $(SRCS:%.c=$(BUILD_DIR)/%.o)\n\n");
    
    makefile.push_str("# Rules\n");
    makefile.push_str("all: $(BUILD_DIR)/$(TARGET).elf $(BUILD_DIR)/$(TARGET).bin\n\n");
    
    makefile.push_str("$(BUILD_DIR)/%.o: %.c\n");
    makefile.push_str("\t@mkdir -p $(dir $@)\n");
    makefile.push_str("\t$(CC) $(CFLAGS) -c $< -o $@\n\n");
    
    makefile.push_str("$(BUILD_DIR)/$(TARGET).elf: $(OBJS)\n");
    makefile.push_str("\t$(CC) $(CFLAGS) $(LDFLAGS) $^ -o $@\n");
    makefile.push_str("\t$(SIZE) $@\n\n");
    
    makefile.push_str("$(BUILD_DIR)/$(TARGET).bin: $(BUILD_DIR)/$(TARGET).elf\n");
    makefile.push_str("\t$(OBJCOPY) -O binary $< $@\n\n");
    
    makefile.push_str("clean:\n");
    makefile.push_str("\trm -rf $(BUILD_DIR)\n\n");
    
    makefile.push_str("flash: $(BUILD_DIR)/$(TARGET).bin\n");
    makefile.push_str("\tst-flash write $< 0x8000000\n\n");
    
    makefile.push_str(".PHONY: all clean flash\n");
    
    makefile
}

/// Generate CMakeLists.txt
pub fn generate_cmake(config: &BuildConfig) -> String {
    let mut cmake = String::new();
    
    cmake.push_str("# Auto-generated CMakeLists.txt for NeuroBench project\n");
    cmake.push_str("cmake_minimum_required(VERSION 3.20)\n\n");
    
    cmake.push_str(&format!("project({} LANGUAGES C ASM)\n\n", config.target));
    
    cmake.push_str("set(CMAKE_C_STANDARD 11)\n");
    cmake.push_str("set(CMAKE_C_STANDARD_REQUIRED ON)\n\n");
    
    // Toolchain
    cmake.push_str("# Toolchain\n");
    cmake.push_str("set(CMAKE_SYSTEM_NAME Generic)\n");
    cmake.push_str("set(CMAKE_SYSTEM_PROCESSOR arm)\n");
    cmake.push_str("set(CMAKE_C_COMPILER arm-none-eabi-gcc)\n");
    cmake.push_str("set(CMAKE_ASM_COMPILER arm-none-eabi-gcc)\n\n");
    
    // Flags
    cmake.push_str("# Compiler flags\n");
    cmake.push_str("set(MCU_FLAGS \"-mcpu=cortex-m4 -mthumb -mfloat-abi=hard -mfpu=fpv4-sp-d16\")\n");
    cmake.push_str(&format!("set(CMAKE_C_FLAGS \"${{MCU_FLAGS}} -{} -Wall -Wextra\")\n\n", 
        config.optimization));
    
    // Sources
    cmake.push_str("# Sources\n");
    cmake.push_str("set(SOURCES\n");
    for src in &config.source_files {
        cmake.push_str(&format!("    {}\n", src));
    }
    cmake.push_str(")\n\n");
    
    // Includes
    cmake.push_str("# Include directories\n");
    cmake.push_str("include_directories(\n");
    for inc in &config.include_paths {
        cmake.push_str(&format!("    {}\n", inc));
    }
    cmake.push_str(")\n\n");
    
    // Defines
    cmake.push_str("# Definitions\n");
    for define in &config.defines {
        cmake.push_str(&format!("add_definitions(-D{})\n", define));
    }
    cmake.push_str("\n");
    
    // Target
    cmake.push_str("# Executable\n");
    cmake.push_str(&format!("add_executable(${{PROJECT_NAME}}.elf ${{SOURCES}})\n\n"));
    
    // Linker
    if let Some(script) = &config.linker_script {
        cmake.push_str(&format!("set(LINKER_SCRIPT {})\n", script));
        cmake.push_str("target_link_options(${PROJECT_NAME}.elf PRIVATE\n");
        cmake.push_str("    -T${LINKER_SCRIPT}\n");
        cmake.push_str("    -Wl,--gc-sections\n");
        cmake.push_str(")\n\n");
    }
    
    // Post-build
    cmake.push_str("# Generate binary\n");
    cmake.push_str("add_custom_command(TARGET ${PROJECT_NAME}.elf POST_BUILD\n");
    cmake.push_str("    COMMAND arm-none-eabi-objcopy -O binary ${PROJECT_NAME}.elf ${PROJECT_NAME}.bin\n");
    cmake.push_str("    COMMAND arm-none-eabi-size ${PROJECT_NAME}.elf\n");
    cmake.push_str(")\n");
    
    cmake
}

/// Check if toolchain is available
pub fn check_toolchain() -> Vec<(String, bool)> {
    let tools = vec![
        ("arm-none-eabi-gcc", "ARM GCC Compiler"),
        ("make", "Make Build Tool"),
        ("cmake", "CMake"),
        ("st-flash", "ST-Link Flasher"),
        ("openocd", "OpenOCD"),
    ];
    
    tools.iter().map(|(cmd, name)| {
        let available = Command::new(cmd)
            .arg("--version")
            .output()
            .is_ok();
        (name.to_string(), available)
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_makefile() {
        let config = BuildConfig {
            system: "make".to_string(),
            target: "firmware".to_string(),
            optimization: "O2".to_string(),
            debug_symbols: true,
            defines: vec!["STM32F407xx".to_string()],
            include_paths: vec!["inc".to_string()],
            source_files: vec!["main.c".to_string()],
            linker_script: Some("stm32f407.ld".to_string()),
        };
        
        let makefile = generate_makefile(&config);
        assert!(makefile.contains("TARGET = firmware"));
    }

    #[test]
    fn test_generate_cmake() {
        let config = BuildConfig {
            system: "cmake".to_string(),
            target: "firmware".to_string(),
            optimization: "O2".to_string(),
            debug_symbols: true,
            defines: vec!["STM32F407xx".to_string()],
            include_paths: vec!["inc".to_string()],
            source_files: vec!["main.c".to_string()],
            linker_script: Some("stm32f407.ld".to_string()),
        };
        
        let cmake = generate_cmake(&config);
        assert!(cmake.contains("project(firmware"));
    }
}
