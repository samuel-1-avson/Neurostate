// Nordic nRF52 HAL Implementation
// Supports nRF52832, nRF52840

use super::*;

/// Nordic nRF52 HAL Implementation
pub struct NordicHal {
    pub family: McuFamily,
}

impl NordicHal {
    pub fn new(family: McuFamily) -> Self {
        Self { family }
    }
    
    fn sdk_version(&self) -> &'static str {
        "17.1.0"  // nRF5 SDK
    }
}

impl McuHal for NordicHal {
    fn family(&self) -> McuFamily {
        self.family
    }
    
    fn generate_gpio(&self, config: &GpioConfig) -> String {
        let pin_num: u32 = config.pin.trim_start_matches("P0.").parse().unwrap_or(0);
        
        let dir_str = match config.mode {
            GpioMode::Input => "NRF_GPIO_PIN_DIR_INPUT",
            GpioMode::Output | GpioMode::AlternateFunction(_) => "NRF_GPIO_PIN_DIR_OUTPUT",
            GpioMode::Analog => "NRF_GPIO_PIN_DIR_INPUT",
        };
        
        let pull_str = match config.pull {
            GpioPull::None => "NRF_GPIO_PIN_NOPULL",
            GpioPull::Up => "NRF_GPIO_PIN_PULLUP",
            GpioPull::Down => "NRF_GPIO_PIN_PULLDOWN",
        };

        format!(r#"/**
 * GPIO Configuration: P0.{pin}
 * nRF5 SDK {sdk}
 */

#include "nrf_gpio.h"

void gpio_{pin}_init(void) {{
    nrf_gpio_cfg({pin},
                 {dir},
                 NRF_GPIO_PIN_INPUT_CONNECT,
                 {pull},
                 NRF_GPIO_PIN_S0S1,
                 NRF_GPIO_PIN_NOSENSE);
{init_state}}}

void gpio_{pin}_set(bool level) {{
    if (level) {{
        nrf_gpio_pin_set({pin});
    }} else {{
        nrf_gpio_pin_clear({pin});
    }}
}}

bool gpio_{pin}_get(void) {{
    return nrf_gpio_pin_read({pin}) != 0;
}}

void gpio_{pin}_toggle(void) {{
    nrf_gpio_pin_toggle({pin});
}}
"#,
            pin = pin_num,
            sdk = self.sdk_version(),
            dir = dir_str,
            pull = pull_str,
            init_state = if let Some(state) = config.initial_state {
                if state {
                    format!("    nrf_gpio_pin_set({});\n", pin_num)
                } else {
                    format!("    nrf_gpio_pin_clear({});\n", pin_num)
                }
            } else {
                String::new()
            },
        )
    }
    
    fn generate_spi(&self, config: &SpiConfigAbstract) -> String {
        format!(r#"/**
 * SPI Configuration: SPIM{instance}
 * Clock: {clock} Hz
 */

#include "nrf_drv_spi.h"

static const nrf_drv_spi_t spi{instance} = NRF_DRV_SPI_INSTANCE({instance});
static volatile bool spi{instance}_xfer_done;

static void spi{instance}_event_handler(nrf_drv_spi_evt_t const *p_event, void *p_context) {{
    spi{instance}_xfer_done = true;
}}

void spi{instance}_init(void) {{
    nrf_drv_spi_config_t spi_config = NRF_DRV_SPI_DEFAULT_CONFIG;
    spi_config.ss_pin   = NRF_DRV_SPI_PIN_NOT_USED;
    spi_config.miso_pin = 28;
    spi_config.mosi_pin = 29;
    spi_config.sck_pin  = 30;
    spi_config.frequency = {freq_enum};
    spi_config.mode = {mode_enum};
    
    APP_ERROR_CHECK(nrf_drv_spi_init(&spi{instance}, &spi_config, spi{instance}_event_handler, NULL));
}}

uint8_t spi{instance}_transfer(uint8_t tx_data) {{
    uint8_t rx_data;
    spi{instance}_xfer_done = false;
    APP_ERROR_CHECK(nrf_drv_spi_transfer(&spi{instance}, &tx_data, 1, &rx_data, 1));
    while (!spi{instance}_xfer_done) {{
        __WFE();
    }}
    return rx_data;
}}
"#,
            instance = config.instance,
            clock = config.clock_hz,
            freq_enum = match config.clock_hz {
                x if x >= 8_000_000 => "NRF_DRV_SPI_FREQ_8M",
                x if x >= 4_000_000 => "NRF_DRV_SPI_FREQ_4M",
                x if x >= 2_000_000 => "NRF_DRV_SPI_FREQ_2M",
                x if x >= 1_000_000 => "NRF_DRV_SPI_FREQ_1M",
                _ => "NRF_DRV_SPI_FREQ_500K",
            },
            mode_enum = match config.mode {
                0 => "NRF_DRV_SPI_MODE_0",
                1 => "NRF_DRV_SPI_MODE_1",
                2 => "NRF_DRV_SPI_MODE_2",
                3 => "NRF_DRV_SPI_MODE_3",
                _ => "NRF_DRV_SPI_MODE_0",
            },
        )
    }
    
    fn generate_i2c(&self, config: &I2cConfigAbstract) -> String {
        let speed_enum = match config.speed {
            I2cSpeedAbstract::Standard100k => "NRF_DRV_TWI_FREQ_100K",
            I2cSpeedAbstract::Fast400k => "NRF_DRV_TWI_FREQ_400K",
            I2cSpeedAbstract::FastPlus1m => "NRF_DRV_TWI_FREQ_400K",  // nRF52 max is 400k
        };

        format!(r#"/**
 * I2C (TWI) Configuration: TWIM{instance}
 * Speed: {speed:?}
 */

#include "nrf_drv_twi.h"

static const nrf_drv_twi_t twi{instance} = NRF_DRV_TWI_INSTANCE({instance});

static void twi{instance}_event_handler(nrf_drv_twi_evt_t const *p_event, void *p_context) {{
    // Handle TWI events if needed
}}

void twi{instance}_init(void) {{
    nrf_drv_twi_config_t twi_config = NRF_DRV_TWI_DEFAULT_CONFIG;
    twi_config.scl = 26;
    twi_config.sda = 27;
    twi_config.frequency = {freq};
    
    APP_ERROR_CHECK(nrf_drv_twi_init(&twi{instance}, &twi_config, twi{instance}_event_handler, NULL));
    nrf_drv_twi_enable(&twi{instance});
}}

ret_code_t twi{instance}_write(uint8_t addr, uint8_t reg, const uint8_t *data, uint8_t len) {{
    uint8_t buf[len + 1];
    buf[0] = reg;
    memcpy(&buf[1], data, len);
    return nrf_drv_twi_tx(&twi{instance}, addr, buf, len + 1, false);
}}

ret_code_t twi{instance}_read(uint8_t addr, uint8_t reg, uint8_t *data, uint8_t len) {{
    ret_code_t err = nrf_drv_twi_tx(&twi{instance}, addr, &reg, 1, true);
    if (err != NRF_SUCCESS) return err;
    return nrf_drv_twi_rx(&twi{instance}, addr, data, len);
}}
"#,
            instance = config.instance,
            speed = config.speed,
            freq = speed_enum,
        )
    }
    
    fn generate_uart(&self, config: &UartConfigAbstract) -> String {
        format!(r#"/**
 * UART Configuration
 * Baud: {baud}
 */

#include "nrf_drv_uart.h"

static nrf_drv_uart_t uart{instance} = NRF_DRV_UART_INSTANCE({instance});

static void uart{instance}_event_handler(nrf_drv_uart_event_t *p_event, void *p_context) {{
    // Handle UART events
}}

void uart{instance}_init(void) {{
    nrf_drv_uart_config_t config = NRF_DRV_UART_DEFAULT_CONFIG;
    config.pseltxd = 6;
    config.pselrxd = 8;
    config.baudrate = {baud_enum};
    config.hwfc = {hwfc};
    config.parity = {parity};
    
    APP_ERROR_CHECK(nrf_drv_uart_init(&uart{instance}, &config, uart{instance}_event_handler));
}}

void uart{instance}_putc(char c) {{
    nrf_drv_uart_tx(&uart{instance}, (uint8_t*)&c, 1);
}}

void uart{instance}_puts(const char *str) {{
    nrf_drv_uart_tx(&uart{instance}, (uint8_t*)str, strlen(str));
}}
"#,
            instance = config.instance,
            baud = config.baud_rate,
            baud_enum = match config.baud_rate {
                9600 => "NRF_UART_BAUDRATE_9600",
                19200 => "NRF_UART_BAUDRATE_19200",
                38400 => "NRF_UART_BAUDRATE_38400",
                57600 => "NRF_UART_BAUDRATE_57600",
                115200 => "NRF_UART_BAUDRATE_115200",
                230400 => "NRF_UART_BAUDRATE_230400",
                460800 => "NRF_UART_BAUDRATE_460800",
                921600 => "NRF_UART_BAUDRATE_921600",
                _ => "NRF_UART_BAUDRATE_115200",
            },
            hwfc = if config.flow_control { "NRF_UART_HWFC_ENABLED" } else { "NRF_UART_HWFC_DISABLED" },
            parity = match config.parity {
                UartParity::None => "NRF_UART_PARITY_EXCLUDED",
                UartParity::Even | UartParity::Odd => "NRF_UART_PARITY_INCLUDED",
            },
        )
    }
    
    fn generate_timer(&self, config: &TimerConfigAbstract) -> String {
        format!(r#"/**
 * Timer Configuration: TIMER{instance}
 * Frequency: {freq} Hz
 */

#include "nrf_drv_timer.h"

static const nrf_drv_timer_t timer{instance} = NRF_DRV_TIMER_INSTANCE({instance});

static void timer{instance}_event_handler(nrf_timer_event_t event_type, void *p_context) {{
    if (event_type == NRF_TIMER_EVENT_COMPARE0) {{
        // Timer callback at {freq} Hz
    }}
}}

void timer{instance}_init(void) {{
    nrf_drv_timer_config_t timer_cfg = NRF_DRV_TIMER_DEFAULT_CONFIG;
    timer_cfg.frequency = NRF_TIMER_FREQ_1MHz;
    
    APP_ERROR_CHECK(nrf_drv_timer_init(&timer{instance}, &timer_cfg, timer{instance}_event_handler));
    
    uint32_t ticks = nrf_drv_timer_us_to_ticks(&timer{instance}, 1000000 / {freq});
    nrf_drv_timer_extended_compare(&timer{instance}, NRF_TIMER_CC_CHANNEL0, ticks,
                                   NRF_TIMER_SHORT_COMPARE0_CLEAR_MASK, true);
    
    nrf_drv_timer_enable(&timer{instance});
}}

void timer{instance}_stop(void) {{
    nrf_drv_timer_disable(&timer{instance});
}}
"#,
            instance = config.instance,
            freq = config.frequency_hz,
        )
    }
    
    fn generate_adc(&self, config: &AdcConfigAbstract) -> String {
        format!(r#"/**
 * ADC (SAADC) Configuration
 * nRF52 uses SAADC with {bits}-bit resolution
 */

#include "nrf_drv_saadc.h"

static nrf_saadc_value_t adc_buffer[1];

static void saadc_callback(nrf_drv_saadc_evt_t const *p_event) {{
    // Handle ADC events
}}

void saadc_init(void) {{
    nrf_drv_saadc_config_t saadc_config = NRF_DRV_SAADC_DEFAULT_CONFIG;
    saadc_config.resolution = {resolution};
    
    APP_ERROR_CHECK(nrf_drv_saadc_init(&saadc_config, saadc_callback));
    
    // Configure channels
{channel_config}}}

int16_t saadc_read(uint8_t channel) {{
    nrf_saadc_value_t value;
    nrf_drv_saadc_sample_convert(channel, &value);
    return value;
}}

int32_t saadc_read_mv(uint8_t channel) {{
    int16_t raw = saadc_read(channel);
    // Convert to millivolts (3.6V reference, gain 1/6)
    return (raw * 3600) / {max_val};
}}
"#,
            bits = config.resolution_bits,
            resolution = match config.resolution_bits {
                8 => "NRF_SAADC_RESOLUTION_8BIT",
                10 => "NRF_SAADC_RESOLUTION_10BIT",
                12 => "NRF_SAADC_RESOLUTION_12BIT",
                _ => "NRF_SAADC_RESOLUTION_14BIT",
            },
            max_val = (1 << config.resolution_bits) - 1,
            channel_config = config.channels.iter()
                .map(|ch| format!(r#"    {{
        nrf_saadc_channel_config_t channel_config = NRF_DRV_SAADC_DEFAULT_CHANNEL_CONFIG_SE(NRF_SAADC_INPUT_AIN{});
        nrf_drv_saadc_channel_init({}, &channel_config);
    }}
"#, ch, ch))
                .collect::<String>(),
        )
    }
    
    fn generate_clock_init(&self, freq_mhz: u32) -> String {
        format!(r#"/**
 * nRF52 Clock Configuration
 * Target: {freq} MHz (nRF52 runs at 64MHz fixed)
 */

#include "nrf_drv_clock.h"

void clock_init(void) {{
    ret_code_t err_code;
    
    err_code = nrf_drv_clock_init();
    APP_ERROR_CHECK(err_code);
    
    // Start HFCLK (64 MHz from external crystal)
    nrf_drv_clock_hfclk_request(NULL);
    while (!nrf_drv_clock_hfclk_is_running()) {{
        // Wait for HFCLK to start
    }}
    
    // Start LFCLK (32.768 kHz for RTC/timers)
    nrf_drv_clock_lfclk_request(NULL);
    while (!nrf_drv_clock_lfclk_is_running()) {{
        // Wait for LFCLK to start
    }}
}}
"#,
            freq = freq_mhz,
        )
    }
    
    fn generate_system_init(&self) -> String {
        r#"/**
 * nRF52 Application Entry Point
 */

#include "nrf.h"
#include "nrf_drv_clock.h"
#include "app_error.h"
#include "app_timer.h"
#include "nrf_pwr_mgmt.h"

int main(void) {
    // Initialize modules
    APP_ERROR_CHECK(NRF_LOG_INIT(NULL));
    NRF_LOG_DEFAULT_BACKENDS_INIT();
    
    clock_init();
    APP_ERROR_CHECK(app_timer_init());
    APP_ERROR_CHECK(nrf_pwr_mgmt_init());
    
    // Initialize peripherals
    
    // Main loop
    for (;;) {
        NRF_LOG_FLUSH();
        nrf_pwr_mgmt_run();
    }
}
"#.to_string()
    }
    
    fn include_headers(&self) -> Vec<&'static str> {
        vec![
            "nrf.h",
            "nrf_gpio.h",
            "nrf_drv_spi.h",
            "nrf_drv_twi.h",
            "nrf_drv_uart.h",
            "nrf_drv_timer.h",
            "app_error.h",
        ]
    }
    
    fn linker_script(&self) -> &'static str {
        match self.family {
            McuFamily::NRF52832 => "nrf52832_xxaa.ld",
            McuFamily::NRF52840 => "nrf52840_xxaa.ld",
            _ => "nrf52840_xxaa.ld",
        }
    }
    
    fn startup_file(&self) -> &'static str {
        "system_nrf52.c"
    }
}
