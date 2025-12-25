// WiFi Code Generator
// Generates WiFi station/AP code for ESP32

use super::*;

/// Generate ESP32 WiFi Station code
pub fn generate_esp32_wifi_station(config: &WifiConfig) -> String {
    format!(r#"/**
 * WiFi Station Configuration
 * SSID: {ssid}
 */

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "freertos/event_groups.h"
#include "esp_system.h"
#include "esp_wifi.h"
#include "esp_event.h"
#include "esp_log.h"
#include "nvs_flash.h"
#include "lwip/err.h"
#include "lwip/sys.h"

#define WIFI_SSID      "{ssid}"
#define WIFI_PASSWORD  "{password}"
#define MAX_RETRY      5

static const char *TAG = "wifi_station";
static EventGroupHandle_t s_wifi_event_group;
static int s_retry_num = 0;

#define WIFI_CONNECTED_BIT BIT0
#define WIFI_FAIL_BIT      BIT1

static void event_handler(void* arg, esp_event_base_t event_base,
                          int32_t event_id, void* event_data) {{
    if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_START) {{
        esp_wifi_connect();
    }} else if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_DISCONNECTED) {{
        if (s_retry_num < MAX_RETRY) {{
            esp_wifi_connect();
            s_retry_num++;
            ESP_LOGI(TAG, "Retry connecting to AP");
        }} else {{
            xEventGroupSetBits(s_wifi_event_group, WIFI_FAIL_BIT);
        }}
        ESP_LOGI(TAG, "Connect to AP failed");
    }} else if (event_base == IP_EVENT && event_id == IP_EVENT_STA_GOT_IP) {{
        ip_event_got_ip_t* event = (ip_event_got_ip_t*) event_data;
        ESP_LOGI(TAG, "Got IP: " IPSTR, IP2STR(&event->ip_info.ip));
        s_retry_num = 0;
        xEventGroupSetBits(s_wifi_event_group, WIFI_CONNECTED_BIT);
    }}
}}

void wifi_init_sta(void) {{
    s_wifi_event_group = xEventGroupCreate();
    
    ESP_ERROR_CHECK(esp_netif_init());
    ESP_ERROR_CHECK(esp_event_loop_create_default());
    esp_netif_create_default_wifi_sta();
    
    wifi_init_config_t cfg = WIFI_INIT_CONFIG_DEFAULT();
    ESP_ERROR_CHECK(esp_wifi_init(&cfg));
    
    esp_event_handler_instance_t instance_any_id;
    esp_event_handler_instance_t instance_got_ip;
    ESP_ERROR_CHECK(esp_event_handler_instance_register(WIFI_EVENT,
                                                        ESP_EVENT_ANY_ID,
                                                        &event_handler,
                                                        NULL,
                                                        &instance_any_id));
    ESP_ERROR_CHECK(esp_event_handler_instance_register(IP_EVENT,
                                                        IP_EVENT_STA_GOT_IP,
                                                        &event_handler,
                                                        NULL,
                                                        &instance_got_ip));
    
    wifi_config_t wifi_config = {{
        .sta = {{
            .ssid = WIFI_SSID,
            .password = WIFI_PASSWORD,
            .threshold.authmode = {auth_mode},
            .sae_pwe_h2e = WPA3_SAE_PWE_BOTH,
        }},
    }};
    
    ESP_ERROR_CHECK(esp_wifi_set_mode(WIFI_MODE_STA));
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, &wifi_config));
    ESP_ERROR_CHECK(esp_wifi_start());
    
    ESP_LOGI(TAG, "wifi_init_sta finished");
    
    EventBits_t bits = xEventGroupWaitBits(s_wifi_event_group,
                                           WIFI_CONNECTED_BIT | WIFI_FAIL_BIT,
                                           pdFALSE, pdFALSE, portMAX_DELAY);
    
    if (bits & WIFI_CONNECTED_BIT) {{
        ESP_LOGI(TAG, "Connected to SSID: %s", WIFI_SSID);
    }} else if (bits & WIFI_FAIL_BIT) {{
        ESP_LOGI(TAG, "Failed to connect to SSID: %s", WIFI_SSID);
    }} else {{
        ESP_LOGE(TAG, "Unexpected event");
    }}
}}

bool wifi_is_connected(void) {{
    return (xEventGroupGetBits(s_wifi_event_group) & WIFI_CONNECTED_BIT) != 0;
}}
"#,
        ssid = config.ssid,
        password = config.password,
        auth_mode = match config.security {
            WifiSecurity::Open => "WIFI_AUTH_OPEN",
            WifiSecurity::WPA2Personal => "WIFI_AUTH_WPA2_PSK",
            WifiSecurity::WPA3Personal => "WIFI_AUTH_WPA3_PSK",
            WifiSecurity::WPA2Enterprise => "WIFI_AUTH_WPA2_ENTERPRISE",
        },
    )
}

/// Generate ESP32 WiFi Access Point code
pub fn generate_esp32_wifi_ap(config: &WifiConfig) -> String {
    format!(r#"/**
 * WiFi Access Point Configuration
 * SSID: {ssid}
 * Channel: {channel}
 */

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_system.h"
#include "esp_wifi.h"
#include "esp_event.h"
#include "esp_log.h"
#include "nvs_flash.h"
#include "lwip/err.h"
#include "lwip/sys.h"

#define WIFI_SSID      "{ssid}"
#define WIFI_PASSWORD  "{password}"
#define WIFI_CHANNEL   {channel}
#define MAX_STA_CONN   4

static const char *TAG = "wifi_ap";

static void wifi_event_handler(void* arg, esp_event_base_t event_base,
                               int32_t event_id, void* event_data) {{
    if (event_id == WIFI_EVENT_AP_STACONNECTED) {{
        wifi_event_ap_staconnected_t* event = (wifi_event_ap_staconnected_t*) event_data;
        ESP_LOGI(TAG, "Station " MACSTR " joined, AID=%d", MAC2STR(event->mac), event->aid);
    }} else if (event_id == WIFI_EVENT_AP_STADISCONNECTED) {{
        wifi_event_ap_stadisconnected_t* event = (wifi_event_ap_stadisconnected_t*) event_data;
        ESP_LOGI(TAG, "Station " MACSTR " left, AID=%d", MAC2STR(event->mac), event->aid);
    }}
}}

void wifi_init_softap(void) {{
    ESP_ERROR_CHECK(esp_netif_init());
    ESP_ERROR_CHECK(esp_event_loop_create_default());
    esp_netif_create_default_wifi_ap();
    
    wifi_init_config_t cfg = WIFI_INIT_CONFIG_DEFAULT();
    ESP_ERROR_CHECK(esp_wifi_init(&cfg));
    
    ESP_ERROR_CHECK(esp_event_handler_instance_register(WIFI_EVENT,
                                                        ESP_EVENT_ANY_ID,
                                                        &wifi_event_handler,
                                                        NULL,
                                                        NULL));
    
    wifi_config_t wifi_config = {{
        .ap = {{
            .ssid = WIFI_SSID,
            .ssid_len = strlen(WIFI_SSID),
            .channel = WIFI_CHANNEL,
            .password = WIFI_PASSWORD,
            .max_connection = MAX_STA_CONN,
            .authmode = {auth_mode},
        }},
    }};
    
    if (strlen(WIFI_PASSWORD) == 0) {{
        wifi_config.ap.authmode = WIFI_AUTH_OPEN;
    }}
    
    ESP_ERROR_CHECK(esp_wifi_set_mode(WIFI_MODE_AP));
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_AP, &wifi_config));
    ESP_ERROR_CHECK(esp_wifi_start());
    
    ESP_LOGI(TAG, "wifi_init_softap finished. SSID: %s, Channel: %d", WIFI_SSID, WIFI_CHANNEL);
}}

int wifi_get_connected_stations(void) {{
    wifi_sta_list_t wifi_sta_list;
    esp_wifi_ap_get_sta_list(&wifi_sta_list);
    return wifi_sta_list.num;
}}
"#,
        ssid = config.ssid,
        password = config.password,
        channel = config.channel,
        auth_mode = match config.security {
            WifiSecurity::Open => "WIFI_AUTH_OPEN",
            WifiSecurity::WPA2Personal => "WIFI_AUTH_WPA2_PSK",
            WifiSecurity::WPA3Personal => "WIFI_AUTH_WPA3_PSK",
            WifiSecurity::WPA2Enterprise => "WIFI_AUTH_WPA2_ENTERPRISE",
        },
    )
}

/// Generate WiFi configuration based on mode
pub fn generate_wifi_code(config: &WifiConfig) -> String {
    match config.mode {
        WifiMode::Station => generate_esp32_wifi_station(config),
        WifiMode::AccessPoint => generate_esp32_wifi_ap(config),
        WifiMode::StationAndAP => {
            let mut code = generate_esp32_wifi_station(config);
            code.push_str("\n// Note: For STA+AP mode, configure both interfaces\n");
            code
        }
    }
}
