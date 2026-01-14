//! Agent Memory System
//!
//! Provides persistent context across sessions for agents.
//! Three-tier memory: short-term, working, and long-term.
//!
//! Enhanced with semantic search via vector embeddings.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc};

// Note: Vector store integration is handled by SemanticMemoryManager in memory_storage.rs

/// Memory entry with timestamp and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub category: MemoryCategory,
    pub importance: f32,  // 0.0 to 1.0
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u32,
    pub metadata: HashMap<String, String>,
}

/// Memory categories for organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MemoryCategory {
    /// User preferences and patterns
    UserPreference,
    /// Project-specific knowledge
    ProjectContext,
    /// MCU/hardware configurations
    HardwareConfig,
    /// Common errors and solutions
    ErrorPattern,
    /// Code patterns and templates
    CodePattern,
    /// Conversation context
    Conversation,
    /// Task execution history
    TaskHistory,
}

impl MemoryEntry {
    pub fn new(content: &str, category: MemoryCategory, importance: f32) -> Self {
        let now = Utc::now();
        Self {
            id: format!("mem_{}", now.timestamp_millis()),
            content: content.to_string(),
            category,
            importance: importance.clamp(0.0, 1.0),
            created_at: now,
            last_accessed: now,
            access_count: 0,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Calculate relevance score based on recency and importance
    pub fn relevance_score(&self) -> f32 {
        let age_hours = (Utc::now() - self.last_accessed).num_hours() as f32;
        let recency_factor = 1.0 / (1.0 + age_hours / 24.0); // Decay over ~24 hours
        let frequency_factor = (self.access_count as f32).log2().max(1.0) / 10.0;
        
        (self.importance * 0.5) + (recency_factor * 0.3) + (frequency_factor * 0.2)
    }
}

/// Short-term memory - recent conversation context
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShortTermMemory {
    /// Recent messages (FIFO)
    messages: VecDeque<MemoryEntry>,
    /// Maximum entries to keep
    max_entries: usize,
}

impl ShortTermMemory {
    pub fn new(max_entries: usize) -> Self {
        Self {
            messages: VecDeque::new(),
            max_entries,
        }
    }
    
    pub fn add(&mut self, content: &str) {
        let entry = MemoryEntry::new(content, MemoryCategory::Conversation, 0.5);
        self.messages.push_back(entry);
        
        while self.messages.len() > self.max_entries {
            self.messages.pop_front();
        }
    }
    
    pub fn get_recent(&self, count: usize) -> Vec<&MemoryEntry> {
        self.messages.iter().rev().take(count).collect()
    }
    
    pub fn to_context_string(&self) -> String {
        self.messages
            .iter()
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    pub fn clear(&mut self) {
        self.messages.clear();
    }
}

/// Working memory - current task context
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkingMemory {
    /// Current task description
    pub current_task: Option<String>,
    /// Active variables/state
    pub variables: HashMap<String, serde_json::Value>,
    /// Temporary notes
    pub notes: Vec<String>,
    /// Current focus (selected node, file, etc.)
    pub focus: Option<String>,
}

impl WorkingMemory {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn set_task(&mut self, task: &str) {
        self.current_task = Some(task.to_string());
    }
    
    pub fn clear_task(&mut self) {
        self.current_task = None;
        self.variables.clear();
        self.notes.clear();
    }
    
    pub fn set_variable(&mut self, key: &str, value: serde_json::Value) {
        self.variables.insert(key.to_string(), value);
    }
    
    pub fn get_variable(&self, key: &str) -> Option<&serde_json::Value> {
        self.variables.get(key)
    }
    
    pub fn add_note(&mut self, note: &str) {
        self.notes.push(note.to_string());
    }
    
    pub fn set_focus(&mut self, focus: &str) {
        self.focus = Some(focus.to_string());
    }
    
    pub fn to_context_string(&self) -> String {
        let mut parts = Vec::new();
        
        if let Some(task) = &self.current_task {
            parts.push(format!("Current Task: {}", task));
        }
        
        if let Some(focus) = &self.focus {
            parts.push(format!("Focus: {}", focus));
        }
        
        if !self.variables.is_empty() {
            let vars: Vec<String> = self.variables
                .iter()
                .map(|(k, v)| format!("  {}: {}", k, v))
                .collect();
            parts.push(format!("Variables:\n{}", vars.join("\n")));
        }
        
        if !self.notes.is_empty() {
            parts.push(format!("Notes:\n  - {}", self.notes.join("\n  - ")));
        }
        
        parts.join("\n")
    }
}

/// Long-term memory - persistent knowledge base
/// Enhanced with optional semantic search support
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LongTermMemory {
    /// All stored memories
    entries: HashMap<String, MemoryEntry>,
    /// Index by category
    category_index: HashMap<MemoryCategory, Vec<String>>,
    /// Whether semantic search is enabled (vector store is external)
    #[serde(skip)]
    semantic_enabled: bool,
}

impl LongTermMemory {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Enable semantic search capability
    pub fn enable_semantic(&mut self) {
        self.semantic_enabled = true;
    }
    
    /// Check if semantic search is enabled
    pub fn is_semantic_enabled(&self) -> bool {
        self.semantic_enabled
    }
    
    pub fn store(&mut self, entry: MemoryEntry) {
        let id = entry.id.clone();
        let category = entry.category.clone();
        
        self.entries.insert(id.clone(), entry);
        
        self.category_index
            .entry(category)
            .or_default()
            .push(id);
    }
    
    pub fn recall(&mut self, id: &str) -> Option<&MemoryEntry> {
        if let Some(entry) = self.entries.get_mut(id) {
            entry.last_accessed = Utc::now();
            entry.access_count += 1;
        }
        self.entries.get(id)
    }
    
    pub fn search(&self, query: &str, limit: usize) -> Vec<&MemoryEntry> {
        let query_lower = query.to_lowercase();
        
        let mut matches: Vec<_> = self.entries
            .values()
            .filter(|e| e.content.to_lowercase().contains(&query_lower))
            .collect();
        
        // Sort by relevance
        matches.sort_by(|a, b| {
            b.relevance_score().partial_cmp(&a.relevance_score()).unwrap()
        });
        
        matches.into_iter().take(limit).collect()
    }
    
    pub fn get_by_category(&self, category: &MemoryCategory, limit: usize) -> Vec<&MemoryEntry> {
        self.category_index
            .get(category)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.entries.get(id))
                    .take(limit)
                    .collect()
            })
            .unwrap_or_default()
    }
    
    pub fn get_most_relevant(&self, limit: usize) -> Vec<&MemoryEntry> {
        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_by(|a, b| {
            b.relevance_score().partial_cmp(&a.relevance_score()).unwrap()
        });
        entries.into_iter().take(limit).collect()
    }
    
    pub fn forget(&mut self, id: &str) {
        if let Some(entry) = self.entries.remove(id) {
            if let Some(ids) = self.category_index.get_mut(&entry.category) {
                ids.retain(|i| i != id);
            }
        }
    }
    
    pub fn prune(&mut self, min_relevance: f32) {
        let to_remove: Vec<String> = self.entries
            .iter()
            .filter(|(_, e)| e.relevance_score() < min_relevance)
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in to_remove {
            self.forget(&id);
        }
    }
    
    /// Get all entries (for syncing to vector store)
    pub fn all_entries(&self) -> Vec<&MemoryEntry> {
        self.entries.values().collect()
    }
    
    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Complete memory system for an agent
/// Supports both keyword and semantic search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMemory {
    pub agent_id: String,
    pub short_term: ShortTermMemory,
    pub working: WorkingMemory,
    pub long_term: LongTermMemory,
}

impl AgentMemory {
    pub fn new(agent_id: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            short_term: ShortTermMemory::new(50),
            working: WorkingMemory::new(),
            long_term: LongTermMemory::new(),
        }
    }
    
    /// Create with semantic search enabled
    pub fn with_semantic(agent_id: &str) -> Self {
        let mut memory = Self::new(agent_id);
        memory.long_term.enable_semantic();
        memory
    }
    
    /// Record a user message
    pub fn record_user_message(&mut self, message: &str) {
        self.short_term.add(&format!("User: {}", message));
    }
    
    /// Record an assistant response
    pub fn record_assistant_message(&mut self, message: &str) {
        self.short_term.add(&format!("Assistant: {}", message));
    }
    
    /// Store a learning for long-term
    pub fn learn(&mut self, content: &str, category: MemoryCategory, importance: f32) {
        let entry = MemoryEntry::new(content, category, importance);
        self.long_term.store(entry);
    }
    
    /// Generate context string for prompt injection
    pub fn to_context(&self) -> String {
        let mut context_parts = Vec::new();
        
        // Working memory
        let working = self.working.to_context_string();
        if !working.is_empty() {
            context_parts.push(format!("## Working Context\n{}", working));
        }
        
        // Recent conversation
        let recent = self.short_term.to_context_string();
        if !recent.is_empty() {
            context_parts.push(format!("## Recent Conversation\n{}", recent));
        }
        
        // Relevant long-term memories
        let relevant = self.long_term.get_most_relevant(5);
        if !relevant.is_empty() {
            let mem_str: Vec<String> = relevant
                .iter()
                .map(|m| format!("- [{}] {}", format!("{:?}", m.category), m.content))
                .collect();
            context_parts.push(format!("## Relevant Knowledge\n{}", mem_str.join("\n")));
        }
        
        context_parts.join("\n\n")
    }
    
    /// Clear short-term and working memory (new session)
    pub fn new_session(&mut self) {
        self.short_term.clear();
        self.working.clear_task();
    }
}

/// Memory manager for all agents
#[derive(Debug, Default)]
pub struct MemoryManager {
    memories: HashMap<String, AgentMemory>,
}

impl MemoryManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn get_or_create(&mut self, agent_id: &str) -> &mut AgentMemory {
        self.memories
            .entry(agent_id.to_string())
            .or_insert_with(|| AgentMemory::new(agent_id))
    }
    
    pub fn get(&self, agent_id: &str) -> Option<&AgentMemory> {
        self.memories.get(agent_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_short_term_memory() {
        let mut stm = ShortTermMemory::new(3);
        stm.add("Message 1");
        stm.add("Message 2");
        stm.add("Message 3");
        stm.add("Message 4");
        
        assert_eq!(stm.messages.len(), 3);
        assert!(stm.to_context_string().contains("Message 4"));
        assert!(!stm.to_context_string().contains("Message 1"));
    }
    
    #[test]
    fn test_working_memory() {
        let mut wm = WorkingMemory::new();
        wm.set_task("Create motor FSM");
        wm.set_variable("mcu", serde_json::json!("STM32F4"));
        wm.add_note("User prefers switch-case style");
        
        let context = wm.to_context_string();
        assert!(context.contains("Create motor FSM"));
        assert!(context.contains("STM32F4"));
    }
    
    #[test]
    fn test_long_term_memory() {
        let mut ltm = LongTermMemory::new();
        
        ltm.store(MemoryEntry::new(
            "User prefers C over C++",
            MemoryCategory::UserPreference,
            0.9
        ));
        
        ltm.store(MemoryEntry::new(
            "Project uses STM32F4",
            MemoryCategory::ProjectContext,
            0.8
        ));
        
        let results = ltm.search("STM32", 10);
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("STM32F4"));
    }
}
