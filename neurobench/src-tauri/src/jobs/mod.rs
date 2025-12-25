// Unified Job Manager
//
// Provides common infrastructure for all long-running operations:
// - Build, Flash, RTT, Agent jobs
// - Single-emitter pattern (one task stamps seq)
// - Ring buffer logging with bytes cap
// - Cancellation with terminal event guarantee
// - Exclusive device lock for hardware operations

pub mod flash;
pub mod rtt;
#[cfg(feature = "hardware")]
pub mod probe_rs_backend;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use dashmap::DashMap;

/// Current protocol version for all job events
pub const PROTOCOL_VERSION: u32 = 1;

/// Maximum ring buffer size (lines)
pub const DEFAULT_LOG_LINES: usize = 5000;
/// Maximum ring buffer size (bytes)
pub const DEFAULT_LOG_BYTES: usize = 5 * 1024 * 1024; // 5MB

/// Unique job identifier
pub type JobId = String;

// ==================== Core Types ====================

/// Kind of job - determines event namespace and device locking
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    Build,
    Flash,
    Rtt,
    Agent,
    Index,
}

impl JobKind {
    /// Returns true if this job requires exclusive device access
    pub fn requires_device(&self) -> bool {
        matches!(self, JobKind::Flash | JobKind::Rtt)
    }
    
    /// Event namespace prefix
    pub fn event_prefix(&self) -> &'static str {
        match self {
            JobKind::Build => "build",
            JobKind::Flash => "flash",
            JobKind::Rtt => "rtt",
            JobKind::Agent => "agent",
            JobKind::Index => "index",
        }
    }
}

/// Common header for all job events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobEventHeader {
    pub protocol_version: u32,
    pub job_id: JobId,
    pub seq: u64,
    pub timestamp_ms: u64,
}

/// Job terminal state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum JobTerminal {
    Completed {
        success: bool,
        exit_code: Option<i32>,
        duration_ms: u64,
    },
    Cancelled {
        reason: CancelReason,
    },
    InternalError {
        error_code: InternalErrorCode,
        message: String,
        retryable: bool,
    },
}

/// Reason for cancellation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CancelReason {
    UserRequest,
    Superseded,
    Shutdown,
    Timeout,
}

/// Machine-readable internal error codes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InternalErrorCode {
    SpawnFailed,
    WorkdirMissing,
    PermissionDenied,
    ToolchainNotFound,
    ProbeNotFound,
    ProbeConnectionFailed,
    FlashFailed,
    RttStartFailed,
    IoError,
    Unknown,
}

/// Current job status (O(1) lookup, not computed from logs)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JobStatus {
    pub phase: Option<String>,
    pub percent: Option<f32>,
    pub message: Option<String>,
    pub terminal: Option<JobTerminal>,
}

/// Device status for UI status strip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatus {
    pub device_locked: bool,
    pub lock_holder_id: Option<String>,
    pub rtt_active: bool,
    pub active_rtt_id: Option<String>,
    pub active_flash_id: Option<String>,
    pub active_jobs_count: usize,
}

// ==================== Ring Buffer ====================

/// Ring buffer with both line and byte caps
#[derive(Debug, Clone)]
pub struct RingBuffer {
    lines: Vec<String>,
    max_lines: usize,
    max_bytes: usize,
    current_bytes: usize,
}

impl RingBuffer {
    pub fn new(max_lines: usize, max_bytes: usize) -> Self {
        Self {
            lines: Vec::with_capacity(max_lines.min(1000)),
            max_lines,
            max_bytes,
            current_bytes: 0,
        }
    }
    
    pub fn push(&mut self, line: String) {
        let line_bytes = line.len();
        
        // Evict oldest lines until under byte budget
        while self.current_bytes + line_bytes > self.max_bytes && !self.lines.is_empty() {
            if let Some(removed) = self.lines.first() {
                self.current_bytes = self.current_bytes.saturating_sub(removed.len());
            }
            self.lines.remove(0);
        }
        
        // Evict if over line limit
        while self.lines.len() >= self.max_lines {
            if let Some(removed) = self.lines.first() {
                self.current_bytes = self.current_bytes.saturating_sub(removed.len());
            }
            self.lines.remove(0);
        }
        
        self.current_bytes += line_bytes;
        self.lines.push(line);
    }
    
    pub fn get_lines(&self, last_n: Option<usize>) -> Vec<String> {
        match last_n {
            Some(n) => self.lines.iter().rev().take(n).rev().cloned().collect(),
            None => self.lines.clone(),
        }
    }
    
    pub fn len(&self) -> usize {
        self.lines.len()
    }
    
    pub fn bytes(&self) -> usize {
        self.current_bytes
    }
}

// ==================== Job Record ====================

/// Record for a single job
pub struct JobRecord {
    pub id: JobId,
    pub kind: JobKind,
    pub started_at: Instant,
    pub cancel_token: CancellationToken,
    pub terminal_sent: Arc<AtomicBool>,
    pub log: Arc<Mutex<RingBuffer>>,
    pub status: Arc<RwLock<JobStatus>>,
}

impl JobRecord {
    pub fn new(id: JobId, kind: JobKind) -> Self {
        Self {
            id,
            kind,
            started_at: Instant::now(),
            cancel_token: CancellationToken::new(),
            terminal_sent: Arc::new(AtomicBool::new(false)),
            log: Arc::new(Mutex::new(RingBuffer::new(DEFAULT_LOG_LINES, DEFAULT_LOG_BYTES))),
            status: Arc::new(RwLock::new(JobStatus::default())),
        }
    }
    
    pub fn elapsed_ms(&self) -> u64 {
        self.started_at.elapsed().as_millis() as u64
    }
    
    /// Mark terminal event as sent (returns false if already sent)
    pub fn mark_terminal(&self) -> bool {
        !self.terminal_sent.swap(true, Ordering::SeqCst)
    }
    
    /// Check if job is cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }
    
    /// Request cancellation
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }
}

// ==================== Job Emitter ====================

/// Message sent to the single emitter task
pub enum EmitterMessage {
    Log { line: String },
    Progress { phase: String, percent: f32, message: Option<String> },
    Terminal { terminal: JobTerminal },
    Custom { event_suffix: String, payload: serde_json::Value },
}

/// Single-emitter task state
pub struct JobEmitter {
    pub job_id: JobId,
    pub kind: JobKind,
    seq: u64,
    started_at: Instant,
    log: Arc<Mutex<RingBuffer>>,
    status: Arc<RwLock<JobStatus>>,
    terminal_sent: Arc<AtomicBool>,
}

impl JobEmitter {
    pub fn new(record: &JobRecord) -> Self {
        Self {
            job_id: record.id.clone(),
            kind: record.kind,
            seq: 0,
            started_at: record.started_at,
            log: record.log.clone(),
            status: record.status.clone(),
            terminal_sent: record.terminal_sent.clone(),
        }
    }
    
    /// Create header with next sequence number
    pub fn next_header(&mut self) -> JobEventHeader {
        let header = JobEventHeader {
            protocol_version: PROTOCOL_VERSION,
            job_id: self.job_id.clone(),
            seq: self.seq,
            timestamp_ms: self.started_at.elapsed().as_millis() as u64,
        };
        self.seq += 1;
        header
    }
    
    /// Process an emitter message, returns event name and payload
    pub async fn process(&mut self, msg: EmitterMessage) -> Option<(String, serde_json::Value)> {
        let prefix = self.kind.event_prefix();
        
        match msg {
            EmitterMessage::Log { line } => {
                self.log.lock().await.push(line.clone());
                let header = self.next_header();
                let event = serde_json::json!({
                    "type": "output",
                    "header": header,
                    "line": line,
                });
                Some((format!("{}:output", prefix), event))
            }
            EmitterMessage::Progress { phase, percent, message } => {
                {
                    let mut status = self.status.write().await;
                    status.phase = Some(phase.clone());
                    status.percent = Some(percent);
                    status.message = message.clone();
                }
                let header = self.next_header();
                let event = serde_json::json!({
                    "type": "progress",
                    "header": header,
                    "phase": phase,
                    "percent": percent,
                    "message": message,
                });
                Some((format!("{}:progress", prefix), event))
            }
            EmitterMessage::Terminal { terminal } => {
                // Only emit if terminal not already sent
                if !self.terminal_sent.swap(true, Ordering::SeqCst) {
                    {
                        let mut status = self.status.write().await;
                        status.terminal = Some(terminal.clone());
                    }
                    let header = self.next_header();
                    let event_type = match &terminal {
                        JobTerminal::Completed { .. } => "completed",
                        JobTerminal::Cancelled { .. } => "cancelled",
                        JobTerminal::InternalError { .. } => "internal_error",
                    };
                    let event = serde_json::json!({
                        "type": event_type,
                        "header": header,
                        "terminal": terminal,
                    });
                    Some((format!("{}:{}", prefix, event_type), event))
                } else {
                    None
                }
            }
            EmitterMessage::Custom { event_suffix, payload } => {
                let header = self.next_header();
                let mut event = payload;
                event["header"] = serde_json::to_value(header).unwrap_or_default();
                Some((format!("{}:{}", prefix, event_suffix), event))
            }
        }
    }
}

// ==================== Job Manager ====================

/// Information about a job for IPC responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub id: JobId,
    pub kind: JobKind,
    pub started_at_ms: u64,
    pub status: JobStatus,
}

/// Unified job manager for all long-running operations
pub struct JobManager {
    jobs: DashMap<JobId, Arc<JobRecord>>,
    completed_logs: Arc<RwLock<HashMap<JobId, RingBuffer>>>,
    device_lock: Arc<Mutex<Option<JobId>>>,  // Exclusive device access
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            jobs: DashMap::new(),
            completed_logs: Arc::new(RwLock::new(HashMap::new())),
            device_lock: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Try to acquire device lock for a job (Flash/RTT)
    pub async fn try_acquire_device(&self, job_id: &str) -> Result<(), String> {
        let mut lock = self.device_lock.lock().await;
        if lock.is_some() {
            Err("Device is in use by another job. Stop RTT or wait for flash to complete.".to_string())
        } else {
            *lock = Some(job_id.to_string());
            Ok(())
        }
    }
    
    /// Release device lock
    pub async fn release_device(&self, job_id: &str) {
        let mut lock = self.device_lock.lock().await;
        if lock.as_deref() == Some(job_id) {
            *lock = None;
        }
    }
    
    /// Get current device status for UI status strip
    pub async fn get_device_status(&self) -> DeviceStatus {
        let lock = self.device_lock.lock().await;
        let lock_holder = lock.clone();
        drop(lock);
        
        // Find active RTT job
        let mut rtt_active = false;
        let mut active_rtt_id = None;
        let mut active_flash_id = None;
        
        for entry in self.jobs.iter() {
            let record = entry.value();
            if !record.is_cancelled() {
                match record.kind {
                    JobKind::Rtt => {
                        rtt_active = true;
                        active_rtt_id = Some(record.id.clone());
                    }
                    JobKind::Flash => {
                        active_flash_id = Some(record.id.clone());
                    }
                    _ => {}
                }
            }
        }
        
        DeviceStatus {
            device_locked: lock_holder.is_some(),
            lock_holder_id: lock_holder,
            rtt_active,
            active_rtt_id,
            active_flash_id,
            active_jobs_count: self.jobs.len(),
        }
    }
    
    /// Create a new job and return (record, sender for emitter)
    pub fn create_job(&self, kind: JobKind) -> (Arc<JobRecord>, mpsc::Sender<EmitterMessage>) {
        let id = format!("{}_{}", kind.event_prefix(), uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("x"));
        let record = Arc::new(JobRecord::new(id.clone(), kind));
        self.jobs.insert(id, record.clone());
        
        let (tx, _rx) = mpsc::channel(256);
        (record, tx)
    }
    
    /// Get a job by ID
    pub fn get_job(&self, job_id: &str) -> Option<Arc<JobRecord>> {
        self.jobs.get(job_id).map(|r| r.clone())
    }
    
    /// List jobs, optionally filtered by kind
    pub async fn list_jobs(&self, kind: Option<JobKind>) -> Vec<JobInfo> {
        let mut result = Vec::new();
        for entry in self.jobs.iter() {
            let record = entry.value();
            if kind.is_none() || kind == Some(record.kind) {
                let status = record.status.read().await.clone();
                result.push(JobInfo {
                    id: record.id.clone(),
                    kind: record.kind,
                    started_at_ms: record.elapsed_ms(),
                    status,
                });
            }
        }
        result
    }
    
    /// Cancel a job
    pub fn cancel_job(&self, job_id: &str) -> bool {
        if let Some(record) = self.jobs.get(job_id) {
            record.cancel();
            true
        } else {
            false
        }
    }
    
    /// Get job status
    pub async fn get_status(&self, job_id: &str) -> Option<JobStatus> {
        if let Some(record) = self.jobs.get(job_id) {
            Some(record.status.read().await.clone())
        } else {
            None
        }
    }
    
    /// Get job log
    pub async fn get_log(&self, job_id: &str, last_n: Option<usize>) -> Option<Vec<String>> {
        // Check active jobs first
        if let Some(record) = self.jobs.get(job_id) {
            let log = record.log.lock().await;
            return Some(log.get_lines(last_n));
        }
        
        // Check completed logs
        let completed = self.completed_logs.read().await;
        completed.get(job_id).map(|log| log.get_lines(last_n))
    }
    
    /// Move job to completed and release resources
    pub async fn finish_job(&self, job_id: &str) {
        if let Some((_, record)) = self.jobs.remove(job_id) {
            // Release device lock if held
            if record.kind.requires_device() {
                self.release_device(job_id).await;
            }
            
            // Move log to completed
            let log = record.log.lock().await;
            self.completed_logs.write().await.insert(
                job_id.to_string(),
                RingBuffer {
                    lines: log.lines.clone(),
                    max_lines: log.max_lines,
                    max_bytes: log.max_bytes,
                    current_bytes: log.current_bytes,
                },
            );
        }
        
        // Run GC periodically
        self.job_gc(20).await; // Keep last 20 completed jobs per kind
    }
    
    /// Garbage collect old completed job logs
    /// Keeps `max_per_kind` most recent logs for each job kind
    pub async fn job_gc(&self, max_per_kind: usize) {
        let mut completed = self.completed_logs.write().await;
        
        if completed.len() <= max_per_kind * 5 {
            // Under limit, no GC needed (5 job kinds)
            return;
        }
        
        // Group by kind prefix and collect with timestamps (from job ID)
        let mut by_kind: HashMap<&str, Vec<&String>> = HashMap::new();
        for job_id in completed.keys() {
            let kind = job_id.split('_').next().unwrap_or("unknown");
            by_kind.entry(kind).or_default().push(job_id);
        }
        
        // Remove oldest jobs for each kind that exceed limit
        let mut to_remove = Vec::new();
        for (_kind, mut ids) in by_kind {
            if ids.len() > max_per_kind {
                // Sort by ID (which contains timestamp-ish UUID portion)
                ids.sort();
                // Remove oldest (first N - max_per_kind)
                let remove_count = ids.len() - max_per_kind;
                for id in ids.into_iter().take(remove_count) {
                    to_remove.push(id.clone());
                }
            }
        }
        
        for id in to_remove {
            completed.remove(&id);
        }
    }
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ring_buffer_line_cap() {
        let mut buf = RingBuffer::new(3, 1_000_000);
        buf.push("line1".to_string());
        buf.push("line2".to_string());
        buf.push("line3".to_string());
        buf.push("line4".to_string());
        
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.get_lines(None), vec!["line2", "line3", "line4"]);
    }
    
    #[test]
    fn test_ring_buffer_byte_cap() {
        let mut buf = RingBuffer::new(1000, 20);
        buf.push("12345".to_string()); // 5 bytes
        buf.push("67890".to_string()); // 5 bytes, total 10
        buf.push("abcde".to_string()); // 5 bytes, total 15
        buf.push("fghij".to_string()); // 5 bytes, total 20
        buf.push("klmno".to_string()); // evicts until under 20
        
        assert!(buf.bytes() <= 20);
    }
    
    #[tokio::test]
    async fn test_terminal_sent_once() {
        let record = JobRecord::new("test".to_string(), JobKind::Build);
        
        assert!(record.mark_terminal()); // First call succeeds
        assert!(!record.mark_terminal()); // Second call fails
    }
    
    #[tokio::test]
    async fn test_job_manager_create_and_cancel() {
        let manager = JobManager::new();
        let (record, _tx) = manager.create_job(JobKind::Build);
        let job_id = record.id.clone();
        
        assert!(manager.get_job(&job_id).is_some());
        assert!(manager.cancel_job(&job_id));
        assert!(manager.get_job(&job_id).unwrap().is_cancelled());
    }
    
    #[tokio::test]
    async fn test_device_lock_exclusive() {
        let manager = JobManager::new();
        
        // First lock succeeds
        assert!(manager.try_acquire_device(&"flash_1".to_string()).await.is_ok());
        
        // Second lock fails
        assert!(manager.try_acquire_device(&"rtt_1".to_string()).await.is_err());
        
        // Release and retry
        manager.release_device(&"flash_1".to_string()).await;
        assert!(manager.try_acquire_device(&"rtt_1".to_string()).await.is_ok());
    }
}
