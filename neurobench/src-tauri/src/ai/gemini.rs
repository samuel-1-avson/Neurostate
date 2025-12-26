// Gemini API Client
// Integration with Google's Gemini AI

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use super::providers::{AIModel, ChatMessage, ModelConfig, ModelError, ModelResponse, Role, TokenUsage};

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";

/// Gemini AI Model - implements AIModel trait
pub struct GeminiModel {
    config: ModelConfig,
    client: Client,
}

impl GeminiModel {
    pub fn new(config: ModelConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .unwrap_or_default();
        
        Self { config, client }
    }
    
    /// Create with default config from environment
    pub fn from_env() -> Self {
        let api_key = env::var("GEMINI_API_KEY").ok()
            .or_else(|| env::var("API_KEY").ok());
        
        Self::new(ModelConfig {
            provider: super::providers::ModelProvider::Gemini,
            model_name: "gemini-1.5-flash".to_string(),
            api_key,
            base_url: None,
            temperature: 0.7,
            max_tokens: 4096,
            timeout_secs: 120,
        })
    }
    
    /// Create with specific model
    pub fn with_model(model_name: &str) -> Self {
        let api_key = env::var("GEMINI_API_KEY").ok()
            .or_else(|| env::var("API_KEY").ok());
        
        Self::new(ModelConfig {
            provider: super::providers::ModelProvider::Gemini,
            model_name: model_name.to_string(),
            api_key,
            base_url: None,
            temperature: 0.7,
            max_tokens: 8192,
            timeout_secs: 120,
        })
    }
    
    fn get_api_url(&self) -> String {
        format!("{}/{}:generateContent", GEMINI_API_BASE, self.config.model_name)
    }
}

#[async_trait]
impl AIModel for GeminiModel {
    fn name(&self) -> &str {
        &self.config.model_name
    }
    
    fn is_configured(&self) -> bool {
        self.config.api_key.is_some()
    }
    
    async fn generate(&self, prompt: &str) -> Result<ModelResponse, ModelError> {
        self.chat(&[ChatMessage {
            role: Role::User,
            content: prompt.to_string(),
        }]).await
    }
    
    async fn generate_with_system(
        &self,
        system: &str,
        prompt: &str,
    ) -> Result<ModelResponse, ModelError> {
        // Gemini uses system instruction differently
        self.chat(&[
            ChatMessage { role: Role::System, content: system.to_string() },
            ChatMessage { role: Role::User, content: prompt.to_string() },
        ]).await
    }
    
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ModelResponse, ModelError> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| ModelError::NotConfigured("GEMINI_API_KEY not set".to_string()))?;
        
        let url = format!("{}?key={}", self.get_api_url(), api_key);
        
        // Extract system instruction if present
        let system_instruction: Option<String> = messages.iter()
            .find(|m| matches!(m.role, Role::System))
            .map(|m| m.content.clone());
        
        // Convert messages to Gemini format (excluding system)
        let contents: Vec<GeminiContent> = messages.iter()
            .filter(|m| !matches!(m.role, Role::System))
            .map(|m| GeminiContent {
                role: match m.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "model".to_string(),
                    Role::System => "user".to_string(), // fallback
                },
                parts: vec![GeminiPart { text: m.content.clone() }],
            })
            .collect();
        
        let mut request = GeminiRequest {
            contents,
            generation_config: Some(GeminiGenerationConfig {
                temperature: self.config.temperature,
                max_output_tokens: self.config.max_tokens,
            }),
            system_instruction: None,
        };
        
        // Add system instruction if present
        if let Some(sys) = system_instruction {
            request.system_instruction = Some(GeminiSystemInstruction {
                parts: vec![GeminiPart { text: sys }],
            });
        }
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ModelError::NetworkError(e.to_string()))?;
        
        if response.status() == 429 {
            return Err(ModelError::RateLimited);
        }
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ModelError::ApiError(error_text));
        }
        
        let gemini_response: GeminiResponse = response.json().await
            .map_err(|e| ModelError::ParseError(e.to_string()))?;
        
        let content = gemini_response.candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .unwrap_or_default();
        
        let usage = gemini_response.usage_metadata.map(|u| TokenUsage {
            prompt_tokens: u.prompt_token_count,
            completion_tokens: u.candidates_token_count,
            total_tokens: u.total_token_count,
        });
        
        Ok(ModelResponse {
            content,
            model: self.config.model_name.clone(),
            usage,
            finish_reason: gemini_response.candidates
                .first()
                .and_then(|c| c.finish_reason.clone()),
        })
    }
}

// --- Legacy GeminiClient for backward compatibility ---

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
        let model = GeminiModel::from_env();
        model.generate(prompt).await
            .map(|r| r.content)
            .map_err(|e| e.to_string())
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
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiSystemInstruction>,
}

#[derive(Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize, Clone)]
struct GeminiPart {
    text: String,
}

#[derive(Serialize)]
struct GeminiGenerationConfig {
    temperature: f32,
    max_output_tokens: u32,
}

#[derive(Serialize)]
struct GeminiSystemInstruction {
    parts: Vec<GeminiPart>,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(default)]
    usage_metadata: Option<GeminiUsage>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiCandidateContent,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct GeminiCandidateContent {
    parts: Vec<GeminiPart>,
}

#[derive(Deserialize)]
struct GeminiUsage {
    #[serde(default)]
    prompt_token_count: u32,
    #[serde(default)]
    candidates_token_count: u32,
    #[serde(default)]
    total_token_count: u32,
}

