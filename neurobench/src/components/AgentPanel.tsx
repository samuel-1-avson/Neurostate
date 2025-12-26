// Agent Panel Component
// UI for interacting with AI agents

import { createSignal, createEffect, For, Show, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import useVoice from "../hooks/useVoice";

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

// Model configuration summary
interface AgentModelSummary {
  agent_id: string;
  agent_name: string;
  primary: string;
  fallback: string | null;
  purpose: string;
}

// Task planning
interface Task {
  id: string;
  intent: string;
  description: string;
  agent_id: string;
  status: "pending" | "in_progress" | "completed" | "failed";
}

interface TaskPlan {
  id: string;
  original_request: string;
  tasks: Task[];
  current_step: number;
  status: string;
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
  const [modelConfigs, setModelConfigs] = createSignal<AgentModelSummary[]>([]);
  const [currentPlan, setCurrentPlan] = createSignal<TaskPlan | null>(null);
  const [showModels, setShowModels] = createSignal(false);

  let chatContainerRef: HTMLDivElement | undefined;
  let inputRef: HTMLTextAreaElement | undefined;

  // Voice integration
  const voice = useVoice({ continuous: false, interimResults: true });
  const [speakResponses, setSpeakResponses] = createSignal(false);

  // Effect: Update input with interim transcript while listening
  createEffect(() => {
    if (voice.isListening()) {
      const interim = voice.interimTranscript();
      if (interim) {
        setInput(interim);
      }
    }
  });

  // Effect: Auto-send when speech ends with final transcript
  createEffect(() => {
    const transcript = voice.transcript();
    if (transcript && !voice.isListening()) {
      setInput(transcript);
      // Auto-send after a brief delay
      setTimeout(() => {
        if (input().trim()) {
          sendMessage();
          voice.clearTranscript();
        }
      }, 300);
    }
  });

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
      
      // Load model configurations
      try {
        const configs = await invoke("agent_get_model_config") as AgentModelSummary[];
        setModelConfigs(configs);
      } catch {
        console.log("Model config not available");
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
      
      // Check for task planning delegation
      const delegateTool = response.tool_calls.find(t => t.tool === "delegate");
      if (delegateTool && delegateTool.params.intent === "Complex") {
        // Create a task plan for complex requests
        try {
          const plan = await invoke("agent_create_plan", { request: msg }) as TaskPlan;
          setCurrentPlan(plan);
        } catch {
          console.log("Task planning not available");
        }
      }
    } catch (e) {
      setMessages([...messages(), {
        role: "assistant",
        content: `❌ Error: ${e}`,
        timestamp: getTime(),
      }]);
    }

    setIsLoading(false);

    // Speak response if TTS enabled
    const lastMsg = messages()[messages().length - 1];
    if (speakResponses() && lastMsg?.role === "assistant" && !lastMsg.content.startsWith("❌")) {
      voice.speak(lastMsg.content);
    }

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
    // Voice shortcut: Ctrl+Shift+V
    if (e.key === "v" && e.ctrlKey && e.shiftKey) {
      e.preventDefault();
      voice.toggleListening();
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
              {/* Show model info for current agent */}
              <Show when={modelConfigs().find(m => m.agent_id === activeAgent()!.id)}>
                <span class="cap-badge model-badge">
                  🤖 {modelConfigs().find(m => m.agent_id === activeAgent()!.id)?.primary}
                </span>
              </Show>
            </div>
          </div>
          <button 
            class={`agent-header-btn ${showModels() ? 'active' : ''}`} 
            onClick={() => setShowModels(!showModels())}
            title="View AI model configuration"
          >
            ⚙️
          </button>
        </div>
        
        {/* Model Configuration Panel */}
        <Show when={showModels()}>
          <div class="model-config-panel">
            <div class="model-config-title">🤖 AI Model Configuration</div>
            <div class="model-config-grid">
              <For each={modelConfigs()}>
                {(config) => (
                  <div class={`model-config-item ${config.agent_id === activeAgent()?.id ? 'active' : ''}`}>
                    <div class="model-agent-name">{config.agent_name}</div>
                    <div class="model-primary">
                      <span class="model-label">Primary:</span> {config.primary}
                    </div>
                    <Show when={config.fallback}>
                      <div class="model-fallback">
                        <span class="model-label">Fallback:</span> {config.fallback}
                      </div>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </div>
        </Show>
      </Show>

      {/* Task Plan Display */}
      <Show when={currentPlan()}>
        <div class="task-plan-panel">
          <div class="task-plan-title">📋 Task Plan</div>
          <div class="task-plan-request">{currentPlan()!.original_request}</div>
          <div class="task-plan-steps">
            <For each={currentPlan()!.tasks}>
              {(task, index) => (
                <div class={`task-step ${task.status} ${index() === currentPlan()!.current_step ? 'current' : ''}`}>
                  <span class="task-step-num">{index() + 1}</span>
                  <span class="task-step-agent">{task.agent_id}</span>
                  <span class="task-step-desc">{task.description}</span>
                </div>
              )}
            </For>
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
          placeholder={voice.isListening() ? "🎤 Listening..." : (activeAgent() ? `Ask ${activeAgent()!.name}...` : "Select an agent...")}
          disabled={!activeAgent() || isLoading()}
          rows={2}
        />
        <div class="input-actions">
          {/* Microphone Button */}
          <Show when={voice.isSupported}>
            <button
              class={`voice-btn ${voice.isListening() ? "listening" : ""}`}
              onClick={() => voice.toggleListening()}
              title={voice.isListening() ? "Stop listening" : "Start voice input (Ctrl+Shift+V)"}
              disabled={!activeAgent() || isLoading()}
            >
              {voice.isListening() ? (
                <span class="voice-wave">
                  <span></span><span></span><span></span>
                </span>
              ) : (
                "🎤"
              )}
            </button>
          </Show>
          {/* TTS Toggle */}
          <button
            class={`tts-btn ${speakResponses() ? "active" : ""}`}
            onClick={() => setSpeakResponses(!speakResponses())}
            title={speakResponses() ? "Disable voice responses" : "Enable voice responses"}
          >
            {speakResponses() ? "🔊" : "🔇"}
          </button>
          {/* Send Button */}
          <button 
            class="send-btn" 
            onClick={sendMessage}
            disabled={!input().trim() || isLoading()}
          >
            {isLoading() ? "..." : "Send"}
          </button>
        </div>
      </div>
    </div>
  );
}

export default AgentPanel;
