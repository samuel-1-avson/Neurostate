// Agent Panel Component
// UI for interacting with AI agents

import { createSignal, createEffect, For, Show, onMount, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import useVoice from "../hooks/useVoice";
import { Icons } from "./AppIcons";

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

// Unified Assistant Response (new)
interface AssistantMetadata {
  iterations: number;
  tokens_used: number;
  time_ms: number;
  agents_used: string[];
  complexity: number;
}

interface AssistantResponse {
  message: string;
  tool_calls: ToolCall[];
  tool_results: ToolResult[];
  suggestions: string[];
  is_complete: boolean;
  progress: number;
  agent_id: string;
  metadata: AssistantMetadata;
}

interface ChatMessage {
  role: "user" | "assistant" | "system";
  content: string;
  timestamp: string;
  toolCalls?: ToolCall[];
  toolResults?: ToolResult[];
  metadata?: AssistantMetadata;
  isError?: boolean;
  isSuccess?: boolean;
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

// Context information for awareness
interface ContextInfo {
  projectName: string;
  mcuTarget: string;
  language: string;
  nodeCount: number;
  edgeCount: number;
  selectedNodeId: string | null;
  selectedNodeLabel: string | null;
}

// Props for parent communication
interface AgentPanelProps {
  onToolAction?: (action: any) => void;
  // Context awareness props
  context?: ContextInfo;
  // Hide header when embedded in parent with its own header
  hideHeader?: boolean;
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
  const [useStreaming, setUseStreaming] = createSignal(true); // Enable streaming by default
  const [streamingContent, setStreamingContent] = createSignal(""); // Accumulator for streaming text
  const [isStreaming, setIsStreaming] = createSignal(false);
  const [lastMetadata, setLastMetadata] = createSignal<AssistantMetadata | null>(null);

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
          ? `SUCCESS: ${result.message}` 
          : `FAILED: ${result.message}`,
        timestamp: getTime(),
        isSuccess: result.success
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

  // Store unlisten function for cleanup
  let unlistenStream: UnlistenFn | null = null;

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
      
      // Set up streaming event listener
      unlistenStream = await listen<any>("agent-stream", (event) => {
        const data = event.payload;
        
        switch (data.type) {
          case "start":
            setStreamingContent("");
            setIsStreaming(true);
            break;
            
          case "chunk":
            setStreamingContent(prev => prev + data.content);
            // Scroll to bottom on each chunk
            if (chatContainerRef) {
              chatContainerRef.scrollTop = chatContainerRef.scrollHeight;
            }
            break;
            
          case "complete":
            setIsStreaming(false);
            // Access the full response from the event
            if (data.response) {
              const response = data.response as AssistantResponse;
              setMessages(prev => [...prev, {
                role: "assistant",
                content: response.message,
                timestamp: getTime(),
                toolCalls: response.tool_calls,
                toolResults: response.tool_results,
                metadata: response.metadata,
              }]);
              setLastMetadata(response.metadata);
              if (response.suggestions.length > 0) {
                setSuggestions(response.suggestions);
              }
              
              // Auto-execute safe tools
              const safeTools = ["add_node", "add_edge", "validate_fsm", "analyze_fsm", "get_pinout", "calc_clock", "gen_docs"];
              const toolsToExecute = response.tool_calls.filter(t => safeTools.includes(t.tool));
              
              // Execute safe tools automatically
              for (const tool of toolsToExecute) {
                executeTool(tool);
              }
            }
            setStreamingContent("");
            setIsLoading(false);
            break;
            
          case "error":
            setIsStreaming(false);
            setMessages(prev => [...prev, {
              role: "assistant",
              content: `Error: ${data.error}`,
              timestamp: getTime(),
              isError: true
            }]);
            setStreamingContent("");
            setIsLoading(false);
            break;
        }
      });
    } catch (e) {
      console.error("Failed to load agents:", e);
    }
  });

  // Cleanup event listener on unmount
  onCleanup(() => {
    if (unlistenStream) {
      unlistenStream();
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
          content: `${agent.name} is ready to help! ${agent.description}`,
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
      // Always use unified streaming mode
      if (useStreaming()) {
        // Use streaming - event listener will handle the response
        await invoke("unified_chat_stream", { 
          message: msg, 
          userId: null
        });
        // Response is handled by the event listener, don't set isLoading false here
        // The event listener will do it on complete/error
        return;
      } else {
        // Non-streaming unified mode
        const response = await invoke("unified_chat", { 
          message: msg, 
          userId: null
        }) as AssistantResponse;
        
        setMessages([...messages(), {
          role: "assistant",
          content: response.message,
          timestamp: getTime(),
          toolCalls: response.tool_calls,
          toolResults: response.tool_results,
          metadata: response.metadata,
        }]);

        setLastMetadata(response.metadata);
        
        if (response.suggestions.length > 0) {
          setSuggestions(response.suggestions);
        }
      }
    } catch (e) {
      setMessages([...messages(), {
        role: "assistant",
        content: `Error: ${e}`,
        timestamp: getTime(),
        isError: true
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
      {/* Unified AI Assistant Header - hidden when parent has header */}
      <Show when={!props.hideHeader}>
        <div class="unified-header">
          <div class="unified-header-main">
            <span class="unified-header-icon">{Icons.brain()}</span>
            <div class="unified-header-info">
              <div class="unified-header-name">AI Assistant</div>
              <div class="unified-header-context">
                <Show when={props.context?.mcuTarget}>
                  <span class="context-badge mcu">{Icons.chip()} {props.context?.mcuTarget}</span>
                </Show>
                <Show when={props.context?.projectName}>
                  <span class="context-badge project">{Icons.folder()} {props.context?.projectName}</span>
                </Show>
                <Show when={props.context?.selectedNodeLabel}>
                  <span class="context-badge node">{Icons.activity()} {props.context?.selectedNodeLabel}</span>
                </Show>
                <Show when={!props.context}>
                  <span class="context-badge dim">No context</span>
                </Show>
              </div>
            </div>
          </div>
          <div class="unified-header-actions">
            <Show when={props.context}>
              <span class="fsm-stats" title="FSM Statistics">
                {props.context?.nodeCount || 0} states • {props.context?.edgeCount || 0} transitions
              </span>
            </Show>
            <button 
              class={`agent-header-btn ${showModels() ? 'active' : ''}`} 
              onClick={() => setShowModels(!showModels())}
              title="View AI model configuration"
            >
              <span class="settings-icon">{Icons.gear()}</span>
            </button>
          </div>
        </div>
      </Show>

      {/* Model Configuration Panel */}
      <Show when={showModels()}>
        <div class="model-config-panel">
          <div class="model-config-title"><span class="title-icon">{Icons.brain()}</span> AI Model Configuration</div>
          <div class="model-config-grid">
            <For each={modelConfigs()}>
              {(config) => (
                <div class="model-config-item">
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

      {/* Task Plan Display */}
      <Show when={currentPlan()}>
        <div class="task-plan-panel">
          <div class="task-plan-title"><span class="title-icon">{Icons.tasks()}</span> Task Plan</div>
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
                          <span class="tool-icon">{Icons.build()}</span>
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
                {/* Tool Results */}
                <Show when={msg.toolResults && msg.toolResults.length > 0}>
                  <div class="tool-results">
                    <For each={msg.toolResults}>
                      {(result) => (
                        <div class={`tool-result ${result.success ? 'success' : 'error'}`}>
                          <span class="result-icon">{result.success ? Icons.checkCircle() : Icons.errorCircle()}</span>
                          <span class="result-msg">{result.message}</span>
                        </div>
                      )}
                    </For>
                  </div>
                </Show>
                {/* Metadata Badge */}
                <Show when={msg.metadata}>
                  <div class="metadata-badge">
                    <span class="meta-item" title="Complexity Score">
                      {Icons.activity()} {msg.metadata!.complexity}/10
                    </span>
                    <span class="meta-item" title="Response Time">
                      {Icons.clock()} {msg.metadata!.time_ms}ms
                    </span>
                    <Show when={msg.metadata!.agents_used.length > 0}>
                      <span class="meta-item" title="Agents Used">
                        {Icons.layers()} {msg.metadata!.agents_used.join(', ')}
                      </span>
                    </Show>
                  </div>
                </Show>
              </div>
              <span class="chat-time">{msg.timestamp}</span>
            </div>
          )}
        </For>
        
        {/* Loading / Streaming indicator */}
        <Show when={isLoading()}>
          <Show when={isStreaming() && streamingContent()}>
            {/* Show streaming content in real-time */}
            <div class="chat-message assistant">
              <div class="chat-bubble streaming">
                <div class="chat-content">{streamingContent()}</div>
                <div class="streaming-indicator">
                  <span class="streaming-dot"></span>
                  <span class="streaming-dot"></span>
                  <span class="streaming-dot"></span>
                </div>
              </div>
            </div>
          </Show>
          <Show when={!isStreaming() || !streamingContent()}>
            {/* Regular loading dots */}
            <div class="chat-message assistant">
              <div class="chat-bubble loading">
                <span class="loading-dot"></span>
                <span class="loading-dot"></span>
                <span class="loading-dot"></span>
              </div>
            </div>
          </Show>
        </Show>
      </div>

      {/* Suggestions */}
      <Show when={suggestions().length > 0}>
        <div class="agent-suggestions">
          <For each={suggestions()}>
            {(suggestion) => (
              <button class="suggestion-btn" onClick={() => useSuggestion(suggestion)}>
                                <span class="sug-icon">{Icons.lightbulb()}</span> {suggestion}
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
                <span class="voice-wave-active">
                   {Icons.waveform()}
                </span>
              ) : (
                Icons.mic()
              )}
            </button>
          </Show>
          {/* TTS Toggle */}
          <button
            class={`tts-btn ${speakResponses() ? "active" : ""}`}
            onClick={() => setSpeakResponses(!speakResponses())}
            title={speakResponses() ? "Disable voice responses" : "Enable voice responses"}
          >
            {speakResponses() ? Icons.volume() : Icons.volumeMute()}
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
