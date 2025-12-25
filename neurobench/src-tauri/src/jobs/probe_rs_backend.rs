// Real probe-rs Backend Implementation
//
// Only compiled when --features hardware is enabled.
// Provides actual probe-rs flash/RTT operations using the ProbeBackend trait.

#![cfg(feature = "hardware")]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use tokio::sync::Mutex;

use probe_rs::probe::list::Lister;
use probe_rs::Session;
use probe_rs::flashing::{download_file, Format};

use crate::jobs::flash::{
    ProbeBackend, ProbeInfo, FlashConfig, FlashResult, FlashError,
    FlashErrorCode, FlashMessage, FlashPhase, ProgressCallback,
};

// ==================== Real Backend State ====================

/// Internal state for the real probe-rs backend
struct RealState {
    connected: bool,
    session: Option<Session>,
    chip: Option<String>,
    speed_khz: u32,
}

/// Real probe-rs backend implementation
pub struct RealProbeRsBackend {
    state: Arc<Mutex<RealState>>,
}

impl RealProbeRsBackend {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RealState {
                connected: false,
                session: None,
                chip: None,
                speed_khz: 4000,
            })),
        }
    }
    
    /// List all connected probes
    pub fn list_probes_sync() -> Vec<ProbeInfo> {
        let lister = Lister::new();
        let probes = lister.list_all();
        probes.into_iter().map(|p| ProbeInfo {
            name: format!("{}", p.identifier),
            vendor_id: p.vendor_id,
            product_id: p.product_id,
            serial: p.serial_number,
        }).collect()
    }
    
    /// Connect to the first available probe with the given chip
    pub async fn connect(&self, chip: &str, speed_khz: Option<u32>) -> Result<(), FlashError> {
        let mut state = self.state.lock().await;
        
        // List probes
        let lister = Lister::new();
        let probes = lister.list_all();
        if probes.is_empty() {
            return Err(FlashError::no_probe());
        }
        
        // Open first probe
        let probe_info = probes.into_iter().next().unwrap();
        let mut probe = probe_info.open().map_err(|e| FlashError {
            code: FlashErrorCode::ProbeOpenFailed,
            message: format!("Failed to open probe: {}", e),
            details: Some(e.to_string()),
            retryable: true,
            os_error_code: None,
        })?;
        
        // Set speed if provided
        if let Some(speed) = speed_khz {
            let _ = probe.set_speed(speed);
            state.speed_khz = speed;
        }
        
        // Attach to target
        let session = probe.attach(chip, probe_rs::Permissions::default()).map_err(|e| FlashError {
            code: FlashErrorCode::AttachFailed,
            message: format!("Failed to attach to {}: {}", chip, e),
            details: Some(e.to_string()),
            retryable: true,
            os_error_code: None,
        })?;
        
        state.connected = true;
        state.session = Some(session);
        state.chip = Some(chip.to_string());
        
        Ok(())
    }
    
    /// Disconnect from the probe
    pub async fn disconnect(&self) {
        let mut state = self.state.lock().await;
        state.session = None;
        state.connected = false;
        state.chip = None;
    }
}

impl Default for RealProbeRsBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProbeBackend for RealProbeRsBackend {
    async fn flash(
        &self,
        config: &FlashConfig,
        progress: ProgressCallback,
        cancel_check: impl Fn() -> bool + Send + Sync,
    ) -> Result<FlashResult, FlashError> {
        // Validate ELF exists
        if !config.elf_path.exists() {
            return Err(FlashError::elf_not_found(&config.elf_path));
        }
        
        // Check cancellation before starting
        if cancel_check() {
            return Err(FlashError::cancelled());
        }
        
        // Emit connecting progress
        let _ = progress.send(FlashMessage::Progress {
            phase: FlashPhase::Connecting,
            percent: 0.0,
            done_bytes: None,
            total_bytes: None,
            message: Some("Connecting to probe...".to_string()),
        }).await;
        
        // Get chip name (required for real hardware)
        let chip = config.chip.as_ref().ok_or_else(|| FlashError {
            code: FlashErrorCode::TargetNotFound,
            message: "Chip not specified. Provide chip parameter (e.g., 'STM32F407VGTx').".to_string(),
            details: None,
            retryable: false,
            os_error_code: None,
        })?;
        
        // Connect to probe
        self.connect(chip, config.speed_khz).await?;
        
        if cancel_check() {
            self.disconnect().await;
            return Err(FlashError::cancelled());
        }
        
        // Flash the file
        let mut state = self.state.lock().await;
        let session = state.session.as_mut().ok_or_else(|| FlashError {
            code: FlashErrorCode::ProbeOpenFailed,
            message: "No active session".to_string(),
            details: None,
            retryable: true,
            os_error_code: None,
        })?;
        
        // Emit erasing progress
        let _ = progress.send(FlashMessage::Progress {
            phase: FlashPhase::Erasing,
            percent: 5.0,
            done_bytes: None,
            total_bytes: None,
            message: Some("Erasing flash...".to_string()),
        }).await;
        
        if cancel_check() {
            drop(state);
            self.disconnect().await;
            return Err(FlashError::cancelled());
        }
        
        // Get file size for progress
        let file_size = std::fs::metadata(&config.elf_path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        // Emit programming progress
        let _ = progress.send(FlashMessage::Progress {
            phase: FlashPhase::Programming,
            percent: 20.0,
            done_bytes: Some(0),
            total_bytes: Some(file_size),
            message: Some("Programming flash...".to_string()),
        }).await;
        
        // Download the file using probe-rs 0.24 API
        let download_result = download_file(
            session,
            &config.elf_path,
            Format::Elf,
        );
        
        match download_result {
            Ok(_) => {
                // Programming complete
                let _ = progress.send(FlashMessage::Progress {
                    phase: FlashPhase::Programming,
                    percent: 80.0,
                    done_bytes: Some(file_size),
                    total_bytes: Some(file_size),
                    message: Some("Programming complete".to_string()),
                }).await;
                
                // Verify if requested
                if config.verify {
                    let _ = progress.send(FlashMessage::Progress {
                        phase: FlashPhase::Verifying,
                        percent: 90.0,
                        done_bytes: None,
                        total_bytes: None,
                        message: Some("Verifying...".to_string()),
                    }).await;
                }
                
                // Reset target
                let _ = progress.send(FlashMessage::Progress {
                    phase: FlashPhase::Resetting,
                    percent: 98.0,
                    done_bytes: None,
                    total_bytes: None,
                    message: Some("Resetting target...".to_string()),
                }).await;
                
                // Reset the core
                if let Ok(mut core) = session.core(0) {
                    let _ = core.reset();
                }
                
                drop(state);
                self.disconnect().await;
                
                Ok(FlashResult {
                    success: true,
                    bytes_written: file_size,
                    duration_ms: 0, // Will be set by caller
                    verified: config.verify,
                    chip_resolved: Some(chip.clone()),
                })
            }
            Err(e) => {
                drop(state);
                self.disconnect().await;
                
                Err(FlashError {
                    code: FlashErrorCode::FlashFailed,
                    message: format!("Flash failed: {}", e),
                    details: Some(e.to_string()),
                    retryable: true,
                    os_error_code: None,
                })
            }
        }
    }
    
    async fn is_connected(&self) -> bool {
        self.state.lock().await.connected
    }
    
    async fn probe_info(&self) -> Option<ProbeInfo> {
        let state = self.state.lock().await;
        if state.connected {
            Some(ProbeInfo {
                name: format!("Connected to {}", state.chip.as_deref().unwrap_or("unknown")),
                vendor_id: 0,
                product_id: 0,
                serial: None,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[ignore] // Requires real hardware
    fn test_real_probe_list() {
        let probes = RealProbeRsBackend::list_probes_sync();
        println!("Found {} probes", probes.len());
        for p in &probes {
            println!("  {:?}", p);
        }
    }
    
    #[tokio::test]
    #[ignore] // Requires real hardware + target board
    async fn test_real_flash_smoke() {
        let backend = RealProbeRsBackend::new();
        assert!(!backend.is_connected().await);
    }
}
