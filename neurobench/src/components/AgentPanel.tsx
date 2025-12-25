// Agent Panel Component
// UI for interacting with AI agents

import { createSignal, For, Show, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface AgentInfo {
  id: string;
  name: string;
  description: string;
  icon: string;
  capabilities: {
    can_edit_fsm: boolean;
    can_generate_code: boolean;
    can_execute_terminal: boolean;
    can_access_hardware: boolean;
  };
}

interface ToolCall {
  tool: string;
  params: any;
}

interface AgentResponse {
  message: string;
  tool_calls: ToolCall[];
  suggestions: string[];
}

interface ChatMessage {
  role: "user" | "assistant" | "system";
  content: string;
  timestamp: string;
  toolCalls?: ToolCall[];
}

interface ToolResult {
  success: boolean;
  message: string;
  data?: any;
}

// Props for parent communication
interface AgentPanelProps {
  onToolAction?: (action: any) => void;
}

export function AgentPanel(props: AgentPanelProps) {
  const [agents, setAgents] = createSignal<AgentInfo[]>([]);
  const [activeAgent, setActiveAgent] = createSignal<AgentInfo | null>(null);
  const [messages, setMessages] = createSignal<ChatMessage[]>([]);
  const [input, setInput] = createSignal("");
  const [isLoading, setIsLoading] = createSignal(false);
  const [suggestions, setSuggestions] = createSignal<string[]>([]);
  const [executedTools, setExecutedTools] = createSignal<Set<string>>(new Set());

  let chatContainerRef: HTMLDivElement | undefined;
  let inputRef: HTMLTextAreaElement | undefined;

  function getTime(): string {
    return new Date().toLocaleTimeString("en-US", { hour12: false, hour: "2-digit", minute: "2-digit" });
  }

  // Execute a tool call
  async function executeTool(tool: ToolCall) {
    try {
      const result = await invoke("execute_tool", { 
        tool: tool.tool, 
        params: tool.params 
      }) as ToolResult;
      
      // Mark tool as executed
      setExecutedTools(prev => new Set([...prev, `${tool.tool}-${JSON.stringify(tool.params)}`]));
      
      // If tool has action data, pass to parent
      if (result.data?.action && props.onToolAction) {
        props.onToolAction(result.data);
      }
      
      // Add result message
      setMessages([...messages(), {
        role: "system",
        content: result.success 
          ? `✅ ${result.message}` 
          : `❌ ${result.message}`,
        timestamp: getTime(),
      }]);
      
      // Scroll to bottom
      setTimeout(() => {
        if (chatContainerRef) {
          chatContainerRef.scrollTop = chatContainerRef.scrollHeight;
        }
      }, 10);
    } catch (e) {
      setMessages([...messages(), {
        role: "system",
        content: `❌ Tool execution failed: ${e}`,
        timestamp: getTime(),
      }]);
    }
  }

  function isToolExecuted(tool: ToolCall): boolean {
    return executedTools().has(`${tool.tool}-${JSON.stringify(tool.params)}`);
  }

  onMount(async () => {
    try {
      const agentList = await invoke("list_agents") as AgentInfo[];
      setAgents(agentList);
      
      const active = await invoke("get_active_agent") as AgentInfo | null;
      if (active) {
        setActiveAgent(active);
      } else if (agentList.length > 0) {
        await selectAgent(agentList[0].id);
      }
    } catch (e) {
      console.error("Failed to load agents:", e);
    }
  });

  async function selectAgent(agentId: string) {
    try {
      await invoke("set_active_agent", { agentId });
      const agent = agents().find(a => a.id === agentId);
      if (agent) {
        setActiveAgent(agent);
        setMessages([{
          role: "system",
          content: `${agent.icon} ${agent.name} is ready to help! ${agent.description}`,
          timestamp: getTime(),
        }]);
        setSuggestions([]);
      }
    } catch (e) {
      console.error("Failed to select agent:", e);
    }
  }

  async function sendMessage() {
    const msg = input().trim();
    if (!msg || isLoading()) return;

    // Add user message
    setMessages([...messages(), {
      role: "user",
      content: msg,
      timestamp: getTime(),
    }]);
    setInput("");
    setIsLoading(true);
    setSuggestions([]);

    // Scroll to bottom
    setTimeout(() => {
      if (chatContainerRef) {
        chatContainerRef.scrollTop = chatContainerRef.scrollHeight;
      }
    }, 10);

    try {
      const response = await invoke("agent_chat", { message: msg }) as AgentResponse;
      
      setMessages([...messages(), {
        role: "assistant",
        content: response.message,
        timestamp: getTime(),
        toolCalls: response.tool_calls,
      }]);

      if (response.suggestions.length > 0) {
        setSuggestions(response.suggestions);
      }
    } catch (e) {
      setMessages([...messages(), {
        role: "assistant",
        content: `❌ Error: ${e}`,
        timestamp: getTime(),
      }]);
    }

    setIsLoading(false);

    // Scroll to bottom
    setTimeout(() => {
      if (chatContainerRef) {
        chatContainerRef.scrollTop = chatContainerRef.scrollHeight;
      }
    }, 10);
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }

  function useSuggestion(suggestion: string) {
    setInput(suggestion);
    inputRef?.focus();
  }

  return (
    <div class="agent-panel">
      {/* Agent Selector */}
      <div class="agent-selector">
        <For each={agents()}>
          {(agent) => (
            <button
              class={`agent-btn ${activeAgent()?.id === agent.id ? "active" : ""}`}
              onClick={() => selectAgent(agent.id)}
              title={agent.description}
            >
              <span class="agent-icon">{agent.icon}</span>
              <span class="agent-name">{agent.name}</span>
            </button>
          )}
        </For>
      </div>

      {/* Active Agent Header */}
      <Show when={activeAgent()}>
        <div class="agent-header">
          <span class="agent-header-icon">{activeAgent()!.icon}</span>
          <div class="agent-header-info">
            <div class="agent-header-name">{activeAgent()!.name}</div>
            <div class="agent-header-caps">
              <Show when={activeAgent()!.capabilities.can_edit_fsm}>
                <span class="cap-badge">FSM</span>
              </Show>
              <Show when={activeAgent()!.capabilities.can_generate_code}>
                <span class="cap-badge">Code</span>
              </Show>
              <Show when={activeAgent()!.capabilities.can_execute_terminal}>
                <span class="cap-badge">Terminal</span>
              </Show>
            </div>
          </div>
        </div>
      </Show>

      {/* Chat Messages */}
      <div class="agent-chat" ref={chatContainerRef}>
        <For each={messages()}>
          {(msg) => (
            <div class={`chat-message ${msg.role}`}>
              <div class="chat-bubble">
                <div class="chat-content">{msg.content}</div>
                <Show when={msg.toolCalls && msg.toolCalls.length > 0}>
                  <div class="tool-calls">
                    <For each={msg.toolCalls}>
                      {(tool) => (
                        <div class="tool-call">
                          <span class="tool-icon">🔧</span>
                          <span class="tool-name">{tool.tool}</span>
                          <button 
                            class={`tool-exec-btn ${isToolExecuted(tool) ? "executed" : ""}`}
                            onClick={() => executeTool(tool)}
                            disabled={isToolExecuted(tool)}
                          >
                            {isToolExecuted(tool) ? "✓" : "Run"}
                          </button>
                        </div>
                      )}
                    </For>
                  </div>
                </Show>
              </div>
              <span class="chat-time">{msg.timestamp}</span>
            </div>
          )}
        </For>
        
        <Show when={isLoading()}>
          <div class="chat-message assistant">
            <div class="chat-bubble loading">
              <span class="loading-dot"></span>
              <span class="loading-dot"></span>
              <span class="loading-dot"></span>
            </div>
          </div>
        </Show>
      </div>

      {/* Suggestions */}
      <Show when={suggestions().length > 0}>
        <div class="agent-suggestions">
          <For each={suggestions()}>
            {(suggestion) => (
              <button class="suggestion-btn" onClick={() => useSuggestion(suggestion)}>
                💡 {suggestion}
              </button>
            )}
          </For>
        </div>
      </Show>

      {/* Input */}
      <div class="agent-input">
        <textarea
          ref={inputRef}
          value={input()}
          onInput={(e) => setInput(e.currentTarget.value)}
          onKeyDown={handleKeyDown}
          placeholder={activeAgent() ? `Ask ${activeAgent()!.name}...` : "Select an agent..."}
          disabled={!activeAgent() || isLoading()}
          rows={2}
        />
        <button 
          class="send-btn" 
          onClick={sendMessage}
          disabled={!input().trim() || isLoading()}
        >
          {isLoading() ? "..." : "Send"}
        </button>
      </div>
    </div>
  );
}

export default AgentPanel;
