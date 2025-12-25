// Agent System Tests
// Comprehensive tests for agents, context, and tool executor

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::{
        Agent, AgentContext, AgentRegistry, AgentInfo, AgentCapabilities,
        ToolExecutor, ToolResult,
        fsm_agent::FsmAgent,
        code_agent::CodeAgent,
        debug_agent::DebugAgent,
        hardware_agent::HardwareAgent,
        docs_agent::DocsAgent,
        context::{ContextNode, ContextEdge, McuConfig, ProjectContext},
    };
    use serde_json::json;

    // ==================== Agent Registry Tests ====================

    #[test]
    fn test_agent_registry_creation() {
        let registry = AgentRegistry::new();
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_agent_registration() {
        let mut registry = AgentRegistry::new();
        registry.register(Box::new(FsmAgent::new()));
        registry.register(Box::new(CodeAgent::new()));
        
        let agents = registry.list();
        assert_eq!(agents.len(), 2);
    }

    #[test]
    fn test_get_agent_by_id() {
        let mut registry = AgentRegistry::new();
        registry.register(Box::new(FsmAgent::new()));
        
        let agent = registry.get("fsm");
        assert!(agent.is_some());
        assert_eq!(agent.unwrap().info().id, "fsm");
        
        let missing = registry.get("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_enable_disable_agent() {
        let mut registry = AgentRegistry::new();
        registry.register(Box::new(FsmAgent::new()));
        
        assert_eq!(registry.list().len(), 1);
        
        registry.enable("fsm", false);
        assert_eq!(registry.list().len(), 0);
        
        registry.enable("fsm", true);
        assert_eq!(registry.list().len(), 1);
    }

    // ==================== Agent Info Tests ====================

    #[test]
    fn test_fsm_agent_info() {
        let agent = FsmAgent::new();
        let info = agent.info();
        
        assert_eq!(info.id, "fsm");
        assert_eq!(info.name, "FSM Architect");
        assert!(info.capabilities.can_edit_fsm);
        assert!(!info.capabilities.can_generate_code);
    }

    #[test]
    fn test_code_agent_info() {
        let agent = CodeAgent::new();
        let info = agent.info();
        
        assert_eq!(info.id, "code");
        assert_eq!(info.name, "Code Generator");
        assert!(!info.capabilities.can_edit_fsm);
        assert!(info.capabilities.can_generate_code);
    }

    #[test]
    fn test_debug_agent_info() {
        let agent = DebugAgent::new();
        let info = agent.info();
        
        assert_eq!(info.id, "debug");
        assert_eq!(info.name, "Debug Assistant");
        assert!(info.capabilities.can_edit_fsm);
        assert!(info.capabilities.can_generate_code);
        assert!(info.capabilities.can_execute_terminal);
        assert!(info.capabilities.can_access_hardware);
    }

    #[test]
    fn test_hardware_agent_info() {
        let agent = HardwareAgent::new();
        let info = agent.info();
        
        assert_eq!(info.id, "hardware");
        assert_eq!(info.name, "Hardware Advisor");
        assert!(info.capabilities.can_access_hardware);
    }

    #[test]
    fn test_docs_agent_info() {
        let agent = DocsAgent::new();
        let info = agent.info();
        
        assert_eq!(info.id, "docs");
        assert_eq!(info.name, "Documentation");
        assert!(info.capabilities.can_generate_code);
    }

    // ==================== Context Tests ====================

    #[test]
    fn test_context_creation() {
        let ctx = AgentContext::new();
        
        assert_eq!(ctx.project.name, "Untitled Project");
        assert_eq!(ctx.mcu.target, "STM32F401");
        assert!(ctx.nodes.is_empty());
        assert!(ctx.edges.is_empty());
        assert!(ctx.selected_node.is_none());
    }

    #[test]
    fn test_mcu_config_default() {
        let mcu = McuConfig::default();
        
        assert_eq!(mcu.target, "STM32F401");
        assert_eq!(mcu.clock_speed, 84_000_000);
        assert_eq!(mcu.flash_size, 256 * 1024);
        assert_eq!(mcu.ram_size, 64 * 1024);
    }

    #[test]
    fn test_project_context_default() {
        let project = ProjectContext::default();
        
        assert_eq!(project.name, "Untitled Project");
        assert_eq!(project.language, "c");
        assert_eq!(project.ide, "stm32cubeide");
    }

    #[test]
    fn test_context_add_messages() {
        let mut ctx = AgentContext::new();
        
        ctx.add_user_message("Hello");
        ctx.add_assistant_message("Hi there!");
        
        assert_eq!(ctx.conversation.len(), 2);
        assert_eq!(ctx.conversation[0].role, "user");
        assert_eq!(ctx.conversation[0].content, "Hello");
        assert_eq!(ctx.conversation[1].role, "assistant");
    }

    #[test]
    fn test_context_to_prompt() {
        let mut ctx = AgentContext::new();
        ctx.nodes.push(ContextNode {
            id: "1".to_string(),
            label: "IDLE".to_string(),
            node_type: "input".to_string(),
            x: 100.0,
            y: 100.0,
            entry_action: None,
        });
        ctx.nodes.push(ContextNode {
            id: "2".to_string(),
            label: "RUNNING".to_string(),
            node_type: "process".to_string(),
            x: 100.0,
            y: 200.0,
            entry_action: Some("start_motor()".to_string()),
        });
        
        let prompt = ctx.to_prompt_context();
        
        assert!(prompt.contains("STM32F401"));
        assert!(prompt.contains("IDLE"));
        assert!(prompt.contains("RUNNING"));
        assert!(prompt.contains("FSM States (2)"));
    }

    // ==================== Tool Executor Tests ====================

    #[test]
    fn test_add_node_tool() {
        let params = json!({
            "label": "TEST_STATE",
            "type": "process",
            "x": 200,
            "y": 300
        });
        
        let result = ToolExecutor::execute("add_node", &params);
        
        assert!(result.success);
        assert!(result.message.contains("TEST_STATE"));
        assert!(result.data.is_some());
        
        let data = result.data.unwrap();
        assert_eq!(data["action"], "add_node");
        assert_eq!(data["node"]["label"], "TEST_STATE");
        assert_eq!(data["node"]["type"], "process");
    }

    #[test]
    fn test_remove_node_tool() {
        let params = json!({ "id": "node_123" });
        let result = ToolExecutor::execute("remove_node", &params);
        
        assert!(result.success);
        assert!(result.message.contains("node_123"));
    }

    #[test]
    fn test_remove_node_without_id() {
        let params = json!({});
        let result = ToolExecutor::execute("remove_node", &params);
        
        assert!(!result.success);
        assert!(result.message.contains("Node ID required"));
    }

    #[test]
    fn test_add_edge_tool() {
        let params = json!({
            "source": "node_1",
            "target": "node_2",
            "label": "START"
        });
        
        let result = ToolExecutor::execute("add_edge", &params);
        
        assert!(result.success);
        assert!(result.message.contains("node_1"));
        assert!(result.message.contains("node_2"));
        
        let data = result.data.unwrap();
        assert_eq!(data["action"], "add_edge");
        assert_eq!(data["edge"]["source"], "node_1");
        assert_eq!(data["edge"]["target"], "node_2");
    }

    #[test]
    fn test_add_edge_without_source_target() {
        let params = json!({ "source": "node_1" });
        let result = ToolExecutor::execute("add_edge", &params);
        
        assert!(!result.success);
        assert!(result.message.contains("Source and target"));
    }

    #[test]
    fn test_validate_fsm_tool() {
        let params = json!({});
        let result = ToolExecutor::execute("validate_fsm", &params);
        
        assert!(result.success);
        assert!(result.data.is_some());
        
        let data = result.data.unwrap();
        assert_eq!(data["action"], "validate_fsm");
    }

    #[test]
    fn test_get_pinout_tool() {
        let params = json!({ "mcu": "STM32F407" });
        let result = ToolExecutor::execute("get_pinout", &params);
        
        assert!(result.success);
        assert!(result.message.contains("STM32F407"));
    }

    #[test]
    fn test_calc_clock_tool() {
        let params = json!({ "target_freq": 168000000 });
        let result = ToolExecutor::execute("calc_clock", &params);
        
        assert!(result.success);
        assert!(result.message.contains("168MHz"));
        
        let data = result.data.unwrap();
        assert_eq!(data["action"], "calc_clock");
    }

    #[test]
    fn test_gen_docs_tool() {
        let params = json!({ "type": "readme" });
        let result = ToolExecutor::execute("gen_docs", &params);
        
        assert!(result.success);
        assert!(result.message.contains("readme"));
    }

    #[test]
    fn test_unknown_tool() {
        let params = json!({});
        let result = ToolExecutor::execute("unknown_tool", &params);
        
        assert!(!result.success);
        assert!(result.message.contains("Unknown tool"));
    }

    // ==================== Agent System Prompt Tests ====================

    #[test]
    fn test_fsm_agent_system_prompt() {
        let agent = FsmAgent::new();
        let prompt = agent.system_prompt();
        
        assert!(prompt.contains("FSM Architect"));
        assert!(prompt.contains("TOOL:add_node"));
        assert!(prompt.contains("state machine"));
    }

    #[test]
    fn test_code_agent_system_prompt() {
        let agent = CodeAgent::new();
        let prompt = agent.system_prompt();
        
        assert!(prompt.contains("Code Generator"));
        assert!(prompt.contains("MISRA-C"));
        assert!(prompt.contains("STM32"));
    }

    #[test]
    fn test_debug_agent_system_prompt() {
        let agent = DebugAgent::new();
        let prompt = agent.system_prompt();
        
        assert!(prompt.contains("Debug Assistant"));
        assert!(prompt.contains("hard fault"));
        assert!(prompt.contains("stack overflow"));
    }

    #[test]
    fn test_hardware_agent_system_prompt() {
        let agent = HardwareAgent::new();
        let prompt = agent.system_prompt();
        
        assert!(prompt.contains("Hardware Advisor"));
        assert!(prompt.contains("Pin mapping"));  // Capital P in actual prompt
        assert!(prompt.contains("clock") || prompt.contains("Clock"));
    }

    #[test]
    fn test_docs_agent_system_prompt() {
        let agent = DocsAgent::new();
        let prompt = agent.system_prompt();
        
        assert!(prompt.contains("Documentation"));
        assert!(prompt.contains("Doxygen"));
        assert!(prompt.contains("README"));
    }

    // ==================== Agent can_handle Tests ====================

    #[test]
    fn test_fsm_agent_can_handle() {
        let agent = FsmAgent::new();
        
        assert!(agent.can_handle("design_fsm"));
        assert!(agent.can_handle("analyze_fsm"));
        assert!(agent.can_handle("add_state"));
        assert!(!agent.can_handle("generate_code"));
    }

    #[test]
    fn test_code_agent_can_handle() {
        let agent = CodeAgent::new();
        
        assert!(agent.can_handle("generate_code"));
        assert!(agent.can_handle("generate_driver"));
        assert!(agent.can_handle("fsm_to_code"));
        assert!(!agent.can_handle("design_fsm"));
    }

    #[test]
    fn test_debug_agent_can_handle() {
        let agent = DebugAgent::new();
        
        assert!(agent.can_handle("debug"));
        assert!(agent.can_handle("fix"));
        assert!(agent.can_handle("diagnose"));
        assert!(!agent.can_handle("generate_code"));
    }

    // ==================== Tool Result Tests ====================

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success("Operation completed");
        
        assert!(result.success);
        assert_eq!(result.message, "Operation completed");
        assert!(result.data.is_none());
    }

    #[test]
    fn test_tool_result_success_with_data() {
        let data = json!({ "key": "value" });
        let result = ToolResult::success_with_data("With data", data);
        
        assert!(result.success);
        assert!(result.data.is_some());
        assert_eq!(result.data.unwrap()["key"], "value");
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("Something failed");
        
        assert!(!result.success);
        assert_eq!(result.message, "Something failed");
        assert!(result.data.is_none());
    }
}
