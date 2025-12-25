// Flash Module
//
// Provides flash operations using the unified job manager.
// All events flow through JobEmitter (strict single-emitter pattern).

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::jobs::{
    JobManager, JobKind, JobRecord, JobEmitter, EmitterMessage,
    JobTerminal, CancelReason, InternalErrorCode, PROTOCOL_VERSION,
};

// ==================== Flash-Specific Types ====================

/// Flash configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashConfig {
    pub elf_path: PathBuf,
    pub verify: bool,
    pub chip: Option<String>,
    pub speed_khz: Option<u32>,
}

/// Flash result returned on completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashResult {
    pub success: bool,
    pub bytes_written: u64,
    pub duration_ms: u64,
    pub verified: bool,
    pub chip_resolved: Option<String>,
}

/// Flash phases for progress events
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FlashPhase {
    Connecting,
    Erasing,
    Programming,
    Verifying,
    Resetting,
}

impl FlashPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            FlashPhase::Connecting => "connecting",
            FlashPhase::Erasing => "erasing",
            FlashPhase::Programming => "programming",
            FlashPhase::Verifying => "verifying",
            FlashPhase::Resetting => "resetting",
        }
    }
    
    /// Base percent for each phase
    pub fn base_percent(&self) -> f32 {
        match self {
            FlashPhase::Connecting => 0.0,
            FlashPhase::Erasing => 5.0,
            FlashPhase::Programming => 20.0,
            FlashPhase::Verifying => 90.0,
            FlashPhase::Resetting => 98.0,
        }
    }
}

/// Flash internal messages (sent to emitter)
#[derive(Debug, Clone)]
pub enum FlashMessage {
    Started {
        elf_path: String,
        chip: Option<String>,
        verify: bool,
    },
    Progress {
        phase: FlashPhase,
        percent: f32,
        done_bytes: Option<u64>,
        total_bytes: Option<u64>,
        message: Option<String>,
    },
    Completed {
        success: bool,
        bytes_written: u64,
        verified: bool,
        chip_resolved: Option<String>,
    },
    Cancelled {
        reason: CancelReason,
        terminated_by: TerminatedBy,
    },
    InternalError {
        error_code: FlashErrorCode,
        message: String,
        details: Option<String>,
        retryable: bool,
    },
}

/// How the process was terminated (physical)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminatedBy {
    Completed,
    Cancelled,
    Killed,
    InternalError,
}

/// Flash-specific error codes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FlashErrorCode {
    NoProbeFound,
    ProbeOpenFailed,
    AttachFailed,
    TargetNotFound,
    FlashFailed,
    VerifyFailed,
    ElfNotFound,
    InvalidElf,
    IoError,
    Cancelled,
}

impl FlashErrorCode {
    pub fn to_internal_code(&self) -> InternalErrorCode {
        match self {
            FlashErrorCode::NoProbeFound => InternalErrorCode::ProbeNotFound,
            FlashErrorCode::ProbeOpenFailed | FlashErrorCode::AttachFailed => {
                InternalErrorCode::ProbeConnectionFailed
            }
            FlashErrorCode::TargetNotFound => InternalErrorCode::ProbeConnectionFailed,
            FlashErrorCode::FlashFailed | FlashErrorCode::VerifyFailed => {
                InternalErrorCode::FlashFailed
            }
            FlashErrorCode::ElfNotFound | FlashErrorCode::InvalidElf | FlashErrorCode::IoError => {
                InternalErrorCode::IoError
            }
            FlashErrorCode::Cancelled => InternalErrorCode::Unknown,
        }
    }
}

/// Flash error with full details
#[derive(Debug, Clone)]
pub struct FlashError {
    pub code: FlashErrorCode,
    pub message: String,
    pub details: Option<String>,
    pub retryable: bool,
    pub os_error_code: Option<i32>,
}

impl FlashError {
    pub fn no_probe() -> Self {
        Self {
            code: FlashErrorCode::NoProbeFound,
            message: "No debug probe found. Connect ST-Link, J-Link, or CMSIS-DAP.".to_string(),
            details: None,
            retryable: true,
            os_error_code: None,
        }
    }
    
    pub fn elf_not_found(path: &PathBuf) -> Self {
        Self {
            code: FlashErrorCode::ElfNotFound,
            message: format!("ELF file not found: {}", path.display()),
            details: None,
            retryable: false,
            os_error_code: None,
        }
    }
    
    pub fn flash_failed(msg: impl Into<String>) -> Self {
        Self {
            code: FlashErrorCode::FlashFailed,
            message: msg.into(),
            details: None,
            retryable: true,
            os_error_code: None,
        }
    }
    
    pub fn cancelled() -> Self {
        Self {
            code: FlashErrorCode::Cancelled,
            message: "Flash cancelled".to_string(),
            details: None,
            retryable: true,
            os_error_code: None,
        }
    }
}

// ==================== ProbeBackend Trait ====================

/// Progress callback for flash operations
pub type ProgressCallback = mpsc::Sender<FlashMessage>;

/// Backend trait for probe operations
#[async_trait]
pub trait ProbeBackend: Send + Sync {
    /// Flash firmware to target, sending progress through callback
    async fn flash(
        &self,
        config: &FlashConfig,
        progress: ProgressCallback,
        cancel_check: impl Fn() -> bool + Send + Sync,
    ) -> Result<FlashResult, FlashError>;
    
    /// Check if a probe is connected
    async fn is_connected(&self) -> bool;
    
    /// Get info about connected probe
    async fn probe_info(&self) -> Option<ProbeInfo>;
}

/// Probe information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeInfo {
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial: Option<String>,
}

// ==================== Mock Backend ====================

/// Mock probe backend for testing without hardware
pub struct MockProbeBackend {
    connected: bool,
    phase_delay_ms: u64,
    should_fail_at: Option<FlashPhase>,
}

impl MockProbeBackend {
    pub fn new() -> Self {
        Self {
            connected: true,
            phase_delay_ms: 50,
            should_fail_at: None,
        }
    }
    
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.phase_delay_ms = delay_ms;
        self
    }
    
    pub fn disconnected(mut self) -> Self {
        self.connected = false;
        self
    }
    
    pub fn fail_at(mut self, phase: FlashPhase) -> Self {
        self.should_fail_at = Some(phase);
        self
    }
}

impl Default for MockProbeBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProbeBackend for MockProbeBackend {
    async fn flash(
        &self,
        config: &FlashConfig,
        progress: ProgressCallback,
        cancel_check: impl Fn() -> bool + Send + Sync,
    ) -> Result<FlashResult, FlashError> {
        if !self.connected {
            return Err(FlashError::no_probe());
        }
        
        if !config.elf_path.exists() {
            return Err(FlashError::elf_not_found(&config.elf_path));
        }
        
        let file_size = std::fs::metadata(&config.elf_path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        let delay = Duration::from_millis(self.phase_delay_ms);
        let chip = config.chip.clone().unwrap_or_else(|| "STM32F407VG".to_string());
        
        // Helper to check cancellation and send progress
        macro_rules! phase {
            ($phase:expr, $msg:expr) => {
                if cancel_check() {
                    return Err(FlashError::cancelled());
                }
                if self.should_fail_at == Some($phase) {
                    return Err(FlashError::flash_failed(format!("Mock failure at {:?}", $phase)));
                }
                let _ = progress.send(FlashMessage::Progress {
                    phase: $phase,
                    percent: $phase.base_percent(),
                    done_bytes: None,
                    total_bytes: Some(file_size),
                    message: Some($msg.to_string()),
                }).await;
                tokio::time::sleep(delay).await;
            };
        }
        
        // Connecting
        phase!(FlashPhase::Connecting, format!("Connecting to {}...", chip));
        
        // Erasing
        phase!(FlashPhase::Erasing, "Erasing flash sectors...");
        
        // Programming (simulate multiple progress updates)
        if self.should_fail_at == Some(FlashPhase::Programming) {
            return Err(FlashError::flash_failed("Mock failure at Programming"));
        }
        for i in 0..5 {
            if cancel_check() {
                return Err(FlashError::cancelled());
            }
            let percent = 20.0 + (i as f32 * 14.0);
            let bytes_done = (file_size as f32 * (percent - 20.0) / 70.0) as u64;
            let _ = progress.send(FlashMessage::Progress {
                phase: FlashPhase::Programming,
                percent,
                done_bytes: Some(bytes_done),
                total_bytes: Some(file_size),
                message: Some(format!("Programming: {:.0}%", percent)),
            }).await;
            tokio::time::sleep(delay / 2).await;
        }
        
        // Verifying
        if config.verify {
            phase!(FlashPhase::Verifying, "Verifying flash contents...");
        }
        
        // Resetting
        phase!(FlashPhase::Resetting, "Resetting target...");
        
        Ok(FlashResult {
            success: true,
            bytes_written: file_size,
            duration_ms: self.phase_delay_ms * 8,
            verified: config.verify,
            chip_resolved: Some(chip),
        })
    }
    
    async fn is_connected(&self) -> bool {
        self.connected
    }
    
    async fn probe_info(&self) -> Option<ProbeInfo> {
        if self.connected {
            Some(ProbeInfo {
                name: "Mock ST-Link V2.1".to_string(),
                vendor_id: 0x0483,
                product_id: 0x374B,
                serial: Some("MOCK_SERIAL".to_string()),
            })
        } else {
            None
        }
    }
}

// ==================== Flash Job Runner ====================

/// Run a flash job with full event pipeline through JobEmitter
pub async fn run_flash_job<B: ProbeBackend + 'static>(
    job_manager: Arc<JobManager>,
    backend: Arc<B>,
    config: FlashConfig,
    emit_event: impl Fn(String, serde_json::Value) + Send + Sync + Clone + 'static,
) -> Result<String, String> {
    // Validate ELF exists before creating job
    if !config.elf_path.exists() {
        return Err(format!("ELF file not found: {}", config.elf_path.display()));
    }
    
    // Create job
    let (record, _tx) = job_manager.create_job(JobKind::Flash);
    let job_id = record.id.clone();
    
    // Try to acquire device lock
    if let Err(msg) = job_manager.try_acquire_device(&job_id).await {
        // Emit internal error and clean up
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
    
    // Spawn flash worker task
    let job_id_clone = job_id.clone();
    let record_clone = record.clone();
    let job_manager_clone = job_manager.clone();
    let emit_clone = emit_event.clone();
    
    tokio::spawn(async move {
        run_flash_worker(
            record_clone,
            backend,
            config,
            job_manager_clone,
            emit_clone,
        ).await;
    });
    
    Ok(job_id)
}

/// Flash worker task - sends all events through JobEmitter
async fn run_flash_worker<B: ProbeBackend + 'static>(
    record: Arc<JobRecord>,
    backend: Arc<B>,
    config: FlashConfig,
    job_manager: Arc<JobManager>,
    emit_event: impl Fn(String, serde_json::Value) + Send + Sync,
) {
    let mut emitter = JobEmitter::new(&record);
    let start = std::time::Instant::now();
    
    // Create channel for progress messages
    let (progress_tx, mut progress_rx) = mpsc::channel::<FlashMessage>(64);
    
    // Emit started event
    let started_payload = serde_json::json!({
        "type": "started",
        "elf_path": config.elf_path.display().to_string(),
        "chip": config.chip,
        "verify": config.verify,
    });
    if let Some((event_name, payload)) = emitter.process(EmitterMessage::Custom {
        event_suffix: "started".to_string(),
        payload: started_payload,
    }).await {
        emit_event(event_name, payload);
    }
    
    // Spawn backend flash operation
    let cancel_token = record.cancel_token.clone();
    let flash_config = config.clone();
    let flash_handle = tokio::spawn(async move {
        backend.flash(
            &flash_config,
            progress_tx,
            move || cancel_token.is_cancelled(),
        ).await
    });
    
    // Forward progress events through emitter
    loop {
        tokio::select! {
            msg = progress_rx.recv() => {
                match msg {
                    Some(FlashMessage::Progress { phase, percent, done_bytes, total_bytes, message }) => {
                        if let Some((event_name, payload)) = emitter.process(EmitterMessage::Progress {
                            phase: phase.as_str().to_string(),
                            percent,
                            message,
                        }).await {
                            emit_event(event_name, payload);
                        }
                    }
                    Some(_) => {
                        // Other messages handled at end
                    }
                    None => break, // Channel closed
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {
                // Check if flash completed
                if flash_handle.is_finished() {
                    break;
                }
            }
        }
    }
    
    // Get flash result
    let duration_ms = start.elapsed().as_millis() as u64;
    
    match flash_handle.await {
        Ok(Ok(result)) => {
            // Success
            if let Some((event_name, payload)) = emitter.process(EmitterMessage::Terminal {
                terminal: JobTerminal::Completed {
                    success: true,
                    exit_code: Some(0),
                    duration_ms,
                },
            }).await {
                // Add flash-specific fields
                let mut p = payload;
                p["bytes_written"] = serde_json::json!(result.bytes_written);
                p["verified"] = serde_json::json!(result.verified);
                p["chip_resolved"] = serde_json::json!(result.chip_resolved);
                emit_event(event_name, p);
            }
        }
        Ok(Err(err)) => {
            if matches!(err.code, FlashErrorCode::Cancelled) {
                // Cancelled
                if let Some((event_name, payload)) = emitter.process(EmitterMessage::Terminal {
                    terminal: JobTerminal::Cancelled {
                        reason: CancelReason::UserRequest,
                    },
                }).await {
                    emit_event(event_name, payload);
                }
            } else {
                // Internal error
                if let Some((event_name, payload)) = emitter.process(EmitterMessage::Terminal {
                    terminal: JobTerminal::InternalError {
                        error_code: err.code.to_internal_code(),
                        message: err.message.clone(),
                        retryable: err.retryable,
                    },
                }).await {
                    let mut p = payload;
                    if let Some(details) = &err.details {
                        p["details"] = serde_json::json!(details);
                    }
                    if let Some(os_code) = err.os_error_code {
                        p["os_error_code"] = serde_json::json!(os_code);
                    }
                    emit_event(event_name, p);
                }
            }
        }
        Err(join_err) => {
            // Task panicked
            if let Some((event_name, payload)) = emitter.process(EmitterMessage::Terminal {
                terminal: JobTerminal::InternalError {
                    error_code: InternalErrorCode::Unknown,
                    message: format!("Flash task panicked: {}", join_err),
                    retryable: true,
                },
            }).await {
                emit_event(event_name, payload);
            }
        }
    }
    
    // Finish job (releases device lock)
    job_manager.finish_job(&record.id).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    #[tokio::test]
    async fn test_mock_backend_flash_success() {
        let backend = MockProbeBackend::new().with_delay(5);
        
        let elf = NamedTempFile::new().unwrap();
        std::fs::write(elf.path(), vec![0u8; 1024]).unwrap();
        
        let config = FlashConfig {
            elf_path: elf.path().to_path_buf(),
            verify: true,
            chip: Some("STM32F407VG".to_string()),
            speed_khz: Some(4000),
        };
        
        let (tx, mut rx) = mpsc::channel(64);
        let result = backend.flash(&config, tx, || false).await;
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.verified);
        
        // Verify we got progress events
        let mut progress_count = 0;
        while let Ok(msg) = rx.try_recv() {
            if matches!(msg, FlashMessage::Progress { .. }) {
                progress_count += 1;
            }
        }
        assert!(progress_count > 0);
    }
    
    #[tokio::test]
    async fn test_mock_backend_cancellation() {
        let backend = MockProbeBackend::new().with_delay(100);
        
        let elf = NamedTempFile::new().unwrap();
        std::fs::write(elf.path(), vec![0u8; 1024]).unwrap();
        
        let config = FlashConfig {
            elf_path: elf.path().to_path_buf(),
            verify: true,
            chip: None,
            speed_khz: None,
        };
        
        let (tx, _rx) = mpsc::channel(64);
        let cancelled = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let c = cancelled.clone();
        
        let result = backend.flash(&config, tx, move || c.load(std::sync::atomic::Ordering::SeqCst)).await;
        
        assert!(matches!(result, Err(ref e) if matches!(e.code, FlashErrorCode::Cancelled)));
    }
    
    #[tokio::test]
    async fn test_mock_backend_no_probe() {
        let backend = MockProbeBackend::new().disconnected();
        
        let elf = NamedTempFile::new().unwrap();
        std::fs::write(elf.path(), vec![0u8; 1024]).unwrap();
        
        let config = FlashConfig {
            elf_path: elf.path().to_path_buf(),
            verify: false,
            chip: None,
            speed_khz: None,
        };
        
        let (tx, _rx) = mpsc::channel(64);
        let result = backend.flash(&config, tx, || false).await;
        
        assert!(matches!(result, Err(ref e) if matches!(e.code, FlashErrorCode::NoProbeFound)));
    }
    
    #[tokio::test]
    async fn test_mock_backend_fail_at_phase() {
        let backend = MockProbeBackend::new()
            .with_delay(5)
            .fail_at(FlashPhase::Programming);
        
        let elf = NamedTempFile::new().unwrap();
        std::fs::write(elf.path(), vec![0u8; 1024]).unwrap();
        
        let config = FlashConfig {
            elf_path: elf.path().to_path_buf(),
            verify: true,
            chip: None,
            speed_khz: None,
        };
        
        let (tx, _rx) = mpsc::channel(64);
        let result = backend.flash(&config, tx, || false).await;
        
        assert!(matches!(result, Err(ref e) if matches!(e.code, FlashErrorCode::FlashFailed)));
    }
}
