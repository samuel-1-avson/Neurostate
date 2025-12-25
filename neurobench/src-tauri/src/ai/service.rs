// AI Service
// High-level AI operations for embedded systems assistance

use super::gemini::GeminiClient;
use crate::core::*;

pub struct AIService {
    gemini: GeminiClient,
}

impl AIService {
    pub fn new() -> Self {
        Self {
            gemini: GeminiClient::new(),
        }
    }
    
    pub fn is_available(&self) -> bool {
        self.gemini.is_configured()
    }
    
    /// Generate FSM code from nodes and edges
    pub async fn generate_fsm_code(
        &self,
        nodes: &[FSMNode],
        edges: &[FSMEdge],
        language: &str,
    ) -> Result<String, String> {
        let prompt = format!(
            r#"You are an expert embedded systems engineer. Generate {} code for the following Finite State Machine.

## FSM Nodes:
{}

## Transitions:
{}

Generate complete, production-ready {} code for this FSM. Include:
1. State enum/type definition
2. Transition logic
3. Entry/exit action handlers
4. Main FSM processing function

Only output the code, no explanations."#,
            language,
            format_nodes(nodes),
            format_edges(edges, nodes),
            language
        );
        
        self.gemini.generate(&prompt).await
    }
    
    /// Generate unit tests for FSM
    pub async fn generate_tests(
        &self,
        nodes: &[FSMNode],
        edges: &[FSMEdge],
    ) -> Result<String, String> {
        let prompt = format!(
            r#"Generate comprehensive unit tests for this Finite State Machine:

## States:
{}

## Transitions:
{}

Generate test cases that verify:
1. All state transitions work correctly
2. Guard conditions are evaluated properly
3. Entry/exit actions are called
4. Edge cases and error states

Use a common testing framework. Only output the test code."#,
            format_nodes(nodes),
            format_edges(edges, nodes),
        );
        
        self.gemini.generate(&prompt).await
    }
    
    /// Analyze FSM for issues
    pub async fn analyze_fsm(
        &self,
        nodes: &[FSMNode],
        edges: &[FSMEdge],
    ) -> Result<String, String> {
        let prompt = format!(
            r#"Analyze this Finite State Machine for embedded systems and identify any issues:

## States:
{}

## Transitions:
{}

Check for:
1. Unreachable states
2. Deadlock conditions
3. Missing error handling
4. Race conditions
5. Resource leaks
6. Timing issues with interrupts

Provide a detailed analysis with specific recommendations."#,
            format_nodes(nodes),
            format_edges(edges, nodes),
        );
        
        self.gemini.generate(&prompt).await
    }
    
    /// Generate node logic from description
    pub async fn generate_node_logic(
        &self,
        node_label: &str,
        node_type: &str,
        description: &str,
    ) -> Result<String, String> {
        let prompt = format!(
            r#"Generate embedded C code for an FSM node action.

Node: {} (type: {})
Description: {}

Generate concise, efficient C code for the entry action of this state.
Use HAL functions for hardware access.
Only output the code, no explanations."#,
            node_label, node_type, description
        );
        
        self.gemini.generate(&prompt).await
    }
    
    /// Parse natural language description into FSM nodes and edges
    pub async fn parse_fsm_from_description(&self, description: &str) -> Result<String, String> {
        let prompt = format!(
            r#"You are an FSM designer. Convert this natural language description into a JSON FSM graph.

Description: {}

Output ONLY valid JSON with this exact structure (no markdown, no explanation):
{{
  "nodes": [
    {{"id": "1", "label": "STATE_NAME", "type": "input", "x": 300, "y": 100}},
    {{"id": "2", "label": "STATE_NAME", "type": "process", "x": 300, "y": 220, "entryAction": "code here"}},
    {{"id": "3", "label": "STATE_NAME", "type": "output", "x": 300, "y": 340}}
  ],
  "edges": [
    {{"id": "e1", "source": "1", "target": "2", "label": "event_name"}},
    {{"id": "e2", "source": "2", "target": "3", "label": "event_name"}}
  ]
}}

Rules:
- First node type should be "input" (start state)
- Last node type should be "output" (end state)  
- Other nodes should be "process" or "decision"
- Position nodes vertically with y increasing by 120 for each node
- Use meaningful state names in UPPERCASE
- Edge labels should be transition triggers/events
- Include entryAction for states that need code

Output ONLY the JSON, nothing else."#,
            description
        );
        
        self.gemini.generate(&prompt).await
    }
    
    /// Chat with AI assistant
    pub async fn chat(&self, message: &str, context: Option<&str>) -> Result<String, String> {
        let system_context = r#"You are NeuroBench AI, an expert assistant for embedded systems design.
You help users design finite state machines, write firmware code, debug hardware issues, and optimize embedded software.
Be concise and technical. Prefer code examples over lengthy explanations."#;

        let prompt = match context {
            Some(ctx) => format!("{}\n\nContext:\n{}\n\nUser: {}", system_context, ctx, message),
            None => format!("{}\n\nUser: {}", system_context, message),
        };
        
        self.gemini.generate(&prompt).await
    }
}

impl Default for AIService {
    fn default() -> Self {
        Self::new()
    }
}

// --- Helper Functions ---

fn format_nodes(nodes: &[FSMNode]) -> String {
    nodes.iter()
        .map(|n| format!("- {} ({}){}", 
            n.label, 
            format!("{:?}", n.node_type).to_lowercase(),
            n.entry_action.as_ref().map(|a| format!(" [action: {}]", a)).unwrap_or_default()
        ))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_edges(edges: &[FSMEdge], nodes: &[FSMNode]) -> String {
    edges.iter()
        .map(|e| {
            let source = nodes.iter().find(|n| n.id == e.source).map(|n| n.label.as_str()).unwrap_or("?");
            let target = nodes.iter().find(|n| n.id == e.target).map(|n| n.label.as_str()).unwrap_or("?");
            let label = e.label.as_deref().unwrap_or("-");
            let guard = e.guard.as_ref().map(|g| format!(" [guard: {}]", g)).unwrap_or_default();
            format!("- {} -> {} ({}){}", source, target, label, guard)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
