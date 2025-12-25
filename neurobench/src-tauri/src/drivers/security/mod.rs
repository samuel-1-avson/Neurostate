// Production Security Module
// Bootloader, OTA, Secure Boot, Flash Encryption

use serde::{Deserialize, Serialize};

// ============================================================================
// Bootloader Configuration
// ============================================================================

/// Bootloader type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BootloaderType {
    SingleBank,
    DualBank,
    DualBankWithRollback,
}

/// Flash region configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashRegion {
    pub name: String,
    pub start_address: u32,
    pub size: u32,
    pub is_writable: bool,
}

/// Bootloader configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootloaderConfig {
    pub name: String,
    pub bootloader_type: BootloaderType,
    pub flash_base: u32,
    pub flash_size: u32,
    pub bootloader_size: u32,
    pub app_size: u32,
    pub vector_table_offset: u32,
    pub enable_watchdog: bool,
    pub boot_timeout_ms: u32,
    pub enable_crc_check: bool,
    pub enable_signature_check: bool,
}

impl Default for BootloaderConfig {
    fn default() -> Self {
        Self {
            name: "bootloader".to_string(),
            bootloader_type: BootloaderType::DualBank,
            flash_base: 0x08000000,  // STM32 flash base
            flash_size: 512 * 1024,   // 512KB
            bootloader_size: 32 * 1024, // 32KB bootloader
            app_size: 224 * 1024,     // 224KB per app slot
            vector_table_offset: 0x8000,
            enable_watchdog: true,
            boot_timeout_ms: 3000,
            enable_crc_check: true,
            enable_signature_check: false,
        }
    }
}

// ============================================================================
// OTA Update Configuration
// ============================================================================

/// OTA transport
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OtaTransport {
    HTTP,
    HTTPS,
    MQTT,
    BLE,
    UART,
}

/// OTA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtaConfig {
    pub name: String,
    pub transport: OtaTransport,
    pub server_url: String,
    pub firmware_path: String,
    pub version_check: bool,
    pub chunk_size: u32,
    pub retry_count: u8,
    pub timeout_ms: u32,
    pub verify_signature: bool,
    pub verify_checksum: bool,
}

impl Default for OtaConfig {
    fn default() -> Self {
        Self {
            name: "ota_updater".to_string(),
            transport: OtaTransport::HTTPS,
            server_url: "https://firmware.example.com".to_string(),
            firmware_path: "/firmware/latest.bin".to_string(),
            version_check: true,
            chunk_size: 4096,
            retry_count: 3,
            timeout_ms: 30000,
            verify_signature: true,
            verify_checksum: true,
        }
    }
}

// ============================================================================
// Secure Boot Configuration
// ============================================================================

/// Crypto algorithm for secure boot
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SecureBootAlgorithm {
    RSA2048,
    RSA4096,
    ECDSA256,
    ECDSA384,
    ED25519,
}

/// Secure boot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureBootConfig {
    pub name: String,
    pub algorithm: SecureBootAlgorithm,
    pub public_key_hash: Option<String>,
    pub enable_rollback_protection: bool,
    pub secure_counter_address: Option<u32>,
    pub enable_debug_lock: bool,
    pub enable_jtag_disable: bool,
}

impl Default for SecureBootConfig {
    fn default() -> Self {
        Self {
            name: "secure_boot".to_string(),
            algorithm: SecureBootAlgorithm::ECDSA256,
            public_key_hash: None,
            enable_rollback_protection: true,
            secure_counter_address: None,
            enable_debug_lock: true,
            enable_jtag_disable: false,
        }
    }
}

// ============================================================================
// Flash Encryption Configuration
// ============================================================================

/// Encryption algorithm
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    AES128CTR,
    AES256CTR,
    AES128GCM,
    AES256GCM,
    ChaCha20,
}

/// Flash encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashEncryptionConfig {
    pub name: String,
    pub algorithm: EncryptionAlgorithm,
    pub key_source: String,  // "OTP", "FUSE", "External"
    pub encrypt_bootloader: bool,
    pub encrypt_app: bool,
    pub encrypt_data: bool,
    pub iv_strategy: String, // "COUNTER", "RANDOM", "ADDRESS"
}

impl Default for FlashEncryptionConfig {
    fn default() -> Self {
        Self {
            name: "flash_encrypt".to_string(),
            algorithm: EncryptionAlgorithm::AES256CTR,
            key_source: "OTP".to_string(),
            encrypt_bootloader: false,
            encrypt_app: true,
            encrypt_data: true,
            iv_strategy: "ADDRESS".to_string(),
        }
    }
}

// ============================================================================
// Crypto Utilities Configuration
// ============================================================================

/// Hash algorithm
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HashAlgorithm {
    SHA256,
    SHA384,
    SHA512,
    SHA3_256,
}

/// Crypto utility configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    pub name: String,
    pub include_aes: bool,
    pub include_hash: bool,
    pub include_rsa: bool,
    pub include_ecdsa: bool,
    pub include_rng: bool,
    pub use_hardware_crypto: bool,
    pub hash_algorithm: HashAlgorithm,
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            name: "crypto_utils".to_string(),
            include_aes: true,
            include_hash: true,
            include_rsa: false,
            include_ecdsa: true,
            include_rng: true,
            use_hardware_crypto: true,
            hash_algorithm: HashAlgorithm::SHA256,
        }
    }
}

pub mod bootloader;
pub mod ota;
pub mod secure_boot;
pub mod crypto;
