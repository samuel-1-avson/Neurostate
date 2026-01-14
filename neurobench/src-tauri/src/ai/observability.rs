//! Observability - Distributed Tracing & Metrics
//!
//! Provides production observability for the AI engine:
//! - Distributed tracing (span creation, context propagation)
//! - Metrics collection (latency, tokens, costs)
//! - Error tracking and alerting
//! - Performance profiling

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

// =============================================================================
// TYPES
// =============================================================================

/// A trace span for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    /// Unique span ID
    pub span_id: String,
    /// Parent span ID (if nested)
    pub parent_id: Option<String>,
    /// Trace ID (groups all spans in a request)
    pub trace_id: String,
    /// Operation name
    pub operation: String,
    /// Service/component name
    pub service: String,
    /// Start timestamp
    pub start_time: DateTime<Utc>,
    /// End timestamp
    pub end_time: Option<DateTime<Utc>>,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Span status
    pub status: SpanStatus,
    /// Tags/attributes
    pub attributes: HashMap<String, serde_json::Value>,
    /// Events that occurred during the span
    pub events: Vec<SpanEvent>,
}

impl Span {
    pub fn new(operation: &str, service: &str, trace_id: &str) -> Self {
        Self {
            span_id: generate_id(),
            parent_id: None,
            trace_id: trace_id.to_string(),
            operation: operation.to_string(),
            service: service.to_string(),
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            status: SpanStatus::InProgress,
            attributes: HashMap::new(),
            events: Vec::new(),
        }
    }

    pub fn with_parent(mut self, parent_id: &str) -> Self {
        self.parent_id = Some(parent_id.to_string());
        self
    }

    pub fn set_attribute(&mut self, key: &str, value: impl Into<serde_json::Value>) {
        self.attributes.insert(key.to_string(), value.into());
    }

    pub fn add_event(&mut self, name: &str, attributes: HashMap<String, serde_json::Value>) {
        self.events.push(SpanEvent {
            name: name.to_string(),
            timestamp: Utc::now(),
            attributes,
        });
    }

    pub fn finish(&mut self) {
        self.end_time = Some(Utc::now());
        self.duration_ms = Some(
            self.end_time.unwrap()
                .signed_duration_since(self.start_time)
                .num_milliseconds() as u64
        );
        if self.status == SpanStatus::InProgress {
            self.status = SpanStatus::Ok;
        }
    }

    pub fn finish_with_error(&mut self, error: &str) {
        self.finish();
        self.status = SpanStatus::Error;
        self.set_attribute("error.message", error);
    }
}

/// Status of a span
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpanStatus {
    InProgress,
    Ok,
    Error,
}

/// An event within a span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp: DateTime<Utc>,
    pub attributes: HashMap<String, serde_json::Value>,
}

// =============================================================================
// METRICS
// =============================================================================

/// Metrics for AI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIMetrics {
    /// Total requests made
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Total tokens used
    pub total_tokens: u64,
    /// Input tokens
    pub input_tokens: u64,
    /// Output tokens
    pub output_tokens: u64,
    /// Total cost (USD cents)
    pub total_cost_cents: u64,
    /// Average latency (ms)
    pub avg_latency_ms: f64,
    /// P50 latency
    pub p50_latency_ms: u64,
    /// P95 latency
    pub p95_latency_ms: u64,
    /// P99 latency
    pub p99_latency_ms: u64,
    /// Requests by provider
    pub requests_by_provider: HashMap<String, u64>,
    /// Tokens by model
    pub tokens_by_model: HashMap<String, u64>,
    /// Errors by type
    pub errors_by_type: HashMap<String, u64>,
}

impl Default for AIMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_tokens: 0,
            input_tokens: 0,
            output_tokens: 0,
            total_cost_cents: 0,
            avg_latency_ms: 0.0,
            p50_latency_ms: 0,
            p95_latency_ms: 0,
            p99_latency_ms: 0,
            requests_by_provider: HashMap::new(),
            tokens_by_model: HashMap::new(),
            errors_by_type: HashMap::new(),
        }
    }
}

/// Cost rates for different models (per 1M tokens)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRates {
    pub input_per_million: f64,
    pub output_per_million: f64,
}

impl CostRates {
    pub fn gemini_flash() -> Self {
        Self { input_per_million: 0.075, output_per_million: 0.30 }
    }

    pub fn gemini_pro() -> Self {
        Self { input_per_million: 1.25, output_per_million: 5.00 }
    }

    pub fn gpt4_turbo() -> Self {
        Self { input_per_million: 10.00, output_per_million: 30.00 }
    }

    pub fn gpt35_turbo() -> Self {
        Self { input_per_million: 0.50, output_per_million: 1.50 }
    }

    pub fn ollama_local() -> Self {
        Self { input_per_million: 0.0, output_per_million: 0.0 }
    }
}

// =============================================================================
// TRACER
// =============================================================================

/// Distributed tracer for AI operations
pub struct Tracer {
    /// Service name
    service_name: String,
    /// Active spans
    active_spans: Arc<RwLock<HashMap<String, Span>>>,
    /// Completed spans (for export)
    completed_spans: Arc<RwLock<Vec<Span>>>,
    /// Maximum completed spans to keep
    max_completed: usize,
    /// Export callback
    exporter: Option<Arc<dyn SpanExporter + Send + Sync>>,
}

impl Tracer {
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            active_spans: Arc::new(RwLock::new(HashMap::new())),
            completed_spans: Arc::new(RwLock::new(Vec::new())),
            max_completed: 1000,
            exporter: None,
        }
    }

    pub fn with_exporter(mut self, exporter: Arc<dyn SpanExporter + Send + Sync>) -> Self {
        self.exporter = Some(exporter);
        self
    }

    /// Start a new trace
    pub async fn start_trace(&self, operation: &str) -> Span {
        let trace_id = generate_id();
        let span = Span::new(operation, &self.service_name, &trace_id);
        
        let mut active = self.active_spans.write().await;
        active.insert(span.span_id.clone(), span.clone());
        
        span
    }

    /// Start a child span
    pub async fn start_span(&self, operation: &str, parent: &Span) -> Span {
        let span = Span::new(operation, &self.service_name, &parent.trace_id)
            .with_parent(&parent.span_id);
        
        let mut active = self.active_spans.write().await;
        active.insert(span.span_id.clone(), span.clone());
        
        span
    }

    /// End a span
    pub async fn end_span(&self, mut span: Span) {
        span.finish();
        
        // Remove from active
        {
            let mut active = self.active_spans.write().await;
            active.remove(&span.span_id);
        }

        // Add to completed
        {
            let mut completed = self.completed_spans.write().await;
            completed.push(span.clone());
            
            // Trim if too many
            let len = completed.len();
            let max = self.max_completed;
            if len > max {
                let drain_count = len - max;
                completed.drain(0..drain_count);
            }
        }

        // Export if configured
        if let Some(exporter) = &self.exporter {
            let _ = exporter.export(&span).await;
        }
    }

    /// End a span with error
    pub async fn end_span_error(&self, mut span: Span, error: &str) {
        span.finish_with_error(error);
        
        {
            let mut active = self.active_spans.write().await;
            active.remove(&span.span_id);
        }

        {
            let mut completed = self.completed_spans.write().await;
            completed.push(span.clone());
        }

        if let Some(exporter) = &self.exporter {
            let _ = exporter.export(&span).await;
        }
    }

    /// Get recent completed spans
    pub async fn get_completed_spans(&self, limit: usize) -> Vec<Span> {
        let completed = self.completed_spans.read().await;
        completed.iter().rev().take(limit).cloned().collect()
    }

    /// Get active spans count
    pub async fn active_count(&self) -> usize {
        self.active_spans.read().await.len()
    }
}

/// Trait for exporting spans
#[async_trait::async_trait]
pub trait SpanExporter: Send + Sync {
    async fn export(&self, span: &Span) -> Result<(), String>;
}

/// Console exporter (for development)
pub struct ConsoleExporter;

#[async_trait::async_trait]
impl SpanExporter for ConsoleExporter {
    async fn export(&self, span: &Span) -> Result<(), String> {
        let status = match span.status {
            SpanStatus::Ok => "✓",
            SpanStatus::Error => "✗",
            SpanStatus::InProgress => "...",
        };
        
        log::info!(
            "[TRACE] {} {} | {} | {}ms | {}",
            status,
            span.operation,
            span.trace_id,
            span.duration_ms.unwrap_or(0),
            span.service
        );
        
        Ok(())
    }
}

/// JSON file exporter
pub struct JsonFileExporter {
    path: std::path::PathBuf,
}

impl JsonFileExporter {
    pub fn new(path: std::path::PathBuf) -> Self {
        Self { path }
    }
}

#[async_trait::async_trait]
impl SpanExporter for JsonFileExporter {
    async fn export(&self, span: &Span) -> Result<(), String> {
        use std::io::Write;
        
        let json = serde_json::to_string(span)
            .map_err(|e| e.to_string())?;
        
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| e.to_string())?;
        
        writeln!(file, "{}", json)
            .map_err(|e| e.to_string())?;
        
        Ok(())
    }
}

// =============================================================================
// METRICS COLLECTOR
// =============================================================================

/// Metrics collector for AI operations
pub struct MetricsCollector {
    /// Total requests counter
    total_requests: AtomicU64,
    /// Successful requests counter
    successful_requests: AtomicU64,
    /// Failed requests counter
    failed_requests: AtomicU64,
    /// Total tokens counter
    total_tokens: AtomicU64,
    /// Input tokens counter
    input_tokens: AtomicU64,
    /// Output tokens counter
    output_tokens: AtomicU64,
    /// Total cost (in millicents for precision)
    total_cost_millicents: AtomicU64,
    /// Latency samples (for percentiles)
    latencies: Arc<RwLock<Vec<u64>>>,
    /// Per-provider request counts
    provider_counts: Arc<RwLock<HashMap<String, u64>>>,
    /// Per-model token counts
    model_tokens: Arc<RwLock<HashMap<String, u64>>>,
    /// Error counts by type
    error_counts: Arc<RwLock<HashMap<String, u64>>>,
    /// Cost rates by model
    cost_rates: Arc<RwLock<HashMap<String, CostRates>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        let mut default_rates = HashMap::new();
        default_rates.insert("gemini-1.5-flash".to_string(), CostRates::gemini_flash());
        default_rates.insert("gemini-1.5-pro".to_string(), CostRates::gemini_pro());
        default_rates.insert("gpt-4-turbo".to_string(), CostRates::gpt4_turbo());
        default_rates.insert("gpt-3.5-turbo".to_string(), CostRates::gpt35_turbo());

        Self {
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            total_tokens: AtomicU64::new(0),
            input_tokens: AtomicU64::new(0),
            output_tokens: AtomicU64::new(0),
            total_cost_millicents: AtomicU64::new(0),
            latencies: Arc::new(RwLock::new(Vec::new())),
            provider_counts: Arc::new(RwLock::new(HashMap::new())),
            model_tokens: Arc::new(RwLock::new(HashMap::new())),
            error_counts: Arc::new(RwLock::new(HashMap::new())),
            cost_rates: Arc::new(RwLock::new(default_rates)),
        }
    }

    /// Record a successful request
    pub async fn record_success(
        &self,
        provider: &str,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
        latency_ms: u64,
    ) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
        
        let total = input_tokens as u64 + output_tokens as u64;
        self.total_tokens.fetch_add(total, Ordering::Relaxed);
        self.input_tokens.fetch_add(input_tokens as u64, Ordering::Relaxed);
        self.output_tokens.fetch_add(output_tokens as u64, Ordering::Relaxed);

        // Calculate cost
        let cost_millicents = self.calculate_cost(model, input_tokens, output_tokens).await;
        self.total_cost_millicents.fetch_add(cost_millicents, Ordering::Relaxed);

        // Record latency
        {
            let mut latencies = self.latencies.write().await;
            latencies.push(latency_ms);
            // Keep only last 10000 samples
            let len = latencies.len();
            if len > 10000 {
                let drain_count = len - 10000;
                latencies.drain(0..drain_count);
            }
        }

        // Update provider count
        {
            let mut counts = self.provider_counts.write().await;
            *counts.entry(provider.to_string()).or_default() += 1;
        }

        // Update model tokens
        {
            let mut tokens = self.model_tokens.write().await;
            *tokens.entry(model.to_string()).or_default() += total;
        }
    }

    /// Record a failed request
    pub async fn record_failure(&self, provider: &str, error_type: &str, latency_ms: u64) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.failed_requests.fetch_add(1, Ordering::Relaxed);

        // Record latency
        {
            let mut latencies = self.latencies.write().await;
            latencies.push(latency_ms);
        }

        // Update provider count
        {
            let mut counts = self.provider_counts.write().await;
            *counts.entry(provider.to_string()).or_default() += 1;
        }

        // Update error count
        {
            let mut errors = self.error_counts.write().await;
            *errors.entry(error_type.to_string()).or_default() += 1;
        }
    }

    /// Calculate cost in millicents
    async fn calculate_cost(&self, model: &str, input: u32, output: u32) -> u64 {
        let rates = self.cost_rates.read().await;
        
        if let Some(rate) = rates.get(model) {
            let input_cost = (input as f64 / 1_000_000.0) * rate.input_per_million;
            let output_cost = (output as f64 / 1_000_000.0) * rate.output_per_million;
            ((input_cost + output_cost) * 100_000.0) as u64 // Convert to millicents
        } else {
            0
        }
    }

    /// Get current metrics snapshot
    pub async fn get_metrics(&self) -> AIMetrics {
        let latencies = self.latencies.read().await;
        let provider_counts = self.provider_counts.read().await;
        let model_tokens = self.model_tokens.read().await;
        let error_counts = self.error_counts.read().await;

        // Calculate percentiles
        let (avg, p50, p95, p99) = if latencies.is_empty() {
            (0.0, 0, 0, 0)
        } else {
            let mut sorted: Vec<u64> = latencies.clone();
            sorted.sort();
            let len = sorted.len();
            
            let avg = sorted.iter().sum::<u64>() as f64 / len as f64;
            let p50 = sorted[len / 2];
            let p95 = sorted[(len as f64 * 0.95) as usize];
            let p99 = sorted[(len as f64 * 0.99).min(len as f64 - 1.0) as usize];
            
            (avg, p50, p95, p99)
        };

        AIMetrics {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            successful_requests: self.successful_requests.load(Ordering::Relaxed),
            failed_requests: self.failed_requests.load(Ordering::Relaxed),
            total_tokens: self.total_tokens.load(Ordering::Relaxed),
            input_tokens: self.input_tokens.load(Ordering::Relaxed),
            output_tokens: self.output_tokens.load(Ordering::Relaxed),
            total_cost_cents: self.total_cost_millicents.load(Ordering::Relaxed) / 1000,
            avg_latency_ms: avg,
            p50_latency_ms: p50,
            p95_latency_ms: p95,
            p99_latency_ms: p99,
            requests_by_provider: provider_counts.clone(),
            tokens_by_model: model_tokens.clone(),
            errors_by_type: error_counts.clone(),
        }
    }

    /// Reset all metrics
    pub async fn reset(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.successful_requests.store(0, Ordering::Relaxed);
        self.failed_requests.store(0, Ordering::Relaxed);
        self.total_tokens.store(0, Ordering::Relaxed);
        self.input_tokens.store(0, Ordering::Relaxed);
        self.output_tokens.store(0, Ordering::Relaxed);
        self.total_cost_millicents.store(0, Ordering::Relaxed);
        
        self.latencies.write().await.clear();
        self.provider_counts.write().await.clear();
        self.model_tokens.write().await.clear();
        self.error_counts.write().await.clear();
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.total_requests.load(Ordering::Relaxed);
        if total == 0 {
            return 1.0;
        }
        self.successful_requests.load(Ordering::Relaxed) as f64 / total as f64
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// OBSERVABILITY MANAGER
// =============================================================================

/// Combined observability manager
pub struct ObservabilityManager {
    /// Tracer for distributed tracing
    pub tracer: Tracer,
    /// Metrics collector
    pub metrics: MetricsCollector,
}

impl ObservabilityManager {
    pub fn new(service_name: &str) -> Self {
        Self {
            tracer: Tracer::new(service_name),
            metrics: MetricsCollector::new(),
        }
    }

    pub fn with_console_exporter(mut self) -> Self {
        self.tracer = self.tracer.with_exporter(Arc::new(ConsoleExporter));
        self
    }

    pub fn with_file_exporter(mut self, path: std::path::PathBuf) -> Self {
        self.tracer = self.tracer.with_exporter(Arc::new(JsonFileExporter::new(path)));
        self
    }

    /// Start a traced AI operation
    pub async fn start_ai_operation(&self, operation: &str) -> TracedOperation {
        let span = self.tracer.start_trace(operation).await;
        TracedOperation {
            span,
            start_time: std::time::Instant::now(),
        }
    }

    /// End a traced AI operation with success
    pub async fn end_ai_operation_success(
        &self,
        op: TracedOperation,
        provider: &str,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) {
        let latency = op.start_time.elapsed().as_millis() as u64;
        
        let mut span = op.span;
        span.set_attribute("ai.provider", provider);
        span.set_attribute("ai.model", model);
        span.set_attribute("ai.input_tokens", input_tokens as i64);
        span.set_attribute("ai.output_tokens", output_tokens as i64);
        span.set_attribute("ai.latency_ms", latency as i64);
        
        self.tracer.end_span(span).await;
        self.metrics.record_success(provider, model, input_tokens, output_tokens, latency).await;
    }

    /// End a traced AI operation with failure
    pub async fn end_ai_operation_failure(
        &self,
        op: TracedOperation,
        provider: &str,
        error: &str,
        error_type: &str,
    ) {
        let latency = op.start_time.elapsed().as_millis() as u64;
        
        let mut span = op.span;
        span.set_attribute("ai.provider", provider);
        span.set_attribute("error.type", error_type);
        
        self.tracer.end_span_error(span, error).await;
        self.metrics.record_failure(provider, error_type, latency).await;
    }

    /// Get health status
    pub async fn health(&self) -> HealthStatus {
        let metrics = self.metrics.get_metrics().await;
        let success_rate = self.metrics.success_rate();
        
        let status = if success_rate >= 0.99 {
            "healthy"
        } else if success_rate >= 0.95 {
            "degraded"
        } else {
            "unhealthy"
        };

        HealthStatus {
            status: status.to_string(),
            success_rate,
            total_requests: metrics.total_requests,
            avg_latency_ms: metrics.avg_latency_ms,
            active_spans: self.tracer.active_count().await,
        }
    }
}

/// A traced operation handle
pub struct TracedOperation {
    pub span: Span,
    pub start_time: std::time::Instant,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub success_rate: f64,
    pub total_requests: u64,
    pub avg_latency_ms: f64,
    pub active_spans: usize,
}

// =============================================================================
// HELPERS
// =============================================================================

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", nanos)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = Span::new("test_op", "test_service", "trace_123");
        assert_eq!(span.operation, "test_op");
        assert_eq!(span.service, "test_service");
        assert_eq!(span.status, SpanStatus::InProgress);
    }

    #[test]
    fn test_span_finish() {
        let mut span = Span::new("test_op", "test_service", "trace_123");
        std::thread::sleep(std::time::Duration::from_millis(10));
        span.finish();
        
        assert_eq!(span.status, SpanStatus::Ok);
        assert!(span.duration_ms.is_some());
        assert!(span.duration_ms.unwrap() >= 10);
    }

    #[test]
    fn test_span_finish_error() {
        let mut span = Span::new("test_op", "test_service", "trace_123");
        span.finish_with_error("Something went wrong");
        
        assert_eq!(span.status, SpanStatus::Error);
        assert!(span.attributes.contains_key("error.message"));
    }

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        
        collector.record_success("openai", "gpt-3.5-turbo", 100, 50, 200).await;
        collector.record_success("openai", "gpt-3.5-turbo", 150, 75, 300).await;
        collector.record_failure("openai", "rate_limit", 100).await;
        
        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.successful_requests, 2);
        assert_eq!(metrics.failed_requests, 1);
        assert_eq!(metrics.total_tokens, 375);
    }

    #[tokio::test]
    async fn test_tracer() {
        let tracer = Tracer::new("test_service");
        
        let span = tracer.start_trace("test_operation").await;
        assert_eq!(tracer.active_count().await, 1);
        
        tracer.end_span(span).await;
        assert_eq!(tracer.active_count().await, 0);
        
        let completed = tracer.get_completed_spans(10).await;
        assert_eq!(completed.len(), 1);
    }

    #[test]
    fn test_cost_calculation() {
        let rates = CostRates::gpt35_turbo();
        // 1M input tokens at $0.50 = $0.50
        // 1M output tokens at $1.50 = $1.50
        assert_eq!(rates.input_per_million, 0.50);
        assert_eq!(rates.output_per_million, 1.50);
    }
}
