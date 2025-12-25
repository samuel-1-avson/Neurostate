// Gemini API Client
// Integration with Google's Gemini AI

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

const GEMINI_API_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent";

#[derive(Clone)]
pub struct GeminiClient {
    client: Client,
    api_key: Option<String>,
}

impl GeminiClient {
    pub fn new() -> Self {
        let api_key = env::var("GEMINI_API_KEY").ok()
            .or_else(|| env::var("API_KEY").ok());
        
        Self {
            client: Client::new(),
            api_key,
        }
    }
    
    pub fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }
    
    pub async fn generate(&self, prompt: &str) -> Result<String, String> {
        let api_key = self.api_key.as_ref()
            .ok_or("GEMINI_API_KEY not configured")?;
        
        let url = format!("{}?key={}", GEMINI_API_URL, api_key);
        
        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part { text: prompt.to_string() }],
            }],
            generation_config: Some(GenerationConfig {
                temperature: 0.7,
                max_output_tokens: 4096,
            }),
        };
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("API error: {}", error_text));
        }
        
        let gemini_response: GeminiResponse = response.json().await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        gemini_response.candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .ok_or_else(|| "No response text".to_string())
    }
}

impl Default for GeminiClient {
    fn default() -> Self {
        Self::new()
    }
}

// --- API Types ---

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Serialize, Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize)]
struct Part {
    text: String,
}

#[derive(Serialize)]
struct GenerationConfig {
    temperature: f32,
    max_output_tokens: u32,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Content,
}
