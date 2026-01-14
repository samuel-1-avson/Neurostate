/**
 * CommandBar - Professional top navigation/command bar
 * Industrial-grade design following ISA-101 HMI patterns
 */

import { Component, createSignal, Show } from "solid-js";
import "./CommandBar.css";

interface CommandBarProps {
  projectName?: string;
  targetMcu?: string;
  simStatus?: "idle" | "running" | "paused";
  isConnected?: boolean;
  onSimStart?: () => void;
  onSimStop?: () => void;
  onSimStep?: () => void;
  onBuild?: () => void;
  onFlash?: () => void;
  onSettings?: () => void;
  onNewProject?: () => void;
  onOpenProject?: () => void;
  onSaveProject?: () => void;
}

export const CommandBar: Component<CommandBarProps> = (props) => {
  const [showProjectMenu, setShowProjectMenu] = createSignal(false);
  const [showTargetMenu, setShowTargetMenu] = createSignal(false);
  
  const getStatusColor = () => {
    switch (props.simStatus) {
      case "running": return "status-led-ok";
      case "paused": return "status-led-warning";
      default: return "status-led-inactive";
    }
  };
  
  const getConnectionColor = () => {
    return props.isConnected ? "status-led-ok" : "status-led-inactive";
  };

  return (
    <header class="command-bar">
      {/* Left section - Logo and Project */}
      <div class="command-bar-left">
        {/* Logo */}
        <div class="command-logo">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" 
                  stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round"/>
          </svg>
          <span class="logo-text">NeuroBench</span>
          <span class="version-badge">v1.4</span>
        </div>
        
        <div class="divider-vertical" />
        
        {/* Project dropdown */}
        <div class="command-dropdown" classList={{ open: showProjectMenu() }}>
          <button 
            class="command-dropdown-btn"
            onClick={() => setShowProjectMenu(!showProjectMenu())}
            onBlur={() => setTimeout(() => setShowProjectMenu(false), 150)}
          >
            <span class="dropdown-icon">📁</span>
            <span class="dropdown-text">{props.projectName || "Untitled Project"}</span>
            <span class="dropdown-arrow">▾</span>
          </button>
          <Show when={showProjectMenu()}>
            <div class="dropdown-menu">
              <button class="dropdown-item" onClick={props.onNewProject}>
                <span>➕</span> New Project
              </button>
              <button class="dropdown-item" onClick={props.onOpenProject}>
                <span>📂</span> Open Project
              </button>
              <button class="dropdown-item" onClick={props.onSaveProject}>
                <span>💾</span> Save Project
              </button>
            </div>
          </Show>
        </div>
      </div>
      
      {/* Center section - Build/Flash/Simulation controls */}
      <div class="command-bar-center">
        {/* Build actions */}
        <div class="action-group">
          <button 
            class="action-btn" 
            onClick={props.onBuild}
            title="Build Project (Ctrl+B)"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M4 1.5v13h8v-13H4zM3 0h10a1 1 0 011 1v14a1 1 0 01-1 1H3a1 1 0 01-1-1V1a1 1 0 011-1z"/>
              <path d="M5 4h6v1H5zM5 6h6v1H5zM5 8h4v1H5z"/>
            </svg>
            <span>Build</span>
          </button>
          
          <button 
            class="action-btn action-success" 
            onClick={props.onFlash}
            title="Flash to Target (Ctrl+F)"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M9 1L5 8h3v7l4-7H9V1z"/>
            </svg>
            <span>Flash</span>
          </button>
        </div>
        
        <div class="divider-vertical" />
        
        {/* Simulation controls */}
        <div class="action-group">
          <button 
            class="action-btn action-play"
            classList={{ active: props.simStatus === "running" }}
            onClick={props.onSimStart}
            disabled={props.simStatus === "running"}
            title="Start Simulation"
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
              <path d="M3 2l8 5-8 5V2z"/>
            </svg>
          </button>
          
          <button 
            class="action-btn"
            onClick={props.onSimStop}
            disabled={props.simStatus === "idle"}
            title="Stop Simulation"
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
              <rect x="3" y="3" width="8" height="8"/>
            </svg>
          </button>
          
          <button 
            class="action-btn"
            onClick={props.onSimStep}
            disabled={props.simStatus === "running"}
            title="Step Simulation"
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
              <path d="M3 2l5 5-5 5V2z"/>
              <rect x="10" y="2" width="2" height="10"/>
            </svg>
          </button>
        </div>
      </div>
      
      {/* Right section - Target and Status */}
      <div class="command-bar-right">
        {/* Target MCU selector */}
        <div class="command-dropdown" classList={{ open: showTargetMenu() }}>
          <button 
            class="target-selector"
            onClick={() => setShowTargetMenu(!showTargetMenu())}
            onBlur={() => setTimeout(() => setShowTargetMenu(false), 150)}
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <rect x="3" y="3" width="10" height="10" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
              <circle cx="8" cy="8" r="2" fill="currentColor"/>
              <line x1="8" y1="0" x2="8" y2="3" stroke="currentColor" stroke-width="1"/>
              <line x1="8" y1="13" x2="8" y2="16" stroke="currentColor" stroke-width="1"/>
              <line x1="0" y1="8" x2="3" y2="8" stroke="currentColor" stroke-width="1"/>
              <line x1="13" y1="8" x2="16" y2="8" stroke="currentColor" stroke-width="1"/>
            </svg>
            <span>{props.targetMcu || "STM32F407"}</span>
            <span class="dropdown-arrow">▾</span>
          </button>
        </div>
        
        <div class="divider-vertical" />
        
        {/* Status indicators */}
        <div class="status-cluster">
          <div class="status-indicator" title="Simulation Status">
            <div class={`status-led ${getStatusColor()}`} />
            <span class="status-label">SIM</span>
          </div>
          
          <div class="status-indicator" title="Target Connection">
            <div class={`status-led ${getConnectionColor()}`} />
            <span class="status-label">TGT</span>
          </div>
        </div>
        
        <div class="divider-vertical" />
        
        {/* Settings */}
        <button 
          class="icon-btn" 
          onClick={props.onSettings}
          title="Settings"
        >
          <svg width="18" height="18" viewBox="0 0 18 18" fill="currentColor">
            <path d="M9 11.5a2.5 2.5 0 100-5 2.5 2.5 0 000 5z"/>
            <path d="M16.4 7.2l-1.4-.3a6.2 6.2 0 00-.5-1.2l.8-1.2a.5.5 0 00-.1-.6L14 2.8a.5.5 0 00-.6-.1l-1.2.8a6 6 0 00-1.2-.5L10.7 1.6a.5.5 0 00-.5-.4h-2a.5.5 0 00-.5.4L7.4 3a6 6 0 00-1.2.5l-1.2-.8a.5.5 0 00-.6.1l-1.5 1.4a.5.5 0 00-.1.6l.8 1.2a6 6 0 00-.5 1.2l-1.4.3a.5.5 0 00-.4.5v2a.5.5 0 00.4.5l1.4.3a6 6 0 00.5 1.2l-.8 1.2a.5.5 0 00.1.6l1.5 1.4a.5.5 0 00.6.1l1.2-.8a6 6 0 001.2.5l.3 1.4a.5.5 0 00.5.4h2a.5.5 0 00.5-.4l.3-1.4a6 6 0 001.2-.5l1.2.8a.5.5 0 00.6-.1l1.5-1.4a.5.5 0 00.1-.6l-.8-1.2a6 6 0 00.5-1.2l1.4-.3a.5.5 0 00.4-.5v-2a.5.5 0 00-.4-.5zM9 12a3 3 0 110-6 3 3 0 010 6z"/>
          </svg>
        </button>
      </div>
    </header>
  );
};

export default CommandBar;
