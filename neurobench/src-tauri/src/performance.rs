// Performance Monitoring Module
// Task Manager-style system metrics for PC and embedded devices

use serde::{Deserialize, Serialize};
use sysinfo::{System, Disks, Networks, Pid, ProcessesToUpdate};
use std::collections::HashMap;

/// System metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub disks: Vec<DiskMetrics>,
    pub network: NetworkMetrics,
    pub uptime: u64,
    pub timestamp: u64,
}

/// CPU metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub usage_percent: f32,
    pub core_count: usize,
    pub per_core_usage: Vec<f32>,
    pub frequency_mhz: u64,
    pub name: String,
}

/// Memory metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f32,
    pub swap_total: u64,
    pub swap_used: u64,
}

/// Disk metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub usage_percent: f32,
    pub file_system: String,
    pub is_removable: bool,
}

/// Network metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub interfaces: Vec<NetworkInterface>,
    pub total_received: u64,
    pub total_transmitted: u64,
    pub receive_speed_bps: u64,
    pub transmit_speed_bps: u64,
}

/// Network interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
    pub receive_speed_bps: u64,
    pub transmit_speed_bps: u64,
}

/// Process information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub memory_percent: f32,
    pub status: String,
    pub start_time: u64,
}

/// Embedded device metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedMetrics {
    pub device_name: String,
    pub port: String,
    pub connected: bool,
    pub power_mw: f32,
    pub voltage_v: f32,
    pub current_ma: f32,
    pub temperature_c: f32,
    pub clock_mhz: u32,
    pub flash_used_kb: u32,
    pub ram_used_kb: u32,
    pub flash_total_kb: u32,
    pub ram_total_kb: u32,
}

/// Performance history for graphs (last 60 data points)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceHistory {
    pub cpu_history: Vec<f32>,
    pub memory_history: Vec<f32>,
    pub disk_read_history: Vec<u64>,
    pub disk_write_history: Vec<u64>,
    pub network_rx_history: Vec<u64>,
    pub network_tx_history: Vec<u64>,
    pub power_history: Vec<f32>,
    pub temperature_history: Vec<f32>,
}

impl PerformanceHistory {
    pub fn new() -> Self {
        Self {
            cpu_history: Vec::with_capacity(60),
            memory_history: Vec::with_capacity(60),
            disk_read_history: Vec::with_capacity(60),
            disk_write_history: Vec::with_capacity(60),
            network_rx_history: Vec::with_capacity(60),
            network_tx_history: Vec::with_capacity(60),
            power_history: Vec::with_capacity(60),
            temperature_history: Vec::with_capacity(60),
        }
    }

    pub fn push_cpu(&mut self, value: f32) {
        if self.cpu_history.len() >= 60 {
            self.cpu_history.remove(0);
        }
        self.cpu_history.push(value);
    }

    pub fn push_memory(&mut self, value: f32) {
        if self.memory_history.len() >= 60 {
            self.memory_history.remove(0);
        }
        self.memory_history.push(value);
    }

    pub fn push_network(&mut self, rx: u64, tx: u64) {
        if self.network_rx_history.len() >= 60 {
            self.network_rx_history.remove(0);
            self.network_tx_history.remove(0);
        }
        self.network_rx_history.push(rx);
        self.network_tx_history.push(tx);
    }
}

/// Get current system metrics
pub fn get_system_metrics() -> SystemMetrics {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    // CPU metrics
    let cpu_usage: f32 = sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32;
    let per_core: Vec<f32> = sys.cpus().iter().map(|c| c.cpu_usage()).collect();
    let cpu_name = sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default();
    let cpu_freq = sys.cpus().first().map(|c| c.frequency()).unwrap_or(0);
    
    let cpu = CpuMetrics {
        usage_percent: cpu_usage,
        core_count: sys.cpus().len(),
        per_core_usage: per_core,
        frequency_mhz: cpu_freq,
        name: cpu_name,
    };
    
    // Memory metrics
    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();
    let available_mem = sys.available_memory();
    
    let memory = MemoryMetrics {
        total_bytes: total_mem,
        used_bytes: used_mem,
        available_bytes: available_mem,
        usage_percent: if total_mem > 0 { (used_mem as f32 / total_mem as f32) * 100.0 } else { 0.0 },
        swap_total: sys.total_swap(),
        swap_used: sys.used_swap(),
    };
    
    // Disk metrics
    let disks_info = Disks::new_with_refreshed_list();
    let disks: Vec<DiskMetrics> = disks_info.iter().map(|d| {
        let total = d.total_space();
        let available = d.available_space();
        let used = total.saturating_sub(available);
        DiskMetrics {
            name: d.name().to_string_lossy().to_string(),
            mount_point: d.mount_point().to_string_lossy().to_string(),
            total_bytes: total,
            available_bytes: available,
            used_bytes: used,
            usage_percent: if total > 0 { (used as f32 / total as f32) * 100.0 } else { 0.0 },
            file_system: d.file_system().to_string_lossy().to_string(),
            is_removable: d.is_removable(),
        }
    }).collect();
    
    // Network metrics
    let networks = Networks::new_with_refreshed_list();
    let mut interfaces = Vec::new();
    let mut total_rx: u64 = 0;
    let mut total_tx: u64 = 0;
    
    for (name, data) in networks.iter() {
        let rx = data.total_received();
        let tx = data.total_transmitted();
        total_rx += rx;
        total_tx += tx;
        
        interfaces.push(NetworkInterface {
            name: name.clone(),
            received_bytes: rx,
            transmitted_bytes: tx,
            receive_speed_bps: data.received(),
            transmit_speed_bps: data.transmitted(),
        });
    }
    
    let network = NetworkMetrics {
        interfaces,
        total_received: total_rx,
        total_transmitted: total_tx,
        receive_speed_bps: 0, // Calculated from delta
        transmit_speed_bps: 0,
    };
    
    SystemMetrics {
        cpu,
        memory,
        disks,
        network,
        uptime: System::uptime(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    }
}

/// Get running processes sorted by CPU usage
pub fn get_process_list(limit: usize) -> Vec<ProcessInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    // Need a second refresh to get accurate CPU usage
    std::thread::sleep(std::time::Duration::from_millis(100));
    sys.refresh_processes(ProcessesToUpdate::All, true);
    
    let total_mem = sys.total_memory() as f32;
    
    let mut processes: Vec<ProcessInfo> = sys.processes().iter().map(|(pid, proc)| {
        ProcessInfo {
            pid: pid.as_u32(),
            name: proc.name().to_string_lossy().to_string(),
            cpu_percent: proc.cpu_usage(),
            memory_bytes: proc.memory(),
            memory_percent: if total_mem > 0.0 { (proc.memory() as f32 / total_mem) * 100.0 } else { 0.0 },
            status: format!("{:?}", proc.status()),
            start_time: proc.start_time(),
        }
    }).collect();
    
    // Sort by CPU usage descending
    processes.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal));
    
    processes.truncate(limit);
    processes
}

/// Get embedded device metrics (simulated for now, would connect to real device)
pub fn get_embedded_metrics(port: Option<&str>) -> EmbeddedMetrics {
    // In a real implementation, this would:
    // 1. Connect to the device via serial/JTAG
    // 2. Query power monitor (INA226)
    // 3. Read temperature sensor
    // 4. Query debug interface for memory usage
    
    // Simulated data using timestamp for variation
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    
    // Simple pseudo-random variation based on timestamp
    let variation = ((timestamp % 100) as f32 - 50.0) / 10.0;
    
    let base_power: f32 = 42.0;
    let base_temp: f32 = 32.0;
    
    EmbeddedMetrics {
        device_name: "STM32F401CCU6".to_string(),
        port: port.unwrap_or("COM3").to_string(),
        connected: true,
        power_mw: base_power + variation,
        voltage_v: 3.3,
        current_ma: (base_power + variation) / 3.3,
        temperature_c: base_temp + variation * 0.5,
        clock_mhz: 84,
        flash_used_kb: 48,
        ram_used_kb: 12,
        flash_total_kb: 256,
        ram_total_kb: 64,
    }
}

/// Format bytes to human readable
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    
    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format uptime to human readable
pub fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let mins = (seconds % 3600) / 60;
    
    if days > 0 {
        format!("{}d {}h {}m", days, hours, mins)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_system_metrics() {
        let metrics = get_system_metrics();
        assert!(metrics.cpu.core_count > 0);
        assert!(metrics.memory.total_bytes > 0);
    }

    #[test]
    fn test_get_process_list() {
        let processes = get_process_list(10);
        assert!(!processes.is_empty());
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(3600), "1h 0m");
        assert_eq!(format_uptime(86400), "1d 0h 0m");
    }
}
