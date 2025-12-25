// Hardware Interface Commands

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

/// Detect connected hardware devices
#[tauri::command]
pub fn detect_devices() -> Result<Vec<DetectedDevice>, String> {
    log::info!("Scanning for connected devices...");
    
    // TODO: Implement actual hardware detection with probe-rs
    // For now, return mock data
    
    let devices = vec![
        DetectedDevice {
            id: "mock-stm32".to_string(),
            name: "STM32F401 BlackPill".to_string(),
            chip: "STM32F401CCU6".to_string(),
            interface: "ST-Link V2".to_string(),
            serial_number: Some("066DFF323536434E43114124".to_string()),
        },
        DetectedDevice {
            id: "mock-esp32".to_string(),
            name: "ESP32-DevKitC".to_string(),
            chip: "ESP32-D0WDQ6".to_string(),
            interface: "USB-JTAG".to_string(),
            serial_number: None,
        },
    ];
    
    log::info!("Found {} devices", devices.len());
    Ok(devices)
}

/// Connect to a specific device
#[tauri::command]
pub fn connect_device(device_id: String) -> Result<bool, String> {
    log::info!("Connecting to device: {}", device_id);
    
    // TODO: Implement actual connection with probe-rs
    
    Ok(true)
}

/// Disconnect from current device
#[tauri::command]
pub fn disconnect_device() -> Result<bool, String> {
    log::info!("Disconnecting from current device");
    
    // TODO: Implement actual disconnection
    
    Ok(true)
}

/// Flash firmware to connected device
#[tauri::command]
pub fn flash_firmware(firmware_path: String, device_id: String) -> Result<FlashResult, String> {
    log::info!("Flashing firmware {} to device {}", firmware_path, device_id);
    
    // TODO: Implement actual flashing with probe-rs
    
    Ok(FlashResult {
        success: true,
        bytes_written: 0,
        duration_ms: 0,
        message: "Flash simulation - probe-rs not yet enabled".to_string(),
    })
}

/// Read telemetry from connected device
#[tauri::command]
pub fn read_telemetry() -> Result<DeviceTelemetry, String> {
    log::debug!("Reading telemetry from device");
    
    // TODO: Implement actual telemetry reading via RTT
    
    Ok(DeviceTelemetry {
        cpu_load: 12.5,
        memory_used: 8192,
        memory_total: 65536,
        current_state: Some("IDLE".to_string()),
        uptime_ms: 123456,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashResult {
    pub success: bool,
    pub bytes_written: usize,
    pub duration_ms: u64,
    pub message: String,
}
