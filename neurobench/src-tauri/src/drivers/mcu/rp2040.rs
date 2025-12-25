// RP2040 HAL Implementation
// Raspberry Pi Pico SDK

use super::*;

/// RP2040 HAL Implementation
pub struct Rp2040Hal {
    pub family: McuFamily,
}

impl Rp2040Hal {
    pub fn new() -> Self {
        Self { family: McuFamily::RP2040 }
    }
}

impl Default for Rp2040Hal {
    fn default() -> Self {
        Self::new()
    }
}

impl McuHal for Rp2040Hal {
    fn family(&self) -> McuFamily {
        McuFamily::RP2040
    }
    
    fn generate_gpio(&self, config: &GpioConfig) -> String {
        let pin_num: u32 = config.pin.trim_start_matches("GP").parse().unwrap_or(0);
        
        let dir_str = match config.mode {
            GpioMode::Input => "GPIO_IN",
            GpioMode::Output | GpioMode::AlternateFunction(_) => "GPIO_OUT",
            GpioMode::Analog => "GPIO_IN",
        };

        format!(r#"/**
 * GPIO Configuration: GP{pin}
 * Raspberry Pi Pico SDK
 */

#include "pico/stdlib.h"
#include "hardware/gpio.h"

void gpio_{pin}_init(void) {{
    gpio_init({pin});
    gpio_set_dir({pin}, {dir});
{pull}{init_state}}}

void gpio_{pin}_set(bool level) {{
    gpio_put({pin}, level);
}}

bool gpio_{pin}_get(void) {{
    return gpio_get({pin});
}}
"#,
            pin = pin_num,
            dir = dir_str,
            pull = match config.pull {
                GpioPull::Up => format!("    gpio_pull_up({});\n", pin_num),
                GpioPull::Down => format!("    gpio_pull_down({});\n", pin_num),
                GpioPull::None => String::new(),
            },
            init_state = if let Some(state) = config.initial_state {
                format!("    gpio_put({}, {});\n", pin_num, if state { "true" } else { "false" })
            } else {
                String::new()
            },
        )
    }
    
    fn generate_spi(&self, config: &SpiConfigAbstract) -> String {
        let instance = if config.instance == 1 { "spi1" } else { "spi0" };

        format!(r#"/**
 * SPI Configuration: {instance}
 * Clock: {clock} Hz
 */

#include "pico/stdlib.h"
#include "hardware/spi.h"

void {instance}_init(void) {{
    spi_init({instance}, {clock});
    
    // Set SPI format
    spi_set_format({instance}, {bits}, {cpol}, {cpha}, SPI_MSB_FIRST);
    
    // Configure pins (default pinout)
    gpio_set_function(16, GPIO_FUNC_SPI);  // RX
    gpio_set_function(17, GPIO_FUNC_SPI);  // CSn
    gpio_set_function(18, GPIO_FUNC_SPI);  // SCK
    gpio_set_function(19, GPIO_FUNC_SPI);  // TX
}}

uint8_t {instance}_transfer(uint8_t data) {{
    uint8_t rx;
    spi_write_read_blocking({instance}, &data, &rx, 1);
    return rx;
}}

void {instance}_write(const uint8_t *data, size_t len) {{
    spi_write_blocking({instance}, data, len);
}}

void {instance}_read(uint8_t *data, size_t len) {{
    spi_read_blocking({instance}, 0, data, len);
}}
"#,
            instance = instance,
            clock = config.clock_hz,
            bits = config.data_bits,
            cpol = if config.mode >= 2 { "SPI_CPOL_1" } else { "SPI_CPOL_0" },
            cpha = if config.mode % 2 == 1 { "SPI_CPHA_1" } else { "SPI_CPHA_0" },
        )
    }
    
    fn generate_i2c(&self, config: &I2cConfigAbstract) -> String {
        let instance = if config.instance == 1 { "i2c1" } else { "i2c0" };
        
        let speed_hz = match config.speed {
            I2cSpeedAbstract::Standard100k => 100000,
            I2cSpeedAbstract::Fast400k => 400000,
            I2cSpeedAbstract::FastPlus1m => 1000000,
        };

        format!(r#"/**
 * I2C Configuration: {instance}
 * Speed: {speed} Hz
 */

#include "pico/stdlib.h"
#include "hardware/i2c.h"

void {instance}_init(void) {{
    i2c_init({instance}, {speed});
    
    // Configure pins (default pinout)
    gpio_set_function(4, GPIO_FUNC_I2C);  // SDA
    gpio_set_function(5, GPIO_FUNC_I2C);  // SCL
    gpio_pull_up(4);
    gpio_pull_up(5);
}}

int {instance}_write_reg(uint8_t addr, uint8_t reg, const uint8_t *data, size_t len) {{
    uint8_t buf[len + 1];
    buf[0] = reg;
    memcpy(&buf[1], data, len);
    return i2c_write_blocking({instance}, addr, buf, len + 1, false);
}}

int {instance}_read_reg(uint8_t addr, uint8_t reg, uint8_t *data, size_t len) {{
    i2c_write_blocking({instance}, addr, &reg, 1, true);
    return i2c_read_blocking({instance}, addr, data, len, false);
}}
"#,
            instance = instance,
            speed = speed_hz,
        )
    }
    
    fn generate_uart(&self, config: &UartConfigAbstract) -> String {
        let instance = if config.instance == 1 { "uart1" } else { "uart0" };

        format!(r#"/**
 * UART Configuration: {instance}
 * Baud: {baud}
 */

#include "pico/stdlib.h"
#include "hardware/uart.h"

void {instance}_init(void) {{
    uart_init({instance}, {baud});
    
    // Configure pins (default pinout)
    gpio_set_function(0, GPIO_FUNC_UART);  // TX
    gpio_set_function(1, GPIO_FUNC_UART);  // RX
    
    uart_set_format({instance}, {data_bits}, {stop_bits}, UART_PARITY_NONE);
    uart_set_hw_flow({instance}, false, false);
    uart_set_fifo_enabled({instance}, true);
}}

void {instance}_putc(char c) {{
    uart_putc_raw({instance}, c);
}}

void {instance}_puts(const char *str) {{
    uart_puts({instance}, str);
}}

int {instance}_getc(void) {{
    if (uart_is_readable({instance})) {{
        return uart_getc({instance});
    }}
    return -1;
}}
"#,
            instance = instance,
            baud = config.baud_rate,
            data_bits = config.data_bits,
            stop_bits = config.stop_bits,
        )
    }
    
    fn generate_timer(&self, config: &TimerConfigAbstract) -> String {
        format!(r#"/**
 * Timer Configuration using RP2040 Alarm
 * Frequency: {freq} Hz
 */

#include "pico/stdlib.h"
#include "hardware/timer.h"

static volatile bool timer{instance}_fired = false;
static repeating_timer_t timer{instance}_rt;

static bool timer{instance}_callback(repeating_timer_t *rt) {{
    timer{instance}_fired = true;
    // Add your callback code here
    return true;  // Keep repeating
}}

void timer{instance}_init(void) {{
    // Period in microseconds (negative for periodic)
    int64_t period_us = -(1000000 / {freq});
    add_repeating_timer_us(period_us, timer{instance}_callback, NULL, &timer{instance}_rt);
}}

void timer{instance}_stop(void) {{
    cancel_repeating_timer(&timer{instance}_rt);
}}

bool timer{instance}_check_and_clear(void) {{
    if (timer{instance}_fired) {{
        timer{instance}_fired = false;
        return true;
    }}
    return false;
}}
"#,
            instance = config.instance,
            freq = config.frequency_hz,
        )
    }
    
    fn generate_adc(&self, config: &AdcConfigAbstract) -> String {
        format!(r#"/**
 * ADC Configuration
 * RP2040 has 3 ADC channels (GPIO26-28) + temperature sensor
 */

#include "pico/stdlib.h"
#include "hardware/adc.h"

void adc_init_channels(void) {{
    adc_init();
    
    // Configure ADC pins
{channel_config}}}

uint16_t adc_read_channel(uint8_t channel) {{
    adc_select_input(channel);
    return adc_read();  // 12-bit result (0-4095)
}}

float adc_read_voltage(uint8_t channel) {{
    uint16_t raw = adc_read_channel(channel);
    return raw * 3.3f / 4095.0f;
}}

float adc_read_temperature(void) {{
    adc_select_input(4);  // Temperature sensor
    uint16_t raw = adc_read();
    float voltage = raw * 3.3f / 4095.0f;
    return 27.0f - (voltage - 0.706f) / 0.001721f;
}}
"#,
            channel_config = config.channels.iter()
                .map(|ch| format!("    adc_gpio_init(26 + {});  // ADC{}\n", ch, ch))
                .collect::<String>(),
        )
    }
    
    fn generate_clock_init(&self, freq_mhz: u32) -> String {
        format!(r#"/**
 * RP2040 Clock Configuration
 * Target: {freq} MHz (max 133 MHz without overclocking)
 */

#include "pico/stdlib.h"
#include "hardware/clocks.h"
#include "hardware/pll.h"

void clock_config(void) {{
    // RP2040 uses stdio_init_all() which configures clocks
    // For custom frequencies, use set_sys_clock_khz()
    
    set_sys_clock_khz({freq_khz}, true);
    
    // Reconfigure stdio after clock change
    stdio_init_all();
}}
"#,
            freq = freq_mhz,
            freq_khz = freq_mhz * 1000,
        )
    }
    
    fn generate_system_init(&self) -> String {
        r#"/**
 * RP2040 Application Entry Point
 */

#include "pico/stdlib.h"

int main(void) {
    stdio_init_all();
    
    // Initialize peripherals
    
    while (true) {
        // Main loop
        tight_loop_contents();
    }
    
    return 0;
}
"#.to_string()
    }
    
    fn include_headers(&self) -> Vec<&'static str> {
        vec![
            "pico/stdlib.h",
            "hardware/gpio.h",
            "hardware/spi.h",
            "hardware/i2c.h",
            "hardware/uart.h",
            "hardware/adc.h",
        ]
    }
    
    fn linker_script(&self) -> &'static str {
        "memmap_default.ld"
    }
    
    fn startup_file(&self) -> &'static str {
        "crt0.S"
    }
}
