/**
 * Agent System Types
 * Matches Rust definitions in src-tauri/src/agents/mod.rs
 */

export interface AgentCapabilities {
  can_edit_fsm: boolean;
  can_generate_code: boolean;
  can_execute_terminal: boolean;
  can_access_hardware: boolean;
}

export interface AgentInfo {
  id: string;
  name: string;
  description: string;
  icon: string;
  capabilities: AgentCapabilities;
}

export interface ToolCall {
  tool: string;
  params: any;
}

export interface ToolResult {
  success: boolean;
  message: string;
  data?: any;
}

export interface AssistantMetadata {
  iterations: number;
  tokens_used: number;
  time_ms: number;
  agents_used: string[];
  complexity: number;
}

export interface AssistantResponse {
  message: string;
  tool_calls: ToolCall[];
  tool_results: ToolResult[];
  suggestions: string[];
  is_complete: boolean;
  progress: number;
  agent_id: string;
  metadata: AssistantMetadata;
}

// Project/Workspace types
export interface ProjectInfo {
  name: string;
  path: string;
  last_modified: number;
  is_active: boolean;
}

export interface Workspace {
  id: string;
  name: string;
  items: string[]; // Project names or sub-items
  expanded?: boolean;
}
