/**
 * useAgentSystem - Hook for managing the AI Agent System
 */
import { createSignal, createResource } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { AgentInfo, AssistantResponse, ProjectInfo } from "../types/agent";

export interface ChatMessage {
  role: "user" | "assistant" | "system";
  content: string;
  timestamp: number;
}

export function useAgentSystem() {
  // State
  const [activeWorkspace, setActiveWorkspace] = createSignal<string>("Neurostate");
  const [chatHistory, setChatHistory] = createSignal<ChatMessage[]>([]);
  const [isProcessing, setIsProcessing] = createSignal(false);
  const [processingStatus, setProcessingStatus] = createSignal("");
  
  // Resources (Async Data)
  const [agents, { refetch: refetchAgents }] = createResource<AgentInfo[]>(async () => {
    try {
      return await invoke<AgentInfo[]>("unified_list_agents");
    } catch (e) {
      console.error("Failed to list agents:", e);
      return [];
    }
  });

  const [projects, { refetch: refetchProjects }] = createResource<ProjectInfo[]>(async () => {
    try {
      // Map projects to workspace format
      // Note: Assuming 'project_list' returns a list of project names or objects
      // Adjust generic type based on actual return if needed
      return await invoke<ProjectInfo[]>("project_list");
    } catch (e) {
      console.error("Failed to list projects:", e);
      return [];
    }
  });

  // Actions
  const sendMessage = async (message: string) => {
    if (!message.trim()) return;

    // Add user message
    const userMsg: ChatMessage = {
      role: "user",
      content: message,
      timestamp: Date.now()
    };
    
    setChatHistory(prev => [...prev, userMsg]);
    setIsProcessing(true);
    setProcessingStatus("Thinking...");

    try {
      const response = await invoke<AssistantResponse>("unified_chat", { 
        message,
        userId: "user" // Optional, implementation specific
      });

      // Add assistant response
      const aiMsg: ChatMessage = {
        role: "assistant",
        content: response.message,
        timestamp: Date.now()
      };
      
      setChatHistory(prev => [...prev, aiMsg]);
    } catch (e) {
      console.error("Chat error:", e);
      setChatHistory(prev => [...prev, {
        role: "system",
        content: `Error: ${e}`,
        timestamp: Date.now()
      }]);
    } finally {
      setIsProcessing(false);
      setProcessingStatus("");
    }
  };

  const clearHistory = () => {
    setChatHistory([]);
  };

  return {
    // State
    activeWorkspace,
    setActiveWorkspace,
    chatHistory,
    isProcessing,
    processingStatus,
    
    // Data
    agents,
    projects,
    
    // Actions
    sendMessage,
    clearHistory,
    refetchAgents,
    refetchProjects
  };
}
