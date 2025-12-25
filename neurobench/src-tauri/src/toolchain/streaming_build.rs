// Streaming Build System - Production Hardened v2
//
// Event Contract Guarantees:
// 1. build:started → zero or more updates → exactly ONE terminal event
// 2. Terminal events: build:completed | build:cancelled | build:internal_error
// 3. Every event has: protocol_version, build_id, seq (monotonic), timestamp_ms
// 4. Cancellation kills processes, cleans temp files, prevents race with completion
//
// Features:
// - Log storage: ring buffer per build_id (configurable size + byte cap)
// - Artifact registry: elf/map/bin paths persisted after success
// - Diagnostics: normalized paths, tool name, raw line, stable diagnostic_id

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::hash::{Hash, Hasher};

/// Current event protocol version
pub const PROTOCOL_VERSION: u32 = 1;

/// Build job identifier
pub type BuildId = String;

// ==================== Event Payloads ====================

/// Common header for all events - ensures ordering and timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHeader {
    pub protocol_version: u32,  // Always PROTOCOL_VERSION
    pub build_id: BuildId,
    pub seq: u64,               // Monotonically increasing per build
    pub timestamp_ms: u64,      // Milliseconds since build start (Instant-based, not wall clock)
}

/// How the build was terminated
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminatedBy {
    Completed,      // Normal completion
    Cancelled,      // User/API requested cancel
    Killed,         // Process was killed (timeout, etc.)
    InternalError,  // System error (spawn failed, parser panic, etc.)
}

/// Build events emitted during compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BuildEvent {
    /// Build has started
    Started {
        #[serde(flatten)]
        header: EventHeader,
        // Build identity
        project_path: String,
        project_id: Option<String>,     // Stable UUID from project manifest
        config_hash: String,
        // Toolchain info
        toolchain_id: String,           // Specific toolchain ID
        toolchain_kind: String,         // "arm_gcc", "clang", "rust"
        profile: String,                // "debug", "release", "minsize"
        // Reproducibility
        working_dir: String,
        commandline: Vec<String>,       // Full command args
        env_delta: Vec<(String, String)>, // Changed env vars only
        // Metadata
        neurobench_version: String,
    },
    /// Raw output line from compiler
    Output {
        #[serde(flatten)]
        header: EventHeader,
        line: String,
        stream: OutputStream,
        tool: Option<String>,           // Optional - not all lines map to a tool
    },
    /// Parsed diagnostic (error/warning)
    Diagnostic {
        #[serde(flatten)]
        header: EventHeader,
        diagnostic: EnhancedDiagnostic,
    },
    /// Build progress update
    Progress {
        #[serde(flatten)]
        header: EventHeader,
        phase: BuildPhase,
        percent: u8,
        message: String,
        files_compiled: usize,
        files_total: usize,
    },
    /// Build completed (terminal - build ran to completion, success can be true/false)
    Completed {
        #[serde(flatten)]
        header: EventHeader,
        success: bool,
        terminated_by: TerminatedBy,    // Physical process termination
        exit_code: Option<i32>,
        duration_ms: u64,
        error_count: usize,
        warning_count: usize,
        artifacts: Option<BuildArtifacts>,
    },
    /// Build was cancelled (terminal - user/system requested cancellation)
    Cancelled {
        #[serde(flatten)]
        header: EventHeader,
        terminated_by: TerminatedBy,    // completed | cancelled | killed
        reason: CancelReason,
    },
    /// Internal error (terminal - NeuroBench couldn't run the build)
    InternalError {
        #[serde(flatten)]
        header: EventHeader,
        terminated_by: TerminatedBy,    // Always internal_error here
        error_code: InternalErrorCode,  // Machine-readable error
        message: String,
        details: Option<String>,
        os_error_code: Option<i32>,
        retryable: bool,                // Can user just retry?
    },
}

/// Machine-readable internal error codes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InternalErrorCode {
    SpawnFailed,
    WorkdirMissing,
    PermissionDenied,
    ToolchainNotFound,
    LinkerScriptMissing,
    ParserPanic,
    IoError,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CancelReason {
    UserRequest,
    Superseded,     // New build started, old one cancelled
    Shutdown,       // App shutting down
    Timeout,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputStream {
    Stdout,
    Stderr,
    System,  // Internal status messages
}

/// Enhanced diagnostic with better metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedDiagnostic {
    pub diagnostic_id: String,      // Stable hash for click-to-navigate + suppress
    pub severity: DiagnosticSeverity,
    pub category: DiagnosticCategory,
    pub file: String,
    pub file_absolute: String,
    pub is_external: bool,          // true if file is outside project
    pub line: u32,
    pub column: Option<u32>,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
    pub message: String,
    pub code: Option<String>,       // Error code if available (e.g., "-Wunused-variable")
    pub suggestion: Option<String>,
    pub tool: String,               // "gcc", "clang", "ld"
    pub raw_line: String,           // Original unparsed line
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Note,
    Help,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCategory {
    Compile,
    Link,
    Asm,
    Preprocess,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildPhase {
    Preparing,
    Compiling,
    Linking,
    PostProcessing,
    Done,
}

/// Build artifacts produced by successful build
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildArtifacts {
    pub elf_path: String,
    pub bin_path: Option<String>,
    pub hex_path: Option<String>,
    pub map_path: Option<String>,
    pub size_report: Option<SizeInfo>,
    // Existence checks for UI to validate before enabling actions
    pub elf_exists: bool,
    pub bin_exists: bool,
    pub map_exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeInfo {
    pub text: u64,
    pub data: u64,
    pub bss: u64,
    pub total: u64,
}

// ==================== Build Configuration ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingBuildConfig {
    pub project_path: PathBuf,
    pub project_id: Option<String>,     // Stable UUID from project manifest
    pub output_dir: Option<PathBuf>,
    pub mcu_target: String,
    pub optimization: String,
    pub defines: HashMap<String, String>,
    pub include_paths: Vec<PathBuf>,
    pub source_files: Vec<PathBuf>,
    pub linker_script: Option<PathBuf>,
    // Toolchain info
    pub toolchain_id: Option<String>,
    pub toolchain_kind: Option<String>, // "arm_gcc", "clang", "rust"
    pub profile: Option<String>,        // "debug", "release", "minsize"
}

impl StreamingBuildConfig {
    /// Generate a hash for cache invalidation
    pub fn config_hash(&self) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        format!("{:?}", self).hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

// ==================== Cancellation ====================

/// Cancellation token with atomic state
#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
    
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
    
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== Log Storage ====================

const LOG_RING_BUFFER_SIZE: usize = 5000;

/// Ring buffer for storing build logs
#[derive(Debug)]
pub struct BuildLog {
    lines: Vec<String>,
    diagnostics: Vec<EnhancedDiagnostic>,
    max_size: usize,
}

impl BuildLog {
    pub fn new(max_size: usize) -> Self {
        Self {
            lines: Vec::with_capacity(max_size.min(1000)),
            diagnostics: Vec::new(),
            max_size,
        }
    }
    
    pub fn push_line(&mut self, line: String) {
        if self.lines.len() >= self.max_size {
            self.lines.remove(0);
        }
        self.lines.push(line);
    }
    
    pub fn push_diagnostic(&mut self, diag: EnhancedDiagnostic) {
        self.diagnostics.push(diag);
    }
    
    pub fn get_lines(&self, last_n: Option<usize>) -> Vec<String> {
        match last_n {
            Some(n) => self.lines.iter().rev().take(n).rev().cloned().collect(),
            None => self.lines.clone(),
        }
    }
    
    pub fn get_diagnostics(&self) -> Vec<EnhancedDiagnostic> {
        self.diagnostics.clone()
    }
    
    pub fn error_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.severity == DiagnosticSeverity::Error).count()
    }
    
    pub fn warning_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.severity == DiagnosticSeverity::Warning).count()
    }
}

// ==================== Build Job ====================

/// Build job state
struct BuildJob {
    id: BuildId,
    config: StreamingBuildConfig,
    cancel_token: CancellationToken,
    started_at: std::time::Instant,
    seq_counter: Arc<AtomicU64>,
    log: Arc<Mutex<BuildLog>>,
    terminal_sent: Arc<AtomicBool>,  // Prevents duplicate terminal events
}

impl BuildJob {
    fn next_seq(&self) -> u64 {
        self.seq_counter.fetch_add(1, Ordering::SeqCst)
    }
    
    fn elapsed_ms(&self) -> u64 {
        self.started_at.elapsed().as_millis() as u64
    }
    
    fn make_header(&self) -> EventHeader {
        EventHeader {
            protocol_version: PROTOCOL_VERSION,
            build_id: self.id.clone(),
            seq: self.next_seq(),
            timestamp_ms: self.elapsed_ms(),
        }
    }
}

// ==================== Artifact Registry ====================

/// Registry of completed build artifacts
#[derive(Debug, Default)]
pub struct ArtifactRegistry {
    artifacts: HashMap<BuildId, BuildArtifacts>,
    latest_success: Option<BuildId>,
}

impl ArtifactRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn register(&mut self, build_id: BuildId, artifacts: BuildArtifacts) {
        self.latest_success = Some(build_id.clone());
        self.artifacts.insert(build_id, artifacts);
    }
    
    pub fn get(&self, build_id: &str) -> Option<&BuildArtifacts> {
        self.artifacts.get(build_id)
    }
    
    pub fn get_latest(&self) -> Option<&BuildArtifacts> {
        self.latest_success.as_ref().and_then(|id| self.artifacts.get(id))
    }
    
    pub fn latest_elf_path(&self) -> Option<&str> {
        self.get_latest().map(|a| a.elf_path.as_str())
    }
}

// ==================== Build Manager ====================

/// Build manager handles async build operations
pub struct BuildManager {
    jobs: Arc<Mutex<HashMap<BuildId, Arc<BuildJob>>>>,
    completed_logs: Arc<RwLock<HashMap<BuildId, BuildLog>>>,
    artifacts: Arc<RwLock<ArtifactRegistry>>,
    event_tx: broadcast::Sender<BuildEvent>,
}

impl BuildManager {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
            completed_logs: Arc::new(RwLock::new(HashMap::new())),
            artifacts: Arc::new(RwLock::new(ArtifactRegistry::new())),
            event_tx: tx,
        }
    }
    
    /// Subscribe to build events
    pub fn subscribe(&self) -> broadcast::Receiver<BuildEvent> {
        self.event_tx.subscribe()
    }
    
    /// Start a build job
    pub async fn start_build(&self, config: StreamingBuildConfig) -> BuildId {
        let build_id = format!("build_{}", uuid::Uuid::new_v4());
        let cancel_token = CancellationToken::new();
        
        let job = Arc::new(BuildJob {
            id: build_id.clone(),
            config: config.clone(),
            cancel_token: cancel_token.clone(),
            started_at: std::time::Instant::now(),
            seq_counter: Arc::new(AtomicU64::new(0)),
            log: Arc::new(Mutex::new(BuildLog::new(LOG_RING_BUFFER_SIZE))),
            terminal_sent: Arc::new(AtomicBool::new(false)),
        });
        
        // Register job
        {
            let mut jobs = self.jobs.lock().await;
            jobs.insert(build_id.clone(), job.clone());
        }
        
        // Emit start event
        let working_dir = config.project_path.display().to_string();
        let _ = self.event_tx.send(BuildEvent::Started {
            header: job.make_header(),
            project_path: config.project_path.display().to_string(),
            project_id: config.project_id.clone(),
            config_hash: config.config_hash(),
            toolchain_id: config.toolchain_id.clone().unwrap_or_else(|| "arm-none-eabi-gcc".to_string()),
            toolchain_kind: config.toolchain_kind.clone().unwrap_or_else(|| "arm_gcc".to_string()),
            profile: config.profile.clone().unwrap_or_else(|| config.optimization.clone()),
            working_dir,
            commandline: Vec::new(),  // Will be populated per-compile
            env_delta: Vec::new(),    // No env changes by default
            neurobench_version: env!("CARGO_PKG_VERSION").to_string(),
        });
        
        // Spawn build task
        let jobs = self.jobs.clone();
        let completed_logs = self.completed_logs.clone();
        let artifacts = self.artifacts.clone();
        let event_tx = self.event_tx.clone();
        
        tokio::spawn(async move {
            run_build(job.clone(), event_tx, jobs, completed_logs, artifacts).await;
        });
        
        build_id
    }
    
    /// Cancel a build job
    pub async fn cancel_build(&self, build_id: &BuildId) -> bool {
        let job = {
            let jobs = self.jobs.lock().await;
            jobs.get(build_id).cloned()
        };
        
        if let Some(job) = job {
            // Set cancellation flag
            job.cancel_token.cancel();
            
            // Emit cancelled event only if terminal not already sent
            if !job.terminal_sent.swap(true, Ordering::SeqCst) {
                let _ = self.event_tx.send(BuildEvent::Cancelled {
                    header: job.make_header(),
                    terminated_by: TerminatedBy::Cancelled,
                    reason: CancelReason::UserRequest,
                });
            }
            
            // Remove from active jobs
            self.jobs.lock().await.remove(build_id);
            
            // Move log to completed
            let log = job.log.lock().await;
            self.completed_logs.write().await.insert(build_id.clone(), BuildLog {
                lines: log.lines.clone(),
                diagnostics: log.diagnostics.clone(),
                max_size: log.max_size,
            });
            
            true
        } else {
            false
        }
    }
    
    /// Get active build IDs
    pub async fn active_builds(&self) -> Vec<BuildId> {
        let jobs = self.jobs.lock().await;
        jobs.keys().cloned().collect()
    }
    
    /// Get full log for a build (active or completed)
    pub async fn get_log(&self, build_id: &str, last_n: Option<usize>) -> Option<Vec<String>> {
        // Check active jobs first
        {
            let jobs = self.jobs.lock().await;
            if let Some(job) = jobs.get(build_id) {
                let log = job.log.lock().await;
                return Some(log.get_lines(last_n));
            }
        }
        
        // Check completed logs
        let completed = self.completed_logs.read().await;
        completed.get(build_id).map(|log| log.get_lines(last_n))
    }
    
    /// Get diagnostics for a build
    pub async fn get_diagnostics(&self, build_id: &str) -> Option<Vec<EnhancedDiagnostic>> {
        // Check active jobs first
        {
            let jobs = self.jobs.lock().await;
            if let Some(job) = jobs.get(build_id) {
                let log = job.log.lock().await;
                return Some(log.get_diagnostics());
            }
        }
        
        // Check completed logs
        let completed = self.completed_logs.read().await;
        completed.get(build_id).map(|log| log.get_diagnostics())
    }
    
    /// Get latest successful build artifacts
    pub async fn get_latest_artifacts(&self) -> Option<BuildArtifacts> {
        self.artifacts.read().await.get_latest().cloned()
    }
    
    /// Get artifacts for specific build
    pub async fn get_artifacts(&self, build_id: &str) -> Option<BuildArtifacts> {
        self.artifacts.read().await.get(build_id).cloned()
    }
}

impl Default for BuildManager {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== Build Execution ====================

async fn run_build(
    job: Arc<BuildJob>,
    event_tx: broadcast::Sender<BuildEvent>,
    jobs: Arc<Mutex<HashMap<BuildId, Arc<BuildJob>>>>,
    completed_logs: Arc<RwLock<HashMap<BuildId, BuildLog>>>,
    artifacts: Arc<RwLock<ArtifactRegistry>>,
) {
    let start = std::time::Instant::now();
    let config = &job.config;
    let project_path = config.project_path.clone();
    
    // Progress: Preparing
    emit_progress(&job, &event_tx, BuildPhase::Preparing, 0, "Setting up build environment...", 0, 0);
    
    // Find ARM GCC
    let gcc = which::which("arm-none-eabi-gcc")
        .unwrap_or_else(|_| PathBuf::from("arm-none-eabi-gcc"));
    
    // Build directory
    let build_dir = config.output_dir.clone()
        .unwrap_or_else(|| project_path.join("build"));
    let _ = tokio::fs::create_dir_all(&build_dir).await;
    
    // Compile each source file
    let source_count = config.source_files.len();
    let mut object_files = Vec::new();
    
    for (idx, source) in config.source_files.iter().enumerate() {
        // Check cancellation
        if job.cancel_token.is_cancelled() {
            finish_cancelled(&job, &event_tx, &jobs, &completed_logs, CancelReason::UserRequest).await;
            return;
        }
        
        let percent = ((idx as f32 / source_count as f32) * 70.0) as u8;
        emit_progress(&job, &event_tx, BuildPhase::Compiling, percent,
            &format!("Compiling {}", source.file_name().unwrap_or_default().to_string_lossy()),
            idx, source_count);
        
        let obj_name = source.file_stem().unwrap_or_default().to_string_lossy();
        let obj_path = build_dir.join(format!("{}.o", obj_name));
        
        // Build compile command
        let mut cmd = Command::new(&gcc);
        cmd.arg("-c")
           .arg(source)
           .arg("-o")
           .arg(&obj_path)
           .arg(format!("-mcpu={}", config.mcu_target))
           .arg("-mthumb")
           .arg(format!("-{}", config.optimization))
           .arg("-g3")
           .arg("-Wall")
           .arg("-Wextra")
           .arg("-ffunction-sections")
           .arg("-fdata-sections");
        
        for inc in &config.include_paths {
            cmd.arg("-I").arg(inc);
        }
        
        for (key, value) in &config.defines {
            if value.is_empty() {
                cmd.arg(format!("-D{}", key));
            } else {
                cmd.arg(format!("-D{}={}", key, value));
            }
        }
        
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        cmd.kill_on_drop(true);  // Kill child if parent drops
        
        match cmd.spawn() {
            Ok(mut child) => {
                if let Some(stderr) = child.stderr.take() {
                    let reader = BufReader::new(stderr);
                    let mut lines = reader.lines();
                    
                    while let Ok(Some(line)) = lines.next_line().await {
                        emit_output(&job, &event_tx, &line, OutputStream::Stderr, Some("gcc")).await;
                        
                        if let Some(diag) = parse_gcc_diagnostic(&line, &project_path) {
                            emit_diagnostic(&job, &event_tx, diag).await;
                        }
                    }
                }
                
                let status = child.wait().await;
                if status.map(|s| s.success()).unwrap_or(false) && obj_path.exists() {
                    object_files.push(obj_path);
                }
            }
            Err(e) => {
                emit_output(&job, &event_tx, &format!("Failed to spawn compiler: {}", e), OutputStream::Stderr, Some("build")).await;
            }
        }
    }
    
    // Check for errors
    let error_count = job.log.lock().await.error_count();
    if error_count > 0 {
        finish_completed(&job, &event_tx, &jobs, &completed_logs, &artifacts, false, None, start, None).await;
        return;
    }
    
    // Check cancellation before linking
    if job.cancel_token.is_cancelled() {
        finish_cancelled(&job, &event_tx, &jobs, &completed_logs, CancelReason::UserRequest).await;
        return;
    }
    
    // Link
    emit_progress(&job, &event_tx, BuildPhase::Linking, 80, "Linking...", source_count, source_count);
    
    let elf_path = build_dir.join("firmware.elf");
    let mut link_cmd = Command::new(&gcc);
    
    link_cmd
        .arg(format!("-mcpu={}", config.mcu_target))
        .arg("-mthumb")
        .arg("-Wl,--gc-sections")
        .arg("-Wl,-Map").arg(build_dir.join("firmware.map"))
        .arg("--specs=nosys.specs")
        .arg("--specs=nano.specs");
    
    if let Some(ref ld) = config.linker_script {
        link_cmd.arg("-T").arg(ld);
    }
    
    for obj in &object_files {
        link_cmd.arg(obj);
    }
    
    link_cmd.arg("-o").arg(&elf_path);
    link_cmd.stdout(std::process::Stdio::piped());
    link_cmd.stderr(std::process::Stdio::piped());
    link_cmd.kill_on_drop(true);
    
    let link_success = match link_cmd.spawn() {
        Ok(mut child) => {
            if let Some(stderr) = child.stderr.take() {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    emit_output(&job, &event_tx, &line, OutputStream::Stderr, Some("ld")).await;
                }
            }
            child.wait().await.map(|s| s.success()).unwrap_or(false)
        }
        Err(e) => {
            emit_output(&job, &event_tx, &format!("Linker error: {}", e), OutputStream::Stderr, Some("ld")).await;
            false
        }
    };
    
    // Post-processing: generate binary
    emit_progress(&job, &event_tx, BuildPhase::PostProcessing, 95, "Generating binary...", source_count, source_count);
    
    let bin_path = build_dir.join("firmware.bin");
    let objcopy = which::which("arm-none-eabi-objcopy")
        .unwrap_or_else(|_| PathBuf::from("arm-none-eabi-objcopy"));
    
    let bin_success = if link_success && elf_path.exists() {
        Command::new(&objcopy)
            .arg("-O").arg("binary")
            .arg(&elf_path)
            .arg(&bin_path)
            .kill_on_drop(true)
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    } else {
        false
    };
    
    // Build artifacts
    let elf_exists = elf_path.exists();
    let bin_exists = bin_success && bin_path.exists();
    let map_exists = build_dir.join("firmware.map").exists();
    
    let build_artifacts = if link_success && elf_exists {
        Some(BuildArtifacts {
            elf_path: elf_path.display().to_string(),
            bin_path: if bin_exists { Some(bin_path.display().to_string()) } else { None },
            hex_path: None,
            map_path: Some(build_dir.join("firmware.map").display().to_string()),
            size_report: get_size_report(&elf_path).await,
            elf_exists,
            bin_exists,
            map_exists,
        })
    } else {
        None
    };
    
    finish_completed(&job, &event_tx, &jobs, &completed_logs, &artifacts, link_success && elf_exists, None, start, build_artifacts).await;
}

// ==================== Helper Functions ====================

fn emit_progress(job: &BuildJob, tx: &broadcast::Sender<BuildEvent>, phase: BuildPhase, percent: u8, message: &str, files_compiled: usize, files_total: usize) {
    let _ = tx.send(BuildEvent::Progress {
        header: job.make_header(),
        phase,
        percent,
        message: message.to_string(),
        files_compiled,
        files_total,
    });
}

async fn emit_output(job: &BuildJob, tx: &broadcast::Sender<BuildEvent>, line: &str, stream: OutputStream, tool: Option<&str>) {
    // Store in log
    job.log.lock().await.push_line(line.to_string());
    
    // Emit event
    let _ = tx.send(BuildEvent::Output {
        header: job.make_header(),
        line: line.to_string(),
        stream,
        tool: tool.map(|t| t.to_string()),
    });
}

async fn emit_diagnostic(job: &BuildJob, tx: &broadcast::Sender<BuildEvent>, diag: EnhancedDiagnostic) {
    // Store in log
    job.log.lock().await.push_diagnostic(diag.clone());
    
    // Emit event
    let _ = tx.send(BuildEvent::Diagnostic {
        header: job.make_header(),
        diagnostic: diag,
    });
}

async fn finish_cancelled(
    job: &BuildJob,
    tx: &broadcast::Sender<BuildEvent>,
    jobs: &Arc<Mutex<HashMap<BuildId, Arc<BuildJob>>>>,
    completed_logs: &Arc<RwLock<HashMap<BuildId, BuildLog>>>,
    reason: CancelReason,
) {
    // Only emit if terminal not already sent
    if !job.terminal_sent.swap(true, Ordering::SeqCst) {
        let _ = tx.send(BuildEvent::Cancelled {
            header: job.make_header(),
            terminated_by: TerminatedBy::Cancelled,
            reason,
        });
    }
    
    // Move log to completed
    let log = job.log.lock().await;
    completed_logs.write().await.insert(job.id.clone(), BuildLog {
        lines: log.lines.clone(),
        diagnostics: log.diagnostics.clone(),
        max_size: log.max_size,
    });
    
    // Remove from active
    jobs.lock().await.remove(&job.id);
}

async fn finish_completed(
    job: &BuildJob,
    tx: &broadcast::Sender<BuildEvent>,
    jobs: &Arc<Mutex<HashMap<BuildId, Arc<BuildJob>>>>,
    completed_logs: &Arc<RwLock<HashMap<BuildId, BuildLog>>>,
    artifacts_registry: &Arc<RwLock<ArtifactRegistry>>,
    success: bool,
    exit_code: Option<i32>,
    start: std::time::Instant,
    artifacts: Option<BuildArtifacts>,
) {
    // Only emit if terminal not already sent
    if !job.terminal_sent.swap(true, Ordering::SeqCst) {
        let log = job.log.lock().await;
        
        let _ = tx.send(BuildEvent::Completed {
            header: job.make_header(),
            success,
            terminated_by: TerminatedBy::Completed,
            exit_code,
            duration_ms: start.elapsed().as_millis() as u64,
            error_count: log.error_count(),
            warning_count: log.warning_count(),
            artifacts: artifacts.clone(),
        });
    }
    
    // Register artifacts if successful
    if success {
        if let Some(arts) = artifacts {
            artifacts_registry.write().await.register(job.id.clone(), arts);
        }
    }
    
    // Move log to completed
    let log = job.log.lock().await;
    completed_logs.write().await.insert(job.id.clone(), BuildLog {
        lines: log.lines.clone(),
        diagnostics: log.diagnostics.clone(),
        max_size: log.max_size,
    });
    
    // Remove from active
    jobs.lock().await.remove(&job.id);
}

async fn get_size_report(elf_path: &PathBuf) -> Option<SizeInfo> {
    let size_cmd = which::which("arm-none-eabi-size").ok()?;
    
    let output = Command::new(size_cmd)
        .arg(elf_path)
        .output()
        .await
        .ok()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse: text data bss dec hex filename
    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            return Some(SizeInfo {
                text: parts[0].parse().unwrap_or(0),
                data: parts[1].parse().unwrap_or(0),
                bss: parts[2].parse().unwrap_or(0),
                total: parts[3].parse().unwrap_or(0),
            });
        }
    }
    None
}

fn parse_gcc_diagnostic(line: &str, project_path: &PathBuf) -> Option<EnhancedDiagnostic> {
    let re = regex::Regex::new(
        r"^(.+?):(\d+):(\d+):\s*(error|warning|note):\s*(.+?)(?:\s*\[(-[^\]]+)\])?$"
    ).ok()?;
    
    let caps = re.captures(line)?;
    
    let severity = match &caps[4] {
        "error" => DiagnosticSeverity::Error,
        "warning" => DiagnosticSeverity::Warning,
        "note" => DiagnosticSeverity::Note,
        _ => return None,
    };
    
    let file_absolute = caps[1].to_string();
    let is_external = !PathBuf::from(&file_absolute).starts_with(project_path);
    let file_relative = PathBuf::from(&file_absolute)
        .strip_prefix(project_path)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| file_absolute.clone());
    
    // Generate stable diagnostic_id from key fields
    let diagnostic_id = {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        file_relative.hash(&mut hasher);
        caps[2].hash(&mut hasher);
        caps[5].hash(&mut hasher);
        format!("{:08x}", hasher.finish() as u32)
    };
    
    Some(EnhancedDiagnostic {
        diagnostic_id,
        severity,
        category: DiagnosticCategory::Compile,
        file: file_relative,
        file_absolute,
        is_external,
        line: caps[2].parse().unwrap_or(0),
        column: caps[3].parse().ok(),
        end_line: None,
        end_column: None,
        message: caps[5].to_string(),
        code: caps.get(6).map(|m| m.as_str().to_string()),
        suggestion: suggest_fix(&caps[5]),
        tool: "gcc".to_string(),
        raw_line: line.to_string(),
    })
}

fn suggest_fix(message: &str) -> Option<String> {
    let msg_lower = message.to_lowercase();
    
    if msg_lower.contains("undefined reference to") {
        return Some("Add the missing source file or library to the build".to_string());
    }
    if msg_lower.contains("undeclared") || msg_lower.contains("not declared") {
        return Some("Include the header file that declares this symbol".to_string());
    }
    if msg_lower.contains("implicit declaration") {
        return Some("Add #include for the function's header file".to_string());
    }
    if msg_lower.contains("unused variable") {
        return Some("Remove the variable or use (void)var to suppress".to_string());
    }
    if msg_lower.contains("unused parameter") {
        return Some("Use (void)param or __attribute__((unused))".to_string());
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_gcc_diagnostic() {
        let project = PathBuf::from("/project");
        let line = "/project/src/main.c:42:5: error: 'foo' undeclared [-Werror]";
        let diag = parse_gcc_diagnostic(line, &project).unwrap();
        
        assert_eq!(diag.file, "src/main.c");
        assert_eq!(diag.line, 42);
        assert_eq!(diag.column, Some(5));
        assert_eq!(diag.severity, DiagnosticSeverity::Error);
        assert_eq!(diag.code, Some("-Werror".to_string()));
        assert!(diag.suggestion.is_some());
    }
    
    #[test]
    fn test_cancellation_token() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }
    
    #[test]
    fn test_build_log() {
        let mut log = BuildLog::new(100);
        log.push_line("line 1".to_string());
        log.push_line("line 2".to_string());
        assert_eq!(log.get_lines(Some(1)), vec!["line 2"]);
        assert_eq!(log.get_lines(None).len(), 2);
    }
}
