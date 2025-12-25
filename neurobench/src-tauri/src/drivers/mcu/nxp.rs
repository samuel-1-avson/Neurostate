// NXP LPC HAL Implementation
// Supports LPC1768, LPC5500

use super::*;

/// NXP LPC HAL Implementation
pub struct NxpHal {
    pub family: McuFamily,
}

impl NxpHal {
    pub fn new(family: McuFamily) -> Self {
        Self { family }
    }
    
    fn sdk_name(&self) -> &'static str {
        match self.family {
            McuFamily::LPC1768 => "LPCOpen",
            McuFamily::LPC5500 => "MCUXpresso SDK",
            _ => "LPCOpen",
        }
    }
}

impl McuHal for NxpHal {
    fn family(&self) -> McuFamily {
        self.family
    }
    
    fn generate_gpio(&self, config: &GpioConfig) -> String {
        // Parse "P0_5" -> (0, 5)
        let parts: Vec<&str> = config.pin.trim_start_matches("P").split('_').collect();
        let port: u8 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let pin: u8 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        
        let dir_str = match config.mode {
            GpioMode::Input => "false",
            GpioMode::Output | GpioMode::AlternateFunction(_) => "true",
            GpioMode::Analog => "false",
        };

        format!(r#"/**
 * GPIO Configuration: P{port}_{pin}
 * {sdk}
 */

#include "chip.h"

void gpio_p{port}_{pin}_init(void) {{
    Chip_GPIO_SetPinDIR(LPC_GPIO, {port}, {pin}, {dir});
{pull}{init_state}}}

void gpio_p{port}_{pin}_set(bool level) {{
    Chip_GPIO_SetPinState(LPC_GPIO, {port}, {pin}, level);
}}

bool gpio_p{port}_{pin}_get(void) {{
    return Chip_GPIO_GetPinState(LPC_GPIO, {port}, {pin});
}}

void gpio_p{port}_{pin}_toggle(void) {{
    Chip_GPIO_SetPinToggle(LPC_GPIO, {port}, {pin});
}}
"#,
            port = port,
            pin = pin,
            sdk = self.sdk_name(),
            dir = dir_str,
            pull = match config.pull {
                GpioPull::Up => format!("    Chip_IOCON_PinMux(LPC_IOCON, {}, {}, IOCON_MODE_PULLUP, IOCON_FUNC0);\n", port, pin),
                GpioPull::Down => format!("    Chip_IOCON_PinMux(LPC_IOCON, {}, {}, IOCON_MODE_PULLDOWN, IOCON_FUNC0);\n", port, pin),
                GpioPull::None => format!("    Chip_IOCON_PinMux(LPC_IOCON, {}, {}, IOCON_MODE_INACT, IOCON_FUNC0);\n", port, pin),
            },
            init_state = if let Some(state) = config.initial_state {
                format!("    Chip_GPIO_SetPinState(LPC_GPIO, {}, {}, {});\n", port, pin, state)
            } else {
                String::new()
            },
        )
    }
    
    fn generate_spi(&self, config: &SpiConfigAbstract) -> String {
        format!(r#"/**
 * SPI Configuration: SSP{instance}
 * Clock: {clock} Hz
 */

#include "chip.h"

void ssp{instance}_init(void) {{
    Chip_SSP_Init(LPC_SSP{instance});
    
    Chip_SSP_SetFormat(LPC_SSP{instance}, SSP_BITS_{bits}, SSP_FRAMEFORMAT_SPI, {mode});
    Chip_SSP_SetMaster(LPC_SSP{instance}, true);
    Chip_SSP_SetBitRate(LPC_SSP{instance}, {clock});
    Chip_SSP_Enable(LPC_SSP{instance});
}}

uint8_t ssp{instance}_transfer(uint8_t data) {{
    Chip_SSP_DATA_SETUP_T xferConfig;
    uint8_t rx;
    
    xferConfig.tx_data = &data;
    xferConfig.tx_cnt = 0;
    xferConfig.rx_data = &rx;
    xferConfig.rx_cnt = 0;
    xferConfig.length = 1;
    
    Chip_SSP_RWFrames_Blocking(LPC_SSP{instance}, &xferConfig);
    return rx;
}}
"#,
            instance = config.instance,
            clock = config.clock_hz,
            bits = config.data_bits,
            mode = match config.mode {
                0 => "SSP_CLOCK_CPHA0_CPOL0",
                1 => "SSP_CLOCK_CPHA1_CPOL0",
                2 => "SSP_CLOCK_CPHA0_CPOL1",
                3 => "SSP_CLOCK_CPHA1_CPOL1",
                _ => "SSP_CLOCK_CPHA0_CPOL0",
            },
        )
    }
    
    fn generate_i2c(&self, config: &I2cConfigAbstract) -> String {
        let speed_hz = match config.speed {
            I2cSpeedAbstract::Standard100k => 100000,
            I2cSpeedAbstract::Fast400k => 400000,
            I2cSpeedAbstract::FastPlus1m => 1000000,
        };

        format!(r#"/**
 * I2C Configuration: I2C{instance}
 * Speed: {speed} Hz
 */

#include "chip.h"

void i2c{instance}_init(void) {{
    Chip_I2C_Init(I2C{instance});
    Chip_I2C_SetClockRate(I2C{instance}, {speed});
    Chip_I2C_SetMasterEventHandler(I2C{instance}, Chip_I2C_EventHandlerPolling);
}}

Status i2c{instance}_write(uint8_t addr, uint8_t reg, const uint8_t *data, uint8_t len) {{
    I2C_XFER_T xfer;
    uint8_t buf[len + 1];
    
    buf[0] = reg;
    memcpy(&buf[1], data, len);
    
    xfer.slaveAddr = addr;
    xfer.txBuff = buf;
    xfer.txSz = len + 1;
    xfer.rxBuff = NULL;
    xfer.rxSz = 0;
    
    return Chip_I2C_MasterTransfer(I2C{instance}, &xfer);
}}

Status i2c{instance}_read(uint8_t addr, uint8_t reg, uint8_t *data, uint8_t len) {{
    I2C_XFER_T xfer;
    
    xfer.slaveAddr = addr;
    xfer.txBuff = &reg;
    xfer.txSz = 1;
    xfer.rxBuff = data;
    xfer.rxSz = len;
    
    return Chip_I2C_MasterTransfer(I2C{instance}, &xfer);
}}
"#,
            instance = config.instance,
            speed = speed_hz,
        )
    }
    
    fn generate_uart(&self, config: &UartConfigAbstract) -> String {
        format!(r#"/**
 * UART Configuration: UART{instance}
 * Baud: {baud}
 */

#include "chip.h"

void uart{instance}_init(void) {{
    Chip_UART_Init(LPC_UART{instance});
    Chip_UART_SetBaud(LPC_UART{instance}, {baud});
    Chip_UART_ConfigData(LPC_UART{instance}, 
                         UART_LCR_WLEN{data_bits} | 
                         {parity} | 
                         {stop_bits});
    Chip_UART_TXEnable(LPC_UART{instance});
}}

void uart{instance}_putc(char c) {{
    while (!(Chip_UART_ReadLineStatus(LPC_UART{instance}) & UART_LSR_THRE)) {{}}
    Chip_UART_SendByte(LPC_UART{instance}, c);
}}

void uart{instance}_puts(const char *str) {{
    while (*str) {{
        uart{instance}_putc(*str++);
    }}
}}

int uart{instance}_getc(void) {{
    if (Chip_UART_ReadLineStatus(LPC_UART{instance}) & UART_LSR_RDR) {{
        return Chip_UART_ReadByte(LPC_UART{instance});
    }}
    return -1;
}}
"#,
            instance = config.instance,
            baud = config.baud_rate,
            data_bits = config.data_bits,
            parity = match config.parity {
                UartParity::None => "UART_LCR_PARITY_DIS",
                UartParity::Even => "UART_LCR_PARITY_EVEN",
                UartParity::Odd => "UART_LCR_PARITY_ODD",
            },
            stop_bits = if config.stop_bits == 2 { "UART_LCR_SBS_2BIT" } else { "UART_LCR_SBS_1BIT" },
        )
    }
    
    fn generate_timer(&self, config: &TimerConfigAbstract) -> String {
        format!(r#"/**
 * Timer Configuration: TIMER{instance}
 * Frequency: {freq} Hz
 */

#include "chip.h"

void timer{instance}_init(void) {{
    Chip_TIMER_Init(LPC_TIMER{instance});
    Chip_TIMER_Reset(LPC_TIMER{instance});
    
    // Set prescaler for 1MHz timer clock
    uint32_t timerFreq = Chip_Clock_GetPeripheralClockRate(SYSCTL_PCLK_TIMER{instance});
    Chip_TIMER_PrescaleSet(LPC_TIMER{instance}, (timerFreq / 1000000) - 1);
    
    // Set match value for desired frequency
    uint32_t matchValue = 1000000 / {freq};
    Chip_TIMER_SetMatch(LPC_TIMER{instance}, 0, matchValue);
    Chip_TIMER_ResetOnMatchEnable(LPC_TIMER{instance}, 0);
    Chip_TIMER_MatchEnableInt(LPC_TIMER{instance}, 0);
    
    NVIC_EnableIRQ(TIMER{instance}_IRQn);
}}

void timer{instance}_start(void) {{
    Chip_TIMER_Enable(LPC_TIMER{instance});
}}

void timer{instance}_stop(void) {{
    Chip_TIMER_Disable(LPC_TIMER{instance});
}}

void TIMER{instance}_IRQHandler(void) {{
    if (Chip_TIMER_MatchPending(LPC_TIMER{instance}, 0)) {{
        Chip_TIMER_ClearMatch(LPC_TIMER{instance}, 0);
        // Timer callback at {freq} Hz
    }}
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

#include "chip.h"

void adc_init(void) {{
    Chip_ADC_Init(LPC_ADC, &ADCClockSetup);
    Chip_ADC_SetSampleRate(LPC_ADC, &ADCClockSetup, 200000);
    Chip_ADC_EnableChannel(LPC_ADC, ADC_CH0, ENABLE);
    Chip_ADC_SetBurstCmd(LPC_ADC, {burst});
}}

uint16_t adc_read(uint8_t channel) {{
    uint16_t dataADC;
    
    Chip_ADC_EnableChannel(LPC_ADC, channel, ENABLE);
    Chip_ADC_SetStartMode(LPC_ADC, ADC_START_NOW, ADC_TRIGGERMODE_RISING);
    
    while (Chip_ADC_ReadStatus(LPC_ADC, channel, ADC_DR_DONE_STAT) != SET) {{}}
    
    Chip_ADC_ReadValue(LPC_ADC, channel, &dataADC);
    return dataADC;
}}

uint32_t adc_read_mv(uint8_t channel) {{
    uint16_t raw = adc_read(channel);
    // 3.3V reference, 12-bit ADC
    return (raw * 3300) / 4095;
}}
"#,
            bits = config.resolution_bits,
            burst = if config.continuous { "ENABLE" } else { "DISABLE" },
        )
    }
    
    fn generate_clock_init(&self, freq_mhz: u32) -> String {
        format!(r#"/**
 * LPC Clock Configuration
 * Target: {freq} MHz
 */

#include "chip.h"

void clock_init(void) {{
    // Set up PLL for {freq} MHz operation
    Chip_SetupXtalClocking();
    
    // Set CPU clock divider
    Chip_Clock_SetCPUClockDiv(1);
    
    // Update SystemCoreClock variable
    SystemCoreClockUpdate();
}}
"#,
            freq = freq_mhz,
        )
    }
    
    fn generate_system_init(&self) -> String {
        r#"/**
 * LPC Application Entry Point
 */

#include "chip.h"

int main(void) {
    SystemCoreClockUpdate();
    
    // Initialize peripherals
    
    while (1) {
        // Main loop
        __WFI();
    }
    
    return 0;
}
"#.to_string()
    }
    
    fn include_headers(&self) -> Vec<&'static str> {
        vec![
            "chip.h",
            "board.h",
        ]
    }
    
    fn linker_script(&self) -> &'static str {
        match self.family {
            McuFamily::LPC1768 => "LPC1768.ld",
            McuFamily::LPC5500 => "LPC55S69.ld",
            _ => "LPC1768.ld",
        }
    }
    
    fn startup_file(&self) -> &'static str {
        match self.family {
            McuFamily::LPC1768 => "startup_LPC17xx.S",
            McuFamily::LPC5500 => "startup_LPC55S69.c",
            _ => "startup_LPC17xx.S",
        }
    }
}
