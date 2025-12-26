//! Streaming Response System
//!
//! Provides real-time token-by-token streaming for agent responses.
//! Uses Tauri events to push updates to the frontend.

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// A streaming chunk from an agent response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// Unique ID for this stream
    pub stream_id: String,
    /// Chunk sequence number
    pub sequence: u32,
    /// The text content of this chunk
    pub content: String,
    /// Whether this is the final chunk
    pub done: bool,
    /// Optional metadata
    pub metadata: Option<StreamMetadata>,
}

/// Metadata for streaming responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMetadata {
    pub agent_id: String,
    pub model: String,
    pub tokens_used: Option<u32>,
    pub finish_reason: Option<String>,
}

/// Stream event types for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// Stream started
    Start {
        stream_id: String,
        agent_id: String,
    },
    /// New content chunk
    Chunk {
        stream_id: String,
        content: String,
        sequence: u32,
    },
    /// Stream completed
    Complete {
        stream_id: String,
        total_tokens: Option<u32>,
    },
    /// Stream error
    Error {
        stream_id: String,
        error: String,
    },
}

/// Manages streaming responses
pub struct StreamManager {
    active_streams: std::collections::HashMap<String, StreamState>,
}

struct StreamState {
    sequence: u32,
    buffer: String,
    sender: Option<mpsc::Sender<StreamChunk>>,
}

impl StreamManager {
    pub fn new() -> Self {
        Self {
            active_streams: std::collections::HashMap::new(),
        }
    }
    
    /// Start a new stream
    pub fn start_stream(&mut self, agent_id: &str) -> (String, mpsc::Receiver<StreamChunk>) {
        let stream_id = format!("stream_{}_{}", agent_id, chrono::Utc::now().timestamp_millis());
        let (tx, rx) = mpsc::channel(100);
        
        self.active_streams.insert(stream_id.clone(), StreamState {
            sequence: 0,
            buffer: String::new(),
            sender: Some(tx),
        });
        
        (stream_id, rx)
    }
    
    /// Push a chunk to the stream
    pub async fn push_chunk(&mut self, stream_id: &str, content: &str) -> Result<(), String> {
        let state = self.active_streams
            .get_mut(stream_id)
            .ok_or_else(|| "Stream not found".to_string())?;
        
        state.sequence += 1;
        state.buffer.push_str(content);
        
        let chunk = StreamChunk {
            stream_id: stream_id.to_string(),
            sequence: state.sequence,
            content: content.to_string(),
            done: false,
            metadata: None,
        };
        
        if let Some(sender) = &state.sender {
            sender.send(chunk).await
                .map_err(|e| format!("Failed to send chunk: {}", e))?;
        }
        
        Ok(())
    }
    
    /// Complete a stream
    pub async fn complete_stream(&mut self, stream_id: &str, metadata: Option<StreamMetadata>) -> Result<String, String> {
        let state = self.active_streams
            .remove(stream_id)
            .ok_or_else(|| "Stream not found".to_string())?;
        
        // Send final chunk
        if let Some(sender) = state.sender {
            let chunk = StreamChunk {
                stream_id: stream_id.to_string(),
                sequence: state.sequence + 1,
                content: String::new(),
                done: true,
                metadata,
            };
            let _ = sender.send(chunk).await;
        }
        
        Ok(state.buffer)
    }
    
    /// Cancel a stream
    pub fn cancel_stream(&mut self, stream_id: &str) {
        self.active_streams.remove(stream_id);
    }
    
    /// Get the current buffer for a stream
    pub fn get_buffer(&self, stream_id: &str) -> Option<&str> {
        self.active_streams.get(stream_id).map(|s| s.buffer.as_str())
    }
}

impl Default for StreamManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Streaming response builder
#[derive(Debug, Clone)]
pub struct StreamingResponse {
    pub stream_id: String,
    chunks: Vec<String>,
}

impl StreamingResponse {
    pub fn new(stream_id: &str) -> Self {
        Self {
            stream_id: stream_id.to_string(),
            chunks: Vec::new(),
        }
    }
    
    pub fn push(&mut self, content: &str) {
        self.chunks.push(content.to_string());
    }
    
    pub fn full_content(&self) -> String {
        self.chunks.join("")
    }
    
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }
}

/// Trait for streamable responses
pub trait Streamable {
    /// Stream the response token by token
    fn stream<F>(&self, on_chunk: F) where F: FnMut(&str);
}

/// Simulated word-by-word streaming for non-streaming APIs
pub fn simulate_streaming(text: &str, delay_ms: u64) -> Vec<(String, u64)> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut chunks = Vec::new();
    let mut cumulative_delay = 0u64;
    
    for word in words {
        cumulative_delay += delay_ms;
        chunks.push((format!("{} ", word), cumulative_delay));
    }
    
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_stream_manager() {
        let mut manager = StreamManager::new();
        
        // Start stream
        let (stream_id, mut rx) = manager.start_stream("test");
        
        // Push chunks
        manager.push_chunk(&stream_id, "Hello ").await.unwrap();
        manager.push_chunk(&stream_id, "World!").await.unwrap();
        
        // Receive chunks
        let chunk1 = rx.recv().await.unwrap();
        assert_eq!(chunk1.content, "Hello ");
        
        let chunk2 = rx.recv().await.unwrap();
        assert_eq!(chunk2.content, "World!");
        
        // Complete
        let full = manager.complete_stream(&stream_id, None).await.unwrap();
        assert_eq!(full, "Hello World!");
    }
    
    #[test]
    fn test_simulate_streaming() {
        let text = "Hello world this is a test";
        let chunks = simulate_streaming(text, 50);
        
        assert_eq!(chunks.len(), 6);
        assert_eq!(chunks[0].0, "Hello ");
    }
}
