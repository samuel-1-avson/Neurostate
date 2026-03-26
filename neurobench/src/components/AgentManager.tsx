/**
 * AgentManager - Dashboard for managing AI agents and conversations
 * Modeled after the reference design (sidebar + main content)
 */
import { Component, createEffect, createSignal, For, Show } from "solid-js";
import { Icons } from "./AppIcons";
import { useAgentSystem } from "../hooks/useAgentSystem";
import { renderMarkdown } from "../utils/markdown";
import { SettingsModal } from "./SettingsModal";
import "./AgentManager.css";

export const AgentManager: Component<{ onClose: () => void }> = (props) => {
  const sys = useAgentSystem();
  const [inputMessage, setInputMessage] = createSignal("");
  const [selectedModel, setSelectedModel] = createSignal("Gemini 3 Pro (High)");
  const [showModelMenu, setShowModelMenu] = createSignal(false);
  const [showSettings, setShowSettings] = createSignal(false);
  let chatContainerRef: HTMLDivElement | undefined;
  
  // Auto-scroll to bottom of chat
  createEffect(() => {
    // track history length to trigger scroll
    sys.chatHistory().length; 
    if (chatContainerRef) {
      setTimeout(() => {
        chatContainerRef!.scrollTop = chatContainerRef!.scrollHeight;
      }, 50);
    }
  });

  // Handle sending message
  const handleSend = () => {
    if (inputMessage().trim()) {
      sys.sendMessage(inputMessage());
      setInputMessage("");
    }
  };

  // Handle Enter key
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  // Helper to get agent icon
  const getAgentIcon = (id: string, name: string) => {
    const lowerId = id.toLowerCase();
    const lowerName = name.toLowerCase();
    if (lowerId.includes("code") || lowerName.includes("code")) return Icons.code();
    if (lowerId.includes("director") || lowerName.includes("director")) return Icons.gitBranch(); // Director orchestrates
    if (lowerId.includes("debug") || lowerName.includes("debug")) return Icons.debug();
    if (lowerId.includes("hardware") || lowerName.includes("hardware")) return Icons.chip();
    if (lowerId.includes("doc") || lowerName.includes("doc")) return Icons.docs();
    // Default
    return Icons.bot();
  };

  return (
    <div class="agent-manager-overlay">
      <Show when={showSettings()}>
        <SettingsModal onClose={() => setShowSettings(false)} />
      </Show>
      <div class="agent-manager-container">
        {/* Header / Title Bar */}
        <div class="am-header">
          <div class="am-header-left">
            <span class="am-logo-icon">{Icons.brain()}</span>
            <span class="am-title">Agent Manager</span>
            <Show when={sys.isProcessing()}>
              <span class="am-status-badge">Thinking...</span>
            </Show>
          </div>
          <div class="am-header-right">
            <button class="am-icon-btn" title="Open Editor">Open Editor</button>
            <button class="am-icon-btn" onClick={props.onClose} title="Close">
              {Icons.x()}
            </button>
          </div>
        </div>

        <div class="am-body">
          {/* sidebar */}
          <div class="am-sidebar">
            <div class="am-section">
              <div class="am-section-header">
                <span>Agents</span>
                <button class="am-icon-btn-small" title="Refresh Agents" onClick={() => sys.refetchAgents()}>{Icons.layout()}</button>
              </div>
              <div class="am-agent-list">
                <For each={sys.agents()}>
                  {(agent) => (
                    <div class="am-agent-item" title={agent.description}>
                      <span class="am-agent-icon">{getAgentIcon(agent.id, agent.name)}</span>
                      <span class="am-agent-name">{agent.name}</span>
                    </div>
                  )}
                </For>
                <Show when={sys.agents.loading}>
                   <div class="am-loading">Loading agents...</div>
                </Show>
              </div>
              
              <button class="am-btn-primary full-width" onClick={() => sys.clearHistory()}>
                <span>+ New Chat</span>
              </button>
            </div>

            <div class="am-section">
              <div class="am-section-header">
                <span>Projects</span>
                 <button class="am-icon-btn-small" title="Refresh Projects" onClick={() => sys.refetchProjects()}>↻</button>
              </div>
              <div class="am-workspace-list">
                <For each={sys.projects()}>
                  {(project) => (
                    <div class="am-workspace-item">
                      <div class="am-workspace-row" onClick={() => sys.setActiveWorkspace(project.name)}>
                         <span class="am-ws-icon">📂</span>
                        <span class="am-ws-name" classList={{ "active": sys.activeWorkspace() === project.name }}>
                          {project.name}
                        </span>
                      </div>
                    </div>
                  )}
                </For>
                <Show when={sys.projects.loading}>
                   <div class="am-loading">Loading projects...</div>
                </Show>
              </div>
            </div>
            
            <div class="am-sidebar-footer">
              <div class="am-footer-item">{Icons.docs()} Knowledge</div>
              <div class="am-footer-item">{Icons.globe()} Browser</div>
              <div class="am-footer-item" onClick={() => setShowSettings(true)}>{Icons.settings()} Settings</div>
              <div class="am-footer-item">{Icons.message()} Provide Feedback</div>
            </div>
          </div>


          {/* Main Content */}
          <div class="am-main">
            <Show 
              when={sys.chatHistory().length > 0}
              fallback={
                <div class="am-welcome-screen">
                  <h1>Start new conversation in <span>{sys.activeWorkspace()}</span></h1>
                  <div class="am-input-box">
                    <input 
                      type="text" 
                      placeholder="Ask anything, @ for context" 
                      value={inputMessage()}
                      onInput={(e) => setInputMessage(e.currentTarget.value)}
                      onKeyDown={handleKeyDown}
                    />
                    <div class="am-input-actions">
                      <div class="am-input-controls-left">
                        <div class="am-context-pill">+ Planning</div>
                        <div class="am-model-pill-container">
                          <div class="am-model-pill" onClick={() => setShowModelMenu(!showModelMenu())}>
                            {selectedModel()} ▼
                          </div>
                          <Show when={showModelMenu()}>
                              <div class="am-model-menu">
                                  <div onClick={() => { setSelectedModel("Gemini 3 Pro (High)"); setShowModelMenu(false); }}>Gemini 3 Pro (High)</div>
                                  <div onClick={() => { setSelectedModel("Gemini 3 Flash (Fast)"); setShowModelMenu(false); }}>Gemini 3 Flash (Fast)</div>
                                  <div onClick={() => { setSelectedModel("Claude 3.5 Sonnet"); setShowModelMenu(false); }}>Claude 3.5 Sonnet</div>
                              </div>
                          </Show>
                        </div>
                      </div>
                      <div class="am-send-actions">
                        <button class="am-mic-btn">{Icons.mic()}</button>
                        <button class="am-send-btn" onClick={handleSend}>{Icons.send()}</button>
                      </div>
                    </div>
                  </div>
                  <div class="am-input-footer">
                    <span>Open editor</span>
                    <span>Use Playground</span>
                  </div>
                </div>
              }
            >
              {/* Chat Interface */}
              <div class="am-chat-interface">
                <div class="am-chat-history" ref={chatContainerRef}>
                  <For each={sys.chatHistory()}>
                    {(msg) => (
                      <div class={`am-chat-message ${msg.role}`}>
                        <div class="am-message-avatar">
                          {msg.role === "user" ? "👤" : getAgentIcon("sys", "AI")}
                        </div>
                        <div class="am-message-content" innerHTML={renderMarkdown(msg.content)}></div>
                      </div>
                    )}
                  </For>
                  {/* Invisible spacer for scrolling to bottom */}
                  <div id="chat-bottom"></div>
                </div>

                {/* Input Area (stick to bottom) */}
                <div class="am-chat-input-area">
                   <div class="am-input-box">
                    <input 
                      type="text" 
                      placeholder="Reply..." 
                      value={inputMessage()}
                      onInput={(e) => setInputMessage(e.currentTarget.value)}
                      onKeyDown={handleKeyDown}
                      disabled={sys.isProcessing()}
                    />
                    <div class="am-input-actions">
                      <div class="am-send-actions">
                         <button class="am-send-btn" onClick={handleSend} disabled={sys.isProcessing()}>
                           {sys.isProcessing() ? "..." : Icons.send()}
                         </button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </Show>
          </div>
        </div>
      </div>
    </div>
  );
};

