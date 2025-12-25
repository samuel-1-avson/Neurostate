// Documentation Agent
// Generates and maintains project documentation

use super::{Agent, AgentCapabilities, AgentContext, AgentInfo, AgentResponse};
use async_trait::async_trait;

pub struct DocsAgent;

impl DocsAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DocsAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for DocsAgent {
    fn info(&self) -> AgentInfo {
        AgentInfo {
            id: "docs".to_string(),
            name: "Documentation".to_string(),
            description: "Generate comments, README files, API docs, and diagrams".to_string(),
            icon: "📝".to_string(),
            capabilities: AgentCapabilities {
                can_edit_fsm: false,
                can_generate_code: true,
                can_execute_terminal: false,
                can_access_hardware: false,
            },
        }
    }
    
    fn system_prompt(&self) -> String {
        r#"You are the Documentation Agent in NeuroBench, an embedded systems development platform.

Your job is to generate high-quality documentation for embedded systems projects.

## Documentation Types:
- **Code Comments**: Doxygen-style function/file headers
- **README**: Project overview, build instructions, usage
- **API Docs**: Function signatures, parameters, return values
- **State Diagrams**: Mermaid/PlantUML for FSM visualization
- **Hardware Docs**: Pin assignments, connections, BOM

## Documentation Standards:
- Follow Doxygen format for C/C++:
  ```c
  /**
   * @brief Brief description
   * @param param1 Description of param1
   * @return Description of return value
   */
  ```
- Include author, date, version info
- Document all public interfaces
- Add examples for complex functions

## FSM Documentation:
When documenting FSMs, include:
1. State descriptions
2. Transition triggers/guards
3. Entry/exit actions
4. Timing constraints
5. Mermaid diagram

## Tool Calls:
- Generate docs: [TOOL:gen_docs:{"type":"readme"}]
- Add comments: [TOOL:comment:{"function":"main"}]
- Create diagram: [TOOL:diagram:{"type":"mermaid"}]

## Response Format:
Provide complete, copy-paste ready documentation.
Use proper markdown formatting.
Include all relevant sections."#.to_string()
    }
    
    fn can_handle(&self, request_type: &str) -> bool {
        matches!(request_type, 
            "docs" | "document" | "readme" | "comment" | 
            "doxygen" | "diagram" | "explain"
        )
    }
    
    async fn process(
        &self,
        _message: &str,
        _context: &AgentContext,
    ) -> Result<AgentResponse, String> {
        Ok(AgentResponse {
            message: "Docs Agent processing...".to_string(),
            tool_calls: Vec::new(),
            suggestions: Vec::new(),
        })
    }
}
