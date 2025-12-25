// Toolchain Module
// Build, flash, and debug integration for embedded development

pub mod discovery;
pub mod arm_gcc;
pub mod output_parser;
pub mod probe;
pub mod streaming_build;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

/// Toolchain errors
#[derive(Debug, Error)]
pub enum ToolchainError {
    #[error("Toolchain not found: {0}")]
    NotFound(String),
    
    #[error("Build failed: {0}")]
    BuildFailed(String),
    
    #[error("Flash failed: {0}")]
    FlashFailed(String),
    
    #[error("Probe error: {0}")]
    ProbeError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    ParseError(String),
}

/// Information about a discovered toolchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub toolchain_type: ToolchainType,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolchainType {
    ArmGcc,
    Clang,
    RustEmbedded,
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub project_path: PathBuf,
    pub output_dir: Option<PathBuf>,
    pub mcu_target: String,
    pub optimization: OptLevel,
    pub defines: HashMap<String, String>,
    pub include_paths: Vec<PathBuf>,
    pub source_files: Vec<PathBuf>,
    pub linker_script: Option<PathBuf>,
    pub toolchain_id: Option<String>,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            project_path: PathBuf::new(),
            output_dir: None,
            mcu_target: "cortex-m4".to_string(),
            optimization: OptLevel::Debug,
            defines: HashMap::new(),
            include_paths: vec![],
            source_files: vec![],
            linker_script: None,
            toolchain_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OptLevel {
    #[default]
    Debug,
    Release,
    MinSize,
    MaxSpeed,
}

impl OptLevel {
    pub fn as_gcc_flag(&self) -> &'static str {
        match self {
            OptLevel::Debug => "-Og",
            OptLevel::Release => "-O2",
            OptLevel::MinSize => "-Os",
            OptLevel::MaxSpeed => "-O3",
        }
    }
}

/// Result of a build operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub success: bool,
    pub elf_path: Option<PathBuf>,
    pub binary_path: Option<PathBuf>,
    pub errors: Vec<CompilerDiagnostic>,
    pub warnings: Vec<CompilerDiagnostic>,
    pub duration_ms: u64,
    pub output: String,
}

/// A compiler diagnostic (error or warning)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerDiagnostic {
    pub file: PathBuf,
    pub line: u32,
    pub column: Option<u32>,
    pub severity: DiagnosticSeverity,
    pub code: Option<String>,
    pub message: String,
    pub suggestion: Option<String>,
    pub context_lines: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Note,
    Info,
}

/// Size report for compiled binary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeReport {
    pub text: u64,
    pub data: u64,
    pub bss: u64,
    pub total: u64,
    pub flash_used: u64,
    pub ram_used: u64,
    pub flash_total: u64,
    pub ram_total: u64,
    pub flash_percent: f32,
    pub ram_percent: f32,
    pub sections: Vec<SectionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionInfo {
    pub name: String,
    pub size: u64,
    pub address: u64,
    pub section_type: SectionType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SectionType {
    Code,
    Data,
    Bss,
    RoData,
    Stack,
    Heap,
    Other,
}

/// Map file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapFileInfo {
    pub memory_regions: Vec<MemoryRegion>,
    pub symbols: Vec<SymbolEntry>,
    pub sections: Vec<SectionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRegion {
    pub name: String,
    pub origin: u64,
    pub length: u64,
    pub used: u64,
    pub attributes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolEntry {
    pub name: String,
    pub address: u64,
    pub size: u64,
    pub section: String,
    pub object_file: Option<String>,
}

/// Trait for toolchain implementations
pub trait Toolchain: Send + Sync {
    fn info(&self) -> &ToolchainInfo;
    
    fn build(&self, config: &BuildConfig) -> Result<BuildResult, ToolchainError>;
    
    fn clean(&self, project_path: &std::path::Path) -> Result<(), ToolchainError>;
    
    fn size(&self, elf_path: &std::path::Path) -> Result<SizeReport, ToolchainError>;
    
    fn parse_map(&self, map_path: &std::path::Path) -> Result<MapFileInfo, ToolchainError>;
}
