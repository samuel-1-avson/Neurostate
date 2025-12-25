// Interrupt Driver Generation
// Based on Chapter 9: Interrupts, Timers, and Tasks

use serde::{Deserialize, Serialize};

/// Interrupt edge trigger type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterruptEdge {
    Rising,
    Falling,
    Both,
}

impl Default for InterruptEdge {
    fn default() -> Self {
        InterruptEdge::Rising
    }
}

/// Interrupt configuration for a pin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterruptConfig {
    pub pin: String,           // e.g., "PA0", "PB5"
    pub edge: InterruptEdge,
    pub priority: u8,          // NVIC priority 0-15
    pub debounce_ms: u32,      // Debounce time in ms (0 = disabled)
    pub handler_name: String,  // ISR function name
    pub handler_code: String,  // Code to execute in ISR
}

impl Default for InterruptConfig {
    fn default() -> Self {
        Self {
            pin: "PA0".to_string(),
            edge: InterruptEdge::Rising,
            priority: 5,
            debounce_ms: 50,
            handler_name: "EXTI0_IRQHandler".to_string(),
            handler_code: "// Handle interrupt\nfsm_trigger_event(EVT_BUTTON);".to_string(),
        }
    }
}

/// Timer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerConfig {
    pub instance: String,      // e.g., "TIM2", "TIM3"
    pub prescaler: u32,
    pub period: u32,
    pub auto_reload: bool,
    pub interrupt_enabled: bool,
    pub handler_name: String,
    pub handler_code: String,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            instance: "TIM2".to_string(),
            prescaler: 8400,       // For 84MHz clock -> 10kHz
            period: 10000,          // 10000 ticks = 1 second
            auto_reload: true,
            interrupt_enabled: true,
            handler_name: "TIM2_IRQHandler".to_string(),
            handler_code: "system_tick++;".to_string(),
        }
    }
}

/// Ticker (periodic callback) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerConfig {
    pub name: String,
    pub interval_ms: u32,
    pub callback_code: String,
}

impl Default for TickerConfig {
    fn default() -> Self {
        Self {
            name: "main_ticker".to_string(),
            interval_ms: 1,
            callback_code: "// 1ms periodic task".to_string(),
        }
    }
}

/// Generate EXTI interrupt initialization code for STM32
pub fn generate_exti_init(config: &InterruptConfig, _mcu: &str) -> String {
    let port_letter = config.pin.chars().nth(1).unwrap_or('A');
    let pin_num: u32 = config.pin[2..].parse().unwrap_or(0);
    
    let edge_config = match config.edge {
        InterruptEdge::Rising => ("EXTI_TRIGGER_RISING", "GPIO_MODE_IT_RISING"),
        InterruptEdge::Falling => ("EXTI_TRIGGER_FALLING", "GPIO_MODE_IT_FALLING"),
        InterruptEdge::Both => ("EXTI_TRIGGER_RISING_FALLING", "GPIO_MODE_IT_RISING_FALLING"),
    };
    
    let irq_name = match pin_num {
        0 => "EXTI0_IRQn",
        1 => "EXTI1_IRQn",
        2 => "EXTI2_IRQn",
        3 => "EXTI3_IRQn",
        4 => "EXTI4_IRQn",
        5..=9 => "EXTI9_5_IRQn",
        _ => "EXTI15_10_IRQn",
    };
    
    format!(r#"// EXTI Interrupt Configuration for {pin}
// Priority: {priority}, Edge: {edge:?}

void {handler}_Init(void) {{
    GPIO_InitTypeDef GPIO_InitStruct = {{0}};
    
    // Enable GPIO clock
    __HAL_RCC_GPIO{port}_CLK_ENABLE();
    
    // Configure GPIO pin as interrupt
    GPIO_InitStruct.Pin = GPIO_PIN_{pin_num};
    GPIO_InitStruct.Mode = {mode};
    GPIO_InitStruct.Pull = GPIO_PULLUP;
    HAL_GPIO_Init(GPIO{port}, &GPIO_InitStruct);
    
    // Configure NVIC
    HAL_NVIC_SetPriority({irq}, {priority}, 0);
    HAL_NVIC_EnableIRQ({irq});
}}

{debounce_vars}

void {handler}(void) {{
    if (__HAL_GPIO_EXTI_GET_IT(GPIO_PIN_{pin_num}) != RESET) {{
        __HAL_GPIO_EXTI_CLEAR_IT(GPIO_PIN_{pin_num});
        {debounce_check}
        {code}
    }}
}}
"#,
        pin = config.pin,
        priority = config.priority,
        edge = config.edge,
        handler = config.handler_name.replace("_IRQHandler", ""),
        port = port_letter,
        pin_num = pin_num,
        mode = edge_config.1,
        irq = irq_name,
        debounce_vars = if config.debounce_ms > 0 {
            format!("static uint32_t last_{}_time = 0;", config.handler_name.to_lowercase())
        } else {
            String::new()
        },
        debounce_check = if config.debounce_ms > 0 {
            format!(r#"
        uint32_t now = HAL_GetTick();
        if (now - last_{handler}_time < {ms}) return;
        last_{handler}_time = now;"#, 
                handler = config.handler_name.to_lowercase(),
                ms = config.debounce_ms
            )
        } else {
            String::new()
        },
        code = config.handler_code.lines()
            .map(|l| format!("        {}", l))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

/// Generate Timer initialization code for STM32
pub fn generate_timer_init(config: &TimerConfig, clock_hz: u32) -> String {
    let timer_num = config.instance.replace("TIM", "");
    let irq_name = format!("TIM{}_IRQn", timer_num);
    
    format!(r#"// Timer Configuration for {instance}
// Prescaler: {prescaler}, Period: {period}
// Frequency: {freq:.2} Hz

TIM_HandleTypeDef h{instance_lower};

void {instance}_Init(void) {{
    __HAL_RCC_{instance}_CLK_ENABLE();
    
    h{instance_lower}.Instance = {instance};
    h{instance_lower}.Init.Prescaler = {prescaler} - 1;
    h{instance_lower}.Init.CounterMode = TIM_COUNTERMODE_UP;
    h{instance_lower}.Init.Period = {period} - 1;
    h{instance_lower}.Init.ClockDivision = TIM_CLOCKDIVISION_DIV1;
    h{instance_lower}.Init.AutoReloadPreload = {auto_reload};
    
    if (HAL_TIM_Base_Init(&h{instance_lower}) != HAL_OK) {{
        Error_Handler();
    }}
    
    {irq_setup}
    
    HAL_TIM_Base_Start{start_mode}(&h{instance_lower});
}}

{isr}
"#,
        instance = config.instance,
        instance_lower = config.instance.to_lowercase(),
        prescaler = config.prescaler,
        period = config.period,
        freq = clock_hz as f64 / (config.prescaler as f64 * config.period as f64),
        auto_reload = if config.auto_reload { "TIM_AUTORELOAD_PRELOAD_ENABLE" } else { "TIM_AUTORELOAD_PRELOAD_DISABLE" },
        irq_setup = if config.interrupt_enabled {
            format!(r#"HAL_NVIC_SetPriority({irq}, 5, 0);
    HAL_NVIC_EnableIRQ({irq});"#, irq = irq_name)
        } else {
            String::new()
        },
        start_mode = if config.interrupt_enabled { "_IT" } else { "" },
        isr = if config.interrupt_enabled {
            format!(r#"void {handler}(void) {{
    if (__HAL_TIM_GET_FLAG(&h{instance_lower}, TIM_FLAG_UPDATE) != RESET) {{
        __HAL_TIM_CLEAR_FLAG(&h{instance_lower}, TIM_FLAG_UPDATE);
        {code}
    }}
}}"#,
                handler = config.handler_name,
                instance_lower = config.instance.to_lowercase(),
                code = config.handler_code,
            )
        } else {
            String::new()
        },
    )
}

/// Generate SysTick-based ticker code
pub fn generate_ticker(config: &TickerConfig) -> String {
    format!(r#"// Ticker: {name}
// Interval: {interval}ms

volatile uint32_t {name}_counter = 0;
volatile uint32_t system_tick = 0;

void SysTick_Handler(void) {{
    HAL_IncTick();
    system_tick++;
    
    // {name} - every {interval}ms
    if (++{name}_counter >= {interval}) {{
        {name}_counter = 0;
        {code}
    }}
}}
"#,
        name = config.name,
        interval = config.interval_ms,
        code = config.callback_code,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exti_generation() {
        let config = InterruptConfig::default();
        let code = generate_exti_init(&config, "STM32F401");
        assert!(code.contains("EXTI0_IRQn"));
        assert!(code.contains("GPIO_MODE_IT_RISING"));
    }
    
    #[test]
    fn test_timer_generation() {
        let config = TimerConfig::default();
        let code = generate_timer_init(&config, 84_000_000);
        assert!(code.contains("TIM2"));
        assert!(code.contains("HAL_TIM_Base_Init"));
    }
    
    #[test]
    fn test_ticker_generation() {
        let config = TickerConfig::default();
        let code = generate_ticker(&config);
        assert!(code.contains("SysTick_Handler"));
    }
}
