// AI Model Providers
// Abstraction layer for multiple LLM backends (Gemini, OpenAI, Local/Ollama)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for AI model providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: ModelProvider,
    pub model_name: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout_secs: u64,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: ModelProvider::Gemini,
            model_name: "gemini-1.5-flash".to_string(),
            api_key: None,
            base_url: None,
            temperature: 0.7,
            max_tokens: 4096,
            timeout_secs: 60,
        }
    }
}

/// Supported model providers
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelProvider {
    Gemini,
    OpenAI,
    Ollama,
    Custom,
}

/// Response from an AI model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Trait for AI model providers
#[async_trait]
pub trait AIModel: Send + Sync {
    fn name(&self) -> &str;
    fn is_configured(&self) -> bool;
    
    async fn generate(&self, prompt: &str) -> Result<ModelResponse, ModelError>;
    
    async fn generate_with_system(
        &self,
        system: &str,
        prompt: &str,
    ) -> Result<ModelResponse, ModelError>;
    
    async fn chat(
        &self,
        messages: &[ChatMessage],
    ) -> Result<ModelResponse, ModelError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// Model errors
#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("Model not configured: {0}")]
    NotConfigured(String),
    
    #[error("API error: {0}")]
    ApiError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Rate limited")]
    RateLimited,
    
    #[error("Timeout")]
    Timeout,
}

// ==================== OpenAI Implementation ====================

pub struct OpenAIModel {
    config: ModelConfig,
    client: reqwest::Client,
}

impl OpenAIModel {
    pub fn new(config: ModelConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .unwrap_or_default();
        
        Self { config, client }
    }
    
    pub fn with_api_key(api_key: String) -> Self {
        Self::new(ModelConfig {
            provider: ModelProvider::OpenAI,
            model_name: "gpt-4o-mini".to_string(),
            api_key: Some(api_key),
            base_url: Some("https://api.openai.com/v1".to_string()),
            ..Default::default()
        })
    }
}

#[async_trait]
impl AIModel for OpenAIModel {
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
        self.chat(&[
            ChatMessage { role: Role::System, content: system.to_string() },
            ChatMessage { role: Role::User, content: prompt.to_string() },
        ]).await
    }
    
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ModelResponse, ModelError> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| ModelError::NotConfigured("OpenAI API key not set".to_string()))?;
        
        let base_url = self.config.base_url.as_deref()
            .unwrap_or("https://api.openai.com/v1");
        
        let request_body = serde_json::json!({
            "model": self.config.model_name,
            "messages": messages.iter().map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        Role::System => "system",
                        Role::User => "user",
                        Role::Assistant => "assistant",
                    },
                    "content": m.content
                })
            }).collect::<Vec<_>>(),
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
        });
        
        let response = self.client
            .post(format!("{}/chat/completions", base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
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
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| ModelError::ParseError(e.to_string()))?;
        
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        let usage = json.get("usage").map(|u| TokenUsage {
            prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
        });
        
        Ok(ModelResponse {
            content,
            model: self.config.model_name.clone(),
            usage,
            finish_reason: json["choices"][0]["finish_reason"]
                .as_str()
                .map(|s| s.to_string()),
        })
    }
}

// ==================== Ollama (Local) Implementation ====================

pub struct OllamaModel {
    config: ModelConfig,
    client: reqwest::Client,
}

impl OllamaModel {
    pub fn new(config: ModelConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .unwrap_or_default();
        
        Self { config, client }
    }
    
    pub fn default_local() -> Self {
        Self::new(ModelConfig {
            provider: ModelProvider::Ollama,
            model_name: "llama3.2".to_string(),
            api_key: None,
            base_url: Some("http://localhost:11434".to_string()),
            temperature: 0.7,
            max_tokens: 4096,
            timeout_secs: 120, // Local models can be slower
        })
    }
    
    /// Check if Ollama is running and the model is available
    pub async fn check_available(&self) -> bool {
        let base_url = self.config.base_url.as_deref()
            .unwrap_or("http://localhost:11434");
        
        self.client
            .get(format!("{}/api/tags", base_url))
            .send()
            .await
            .is_ok()
    }
}

#[async_trait]
impl AIModel for OllamaModel {
    fn name(&self) -> &str {
        &self.config.model_name
    }
    
    fn is_configured(&self) -> bool {
        // Ollama doesn't need an API key
        true
    }
    
    async fn generate(&self, prompt: &str) -> Result<ModelResponse, ModelError> {
        let base_url = self.config.base_url.as_deref()
            .unwrap_or("http://localhost:11434");
        
        let request_body = serde_json::json!({
            "model": self.config.model_name,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": self.config.temperature,
                "num_predict": self.config.max_tokens,
            }
        });
        
        let response = self.client
            .post(format!("{}/api/generate", base_url))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ModelError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ModelError::ApiError(error_text));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| ModelError::ParseError(e.to_string()))?;
        
        let content = json["response"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        Ok(ModelResponse {
            content,
            model: self.config.model_name.clone(),
            usage: None,
            finish_reason: Some("done".to_string()),
        })
    }
    
    async fn generate_with_system(
        &self,
        system: &str,
        prompt: &str,
    ) -> Result<ModelResponse, ModelError> {
        let base_url = self.config.base_url.as_deref()
            .unwrap_or("http://localhost:11434");
        
        let request_body = serde_json::json!({
            "model": self.config.model_name,
            "prompt": prompt,
            "system": system,
            "stream": false,
            "options": {
                "temperature": self.config.temperature,
                "num_predict": self.config.max_tokens,
            }
        });
        
        let response = self.client
            .post(format!("{}/api/generate", base_url))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ModelError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ModelError::ApiError(error_text));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| ModelError::ParseError(e.to_string()))?;
        
        let content = json["response"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        Ok(ModelResponse {
            content,
            model: self.config.model_name.clone(),
            usage: None,
            finish_reason: Some("done".to_string()),
        })
    }
    
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ModelResponse, ModelError> {
        let base_url = self.config.base_url.as_deref()
            .unwrap_or("http://localhost:11434");
        
        let request_body = serde_json::json!({
            "model": self.config.model_name,
            "messages": messages.iter().map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        Role::System => "system",
                        Role::User => "user",
                        Role::Assistant => "assistant",
                    },
                    "content": m.content
                })
            }).collect::<Vec<_>>(),
            "stream": false,
            "options": {
                "temperature": self.config.temperature,
                "num_predict": self.config.max_tokens,
            }
        });
        
        let response = self.client
            .post(format!("{}/api/chat", base_url))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ModelError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ModelError::ApiError(error_text));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| ModelError::ParseError(e.to_string()))?;
        
        let content = json["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        Ok(ModelResponse {
            content,
            model: self.config.model_name.clone(),
            usage: None,
            finish_reason: Some("done".to_string()),
        })
    }
}

// ==================== Model Manager ====================

/// Manages multiple AI model providers and handles fallback
pub struct ModelManager {
    primary: Box<dyn AIModel>,
    fallback: Option<Box<dyn AIModel>>,
    config: ModelManagerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelManagerConfig {
    pub auto_fallback: bool,
    pub retry_count: u32,
}

impl Default for ModelManagerConfig {
    fn default() -> Self {
        Self {
            auto_fallback: true,
            retry_count: 2,
        }
    }
}

impl ModelManager {
    pub fn new(primary: Box<dyn AIModel>) -> Self {
        Self {
            primary,
            fallback: None,
            config: ModelManagerConfig::default(),
        }
    }
    
    pub fn with_fallback(mut self, fallback: Box<dyn AIModel>) -> Self {
        self.fallback = Some(fallback);
        self
    }
    
    pub fn with_config(mut self, config: ModelManagerConfig) -> Self {
        self.config = config;
        self
    }
    
    pub async fn generate(&self, prompt: &str) -> Result<ModelResponse, ModelError> {
        // Try primary
        match self.primary.generate(prompt).await {
            Ok(response) => return Ok(response),
            Err(e) => {
                log::warn!("Primary model failed: {}", e);
                
                // Try fallback if enabled
                if self.config.auto_fallback {
                    if let Some(ref fallback) = self.fallback {
                        log::info!("Trying fallback model...");
                        return fallback.generate(prompt).await;
                    }
                }
                
                Err(e)
            }
        }
    }
    
    pub async fn generate_with_system(
        &self,
        system: &str,
        prompt: &str,
    ) -> Result<ModelResponse, ModelError> {
        match self.primary.generate_with_system(system, prompt).await {
            Ok(response) => Ok(response),
            Err(e) if self.config.auto_fallback => {
                if let Some(ref fallback) = self.fallback {
                    return fallback.generate_with_system(system, prompt).await;
                }
                Err(e)
            }
            Err(e) => Err(e),
        }
    }
    
    pub async fn chat(&self, messages: &[ChatMessage]) -> Result<ModelResponse, ModelError> {
        match self.primary.chat(messages).await {
            Ok(response) => Ok(response),
            Err(e) if self.config.auto_fallback => {
                if let Some(ref fallback) = self.fallback {
                    return fallback.chat(messages).await;
                }
                Err(e)
            }
            Err(e) => Err(e),
        }
    }
    
    pub fn primary_name(&self) -> &str {
        self.primary.name()
    }
    
    pub fn is_configured(&self) -> bool {
        self.primary.is_configured()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_config_default() {
        let config = ModelConfig::default();
        assert_eq!(config.provider, ModelProvider::Gemini);
        assert_eq!(config.temperature, 0.7);
    }
    
    #[tokio::test]
    async fn test_ollama_model_check() {
        let model = OllamaModel::default_local();
        // This will fail if Ollama isn't running, which is expected
        let _available = model.check_available().await;
    }
}
