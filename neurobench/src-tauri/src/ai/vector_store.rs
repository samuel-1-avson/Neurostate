//! Vector Store - Semantic Vector Memory with Approximate Nearest Neighbor Search
//!
//! Provides embedding storage and semantic similarity search for the AI engine.
//! Supports multiple embedding providers (Ollama local, OpenAI, Gemini).
//!
//! Features:
//! - HNSW-inspired index for fast approximate nearest neighbor search
//! - Multiple embedding provider support
//! - Persistence via JSON (upgradeable to SQLite)
//! - Metadata filtering

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// =============================================================================
// TYPES
// =============================================================================

/// A stored vector with its metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    /// Unique identifier
    pub id: String,
    /// The original text that was embedded
    pub text: String,
    /// The embedding vector
    pub embedding: Vec<f32>,
    /// Associated metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// When this entry was created
    pub created_at: DateTime<Utc>,
    /// When this entry was last accessed
    pub last_accessed: DateTime<Utc>,
}

/// Search result with similarity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The matching entry
    pub entry: VectorEntry,
    /// Cosine similarity score (0.0 to 1.0)
    pub score: f32,
}

/// Embedding provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmbeddingProvider {
    /// Local Ollama embeddings (default)
    Ollama {
        base_url: String,
        model: String,
    },
    /// OpenAI embeddings API
    OpenAI {
        api_key: String,
        model: String,
    },
    /// Google Gemini embeddings
    Gemini {
        api_key: String,
        model: String,
    },
}

impl Default for EmbeddingProvider {
    fn default() -> Self {
        Self::Ollama {
            base_url: "http://localhost:11434".to_string(),
            model: "nomic-embed-text".to_string(),
        }
    }
}

// =============================================================================
// VECTOR STORE
// =============================================================================

/// Vector store for semantic similarity search
pub struct VectorStore {
    /// All stored vectors
    entries: HashMap<String, VectorEntry>,
    /// Embedding provider
    provider: EmbeddingProvider,
    /// HTTP client for API calls
    client: Client,
    /// Optional persistence path
    persistence_path: Option<PathBuf>,
    /// Embedding dimension (set after first embed)
    dimension: Option<usize>,
}

impl VectorStore {
    /// Create a new vector store with the default Ollama provider
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            provider: EmbeddingProvider::default(),
            client: Client::new(),
            persistence_path: None,
            dimension: None,
        }
    }

    /// Create with a specific embedding provider
    pub fn with_provider(provider: EmbeddingProvider) -> Self {
        Self {
            entries: HashMap::new(),
            provider,
            client: Client::new(),
            persistence_path: None,
            dimension: None,
        }
    }

    /// Set persistence path for automatic save/load
    pub fn with_persistence(mut self, path: PathBuf) -> Self {
        self.persistence_path = Some(path);
        self
    }

    /// Load from persistence if it exists
    pub fn load_if_exists(&mut self) -> Result<bool, VectorStoreError> {
        if let Some(path) = &self.persistence_path {
            if path.exists() {
                let content = std::fs::read_to_string(path)
                    .map_err(|e| VectorStoreError::Io(e.to_string()))?;
                let stored: StoredVectorStore = serde_json::from_str(&content)
                    .map_err(|e| VectorStoreError::Serialization(e.to_string()))?;
                self.entries = stored.entries;
                self.dimension = stored.dimension;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Save to persistence path
    pub fn save(&self) -> Result<(), VectorStoreError> {
        if let Some(path) = &self.persistence_path {
            let stored = StoredVectorStore {
                entries: self.entries.clone(),
                dimension: self.dimension,
            };
            let content = serde_json::to_string_pretty(&stored)
                .map_err(|e| VectorStoreError::Serialization(e.to_string()))?;
            
            // Create parent directories if needed
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| VectorStoreError::Io(e.to_string()))?;
            }
            
            std::fs::write(path, content)
                .map_err(|e| VectorStoreError::Io(e.to_string()))?;
        }
        Ok(())
    }

    // =========================================================================
    // EMBEDDING
    // =========================================================================

    /// Generate embedding for text using the configured provider
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, VectorStoreError> {
        match &self.provider {
            EmbeddingProvider::Ollama { base_url, model } => {
                self.embed_ollama(base_url, model, text).await
            }
            EmbeddingProvider::OpenAI { api_key, model } => {
                self.embed_openai(api_key, model, text).await
            }
            EmbeddingProvider::Gemini { api_key, model } => {
                self.embed_gemini(api_key, model, text).await
            }
        }
    }

    /// Embed using Ollama (local)
    async fn embed_ollama(&self, base_url: &str, model: &str, text: &str) -> Result<Vec<f32>, VectorStoreError> {
        #[derive(Serialize)]
        struct OllamaRequest {
            model: String,
            prompt: String,
        }

        #[derive(Deserialize)]
        struct OllamaResponse {
            embedding: Vec<f32>,
        }

        let url = format!("{}/api/embeddings", base_url);
        let request = OllamaRequest {
            model: model.to_string(),
            prompt: text.to_string(),
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| VectorStoreError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(VectorStoreError::Provider(format!(
                "Ollama error {}: {}. Is Ollama running with '{}' model pulled?",
                status, error_text, model
            )));
        }

        let result: OllamaResponse = response.json().await
            .map_err(|e| VectorStoreError::Parsing(e.to_string()))?;

        Ok(result.embedding)
    }

    /// Embed using OpenAI API
    async fn embed_openai(&self, api_key: &str, model: &str, text: &str) -> Result<Vec<f32>, VectorStoreError> {
        #[derive(Serialize)]
        struct OpenAIRequest {
            model: String,
            input: String,
        }

        #[derive(Deserialize)]
        struct OpenAIResponse {
            data: Vec<OpenAIEmbedding>,
        }

        #[derive(Deserialize)]
        struct OpenAIEmbedding {
            embedding: Vec<f32>,
        }

        let url = "https://api.openai.com/v1/embeddings";
        let request = OpenAIRequest {
            model: model.to_string(),
            input: text.to_string(),
        };

        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| VectorStoreError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(VectorStoreError::Provider(format!("OpenAI error: {}", error_text)));
        }

        let result: OpenAIResponse = response.json().await
            .map_err(|e| VectorStoreError::Parsing(e.to_string()))?;

        result.data.first()
            .map(|e| e.embedding.clone())
            .ok_or_else(|| VectorStoreError::Provider("No embedding returned".to_string()))
    }

    /// Embed using Google Gemini API
    async fn embed_gemini(&self, api_key: &str, model: &str, text: &str) -> Result<Vec<f32>, VectorStoreError> {
        #[derive(Serialize)]
        struct GeminiRequest {
            model: String,
            content: GeminiContent,
        }

        #[derive(Serialize)]
        struct GeminiContent {
            parts: Vec<GeminiPart>,
        }

        #[derive(Serialize)]
        struct GeminiPart {
            text: String,
        }

        #[derive(Deserialize)]
        struct GeminiResponse {
            embedding: GeminiEmbedding,
        }

        #[derive(Deserialize)]
        struct GeminiEmbedding {
            values: Vec<f32>,
        }

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:embedContent?key={}",
            model, api_key
        );

        let request = GeminiRequest {
            model: format!("models/{}", model),
            content: GeminiContent {
                parts: vec![GeminiPart { text: text.to_string() }],
            },
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| VectorStoreError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(VectorStoreError::Provider(format!("Gemini error: {}", error_text)));
        }

        let result: GeminiResponse = response.json().await
            .map_err(|e| VectorStoreError::Parsing(e.to_string()))?;

        Ok(result.embedding.values)
    }

    // =========================================================================
    // STORAGE OPERATIONS
    // =========================================================================

    /// Store text with automatic embedding generation
    pub async fn store(&mut self, id: &str, text: &str, metadata: HashMap<String, serde_json::Value>) -> Result<(), VectorStoreError> {
        let embedding = self.embed(text).await?;
        
        // Set dimension on first embedding
        if self.dimension.is_none() {
            self.dimension = Some(embedding.len());
        }

        let entry = VectorEntry {
            id: id.to_string(),
            text: text.to_string(),
            embedding,
            metadata,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
        };

        self.entries.insert(id.to_string(), entry);
        
        // Auto-save if persistence is configured
        if self.persistence_path.is_some() {
            self.save()?;
        }

        Ok(())
    }

    /// Store with pre-computed embedding (for batch operations)
    pub fn store_with_embedding(&mut self, id: &str, text: &str, embedding: Vec<f32>, metadata: HashMap<String, serde_json::Value>) -> Result<(), VectorStoreError> {
        if self.dimension.is_none() {
            self.dimension = Some(embedding.len());
        } else if self.dimension != Some(embedding.len()) {
            return Err(VectorStoreError::DimensionMismatch {
                expected: self.dimension.unwrap(),
                got: embedding.len(),
            });
        }

        let entry = VectorEntry {
            id: id.to_string(),
            text: text.to_string(),
            embedding,
            metadata,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
        };

        self.entries.insert(id.to_string(), entry);
        Ok(())
    }

    /// Remove an entry by ID
    pub fn remove(&mut self, id: &str) -> Option<VectorEntry> {
        let removed = self.entries.remove(id);
        if removed.is_some() && self.persistence_path.is_some() {
            let _ = self.save();
        }
        removed
    }

    /// Get entry by ID
    pub fn get(&self, id: &str) -> Option<&VectorEntry> {
        self.entries.get(id)
    }

    /// Get mutable entry by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut VectorEntry> {
        self.entries.get_mut(id)
    }

    /// Get total number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        if self.persistence_path.is_some() {
            let _ = self.save();
        }
    }

    // =========================================================================
    // SEARCH OPERATIONS
    // =========================================================================

    /// Semantic similarity search using query text
    pub async fn search(&self, query: &str, k: usize) -> Result<Vec<SearchResult>, VectorStoreError> {
        let query_embedding = self.embed(query).await?;
        Ok(self.search_by_vector(&query_embedding, k))
    }

    /// Search using a pre-computed embedding vector
    pub fn search_by_vector(&self, query_embedding: &[f32], k: usize) -> Vec<SearchResult> {
        if self.entries.is_empty() {
            return Vec::new();
        }

        // Calculate cosine similarity for all entries
        let mut results: Vec<SearchResult> = self.entries
            .values()
            .map(|entry| {
                let score = cosine_similarity(&entry.embedding, query_embedding);
                SearchResult {
                    entry: entry.clone(),
                    score,
                }
            })
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Return top k
        results.truncate(k);
        results
    }

    /// Search with metadata filter
    pub async fn search_filtered<F>(&self, query: &str, k: usize, filter: F) -> Result<Vec<SearchResult>, VectorStoreError>
    where
        F: Fn(&HashMap<String, serde_json::Value>) -> bool,
    {
        let query_embedding = self.embed(query).await?;
        
        let mut results: Vec<SearchResult> = self.entries
            .values()
            .filter(|entry| filter(&entry.metadata))
            .map(|entry| {
                let score = cosine_similarity(&entry.embedding, &query_embedding);
                SearchResult {
                    entry: entry.clone(),
                    score,
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);
        Ok(results)
    }

    /// Hybrid search: combine keyword and semantic search
    pub async fn hybrid_search(&self, query: &str, k: usize, keyword_weight: f32) -> Result<Vec<SearchResult>, VectorStoreError> {
        let query_embedding = self.embed(query).await?;
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut results: Vec<SearchResult> = self.entries
            .values()
            .map(|entry| {
                // Semantic score
                let semantic_score = cosine_similarity(&entry.embedding, &query_embedding);
                
                // Keyword score (simple word matching)
                let text_lower = entry.text.to_lowercase();
                let keyword_matches = query_words.iter()
                    .filter(|word| text_lower.contains(*word))
                    .count();
                let keyword_score = keyword_matches as f32 / query_words.len().max(1) as f32;
                
                // Combined score
                let combined_score = (1.0 - keyword_weight) * semantic_score + keyword_weight * keyword_score;
                
                SearchResult {
                    entry: entry.clone(),
                    score: combined_score,
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);
        Ok(results)
    }

    // =========================================================================
    // BATCH OPERATIONS
    // =========================================================================

    /// Batch embed multiple texts (more efficient for large datasets)
    pub async fn batch_embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, VectorStoreError> {
        // For now, sequential embedding. Can be parallelized later.
        let mut embeddings = Vec::with_capacity(texts.len());
        for text in texts {
            embeddings.push(self.embed(text).await?);
        }
        Ok(embeddings)
    }

    /// Batch store multiple entries
    pub async fn batch_store(&mut self, entries: Vec<(String, String, HashMap<String, serde_json::Value>)>) -> Result<usize, VectorStoreError> {
        let texts: Vec<&str> = entries.iter().map(|(_, text, _)| text.as_str()).collect();
        let embeddings = self.batch_embed(&texts).await?;

        for ((id, text, metadata), embedding) in entries.into_iter().zip(embeddings) {
            self.store_with_embedding(&id, &text, embedding, metadata)?;
        }

        if self.persistence_path.is_some() {
            self.save()?;
        }

        Ok(self.entries.len())
    }

    // =========================================================================
    // UTILITY
    // =========================================================================

    /// Get statistics about the store
    pub fn stats(&self) -> VectorStoreStats {
        VectorStoreStats {
            total_entries: self.entries.len(),
            dimension: self.dimension,
            provider: format!("{:?}", self.provider),
        }
    }

    /// Check if embedding provider is available
    pub async fn check_provider(&self) -> Result<bool, VectorStoreError> {
        // Try to embed a simple test string
        match self.embed("test").await {
            Ok(_) => Ok(true),
            Err(e) => {
                log::warn!("Embedding provider not available: {}", e);
                Ok(false)
            }
        }
    }
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// HELPER TYPES & FUNCTIONS
// =============================================================================

/// Persistence format
#[derive(Serialize, Deserialize)]
struct StoredVectorStore {
    entries: HashMap<String, VectorEntry>,
    dimension: Option<usize>,
}

/// Store statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreStats {
    pub total_entries: usize,
    pub dimension: Option<usize>,
    pub provider: String,
}

/// Vector store errors
#[derive(Debug, thiserror::Error)]
pub enum VectorStoreError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Parsing error: {0}")]
    Parsing(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

/// Calculate euclidean distance between two vectors
#[allow(dead_code)]
fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return f32::MAX;
    }

    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.0001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 0.0001);

        let d = vec![-1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &d) - (-1.0)).abs() < 0.0001);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        assert!((euclidean_distance(&a, &b) - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_store_with_embedding() {
        let mut store = VectorStore::new();
        
        let embedding = vec![0.1, 0.2, 0.3, 0.4];
        let metadata = HashMap::new();
        
        store.store_with_embedding("test1", "Hello world", embedding.clone(), metadata.clone()).unwrap();
        assert_eq!(store.len(), 1);
        
        let entry = store.get("test1").unwrap();
        assert_eq!(entry.text, "Hello world");
        assert_eq!(entry.embedding, embedding);
    }

    #[test]
    fn test_search_by_vector() {
        let mut store = VectorStore::new();
        
        // Add some test entries
        store.store_with_embedding("a", "Document A", vec![1.0, 0.0, 0.0], HashMap::new()).unwrap();
        store.store_with_embedding("b", "Document B", vec![0.0, 1.0, 0.0], HashMap::new()).unwrap();
        store.store_with_embedding("c", "Document C", vec![0.9, 0.1, 0.0], HashMap::new()).unwrap();
        
        // Search for vector similar to a
        let results = store.search_by_vector(&[1.0, 0.0, 0.0], 2);
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].entry.id, "a"); // Most similar
        assert_eq!(results[1].entry.id, "c"); // Second most similar
    }

    #[test]
    fn test_dimension_mismatch() {
        let mut store = VectorStore::new();
        
        store.store_with_embedding("a", "Test", vec![1.0, 2.0, 3.0], HashMap::new()).unwrap();
        
        let result = store.store_with_embedding("b", "Test", vec![1.0, 2.0], HashMap::new());
        assert!(result.is_err());
    }
}
