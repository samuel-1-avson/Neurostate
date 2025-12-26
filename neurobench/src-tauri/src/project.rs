// Project Module
//
// Provides project manifest management for persistent projects.
// Each project has a project.json file with metadata and config.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use chrono::Utc;

/// Project manifest stored in project.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub id: String,              // UUID
    pub name: String,
    pub mcu_target: String,      // Primary target e.g., "STM32F407VG"
    pub chip: Option<String>,    // probe-rs chip name
    pub created_at: String,      // ISO 8601
    pub modified_at: String,     // ISO 8601
    pub config_hash: String,     // for artifact matching
    pub settings: ProjectSettings,
    #[serde(default)]
    pub targets: Vec<TargetProfile>,  // Multi-target support
}

/// Project-specific settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectSettings {
    pub clock_speed_hz: Option<u32>,
    pub flash_size_kb: Option<u32>,
    pub ram_size_kb: Option<u32>,
    pub rtos: Option<String>,
    pub optimization: Option<String>,
    pub defines: HashMap<String, String>,
}

/// Target profile for multi-target builds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetProfile {
    pub id: String,              // e.g., "stm32f4", "nrf52"
    pub name: String,            // Display name
    pub mcu: String,             // MCU identifier
    pub chip: String,            // probe-rs chip name
    pub enabled: bool,           // Whether to include in builds
    pub output_dir: String,      // e.g., "build/stm32f4"
    pub defines: HashMap<String, String>,  // Target-specific defines
    pub link_script: Option<String>,       // Custom linker script
}

/// Project info for listing (lighter weight than full manifest)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub mcu_target: String,
    pub path: String,
    pub modified_at: String,
}

/// Generated file from code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
    pub language: String,
    pub is_generated: bool,
}

/// File diff for preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub path: String,
    pub old_content: Option<String>,
    pub new_content: String,
    pub additions: usize,
    pub deletions: usize,
}

const MANIFEST_FILE: &str = "project.json";

impl ProjectManifest {
    /// Create a new project manifest
    pub fn new(name: &str, mcu_target: &str) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            mcu_target: mcu_target.to_string(),
            chip: None,
            created_at: now.clone(),
            modified_at: now,
            config_hash: String::new(),
            settings: ProjectSettings::default(),
            targets: Vec::new(),
        }
    }
    
    /// Update the config hash based on settings
    pub fn update_hash(&mut self) {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        format!("{:?}{:?}", self.mcu_target, self.settings).hash(&mut hasher);
        self.config_hash = format!("{:016x}", hasher.finish());
    }
    
    /// Mark as modified
    pub fn touch(&mut self) {
        self.modified_at = Utc::now().to_rfc3339();
        self.update_hash();
    }
    
    /// Add a target profile
    pub fn add_target(&mut self, target: TargetProfile) {
        // Remove existing with same ID
        self.targets.retain(|t| t.id != target.id);
        self.targets.push(target);
        self.touch();
    }
    
    /// Remove a target profile by ID
    pub fn remove_target(&mut self, id: &str) -> bool {
        let len_before = self.targets.len();
        self.targets.retain(|t| t.id != id);
        if self.targets.len() != len_before {
            self.touch();
            true
        } else {
            false
        }
    }
    
    /// Get enabled targets
    pub fn enabled_targets(&self) -> Vec<&TargetProfile> {
        self.targets.iter().filter(|t| t.enabled).collect()
    }
}

/// Load a project manifest from a directory
pub fn load_project(path: &str) -> Result<ProjectManifest, String> {
    let manifest_path = Path::new(path).join(MANIFEST_FILE);
    
    if !manifest_path.exists() {
        return Err(format!("Project manifest not found: {}", manifest_path.display()));
    }
    
    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;
    
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse manifest: {}", e))
}

/// Save a project manifest to a directory
pub fn save_project(path: &str, manifest: &ProjectManifest) -> Result<(), String> {
    let dir_path = Path::new(path);
    
    // Create directory if needed
    if !dir_path.exists() {
        fs::create_dir_all(dir_path)
            .map_err(|e| format!("Failed to create project directory: {}", e))?;
    }
    
    let manifest_path = dir_path.join(MANIFEST_FILE);
    let content = serde_json::to_string_pretty(manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
    
    fs::write(&manifest_path, content)
        .map_err(|e| format!("Failed to write manifest: {}", e))
}

/// Create a new project in the specified directory
pub fn create_project(path: &str, name: &str, mcu_target: &str) -> Result<ProjectManifest, String> {
    let dir_path = Path::new(path);
    
    // Check if project already exists
    if dir_path.join(MANIFEST_FILE).exists() {
        return Err("Project already exists in this directory".to_string());
    }
    
    // Create manifest
    let mut manifest = ProjectManifest::new(name, mcu_target);
    manifest.chip = Some(mcu_target.to_string());
    manifest.update_hash();
    
    // Save it
    save_project(path, &manifest)?;
    
    // Create basic directory structure
    let src_dir = dir_path.join("src");
    if !src_dir.exists() {
        fs::create_dir_all(&src_dir)
            .map_err(|e| format!("Failed to create src directory: {}", e))?;
    }
    
    Ok(manifest)
}

/// List projects in a directory
pub fn list_projects(dir: &str) -> Result<Vec<ProjectInfo>, String> {
    let dir_path = Path::new(dir);
    
    if !dir_path.exists() {
        return Ok(Vec::new());
    }
    
    let mut projects = Vec::new();
    
    let entries = fs::read_dir(dir_path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            let manifest_path = path.join(MANIFEST_FILE);
            if manifest_path.exists() {
                if let Ok(manifest) = load_project(path.to_str().unwrap_or("")) {
                    projects.push(ProjectInfo {
                        id: manifest.id,
                        name: manifest.name,
                        mcu_target: manifest.mcu_target,
                        path: path.to_string_lossy().to_string(),
                        modified_at: manifest.modified_at,
                    });
                }
            }
        }
    }
    
    // Sort by modified_at descending
    projects.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
    
    Ok(projects)
}

/// Delete a project
pub fn delete_project(path: &str) -> Result<(), String> {
    let manifest_path = Path::new(path).join(MANIFEST_FILE);
    
    if !manifest_path.exists() {
        return Err("Not a valid project directory".to_string());
    }
    
    // Only delete the manifest file, not the whole directory
    fs::remove_file(&manifest_path)
        .map_err(|e| format!("Failed to delete project: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_create_project() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        
        let manifest = create_project(path, "Test Project", "STM32F407VG").unwrap();
        
        assert_eq!(manifest.name, "Test Project");
        assert_eq!(manifest.mcu_target, "STM32F407VG");
        assert!(!manifest.id.is_empty());
        assert!(!manifest.config_hash.is_empty());
    }
    
    #[test]
    fn test_load_save_project() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        
        let mut manifest = ProjectManifest::new("My Project", "STM32F103C8");
        manifest.settings.clock_speed_hz = Some(72_000_000);
        manifest.touch();
        
        save_project(path, &manifest).unwrap();
        
        let loaded = load_project(path).unwrap();
        assert_eq!(loaded.name, "My Project");
        assert_eq!(loaded.settings.clock_speed_hz, Some(72_000_000));
    }
    
    #[test]
    fn test_list_projects() {
        let dir = tempdir().unwrap();
        let base = dir.path();
        
        // Create two projects
        let path1 = base.join("project1");
        let path2 = base.join("project2");
        
        create_project(path1.to_str().unwrap(), "Project 1", "STM32F407VG").unwrap();
        create_project(path2.to_str().unwrap(), "Project 2", "nRF52840").unwrap();
        
        let projects = list_projects(base.to_str().unwrap()).unwrap();
        assert_eq!(projects.len(), 2);
    }
}
