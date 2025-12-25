// ESP32 HAL Implementation
// Supports ESP32, ESP32-S3, ESP32-C3

use super::*;

/// ESP32 HAL Implementation
pub struct Esp32Hal {
    pub family: McuFamily,
}

impl Esp32Hal {
    pub fn new(family: McuFamily) -> Self {
        Self { family }
    }
    
    fn idf_version(&self) -> &'static str {
        "5.1"  // ESP-IDF 5.1
    }
}

impl McuHal for Esp32Hal {
    fn family(&self) -> McuFamily {
        self.family
    }
    
    fn generate_gpio(&self, config: &GpioConfig) -> String {
        let pin_num: u32 = config.pin.trim_start_matches("GPIO").parse().unwrap_or(0);
        
        let mode_str = match config.mode {
            GpioMode::Input => "GPIO_MODE_INPUT",
            GpioMode::Output => "GPIO_MODE_OUTPUT",
            GpioMode::AlternateFunction(_) => "GPIO_MODE_OUTPUT",
            GpioMode::Analog => "GPIO_MODE_DISABLE",
        };
        
        let _pull_str = match config.pull {
            GpioPull::None => "GPIO_FLOATING",
            GpioPull::Up => "GPIO_PULLUP_ONLY",
            GpioPull::Down => "GPIO_PULLDOWN_ONLY",
        };

        format!(r#"/**
 * GPIO Configuration: GPIO{pin}
 * ESP-IDF {idf}
 */

#include "driver/gpio.h"

void gpio_{pin}_init(void) {{
    gpio_config_t io_conf = {{
        .pin_bit_mask = (1ULL << {pin}),
        .mode = {mode},
        .pull_up_en = {pull_up},
        .pull_down_en = {pull_down},
        .intr_type = GPIO_INTR_DISABLE,
    }};
    gpio_config(&io_conf);
{init_state}}}

void gpio_{pin}_set(bool level) {{
    gpio_set_level({pin}, level ? 1 : 0);
}}

int gpio_{pin}_get(void) {{
    return gpio_get_level({pin});
}}
"#,
            pin = pin_num,
            idf = self.idf_version(),
            mode = mode_str,
            pull_up = if matches!(config.pull, GpioPull::Up) { "GPIO_PULLUP_ENABLE" } else { "GPIO_PULLUP_DISABLE" },
            pull_down = if matches!(config.pull, GpioPull::Down) { "GPIO_PULLDOWN_ENABLE" } else { "GPIO_PULLDOWN_DISABLE" },
            init_state = if let Some(state) = config.initial_state {
                format!("\n    gpio_set_level({}, {});\n", pin_num, if state { 1 } else { 0 })
            } else {
                String::new()
            },
        )
    }
    
    fn generate_spi(&self, config: &SpiConfigAbstract) -> String {
        let host = match config.instance {
            1 => "SPI2_HOST",
            2 => "SPI3_HOST",
            _ => "SPI2_HOST",
        };

        format!(r#"/**
 * SPI Configuration: {host}
 * Clock: {clock} Hz
 */

#include "driver/spi_master.h"

static spi_device_handle_t spi_handle;

void spi{instance}_init(void) {{
    spi_bus_config_t buscfg = {{
        .miso_io_num = GPIO_NUM_19,
        .mosi_io_num = GPIO_NUM_23,
        .sclk_io_num = GPIO_NUM_18,
        .quadwp_io_num = -1,
        .quadhd_io_num = -1,
        .max_transfer_sz = 4096,
    }};
    
    spi_device_interface_config_t devcfg = {{
        .clock_speed_hz = {clock},
        .mode = {mode},
        .spics_io_num = GPIO_NUM_5,
        .queue_size = 7,
        .flags = {flags},
    }};
    
    ESP_ERROR_CHECK(spi_bus_initialize({host}, &buscfg, SPI_DMA_CH_AUTO));
    ESP_ERROR_CHECK(spi_bus_add_device({host}, &devcfg, &spi_handle));
}}

uint8_t spi{instance}_transfer(uint8_t data) {{
    spi_transaction_t t = {{
        .length = 8,
        .tx_buffer = &data,
        .rx_buffer = &data,
    }};
    spi_device_transmit(spi_handle, &t);
    return data;
}}
"#,
            host = host,
            instance = config.instance,
            clock = config.clock_hz,
            mode = config.mode,
            flags = if config.msb_first { "0" } else { "SPI_DEVICE_BIT_LSBFIRST" },
        )
    }
    
    fn generate_i2c(&self, config: &I2cConfigAbstract) -> String {
        let port = format!("I2C_NUM_{}", config.instance - 1);
        
        let speed_hz = match config.speed {
            I2cSpeedAbstract::Standard100k => 100000,
            I2cSpeedAbstract::Fast400k => 400000,
            I2cSpeedAbstract::FastPlus1m => 1000000,
        };

        format!(r#"/**
 * I2C Configuration: {port}
 * Speed: {speed} Hz
 */

#include "driver/i2c.h"

void i2c{instance}_init(void) {{
    i2c_config_t conf = {{
        .mode = I2C_MODE_MASTER,
        .sda_io_num = GPIO_NUM_21,
        .scl_io_num = GPIO_NUM_22,
        .sda_pullup_en = GPIO_PULLUP_ENABLE,
        .scl_pullup_en = GPIO_PULLUP_ENABLE,
        .master.clk_speed = {speed},
    }};
    
    ESP_ERROR_CHECK(i2c_param_config({port}, &conf));
    ESP_ERROR_CHECK(i2c_driver_install({port}, I2C_MODE_MASTER, 0, 0, 0));
}}

esp_err_t i2c{instance}_write(uint8_t addr, uint8_t reg, uint8_t *data, size_t len) {{
    i2c_cmd_handle_t cmd = i2c_cmd_link_create();
    i2c_master_start(cmd);
    i2c_master_write_byte(cmd, (addr << 1) | I2C_MASTER_WRITE, true);
    i2c_master_write_byte(cmd, reg, true);
    i2c_master_write(cmd, data, len, true);
    i2c_master_stop(cmd);
    esp_err_t ret = i2c_master_cmd_begin({port}, cmd, pdMS_TO_TICKS(100));
    i2c_cmd_link_delete(cmd);
    return ret;
}}

esp_err_t i2c{instance}_read(uint8_t addr, uint8_t reg, uint8_t *data, size_t len) {{
    i2c_cmd_handle_t cmd = i2c_cmd_link_create();
    i2c_master_start(cmd);
    i2c_master_write_byte(cmd, (addr << 1) | I2C_MASTER_WRITE, true);
    i2c_master_write_byte(cmd, reg, true);
    i2c_master_start(cmd);
    i2c_master_write_byte(cmd, (addr << 1) | I2C_MASTER_READ, true);
    i2c_master_read(cmd, data, len, I2C_MASTER_LAST_NACK);
    i2c_master_stop(cmd);
    esp_err_t ret = i2c_master_cmd_begin({port}, cmd, pdMS_TO_TICKS(100));
    i2c_cmd_link_delete(cmd);
    return ret;
}}
"#,
            port = port,
            instance = config.instance,
            speed = speed_hz,
        )
    }
    
    fn generate_uart(&self, config: &UartConfigAbstract) -> String {
        let port = format!("UART_NUM_{}", config.instance);
        
        let parity_str = match config.parity {
            UartParity::None => "UART_PARITY_DISABLE",
            UartParity::Even => "UART_PARITY_EVEN",
            UartParity::Odd => "UART_PARITY_ODD",
        };

        format!(r#"/**
 * UART Configuration: {port}
 * Baud: {baud}
 */

#include "driver/uart.h"

#define UART{instance}_TX_PIN  GPIO_NUM_17
#define UART{instance}_RX_PIN  GPIO_NUM_16
#define UART{instance}_BUF_SIZE 1024

void uart{instance}_init(void) {{
    uart_config_t uart_config = {{
        .baud_rate = {baud},
        .data_bits = {data_bits},
        .parity = {parity},
        .stop_bits = {stop_bits},
        .flow_ctrl = {flow},
        .source_clk = UART_SCLK_DEFAULT,
    }};
    
    ESP_ERROR_CHECK(uart_driver_install({port}, UART{instance}_BUF_SIZE, 0, 0, NULL, 0));
    ESP_ERROR_CHECK(uart_param_config({port}, &uart_config));
    ESP_ERROR_CHECK(uart_set_pin({port}, UART{instance}_TX_PIN, UART{instance}_RX_PIN, 
                                 UART_PIN_NO_CHANGE, UART_PIN_NO_CHANGE));
}}

int uart{instance}_write(const char *data, size_t len) {{
    return uart_write_bytes({port}, data, len);
}}

int uart{instance}_read(char *data, size_t max_len, uint32_t timeout_ms) {{
    return uart_read_bytes({port}, data, max_len, pdMS_TO_TICKS(timeout_ms));
}}
"#,
            port = port,
            instance = config.instance,
            baud = config.baud_rate,
            data_bits = if config.data_bits == 9 { "UART_DATA_9_BITS" } else { "UART_DATA_8_BITS" },
            parity = parity_str,
            stop_bits = if config.stop_bits == 2 { "UART_STOP_BITS_2" } else { "UART_STOP_BITS_1" },
            flow = if config.flow_control { "UART_HW_FLOWCTRL_CTS_RTS" } else { "UART_HW_FLOWCTRL_DISABLE" },
        )
    }
    
    fn generate_timer(&self, config: &TimerConfigAbstract) -> String {
        format!(r#"/**
 * Timer Configuration using ESP-IDF High Resolution Timer
 * Frequency: {freq} Hz
 */

#include "esp_timer.h"

static esp_timer_handle_t timer{instance}_handle;

static void timer{instance}_callback(void* arg) {{
    // Timer callback - runs at {freq} Hz
}}

void timer{instance}_init(void) {{
    esp_timer_create_args_t timer_args = {{
        .callback = timer{instance}_callback,
        .arg = NULL,
        .dispatch_method = ESP_TIMER_TASK,
        .name = "timer{instance}",
    }};
    
    ESP_ERROR_CHECK(esp_timer_create(&timer_args, &timer{instance}_handle));
}}

void timer{instance}_start(void) {{
    // Period in microseconds
    uint64_t period_us = 1000000 / {freq};
    ESP_ERROR_CHECK(esp_timer_start_periodic(timer{instance}_handle, period_us));
}}

void timer{instance}_stop(void) {{
    ESP_ERROR_CHECK(esp_timer_stop(timer{instance}_handle));
}}
"#,
            instance = config.instance,
            freq = config.frequency_hz,
        )
    }
    
    fn generate_adc(&self, config: &AdcConfigAbstract) -> String {
        format!(r#"/**
 * ADC Configuration
 * Resolution: {bits}-bit
 */

#include "esp_adc/adc_oneshot.h"
#include "esp_adc/adc_cali.h"

static adc_oneshot_unit_handle_t adc{instance}_handle;

void adc{instance}_init(void) {{
    adc_oneshot_unit_init_cfg_t init_config = {{
        .unit_id = ADC_UNIT_{instance},
    }};
    ESP_ERROR_CHECK(adc_oneshot_new_unit(&init_config, &adc{instance}_handle));
    
    adc_oneshot_chan_cfg_t chan_config = {{
        .atten = ADC_ATTEN_DB_11,
        .bitwidth = {bitwidth},
    }};
    
    // Configure channels
{channel_config}}}

int adc{instance}_read(int channel) {{
    int raw;
    ESP_ERROR_CHECK(adc_oneshot_read(adc{instance}_handle, channel, &raw));
    return raw;
}}

int adc{instance}_read_mv(int channel) {{
    int raw = adc{instance}_read(channel);
    // Approximate conversion (3.3V reference, 11dB attenuation)
    return (raw * 3300) / {max_val};
}}
"#,
            instance = config.instance,
            bits = config.resolution_bits,
            bitwidth = match config.resolution_bits {
                9 => "ADC_BITWIDTH_9",
                10 => "ADC_BITWIDTH_10",
                11 => "ADC_BITWIDTH_11",
                _ => "ADC_BITWIDTH_12",
            },
            max_val = (1 << config.resolution_bits) - 1,
            channel_config = config.channels.iter()
                .map(|ch| format!("    adc_oneshot_config_channel(adc{}_handle, ADC_CHANNEL_{}, &chan_config);\n", config.instance, ch))
                .collect::<String>(),
        )
    }
    
    fn generate_clock_init(&self, _freq_mhz: u32) -> String {
        format!(r#"/**
 * ESP32 Clock Configuration
 * Note: Clock is configured in sdkconfig
 */

// ESP32 uses sdkconfig for clock configuration
// Typical settings in menuconfig:
// - CONFIG_ESP_DEFAULT_CPU_FREQ_MHZ=240
// - CONFIG_XTAL_FREQ_40=y
"#)
    }
    
    fn generate_system_init(&self) -> String {
        format!(r#"/**
 * ESP32 Application Entry Point
 */

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_log.h"

static const char *TAG = "app_main";

void app_main(void) {{
    ESP_LOGI(TAG, "Application starting...");
    
    // Initialize peripherals
    
    // Main loop (or create tasks)
    while (1) {{
        vTaskDelay(pdMS_TO_TICKS(1000));
    }}
}}
"#)
    }
    
    fn include_headers(&self) -> Vec<&'static str> {
        vec![
            "freertos/FreeRTOS.h",
            "freertos/task.h",
            "driver/gpio.h",
            "esp_log.h",
        ]
    }
    
    fn linker_script(&self) -> &'static str {
        "esp32.ld"  // ESP-IDF handles this automatically
    }
    
    fn startup_file(&self) -> &'static str {
        "startup.c"  // ESP-IDF handles this automatically
    }
}
