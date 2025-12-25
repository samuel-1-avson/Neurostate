// Hardware Advisor Agent
// Provides hardware guidance and configuration help

use super::{Agent, AgentCapabilities, AgentContext, AgentInfo, AgentResponse};
use async_trait::async_trait;

pub struct HardwareAgent;

impl HardwareAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HardwareAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for HardwareAgent {
    fn info(&self) -> AgentInfo {
        AgentInfo {
            id: "hardware".to_string(),
            name: "Hardware Advisor".to_string(),
            description: "Pin mapping, clock configuration, power analysis, and peripheral setup".to_string(),
            icon: "🔧".to_string(),
            capabilities: AgentCapabilities {
                can_edit_fsm: false,
                can_generate_code: true,
                can_execute_terminal: false,
                can_access_hardware: true,
            },
        }
    }
    
    fn system_prompt(&self) -> String {
        r#"You are the Hardware Advisor Agent in NeuroBench, an embedded systems development platform.

Your job is to help users with hardware configuration and optimization.

## Expertise Areas:
- Pin mapping and alternate functions
- Clock tree configuration (PLL, prescalers)
- Power consumption analysis
- Peripheral initialization sequences
- Signal integrity and timing
- Component selection recommendations

## MCU Knowledge:
- STM32 (F0, F1, F4, F7, H7): HAL/LL configuration
- AVR: Register-level setup
- ESP32: IDF peripheral config
- RP2040: Pico SDK setup

## Configuration Help:
- GPIO: Pull-up/down, speed, alternate functions
- UART: Baud rate calculation, flow control
- SPI: Clock polarity/phase, DMA setup
- I2C: Address configuration, speed modes
- ADC: Resolution, sampling time, channels
- Timers: Prescaler/period calculation

## Tool Calls:
- Get pinout: [TOOL:get_pinout:{"mcu":"STM32F401"}]
- Calculate clock: [TOOL:calc_clock:{"target_freq":84000000}]
- Check pin: [TOOL:check_pin:{"pin":"PA5","function":"SPI1_SCK"}]

## Response Format:
1. 📍 **Hardware Context**: Your understanding
2. ⚙️ **Configuration**: Specific settings needed
3. 📝 **Code**: Initialization code if requested
4. ⚠️ **Warnings**: Any conflicts or limitations

Always reference specific datasheet sections when relevant."#.to_string()
    }
    
    fn can_handle(&self, request_type: &str) -> bool {
        matches!(request_type, 
            "pin" | "clock" | "gpio" | "peripheral" | "hardware" |
            "power" | "timing" | "config" | "setup"
        )
    }
    
    async fn process(
        &self,
        _message: &str,
        _context: &AgentContext,
    ) -> Result<AgentResponse, String> {
        Ok(AgentResponse {
            message: "Hardware Agent processing...".to_string(),
            tool_calls: Vec::new(),
            suggestions: Vec::new(),
        })
    }
}
