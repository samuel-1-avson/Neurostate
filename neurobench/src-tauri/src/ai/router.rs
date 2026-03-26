//! AI Router - Intelligent model routing with fallback, health checks, and cost tracking
//!
//! Provides a unified interface to multiple AI providers with automatic failover.
//! 
//! Phase 3 Enhancements:
//! - User-level rate limiting
//! - Semantic cache integration
//! - Observability hooks

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use super::providers::{AIModel, ModelError, ModelResponse, ChatMessage, ModelProvider};
use super::observability::{ObservabilityManager, TracedOperation};
use super::semantic_cache::{SemanticCache, CacheResult};

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
// RATE LIMITING
// =============================================================================

/// User-level rate limiter using sliding window
#[derive(Debug)]
pub struct RateLimiter {
    /// Requests per window per user
    requests_per_window: u32,
    /// Window duration
    window_duration: Duration,
    /// User request timestamps
    user_requests: RwLock<HashMap<String, Vec<Instant>>>,
}

impl RateLimiter {
    pub fn new(requests_per_window: u32, window_duration: Duration) -> Self {
        Self {
            requests_per_window,
            window_duration,
            user_requests: RwLock::new(HashMap::new()),
        }
    }

    /// Default: 60 requests per minute
    pub fn default_limits() -> Self {
        Self::new(60, Duration::from_secs(60))
    }

    /// Strict: 10 requests per minute
    pub fn strict() -> Self {
        Self::new(10, Duration::from_secs(60))
    }

    /// Check if user can make a request
    pub async fn check(&self, user_id: &str) -> RateLimitResult {
        let mut users = self.user_requests.write().await;
        let now = Instant::now();
        let cutoff = now - self.window_duration;

        let requests = users.entry(user_id.to_string()).or_default();
        
        // Remove old requests
        requests.retain(|t| *t > cutoff);

        if requests.len() >= self.requests_per_window as usize {
            let oldest = requests.first().copied().unwrap_or(now);
            let reset_in = self.window_duration.saturating_sub(now.duration_since(oldest));
            return RateLimitResult::Limited {
                remaining: 0,
                reset_in,
            };
        }

        let remaining = self.requests_per_window as usize - requests.len();
        RateLimitResult::Allowed {
            remaining: remaining as u32,
        }
    }

    /// Record a request for a user
    pub async fn record(&self, user_id: &str) {
        let mut users = self.user_requests.write().await;
        users.entry(user_id.to_string()).or_default().push(Instant::now());
    }

    /// Get usage stats for a user
    pub async fn usage(&self, user_id: &str) -> RateLimitUsage {
        let users = self.user_requests.read().await;
        let now = Instant::now();
        let cutoff = now - self.window_duration;

        let requests = users.get(user_id);
        let used = requests
            .map(|r| r.iter().filter(|t| **t > cutoff).count())
            .unwrap_or(0);

        RateLimitUsage {
            used: used as u32,
            limit: self.requests_per_window,
            remaining: self.requests_per_window.saturating_sub(used as u32),
            window_secs: self.window_duration.as_secs(),
        }
    }

    /// Clear rate limit data for a user
    pub async fn clear(&self, user_id: &str) {
        let mut users = self.user_requests.write().await;
        users.remove(user_id);
    }
}

/// Rate limit check result
#[derive(Debug, Clone)]
pub enum RateLimitResult {
    Allowed { remaining: u32 },
    Limited { remaining: u32, reset_in: Duration },
}

impl RateLimitResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, RateLimitResult::Allowed { .. })
    }
}

/// Rate limit usage stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitUsage {
    pub used: u32,
    pub limit: u32,
    pub remaining: u32,
    pub window_secs: u64,
}

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
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
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

    /// Record cache hit
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    /// Record cache miss
    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }

    /// Get cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            return 0.0;
        }
        self.cache_hits as f64 / total as f64
    }

    /// Get summary
    pub fn summary(&self) -> String {
        format!(
            "Tokens: {} in / {} out | Cost: ${:.2} | Cache: {:.1}% hit | Requests: {:?}",
            self.input_tokens,
            self.output_tokens,
            self.total_cost_cents as f64 / 100.0,
            self.cache_hit_rate() * 100.0,
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
    /// Total requests
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
}

impl Default for ProviderHealth {
    fn default() -> Self {
        Self {
            available: true,
            last_success: None,
            last_error: None,
            consecutive_failures: 0,
            avg_latency_ms: 0.0,
            total_requests: 0,
            successful_requests: 0,
        }
    }
}

impl ProviderHealth {
    /// Record a successful request
    pub fn record_success(&mut self, latency_ms: u64) {
        self.available = true;
        self.last_success = Some(Instant::now());
        self.consecutive_failures = 0;
        self.total_requests += 1;
        self.successful_requests += 1;
        // Rolling average
        self.avg_latency_ms = self.avg_latency_ms * 0.9 + latency_ms as f64 * 0.1;
    }

    /// Record a failure
    pub fn record_failure(&mut self, error: &str) {
        self.consecutive_failures += 1;
        self.last_error = Some(error.to_string());
        self.total_requests += 1;
        
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

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 1.0;
        }
        self.successful_requests as f64 / self.total_requests as f64
    }
}

// =============================================================================
// ROUTER CONFIG
// =============================================================================

/// Router configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Enable fallback to next provider
    pub fallback_enabled: bool,
    /// Enable response caching
    pub cache_enabled: bool,
    /// Enable observability tracing
    pub tracing_enabled: bool,
    /// Enable rate limiting
    pub rate_limiting_enabled: bool,
    /// Default user for rate limiting (if no user specified)
    pub default_user: String,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            fallback_enabled: true,
            cache_enabled: true,
            tracing_enabled: true,
            rate_limiting_enabled: true,
            default_user: "default".to_string(),
        }
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
    /// Configuration
    config: RouterConfig,
    /// Rate limiter
    rate_limiter: RateLimiter,
    /// Semantic cache (optional)
    cache: Option<Arc<SemanticCache>>,
    /// Observability manager (optional)
    observability: Option<Arc<ObservabilityManager>>,
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
            config: RouterConfig::default(),
            rate_limiter: RateLimiter::default_limits(),
            cache: None,
            observability: None,
        }
    }

    /// Set retry policy
    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    /// Set configuration
    pub fn with_config(mut self, config: RouterConfig) -> Self {
        self.config = config;
        self
    }

    /// Add semantic cache
    pub fn with_cache(mut self, cache: Arc<SemanticCache>) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Add observability
    pub fn with_observability(mut self, obs: Arc<ObservabilityManager>) -> Self {
        self.observability = Some(obs);
        self
    }

    /// Set rate limiter
    pub fn with_rate_limiter(mut self, limiter: RateLimiter) -> Self {
        self.rate_limiter = limiter;
        self
    }

    /// Disable fallback (use only first provider)
    pub fn without_fallback(mut self) -> Self {
        self.config.fallback_enabled = false;
        self
    }

    /// Check rate limit for a user
    pub async fn check_rate_limit(&self, user_id: Option<&str>) -> RateLimitResult {
        if !self.config.rate_limiting_enabled {
            return RateLimitResult::Allowed { remaining: u32::MAX };
        }
        let user = user_id.unwrap_or(&self.config.default_user);
        self.rate_limiter.check(user).await
    }

    /// Get rate limit usage for a user
    pub async fn rate_limit_usage(&self, user_id: Option<&str>) -> RateLimitUsage {
        let user = user_id.unwrap_or(&self.config.default_user);
        self.rate_limiter.usage(user).await
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

    /// Generate with automatic retry, fallback, caching, and observability
    pub async fn generate(&self, prompt: &str) -> Result<ModelResponse, RouterError> {
        self.generate_for_user(None, "", prompt).await
    }

    /// Generate with system prompt
    pub async fn generate_with_system(
        &self,
        system: &str,
        prompt: &str,
    ) -> Result<ModelResponse, RouterError> {
        self.generate_for_user(None, system, prompt).await
    }

    /// Generate for a specific user (with rate limiting)
    pub async fn generate_for_user(
        &self,
        user_id: Option<&str>,
        system: &str,
        prompt: &str,
    ) -> Result<ModelResponse, RouterError> {
        let user = user_id.unwrap_or(&self.config.default_user);

        // Check rate limit
        if self.config.rate_limiting_enabled {
            let limit_result = self.rate_limiter.check(user).await;
            if !limit_result.is_allowed() {
                if let RateLimitResult::Limited { reset_in, .. } = limit_result {
                    return Err(RouterError::RateLimited { 
                        user: user.to_string(),
                        reset_in_secs: reset_in.as_secs(),
                    });
                }
            }
            self.rate_limiter.record(user).await;
        }

        // Start tracing if enabled
        let trace_op = if self.config.tracing_enabled {
            if let Some(obs) = &self.observability {
                Some(obs.start_ai_operation("router_generate").await)
            } else {
                None
            }
        } else {
            None
        };

        // Build cache key
        let cache_key = format!("{}|{}", system, prompt);

        // Check cache if enabled
        if self.config.cache_enabled {
            if let Some(cache) = &self.cache {
                if let Ok(result) = cache.get(&cache_key).await {
                    match result {
                        CacheResult::ExactHit(entry) | CacheResult::SemanticHit { entry, .. } => {
                            // Cache hit!
                            {
                                let mut cost = self.cost_tracker.write().await;
                                cost.record_cache_hit();
                            }

                            let response = ModelResponse {
                                content: entry.response.clone(),
                                model: "cached".to_string(),
                                usage: None,
                                finish_reason: Some("cached".to_string()),
                            };

                            // End trace with cache hit
                            if let Some(op) = trace_op {
                                if let Some(obs) = &self.observability {
                                    obs.end_ai_operation_success(op, "cache", "cached", 0, 0).await;
                                }
                            }

                            return Ok(response);
                        }
                        _ => {
                            let mut cost = self.cost_tracker.write().await;
                            cost.record_cache_miss();
                        }
                    }
                }
            }
        }

        // Route to provider
        let result = self.route_to_provider(system, prompt).await;

        // Handle observability
        match &result {
            Ok(response) => {
                // Cache the response
                if self.config.cache_enabled {
                    if let Some(cache) = &self.cache {
                        let _ = cache.put(&cache_key, &response.content, &response.model).await;
                    }
                }

                // End trace with success
                if let Some(op) = trace_op {
                    if let Some(obs) = &self.observability {
                        let (input, output) = response.usage.as_ref()
                            .map(|u| (u.prompt_tokens, u.completion_tokens))
                            .unwrap_or((0, 0));
                        obs.end_ai_operation_success(op, &response.model, &response.model, input, output).await;
                    }
                }
            }
            Err(e) => {
                // End trace with error
                if let Some(op) = trace_op {
                    if let Some(obs) = &self.observability {
                        obs.end_ai_operation_failure(op, "unknown", &e.to_string(), "router_error").await;
                    }
                }
            }
        }

        result.map_err(|e| RouterError::Model(e))
    }

    /// Chat with message history
    pub async fn chat(&self, messages: &[ChatMessage]) -> Result<ModelResponse, RouterError> {
        self.chat_for_user(None, messages).await
    }

    /// Chat for a specific user
    pub async fn chat_for_user(
        &self,
        user_id: Option<&str>,
        messages: &[ChatMessage],
    ) -> Result<ModelResponse, RouterError> {
        let user = user_id.unwrap_or(&self.config.default_user);

        // Check rate limit
        if self.config.rate_limiting_enabled {
            let limit_result = self.rate_limiter.check(user).await;
            if !limit_result.is_allowed() {
                if let RateLimitResult::Limited { reset_in, .. } = limit_result {
                    return Err(RouterError::RateLimited {
                        user: user.to_string(),
                        reset_in_secs: reset_in.as_secs(),
                    });
                }
            }
            self.rate_limiter.record(user).await;
        }

        // Start tracing
        let trace_op = if self.config.tracing_enabled {
            if let Some(obs) = &self.observability {
                Some(obs.start_ai_operation("router_chat").await)
            } else {
                None
            }
        } else {
            None
        };

        let result = self.route_chat_to_provider(messages).await;

        // Handle observability
        match &result {
            Ok(response) => {
                if let Some(op) = trace_op {
                    if let Some(obs) = &self.observability {
                        let (input, output) = response.usage.as_ref()
                            .map(|u| (u.prompt_tokens, u.completion_tokens))
                            .unwrap_or((0, 0));
                        obs.end_ai_operation_success(op, &response.model, &response.model, input, output).await;
                    }
                }
            }
            Err(e) => {
                if let Some(op) = trace_op {
                    if let Some(obs) = &self.observability {
                        obs.end_ai_operation_failure(op, "unknown", &e.to_string(), "router_error").await;
                    }
                }
            }
        }

        result.map_err(|e| RouterError::Model(e))
    }

    /// Internal: Route to providers with fallback
    async fn route_to_provider(
        &self,
        system: &str,
        prompt: &str,
    ) -> Result<ModelResponse, ModelError> {
        let providers: Vec<_> = if self.config.fallback_enabled {
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

    /// Internal: Route chat to providers
    async fn route_chat_to_provider(
        &self,
        messages: &[ChatMessage],
    ) -> Result<ModelResponse, ModelError> {
        let providers: Vec<_> = if self.config.fallback_enabled {
            self.providers.iter().collect()
        } else {
            self.providers.iter().take(1).collect()
        };

        let mut last_error = ModelError::ConfigError("No providers available".into());

        for (provider, model) in providers {
            if !model.is_configured() {
                continue;
            }

            {
                let health = self.health.read().await;
                if let Some(h) = health.get(provider) {
                    if !h.should_use() {
                        continue;
                    }
                }
            }

            let start = Instant::now();
            let result = model.chat(messages).await;
            let latency = start.elapsed().as_millis() as u64;

            match result {
                Ok(response) => {
                    {
                        let mut health = self.health.write().await;
                        if let Some(h) = health.get_mut(provider) {
                            h.record_success(latency);
                        }
                    }
                    let mut cost = self.cost_tracker.write().await;
                    cost.record(model.name(), &response);
                    return Ok(response);
                }
                Err(e) => {
                    {
                        let mut health = self.health.write().await;
                        if let Some(h) = health.get_mut(provider) {
                            h.record_failure(&e.to_string());
                        }
                    }
                    last_error = e;
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

    /// Get cost tracker data
    pub async fn cost_data(&self) -> CostTracker {
        self.cost_tracker.read().await.clone()
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

    /// Get router stats
    pub async fn stats(&self) -> RouterStats {
        let health = self.health.read().await;
        let cost = self.cost_tracker.read().await;

        let providers: Vec<_> = health.iter()
            .map(|(p, h)| ProviderStats {
                provider: *p,
                available: h.available,
                success_rate: h.success_rate(),
                avg_latency_ms: h.avg_latency_ms,
                total_requests: h.total_requests,
            })
            .collect();

        RouterStats {
            providers,
            cache_hit_rate: cost.cache_hit_rate(),
            total_cost_cents: cost.total_cost_cents,
            total_tokens: cost.input_tokens + cost.output_tokens,
        }
    }
}

/// Router error types
#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("Rate limited for user {user}, reset in {reset_in_secs}s")]
    RateLimited { user: String, reset_in_secs: u64 },

    #[error("Model error: {0}")]
    Model(#[from] ModelError),

    #[error("No providers available")]
    NoProviders,

    #[error("Cache error: {0}")]
    CacheError(String),
}

/// Stats for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStats {
    pub provider: ModelProvider,
    pub available: bool,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub total_requests: u64,
}

/// Overall router stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterStats {
    pub providers: Vec<ProviderStats>,
    pub cache_hit_rate: f64,
    pub total_cost_cents: u64,
    pub total_tokens: u64,
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
    fn test_cost_tracking_cache() {
        let mut tracker = CostTracker::new();
        
        tracker.record_cache_hit();
        tracker.record_cache_hit();
        tracker.record_cache_miss();
        
        assert_eq!(tracker.cache_hits, 2);
        assert_eq!(tracker.cache_misses, 1);
        assert!((tracker.cache_hit_rate() - 0.6666).abs() < 0.01);
    }

    #[test]
    fn test_provider_health() {
        let mut health = ProviderHealth::default();
        
        health.record_success(100);
        assert!(health.should_use());
        assert_eq!(health.success_rate(), 1.0);
        
        health.record_failure("error");
        health.record_failure("error");
        health.record_failure("error");
        assert!(!health.should_use());
        assert_eq!(health.success_rate(), 0.25);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(3, Duration::from_secs(60));
        
        // First 3 requests should be allowed
        for _ in 0..3 {
            limiter.record("user1").await;
        }
        
        // 4th should be limited
        let result = limiter.check("user1").await;
        assert!(!result.is_allowed());
        
        // Different user should be allowed
        let result = limiter.check("user2").await;
        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn test_rate_limit_usage() {
        let limiter = RateLimiter::new(10, Duration::from_secs(60));
        
        limiter.record("user1").await;
        limiter.record("user1").await;
        
        let usage = limiter.usage("user1").await;
        assert_eq!(usage.used, 2);
        assert_eq!(usage.remaining, 8);
        assert_eq!(usage.limit, 10);
    }
}

