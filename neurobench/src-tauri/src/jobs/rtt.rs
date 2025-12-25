// RTT Streaming Module
//
// Provides RTT (Real-Time Transfer) streaming using the unified job manager.
// Key features:
// - Batched message emission (50-100ms or size limit)
// - Dropped count tracking
// - Device lock exclusivity with Flash
// - All events flow through JobEmitter (strict single-emitter pattern)

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::jobs::{
    JobManager, JobKind, JobRecord, JobEmitter, EmitterMessage,
    JobTerminal, CancelReason, InternalErrorCode,
};

// ==================== RTT Configuration ====================

/// RTT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RttConfig {
    /// Chip target (required)
    pub chip: String,
    /// RTT channels to monitor (0 = default terminal)
    pub channels: Vec<u32>,
    /// Polling interval in ms (default 10ms)
    pub poll_interval_ms: u64,
    /// Max batch size in lines before emitting
    pub max_batch_lines: usize,
    /// Max batch size in bytes before emitting
    pub max_batch_bytes: usize,
    /// Max time between emissions in ms
    pub max_batch_interval_ms: u64,
    /// Probe speed in kHz
    pub speed_khz: Option<u32>,
}

impl Default for RttConfig {
    fn default() -> Self {
        Self {
            chip: String::new(),
            channels: vec![0], // Default to channel 0
            poll_interval_ms: 10,
            max_batch_lines: 100,
            max_batch_bytes: 4096,
            max_batch_interval_ms: 100, // Emit at least every 100ms
            speed_khz: Some(4000),
        }
    }
}

// ==================== RTT Message Types ====================

/// Single RTT message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RttMessage {
    pub channel: u32,
    pub text: String,
    pub timestamp_ms: u64,
}

/// Batched RTT messages for efficient emission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RttBatch {
    pub messages: Vec<RttMessage>,
    pub dropped_count: u64,
    pub total_bytes: usize,
}

impl RttBatch {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            dropped_count: 0,
            total_bytes: 0,
        }
    }
    
    pub fn push(&mut self, msg: RttMessage) {
        self.total_bytes += msg.text.len();
        self.messages.push(msg);
    }
    
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
    
    pub fn len(&self) -> usize {
        self.messages.len()
    }
    
    pub fn clear(&mut self) {
        self.messages.clear();
        self.total_bytes = 0;
        // Note: dropped_count is cumulative, don't reset
    }
}

impl Default for RttBatch {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal RTT events (sent to emitter)
#[derive(Debug, Clone)]
pub enum RttEvent {
    Started {
        chip: String,
        channels: Vec<u32>,
    },
    Batch(RttBatch),
    Stopped {
        terminated_by: TerminatedBy,
        total_messages: u64,
        total_dropped: u64,
    },
    Cancelled {
        reason: CancelReason,
        terminated_by: TerminatedBy,
    },
    InternalError {
        error_code: RttErrorCode,
        message: String,
        details: Option<String>,
        retryable: bool,
    },
}

/// How the RTT stream was terminated
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminatedBy {
    Stopped,     // Clean stop requested
    Cancelled,   // User cancelled
    Killed,      // Forced teardown
    Lost,        // Probe connection lost
    InternalError,
}

/// RTT-specific error codes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RttErrorCode {
    NoProbeFound,
    ProbeOpenFailed,
    AttachFailed,
    TargetNotFound,
    RttNotAvailable,
    ChannelNotFound,
    ConnectionLost,
    IoError,
}

impl RttErrorCode {
    pub fn to_internal_code(&self) -> InternalErrorCode {
        match self {
            RttErrorCode::NoProbeFound => InternalErrorCode::ProbeNotFound,
            RttErrorCode::ProbeOpenFailed | RttErrorCode::AttachFailed => {
                InternalErrorCode::ProbeConnectionFailed
            }
            RttErrorCode::TargetNotFound => InternalErrorCode::ProbeConnectionFailed,
            RttErrorCode::RttNotAvailable | RttErrorCode::ChannelNotFound => {
                InternalErrorCode::RttStartFailed
            }
            RttErrorCode::ConnectionLost | RttErrorCode::IoError => {
                InternalErrorCode::IoError
            }
        }
    }
}

/// RTT error with full details
#[derive(Debug, Clone)]
pub struct RttError {
    pub code: RttErrorCode,
    pub message: String,
    pub details: Option<String>,
    pub retryable: bool,
}

impl RttError {
    pub fn no_probe() -> Self {
        Self {
            code: RttErrorCode::NoProbeFound,
            message: "No debug probe found. Connect ST-Link, J-Link, or CMSIS-DAP.".to_string(),
            details: None,
            retryable: true,
        }
    }
    
    pub fn target_not_found(chip: &str) -> Self {
        Self {
            code: RttErrorCode::TargetNotFound,
            message: format!("Target '{}' not found in probe-rs database", chip),
            details: None,
            retryable: false,
        }
    }
    
    pub fn rtt_not_available() -> Self {
        Self {
            code: RttErrorCode::RttNotAvailable,
            message: "RTT control block not found on target. Ensure firmware has RTT enabled.".to_string(),
            details: None,
            retryable: true,
        }
    }
    
    pub fn connection_lost() -> Self {
        Self {
            code: RttErrorCode::ConnectionLost,
            message: "Probe connection lost".to_string(),
            details: None,
            retryable: true,
        }
    }
}

// ==================== RTT Backend Trait ====================

/// RTT data callback
pub type RttDataCallback = mpsc::Sender<RttMessage>;

/// Backend trait for RTT operations
#[async_trait]
pub trait RttBackend: Send + Sync {
    /// Start RTT streaming, returns immediately
    /// Data is sent via the callback
    async fn start_rtt(
        &self,
        config: &RttConfig,
        data_callback: RttDataCallback,
        cancel_check: impl Fn() -> bool + Send + Sync + 'static,
    ) -> Result<(), RttError>;
    
    /// Stop RTT streaming
    async fn stop_rtt(&self) -> Result<(), RttError>;
    
    /// Check if RTT is active
    async fn is_rtt_active(&self) -> bool;
}

// ==================== Mock RTT Backend ====================

/// Mock RTT backend for testing without hardware
pub struct MockRttBackend {
    active: Arc<tokio::sync::Mutex<bool>>,
    message_interval_ms: u64,
    should_fail: bool,
}

impl MockRttBackend {
    pub fn new() -> Self {
        Self {
            active: Arc::new(tokio::sync::Mutex::new(false)),
            message_interval_ms: 100,
            should_fail: false,
        }
    }
    
    pub fn with_interval(mut self, ms: u64) -> Self {
        self.message_interval_ms = ms;
        self
    }
    
    pub fn failing(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

impl Default for MockRttBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RttBackend for MockRttBackend {
    async fn start_rtt(
        &self,
        config: &RttConfig,
        data_callback: RttDataCallback,
        cancel_check: impl Fn() -> bool + Send + Sync + 'static,
    ) -> Result<(), RttError> {
        if self.should_fail {
            return Err(RttError::rtt_not_available());
        }
        
        if config.chip.is_empty() {
            return Err(RttError::target_not_found(""));
        }
        
        *self.active.lock().await = true;
        let active = self.active.clone();
        let interval = Duration::from_millis(self.message_interval_ms);
        let channels = config.channels.clone();
        
        // Spawn mock message generator
        tokio::spawn(async move {
            let mut counter = 0u64;
            let start = Instant::now();
            
            loop {
                if cancel_check() || !*active.lock().await {
                    break;
                }
                
                // Generate mock messages for each channel
                for &channel in &channels {
                    let msg = RttMessage {
                        channel,
                        text: format!("[{}] Mock RTT message #{}\n", channel, counter),
                        timestamp_ms: start.elapsed().as_millis() as u64,
                    };
                    
                    if data_callback.send(msg).await.is_err() {
                        // Channel closed, stop
                        return;
                    }
                }
                
                counter += 1;
                tokio::time::sleep(interval).await;
            }
        });
        
        Ok(())
    }
    
    async fn stop_rtt(&self) -> Result<(), RttError> {
        *self.active.lock().await = false;
        Ok(())
    }
    
    async fn is_rtt_active(&self) -> bool {
        *self.active.lock().await
    }
}

// ==================== RTT Job Runner ====================

/// Run an RTT streaming job with batched event emission
pub async fn run_rtt_job<B: RttBackend + 'static>(
    job_manager: Arc<JobManager>,
    backend: Arc<B>,
    config: RttConfig,
    emit_event: impl Fn(String, serde_json::Value) + Send + Sync + Clone + 'static,
) -> Result<String, String> {
    // Validate config
    if config.chip.is_empty() {
        return Err("Chip not specified".to_string());
    }
    
    // Create job
    let (record, _tx) = job_manager.create_job(JobKind::Rtt);
    let job_id = record.id.clone();
    
    // Try to acquire device lock (exclusive with Flash)
    if let Err(msg) = job_manager.try_acquire_device(&job_id).await {
        let mut emitter = JobEmitter::new(&record);
        if let Some((event_name, payload)) = emitter.process(EmitterMessage::Terminal {
            terminal: JobTerminal::InternalError {
                error_code: InternalErrorCode::ProbeConnectionFailed,
                message: msg.clone(),
                retryable: true,
            },
        }).await {
            emit_event(event_name, payload);
        }
        job_manager.finish_job(&job_id).await;
        return Err(msg);
    }
    
    // Spawn RTT worker task
    let job_id_clone = job_id.clone();
    let record_clone = record.clone();
    let job_manager_clone = job_manager.clone();
    let emit_clone = emit_event.clone();
    
    tokio::spawn(async move {
        run_rtt_worker(
            record_clone,
            backend,
            config,
            job_manager_clone,
            emit_clone,
        ).await;
    });
    
    Ok(job_id)
}

/// RTT worker task - batches messages and sends through JobEmitter
async fn run_rtt_worker<B: RttBackend + 'static>(
    record: Arc<JobRecord>,
    backend: Arc<B>,
    config: RttConfig,
    job_manager: Arc<JobManager>,
    emit_event: impl Fn(String, serde_json::Value) + Send + Sync,
) {
    let mut emitter = JobEmitter::new(&record);
    let start = Instant::now();
    let mut total_messages = 0u64;
    let mut total_dropped = 0u64;
    
    // Create channel for RTT data
    let (data_tx, mut data_rx) = mpsc::channel::<RttMessage>(1024);
    
    // Emit started event
    let started_payload = serde_json::json!({
        "type": "started",
        "chip": config.chip,
        "channels": config.channels,
    });
    if let Some((event_name, payload)) = emitter.process(EmitterMessage::Custom {
        event_suffix: "started".to_string(),
        payload: started_payload,
    }).await {
        emit_event(event_name, payload);
    }
    
    // Start RTT backend
    let cancel_token = record.cancel_token.clone();
    let cancel_check = move || cancel_token.is_cancelled();
    
    if let Err(err) = backend.start_rtt(&config, data_tx, cancel_check).await {
        if let Some((event_name, payload)) = emitter.process(EmitterMessage::Terminal {
            terminal: JobTerminal::InternalError {
                error_code: err.code.to_internal_code(),
                message: err.message.clone(),
                retryable: err.retryable,
            },
        }).await {
            emit_event(event_name, payload);
        }
        job_manager.finish_job(&record.id).await;
        return;
    }
    
    // Batching state
    let mut batch = RttBatch::new();
    let mut last_emit = Instant::now();
    let batch_interval = Duration::from_millis(config.max_batch_interval_ms);
    let poll_interval = Duration::from_millis(config.poll_interval_ms.max(5));
    
    // Main RTT loop - batch and emit
    loop {
        tokio::select! {
            // Check for incoming data
            msg = data_rx.recv() => {
                match msg {
                    Some(rtt_msg) => {
                        // Add message to batch
                        if batch.len() >= config.max_batch_lines || 
                           batch.total_bytes >= config.max_batch_bytes {
                            // Batch full, drop oldest if needed
                            total_dropped += 1;
                            batch.dropped_count += 1;
                        } else {
                            batch.push(rtt_msg);
                            total_messages += 1;
                        }
                        
                        // Emit if batch is full
                        if batch.len() >= config.max_batch_lines ||
                           batch.total_bytes >= config.max_batch_bytes {
                            emit_batch(&mut emitter, &mut batch, &emit_event).await;
                            last_emit = Instant::now();
                        }
                    }
                    None => {
                        // Channel closed (backend stopped)
                        break;
                    }
                }
            }
            
            // Periodic batch emission
            _ = tokio::time::sleep(poll_interval) => {
                // Check if we should emit based on time
                if !batch.is_empty() && last_emit.elapsed() >= batch_interval {
                    emit_batch(&mut emitter, &mut batch, &emit_event).await;
                    last_emit = Instant::now();
                }
                
                // Check cancellation
                if record.is_cancelled() {
                    break;
                }
            }
        }
    }
    
    // Emit any remaining batch
    if !batch.is_empty() {
        emit_batch(&mut emitter, &mut batch, &emit_event).await;
    }
    
    // Stop backend
    let _ = backend.stop_rtt().await;
    
    // Emit terminal event
    let duration_ms = start.elapsed().as_millis() as u64;
    
    if record.is_cancelled() {
        if let Some((event_name, payload)) = emitter.process(EmitterMessage::Terminal {
            terminal: JobTerminal::Cancelled {
                reason: CancelReason::UserRequest,
            },
        }).await {
            let mut p = payload;
            p["total_messages"] = serde_json::json!(total_messages);
            p["total_dropped"] = serde_json::json!(total_dropped);
            p["duration_ms"] = serde_json::json!(duration_ms);
            emit_event(event_name, p);
        }
    } else {
        if let Some((event_name, payload)) = emitter.process(EmitterMessage::Terminal {
            terminal: JobTerminal::Completed {
                success: true,
                exit_code: Some(0),
                duration_ms,
            },
        }).await {
            let mut p = payload;
            p["total_messages"] = serde_json::json!(total_messages);
            p["total_dropped"] = serde_json::json!(total_dropped);
            emit_event(event_name, p);
        }
    }
    
    // Finish job (releases device lock)
    job_manager.finish_job(&record.id).await;
}

/// Helper to emit a batch through the emitter
async fn emit_batch(
    emitter: &mut JobEmitter,
    batch: &mut RttBatch,
    emit_event: &impl Fn(String, serde_json::Value),
) {
    if batch.is_empty() {
        return;
    }
    
    let payload = serde_json::json!({
        "type": "message",
        "messages": batch.messages,
        "dropped_count": batch.dropped_count,
        "message_count": batch.len(),
        "total_bytes": batch.total_bytes,
    });
    
    if let Some((event_name, p)) = emitter.process(EmitterMessage::Custom {
        event_suffix: "message".to_string(),
        payload,
    }).await {
        emit_event(event_name, p);
    }
    
    batch.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    
    #[tokio::test]
    async fn test_mock_rtt_backend_starts() {
        let backend = MockRttBackend::new().with_interval(10);
        let (tx, mut rx) = mpsc::channel(64);
        
        let config = RttConfig {
            chip: "STM32F407VG".to_string(),
            ..Default::default()
        };
        
        let cancelled = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let c = cancelled.clone();
        
        backend.start_rtt(&config, tx, move || c.load(Ordering::SeqCst)).await.unwrap();
        
        // Wait for some messages
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Should have received some messages
        let mut count = 0;
        while let Ok(msg) = rx.try_recv() {
            count += 1;
            assert_eq!(msg.channel, 0);
        }
        
        assert!(count > 0, "Should have received RTT messages");
        
        cancelled.store(true, Ordering::SeqCst);
        backend.stop_rtt().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_mock_rtt_backend_fails_no_chip() {
        let backend = MockRttBackend::new();
        let (tx, _rx) = mpsc::channel(64);
        
        let config = RttConfig {
            chip: "".to_string(),
            ..Default::default()
        };
        
        let result = backend.start_rtt(&config, tx, || false).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_rtt_batch_batching() {
        let mut batch = RttBatch::new();
        
        for i in 0..10 {
            batch.push(RttMessage {
                channel: 0,
                text: format!("Message {}", i),
                timestamp_ms: i as u64,
            });
        }
        
        assert_eq!(batch.len(), 10);
        assert!(batch.total_bytes > 0);
        
        batch.clear();
        assert!(batch.is_empty());
        assert_eq!(batch.total_bytes, 0);
    }
    
    #[tokio::test]
    async fn test_device_lock_exclusivity() {
        let manager = Arc::new(JobManager::new());
        
        // Create RTT job and acquire lock
        let (record, _tx) = manager.create_job(JobKind::Rtt);
        let rtt_id = record.id.clone();
        manager.try_acquire_device(&rtt_id).await.unwrap();
        
        // Try to create flash job - should fail to acquire lock
        let (flash_record, _tx2) = manager.create_job(JobKind::Flash);
        let flash_id = flash_record.id.clone();
        let result = manager.try_acquire_device(&flash_id).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("in use"));
        
        // Release RTT lock
        manager.release_device(&rtt_id).await;
        
        // Now flash should work
        let result = manager.try_acquire_device(&flash_id).await;
        assert!(result.is_ok());
    }
}
