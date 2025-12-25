// BLE GATT Code Generator
// Generates BLE peripheral/central code for nRF52 and ESP32

use super::*;

/// Generate BLE GATT service code for nRF52 (Nordic SDK)
pub fn generate_nrf52_ble(config: &BleConfig) -> String {
    let mut code = format!(r#"/**
 * BLE Configuration: {name}
 * Role: {role:?}
 * Services: {num_services}
 */

#include "ble.h"
#include "ble_srv_common.h"
#include "nrf_sdh.h"
#include "nrf_sdh_ble.h"
#include "app_error.h"

#define DEVICE_NAME "{name}"
#define MIN_CONN_INTERVAL MSEC_TO_UNITS({conn_interval}, UNIT_1_25_MS)
#define MAX_CONN_INTERVAL MSEC_TO_UNITS({conn_interval}, UNIT_1_25_MS)
#define SLAVE_LATENCY 0
#define CONN_SUP_TIMEOUT MSEC_TO_UNITS(4000, UNIT_10_MS)

"#,
        name = config.device_name,
        role = config.role,
        num_services = config.services.len(),
        conn_interval = config.connection_interval_ms,
    );

    // Generate service/characteristic UUIDs and handles
    for service in &config.services {
        let service_upper = service.name.to_uppercase().replace(" ", "_");
        
        code.push_str(&format!(r#"// Service: {name}
#define {upper}_UUID_BASE {{{uuid_bytes}}}
#define {upper}_UUID_SERVICE 0x0001

typedef struct {{
    uint16_t service_handle;
    ble_gatts_char_handles_t char_handles[{num_chars}];
}} {lower}_t;

static {lower}_t m_{lower};

"#,
            name = service.name,
            upper = service_upper,
            lower = service.name.to_lowercase().replace(" ", "_"),
            uuid_bytes = format_uuid_bytes(&service.uuid),
            num_chars = service.characteristics.len(),
        ));

        // Generate characteristic definitions
        for (i, char) in service.characteristics.iter().enumerate() {
            code.push_str(&format!(r#"#define {upper}_CHAR_{i}_UUID 0x{char_uuid}
"#,
                upper = service_upper,
                i = i,
                char_uuid = &char.uuid[..4],
            ));
        }
        code.push('\n');
    }

    // Generate init function
    code.push_str(r#"static void gap_params_init(void) {
    ble_gap_conn_sec_mode_t sec_mode;
    BLE_GAP_CONN_SEC_MODE_SET_OPEN(&sec_mode);
    
    sd_ble_gap_device_name_set(&sec_mode, (uint8_t *)DEVICE_NAME, strlen(DEVICE_NAME));
    
    ble_gap_conn_params_t gap_conn_params = {
        .min_conn_interval = MIN_CONN_INTERVAL,
        .max_conn_interval = MAX_CONN_INTERVAL,
        .slave_latency = SLAVE_LATENCY,
        .conn_sup_timeout = CONN_SUP_TIMEOUT,
    };
    sd_ble_gap_ppcp_set(&gap_conn_params);
}

static void advertising_init(void) {
    ble_advdata_t advdata;
    ble_advdata_t scanrsp;
    
    memset(&advdata, 0, sizeof(advdata));
    advdata.name_type = BLE_ADVDATA_FULL_NAME;
    advdata.include_appearance = true;
    advdata.flags = BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE;
    
    memset(&scanrsp, 0, sizeof(scanrsp));
    
    ble_advertising_init_t init;
    memset(&init, 0, sizeof(init));
    init.advdata = advdata;
    init.srdata = scanrsp;
    
    ble_advertising_init(&init);
}

"#);

    // Generate service init functions
    for service in &config.services {
        let lower = service.name.to_lowercase().replace(" ", "_");
        let upper = service.name.to_uppercase().replace(" ", "_");
        
        code.push_str(&format!(r#"static uint32_t {lower}_init(void) {{
    uint32_t err_code;
    ble_uuid_t ble_uuid;
    ble_uuid128_t base_uuid = {upper}_UUID_BASE;
    
    err_code = sd_ble_uuid_vs_add(&base_uuid, &ble_uuid.type);
    VERIFY_SUCCESS(err_code);
    
    ble_uuid.uuid = {upper}_UUID_SERVICE;
    
    err_code = sd_ble_gatts_service_add(BLE_GATTS_SRVC_TYPE_PRIMARY, 
                                        &ble_uuid, 
                                        &m_{lower}.service_handle);
    VERIFY_SUCCESS(err_code);
    
"#,
            lower = lower,
            upper = upper,
        ));

        // Add characteristics
        for (i, char) in service.characteristics.iter().enumerate() {
            let props = &char.properties;
            code.push_str(&format!(r#"    // Characteristic: {char_name}
    {{
        ble_gatts_char_md_t char_md;
        ble_gatts_attr_t attr_char_value;
        ble_uuid_t char_uuid;
        ble_gatts_attr_md_t attr_md;
        
        memset(&char_md, 0, sizeof(char_md));
        char_md.char_props.read = {read};
        char_md.char_props.write = {write};
        char_md.char_props.write_wo_resp = {write_no_resp};
        char_md.char_props.notify = {notify};
        char_md.char_props.indicate = {indicate};
        
        memset(&attr_md, 0, sizeof(attr_md));
        BLE_GAP_CONN_SEC_MODE_SET_OPEN(&attr_md.read_perm);
        BLE_GAP_CONN_SEC_MODE_SET_OPEN(&attr_md.write_perm);
        attr_md.vloc = BLE_GATTS_VLOC_STACK;
        
        char_uuid.type = ble_uuid.type;
        char_uuid.uuid = {upper}_CHAR_{i}_UUID;
        
        memset(&attr_char_value, 0, sizeof(attr_char_value));
        attr_char_value.p_uuid = &char_uuid;
        attr_char_value.p_attr_md = &attr_md;
        attr_char_value.max_len = {max_len};
        
        err_code = sd_ble_gatts_characteristic_add(m_{lower}.service_handle,
                                                   &char_md,
                                                   &attr_char_value,
                                                   &m_{lower}.char_handles[{i}]);
        VERIFY_SUCCESS(err_code);
    }}
    
"#,
                char_name = char.name,
                upper = upper,
                lower = lower,
                i = i,
                read = if props.read { 1 } else { 0 },
                write = if props.write { 1 } else { 0 },
                write_no_resp = if props.write_no_response { 1 } else { 0 },
                notify = if props.notify { 1 } else { 0 },
                indicate = if props.indicate { 1 } else { 0 },
                max_len = char.max_length,
            ));
        }
        
        code.push_str("    return NRF_SUCCESS;\n}\n\n");
    }

    // Main BLE init
    code.push_str(r#"void ble_stack_init(void) {
    ret_code_t err_code;
    
    err_code = nrf_sdh_enable_request();
    APP_ERROR_CHECK(err_code);
    
    uint32_t ram_start = 0;
    err_code = nrf_sdh_ble_default_cfg_set(1, &ram_start);
    APP_ERROR_CHECK(err_code);
    
    err_code = nrf_sdh_ble_enable(&ram_start);
    APP_ERROR_CHECK(err_code);
}

void ble_init(void) {
    ble_stack_init();
    gap_params_init();
"#);

    for service in &config.services {
        let lower = service.name.to_lowercase().replace(" ", "_");
        code.push_str(&format!("    {}_init();\n", lower));
    }
    
    code.push_str(r#"    advertising_init();
}

void ble_start_advertising(void) {
    ble_advertising_start(BLE_ADV_MODE_FAST);
}
"#);

    code
}

/// Generate BLE code for ESP32 (ESP-IDF)
pub fn generate_esp32_ble(config: &BleConfig) -> String {
    let mut code = format!(r#"/**
 * BLE Configuration: {name}
 * ESP-IDF BLE GATT Server
 */

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_bt.h"
#include "esp_gap_ble_api.h"
#include "esp_gatts_api.h"
#include "esp_bt_main.h"
#include "esp_log.h"

#define DEVICE_NAME "{name}"
#define GATTS_TAG "GATTS"

static uint8_t adv_config_done = 0;
static uint16_t gatts_if = ESP_GATT_IF_NONE;
static uint16_t conn_id = 0;

"#,
        name = config.device_name,
    );

    // Generate service/characteristic handles
    for (si, service) in config.services.iter().enumerate() {
        code.push_str(&format!(r#"// Service {si}: {name}
#define SERVICE_{si}_UUID 0x{uuid}
static uint16_t service_{si}_handle;
"#,
            si = si,
            name = service.name,
            uuid = &service.uuid[..4],
        ));

        for (ci, char) in service.characteristics.iter().enumerate() {
            code.push_str(&format!(r#"#define CHAR_{si}_{ci}_UUID 0x{uuid}
static uint16_t char_{si}_{ci}_handle;
static uint16_t char_{si}_{ci}_descr_handle;
"#,
                si = si,
                ci = ci,
                uuid = &char.uuid[..4],
            ));
        }
        code.push('\n');
    }

    // Advertising data
    code.push_str(r#"
static esp_ble_adv_params_t adv_params = {
    .adv_int_min = 0x20,
    .adv_int_max = 0x40,
    .adv_type = ADV_TYPE_IND,
    .own_addr_type = BLE_ADDR_TYPE_PUBLIC,
    .channel_map = ADV_CHNL_ALL,
    .adv_filter_policy = ADV_FILTER_ALLOW_SCAN_ANY_CON_ANY,
};

static esp_ble_adv_data_t adv_data = {
    .set_scan_rsp = false,
    .include_name = true,
    .include_txpower = true,
    .min_interval = 0x0006,
    .max_interval = 0x0010,
    .appearance = 0x00,
    .manufacturer_len = 0,
    .p_manufacturer_data = NULL,
    .service_data_len = 0,
    .p_service_data = NULL,
    .service_uuid_len = 0,
    .p_service_uuid = NULL,
    .flag = (ESP_BLE_ADV_FLAG_GEN_DISC | ESP_BLE_ADV_FLAG_BREDR_NOT_SPT),
};

"#);

    // GATTS event handler
    code.push_str(r#"static void gatts_event_handler(esp_gatts_cb_event_t event, esp_gatt_if_t gatts_if_param,
                                 esp_ble_gatts_cb_param_t *param) {
    switch (event) {
        case ESP_GATTS_REG_EVT:
            ESP_LOGI(GATTS_TAG, "REGISTER_APP_EVT");
            gatts_if = gatts_if_param;
            esp_ble_gap_set_device_name(DEVICE_NAME);
            esp_ble_gap_config_adv_data(&adv_data);
            // Add services here
            break;
            
        case ESP_GATTS_CONNECT_EVT:
            ESP_LOGI(GATTS_TAG, "CONNECT_EVT, conn_id %d", param->connect.conn_id);
            conn_id = param->connect.conn_id;
            break;
            
        case ESP_GATTS_DISCONNECT_EVT:
            ESP_LOGI(GATTS_TAG, "DISCONNECT_EVT");
            esp_ble_gap_start_advertising(&adv_params);
            break;
            
        case ESP_GATTS_WRITE_EVT:
            ESP_LOGI(GATTS_TAG, "WRITE_EVT, handle = %d", param->write.handle);
            // Handle write events
            break;
            
        case ESP_GATTS_READ_EVT:
            ESP_LOGI(GATTS_TAG, "READ_EVT, handle = %d", param->read.handle);
            break;
            
        default:
            break;
    }
}

static void gap_event_handler(esp_gap_ble_cb_event_t event, esp_ble_gap_cb_param_t *param) {
    switch (event) {
        case ESP_GAP_BLE_ADV_DATA_SET_COMPLETE_EVT:
            adv_config_done |= 0x01;
            if (adv_config_done == 0x03) {
                esp_ble_gap_start_advertising(&adv_params);
            }
            break;
            
        case ESP_GAP_BLE_SCAN_RSP_DATA_SET_COMPLETE_EVT:
            adv_config_done |= 0x02;
            if (adv_config_done == 0x03) {
                esp_ble_gap_start_advertising(&adv_params);
            }
            break;
            
        default:
            break;
    }
}

void ble_init(void) {
    esp_err_t ret;
    
    ESP_ERROR_CHECK(esp_bt_controller_mem_release(ESP_BT_MODE_CLASSIC_BT));
    
    esp_bt_controller_config_t bt_cfg = BT_CONTROLLER_INIT_CONFIG_DEFAULT();
    ret = esp_bt_controller_init(&bt_cfg);
    if (ret) {
        ESP_LOGE(GATTS_TAG, "controller init failed: %s", esp_err_to_name(ret));
        return;
    }
    
    ret = esp_bt_controller_enable(ESP_BT_MODE_BLE);
    if (ret) {
        ESP_LOGE(GATTS_TAG, "controller enable failed: %s", esp_err_to_name(ret));
        return;
    }
    
    ret = esp_bluedroid_init();
    if (ret) {
        ESP_LOGE(GATTS_TAG, "bluedroid init failed: %s", esp_err_to_name(ret));
        return;
    }
    
    ret = esp_bluedroid_enable();
    if (ret) {
        ESP_LOGE(GATTS_TAG, "bluedroid enable failed: %s", esp_err_to_name(ret));
        return;
    }
    
    ret = esp_ble_gatts_register_callback(gatts_event_handler);
    if (ret) {
        ESP_LOGE(GATTS_TAG, "gatts register callback failed: %s", esp_err_to_name(ret));
        return;
    }
    
    ret = esp_ble_gap_register_callback(gap_event_handler);
    if (ret) {
        ESP_LOGE(GATTS_TAG, "gap register callback failed: %s", esp_err_to_name(ret));
        return;
    }
    
    ret = esp_ble_gatts_app_register(0);
    if (ret) {
        ESP_LOGE(GATTS_TAG, "gatts app register failed: %s", esp_err_to_name(ret));
        return;
    }
    
    esp_ble_gatt_set_local_mtu(500);
    ESP_LOGI(GATTS_TAG, "BLE initialized successfully");
}
"#);

    code
}

fn format_uuid_bytes(uuid: &str) -> String {
    // Convert UUID string to byte array format
    let cleaned: String = uuid.chars().filter(|c| c.is_alphanumeric()).collect();
    if cleaned.len() >= 32 {
        let bytes: Vec<String> = (0..16)
            .map(|i| format!("0x{}", &cleaned[i*2..i*2+2]))
            .collect();
        bytes.join(", ")
    } else {
        "0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00".to_string()
    }
}
