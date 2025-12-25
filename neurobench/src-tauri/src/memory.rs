// Memory Analyzer Module
// RAM/Flash usage visualization and analysis

use serde::{Deserialize, Serialize};

/// Memory region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRegion {
    pub name: String,
    pub start: u32,
    pub size: u32,
    pub used: u32,
    pub region_type: String,  // "flash", "ram", "ccm", "backup"
}

/// Symbol with memory usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub name: String,
    pub address: u32,
    pub size: u32,
    pub section: String,
    pub symbol_type: String,  // "function", "variable", "constant"
}

/// Memory map analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAnalysis {
    pub total_flash: u32,
    pub used_flash: u32,
    pub total_ram: u32,
    pub used_ram: u32,
    pub regions: Vec<MemoryRegion>,
    pub largest_symbols: Vec<SymbolInfo>,
    pub section_sizes: Vec<SectionSize>,
}

/// Section size info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionSize {
    pub name: String,
    pub size: u32,
    pub percent: f32,
}

/// MCU memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McuMemoryConfig {
    pub name: String,
    pub flash_start: u32,
    pub flash_size: u32,
    pub ram_start: u32,
    pub ram_size: u32,
    pub ccm_start: Option<u32>,
    pub ccm_size: Option<u32>,
}

/// Get MCU memory configurations
pub fn get_mcu_configs() -> Vec<McuMemoryConfig> {
    vec![
        McuMemoryConfig {
            name: "STM32F407VG".to_string(),
            flash_start: 0x0800_0000,
            flash_size: 1024 * 1024,  // 1MB
            ram_start: 0x2000_0000,
            ram_size: 128 * 1024,      // 128KB
            ccm_start: Some(0x1000_0000),
            ccm_size: Some(64 * 1024), // 64KB CCM
        },
        McuMemoryConfig {
            name: "STM32F103C8".to_string(),
            flash_start: 0x0800_0000,
            flash_size: 64 * 1024,    // 64KB
            ram_start: 0x2000_0000,
            ram_size: 20 * 1024,      // 20KB
            ccm_start: None,
            ccm_size: None,
        },
        McuMemoryConfig {
            name: "STM32F411RE".to_string(),
            flash_start: 0x0800_0000,
            flash_size: 512 * 1024,   // 512KB
            ram_start: 0x2000_0000,
            ram_size: 128 * 1024,     // 128KB
            ccm_start: None,
            ccm_size: None,
        },
        McuMemoryConfig {
            name: "ESP32".to_string(),
            flash_start: 0x0000_0000,
            flash_size: 4 * 1024 * 1024, // 4MB
            ram_start: 0x3FFB_0000,
            ram_size: 520 * 1024,        // 520KB
            ccm_start: None,
            ccm_size: None,
        },
        McuMemoryConfig {
            name: "nRF52832".to_string(),
            flash_start: 0x0000_0000,
            flash_size: 512 * 1024,   // 512KB
            ram_start: 0x2000_0000,
            ram_size: 64 * 1024,      // 64KB
            ccm_start: None,
            ccm_size: None,
        },
    ]
}

/// Estimate memory usage from code
pub fn estimate_memory(code: &str, mcu: &str) -> Result<MemoryAnalysis, String> {
    let config = get_mcu_configs()
        .into_iter()
        .find(|c| c.name.to_lowercase().contains(&mcu.to_lowercase()))
        .unwrap_or_else(|| get_mcu_configs()[0].clone());

    // Simple estimation based on code analysis
    let lines: Vec<&str> = code.lines().collect();
    
    // Count various elements
    let function_count = lines.iter()
        .filter(|l| l.contains("void ") || l.contains("int ") || l.contains("uint"))
        .filter(|l| l.contains("(") && l.contains(")"))
        .count();
    
    let variable_count = lines.iter()
        .filter(|l| l.contains("volatile") || l.contains("static") || l.contains("="))
        .filter(|l| l.ends_with(";"))
        .count();

    let string_literals: usize = lines.iter()
        .filter(|l| l.contains("\""))
        .map(|l| l.matches("\"").count() / 2)
        .sum();

    // Estimate sizes
    let estimated_text = function_count * 100;  // ~100 bytes per function avg
    let estimated_data = variable_count * 4;    // ~4 bytes per variable
    let estimated_rodata = string_literals * 20; // ~20 bytes per string avg
    let estimated_bss = variable_count * 2;     // Uninitialized data
    
    let used_flash = (estimated_text + estimated_rodata) as u32;
    let used_ram = (estimated_data + estimated_bss) as u32;

    let sections = vec![
        SectionSize {
            name: ".text (code)".to_string(),
            size: estimated_text as u32,
            percent: (estimated_text as f32 / config.flash_size as f32) * 100.0,
        },
        SectionSize {
            name: ".rodata (constants)".to_string(),
            size: estimated_rodata as u32,
            percent: (estimated_rodata as f32 / config.flash_size as f32) * 100.0,
        },
        SectionSize {
            name: ".data (initialized)".to_string(),
            size: estimated_data as u32,
            percent: (estimated_data as f32 / config.ram_size as f32) * 100.0,
        },
        SectionSize {
            name: ".bss (uninitialized)".to_string(),
            size: estimated_bss as u32,
            percent: (estimated_bss as f32 / config.ram_size as f32) * 100.0,
        },
    ];

    Ok(MemoryAnalysis {
        total_flash: config.flash_size,
        used_flash,
        total_ram: config.ram_size,
        used_ram,
        regions: vec![
            MemoryRegion {
                name: "FLASH".to_string(),
                start: config.flash_start,
                size: config.flash_size,
                used: used_flash,
                region_type: "flash".to_string(),
            },
            MemoryRegion {
                name: "RAM".to_string(),
                start: config.ram_start,
                size: config.ram_size,
                used: used_ram,
                region_type: "ram".to_string(),
            },
        ],
        largest_symbols: vec![],
        section_sizes: sections,
    })
}

/// Format bytes to human readable
pub fn format_bytes(bytes: u32) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes as f32 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f32 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_mcu_configs() {
        let configs = get_mcu_configs();
        assert!(!configs.is_empty());
    }

    #[test]
    fn test_estimate_memory() {
        let code = r#"
            void led_init(void) {}
            void led_toggle(void) {}
            volatile uint32_t counter = 0;
        "#;
        
        let result = estimate_memory(code, "STM32F407");
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2048), "2.00 KB");
    }
}
