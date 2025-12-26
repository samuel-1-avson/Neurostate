//! Memory Persistence
//!
//! Saves and loads agent memory to/from disk for persistence across sessions.

use super::memory::{AgentMemory, MemoryManager};
use std::path::PathBuf;
use std::fs;
use serde_json;

/// Memory storage configuration
pub struct MemoryStorage {
    base_path: PathBuf,
}

impl MemoryStorage {
    /// Create a new memory storage with default path
    pub fn new() -> Self {
        let base_path = Self::get_default_path();
        Self { base_path }
    }
    
    /// Create with custom path
    pub fn with_path(path: PathBuf) -> Self {
        Self { base_path: path }
    }
    
    /// Get default storage path (app data directory)
    fn get_default_path() -> PathBuf {
        // Try to get app data directory, fall back to current dir
        if let Some(data_dir) = dirs::data_dir() {
            data_dir.join("NeuroBench").join("memory")
        } else {
            PathBuf::from(".").join(".neurobench").join("memory")
        }
    }
    
    /// Ensure storage directory exists
    fn ensure_dir(&self) -> Result<(), String> {
        fs::create_dir_all(&self.base_path)
            .map_err(|e| format!("Failed to create memory directory: {}", e))
    }
    
    /// Get path for an agent's memory file
    fn agent_path(&self, agent_id: &str) -> PathBuf {
        self.base_path.join(format!("{}_memory.json", agent_id))
    }
    
    /// Save a single agent's memory
    pub fn save_agent_memory(&self, memory: &AgentMemory) -> Result<(), String> {
        self.ensure_dir()?;
        
        let path = self.agent_path(&memory.agent_id);
        let json = serde_json::to_string_pretty(memory)
            .map_err(|e| format!("Failed to serialize memory: {}", e))?;
        
        fs::write(&path, json)
            .map_err(|e| format!("Failed to write memory file: {}", e))?;
        
        log::info!("Saved memory for agent {} to {:?}", memory.agent_id, path);
        Ok(())
    }
    
    /// Load a single agent's memory
    pub fn load_agent_memory(&self, agent_id: &str) -> Result<Option<AgentMemory>, String> {
        let path = self.agent_path(agent_id);
        
        if !path.exists() {
            return Ok(None);
        }
        
        let json = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read memory file: {}", e))?;
        
        let memory: AgentMemory = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse memory file: {}", e))?;
        
        log::info!("Loaded memory for agent {} from {:?}", agent_id, path);
        Ok(Some(memory))
    }
    
    /// Save all memories from a manager
    pub fn save_all(&self, manager: &MemoryManager) -> Result<(), String> {
        self.ensure_dir()?;
        
        for agent_id in &["director", "code", "debug", "hardware", "fsm", "canvas", "build", "deploy", "docs"] {
            if let Some(memory) = manager.get(agent_id) {
                self.save_agent_memory(memory)?;
            }
        }
        
        Ok(())
    }
    
    /// Load all memories into a manager
    pub fn load_all(&self, manager: &mut MemoryManager) -> Result<(), String> {
        for agent_id in &["director", "code", "debug", "hardware", "fsm", "canvas", "build", "deploy", "docs"] {
            if let Some(memory) = self.load_agent_memory(agent_id)? {
                // Get or create returns &mut, so we need to update it
                let mem = manager.get_or_create(agent_id);
                *mem = memory;
            }
        }
        
        Ok(())
    }
    
    /// Delete memory for an agent
    pub fn delete_agent_memory(&self, agent_id: &str) -> Result<(), String> {
        let path = self.agent_path(agent_id);
        
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("Failed to delete memory file: {}", e))?;
        }
        
        Ok(())
    }
    
    /// Delete all memories
    pub fn delete_all(&self) -> Result<(), String> {
        if self.base_path.exists() {
            fs::remove_dir_all(&self.base_path)
                .map_err(|e| format!("Failed to delete memory directory: {}", e))?;
        }
        
        Ok(())
    }
    
    /// Get storage statistics
    pub fn get_stats(&self) -> MemoryStats {
        let mut stats = MemoryStats::default();
        
        if let Ok(entries) = fs::read_dir(&self.base_path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    stats.file_count += 1;
                    stats.total_bytes += metadata.len();
                }
            }
        }
        
        stats
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about stored memories
#[derive(Debug, Default, Clone)]
pub struct MemoryStats {
    pub file_count: usize,
    pub total_bytes: u64,
}

impl MemoryStats {
    pub fn total_kb(&self) -> f64 {
        self.total_bytes as f64 / 1024.0
    }
}

/// Auto-save wrapper for memory manager
pub struct PersistedMemoryManager {
    manager: MemoryManager,
    storage: MemoryStorage,
    dirty: bool,
}

impl PersistedMemoryManager {
    /// Create a new persisted manager and load existing memories
    pub fn new() -> Self {
        let storage = MemoryStorage::new();
        let mut manager = MemoryManager::new();
        
        // Try to load existing memories
        if let Err(e) = storage.load_all(&mut manager) {
            log::warn!("Failed to load memories: {}", e);
        }
        
        Self {
            manager,
            storage,
            dirty: false,
        }
    }
    
    /// Get or create memory for an agent
    pub fn get_or_create(&mut self, agent_id: &str) -> &mut AgentMemory {
        self.dirty = true;
        self.manager.get_or_create(agent_id)
    }
    
    /// Get memory for an agent (read-only)
    pub fn get(&self, agent_id: &str) -> Option<&AgentMemory> {
        self.manager.get(agent_id)
    }
    
    /// Save all memories to disk
    pub fn save(&mut self) -> Result<(), String> {
        self.storage.save_all(&self.manager)?;
        self.dirty = false;
        Ok(())
    }
    
    /// Save if there are unsaved changes
    pub fn save_if_dirty(&mut self) -> Result<(), String> {
        if self.dirty {
            self.save()
        } else {
            Ok(())
        }
    }
    
    /// Get storage statistics
    pub fn stats(&self) -> MemoryStats {
        self.storage.get_stats()
    }
}

impl Default for PersistedMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_memory_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = MemoryStorage::with_path(temp_dir.path().to_path_buf());
        
        // Create and save memory
        let mut memory = AgentMemory::new("test_agent");
        memory.record_user_message("Hello");
        memory.record_assistant_message("Hi there!");
        
        storage.save_agent_memory(&memory).unwrap();
        
        // Load memory
        let loaded = storage.load_agent_memory("test_agent").unwrap();
        assert!(loaded.is_some());
        
        let loaded = loaded.unwrap();
        assert_eq!(loaded.agent_id, "test_agent");
    }
}
