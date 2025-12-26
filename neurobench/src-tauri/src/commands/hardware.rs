// Hardware Interface Commands - with real probe-rs integration

use serde::{Deserialize, Serialize};

/// Detected hardware device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedDevice {
    pub id: String,
    pub name: String,
    pub chip: String,
    pub interface: String,
    pub serial_number: Option<String>,
}

/// Telemetry data from connected device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceTelemetry {
    pub cpu_load: f32,
    pub memory_used: u32,
    pub memory_total: u32,
    pub current_state: Option<String>,
    pub uptime_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashResult {
    pub success: bool,
    pub bytes_written: usize,
    pub duration_ms: u64,
    pub message: String,
}

// ============================================================================
// Probe-RS Implementation (when hardware feature is enabled)
// ============================================================================

#[cfg(feature = "hardware")]
mod hardware_impl {
    use super::*;
    use probe_rs::{
        probe::list::Lister,
        Permissions,
    };
    use std::time::Instant;

    /// Detect connected hardware devices using probe-rs
    pub fn detect_devices_impl() -> Result<Vec<DetectedDevice>, String> {
        log::info!("Scanning for connected probes using probe-rs...");
        
        let lister = Lister::new();
        let probes = lister.list_all();
        
        if probes.is_empty() {
            log::info!("No debug probes found");
            return Ok(vec![]);
        }
        
        let devices: Vec<DetectedDevice> = probes
            .into_iter()
            .enumerate()
            .map(|(idx, probe_info)| {
                let name = format!("{:?}", probe_info.probe_type());
                let serial = probe_info.serial_number.clone();
                
                DetectedDevice {
                    id: format!("probe-{}", idx),
                    name: name.clone(),
                    chip: "Attach to detect chip".to_string(),
                    interface: name,
                    serial_number: serial,
                }
            })
            .collect();
        
        log::info!("Found {} probe(s)", devices.len());
        Ok(devices)
    }

    /// Connect to a specific device and try to identify the chip
    pub fn connect_device_impl(device_id: String) -> Result<bool, String> {
        log::info!("Connecting to device: {}", device_id);
        
        let lister = Lister::new();
        let probes = lister.list_all();
        
        // Parse device index from id like "probe-0"
        let idx: usize = device_id
            .strip_prefix("probe-")
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| format!("Invalid device id: {}", device_id))?;
        
        let probe_info = probes.get(idx).ok_or("Device index out of range")?;
        
        // Try to open the probe
        match probe_info.open() {
            Ok(probe) => {
                log::info!("Successfully opened probe: {:?}", probe.get_name());
                // Note: We're not keeping the session open here, just testing connectivity
                Ok(true)
            }
            Err(e) => {
                log::error!("Failed to open probe: {:?}", e);
                Err(format!("Failed to connect: {:?}", e))
            }
        }
    }

    /// Flash firmware to connected device
    pub fn flash_firmware_impl(firmware_path: String, chip: String) -> Result<FlashResult, String> {
        use probe_rs::flashing::{download_file, Format, BinOptions};
        use std::path::Path;
        
        log::info!("Flashing firmware {} to chip {}", firmware_path, chip);
        
        let path = Path::new(&firmware_path);
        if !path.exists() {
            return Err(format!("Firmware file not found: {}", firmware_path));
        }
        
        let lister = Lister::new();
        let probes = lister.list_all();
        
        if probes.is_empty() {
            return Err("No debug probe connected".to_string());
        }
        
        // Open first available probe
        let probe_info = probes.into_iter().next().ok_or("No probe available")?;
        let probe = probe_info.open().map_err(|e| format!("Failed to open probe: {:?}", e))?;
        
        // Attach to target
        let mut session = probe.attach(&chip, Permissions::default())
            .map_err(|e| format!("Failed to attach to target: {:?}", e))?;
        
        let start = Instant::now();
        
        // Determine format from extension
        let format = if firmware_path.ends_with(".hex") {
            Format::Hex
        } else if firmware_path.ends_with(".bin") {
            // For .bin files, probe-rs needs base address - use default
            Format::Bin(BinOptions { base_address: Some(0x0800_0000), skip: 0 })
        } else {
            Format::Elf
        };
        
        // Download firmware
        download_file(&mut session, path, format)
            .map_err(|e| format!("Flash failed: {:?}", e))?;
        
        let duration = start.elapsed();
        
        // Get file size for bytes_written
        let bytes_written = std::fs::metadata(path)
            .map(|m| m.len() as usize)
            .unwrap_or(0);
        
        log::info!("Flash completed: {} bytes in {:?}", bytes_written, duration);
        
        Ok(FlashResult {
            success: true,
            bytes_written,
            duration_ms: duration.as_millis() as u64,
            message: format!("Successfully flashed {} to {}", firmware_path, chip),
        })
    }

    /// Read telemetry - placeholder, RTT would need more setup
    pub fn read_telemetry_impl() -> Result<DeviceTelemetry, String> {
        // RTT support requires maintaining a session and channel
        // For now, return a placeholder indicating RTT is available but not implemented
        log::debug!("RTT telemetry not yet implemented");
        
        Ok(DeviceTelemetry {
            cpu_load: 0.0,
            memory_used: 0,
            memory_total: 0,
            current_state: Some("RTT_NOT_CONNECTED".to_string()),
            uptime_ms: 0,
        })
    }
}

// ============================================================================
// Fallback Implementation (when hardware feature is disabled)
// ============================================================================

#[cfg(not(feature = "hardware"))]
mod hardware_impl {
    use super::*;

    pub fn detect_devices_impl() -> Result<Vec<DetectedDevice>, String> {
        log::info!("probe-rs feature not enabled - returning empty device list");
        Ok(vec![])
    }

    pub fn connect_device_impl(_device_id: String) -> Result<bool, String> {
        Err("probe-rs feature not enabled. Rebuild with --features hardware".to_string())
    }

    pub fn flash_firmware_impl(_firmware_path: String, _chip: String) -> Result<FlashResult, String> {
        Err("probe-rs feature not enabled. Rebuild with --features hardware".to_string())
    }

    pub fn read_telemetry_impl() -> Result<DeviceTelemetry, String> {
        Err("probe-rs feature not enabled. Rebuild with --features hardware".to_string())
    }
}

// ============================================================================
// Tauri Commands - delegate to appropriate implementation
// ============================================================================

/// Detect connected hardware devices
#[tauri::command]
pub fn detect_devices() -> Result<Vec<DetectedDevice>, String> {
    hardware_impl::detect_devices_impl()
}

/// Connect to a specific device
#[tauri::command]
pub fn connect_device(device_id: String) -> Result<bool, String> {
    hardware_impl::connect_device_impl(device_id)
}

/// Disconnect from current device
#[tauri::command]
pub fn disconnect_device() -> Result<bool, String> {
    log::info!("Disconnecting from current device");
    // Disconnect happens automatically when session is dropped
    Ok(true)
}

/// Flash firmware to connected device
#[tauri::command]
pub fn flash_firmware(firmware_path: String, chip: String) -> Result<FlashResult, String> {
    hardware_impl::flash_firmware_impl(firmware_path, chip)
}

/// Read telemetry from connected device
#[tauri::command]
pub fn read_telemetry() -> Result<DeviceTelemetry, String> {
    hardware_impl::read_telemetry_impl()
}
