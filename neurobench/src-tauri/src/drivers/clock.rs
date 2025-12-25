// Clock and Power Management Driver
// Based on Chapter 15: Clocks, Resets, and Power Supply

use serde::{Deserialize, Serialize};

/// Clock source options
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ClockSource {
    HSI,      // High-Speed Internal (16MHz for STM32F4)
    HSE,      // High-Speed External (typically 8-25MHz)
    LSI,      // Low-Speed Internal (32kHz)
    LSE,      // Low-Speed External (32.768kHz)
    PLL,      // Phase-Locked Loop
}

impl Default for ClockSource {
    fn default() -> Self {
        ClockSource::HSI
    }
}

/// PLL Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PllConfig {
    pub source: ClockSource,     // HSI or HSE
    pub pllm: u32,               // Division factor (2-63)
    pub plln: u32,               // Multiplication factor (50-432)
    pub pllp: u32,               // Division for main clock (2, 4, 6, 8)
    pub pllq: u32,               // Division for USB/SDIO (2-15)
}

impl Default for PllConfig {
    fn default() -> Self {
        // Default: 16MHz HSI -> 84MHz SYSCLK
        Self {
            source: ClockSource::HSI,
            pllm: 16,    // 16MHz / 16 = 1MHz
            plln: 336,   // 1MHz * 336 = 336MHz VCO
            pllp: 4,     // 336MHz / 4 = 84MHz
            pllq: 7,     // 336MHz / 7 = 48MHz for USB
        }
    }
}

/// Bus prescalers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusClocksConfig {
    pub ahb_prescaler: u32,      // AHB divider (1, 2, 4, 8, ..., 512)
    pub apb1_prescaler: u32,     // APB1 low-speed (1, 2, 4, 8, 16) - max 42MHz
    pub apb2_prescaler: u32,     // APB2 high-speed (1, 2, 4, 8, 16) - max 84MHz
}

impl Default for BusClocksConfig {
    fn default() -> Self {
        Self {
            ahb_prescaler: 1,
            apb1_prescaler: 2,   // 84MHz / 2 = 42MHz
            apb2_prescaler: 1,   // 84MHz
        }
    }
}

/// Full clock configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockConfig {
    pub sysclk_source: ClockSource,
    pub hse_freq_hz: Option<u32>,  // External crystal frequency
    pub pll: PllConfig,
    pub bus_clocks: BusClocksConfig,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            sysclk_source: ClockSource::PLL,
            hse_freq_hz: Some(8_000_000),  // Common 8MHz crystal
            pll: PllConfig::default(),
            bus_clocks: BusClocksConfig::default(),
        }
    }
}

/// Low-power mode configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LowPowerMode {
    Sleep,      // CPU stops, peripherals run
    Stop,       // All clocks stopped except RTC, SRAM retained
    Standby,    // Deepest sleep, only wakeup pins + RTC active
}

impl Default for LowPowerMode {
    fn default() -> Self {
        LowPowerMode::Sleep
    }
}

/// Wake-up source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WakeupConfig {
    pub wakeup_pin: bool,       // WKUP pin
    pub rtc_alarm: bool,        // RTC alarm
    pub rtc_wakeup: bool,       // RTC wakeup timer
    pub external_interrupt: Option<String>,  // EXTI line
}

impl Default for WakeupConfig {
    fn default() -> Self {
        Self {
            wakeup_pin: true,
            rtc_alarm: false,
            rtc_wakeup: false,
            external_interrupt: None,
        }
    }
}

/// Power estimation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerEstimate {
    pub run_mode_ma: f32,
    pub sleep_mode_ma: f32,
    pub stop_mode_ua: f32,
    pub standby_mode_ua: f32,
    pub estimated_battery_life_hours: Option<f32>,
}

/// Calculate clock frequencies from config
pub fn calculate_clocks(config: &ClockConfig) -> ClockFrequencies {
    let pll_input = match config.pll.source {
        ClockSource::HSI => 16_000_000,
        ClockSource::HSE => config.hse_freq_hz.unwrap_or(8_000_000),
        _ => 16_000_000,
    };
    
    let vco_input = pll_input / config.pll.pllm;
    let vco_output = vco_input * config.pll.plln;
    let pll_output = vco_output / config.pll.pllp;
    let pll_48_clk = vco_output / config.pll.pllq;
    
    let sysclk = match config.sysclk_source {
        ClockSource::HSI => 16_000_000,
        ClockSource::HSE => config.hse_freq_hz.unwrap_or(8_000_000),
        ClockSource::PLL => pll_output,
        _ => 16_000_000,
    };
    
    let hclk = sysclk / config.bus_clocks.ahb_prescaler;
    let pclk1 = hclk / config.bus_clocks.apb1_prescaler;
    let pclk2 = hclk / config.bus_clocks.apb2_prescaler;
    
    ClockFrequencies {
        sysclk,
        hclk,
        pclk1,
        pclk2,
        pll_48_clk,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockFrequencies {
    pub sysclk: u32,
    pub hclk: u32,
    pub pclk1: u32,
    pub pclk2: u32,
    pub pll_48_clk: u32,
}

/// Generate clock initialization code
pub fn generate_clock_init(config: &ClockConfig) -> String {
    let freqs = calculate_clocks(config);
    
    let pll_source_str = match config.pll.source {
        ClockSource::HSI => "RCC_PLLSOURCE_HSI",
        ClockSource::HSE => "RCC_PLLSOURCE_HSE",
        _ => "RCC_PLLSOURCE_HSI",
    };
    
    let sysclk_source_str = match config.sysclk_source {
        ClockSource::HSI => "RCC_SYSCLKSOURCE_HSI",
        ClockSource::HSE => "RCC_SYSCLKSOURCE_HSE",
        ClockSource::PLL => "RCC_SYSCLKSOURCE_PLLCLK",
        _ => "RCC_SYSCLKSOURCE_HSI",
    };
    
    let ahb_div = match config.bus_clocks.ahb_prescaler {
        1 => "RCC_SYSCLK_DIV1",
        2 => "RCC_SYSCLK_DIV2",
        4 => "RCC_SYSCLK_DIV4",
        8 => "RCC_SYSCLK_DIV8",
        16 => "RCC_SYSCLK_DIV16",
        64 => "RCC_SYSCLK_DIV64",
        128 => "RCC_SYSCLK_DIV128",
        256 => "RCC_SYSCLK_DIV256",
        512 => "RCC_SYSCLK_DIV512",
        _ => "RCC_SYSCLK_DIV1",
    };
    
    let apb1_div = match config.bus_clocks.apb1_prescaler {
        1 => "RCC_HCLK_DIV1",
        2 => "RCC_HCLK_DIV2",
        4 => "RCC_HCLK_DIV4",
        8 => "RCC_HCLK_DIV8",
        16 => "RCC_HCLK_DIV16",
        _ => "RCC_HCLK_DIV1",
    };
    
    let apb2_div = match config.bus_clocks.apb2_prescaler {
        1 => "RCC_HCLK_DIV1",
        2 => "RCC_HCLK_DIV2",
        4 => "RCC_HCLK_DIV4",
        8 => "RCC_HCLK_DIV8",
        16 => "RCC_HCLK_DIV16",
        _ => "RCC_HCLK_DIV1",
    };

    format!(r#"/**
 * System Clock Configuration
 * Auto-generated by NeuroBench
 * 
 * SYSCLK = {sysclk} MHz
 * HCLK   = {hclk} MHz (AHB)
 * PCLK1  = {pclk1} MHz (APB1)
 * PCLK2  = {pclk2} MHz (APB2)
 */

void SystemClock_Config(void) {{
    RCC_OscInitTypeDef RCC_OscInitStruct = {{0}};
    RCC_ClkInitTypeDef RCC_ClkInitStruct = {{0}};
    
    // Configure power regulator voltage
    __HAL_RCC_PWR_CLK_ENABLE();
    __HAL_PWR_VOLTAGESCALING_CONFIG(PWR_REGULATOR_VOLTAGE_SCALE1);
    
    // Configure oscillators
    RCC_OscInitStruct.OscillatorType = RCC_OSCILLATORTYPE_HSE | RCC_OSCILLATORTYPE_HSI;
    RCC_OscInitStruct.HSEState = {hse_state};
    RCC_OscInitStruct.HSIState = RCC_HSI_ON;
    RCC_OscInitStruct.HSICalibrationValue = RCC_HSICALIBRATION_DEFAULT;
    RCC_OscInitStruct.PLL.PLLState = RCC_PLL_ON;
    RCC_OscInitStruct.PLL.PLLSource = {pll_source};
    RCC_OscInitStruct.PLL.PLLM = {pllm};
    RCC_OscInitStruct.PLL.PLLN = {plln};
    RCC_OscInitStruct.PLL.PLLP = RCC_PLLP_DIV{pllp};
    RCC_OscInitStruct.PLL.PLLQ = {pllq};
    
    if (HAL_RCC_OscConfig(&RCC_OscInitStruct) != HAL_OK) {{
        Error_Handler();
    }}
    
    // Configure bus clocks
    RCC_ClkInitStruct.ClockType = RCC_CLOCKTYPE_HCLK | RCC_CLOCKTYPE_SYSCLK |
                                  RCC_CLOCKTYPE_PCLK1 | RCC_CLOCKTYPE_PCLK2;
    RCC_ClkInitStruct.SYSCLKSource = {sysclk_source};
    RCC_ClkInitStruct.AHBCLKDivider = {ahb_div};
    RCC_ClkInitStruct.APB1CLKDivider = {apb1_div};
    RCC_ClkInitStruct.APB2CLKDivider = {apb2_div};
    
    // Flash latency for {sysclk}MHz
    if (HAL_RCC_ClockConfig(&RCC_ClkInitStruct, {flash_latency}) != HAL_OK) {{
        Error_Handler();
    }}
}}
"#,
        sysclk = freqs.sysclk / 1_000_000,
        hclk = freqs.hclk / 1_000_000,
        pclk1 = freqs.pclk1 / 1_000_000,
        pclk2 = freqs.pclk2 / 1_000_000,
        hse_state = if config.hse_freq_hz.is_some() { "RCC_HSE_ON" } else { "RCC_HSE_OFF" },
        pll_source = pll_source_str,
        pllm = config.pll.pllm,
        plln = config.pll.plln,
        pllp = config.pll.pllp,
        pllq = config.pll.pllq,
        sysclk_source = sysclk_source_str,
        ahb_div = ahb_div,
        apb1_div = apb1_div,
        apb2_div = apb2_div,
        flash_latency = calculate_flash_latency(freqs.sysclk),
    )
}

fn calculate_flash_latency(sysclk_hz: u32) -> &'static str {
    let mhz = sysclk_hz / 1_000_000;
    match mhz {
        0..=30 => "FLASH_LATENCY_0",
        31..=60 => "FLASH_LATENCY_1",
        61..=90 => "FLASH_LATENCY_2",
        91..=120 => "FLASH_LATENCY_3",
        121..=150 => "FLASH_LATENCY_4",
        _ => "FLASH_LATENCY_5",
    }
}

/// Generate low-power mode code
pub fn generate_low_power_code(mode: LowPowerMode, wakeup: &WakeupConfig) -> String {
    match mode {
        LowPowerMode::Sleep => generate_sleep_mode(wakeup),
        LowPowerMode::Stop => generate_stop_mode(wakeup),
        LowPowerMode::Standby => generate_standby_mode(wakeup),
    }
}

fn generate_sleep_mode(_wakeup: &WakeupConfig) -> String {
    format!(r#"/**
 * Enter Sleep Mode
 * CPU halted, peripherals still running
 * Wake on any interrupt
 */

void Enter_SleepMode(void) {{
    // Suspend SysTick to prevent wakeup
    HAL_SuspendTick();
    
    // Enter Sleep Mode
    HAL_PWR_EnterSLEEPMode(PWR_MAINREGULATOR_ON, PWR_SLEEPENTRY_WFI);
    
    // Resume after wakeup
    HAL_ResumeTick();
}}

// Wake sources: Any interrupt
// Typical current: ~1-2mA (depends on active peripherals)
"#)
}

fn generate_stop_mode(wakeup: &WakeupConfig) -> String {
    let wakeup_sources = build_wakeup_config(wakeup);
    
    format!(r#"/**
 * Enter Stop Mode
 * All clocks stopped, SRAM and registers retained
 * Wake on EXTI, RTC alarm, or WKUP pin
 */

void Enter_StopMode(void) {{
    // Disable SysTick
    HAL_SuspendTick();
    
{wakeup_config}
    
    // Enter Stop Mode with low-power regulator
    HAL_PWR_EnterSTOPMode(PWR_LOWPOWERREGULATOR_ON, PWR_STOPENTRY_WFI);
    
    // After wakeup: restore clocks
    SystemClock_Config();
    HAL_ResumeTick();
}}

// Typical current: ~10-20µA
// Wake sources: {sources}
"#,
        wakeup_config = wakeup_sources.0,
        sources = wakeup_sources.1,
    )
}

fn generate_standby_mode(wakeup: &WakeupConfig) -> String {
    let wakeup_sources = build_wakeup_config(wakeup);
    
    format!(r#"/**
 * Enter Standby Mode
 * Lowest power consumption, RAM lost
 * Wake on WKUP pin or RTC alarm (reset on wakeup)
 */

void Enter_StandbyMode(void) {{
    // Clear wakeup flags
    __HAL_PWR_CLEAR_FLAG(PWR_FLAG_WU);
    
{wakeup_config}
    
    // Enter Standby Mode
    HAL_PWR_EnterSTANDBYMode();
    
    // Note: Code below will NOT execute
    // MCU resets on wakeup from Standby
}}

// Typical current: ~2-3µA
// Wake sources: {sources}
// WARNING: All RAM content is lost!
"#,
        wakeup_config = wakeup_sources.0,
        sources = wakeup_sources.1,
    )
}

fn build_wakeup_config(wakeup: &WakeupConfig) -> (String, String) {
    let mut config_lines: Vec<String> = Vec::new();
    let mut sources = Vec::new();
    
    if wakeup.wakeup_pin {
        config_lines.push("    // Enable WKUP pin\n    HAL_PWR_EnableWakeUpPin(PWR_WAKEUP_PIN1);".to_string());
        sources.push("WKUP Pin");
    }
    
    if wakeup.rtc_alarm {
        config_lines.push("    // RTC Alarm wakeup (configure RTC separately)".to_string());
        sources.push("RTC Alarm");
    }
    
    if wakeup.rtc_wakeup {
        config_lines.push("    // RTC Wakeup timer (configure RTC separately)".to_string());
        sources.push("RTC Wakeup");
    }
    
    if let Some(exti) = &wakeup.external_interrupt {
        config_lines.push(format!("    // EXTI {} interrupt", exti));
        sources.push("EXTI");
    }
    
    (config_lines.join("\n"), sources.join(", "))
}

/// Estimate power consumption
pub fn estimate_power(config: &ClockConfig, _mode: LowPowerMode) -> PowerEstimate {
    let freqs = calculate_clocks(config);
    let mhz = freqs.sysclk / 1_000_000;
    
    // Rough estimates for STM32F4
    let run_mode_ma = 0.15 * mhz as f32 + 5.0;  // ~18mA at 84MHz
    let sleep_mode_ma = run_mode_ma * 0.15;     // ~2-3mA
    let stop_mode_ua = 12.0;                     // ~12µA
    let standby_mode_ua = 2.5;                   // ~2.5µA
    
    PowerEstimate {
        run_mode_ma,
        sleep_mode_ma,
        stop_mode_ua,
        standby_mode_ua,
        estimated_battery_life_hours: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_clock_calculation() {
        let config = ClockConfig::default();
        let freqs = calculate_clocks(&config);
        
        // Default should give 84MHz SYSCLK
        assert_eq!(freqs.sysclk, 84_000_000);
        assert_eq!(freqs.pclk1, 42_000_000);  // APB1 = SYSCLK / 2
        assert_eq!(freqs.pclk2, 84_000_000);  // APB2 = SYSCLK
    }
    
    #[test]
    fn test_clock_init_generation() {
        let config = ClockConfig::default();
        let code = generate_clock_init(&config);
        
        assert!(code.contains("SystemClock_Config"));
        assert!(code.contains("PLLM = 16"));
        assert!(code.contains("PLLN = 336"));
    }
    
    #[test]
    fn test_low_power_generation() {
        let wakeup = WakeupConfig::default();
        let code = generate_low_power_code(LowPowerMode::Stop, &wakeup);
        
        assert!(code.contains("Enter_StopMode"));
        assert!(code.contains("PWR_LOWPOWERREGULATOR_ON"));
    }
}
