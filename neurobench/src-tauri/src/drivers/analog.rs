// Analog I/O Driver Generation
// Based on Chapters 4-5: Analog Output and Analog Input

use serde::{Deserialize, Serialize};

/// ADC Resolution options
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AdcResolution {
    Bits8,
    Bits10,
    Bits12,
}

impl Default for AdcResolution {
    fn default() -> Self {
        AdcResolution::Bits12
    }
}

/// ADC Sample time options (cycles)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AdcSampleTime {
    Cycles3,
    Cycles15,
    Cycles28,
    Cycles56,
    Cycles84,
    Cycles112,
    Cycles144,
    Cycles480,
}

impl Default for AdcSampleTime {
    fn default() -> Self {
        AdcSampleTime::Cycles84
    }
}

/// ADC Channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdcChannelConfig {
    pub channel: u8,           // 0-15
    pub sample_time: AdcSampleTime,
    pub gpio_pin: String,      // e.g., "PA0"
}

/// ADC Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdcConfig {
    pub instance: String,      // "ADC1", "ADC2", "ADC3"
    pub resolution: AdcResolution,
    pub channels: Vec<AdcChannelConfig>,
    pub continuous_mode: bool,
    pub dma_enabled: bool,
    pub scan_mode: bool,
}

impl Default for AdcConfig {
    fn default() -> Self {
        Self {
            instance: "ADC1".to_string(),
            resolution: AdcResolution::Bits12,
            channels: vec![AdcChannelConfig {
                channel: 0,
                sample_time: AdcSampleTime::Cycles84,
                gpio_pin: "PA0".to_string(),
            }],
            continuous_mode: false,
            dma_enabled: false,
            scan_mode: false,
        }
    }
}

/// DAC Waveform type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DacWaveform {
    None,
    Noise,
    Triangle,
}

impl Default for DacWaveform {
    fn default() -> Self {
        DacWaveform::None
    }
}

/// DAC Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DacConfig {
    pub channel: u8,           // 1 or 2
    pub output_buffer: bool,   // Enable output buffer
    pub trigger_enabled: bool,
    pub waveform: DacWaveform,
    pub amplitude: u8,         // 0-11 for triangle/noise
}

impl Default for DacConfig {
    fn default() -> Self {
        Self {
            channel: 1,
            output_buffer: true,
            trigger_enabled: false,
            waveform: DacWaveform::None,
            amplitude: 0,
        }
    }
}

/// PWM Mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PwmMode {
    EdgeAligned,
    CenterAligned,
}

impl Default for PwmMode {
    fn default() -> Self {
        PwmMode::EdgeAligned
    }
}

/// PWM Channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PwmChannelConfig {
    pub channel: u8,           // 1-4
    pub duty_cycle_percent: f32,
    pub gpio_pin: String,
    pub polarity_high: bool,
}

/// PWM Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PwmConfig {
    pub timer: String,         // "TIM1", "TIM2", etc.
    pub frequency_hz: u32,
    pub mode: PwmMode,
    pub channels: Vec<PwmChannelConfig>,
    pub dead_time_ns: Option<u32>,  // For complementary outputs
}

impl Default for PwmConfig {
    fn default() -> Self {
        Self {
            timer: "TIM2".to_string(),
            frequency_hz: 1000,    // 1kHz default
            mode: PwmMode::EdgeAligned,
            channels: vec![PwmChannelConfig {
                channel: 1,
                duty_cycle_percent: 50.0,
                gpio_pin: "PA0".to_string(),
                polarity_high: true,
            }],
            dead_time_ns: None,
        }
    }
}

/// Generate ADC initialization code
pub fn generate_adc_init(config: &AdcConfig, _clock_hz: u32) -> String {
    let resolution_str = match config.resolution {
        AdcResolution::Bits8 => "ADC_RESOLUTION_8B",
        AdcResolution::Bits10 => "ADC_RESOLUTION_10B",
        AdcResolution::Bits12 => "ADC_RESOLUTION_12B",
    };
    
    let max_value = match config.resolution {
        AdcResolution::Bits8 => 255,
        AdcResolution::Bits10 => 1023,
        AdcResolution::Bits12 => 4095,
    };
    
    let channel_configs: String = config.channels.iter().map(|ch| {
        let sample_str = match ch.sample_time {
            AdcSampleTime::Cycles3 => "ADC_SAMPLETIME_3CYCLES",
            AdcSampleTime::Cycles15 => "ADC_SAMPLETIME_15CYCLES",
            AdcSampleTime::Cycles28 => "ADC_SAMPLETIME_28CYCLES",
            AdcSampleTime::Cycles56 => "ADC_SAMPLETIME_56CYCLES",
            AdcSampleTime::Cycles84 => "ADC_SAMPLETIME_84CYCLES",
            AdcSampleTime::Cycles112 => "ADC_SAMPLETIME_112CYCLES",
            AdcSampleTime::Cycles144 => "ADC_SAMPLETIME_144CYCLES",
            AdcSampleTime::Cycles480 => "ADC_SAMPLETIME_480CYCLES",
        };
        format!(r#"
    // Channel {} ({})
    sConfig.Channel = ADC_CHANNEL_{};
    sConfig.Rank = {};
    sConfig.SamplingTime = {};
    HAL_ADC_ConfigChannel(&h{}, &sConfig);"#,
            ch.channel, ch.gpio_pin, ch.channel, 
            config.channels.iter().position(|c| c.channel == ch.channel).unwrap_or(0) + 1,
            sample_str, config.instance.to_lowercase()
        )
    }).collect();

    format!(r#"/**
 * ADC Configuration for {instance}
 * Auto-generated by NeuroBench
 * Resolution: {bits}-bit (0-{max_value})
 * Channels: {num_channels}
 */

ADC_HandleTypeDef h{instance_lower};

void {instance}_Init(void) {{
    ADC_ChannelConfTypeDef sConfig = {{0}};
    
    // Enable ADC clock
    __HAL_RCC_{instance}_CLK_ENABLE();
    
    // Configure ADC
    h{instance_lower}.Instance = {instance};
    h{instance_lower}.Init.ClockPrescaler = ADC_CLOCK_SYNC_PCLK_DIV4;
    h{instance_lower}.Init.Resolution = {resolution};
    h{instance_lower}.Init.ScanConvMode = {scan_mode};
    h{instance_lower}.Init.ContinuousConvMode = {continuous};
    h{instance_lower}.Init.DiscontinuousConvMode = DISABLE;
    h{instance_lower}.Init.ExternalTrigConvEdge = ADC_EXTERNALTRIGCONVEDGE_NONE;
    h{instance_lower}.Init.ExternalTrigConv = ADC_SOFTWARE_START;
    h{instance_lower}.Init.DataAlign = ADC_DATAALIGN_RIGHT;
    h{instance_lower}.Init.NbrOfConversion = {num_channels};
    h{instance_lower}.Init.DMAContinuousRequests = {dma};
    h{instance_lower}.Init.EOCSelection = ADC_EOC_SINGLE_CONV;
    
    if (HAL_ADC_Init(&h{instance_lower}) != HAL_OK) {{
        Error_Handler();
    }}
    
    // Configure channels{channels}
}}

/**
 * Read single ADC value (blocking)
 */
uint16_t {instance}_Read(uint8_t channel) {{
    HAL_ADC_Start(&h{instance_lower});
    HAL_ADC_PollForConversion(&h{instance_lower}, HAL_MAX_DELAY);
    uint16_t value = HAL_ADC_GetValue(&h{instance_lower});
    HAL_ADC_Stop(&h{instance_lower});
    return value;
}}

/**
 * Convert ADC value to voltage (mV)
 */
uint32_t {instance}_ToMillivolts(uint16_t adc_value) {{
    // Assuming 3.3V reference
    return (uint32_t)adc_value * 3300 / {max_value};
}}
"#,
        instance = config.instance,
        instance_lower = config.instance.to_lowercase(),
        bits = match config.resolution {
            AdcResolution::Bits8 => 8,
            AdcResolution::Bits10 => 10,
            AdcResolution::Bits12 => 12,
        },
        max_value = max_value,
        num_channels = config.channels.len(),
        resolution = resolution_str,
        scan_mode = if config.scan_mode { "ENABLE" } else { "DISABLE" },
        continuous = if config.continuous_mode { "ENABLE" } else { "DISABLE" },
        dma = if config.dma_enabled { "ENABLE" } else { "DISABLE" },
        channels = channel_configs,
    )
}

/// Generate DAC initialization code
pub fn generate_dac_init(config: &DacConfig) -> String {
    let channel_str = match config.channel {
        1 => "DAC_CHANNEL_1",
        2 => "DAC_CHANNEL_2",
        _ => "DAC_CHANNEL_1",
    };
    
    let _waveform_str = match config.waveform {
        DacWaveform::None => "DAC_WAVE_NONE",
        DacWaveform::Noise => "DAC_WAVE_NOISE",
        DacWaveform::Triangle => "DAC_WAVE_TRIANGLE",
    };

    format!(r#"/**
 * DAC Configuration for Channel {channel}
 * Auto-generated by NeuroBench
 * Output Pin: PA{pin}
 */

DAC_HandleTypeDef hdac;

void DAC_CH{channel}_Init(void) {{
    DAC_ChannelConfTypeDef sConfig = {{0}};
    
    // Enable DAC clock
    __HAL_RCC_DAC_CLK_ENABLE();
    
    // Configure GPIO for DAC output
    GPIO_InitTypeDef GPIO_InitStruct = {{0}};
    __HAL_RCC_GPIOA_CLK_ENABLE();
    GPIO_InitStruct.Pin = GPIO_PIN_{pin};
    GPIO_InitStruct.Mode = GPIO_MODE_ANALOG;
    GPIO_InitStruct.Pull = GPIO_NOPULL;
    HAL_GPIO_Init(GPIOA, &GPIO_InitStruct);
    
    // Initialize DAC
    hdac.Instance = DAC;
    if (HAL_DAC_Init(&hdac) != HAL_OK) {{
        Error_Handler();
    }}
    
    // Configure DAC channel
    sConfig.DAC_Trigger = {trigger};
    sConfig.DAC_OutputBuffer = {buffer};
    if (HAL_DAC_ConfigChannel(&hdac, &sConfig, {channel_enum}) != HAL_OK) {{
        Error_Handler();
    }}
    
    // Start DAC
    HAL_DAC_Start(&hdac, {channel_enum});
}}

/**
 * Set DAC output value (0-4095 for 12-bit)
 */
void DAC_CH{channel}_SetValue(uint16_t value) {{
    HAL_DAC_SetValue(&hdac, {channel_enum}, DAC_ALIGN_12B_R, value & 0x0FFF);
}}

/**
 * Set DAC output voltage in millivolts (0-3300mV)
 */
void DAC_CH{channel}_SetMillivolts(uint16_t mv) {{
    uint16_t value = (uint32_t)mv * 4095 / 3300;
    DAC_CH{channel}_SetValue(value);
}}
"#,
        channel = config.channel,
        pin = if config.channel == 1 { 4 } else { 5 },
        trigger = if config.trigger_enabled { "DAC_TRIGGER_SOFTWARE" } else { "DAC_TRIGGER_NONE" },
        buffer = if config.output_buffer { "DAC_OUTPUTBUFFER_ENABLE" } else { "DAC_OUTPUTBUFFER_DISABLE" },
        channel_enum = channel_str,
    )
}

/// Generate PWM initialization code
pub fn generate_pwm_init(config: &PwmConfig, timer_clock_hz: u32) -> String {
    // Calculate prescaler and period for desired frequency
    let target_freq = config.frequency_hz;
    let prescaler = (timer_clock_hz / (target_freq * 65536)).max(1);
    let period = timer_clock_hz / (prescaler * target_freq);
    
    let mode_str = match config.mode {
        PwmMode::EdgeAligned => "TIM_COUNTERMODE_UP",
        PwmMode::CenterAligned => "TIM_COUNTERMODE_CENTERALIGNED1",
    };
    
    let channel_inits: String = config.channels.iter().map(|ch| {
        let channel_enum = format!("TIM_CHANNEL_{}", ch.channel);
        let pulse = (period as f32 * ch.duty_cycle_percent / 100.0) as u32;
        format!(r#"
    // Channel {} - {}% duty cycle
    sConfigOC.Pulse = {};
    sConfigOC.OCPolarity = {};
    HAL_TIM_PWM_ConfigChannel(&h{}, &sConfigOC, {});
    HAL_TIM_PWM_Start(&h{}, {});"#,
            ch.channel, ch.duty_cycle_percent, pulse,
            if ch.polarity_high { "TIM_OCPOLARITY_HIGH" } else { "TIM_OCPOLARITY_LOW" },
            config.timer.to_lowercase(), channel_enum,
            config.timer.to_lowercase(), channel_enum,
        )
    }).collect();

    format!(r#"/**
 * PWM Configuration for {timer}
 * Auto-generated by NeuroBench
 * Frequency: {freq} Hz
 * Channels: {num_channels}
 */

TIM_HandleTypeDef h{timer_lower};

void {timer}_PWM_Init(void) {{
    TIM_OC_InitTypeDef sConfigOC = {{0}};
    
    // Enable timer clock
    __HAL_RCC_{timer}_CLK_ENABLE();
    
    // Configure timer base
    h{timer_lower}.Instance = {timer};
    h{timer_lower}.Init.Prescaler = {prescaler} - 1;
    h{timer_lower}.Init.CounterMode = {mode};
    h{timer_lower}.Init.Period = {period} - 1;
    h{timer_lower}.Init.ClockDivision = TIM_CLOCKDIVISION_DIV1;
    h{timer_lower}.Init.AutoReloadPreload = TIM_AUTORELOAD_PRELOAD_ENABLE;
    
    if (HAL_TIM_PWM_Init(&h{timer_lower}) != HAL_OK) {{
        Error_Handler();
    }}
    
    // Configure PWM channels
    sConfigOC.OCMode = TIM_OCMODE_PWM1;
    sConfigOC.OCFastMode = TIM_OCFAST_DISABLE;
{channels}
}}

/**
 * Set PWM duty cycle (0-100%)
 */
void {timer}_SetDuty(uint8_t channel, float duty_percent) {{
    uint32_t pulse = (uint32_t)({period} * duty_percent / 100.0f);
    __HAL_TIM_SET_COMPARE(&h{timer_lower}, channel, pulse);
}}

/**
 * Set PWM frequency (reconfigures timer)
 */
void {timer}_SetFrequency(uint32_t freq_hz) {{
    uint32_t new_period = {clock} / ({prescaler} * freq_hz);
    __HAL_TIM_SET_AUTORELOAD(&h{timer_lower}, new_period - 1);
}}
"#,
        timer = config.timer,
        timer_lower = config.timer.to_lowercase(),
        freq = config.frequency_hz,
        num_channels = config.channels.len(),
        prescaler = prescaler,
        period = period,
        mode = mode_str,
        channels = channel_inits,
        clock = timer_clock_hz,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_adc_generation() {
        let config = AdcConfig::default();
        let code = generate_adc_init(&config, 84_000_000);
        assert!(code.contains("ADC1_Init"));
        assert!(code.contains("ADC_RESOLUTION_12B"));
    }
    
    #[test]
    fn test_dac_generation() {
        let config = DacConfig::default();
        let code = generate_dac_init(&config);
        assert!(code.contains("DAC_CH1_Init"));
        assert!(code.contains("DAC_CHANNEL_1"));
    }
    
    #[test]
    fn test_pwm_generation() {
        let config = PwmConfig::default();
        let code = generate_pwm_init(&config, 84_000_000);
        assert!(code.contains("TIM2_PWM_Init"));
        assert!(code.contains("HAL_TIM_PWM_Start"));
    }
}
