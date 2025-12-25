// FSM Architect Agent
// Designs and optimizes state machines

use super::{Agent, AgentCapabilities, AgentContext, AgentInfo, AgentResponse};
use async_trait::async_trait;

pub struct FsmAgent;

impl FsmAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FsmAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for FsmAgent {
    fn info(&self) -> AgentInfo {
        AgentInfo {
            id: "fsm".to_string(),
            name: "FSM Architect".to_string(),
            description: "Design and optimize state machines from natural language descriptions".to_string(),
            icon: "🔄".to_string(),
            capabilities: AgentCapabilities {
                can_edit_fsm: true,
                can_generate_code: false,
                can_execute_terminal: false,
                can_access_hardware: false,
            },
        }
    }
    
    fn system_prompt(&self) -> String {
        r#"You are the FSM Architect Agent in NeuroBench, an embedded systems development platform.

Your job is to help users design, analyze, and optimize finite state machines (FSMs).

## Capabilities:
- Design new FSMs from natural language descriptions
- Analyze existing FSMs for issues (unreachable states, deadlocks)
- Optimize FSMs (reduce states, merge transitions)
- Explain state machine concepts

## Tool Calls:
When you want to perform actions, include tool calls in your response:
- Add node: [TOOL:add_node:{"label":"StateName","type":"process"}]
- Add transition: [TOOL:add_edge:{"source":"State1","target":"State2","label":"event"}]
- Remove node: [TOOL:remove_node:{"id":"node_id"}]

## Response Format:
1. First explain what you'll do
2. Include any tool calls needed
3. Provide suggestions with [SUGGEST:suggestion text]

## FSM Best Practices:
- Every FSM should have one start state (input type)
- Every FSM should have at least one end state (output type)
- Avoid orphan states with no incoming transitions
- Use decision nodes for conditional branching
- Keep state names concise but descriptive

Be helpful, specific, and practical. Focus on embedded systems use cases."#.to_string()
    }
    
    fn can_handle(&self, request_type: &str) -> bool {
        matches!(request_type, 
            "design_fsm" | "analyze_fsm" | "optimize_fsm" | 
            "add_state" | "add_transition" | "fsm_help"
        )
    }
    
    async fn process(
        &self,
        _message: &str,
        _context: &AgentContext,
    ) -> Result<AgentResponse, String> {
        // This is handled by the orchestrator which calls the AI service
        // The agent just provides the system prompt
        Ok(AgentResponse {
            message: "FSM Agent processing...".to_string(),
            tool_calls: Vec::new(),
            suggestions: Vec::new(),
        })
    }
}
