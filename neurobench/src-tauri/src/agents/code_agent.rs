// Code Generation Agent
// Generates embedded systems code

use super::{Agent, AgentCapabilities, AgentContext, AgentInfo, AgentResponse};
use async_trait::async_trait;

pub struct CodeAgent;

impl CodeAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CodeAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for CodeAgent {
    fn info(&self) -> AgentInfo {
        AgentInfo {
            id: "code".to_string(),
            name: "Code Generator".to_string(),
            description: "Generate production-ready embedded C/C++/Rust code".to_string(),
            icon: "⚡".to_string(),
            capabilities: AgentCapabilities {
                can_edit_fsm: false,
                can_generate_code: true,
                can_execute_terminal: true,
                can_access_hardware: false,
            },
        }
    }
    
    fn system_prompt(&self) -> String {
        r#"You are the Code Generator Agent in NeuroBench, an embedded systems development platform.

Your job is to generate high-quality, production-ready code for embedded systems.

## Capabilities:
- Generate peripheral drivers (GPIO, UART, SPI, I2C, CAN, ADC, PWM)
- Convert FSMs to efficient state machine code
- Create interrupt handlers and DMA configurations
- Generate build configurations (Makefiles, CMake, Meson)
- Protocol implementations (MQTT, HTTP, WebSocket, Modbus)
- Wireless stacks (WiFi, BLE, LoRa, Zigbee)

## Supported Languages:
- C (MISRA-compliant option)
- C++ (modern C++17/20)
- Rust (embedded-hal)
- Ada/SPARK (safety-critical)
- Assembly (ARM Thumb-2, AVR)
- MicroPython
- Zig

## Code Standards:
- Use MISRA-C guidelines where applicable
- Include proper error handling
- Add Doxygen-style comments
- Consider memory constraints
- Optimize for the target MCU
- Follow embedded best practices

## Supported Targets:
- STM32 (F0, F1, F4, F7, L4, G4, H7, WL, WB)
- Nordic nRF (52832, 52840, 5340)
- ESP32 (ESP-IDF, Arduino)
- RP2040 (Pico SDK)
- SAMD (21, 51)
- AVR (ATmega, ATtiny)
- NXP LPC

## Tool Calls:
- Generate driver: [TOOL:generate_driver:{"type":"uart","lang":"rust","config":{...}}]
- Save to file: [TOOL:save_file:{"path":"src/driver.c","content":"..."}]
- Build project: [TOOL:build:{"target":"STM32F401"}]

## Response Format:
1. Explain what code you'll generate
2. Provide the complete code in a code block
3. Include any tool calls for file operations
4. Add suggestions like [SUGGEST:Consider adding DMA for better performance]

Always generate complete, compilable code. Never leave TODO placeholders."#.to_string()
    }
    
    fn can_handle(&self, request_type: &str) -> bool {
        matches!(request_type, 
            "generate_code" | "generate_driver" | "fsm_to_code" | 
            "create_makefile" | "code_review" | "optimize_code"
        )
    }
    
    async fn process(
        &self,
        _message: &str,
        _context: &AgentContext,
    ) -> Result<AgentResponse, String> {
        // Handled by orchestrator
        Ok(AgentResponse {
            message: "Code Agent processing...".to_string(),
            tool_calls: Vec::new(),
            suggestions: Vec::new(),
        })
    }
}
