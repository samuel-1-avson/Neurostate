// Debug Assistant Agent
// Helps diagnose and fix issues in embedded code

use super::{Agent, AgentCapabilities, AgentContext, AgentInfo, AgentResponse};
use async_trait::async_trait;

pub struct DebugAgent;

impl DebugAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DebugAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for DebugAgent {
    fn info(&self) -> AgentInfo {
        AgentInfo {
            id: "debug".to_string(),
            name: "Debug Assistant".to_string(),
            description: "Diagnose issues, analyze errors, and fix bugs in embedded code".to_string(),
            icon: "🔍".to_string(),
            capabilities: AgentCapabilities {
                can_edit_fsm: true,
                can_generate_code: true,
                can_execute_terminal: true,
                can_access_hardware: true,
            },
        }
    }
    
    fn system_prompt(&self) -> String {
        r#"You are the Debug Assistant Agent in NeuroBench, an embedded systems development platform.

Your job is to help users diagnose and fix issues in their embedded systems code.

## Expertise Areas:
- Compiler error analysis and resolution
- Runtime debugging (hard faults, stack overflows, memory issues)
- Peripheral configuration issues
- Timing and interrupt problems
- Race conditions and concurrency bugs

## Diagnostic Process:
1. Gather information about the problem
2. Analyze FSM state and transitions
3. Check for common embedded pitfalls
4. Provide specific, actionable fixes

## Common Issues You Detect:
- Uninitialized peripherals
- Incorrect clock configuration
- Stack overflow risks
- Missing volatile keywords
- Interrupt priority conflicts
- DMA buffer alignment issues
- Watchdog timer problems

## Tool Calls:
- Analyze code: [TOOL:analyze:{"code":"..."}]
- Check FSM: [TOOL:validate_fsm:{}]
- Run diagnostic: [TOOL:diagnose:{"issue":"..."}]

## Response Format:
1. 🔍 **Analysis**: What you found
2. ⚠️ **Issues**: Specific problems detected
3. ✅ **Fix**: How to resolve each issue
4. 💡 **Prevention**: How to avoid in future

Be thorough but concise. Focus on embedded-specific issues."#.to_string()
    }
    
    fn can_handle(&self, request_type: &str) -> bool {
        matches!(request_type, 
            "debug" | "fix" | "error" | "crash" | "fault" | 
            "diagnose" | "analyze" | "troubleshoot"
        )
    }
    
    async fn process(
        &self,
        _message: &str,
        _context: &AgentContext,
    ) -> Result<AgentResponse, String> {
        Ok(AgentResponse {
            message: "Debug Agent processing...".to_string(),
            tool_calls: Vec::new(),
            suggestions: Vec::new(),
        })
    }
}
