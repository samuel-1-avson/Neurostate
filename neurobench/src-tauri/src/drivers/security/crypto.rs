// Crypto Utilities Generator
// AES, SHA, RNG, and ECDSA utilities

use super::*;

/// Generate crypto utilities code
pub fn generate_crypto_code(config: &CryptoConfig) -> String {
    let mut code = format!(r#"/**
 * Crypto Utilities: {name}
 * AES: {aes}, Hash: {hash}, RNG: {rng}
 * Hardware Crypto: {hw_crypto}
 */

#include <stdint.h>
#include <stdbool.h>
#include <string.h>

"#,
        name = config.name,
        aes = config.include_aes,
        hash = config.include_hash,
        rng = config.include_rng,
        hw_crypto = config.use_hardware_crypto,
    );

    // Add AES code
    if config.include_aes {
        code.push_str(r#"
// ============================================================================
// AES Encryption/Decryption
// ============================================================================

#ifdef USE_HARDWARE_CRYPTO
#include "stm32_cryp.h"
#else
#include "mbedtls/aes.h"
#endif

typedef struct {
    uint8_t key[32];
    uint8_t iv[16];
    uint8_t key_size;  // 16, 24, or 32
} aes_context_t;

static mbedtls_aes_context aes_enc_ctx;
static mbedtls_aes_context aes_dec_ctx;

void aes_init(const uint8_t *key, uint8_t key_size, const uint8_t *iv) {
    mbedtls_aes_init(&aes_enc_ctx);
    mbedtls_aes_init(&aes_dec_ctx);
    
    mbedtls_aes_setkey_enc(&aes_enc_ctx, key, key_size * 8);
    mbedtls_aes_setkey_dec(&aes_dec_ctx, key, key_size * 8);
}

void aes_encrypt_cbc(const uint8_t *input, uint8_t *output, 
                     uint32_t length, uint8_t *iv) {
    mbedtls_aes_crypt_cbc(&aes_enc_ctx, MBEDTLS_AES_ENCRYPT,
                          length, iv, input, output);
}

void aes_decrypt_cbc(const uint8_t *input, uint8_t *output,
                     uint32_t length, uint8_t *iv) {
    mbedtls_aes_crypt_cbc(&aes_dec_ctx, MBEDTLS_AES_DECRYPT,
                          length, iv, input, output);
}

void aes_encrypt_ctr(const uint8_t *input, uint8_t *output,
                     uint32_t length, uint8_t *nonce_counter,
                     uint8_t *stream_block, size_t *nc_off) {
    mbedtls_aes_crypt_ctr(&aes_enc_ctx, length, nc_off,
                          nonce_counter, stream_block, input, output);
}

void aes_free(void) {
    mbedtls_aes_free(&aes_enc_ctx);
    mbedtls_aes_free(&aes_dec_ctx);
}

"#);
    }

    // Add Hash code
    if config.include_hash {
        let hash_size = match config.hash_algorithm {
            HashAlgorithm::SHA256 => 32,
            HashAlgorithm::SHA384 => 48,
            HashAlgorithm::SHA512 => 64,
            HashAlgorithm::SHA3_256 => 32,
        };
        
        code.push_str(&format!(r#"
// ============================================================================
// Hash Functions ({hash_algo:?})
// ============================================================================

#include "mbedtls/sha256.h"
#include "mbedtls/sha512.h"

#define HASH_SIZE {hash_size}

typedef struct {{
    mbedtls_sha256_context sha256;
    mbedtls_sha512_context sha512;
}} hash_context_t;

static hash_context_t hash_ctx;

void hash_init(void) {{
{hash_init}}}

void hash_update(const uint8_t *data, uint32_t length) {{
{hash_update}}}

void hash_finish(uint8_t *hash_out) {{
{hash_finish}}}

void hash_compute(const uint8_t *data, uint32_t length, uint8_t *hash_out) {{
    hash_init();
    hash_update(data, length);
    hash_finish(hash_out);
}}

bool hash_verify(const uint8_t *data, uint32_t length, const uint8_t *expected) {{
    uint8_t computed[HASH_SIZE];
    hash_compute(data, length, computed);
    return memcmp(computed, expected, HASH_SIZE) == 0;
}}

"#,
            hash_algo = config.hash_algorithm,
            hash_size = hash_size,
            hash_init = match config.hash_algorithm {
                HashAlgorithm::SHA256 | HashAlgorithm::SHA3_256 => 
                    "    mbedtls_sha256_init(&hash_ctx.sha256);\n    mbedtls_sha256_starts(&hash_ctx.sha256, 0);",
                HashAlgorithm::SHA384 =>
                    "    mbedtls_sha512_init(&hash_ctx.sha512);\n    mbedtls_sha512_starts(&hash_ctx.sha512, 1);",
                HashAlgorithm::SHA512 =>
                    "    mbedtls_sha512_init(&hash_ctx.sha512);\n    mbedtls_sha512_starts(&hash_ctx.sha512, 0);",
            },
            hash_update = match config.hash_algorithm {
                HashAlgorithm::SHA256 | HashAlgorithm::SHA3_256 => 
                    "    mbedtls_sha256_update(&hash_ctx.sha256, data, length);",
                HashAlgorithm::SHA384 | HashAlgorithm::SHA512 =>
                    "    mbedtls_sha512_update(&hash_ctx.sha512, data, length);",
            },
            hash_finish = match config.hash_algorithm {
                HashAlgorithm::SHA256 | HashAlgorithm::SHA3_256 => 
                    "    mbedtls_sha256_finish(&hash_ctx.sha256, hash_out);\n    mbedtls_sha256_free(&hash_ctx.sha256);",
                HashAlgorithm::SHA384 | HashAlgorithm::SHA512 =>
                    "    mbedtls_sha512_finish(&hash_ctx.sha512, hash_out);\n    mbedtls_sha512_free(&hash_ctx.sha512);",
            },
        ));
    }

    // Add RNG code
    if config.include_rng {
        code.push_str(r#"
// ============================================================================
// Random Number Generator
// ============================================================================

#ifdef USE_HARDWARE_CRYPTO
#include "stm32_rng.h"
#else
#include "mbedtls/entropy.h"
#include "mbedtls/ctr_drbg.h"
#endif

static mbedtls_entropy_context entropy;
static mbedtls_ctr_drbg_context ctr_drbg;
static bool rng_initialized = false;

bool rng_init(void) {
    mbedtls_entropy_init(&entropy);
    mbedtls_ctr_drbg_init(&ctr_drbg);
    
    const char *pers = "crypto_rng";
    int ret = mbedtls_ctr_drbg_seed(&ctr_drbg, mbedtls_entropy_func,
                                     &entropy, (uint8_t *)pers, strlen(pers));
    
    if (ret != 0) {
        return false;
    }
    
    rng_initialized = true;
    return true;
}

bool rng_get_bytes(uint8_t *output, uint32_t length) {
    if (!rng_initialized) {
        if (!rng_init()) return false;
    }
    
    return mbedtls_ctr_drbg_random(&ctr_drbg, output, length) == 0;
}

uint32_t rng_get_u32(void) {
    uint32_t value;
    rng_get_bytes((uint8_t *)&value, sizeof(value));
    return value;
}

void rng_free(void) {
    mbedtls_ctr_drbg_free(&ctr_drbg);
    mbedtls_entropy_free(&entropy);
    rng_initialized = false;
}

"#);
    }

    // Add ECDSA code
    if config.include_ecdsa {
        code.push_str(r#"
// ============================================================================
// ECDSA Signature Verification
// ============================================================================

#include "mbedtls/ecdsa.h"
#include "mbedtls/ecp.h"

typedef struct {
    mbedtls_ecdsa_context ecdsa;
    mbedtls_ecp_group grp;
} ecdsa_context_t;

static ecdsa_context_t ecdsa_ctx;

bool ecdsa_init(const uint8_t *public_key, uint32_t key_len) {
    mbedtls_ecdsa_init(&ecdsa_ctx.ecdsa);
    mbedtls_ecp_group_init(&ecdsa_ctx.grp);
    
    // Load SECP256R1 curve
    mbedtls_ecp_group_load(&ecdsa_ctx.grp, MBEDTLS_ECP_DP_SECP256R1);
    
    // Load public key
    int ret = mbedtls_ecp_point_read_binary(&ecdsa_ctx.grp,
                                             &ecdsa_ctx.ecdsa.Q,
                                             public_key, key_len);
    return (ret == 0);
}

bool ecdsa_verify(const uint8_t *hash, uint32_t hash_len,
                  const uint8_t *signature, uint32_t sig_len) {
    mbedtls_mpi r, s;
    mbedtls_mpi_init(&r);
    mbedtls_mpi_init(&s);
    
    // Parse signature (r || s format)
    mbedtls_mpi_read_binary(&r, signature, sig_len / 2);
    mbedtls_mpi_read_binary(&s, signature + sig_len / 2, sig_len / 2);
    
    int ret = mbedtls_ecdsa_verify(&ecdsa_ctx.grp, hash, hash_len,
                                    &ecdsa_ctx.ecdsa.Q, &r, &s);
    
    mbedtls_mpi_free(&r);
    mbedtls_mpi_free(&s);
    
    return (ret == 0);
}

void ecdsa_free(void) {
    mbedtls_ecdsa_free(&ecdsa_ctx.ecdsa);
    mbedtls_ecp_group_free(&ecdsa_ctx.grp);
}

"#);
    }

    code
}
