// Wireless Communication Module
// BLE, Zigbee, WiFi, LoRa, and Industrial Protocols

use serde::{Deserialize, Serialize};

// ============================================================================
// BLE (Bluetooth Low Energy) Configuration
// ============================================================================

/// BLE Role
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BleRole {
    Peripheral,
    Central,
}

/// GATT Characteristic Properties
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacteristicProperties {
    pub read: bool,
    pub write: bool,
    pub write_no_response: bool,
    pub notify: bool,
    pub indicate: bool,
}

/// GATT Characteristic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleCharacteristic {
    pub uuid: String,
    pub name: String,
    pub properties: CharacteristicProperties,
    pub max_length: u16,
    pub description: Option<String>,
}

/// GATT Service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleService {
    pub uuid: String,
    pub name: String,
    pub is_primary: bool,
    pub characteristics: Vec<BleCharacteristic>,
}

/// BLE Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleConfig {
    pub device_name: String,
    pub role: BleRole,
    pub services: Vec<BleService>,
    pub advertising_interval_ms: u16,
    pub connection_interval_ms: u16,
    pub mtu: u16,
}

impl Default for BleConfig {
    fn default() -> Self {
        Self {
            device_name: "NeuroBench".to_string(),
            role: BleRole::Peripheral,
            services: vec![],
            advertising_interval_ms: 100,
            connection_interval_ms: 20,
            mtu: 23,
        }
    }
}

// ============================================================================
// WiFi Configuration
// ============================================================================

/// WiFi Mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WifiMode {
    Station,
    AccessPoint,
    StationAndAP,
}

/// WiFi Security
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WifiSecurity {
    Open,
    WPA2Personal,
    WPA3Personal,
    WPA2Enterprise,
}

/// WiFi Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiConfig {
    pub mode: WifiMode,
    pub ssid: String,
    pub password: String,
    pub security: WifiSecurity,
    pub channel: u8,
    pub hostname: String,
    pub static_ip: Option<String>,
}

impl Default for WifiConfig {
    fn default() -> Self {
        Self {
            mode: WifiMode::Station,
            ssid: "MyNetwork".to_string(),
            password: "".to_string(),
            security: WifiSecurity::WPA2Personal,
            channel: 1,
            hostname: "neurobench".to_string(),
            static_ip: None,
        }
    }
}

// ============================================================================
// Zigbee Configuration
// ============================================================================

/// Zigbee Device Type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ZigbeeDeviceType {
    Coordinator,
    Router,
    EndDevice,
}

/// Zigbee Cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigbeeCluster {
    pub id: u16,
    pub name: String,
    pub is_server: bool,
}

/// Zigbee Endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigbeeEndpoint {
    pub id: u8,
    pub profile_id: u16,
    pub device_id: u16,
    pub clusters: Vec<ZigbeeCluster>,
}

/// Zigbee Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigbeeConfig {
    pub device_type: ZigbeeDeviceType,
    pub pan_id: u16,
    pub channel: u8,
    pub endpoints: Vec<ZigbeeEndpoint>,
    pub extended_pan_id: Option<u64>,
}

impl Default for ZigbeeConfig {
    fn default() -> Self {
        Self {
            device_type: ZigbeeDeviceType::EndDevice,
            pan_id: 0x1234,
            channel: 11,
            endpoints: vec![],
            extended_pan_id: None,
        }
    }
}

// ============================================================================
// LoRa Configuration
// ============================================================================

/// LoRa Spreading Factor
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LoraSpreadingFactor {
    SF7,
    SF8,
    SF9,
    SF10,
    SF11,
    SF12,
}

/// LoRa Bandwidth
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LoraBandwidth {
    BW125,
    BW250,
    BW500,
}

/// LoRa Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraConfig {
    pub frequency_mhz: u32,
    pub spreading_factor: LoraSpreadingFactor,
    pub bandwidth: LoraBandwidth,
    pub coding_rate: u8,
    pub tx_power_dbm: i8,
    pub sync_word: u8,
    pub preamble_length: u16,
}

impl Default for LoraConfig {
    fn default() -> Self {
        Self {
            frequency_mhz: 915,  // US ISM band
            spreading_factor: LoraSpreadingFactor::SF7,
            bandwidth: LoraBandwidth::BW125,
            coding_rate: 5,
            tx_power_dbm: 14,
            sync_word: 0x12,
            preamble_length: 8,
        }
    }
}

pub mod ble;
pub mod wifi;
pub mod zigbee;
pub mod lora;
