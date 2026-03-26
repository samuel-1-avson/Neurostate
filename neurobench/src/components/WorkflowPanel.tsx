import { createSignal, onMount, onCleanup, Show, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { Icons } from "./AppIcons";
import "./WorkflowPanel.css";

// Types matching Rust backend
interface DeviceStatus {
  device_locked: boolean;
  lock_holder_id: string | null;
  lock_holder_kind: string | null; // "flash" | "rtt" | null
  rtt_active: boolean;
  active_rtt_id: string | null;
  active_flash_id: string | null;
  active_jobs_count: number;
  last_terminal: LastTerminal | null;
}

interface LastTerminal {
  kind: string;
  id: string;
  success: boolean;
  reason: string | null;
  error_code: string | null;
  timestamp_ms: number;
}

interface WorkflowStep {
  step: number;
  action: string;
  params: Record<string, unknown>;
  trigger?: string;
  enabled?: boolean;
}

interface WorkflowGuidance {
  workflow: WorkflowStep[];
  chip: string;
  project_path: string;
  start_rtt_after_flash: boolean;
}

interface CancelResult {
  cancelled: [string, string][];
  count: number;
}

// Workflow state machine
type WorkflowState = "idle" | "building" | "flashing" | "rtt" | "failed" | "cancelled";

export default function WorkflowPanel() {
  // Status polling
  const [status, setStatus] = createSignal<DeviceStatus | null>(null);
  const [workflowState, setWorkflowState] = createSignal<WorkflowState>("idle");
  const [statusError, setStatusError] = createSignal<string | null>(null);
  
  // Active job IDs
  const [activeBuildId, setActiveBuildId] = createSignal<string | null>(null);
  const [activeFlashId, setActiveFlashId] = createSignal<string | null>(null);
  const [activeRttId, setActiveRttId] = createSignal<string | null>(null);
  
  // Config
  const [projectPath, setProjectPath] = createSignal("");
  const [chip, setChip] = createSignal("STM32F407VG");
  const [startRtt, setStartRtt] = createSignal(true);
  
  // Messages
  const [messages, setMessages] = createSignal<string[]>([]);
  
  let statusInterval: number | undefined;
  let unlistenBuild: UnlistenFn | undefined;
  let unlistenFlash: UnlistenFn | undefined;
  let unlistenRtt: UnlistenFn | undefined;

  // Poll device status
  const pollStatus = async () => {
    try {
      const result = await invoke<DeviceStatus>("device_status_get");
      setStatus(result);
      setStatusError(null);
      
      // Update workflow state based on status
      if (result.active_flash_id) {
        setWorkflowState("flashing");
      } else if (result.rtt_active) {
        setWorkflowState("rtt");
      } else if (result.active_jobs_count > 0) {
        setWorkflowState("building");
      } else if (workflowState() !== "failed" && workflowState() !== "cancelled") {
        setWorkflowState("idle");
      }
    } catch (err) {
      setStatusError(String(err));
    }
  };
  
  // Run workflow: build → flash → rtt
  const handleRun = async () => {
    if (!projectPath()) {
      addMessage("Error: Project path is required");
      return;
    }
    
    try {
      // Get workflow guidance
      const guidance = await invoke<WorkflowGuidance>("run_chain", {
        projectPath: projectPath(),
        chip: chip(),
        startRtt: startRtt(),
      });
      
      addMessage(`Starting workflow for ${guidance.chip}...`);
      setWorkflowState("building");
      
      // In a full implementation, we'd start streaming_build_start here
      // and chain based on events. For now, show the workflow steps.
      addMessage(`Step 1: ${guidance.workflow[0]?.action || 'build'}`);
      addMessage(`Step 2: ${guidance.workflow[1]?.action || 'flash'} (on build success)`);
      if (startRtt()) {
        addMessage(`Step 3: ${guidance.workflow[2]?.action || 'rtt'} (on flash success)`);
      }
      
    } catch (err) {
      addMessage(`Run failed: ${err}`);
      setWorkflowState("failed");
    }
  };
  
  // Stop all active jobs
  const handleStop = async () => {
    try {
      const result = await invoke<CancelResult>("workflow_cancel");
      addMessage(`Stopped ${result.count} job(s)`);
      
      if (result.count > 0) {
        setWorkflowState("cancelled");
        // Reset to idle after a moment
        setTimeout(() => setWorkflowState("idle"), 2000);
      }
    } catch (err) {
      addMessage(`Stop failed: ${err}`);
    }
  };
  
  // Add a message to the log
  const addMessage = (msg: string) => {
    const timestamp = new Date().toLocaleTimeString();
    setMessages(prev => [...prev.slice(-49), `[${timestamp}] ${msg}`]);
  };
  
  // Setup event listeners
  const setupListeners = async () => {
    // Build events
    unlistenBuild = await listen<any>("build:completed", (event) => {
      const data = event.payload;
      if (data.success) {
        addMessage("Build completed successfully");
        // Trigger flash if in workflow
        if (workflowState() === "building") {
          setWorkflowState("flashing");
          addMessage("Starting flash...");
        }
      } else {
        addMessage(`Build failed: ${data.error || 'unknown error'}`);
        setWorkflowState("failed");
      }
    });
    
    // Flash events
    unlistenFlash = await listen<any>("flash:completed", (event) => {
      const data = event.payload;
      if (data.success) {
        addMessage(`Flash completed: ${data.bytes_written} bytes`);
        // Trigger RTT if in workflow and enabled
        if (workflowState() === "flashing" && startRtt()) {
          setWorkflowState("rtt");
          addMessage("Starting RTT...");
        } else {
          setWorkflowState("idle");
        }
      } else {
        addMessage("Flash failed");
        setWorkflowState("failed");
      }
    });
    
    // RTT events
    unlistenRtt = await listen<any>("rtt:message", (event) => {
      const data = event.payload;
      if (data.messages) {
        addMessage(`RTT: ${data.message_count} messages (${data.dropped_count} dropped)`);
      }
    });
  };

  onMount(() => {
    // Start status polling
    pollStatus();
    statusInterval = setInterval(pollStatus, 300) as unknown as number;
    
    // Setup event listeners
    setupListeners();
  });

  onCleanup(() => {
    if (statusInterval) clearInterval(statusInterval);
    unlistenBuild?.();
    unlistenFlash?.();
    unlistenRtt?.();
  });

  // UI helper: workflow state color
  const stateColor = () => {
    switch (workflowState()) {
      case "building": return "#4CAF50";
      case "flashing": return "#FF9800";
      case "rtt": return "#2196F3";
      case "failed": return "#f44336";
      case "cancelled": return "#9E9E9E";
      default: return "#666";
    }
  };
  
  // UI helper: last terminal status
  const lastTerminalText = () => {
    const lt = status()?.last_terminal;
    if (!lt) return null;
    if (lt.success) {
      return <span class="status-with-icon">{Icons.check()} Last {lt.kind}: Success</span>;
    } else {
      return <span class="status-with-icon">{Icons.errorCircle()} Last {lt.kind}: {lt.error_code || lt.reason || 'failed'}</span>;
    }
  };

  return (
    <div class="workflow-panel">
      {/* Header */}
      <div class="panel-header">
        {Icons.workflow()} <span>Workflow Engine</span>
      </div>

      {/* Status Strip */}
      <div class="status-strip">
        <div class="status-item">
          <span class="status-label">State:</span>
          <span class="status-value" style={{ color: stateColor() }}>
            {workflowState().toUpperCase()}
          </span>
        </div>
        
        <Show when={status()?.device_locked}>
          <div class="status-item">
            <span class="status-label">Lock:</span>
            <span class="status-value">{status()?.lock_holder_kind || 'held'}</span>
          </div>
        </Show>
        
        <Show when={status()?.rtt_active}>
          <div class="status-item rtt-active">
            <span class="status-dot"></span>
            <span>RTT Active</span>
          </div>
        </Show>
        
        <Show when={lastTerminalText()}>
          <div class={`status-item ${status()?.last_terminal?.success ? 'success' : 'error'}`}>
            {lastTerminalText()}
          </div>
        </Show>
        
        <div class="status-item">
          <span class="status-label">Jobs:</span>
          <span class="status-value">{status()?.active_jobs_count || 0}</span>
        </div>
      </div>
      
      {/* Controls */}
      <div class="workflow-controls">
        <div class="control-row">
          <label>
            Project Path:
            <input 
              type="text" 
              value={projectPath()} 
              onInput={(e) => setProjectPath(e.currentTarget.value)}
              placeholder="/path/to/project"
            />
          </label>
        </div>
        
        <div class="control-row">
          <label>
            Chip:
            <select value={chip()} onChange={(e) => setChip(e.currentTarget.value)}>
              <option value="STM32F407VG">STM32F407VG</option>
              <option value="STM32F103C8">STM32F103C8 (Blue Pill)</option>
              <option value="STM32H743ZI">STM32H743ZI</option>
              <option value="nRF52840_xxAA">nRF52840</option>
            </select>
          </label>
          
          <label class="checkbox-label">
            <input 
              type="checkbox" 
              checked={startRtt()} 
              onChange={(e) => setStartRtt(e.currentTarget.checked)}
            />
            Start RTT after flash
          </label>
        </div>
        
        <div class="button-row">
          <button 
            class="run-button" 
            onClick={handleRun}
            disabled={workflowState() !== "idle" && workflowState() !== "failed"}
          >
            <span class="btn-icon">{Icons.play()}</span> Run
          </button>
          
          <button 
            class="stop-button" 
            onClick={handleStop}
            disabled={workflowState() === "idle"}
          >
            <span class="btn-icon">{Icons.stop()}</span> Stop
          </button>
        </div>
      </div>
      
      {/* Message Log */}
      <div class="message-log">
        <For each={messages()}>
          {(msg) => <div class="log-line">{msg}</div>}
        </For>
      </div>
      
      {/* Error display */}
      <Show when={statusError()}>
        <div class="status-error">Status error: {statusError()}</div>
      </Show>


    </div>
  );
}
