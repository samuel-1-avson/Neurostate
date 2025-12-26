//! AI Router - Intelligent model routing with fallback, health checks, and cost tracking
//!
//! Provides a unified interface to multiple AI providers with automatic failover.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use super::providers::{AIModel, ModelConfig, ModelError, ModelResponse, ChatMessage, ModelProvider};

// =============================================================================
// STREAMING
// =============================================================================

/// Token chunk for streaming responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// Text content of this chunk
    pub content: String,
    /// Is this the final chunk?
    pub done: bool,
    /// Token index
    pub index: usize,
}

/// Callback for streaming chunks
pub type StreamCallback = Box<dyn Fn(StreamChunk) + Send + Sync>;

// =============================================================================
// RETRY POLICY
// =============================================================================

/// Retry configuration with exponential backoff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Initial delay between retries (ms)
    pub initial_delay_ms: u64,
    /// Maximum delay between retries (ms)
    pub max_delay_ms: u64,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Errors to retry on
    pub retry_on: Vec<RetryableError>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RetryableError {
    RateLimited,
    Timeout,
    NetworkError,
    ServiceUnavailable,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            retry_on: vec![
                RetryableError::RateLimited,
                RetryableError::Timeout,
                RetryableError::NetworkError,
            ],
        }
    }
}

impl RetryPolicy {
    /// Calculate delay for nth retry
    pub fn delay_for_retry(&self, attempt: u32) -> Duration {
        let delay = self.initial_delay_ms as f64 * self.backoff_multiplier.powi(attempt as i32);
        let capped = delay.min(self.max_delay_ms as f64) as u64;
        Duration::from_millis(capped)
    }

    /// Check if error is retryable
    pub fn is_retryable(&self, error: &ModelError) -> bool {
        match error {
            ModelError::RateLimited => self.retry_on.contains(&RetryableError::RateLimited),
            ModelError::Timeout => self.retry_on.contains(&RetryableError::Timeout),
            ModelError::NetworkError(_) => self.retry_on.contains(&RetryableError::NetworkError),
            _ => false,
        }
    }
}

// =============================================================================
// COST TRACKING
// =============================================================================

/// Cost tracking for API usage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostTracker {
    /// Total input tokens
    pub input_tokens: u64,
    /// Total output tokens
    pub output_tokens: u64,
    /// Total estimated cost (USD cents)
    pub total_cost_cents: u64,
    /// Requests by provider
    pub requests_by_provider: HashMap<String, u64>,
}

impl CostTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record usage from a response
    pub fn record(&mut self, provider: &str, response: &ModelResponse) {
        if let Some(usage) = &response.usage {
            self.input_tokens += usage.prompt_tokens as u64;
            self.output_tokens += usage.completion_tokens as u64;
            
            // Estimate cost (rough approximations)
            let cost = match provider {
                "openai" | "gpt-4" => {
                    // GPT-4: ~$0.03/1K input, ~$0.06/1K output
                    (usage.prompt_tokens as f64 * 0.003) + (usage.completion_tokens as f64 * 0.006)
                }
                "gemini" => {
                    // Gemini: Free tier / much cheaper
                    (usage.prompt_tokens as f64 * 0.0001) + (usage.completion_tokens as f64 * 0.0002)
                }
                _ => 0.0, // Local models are free
            };
            self.total_cost_cents += (cost * 100.0) as u64;
        }
        
        *self.requests_by_provider.entry(provider.to_string()).or_default() += 1;
    }

    /// Get summary
    pub fn summary(&self) -> String {
        format!(
            "Tokens: {} in / {} out | Cost: ${:.2} | Requests: {:?}",
            self.input_tokens,
            self.output_tokens,
            self.total_cost_cents as f64 / 100.0,
            self.requests_by_provider
        )
    }
}

// =============================================================================
// PROVIDER HEALTH
// =============================================================================

/// Health status of a provider
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    /// Is provider available?
    pub available: bool,
    /// Last successful request
    pub last_success: Option<Instant>,
    /// Last error
    pub last_error: Option<String>,
    /// Consecutive failures
    pub consecutive_failures: u32,
    /// Average latency (ms)
    pub avg_latency_ms: f64,
}

impl Default for ProviderHealth {
    fn default() -> Self {
        Self {
            available: true,
            last_success: None,
            last_error: None,
            consecutive_failures: 0,
            avg_latency_ms: 0.0,
        }
    }
}

impl ProviderHealth {
    /// Record a successful request
    pub fn record_success(&mut self, latency_ms: u64) {
        self.available = true;
        self.last_success = Some(Instant::now());
        self.consecutive_failures = 0;
        // Rolling average
        self.avg_latency_ms = self.avg_latency_ms * 0.9 + latency_ms as f64 * 0.1;
    }

    /// Record a failure
    pub fn record_failure(&mut self, error: &str) {
        self.consecutive_failures += 1;
        self.last_error = Some(error.to_string());
        
        // Mark unavailable after 3 consecutive failures
        if self.consecutive_failures >= 3 {
            self.available = false;
        }
    }

    /// Check if provider should be used
    pub fn should_use(&self) -> bool {
        if !self.available {
            // Check if enough time passed to retry (5 minutes)
            if let Some(last) = self.last_success {
                return last.elapsed() > Duration::from_secs(300);
            }
        }
        self.available
    }
}

// =============================================================================
// MODEL ROUTER
// =============================================================================

/// Intelligent router for AI model requests
pub struct ModelRouter {
    /// Available providers in priority order
    providers: Vec<(ModelProvider, Arc<dyn AIModel + Send + Sync>)>,
    /// Health status per provider
    health: RwLock<HashMap<ModelProvider, ProviderHealth>>,
    /// Retry policy
    retry_policy: RetryPolicy,
    /// Cost tracker
    cost_tracker: RwLock<CostTracker>,
    /// Fallback enabled
    fallback_enabled: bool,
}

impl ModelRouter {
    /// Create new router with providers in fallback order
    pub fn new(providers: Vec<(ModelProvider, Arc<dyn AIModel + Send + Sync>)>) -> Self {
        let health: HashMap<_, _> = providers.iter()
            .map(|(p, _)| (*p, ProviderHealth::default()))
            .collect();
        
        Self {
            providers,
            health: RwLock::new(health),
            retry_policy: RetryPolicy::default(),
            cost_tracker: RwLock::new(CostTracker::new()),
            fallback_enabled: true,
        }
    }

    /// Set retry policy
    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    /// Disable fallback (use only first provider)
    pub fn without_fallback(mut self) -> Self {
        self.fallback_enabled = false;
        self
    }

    /// Get best available provider
    async fn get_provider(&self) -> Option<(ModelProvider, Arc<dyn AIModel + Send + Sync>)> {
        let health = self.health.read().await;
        
        for (provider, model) in &self.providers {
            if let Some(h) = health.get(provider) {
                if h.should_use() && model.is_configured() {
                    return Some((*provider, Arc::clone(model)));
                }
            }
        }
        None
    }

    /// Generate with automatic retry and fallback
    pub async fn generate(&self, prompt: &str) -> Result<ModelResponse, ModelError> {
        self.generate_with_system("", prompt).await
    }

    /// Generate with system prompt
    pub async fn generate_with_system(
        &self,
        system: &str,
        prompt: &str,
    ) -> Result<ModelResponse, ModelError> {
        let providers: Vec<_> = if self.fallback_enabled {
            self.providers.iter().collect()
        } else {
            self.providers.iter().take(1).collect()
        };

        let mut last_error = ModelError::ConfigError("No providers available".into());

        for (provider, model) in providers {
            if !model.is_configured() {
                continue;
            }

            // Check health
            {
                let health = self.health.read().await;
                if let Some(h) = health.get(provider) {
                    if !h.should_use() {
                        continue;
                    }
                }
            }

            // Try with retry
            match self.try_with_retry(*provider, model, system, prompt).await {
                Ok(response) => {
                    // Record success
                    let mut cost = self.cost_tracker.write().await;
                    cost.record(model.name(), &response);
                    return Ok(response);
                }
                Err(e) => {
                    last_error = e;
                    // Continue to next provider
                }
            }
        }

        Err(last_error)
    }

    async fn try_with_retry(
        &self,
        provider: ModelProvider,
        model: &Arc<dyn AIModel + Send + Sync>,
        system: &str,
        prompt: &str,
    ) -> Result<ModelResponse, ModelError> {
        let mut attempt = 0;
        
        loop {
            let start = Instant::now();
            let result = model.generate_with_system(system, prompt).await;
            let latency = start.elapsed().as_millis() as u64;

            match result {
                Ok(response) => {
                    // Record success
                    let mut health = self.health.write().await;
                    if let Some(h) = health.get_mut(&provider) {
                        h.record_success(latency);
                    }
                    return Ok(response);
                }
                Err(e) => {
                    // Record failure
                    {
                        let mut health = self.health.write().await;
                        if let Some(h) = health.get_mut(&provider) {
                            h.record_failure(&e.to_string());
                        }
                    }

                    // Check if retryable
                    if attempt < self.retry_policy.max_retries && self.retry_policy.is_retryable(&e) {
                        let delay = self.retry_policy.delay_for_retry(attempt);
                        tokio::time::sleep(delay).await;
                        attempt += 1;
                        continue;
                    }

                    return Err(e);
                }
            }
        }
    }

    /// Get cost tracker summary
    pub async fn cost_summary(&self) -> String {
        self.cost_tracker.read().await.summary()
    }

    /// Get health status
    pub async fn health_status(&self) -> HashMap<ModelProvider, ProviderHealth> {
        self.health.read().await.clone()
    }

    /// Reset health for a provider (force retry)
    pub async fn reset_health(&self, provider: ModelProvider) {
        let mut health = self.health.write().await;
        if let Some(h) = health.get_mut(&provider) {
            *h = ProviderHealth::default();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy() {
        let policy = RetryPolicy::default();
        
        assert_eq!(policy.delay_for_retry(0), Duration::from_millis(1000));
        assert_eq!(policy.delay_for_retry(1), Duration::from_millis(2000));
        assert_eq!(policy.delay_for_retry(2), Duration::from_millis(4000));
    }

    #[test]
    fn test_cost_tracking() {
        let mut tracker = CostTracker::new();
        
        let response = ModelResponse {
            content: "test".into(),
            usage: Some(super::super::providers::TokenUsage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            }),
            model: "gpt-4".into(),
            finish_reason: None,
        };
        
        tracker.record("openai", &response);
        assert!(tracker.total_cost_cents > 0);
    }

    #[test]
    fn test_provider_health() {
        let mut health = ProviderHealth::default();
        
        health.record_success(100);
        assert!(health.should_use());
        
        health.record_failure("error");
        health.record_failure("error");
        health.record_failure("error");
        assert!(!health.should_use());
    }
}
