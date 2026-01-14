//! RAG Pipeline - Retrieval-Augmented Generation
//!
//! Provides document indexing, retrieval, and context-augmented generation.
//! Features:
//! - Multiple chunking strategies (fixed-size, sentence, paragraph)
//! - Hybrid retrieval (keyword + semantic)
//! - Context assembly with token budgeting
//! - Source attribution

use crate::ai::providers::{AIModel, ChatMessage, ModelResponse, Role};
use crate::ai::vector_store::{VectorStore, VectorStoreError, SearchResult, EmbeddingProvider};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

// =============================================================================
// TYPES
// =============================================================================

/// A document to be indexed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique identifier
    pub id: String,
    /// Document title or name
    pub title: String,
    /// Full content
    pub content: String,
    /// Document type/source
    pub doc_type: DocumentType,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// When the document was added
    pub indexed_at: Option<DateTime<Utc>>,
}

/// Types of documents that can be indexed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    /// Code file
    Code { language: String },
    /// Documentation/markdown
    Documentation,
    /// Conversation history
    Conversation,
    /// Error logs
    ErrorLog,
    /// Hardware datasheet
    Datasheet,
    /// General text
    Text,
}

/// A chunk of a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    /// Parent document ID
    pub document_id: String,
    /// Chunk index within document
    pub chunk_index: usize,
    /// The chunk content
    pub content: String,
    /// Start character offset in original document
    pub start_offset: usize,
    /// End character offset in original document
    pub end_offset: usize,
    /// Inherited metadata from document
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Chunking strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkingStrategy {
    /// Fixed size chunks with overlap
    FixedSize {
        chunk_size: usize,
        overlap: usize,
    },
    /// Split by sentences
    Sentence {
        max_sentences: usize,
        overlap_sentences: usize,
    },
    /// Split by paragraphs (double newlines)
    Paragraph {
        max_paragraphs: usize,
    },
    /// Semantic chunking (split at natural boundaries)
    Semantic {
        target_size: usize,
    },
    /// Code-aware chunking (respect function/class boundaries)
    Code {
        max_lines: usize,
    },
}

impl Default for ChunkingStrategy {
    fn default() -> Self {
        Self::FixedSize {
            chunk_size: 512,
            overlap: 64,
        }
    }
}

/// Retrieved chunk with relevance score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedChunk {
    /// The document chunk
    pub chunk: DocumentChunk,
    /// Relevance score (0.0 to 1.0)
    pub score: f32,
    /// Source document title
    pub source_title: String,
}

/// RAG generation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGResponse {
    /// Generated response
    pub content: String,
    /// Sources used
    pub sources: Vec<SourceAttribution>,
    /// Number of chunks retrieved
    pub chunks_used: usize,
    /// Model used for generation
    pub model: String,
}

/// Source attribution for transparency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceAttribution {
    /// Document title
    pub title: String,
    /// Document ID
    pub document_id: String,
    /// Relevance score
    pub relevance: f32,
    /// Brief excerpt
    pub excerpt: String,
}

// =============================================================================
// TEXT CHUNKER
// =============================================================================

/// Text chunker for splitting documents
pub struct TextChunker {
    strategy: ChunkingStrategy,
}

impl TextChunker {
    pub fn new(strategy: ChunkingStrategy) -> Self {
        Self { strategy }
    }

    /// Split a document into chunks
    pub fn chunk(&self, document: &Document) -> Vec<DocumentChunk> {
        match &self.strategy {
            ChunkingStrategy::FixedSize { chunk_size, overlap } => {
                self.chunk_fixed_size(&document.id, &document.content, *chunk_size, *overlap, &document.metadata)
            }
            ChunkingStrategy::Sentence { max_sentences, overlap_sentences } => {
                self.chunk_sentences(&document.id, &document.content, *max_sentences, *overlap_sentences, &document.metadata)
            }
            ChunkingStrategy::Paragraph { max_paragraphs } => {
                self.chunk_paragraphs(&document.id, &document.content, *max_paragraphs, &document.metadata)
            }
            ChunkingStrategy::Semantic { target_size } => {
                self.chunk_semantic(&document.id, &document.content, *target_size, &document.metadata)
            }
            ChunkingStrategy::Code { max_lines } => {
                self.chunk_code(&document.id, &document.content, *max_lines, &document.metadata)
            }
        }
    }

    fn chunk_fixed_size(
        &self,
        doc_id: &str,
        content: &str,
        chunk_size: usize,
        overlap: usize,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = content.chars().collect();
        let total_chars = chars.len();

        if total_chars == 0 {
            return chunks;
        }

        let step = if chunk_size > overlap { chunk_size - overlap } else { chunk_size };
        let mut start = 0;
        let mut chunk_index = 0;

        while start < total_chars {
            let end = (start + chunk_size).min(total_chars);
            let chunk_content: String = chars[start..end].iter().collect();

            chunks.push(DocumentChunk {
                document_id: doc_id.to_string(),
                chunk_index,
                content: chunk_content,
                start_offset: start,
                end_offset: end,
                metadata: metadata.clone(),
            });

            start += step;
            chunk_index += 1;
        }

        chunks
    }

    fn chunk_sentences(
        &self,
        doc_id: &str,
        content: &str,
        max_sentences: usize,
        overlap_sentences: usize,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> Vec<DocumentChunk> {
        // Simple sentence splitting (can be improved with NLP)
        let sentences: Vec<&str> = content
            .split(|c| c == '.' || c == '!' || c == '?')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let mut chunks = Vec::new();
        let step = if max_sentences > overlap_sentences { 
            max_sentences - overlap_sentences 
        } else { 
            max_sentences 
        };

        let mut i = 0;
        let mut chunk_index = 0;

        while i < sentences.len() {
            let end = (i + max_sentences).min(sentences.len());
            let chunk_sentences = &sentences[i..end];
            let chunk_content = chunk_sentences.join(". ") + ".";
            
            // Calculate approximate offsets
            let start_offset = content.find(chunk_sentences[0]).unwrap_or(0);
            let end_offset = start_offset + chunk_content.len();

            chunks.push(DocumentChunk {
                document_id: doc_id.to_string(),
                chunk_index,
                content: chunk_content,
                start_offset,
                end_offset,
                metadata: metadata.clone(),
            });

            i += step;
            chunk_index += 1;
        }

        chunks
    }

    fn chunk_paragraphs(
        &self,
        doc_id: &str,
        content: &str,
        max_paragraphs: usize,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> Vec<DocumentChunk> {
        let paragraphs: Vec<&str> = content
            .split("\n\n")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let mut chunks = Vec::new();
        let mut i = 0;
        let mut chunk_index = 0;

        while i < paragraphs.len() {
            let end = (i + max_paragraphs).min(paragraphs.len());
            let chunk_paragraphs = &paragraphs[i..end];
            let chunk_content = chunk_paragraphs.join("\n\n");
            
            let start_offset = content.find(chunk_paragraphs[0]).unwrap_or(0);
            let end_offset = start_offset + chunk_content.len();

            chunks.push(DocumentChunk {
                document_id: doc_id.to_string(),
                chunk_index,
                content: chunk_content,
                start_offset,
                end_offset,
                metadata: metadata.clone(),
            });

            i += max_paragraphs;
            chunk_index += 1;
        }

        chunks
    }

    fn chunk_semantic(
        &self,
        doc_id: &str,
        content: &str,
        target_size: usize,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> Vec<DocumentChunk> {
        // Semantic chunking: try to break at natural boundaries
        // (periods, paragraph breaks, etc.) near the target size
        let mut chunks = Vec::new();
        let mut current_start = 0;
        let mut chunk_index = 0;

        while current_start < content.len() {
            let ideal_end = (current_start + target_size).min(content.len());
            
            // Look for natural break point near ideal_end
            let search_start = if ideal_end > 50 { ideal_end - 50 } else { current_start };
            let search_area = &content[search_start..ideal_end.min(content.len())];
            
            let break_point = search_area
                .rfind("\n\n")
                .or_else(|| search_area.rfind(". "))
                .or_else(|| search_area.rfind('\n'))
                .map(|p| search_start + p + 1)
                .unwrap_or(ideal_end);

            let actual_end = break_point.max(current_start + 1).min(content.len());
            let chunk_content = content[current_start..actual_end].trim().to_string();

            if !chunk_content.is_empty() {
                chunks.push(DocumentChunk {
                    document_id: doc_id.to_string(),
                    chunk_index,
                    content: chunk_content,
                    start_offset: current_start,
                    end_offset: actual_end,
                    metadata: metadata.clone(),
                });
                chunk_index += 1;
            }

            current_start = actual_end;
        }

        chunks
    }

    fn chunk_code(
        &self,
        doc_id: &str,
        content: &str,
        max_lines: usize,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> Vec<DocumentChunk> {
        let lines: Vec<&str> = content.lines().collect();
        let mut chunks = Vec::new();
        let mut chunk_index = 0;
        let mut i = 0;

        while i < lines.len() {
            let end = (i + max_lines).min(lines.len());
            let chunk_lines = &lines[i..end];
            let chunk_content = chunk_lines.join("\n");
            
            // Calculate byte offsets
            let start_offset: usize = lines[..i].iter().map(|l| l.len() + 1).sum();
            let end_offset = start_offset + chunk_content.len();

            chunks.push(DocumentChunk {
                document_id: doc_id.to_string(),
                chunk_index,
                content: chunk_content,
                start_offset,
                end_offset,
                metadata: metadata.clone(),
            });

            i = end;
            chunk_index += 1;
        }

        chunks
    }
}

impl Default for TextChunker {
    fn default() -> Self {
        Self::new(ChunkingStrategy::default())
    }
}

// =============================================================================
// RAG PIPELINE
// =============================================================================

/// RAG Pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGConfig {
    /// Number of chunks to retrieve
    pub top_k: usize,
    /// Minimum relevance score threshold
    pub min_score: f32,
    /// Maximum context tokens (approximate)
    pub max_context_tokens: usize,
    /// Keyword weight for hybrid search (0.0 = pure semantic, 1.0 = pure keyword)
    pub keyword_weight: f32,
    /// Include source attribution
    pub include_sources: bool,
    /// Chunking strategy
    pub chunking_strategy: ChunkingStrategy,
}

impl Default for RAGConfig {
    fn default() -> Self {
        Self {
            top_k: 5,
            min_score: 0.3,
            max_context_tokens: 4000,
            keyword_weight: 0.2,
            include_sources: true,
            chunking_strategy: ChunkingStrategy::default(),
        }
    }
}

/// RAG Pipeline for retrieval-augmented generation
pub struct RAGPipeline {
    /// Vector store for embeddings
    vector_store: Arc<RwLock<VectorStore>>,
    /// Document metadata (for title lookup)
    documents: Arc<RwLock<HashMap<String, Document>>>,
    /// Text chunker
    chunker: TextChunker,
    /// Configuration
    config: RAGConfig,
}

impl RAGPipeline {
    /// Create a new RAG pipeline with default Ollama embeddings
    pub fn new() -> Self {
        Self {
            vector_store: Arc::new(RwLock::new(VectorStore::new())),
            documents: Arc::new(RwLock::new(HashMap::new())),
            chunker: TextChunker::default(),
            config: RAGConfig::default(),
        }
    }

    /// Create with specific embedding provider
    pub fn with_provider(provider: EmbeddingProvider) -> Self {
        Self {
            vector_store: Arc::new(RwLock::new(VectorStore::with_provider(provider))),
            documents: Arc::new(RwLock::new(HashMap::new())),
            chunker: TextChunker::default(),
            config: RAGConfig::default(),
        }
    }

    /// Set configuration
    pub fn with_config(mut self, config: RAGConfig) -> Self {
        self.chunker = TextChunker::new(config.chunking_strategy.clone());
        self.config = config;
        self
    }

    /// Set persistence path for vector store
    pub async fn with_persistence(self, path: PathBuf) -> Self {
        let mut store = self.vector_store.write().await;
        *store = std::mem::take(&mut *store).with_persistence(path);
        let _ = store.load_if_exists();
        drop(store);
        self
    }

    // =========================================================================
    // INDEXING
    // =========================================================================

    /// Index a single document
    pub async fn index_document(&self, document: Document) -> Result<usize, RAGPipelineError> {
        let chunks = self.chunker.chunk(&document);
        let chunk_count = chunks.len();

        // Store document metadata
        {
            let mut docs = self.documents.write().await;
            docs.insert(document.id.clone(), Document {
                indexed_at: Some(Utc::now()),
                ..document.clone()
            });
        }

        // Index each chunk
        let mut store = self.vector_store.write().await;
        for chunk in chunks {
            let chunk_id = format!("{}::chunk::{}", document.id, chunk.chunk_index);
            let mut metadata = chunk.metadata.clone();
            metadata.insert("document_id".to_string(), serde_json::json!(document.id));
            metadata.insert("chunk_index".to_string(), serde_json::json!(chunk.chunk_index));
            metadata.insert("doc_title".to_string(), serde_json::json!(document.title));
            
            store.store(&chunk_id, &chunk.content, metadata).await
                .map_err(|e| RAGPipelineError::Indexing(e.to_string()))?;
        }

        Ok(chunk_count)
    }

    /// Index multiple documents
    pub async fn index_documents(&self, documents: Vec<Document>) -> Result<usize, RAGPipelineError> {
        let mut total_chunks = 0;
        for doc in documents {
            total_chunks += self.index_document(doc).await?;
        }
        Ok(total_chunks)
    }

    /// Remove a document from the index
    pub async fn remove_document(&self, document_id: &str) -> Result<usize, RAGPipelineError> {
        let mut store = self.vector_store.write().await;
        let mut docs = self.documents.write().await;
        
        // Find all chunks for this document
        // Note: prefix is kept for potential future use with secondary indexing
        let _prefix = format!("{}::chunk::", document_id);
        let mut removed_count = 0;
        let mut chunk_idx = 0;
        loop {
            let chunk_id = format!("{}::chunk::{}", document_id, chunk_idx);
            if store.get(&chunk_id).is_some() {
                store.remove(&chunk_id);
                removed_count += 1;
                chunk_idx += 1;
            } else {
                break;
            }
        }

        docs.remove(document_id);
        Ok(removed_count)
    }

    // =========================================================================
    // RETRIEVAL
    // =========================================================================

    /// Retrieve relevant chunks for a query
    pub async fn retrieve(&self, query: &str) -> Result<Vec<RetrievedChunk>, RAGPipelineError> {
        let store = self.vector_store.read().await;
        
        // Hybrid search if keyword weight > 0
        let results = if self.config.keyword_weight > 0.0 {
            store.hybrid_search(query, self.config.top_k * 2, self.config.keyword_weight).await
        } else {
            store.search(query, self.config.top_k * 2).await
        }.map_err(|e| RAGPipelineError::Retrieval(e.to_string()))?;

        // Filter by minimum score and limit to top_k
        let docs = self.documents.read().await;
        let retrieved: Vec<RetrievedChunk> = results
            .into_iter()
            .filter(|r| r.score >= self.config.min_score)
            .take(self.config.top_k)
            .map(|r| {
                let doc_id = r.entry.metadata.get("document_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let chunk_index = r.entry.metadata.get("chunk_index")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize;
                let source_title = r.entry.metadata.get("doc_title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();

                RetrievedChunk {
                    chunk: DocumentChunk {
                        document_id: doc_id.to_string(),
                        chunk_index,
                        content: r.entry.text.clone(),
                        start_offset: 0,
                        end_offset: r.entry.text.len(),
                        metadata: r.entry.metadata.clone(),
                    },
                    score: r.score,
                    source_title,
                }
            })
            .collect();

        Ok(retrieved)
    }

    /// Build context string from retrieved chunks
    fn build_context(&self, chunks: &[RetrievedChunk]) -> String {
        let mut context = String::new();
        let mut current_tokens = 0;
        let approx_tokens_per_char = 4; // rough approximation

        context.push_str("## Relevant Context\n\n");

        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_text = format!(
                "### Source {}: {} (Relevance: {:.0}%)\n{}\n\n",
                i + 1,
                chunk.source_title,
                chunk.score * 100.0,
                chunk.chunk.content
            );

            let chunk_tokens = chunk_text.len() / approx_tokens_per_char;
            if current_tokens + chunk_tokens > self.config.max_context_tokens {
                break;
            }

            context.push_str(&chunk_text);
            current_tokens += chunk_tokens;
        }

        context
    }

    // =========================================================================
    // GENERATION
    // =========================================================================

    /// Full RAG: retrieve context and generate response
    pub async fn generate<M: AIModel + ?Sized>(
        &self,
        query: &str,
        model: &M,
        system_prompt: Option<&str>,
    ) -> Result<RAGResponse, RAGPipelineError> {
        // Retrieve relevant chunks
        let chunks = self.retrieve(query).await?;
        
        if chunks.is_empty() {
            // No context found, just generate without RAG
            let messages = vec![
                ChatMessage {
                    role: Role::User,
                    content: query.to_string(),
                },
            ];

            let response = model.chat(&messages).await
                .map_err(|e| RAGPipelineError::Generation(e.to_string()))?;

            return Ok(RAGResponse {
                content: response.content,
                sources: vec![],
                chunks_used: 0,
                model: response.model,
            });
        }

        // Build context
        let context = self.build_context(&chunks);

        // Build messages with context
        let mut messages = Vec::new();

        if let Some(sys) = system_prompt {
            messages.push(ChatMessage {
                role: Role::System,
                content: sys.to_string(),
            });
        }

        messages.push(ChatMessage {
            role: Role::System,
            content: format!(
                "Use the following context to answer the user's question. If the context doesn't contain relevant information, say so.\n\n{}",
                context
            ),
        });

        messages.push(ChatMessage {
            role: Role::User,
            content: query.to_string(),
        });

        // Generate response
        let response = model.chat(&messages).await
            .map_err(|e| RAGPipelineError::Generation(e.to_string()))?;

        // Build source attributions
        let sources: Vec<SourceAttribution> = if self.config.include_sources {
            chunks.iter().map(|c| {
                let excerpt = if c.chunk.content.len() > 100 {
                    format!("{}...", &c.chunk.content[..100])
                } else {
                    c.chunk.content.clone()
                };

                SourceAttribution {
                    title: c.source_title.clone(),
                    document_id: c.chunk.document_id.clone(),
                    relevance: c.score,
                    excerpt,
                }
            }).collect()
        } else {
            vec![]
        };

        Ok(RAGResponse {
            content: response.content,
            sources,
            chunks_used: chunks.len(),
            model: response.model,
        })
    }

    /// Query with retrieve-only (no generation)
    pub async fn query(&self, query: &str) -> Result<Vec<RetrievedChunk>, RAGPipelineError> {
        self.retrieve(query).await
    }

    // =========================================================================
    // UTILITIES
    // =========================================================================

    /// Get pipeline statistics
    pub async fn stats(&self) -> RAGPipelineStats {
        let store = self.vector_store.read().await;
        let docs = self.documents.read().await;
        
        RAGPipelineStats {
            total_documents: docs.len(),
            total_chunks: store.len(),
            vector_dimension: store.stats().dimension,
            config: self.config.clone(),
        }
    }

    /// Check if embedding provider is available
    pub async fn check_provider(&self) -> Result<bool, RAGPipelineError> {
        let store = self.vector_store.read().await;
        store.check_provider().await
            .map_err(|e| RAGPipelineError::Retrieval(e.to_string()))
    }

    /// Clear all indexed data
    pub async fn clear(&self) {
        let mut store = self.vector_store.write().await;
        let mut docs = self.documents.write().await;
        store.clear();
        docs.clear();
    }
}

impl Default for RAGPipeline {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// HELPER TYPES
// =============================================================================

/// Pipeline statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGPipelineStats {
    pub total_documents: usize,
    pub total_chunks: usize,
    pub vector_dimension: Option<usize>,
    pub config: RAGConfig,
}

/// RAG Pipeline errors
#[derive(Debug, thiserror::Error)]
pub enum RAGPipelineError {
    #[error("Indexing error: {0}")]
    Indexing(String),

    #[error("Retrieval error: {0}")]
    Retrieval(String),

    #[error("Generation error: {0}")]
    Generation(String),

    #[error("Configuration error: {0}")]
    Configuration(String),
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_size_chunking() {
        let chunker = TextChunker::new(ChunkingStrategy::FixedSize {
            chunk_size: 10,
            overlap: 2,
        });

        let doc = Document {
            id: "test".to_string(),
            title: "Test Doc".to_string(),
            content: "Hello world, this is a test document".to_string(),
            doc_type: DocumentType::Text,
            metadata: HashMap::new(),
            indexed_at: None,
        };

        let chunks = chunker.chunk(&doc);
        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].document_id, "test");
    }

    #[test]
    fn test_sentence_chunking() {
        let chunker = TextChunker::new(ChunkingStrategy::Sentence {
            max_sentences: 2,
            overlap_sentences: 0,
        });

        let doc = Document {
            id: "test".to_string(),
            title: "Test Doc".to_string(),
            content: "First sentence. Second sentence. Third sentence. Fourth sentence.".to_string(),
            doc_type: DocumentType::Text,
            metadata: HashMap::new(),
            indexed_at: None,
        };

        let chunks = chunker.chunk(&doc);
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_paragraph_chunking() {
        let chunker = TextChunker::new(ChunkingStrategy::Paragraph {
            max_paragraphs: 1,
        });

        let doc = Document {
            id: "test".to_string(),
            title: "Test Doc".to_string(),
            content: "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.".to_string(),
            doc_type: DocumentType::Text,
            metadata: HashMap::new(),
            indexed_at: None,
        };

        let chunks = chunker.chunk(&doc);
        assert_eq!(chunks.len(), 3);
    }

    #[test]
    fn test_code_chunking() {
        let chunker = TextChunker::new(ChunkingStrategy::Code {
            max_lines: 5,
        });

        let doc = Document {
            id: "test".to_string(),
            title: "Test Code".to_string(),
            content: "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7".to_string(),
            doc_type: DocumentType::Code { language: "rust".to_string() },
            metadata: HashMap::new(),
            indexed_at: None,
        };

        let chunks = chunker.chunk(&doc);
        assert_eq!(chunks.len(), 2);
    }
}
