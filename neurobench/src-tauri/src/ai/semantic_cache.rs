//! Semantic Cache - Intelligent Response Caching
//!
//! Provides semantic-aware caching for AI responses:
//! - Vector-based similarity lookup (not just exact match)
//! - TTL expiration with configurable policies
//! - Hit/miss statistics
//! - Background eviction

use crate::ai::vector_store::{VectorStore, EmbeddingProvider};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

// =============================================================================
// TYPES
// =============================================================================

/// A cached response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Cache key (normalized query)
    pub key: String,
    /// Original query/prompt
    pub query: String,
    /// Cached response
    pub response: String,
    /// Model used to generate the response
    pub model: String,
    /// When the entry was created
    pub created_at: DateTime<Utc>,
    /// When the entry was last accessed
    pub last_accessed: DateTime<Utc>,
    /// Number of times this entry was hit
    pub hit_count: u64,
    /// Time-to-live in seconds
    pub ttl_secs: i64,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl CacheEntry {
    pub fn new(query: &str, response: &str, model: &str, ttl_secs: i64) -> Self {
        let now = Utc::now();
        Self {
            key: normalize_key(query),
            query: query.to_string(),
            response: response.to_string(),
            model: model.to_string(),
            created_at: now,
            last_accessed: now,
            hit_count: 0,
            ttl_secs,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: &str, value: impl Into<serde_json::Value>) -> Self {
        self.metadata.insert(key.to_string(), value.into());
        self
    }

    /// Check if entry is expired
    pub fn is_expired(&self) -> bool {
        if self.ttl_secs <= 0 {
            return false; // Never expires
        }
        Utc::now() > self.created_at + Duration::seconds(self.ttl_secs)
    }

    /// Get remaining TTL in seconds
    pub fn remaining_ttl(&self) -> i64 {
        if self.ttl_secs <= 0 {
            return -1; // Never expires
        }
        let expires_at = self.created_at + Duration::seconds(self.ttl_secs);
        (expires_at - Utc::now()).num_seconds().max(0)
    }
}

/// Cache lookup result
#[derive(Debug, Clone)]
pub enum CacheResult {
    /// Exact match found
    ExactHit(CacheEntry),
    /// Semantic match found (similar query)
    SemanticHit {
        entry: CacheEntry,
        similarity: f32,
    },
    /// No match found
    Miss,
    /// Match found but expired
    Expired(CacheEntry),
}

impl CacheResult {
    pub fn is_hit(&self) -> bool {
        matches!(self, CacheResult::ExactHit(_) | CacheResult::SemanticHit { .. })
    }

    pub fn get_response(&self) -> Option<&str> {
        match self {
            CacheResult::ExactHit(entry) => Some(&entry.response),
            CacheResult::SemanticHit { entry, .. } => Some(&entry.response),
            _ => None,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStats {
    /// Total lookups
    pub total_lookups: u64,
    /// Exact hits
    pub exact_hits: u64,
    /// Semantic hits
    pub semantic_hits: u64,
    /// Misses
    pub misses: u64,
    /// Expired entries found
    pub expired: u64,
    /// Current entry count
    pub entry_count: usize,
    /// Total tokens saved (approximate)
    pub tokens_saved: u64,
    /// Hit rate
    pub hit_rate: f64,
}

// =============================================================================
// CACHE CONFIGURATION
// =============================================================================

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Default TTL in seconds (0 = never expire)
    pub default_ttl_secs: i64,
    /// Maximum entries to keep
    pub max_entries: usize,
    /// Minimum similarity score for semantic match (0.0 to 1.0)
    pub semantic_threshold: f32,
    /// Enable semantic (vector-based) matching
    pub enable_semantic: bool,
    /// Eviction policy
    pub eviction_policy: EvictionPolicy,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl_secs: 3600, // 1 hour
            max_entries: 10000,
            semantic_threshold: 0.92, // Very high similarity required
            enable_semantic: true,
            eviction_policy: EvictionPolicy::LRU,
        }
    }
}

/// Eviction policy when cache is full
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvictionPolicy {
    /// Least Recently Used
    LRU,
    /// Least Frequently Used
    LFU,
    /// First In First Out
    FIFO,
    /// Random eviction
    Random,
}

// =============================================================================
// SEMANTIC CACHE
// =============================================================================

/// Semantic cache for AI responses
pub struct SemanticCache {
    /// Cache entries by key
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// Vector store for semantic lookup
    vector_store: Arc<RwLock<VectorStore>>,
    /// Configuration
    config: CacheConfig,
    /// Statistics counters
    total_lookups: AtomicU64,
    exact_hits: AtomicU64,
    semantic_hits: AtomicU64,
    misses: AtomicU64,
    expired: AtomicU64,
    tokens_saved: AtomicU64,
}

impl SemanticCache {
    /// Create a new semantic cache with default Ollama embeddings
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            vector_store: Arc::new(RwLock::new(VectorStore::new())),
            config: CacheConfig::default(),
            total_lookups: AtomicU64::new(0),
            exact_hits: AtomicU64::new(0),
            semantic_hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            expired: AtomicU64::new(0),
            tokens_saved: AtomicU64::new(0),
        }
    }

    /// Create with specific embedding provider
    pub fn with_provider(provider: EmbeddingProvider) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            vector_store: Arc::new(RwLock::new(VectorStore::with_provider(provider))),
            config: CacheConfig::default(),
            total_lookups: AtomicU64::new(0),
            exact_hits: AtomicU64::new(0),
            semantic_hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            expired: AtomicU64::new(0),
            tokens_saved: AtomicU64::new(0),
        }
    }

    /// Create with config
    pub fn with_config(mut self, config: CacheConfig) -> Self {
        self.config = config;
        self
    }

    /// Look up a query in the cache
    pub async fn get(&self, query: &str) -> Result<CacheResult, CacheError> {
        self.total_lookups.fetch_add(1, Ordering::Relaxed);
        let key = normalize_key(query);

        // First try exact match
        let exact_match = {
            let entries = self.entries.read().await;
            entries.get(&key).cloned()
        };

        if let Some(entry) = exact_match {
            if entry.is_expired() {
                self.expired.fetch_add(1, Ordering::Relaxed);
                return Ok(CacheResult::Expired(entry));
            }
            
            // Update hit count and last accessed
            self.update_access(&key).await;
            
            self.exact_hits.fetch_add(1, Ordering::Relaxed);
            self.tokens_saved.fetch_add(estimate_tokens(&entry.response), Ordering::Relaxed);
            
            let entries = self.entries.read().await;
            return Ok(CacheResult::ExactHit(entries.get(&key).unwrap().clone()));
        }

        // Try semantic match if enabled
        if self.config.enable_semantic {
            let store = self.vector_store.read().await;
            
            let results = store.search(query, 1).await
                .map_err(|e| CacheError::VectorError(e.to_string()))?;

            if let Some(result) = results.first() {
                if result.score >= self.config.semantic_threshold {
                    // Found a semantically similar query
                    let matched_key = result.entry.id.clone();
                    let score = result.score;
                    
                    drop(store);
                    
                    // Get entry outside of borrow
                    let semantic_match = {
                        let entries = self.entries.read().await;
                        entries.get(&matched_key).cloned()
                    };
                    
                    if let Some(entry) = semantic_match {
                        if entry.is_expired() {
                            self.expired.fetch_add(1, Ordering::Relaxed);
                            return Ok(CacheResult::Expired(entry));
                        }

                        self.update_access(&matched_key).await;
                        
                        self.semantic_hits.fetch_add(1, Ordering::Relaxed);
                        self.tokens_saved.fetch_add(estimate_tokens(&entry.response), Ordering::Relaxed);
                        
                        let entries = self.entries.read().await;
                        return Ok(CacheResult::SemanticHit {
                            entry: entries.get(&matched_key).unwrap().clone(),
                            similarity: score,
                        });
                    }
                }
            }
        }

        self.misses.fetch_add(1, Ordering::Relaxed);
        Ok(CacheResult::Miss)
    }

    /// Store a response in the cache
    pub async fn put(&self, query: &str, response: &str, model: &str) -> Result<(), CacheError> {
        self.put_with_ttl(query, response, model, self.config.default_ttl_secs).await
    }

    /// Store with custom TTL
    pub async fn put_with_ttl(
        &self,
        query: &str,
        response: &str,
        model: &str,
        ttl_secs: i64,
    ) -> Result<(), CacheError> {
        let key = normalize_key(query);
        let entry = CacheEntry::new(query, response, model, ttl_secs);

        // Evict if necessary
        self.evict_if_needed().await;

        // Store in entries map
        {
            let mut entries = self.entries.write().await;
            entries.insert(key.clone(), entry);
        }

        // Store in vector store for semantic matching
        if self.config.enable_semantic {
            let mut store = self.vector_store.write().await;
            let metadata = HashMap::new();
            store.store(&key, query, metadata).await
                .map_err(|e| CacheError::VectorError(e.to_string()))?;
        }

        Ok(())
    }

    /// Invalidate an entry
    pub async fn invalidate(&self, query: &str) {
        let key = normalize_key(query);
        
        {
            let mut entries = self.entries.write().await;
            entries.remove(&key);
        }

        if self.config.enable_semantic {
            let mut store = self.vector_store.write().await;
            store.remove(&key);
        }
    }

    /// Clear all entries
    pub async fn clear(&self) {
        {
            let mut entries = self.entries.write().await;
            entries.clear();
        }

        if self.config.enable_semantic {
            let mut store = self.vector_store.write().await;
            store.clear();
        }

        // Reset stats
        self.total_lookups.store(0, Ordering::Relaxed);
        self.exact_hits.store(0, Ordering::Relaxed);
        self.semantic_hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.expired.store(0, Ordering::Relaxed);
        self.tokens_saved.store(0, Ordering::Relaxed);
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let entries = self.entries.read().await;
        let total = self.total_lookups.load(Ordering::Relaxed);
        let hits = self.exact_hits.load(Ordering::Relaxed) + self.semantic_hits.load(Ordering::Relaxed);

        CacheStats {
            total_lookups: total,
            exact_hits: self.exact_hits.load(Ordering::Relaxed),
            semantic_hits: self.semantic_hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            expired: self.expired.load(Ordering::Relaxed),
            entry_count: entries.len(),
            tokens_saved: self.tokens_saved.load(Ordering::Relaxed),
            hit_rate: if total > 0 { hits as f64 / total as f64 } else { 0.0 },
        }
    }

    /// Evict expired entries
    pub async fn evict_expired(&self) -> usize {
        let mut entries = self.entries.write().await;
        let expired_keys: Vec<String> = entries
            .iter()
            .filter(|(_, e)| e.is_expired())
            .map(|(k, _)| k.clone())
            .collect();

        let count = expired_keys.len();
        for key in &expired_keys {
            entries.remove(key);
        }

        if self.config.enable_semantic && count > 0 {
            let mut store = self.vector_store.write().await;
            for key in expired_keys {
                store.remove(&key);
            }
        }

        count
    }

    /// Update access time and hit count
    async fn update_access(&self, key: &str) {
        let mut entries = self.entries.write().await;
        if let Some(entry) = entries.get_mut(key) {
            entry.last_accessed = Utc::now();
            entry.hit_count += 1;
        }
    }

    /// Evict entries if cache is full
    async fn evict_if_needed(&self) {
        let entries = self.entries.read().await;
        if entries.len() < self.config.max_entries {
            return;
        }
        drop(entries);

        // Evict expired first
        let evicted = self.evict_expired().await;
        if evicted > 0 {
            return;
        }

        // Apply eviction policy
        let key_to_evict = {
            let entries = self.entries.read().await;
            match self.config.eviction_policy {
                EvictionPolicy::LRU => {
                    entries.iter()
                        .min_by_key(|(_, e)| e.last_accessed)
                        .map(|(k, _)| k.clone())
                }
                EvictionPolicy::LFU => {
                    entries.iter()
                        .min_by_key(|(_, e)| e.hit_count)
                        .map(|(k, _)| k.clone())
                }
                EvictionPolicy::FIFO => {
                    entries.iter()
                        .min_by_key(|(_, e)| e.created_at)
                        .map(|(k, _)| k.clone())
                }
                EvictionPolicy::Random => {
                    entries.keys().next().cloned()
                }
            }
        };

        if let Some(key) = key_to_evict {
            let mut entries = self.entries.write().await;
            entries.remove(&key);
            
            if self.config.enable_semantic {
                let mut store = self.vector_store.write().await;
                store.remove(&key);
            }
        }
    }
}

impl Default for SemanticCache {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// CACHE WRAPPER FOR AI MODEL
// =============================================================================

use crate::ai::providers::{AIModel, ChatMessage, ModelResponse, ModelError};

/// Cached AI model wrapper
pub struct CachedModel<M: AIModel> {
    /// Underlying model
    model: M,
    /// Cache
    cache: Arc<SemanticCache>,
    /// Whether to cache responses
    enabled: bool,
}

impl<M: AIModel> CachedModel<M> {
    pub fn new(model: M, cache: Arc<SemanticCache>) -> Self {
        Self {
            model,
            cache,
            enabled: true,
        }
    }

    pub fn disable_cache(&mut self) {
        self.enabled = false;
    }

    pub fn enable_cache(&mut self) {
        self.enabled = true;
    }

    /// Chat with caching
    pub async fn chat_cached(&self, messages: &[ChatMessage]) -> Result<ModelResponse, ModelError> {
        if !self.enabled {
            return self.model.chat(messages).await;
        }

        // Build cache key from messages
        let cache_key = messages_to_cache_key(messages);

        // Check cache
        if let Ok(result) = self.cache.get(&cache_key).await {
            if let Some(response) = result.get_response() {
                return Ok(ModelResponse {
                    content: response.to_string(),
                    model: "cached".to_string(),
                    usage: None,
                    finish_reason: Some("cached".to_string()),
                });
            }
        }

        // Call model
        let response = self.model.chat(messages).await?;

        // Store in cache
        let _ = self.cache.put(&cache_key, &response.content, &response.model).await;

        Ok(response)
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> CacheStats {
        self.cache.stats().await
    }
}

// =============================================================================
// HELPERS
// =============================================================================

/// Normalize a query to use as cache key
fn normalize_key(query: &str) -> String {
    query
        .to_lowercase()
        .trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convert messages to cache key
fn messages_to_cache_key(messages: &[ChatMessage]) -> String {
    messages
        .iter()
        .map(|m| format!("{:?}:{}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("|")
}

/// Estimate tokens in a response (rough approximation)
fn estimate_tokens(text: &str) -> u64 {
    (text.len() as f64 / 4.0) as u64
}

// =============================================================================
// ERRORS
// =============================================================================

/// Cache errors
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Vector store error: {0}")]
    VectorError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Cache full")]
    CacheFull,
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry() {
        let entry = CacheEntry::new("test query", "test response", "gpt-4", 3600);
        assert_eq!(entry.query, "test query");
        assert_eq!(entry.response, "test response");
        assert!(!entry.is_expired());
        assert!(entry.remaining_ttl() > 0);
    }

    #[test]
    fn test_cache_entry_no_expiry() {
        let entry = CacheEntry::new("test", "response", "model", 0);
        assert!(!entry.is_expired());
        assert_eq!(entry.remaining_ttl(), -1);
    }

    #[test]
    fn test_normalize_key() {
        assert_eq!(
            normalize_key("  Hello   World  "),
            "hello world"
        );
    }

    #[tokio::test]
    async fn test_cache_put_get() {
        let cache = SemanticCache::new().with_config(CacheConfig {
            enable_semantic: false, // Disable semantic for unit test
            ..Default::default()
        });

        cache.put("test query", "test response", "model").await.unwrap();

        match cache.get("test query").await.unwrap() {
            CacheResult::ExactHit(entry) => {
                assert_eq!(entry.response, "test response");
            }
            _ => panic!("Expected exact hit"),
        }
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = SemanticCache::new().with_config(CacheConfig {
            enable_semantic: false,
            ..Default::default()
        });

        match cache.get("nonexistent").await.unwrap() {
            CacheResult::Miss => {}
            _ => panic!("Expected miss"),
        }
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = SemanticCache::new().with_config(CacheConfig {
            enable_semantic: false,
            ..Default::default()
        });

        cache.put("q1", "r1", "m1").await.unwrap();
        let _ = cache.get("q1").await;
        let _ = cache.get("q2").await;

        let stats = cache.stats().await;
        assert_eq!(stats.total_lookups, 2);
        assert_eq!(stats.exact_hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.entry_count, 1);
    }

    #[tokio::test]
    async fn test_cache_invalidate() {
        let cache = SemanticCache::new().with_config(CacheConfig {
            enable_semantic: false,
            ..Default::default()
        });

        cache.put("test", "response", "model").await.unwrap();
        assert!(cache.get("test").await.unwrap().is_hit());

        cache.invalidate("test").await;
        assert!(!cache.get("test").await.unwrap().is_hit());
    }
}
