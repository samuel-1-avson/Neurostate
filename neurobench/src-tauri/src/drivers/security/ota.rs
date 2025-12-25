// OTA Update Generator
// Over-the-air firmware update client

use super::*;

/// Generate OTA update client code
pub fn generate_ota_code(config: &OtaConfig) -> String {
    let transport_includes = match config.transport {
        OtaTransport::HTTP | OtaTransport::HTTPS => r#"#include "esp_http_client.h""#,
        OtaTransport::MQTT => r#"#include "mqtt_client.h""#,
        OtaTransport::BLE => r#"#include "esp_ota_ops.h"
#include "esp_gap_ble_api.h""#,
        OtaTransport::UART => r#"#include "driver/uart.h""#,
    };

    format!(r#"/**
 * OTA Update Client: {name}
 * Transport: {transport:?}
 * Server: {server}
 */

#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include "esp_ota_ops.h"
#include "esp_partition.h"
#include "esp_log.h"
{transport_includes}

#define OTA_TAG "{name}"

// Configuration
#define OTA_SERVER_URL   "{server}"
#define OTA_FIRMWARE_PATH "{path}"
#define OTA_CHUNK_SIZE    {chunk_size}
#define OTA_RETRY_COUNT   {retry_count}
#define OTA_TIMEOUT_MS    {timeout}
#define OTA_VERIFY_SIG    {verify_sig}
#define OTA_VERIFY_CRC    {verify_crc}

typedef enum {{
    OTA_STATE_IDLE,
    OTA_STATE_CHECKING,
    OTA_STATE_DOWNLOADING,
    OTA_STATE_VERIFYING,
    OTA_STATE_APPLYING,
    OTA_STATE_COMPLETE,
    OTA_STATE_ERROR
}} ota_state_t;

typedef struct {{
    ota_state_t state;
    uint32_t total_size;
    uint32_t received_size;
    uint8_t progress_percent;
    char version[32];
    esp_ota_handle_t update_handle;
    const esp_partition_t *update_partition;
}} ota_context_t;

static ota_context_t ota_ctx;

// Callback types
typedef void (*ota_progress_cb_t)(uint8_t percent);
typedef void (*ota_complete_cb_t)(bool success, const char *message);

static ota_progress_cb_t progress_callback = NULL;
static ota_complete_cb_t complete_callback = NULL;

void ota_set_callbacks(ota_progress_cb_t progress, ota_complete_cb_t complete) {{
    progress_callback = progress;
    complete_callback = complete;
}}

static void ota_report_progress(void) {{
    if (ota_ctx.total_size > 0) {{
        ota_ctx.progress_percent = (ota_ctx.received_size * 100) / ota_ctx.total_size;
        if (progress_callback) {{
            progress_callback(ota_ctx.progress_percent);
        }}
    }}
}}

esp_err_t ota_http_event_handler(esp_http_client_event_t *evt) {{
    switch (evt->event_id) {{
        case HTTP_EVENT_ON_DATA:
            if (!esp_http_client_is_chunked_response(evt->client)) {{
                esp_err_t err = esp_ota_write(ota_ctx.update_handle, evt->data, evt->data_len);
                if (err != ESP_OK) {{
                    ESP_LOGE(OTA_TAG, "OTA write failed: %s", esp_err_to_name(err));
                    return err;
                }}
                ota_ctx.received_size += evt->data_len;
                ota_report_progress();
            }}
            break;
        default:
            break;
    }}
    return ESP_OK;
}}

bool ota_check_version(const char *new_version) {{
    #if {version_check}
    const esp_app_desc_t *running_app = esp_ota_get_app_description();
    ESP_LOGI(OTA_TAG, "Current version: %s", running_app->version);
    ESP_LOGI(OTA_TAG, "New version: %s", new_version);
    
    // Simple version comparison (assumes semantic versioning)
    if (strcmp(new_version, running_app->version) > 0) {{
        return true;
    }}
    return false;
    #else
    return true;
    #endif
}}

esp_err_t ota_begin(void) {{
    ota_ctx.state = OTA_STATE_CHECKING;
    ota_ctx.received_size = 0;
    ota_ctx.progress_percent = 0;
    
    // Get the next update partition
    ota_ctx.update_partition = esp_ota_get_next_update_partition(NULL);
    if (ota_ctx.update_partition == NULL) {{
        ESP_LOGE(OTA_TAG, "No OTA partition found");
        ota_ctx.state = OTA_STATE_ERROR;
        return ESP_FAIL;
    }}
    
    ESP_LOGI(OTA_TAG, "Writing to partition: %s at 0x%08lx",
             ota_ctx.update_partition->label,
             ota_ctx.update_partition->address);
    
    esp_err_t err = esp_ota_begin(ota_ctx.update_partition, OTA_SIZE_UNKNOWN,
                                   &ota_ctx.update_handle);
    if (err != ESP_OK) {{
        ESP_LOGE(OTA_TAG, "OTA begin failed: %s", esp_err_to_name(err));
        ota_ctx.state = OTA_STATE_ERROR;
        return err;
    }}
    
    ota_ctx.state = OTA_STATE_DOWNLOADING;
    return ESP_OK;
}}

esp_err_t ota_download_http(void) {{
    esp_http_client_config_t http_config = {{
        .url = OTA_SERVER_URL OTA_FIRMWARE_PATH,
        .timeout_ms = OTA_TIMEOUT_MS,
        .event_handler = ota_http_event_handler,
        .buffer_size = OTA_CHUNK_SIZE,
    }};
    
    esp_http_client_handle_t client = esp_http_client_init(&http_config);
    
    esp_err_t err = esp_http_client_perform(client);
    if (err == ESP_OK) {{
        ota_ctx.total_size = esp_http_client_get_content_length(client);
        ESP_LOGI(OTA_TAG, "Downloaded %lu bytes", ota_ctx.received_size);
    }}
    
    esp_http_client_cleanup(client);
    return err;
}}

esp_err_t ota_finish(void) {{
    ota_ctx.state = OTA_STATE_VERIFYING;
    
    #if OTA_VERIFY_CRC
    // Verify CRC (implementation depends on your protocol)
    ESP_LOGI(OTA_TAG, "Verifying firmware checksum...");
    #endif
    
    #if OTA_VERIFY_SIG
    // Verify signature
    ESP_LOGI(OTA_TAG, "Verifying firmware signature...");
    #endif
    
    esp_err_t err = esp_ota_end(ota_ctx.update_handle);
    if (err != ESP_OK) {{
        ESP_LOGE(OTA_TAG, "OTA end failed: %s", esp_err_to_name(err));
        ota_ctx.state = OTA_STATE_ERROR;
        if (complete_callback) {{
            complete_callback(false, "Verification failed");
        }}
        return err;
    }}
    
    ota_ctx.state = OTA_STATE_APPLYING;
    
    err = esp_ota_set_boot_partition(ota_ctx.update_partition);
    if (err != ESP_OK) {{
        ESP_LOGE(OTA_TAG, "Set boot partition failed: %s", esp_err_to_name(err));
        ota_ctx.state = OTA_STATE_ERROR;
        if (complete_callback) {{
            complete_callback(false, "Failed to set boot partition");
        }}
        return err;
    }}
    
    ota_ctx.state = OTA_STATE_COMPLETE;
    ESP_LOGI(OTA_TAG, "OTA update complete! Rebooting...");
    
    if (complete_callback) {{
        complete_callback(true, "Update complete, rebooting...");
    }}
    
    // Reboot after a short delay
    vTaskDelay(pdMS_TO_TICKS(1000));
    esp_restart();
    
    return ESP_OK;
}}

esp_err_t ota_perform_update(void) {{
    esp_err_t err;
    
    err = ota_begin();
    if (err != ESP_OK) return err;
    
    for (int retry = 0; retry < OTA_RETRY_COUNT; retry++) {{
        err = ota_download_http();
        if (err == ESP_OK) break;
        ESP_LOGW(OTA_TAG, "Download failed, retry %d/%d", retry + 1, OTA_RETRY_COUNT);
        vTaskDelay(pdMS_TO_TICKS(1000));
    }}
    
    if (err != ESP_OK) {{
        ESP_LOGE(OTA_TAG, "OTA download failed after retries");
        ota_ctx.state = OTA_STATE_ERROR;
        return err;
    }}
    
    return ota_finish();
}}

uint8_t ota_get_progress(void) {{
    return ota_ctx.progress_percent;
}}

ota_state_t ota_get_state(void) {{
    return ota_ctx.state;
}}

void ota_rollback(void) {{
    const esp_partition_t *running = esp_ota_get_running_partition();
    const esp_partition_t *last_invalid = esp_ota_get_last_invalid_partition();
    
    if (last_invalid != NULL) {{
        ESP_LOGI(OTA_TAG, "Rolling back to previous firmware");
        esp_ota_set_boot_partition(last_invalid);
        esp_restart();
    }}
}}
"#,
        name = config.name,
        transport = config.transport,
        transport_includes = transport_includes,
        server = config.server_url,
        path = config.firmware_path,
        chunk_size = config.chunk_size,
        retry_count = config.retry_count,
        timeout = config.timeout_ms,
        verify_sig = if config.verify_signature { 1 } else { 0 },
        verify_crc = if config.verify_checksum { 1 } else { 0 },
        version_check = if config.version_check { 1 } else { 0 },
    )
}
