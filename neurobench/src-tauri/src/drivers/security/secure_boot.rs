// Secure Boot Generator
// Firmware signature verification and anti-rollback

use super::*;

/// Generate secure boot verification code
pub fn generate_secure_boot_code(config: &SecureBootConfig) -> String {
    let algorithm_code = match config.algorithm {
        SecureBootAlgorithm::RSA2048 => ("RSA", 256, "mbedtls_rsa_pkcs1_verify"),
        SecureBootAlgorithm::RSA4096 => ("RSA", 512, "mbedtls_rsa_pkcs1_verify"),
        SecureBootAlgorithm::ECDSA256 => ("ECDSA", 64, "mbedtls_ecdsa_verify"),
        SecureBootAlgorithm::ECDSA384 => ("ECDSA", 96, "mbedtls_ecdsa_verify"),
        SecureBootAlgorithm::ED25519 => ("ED25519", 64, "ed25519_verify"),
    };

    format!(r#"/**
 * Secure Boot Verification: {name}
 * Algorithm: {algorithm:?}
 * Rollback Protection: {rollback}
 * Debug Lock: {debug_lock}
 */

#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include "mbedtls/sha256.h"
#include "mbedtls/pk.h"
#include "mbedtls/ecdsa.h"

#define SIGNATURE_SIZE {sig_size}
#define ALGORITHM_TYPE "{algo_type}"

// Configuration
#define ENABLE_ROLLBACK_PROTECTION {rollback}
#define ENABLE_DEBUG_LOCK          {debug_lock}
#define ENABLE_JTAG_DISABLE        {jtag_disable}

{rollback_code}

// Public key for signature verification (embed your key here)
static const uint8_t public_key[] = {{
    // Replace with your actual public key bytes
    0x00  // Placeholder
}};

static const uint8_t public_key_hash[32] = {{
    // SHA256 hash of the public key for additional verification
    0x00  // Placeholder
}};

typedef struct {{
    uint8_t signature[SIGNATURE_SIZE];
    uint32_t image_size;
    uint32_t version;
    uint8_t hash[32];
}} secure_header_t;

// Calculate SHA256 hash of firmware image
bool secure_boot_hash_image(const uint8_t *data, uint32_t len, uint8_t *hash_out) {{
    mbedtls_sha256_context sha_ctx;
    
    mbedtls_sha256_init(&sha_ctx);
    mbedtls_sha256_starts(&sha_ctx, 0);  // 0 = SHA256
    mbedtls_sha256_update(&sha_ctx, data, len);
    mbedtls_sha256_finish(&sha_ctx, hash_out);
    mbedtls_sha256_free(&sha_ctx);
    
    return true;
}}

// Verify signature using {algo_type}
bool secure_boot_verify_signature(const uint8_t *hash, const uint8_t *signature) {{
    mbedtls_pk_context pk;
    int ret;
    
    mbedtls_pk_init(&pk);
    
    // Parse public key
    ret = mbedtls_pk_parse_public_key(&pk, public_key, sizeof(public_key));
    if (ret != 0) {{
        mbedtls_pk_free(&pk);
        return false;
    }}
    
    // Verify signature
    ret = mbedtls_pk_verify(&pk, MBEDTLS_MD_SHA256, 
                            hash, 32, 
                            signature, SIGNATURE_SIZE);
    
    mbedtls_pk_free(&pk);
    
    return (ret == 0);
}}

#if ENABLE_ROLLBACK_PROTECTION
// Check if new version is greater than stored version
static uint32_t read_secure_counter(void) {{
    // Read from OTP/eFuse area
    return *(volatile uint32_t *)SECURE_COUNTER_ADDR;
}}

static bool write_secure_counter(uint32_t value) {{
    // Write to OTP/eFuse - this is usually irreversible!
    // Implementation depends on MCU
    return true;
}}

bool secure_boot_check_rollback(uint32_t image_version) {{
    uint32_t stored_version = read_secure_counter();
    
    if (image_version < stored_version) {{
        // Rollback detected!
        return false;
    }}
    
    return true;
}}

bool secure_boot_commit_version(uint32_t image_version) {{
    uint32_t stored_version = read_secure_counter();
    
    if (image_version > stored_version) {{
        return write_secure_counter(image_version);
    }}
    
    return true;
}}
#endif

#if ENABLE_DEBUG_LOCK
void secure_boot_lock_debug(void) {{
    // Disable debug access
    // STM32 example: Set RDP level 2 or DBGMCU->CR = 0
    
    // Read device specific registers
    // Flash Option bytes typically control this
}}
#endif

#if ENABLE_JTAG_DISABLE
void secure_boot_disable_jtag(void) {{
    // Disable JTAG pins
    // Remap pins to GPIO or disable debug interface
    
    // STM32 example
    #ifdef STM32
    __HAL_AFIO_REMAP_SWJ_DISABLE();
    #endif
}}
#endif

// Main secure boot verification function
typedef enum {{
    SECURE_BOOT_OK,
    SECURE_BOOT_HASH_ERROR,
    SECURE_BOOT_SIGNATURE_ERROR,
    SECURE_BOOT_ROLLBACK_ERROR,
    SECURE_BOOT_KEY_ERROR
}} secure_boot_result_t;

secure_boot_result_t secure_boot_verify_image(uint32_t image_address) {{
    secure_header_t *header = (secure_header_t *)image_address;
    uint8_t calculated_hash[32];
    
    // Calculate image hash (skip header)
    const uint8_t *image_data = (uint8_t *)(image_address + sizeof(secure_header_t));
    
    if (!secure_boot_hash_image(image_data, header->image_size, calculated_hash)) {{
        return SECURE_BOOT_HASH_ERROR;
    }}
    
    // Compare hash
    if (memcmp(calculated_hash, header->hash, 32) != 0) {{
        return SECURE_BOOT_HASH_ERROR;
    }}
    
    // Verify signature
    if (!secure_boot_verify_signature(calculated_hash, header->signature)) {{
        return SECURE_BOOT_SIGNATURE_ERROR;
    }}
    
    #if ENABLE_ROLLBACK_PROTECTION
    if (!secure_boot_check_rollback(header->version)) {{
        return SECURE_BOOT_ROLLBACK_ERROR;
    }}
    #endif
    
    return SECURE_BOOT_OK;
}}

void secure_boot_init(void) {{
    #if ENABLE_DEBUG_LOCK
    secure_boot_lock_debug();
    #endif
    
    #if ENABLE_JTAG_DISABLE
    secure_boot_disable_jtag();
    #endif
}}
"#,
        name = config.name,
        algorithm = config.algorithm,
        sig_size = algorithm_code.1,
        algo_type = algorithm_code.0,
        rollback = if config.enable_rollback_protection { 1 } else { 0 },
        debug_lock = if config.enable_debug_lock { 1 } else { 0 },
        jtag_disable = if config.enable_jtag_disable { 1 } else { 0 },
        rollback_code = if config.enable_rollback_protection {
            format!(r#"
#define SECURE_COUNTER_ADDR 0x{:08X}
"#, config.secure_counter_address.unwrap_or(0x1FFF7800))
        } else {
            String::new()
        },
    )
}
