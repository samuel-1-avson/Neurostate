// STM32 HAL Implementation
// Supports STM32F1, F4, H7, L4, G4 families

use super::*;

/// STM32 HAL Implementation
pub struct Stm32Hal {
    pub family: McuFamily,
}

impl Stm32Hal {
    pub fn new(family: McuFamily) -> Self {
        Self { family }
    }
    
    fn hal_prefix(&self) -> &'static str {
        match self.family {
            McuFamily::STM32F1 => "stm32f1xx",
            McuFamily::STM32F4 => "stm32f4xx",
            McuFamily::STM32H7 => "stm32h7xx",
            McuFamily::STM32L4 => "stm32l4xx",
            McuFamily::STM32G4 => "stm32g4xx",
            _ => "stm32f4xx",
        }
    }
    
    fn gpio_port(&self, pin: &str) -> (String, String) {
        // Parse "PA5" -> ("GPIOA", "5")
        let port = &pin[1..2];
        let num = &pin[2..];
        let port_name = match port {
            "A" => "GPIOA",
            "B" => "GPIOB",
            "C" => "GPIOC",
            "D" => "GPIOD",
            "E" => "GPIOE",
            "F" => "GPIOF",
            "G" => "GPIOG",
            "H" => "GPIOH",
            _ => "GPIOA",
        };
        (port_name.to_string(), num.to_string())
    }
}

impl McuHal for Stm32Hal {
    fn family(&self) -> McuFamily {
        self.family
    }
    
    fn generate_gpio(&self, config: &GpioConfig) -> String {
        let (port, pin_num) = self.gpio_port(&config.pin);
        let hal = self.hal_prefix();
        
        let mode_str = match config.mode {
            GpioMode::Input => "GPIO_MODE_INPUT",
            GpioMode::Output => "GPIO_MODE_OUTPUT_PP",
            GpioMode::AlternateFunction(af) => &format!("GPIO_MODE_AF_PP /* AF{} */", af),
            GpioMode::Analog => "GPIO_MODE_ANALOG",
        };
        
        let pull_str = match config.pull {
            GpioPull::None => "GPIO_NOPULL",
            GpioPull::Up => "GPIO_PULLUP",
            GpioPull::Down => "GPIO_PULLDOWN",
        };
        
        let speed_str = match config.speed {
            GpioSpeed::Low => "GPIO_SPEED_FREQ_LOW",
            GpioSpeed::Medium => "GPIO_SPEED_FREQ_MEDIUM",
            GpioSpeed::High => "GPIO_SPEED_FREQ_HIGH",
            GpioSpeed::VeryHigh => "GPIO_SPEED_FREQ_VERY_HIGH",
        };

        format!(r#"/**
 * GPIO Configuration: {pin}
 * Auto-generated for {family:?}
 */

#include "{hal}_hal.h"

void GPIO_{pin}_Init(void) {{
    GPIO_InitTypeDef GPIO_InitStruct = {{0}};
    
    __HAL_RCC_{port}_CLK_ENABLE();
    
    GPIO_InitStruct.Pin = GPIO_PIN_{pin_num};
    GPIO_InitStruct.Mode = {mode};
    GPIO_InitStruct.Pull = {pull};
    GPIO_InitStruct.Speed = {speed};
    
    HAL_GPIO_Init({port}, &GPIO_InitStruct);
{init_state}}}
"#,
            pin = config.pin,
            family = self.family,
            hal = hal,
            port = port,
            pin_num = pin_num,
            mode = mode_str,
            pull = pull_str,
            speed = speed_str,
            init_state = if let Some(state) = config.initial_state {
                format!("\n    HAL_GPIO_WritePin({}, GPIO_PIN_{}, GPIO_PIN_{});\n", 
                    port, pin_num, if state { "SET" } else { "RESET" })
            } else {
                String::new()
            },
        )
    }
    
    fn generate_spi(&self, config: &SpiConfigAbstract) -> String {
        let hal = self.hal_prefix();
        let instance = format!("SPI{}", config.instance);
        
        let (cpol, cpha) = match config.mode {
            0 => ("SPI_POLARITY_LOW", "SPI_PHASE_1EDGE"),
            1 => ("SPI_POLARITY_LOW", "SPI_PHASE_2EDGE"),
            2 => ("SPI_POLARITY_HIGH", "SPI_PHASE_1EDGE"),
            3 => ("SPI_POLARITY_HIGH", "SPI_PHASE_2EDGE"),
            _ => ("SPI_POLARITY_LOW", "SPI_PHASE_1EDGE"),
        };

        format!(r#"/**
 * SPI Configuration: {instance}
 * Clock: {clock} Hz, Mode {mode}
 */

#include "{hal}_hal.h"

SPI_HandleTypeDef h{instance_lower};

void {instance}_Init(void) {{
    h{instance_lower}.Instance = {instance};
    h{instance_lower}.Init.Mode = SPI_MODE_MASTER;
    h{instance_lower}.Init.Direction = SPI_DIRECTION_2LINES;
    h{instance_lower}.Init.DataSize = {data_size};
    h{instance_lower}.Init.CLKPolarity = {cpol};
    h{instance_lower}.Init.CLKPhase = {cpha};
    h{instance_lower}.Init.NSS = SPI_NSS_SOFT;
    h{instance_lower}.Init.BaudRatePrescaler = SPI_BAUDRATEPRESCALER_16;
    h{instance_lower}.Init.FirstBit = {bit_order};
    h{instance_lower}.Init.TIMode = SPI_TIMODE_DISABLE;
    h{instance_lower}.Init.CRCCalculation = SPI_CRCCALCULATION_DISABLE;
    
    if (HAL_SPI_Init(&h{instance_lower}) != HAL_OK) {{
        Error_Handler();
    }}
}}

uint8_t {instance}_Transfer(uint8_t data) {{
    uint8_t rx;
    HAL_SPI_TransmitReceive(&h{instance_lower}, &data, &rx, 1, HAL_MAX_DELAY);
    return rx;
}}
"#,
            instance = instance,
            instance_lower = instance.to_lowercase(),
            hal = hal,
            clock = config.clock_hz,
            mode = config.mode,
            data_size = if config.data_bits == 16 { "SPI_DATASIZE_16BIT" } else { "SPI_DATASIZE_8BIT" },
            cpol = cpol,
            cpha = cpha,
            bit_order = if config.msb_first { "SPI_FIRSTBIT_MSB" } else { "SPI_FIRSTBIT_LSB" },
        )
    }
    
    fn generate_i2c(&self, config: &I2cConfigAbstract) -> String {
        let hal = self.hal_prefix();
        let instance = format!("I2C{}", config.instance);
        
        let speed_hz = match config.speed {
            I2cSpeedAbstract::Standard100k => 100000,
            I2cSpeedAbstract::Fast400k => 400000,
            I2cSpeedAbstract::FastPlus1m => 1000000,
        };

        format!(r#"/**
 * I2C Configuration: {instance}
 * Speed: {speed} Hz
 */

#include "{hal}_hal.h"

I2C_HandleTypeDef h{instance_lower};

void {instance}_Init(void) {{
    h{instance_lower}.Instance = {instance};
    h{instance_lower}.Init.ClockSpeed = {speed};
    h{instance_lower}.Init.DutyCycle = I2C_DUTYCYCLE_2;
    h{instance_lower}.Init.OwnAddress1 = 0;
    h{instance_lower}.Init.AddressingMode = {addr_mode};
    h{instance_lower}.Init.DualAddressMode = I2C_DUALADDRESS_DISABLE;
    h{instance_lower}.Init.GeneralCallMode = I2C_GENERALCALL_DISABLE;
    h{instance_lower}.Init.NoStretchMode = I2C_NOSTRETCH_DISABLE;
    
    if (HAL_I2C_Init(&h{instance_lower}) != HAL_OK) {{
        Error_Handler();
    }}
}}

bool {instance}_Write(uint8_t addr, uint8_t reg, uint8_t *data, uint16_t len) {{
    return HAL_I2C_Mem_Write(&h{instance_lower}, addr << 1, reg, 
                             I2C_MEMADD_SIZE_8BIT, data, len, HAL_MAX_DELAY) == HAL_OK;
}}

bool {instance}_Read(uint8_t addr, uint8_t reg, uint8_t *data, uint16_t len) {{
    return HAL_I2C_Mem_Read(&h{instance_lower}, addr << 1, reg,
                            I2C_MEMADD_SIZE_8BIT, data, len, HAL_MAX_DELAY) == HAL_OK;
}}
"#,
            instance = instance,
            instance_lower = instance.to_lowercase(),
            hal = hal,
            speed = speed_hz,
            addr_mode = if config.address_bits == 10 { "I2C_ADDRESSINGMODE_10BIT" } else { "I2C_ADDRESSINGMODE_7BIT" },
        )
    }
    
    fn generate_uart(&self, config: &UartConfigAbstract) -> String {
        let hal = self.hal_prefix();
        let instance = format!("USART{}", config.instance);
        
        let parity_str = match config.parity {
            UartParity::None => "UART_PARITY_NONE",
            UartParity::Even => "UART_PARITY_EVEN",
            UartParity::Odd => "UART_PARITY_ODD",
        };

        format!(r#"/**
 * UART Configuration: {instance}
 * Baud: {baud}, {data}N{stop}
 */

#include "{hal}_hal.h"

UART_HandleTypeDef h{instance_lower};

void {instance}_Init(void) {{
    h{instance_lower}.Instance = {instance};
    h{instance_lower}.Init.BaudRate = {baud};
    h{instance_lower}.Init.WordLength = {word_len};
    h{instance_lower}.Init.StopBits = {stop_bits};
    h{instance_lower}.Init.Parity = {parity};
    h{instance_lower}.Init.Mode = UART_MODE_TX_RX;
    h{instance_lower}.Init.HwFlowCtl = {flow};
    h{instance_lower}.Init.OverSampling = UART_OVERSAMPLING_16;
    
    if (HAL_UART_Init(&h{instance_lower}) != HAL_OK) {{
        Error_Handler();
    }}
}}

void {instance}_SendChar(char c) {{
    HAL_UART_Transmit(&h{instance_lower}, (uint8_t*)&c, 1, HAL_MAX_DELAY);
}}

void {instance}_SendString(const char *str) {{
    HAL_UART_Transmit(&h{instance_lower}, (uint8_t*)str, strlen(str), HAL_MAX_DELAY);
}}
"#,
            instance = instance,
            instance_lower = instance.to_lowercase(),
            hal = hal,
            baud = config.baud_rate,
            data = config.data_bits,
            stop = config.stop_bits,
            word_len = if config.data_bits == 9 { "UART_WORDLENGTH_9B" } else { "UART_WORDLENGTH_8B" },
            stop_bits = if config.stop_bits == 2 { "UART_STOPBITS_2" } else { "UART_STOPBITS_1" },
            parity = parity_str,
            flow = if config.flow_control { "UART_HWCONTROL_RTS_CTS" } else { "UART_HWCONTROL_NONE" },
        )
    }
    
    fn generate_timer(&self, config: &TimerConfigAbstract) -> String {
        let hal = self.hal_prefix();
        let instance = format!("TIM{}", config.instance);
        
        // Calculate prescaler and period
        let sysclk = self.family.max_frequency_mhz() * 1_000_000;
        let prescaler = sysclk / (config.frequency_hz * 65536) + 1;
        let period = sysclk / (prescaler * config.frequency_hz);

        format!(r#"/**
 * Timer Configuration: {instance}
 * Frequency: {freq} Hz
 */

#include "{hal}_hal.h"

TIM_HandleTypeDef h{instance_lower};

void {instance}_Init(void) {{
    TIM_ClockConfigTypeDef sClockSourceConfig = {{0}};
    
    h{instance_lower}.Instance = {instance};
    h{instance_lower}.Init.Prescaler = {prescaler} - 1;
    h{instance_lower}.Init.CounterMode = TIM_COUNTERMODE_UP;
    h{instance_lower}.Init.Period = {period} - 1;
    h{instance_lower}.Init.ClockDivision = TIM_CLOCKDIVISION_DIV1;
    h{instance_lower}.Init.AutoReloadPreload = TIM_AUTORELOAD_PRELOAD_ENABLE;
    
    if (HAL_TIM_Base_Init(&h{instance_lower}) != HAL_OK) {{
        Error_Handler();
    }}
    
    sClockSourceConfig.ClockSource = TIM_CLOCKSOURCE_INTERNAL;
    HAL_TIM_ConfigClockSource(&h{instance_lower}, &sClockSourceConfig);
}}

void {instance}_Start(void) {{
    HAL_TIM_Base_Start(&h{instance_lower});
}}

void {instance}_StartIT(void) {{
    HAL_TIM_Base_Start_IT(&h{instance_lower});
}}
"#,
            instance = instance,
            instance_lower = instance.to_lowercase(),
            hal = hal,
            freq = config.frequency_hz,
            prescaler = prescaler,
            period = period,
        )
    }
    
    fn generate_adc(&self, config: &AdcConfigAbstract) -> String {
        let hal = self.hal_prefix();
        let instance = format!("ADC{}", config.instance);
        
        let resolution = match config.resolution_bits {
            8 => "ADC_RESOLUTION_8B",
            10 => "ADC_RESOLUTION_10B",
            _ => "ADC_RESOLUTION_12B",
        };

        format!(r#"/**
 * ADC Configuration: {instance}
 * Resolution: {bits}-bit, Channels: {num_ch}
 */

#include "{hal}_hal.h"

ADC_HandleTypeDef h{instance_lower};

void {instance}_Init(void) {{
    ADC_ChannelConfTypeDef sConfig = {{0}};
    
    h{instance_lower}.Instance = {instance};
    h{instance_lower}.Init.ClockPrescaler = ADC_CLOCK_SYNC_PCLK_DIV4;
    h{instance_lower}.Init.Resolution = {resolution};
    h{instance_lower}.Init.ScanConvMode = {scan};
    h{instance_lower}.Init.ContinuousConvMode = {continuous};
    h{instance_lower}.Init.DiscontinuousConvMode = DISABLE;
    h{instance_lower}.Init.ExternalTrigConvEdge = ADC_EXTERNALTRIGCONVEDGE_NONE;
    h{instance_lower}.Init.ExternalTrigConv = ADC_SOFTWARE_START;
    h{instance_lower}.Init.DataAlign = ADC_DATAALIGN_RIGHT;
    h{instance_lower}.Init.NbrOfConversion = {num_ch};
    h{instance_lower}.Init.DMAContinuousRequests = {dma};
    
    if (HAL_ADC_Init(&h{instance_lower}) != HAL_OK) {{
        Error_Handler();
    }}
}}

uint16_t {instance}_Read(void) {{
    HAL_ADC_Start(&h{instance_lower});
    HAL_ADC_PollForConversion(&h{instance_lower}, HAL_MAX_DELAY);
    return HAL_ADC_GetValue(&h{instance_lower});
}}
"#,
            instance = instance,
            instance_lower = instance.to_lowercase(),
            hal = hal,
            bits = config.resolution_bits,
            num_ch = config.channels.len(),
            resolution = resolution,
            scan = if config.channels.len() > 1 { "ENABLE" } else { "DISABLE" },
            continuous = if config.continuous { "ENABLE" } else { "DISABLE" },
            dma = if config.dma { "ENABLE" } else { "DISABLE" },
        )
    }
    
    fn generate_clock_init(&self, freq_mhz: u32) -> String {
        let hal = self.hal_prefix();
        
        format!(r#"/**
 * System Clock Configuration
 * Target: {freq} MHz
 */

#include "{hal}_hal.h"

void SystemClock_Config(void) {{
    RCC_OscInitTypeDef RCC_OscInitStruct = {{0}};
    RCC_ClkInitTypeDef RCC_ClkInitStruct = {{0}};
    
    __HAL_RCC_PWR_CLK_ENABLE();
    __HAL_PWR_VOLTAGESCALING_CONFIG(PWR_REGULATOR_VOLTAGE_SCALE1);
    
    RCC_OscInitStruct.OscillatorType = RCC_OSCILLATORTYPE_HSE;
    RCC_OscInitStruct.HSEState = RCC_HSE_ON;
    RCC_OscInitStruct.PLL.PLLState = RCC_PLL_ON;
    RCC_OscInitStruct.PLL.PLLSource = RCC_PLLSOURCE_HSE;
    
    if (HAL_RCC_OscConfig(&RCC_OscInitStruct) != HAL_OK) {{
        Error_Handler();
    }}
    
    RCC_ClkInitStruct.ClockType = RCC_CLOCKTYPE_HCLK | RCC_CLOCKTYPE_SYSCLK |
                                  RCC_CLOCKTYPE_PCLK1 | RCC_CLOCKTYPE_PCLK2;
    RCC_ClkInitStruct.SYSCLKSource = RCC_SYSCLKSOURCE_PLLCLK;
    RCC_ClkInitStruct.AHBCLKDivider = RCC_SYSCLK_DIV1;
    RCC_ClkInitStruct.APB1CLKDivider = RCC_HCLK_DIV4;
    RCC_ClkInitStruct.APB2CLKDivider = RCC_HCLK_DIV2;
    
    if (HAL_RCC_ClockConfig(&RCC_ClkInitStruct, FLASH_LATENCY_5) != HAL_OK) {{
        Error_Handler();
    }}
}}
"#,
            freq = freq_mhz,
            hal = hal,
        )
    }
    
    fn generate_system_init(&self) -> String {
        format!(r#"/**
 * System Initialization
 * MCU: {family:?}
 */

int main(void) {{
    HAL_Init();
    SystemClock_Config();
    
    // Initialize peripherals
    
    while (1) {{
        // Main loop
    }}
}}

void Error_Handler(void) {{
    __disable_irq();
    while (1) {{
    }}
}}
"#,
            family = self.family,
        )
    }
    
    fn include_headers(&self) -> Vec<&'static str> {
        match self.family {
            McuFamily::STM32F1 => vec!["stm32f1xx_hal.h", "stm32f1xx_hal_conf.h"],
            McuFamily::STM32F4 => vec!["stm32f4xx_hal.h", "stm32f4xx_hal_conf.h"],
            McuFamily::STM32H7 => vec!["stm32h7xx_hal.h", "stm32h7xx_hal_conf.h"],
            McuFamily::STM32L4 => vec!["stm32l4xx_hal.h", "stm32l4xx_hal_conf.h"],
            McuFamily::STM32G4 => vec!["stm32g4xx_hal.h", "stm32g4xx_hal_conf.h"],
            _ => vec!["stm32f4xx_hal.h"],
        }
    }
    
    fn linker_script(&self) -> &'static str {
        match self.family {
            McuFamily::STM32F1 => "STM32F103C8Tx_FLASH.ld",
            McuFamily::STM32F4 => "STM32F407VGTx_FLASH.ld",
            McuFamily::STM32H7 => "STM32H743ZITx_FLASH.ld",
            McuFamily::STM32L4 => "STM32L476RGTx_FLASH.ld",
            McuFamily::STM32G4 => "STM32G474RETx_FLASH.ld",
            _ => "STM32F407VGTx_FLASH.ld",
        }
    }
    
    fn startup_file(&self) -> &'static str {
        match self.family {
            McuFamily::STM32F1 => "startup_stm32f103xb.s",
            McuFamily::STM32F4 => "startup_stm32f407xx.s",
            McuFamily::STM32H7 => "startup_stm32h743xx.s",
            McuFamily::STM32L4 => "startup_stm32l476xx.s",
            McuFamily::STM32G4 => "startup_stm32g474xx.s",
            _ => "startup_stm32f407xx.s",
        }
    }
}
