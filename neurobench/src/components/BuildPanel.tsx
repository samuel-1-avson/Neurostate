import { createSignal, For, Show, onMount, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

// Types matching Rust backend
interface ToolchainInfo {
  id: string;
  name: string;
  version: string;
  path: string;
  toolchain_type: string;
  targets: string[];
}

interface BuildResult {
  success: boolean;
  elf_path?: string;
  binary_path?: string;
  errors: CompilerDiagnostic[];
  warnings: CompilerDiagnostic[];
  duration_ms: number;
  output: string;
}

interface CompilerDiagnostic {
  file: string;
  line: number;
  column?: number;
  severity: string;
  code?: string;
  message: string;
  suggestion?: string;
}

interface SizeReport {
  text: number;
  data: number;
  bss: number;
  total: number;
  flash_used: number;
  ram_used: number;
  flash_total: number;
  ram_total: number;
  flash_percent: number;
  ram_percent: number;
}

interface ProbeInfo {
  name: string;
  vendor_id: number;
  product_id: number;
  serial?: string;
  probe_type: string;
}

interface FlashResult {
  success: boolean;
  bytes_written: number;
  duration_ms: number;
  verified: boolean;
  message: string;
}

interface RttMessage {
  channel: number;
  timestamp_ms: number;
  data: string;
}

// Streaming build event types with header
interface EventHeader {
  build_id: string;
  seq: number;
  timestamp_ms: number;
}

interface BuildOutputEvent {
  type: "output";
  header: EventHeader;
  line: string;
  stream: "stdout" | "stderr";
  tool: string;
}

interface BuildDiagnosticEvent {
  type: "diagnostic";
  header: EventHeader;
  diagnostic: CompilerDiagnostic & { 
    tool: string; 
    raw_line: string;
    file_absolute: string;
  };
}

interface BuildProgressEvent {
  type: "progress";
  header: EventHeader;
  phase: string;
  percent: number;
  message: string;
  files_compiled: number;
  files_total: number;
}

interface BuildCompletedEvent {
  type: "completed";
  header: EventHeader;
  success: boolean;
  duration_ms: number;
  error_count: number;
  warning_count: number;
  artifacts?: {
    elf_path: string;
    bin_path?: string;
    map_path?: string;
    size_report?: { text: number; data: number; bss: number; total: number };
  };
}

interface BuildCancelledEvent {
  type: "cancelled";
  header: EventHeader;
  reason: string;
}

interface BuildPanelProps {
  projectPath?: string;
  projectName?: string;
  sourceFiles?: string[];
  mcuTarget?: string;
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function BuildPanel(props: BuildPanelProps) {
  // Toolchain state
  const [toolchains, setToolchains] = createSignal<ToolchainInfo[]>([]);
  const [selectedToolchain, setSelectedToolchain] = createSignal<string | null>(null);
  const [optimization, setOptimization] = createSignal("debug");
  
  // Build state
  const [isBuilding, setIsBuilding] = createSignal(false);
  const [buildResult, setBuildResult] = createSignal<BuildResult | null>(null);
  const [sizeReport, setSizeReport] = createSignal<SizeReport | null>(null);
  
  // Probe state
  const [probes, setProbes] = createSignal<ProbeInfo[]>([]);
  const [selectedProbe, setSelectedProbe] = createSignal<number>(0);
  const [probeConnected, setProbeConnected] = createSignal(false);
  const [isFlashing, setIsFlashing] = createSignal(false);
  const [flashResult, setFlashResult] = createSignal<FlashResult | null>(null);
  
  // RTT state
  const [rttActive, setRttActive] = createSignal(false);
  const [rttMessages, setRttMessages] = createSignal<RttMessage[]>([]);
  
  // Active tab
  const [activeTab, setActiveTab] = createSignal<"build" | "flash" | "rtt" | "diagnostics">("build");
  
  // Streaming build state
  const [currentBuildId, setCurrentBuildId] = createSignal<string | null>(null);
  const [buildOutput, setBuildOutput] = createSignal<string[]>([]);
  const [buildProgress, setBuildProgress] = createSignal<{ phase: string; percent: number } | null>(null);
  const [diagnostics, setDiagnostics] = createSignal<CompilerDiagnostic[]>([]);
  
  let rttInterval: ReturnType<typeof setInterval> | null = null;
  let buildUnlisteners: UnlistenFn[] = [];

  const optimizationOptions = [
    { value: "debug", label: "Debug (-Og)" },
    { value: "release", label: "Release (-O2)" },
    { value: "min_size", label: "Min Size (-Os)" },
    { value: "max_speed", label: "Max Speed (-O3)" },
  ];

  const discoverToolchains = async () => {
    try {
      const result = await invoke("toolchain_discover") as ToolchainInfo[];
      setToolchains(result);
      if (result.length > 0 && !selectedToolchain()) {
        setSelectedToolchain(result[0].id);
      }
      props.onLog?.("Toolchain", `Found ${result.length} toolchain(s)`, "info");
    } catch (e) {
      props.onLog?.("Toolchain", `Discovery failed: ${e}`, "error");
    }
  };

  const listProbes = async () => {
    try {
      const result = await invoke("probe_list") as ProbeInfo[];
      setProbes(result);
      if (result.length > 0) {
        props.onLog?.("Probe", `Found ${result.length} debug probe(s)`, "info");
      }
    } catch (e) {
      props.onLog?.("Probe", `Failed to list probes: ${e}`, "error");
    }
  };

  // Set up streaming build event listeners
  const setupBuildListeners = async (buildId: string) => {
    // Clean up old listeners
    for (const unlisten of buildUnlisteners) {
      unlisten();
    }
    buildUnlisteners = [];
    
    // Listen for output lines
    buildUnlisteners.push(await listen<BuildOutputEvent>("build:output", (event) => {
      if (event.payload.header.build_id === buildId) {
        setBuildOutput(prev => [...prev.slice(-500), event.payload.line]); // Keep last 500 lines
      }
    }));
    
    // Listen for diagnostics
    buildUnlisteners.push(await listen<BuildDiagnosticEvent>("build:diagnostic", (event) => {
      if (event.payload.header.build_id === buildId) {
        setDiagnostics(prev => [...prev, event.payload.diagnostic]);
      }
    }));
    
    // Listen for progress
    buildUnlisteners.push(await listen<BuildProgressEvent>("build:progress", (event) => {
      if (event.payload.header.build_id === buildId) {
        setBuildProgress({ phase: event.payload.phase, percent: event.payload.percent });
        props.onLog?.("Build", event.payload.message, "info");
      }
    }));
    
    // Listen for completion
    buildUnlisteners.push(await listen<BuildCompletedEvent>("build:completed", (event) => {
      if (event.payload.header.build_id === buildId) {
        setIsBuilding(false);
        setCurrentBuildId(null);
        setBuildProgress(null);
        
        const elfPath = event.payload.artifacts?.elf_path;
        
        // Create result from event
        const result: BuildResult = {
          success: event.payload.success,
          elf_path: elfPath,
          errors: diagnostics().filter(d => d.severity === "error"),
          warnings: diagnostics().filter(d => d.severity === "warning"),
          duration_ms: event.payload.duration_ms,
          output: buildOutput().join("\n"),
        };
        setBuildResult(result);
        
        if (event.payload.success) {
          props.onLog?.("Build", `✓ Build succeeded in ${event.payload.duration_ms}ms`, "success");
          // Set size from artifacts if available
          const sizeReport = event.payload.artifacts?.size_report;
          if (sizeReport) {
            setSizeReport({
              text: sizeReport.text,
              data: sizeReport.data,
              bss: sizeReport.bss,
              total: sizeReport.total,
              flash_used: sizeReport.text + sizeReport.data,
              ram_used: sizeReport.data + sizeReport.bss,
              flash_total: 1024 * 1024, // 1MB default
              ram_total: 128 * 1024,    // 128KB default
              flash_percent: ((sizeReport.text + sizeReport.data) / (1024 * 1024)) * 100,
              ram_percent: ((sizeReport.data + sizeReport.bss) / (128 * 1024)) * 100,
            });
          }
        } else {
          props.onLog?.("Build", `✗ Build failed with ${event.payload.error_count} error(s)`, "error");
        }
        
        // Clean up listeners
        for (const unlisten of buildUnlisteners) {
          unlisten();
        }
        buildUnlisteners = [];
      }
    }));
    
    // Listen for cancellation
    buildUnlisteners.push(await listen<BuildCancelledEvent>("build:cancelled", (event) => {
      if (event.payload.header.build_id === buildId) {
        setIsBuilding(false);
        setCurrentBuildId(null);
        setBuildProgress(null);
        props.onLog?.("Build", `Build cancelled: ${event.payload.reason}`, "warning");
        
        for (const unlisten of buildUnlisteners) {
          unlisten();
        }
        buildUnlisteners = [];
      }
    }));
  };

  const build = async () => {
    setIsBuilding(true);
    setBuildResult(null);
    setSizeReport(null);
    setBuildOutput([]);
    setDiagnostics([]);
    setBuildProgress({ phase: "Preparing", percent: 0 });
    
    try {
      // Build config for streaming build
      const config = {
        project_path: props.projectPath || ".",
        mcu_target: props.mcuTarget || "cortex-m4",
        optimization: optimization(),
        defines: { "STM32F407xx": "", "USE_HAL_DRIVER": "" },
        include_paths: ["inc", "Drivers/CMSIS/Include", "Drivers/STM32F4xx_HAL_Driver/Inc"],
        source_files: props.sourceFiles || ["src/main.c"],
        linker_script: "STM32F407VGTx_FLASH.ld",
      };
      
      // Start streaming build - returns build_id immediately
      const buildId = await invoke("streaming_build_start", { config }) as string;
      setCurrentBuildId(buildId);
      
      // Set up event listeners for this build
      await setupBuildListeners(buildId);
      
      props.onLog?.("Build", `Started build ${buildId.substring(0, 8)}...`, "info");
    } catch (e) {
      setIsBuilding(false);
      setBuildProgress(null);
      props.onLog?.("Build", `Build error: ${e}`, "error");
    }
  };

  const cancelBuild = async () => {
    const buildId = currentBuildId();
    if (buildId) {
      try {
        await invoke("streaming_build_cancel", { buildId });
        props.onLog?.("Build", "Cancelling build...", "warning");
      } catch (e) {
        props.onLog?.("Build", `Cancel failed: ${e}`, "error");
      }
    }
  };

  const clean = async () => {
    try {
      await invoke("toolchain_clean", { projectPath: props.projectPath || "." });
      setBuildResult(null);
      setSizeReport(null);
      props.onLog?.("Build", "Project cleaned", "info");
    } catch (e) {
      props.onLog?.("Build", `Clean failed: ${e}`, "error");
    }
  };

  const connectProbe = async () => {
    try {
      const probe = probes()[selectedProbe()];
      await invoke("probe_connect", {
        config: {
          protocol: "swd",
          speed_khz: 4000,
          target: props.mcuTarget || "STM32F407VGTx",
          reset_mode: "halt_after_reset",
        }
      });
      setProbeConnected(true);
      props.onLog?.("Probe", `Connected to ${probe.name}`, "success");
    } catch (e) {
      props.onLog?.("Probe", `Connection failed: ${e}`, "error");
    }
  };

  const disconnectProbe = async () => {
    try {
      await invoke("probe_disconnect");
      setProbeConnected(false);
      stopRtt();
      props.onLog?.("Probe", "Disconnected", "info");
    } catch (e) {
      props.onLog?.("Probe", `Disconnect failed: ${e}`, "error");
    }
  };

  const flash = async () => {
    const result = buildResult();
    if (!result?.elf_path) {
      props.onLog?.("Flash", "No firmware to flash. Build first.", "warning");
      return;
    }
    
    if (!probeConnected()) {
      props.onLog?.("Flash", "Connect to probe first", "warning");
      return;
    }
    
    setIsFlashing(true);
    setFlashResult(null);
    
    try {
      const flashRes = await invoke("probe_flash", { 
        elfPath: result.elf_path, 
        verify: true 
      }) as FlashResult;
      
      setFlashResult(flashRes);
      
      if (flashRes.success) {
        props.onLog?.("Flash", `✓ Flashed ${(flashRes.bytes_written / 1024).toFixed(1)}KB in ${flashRes.duration_ms}ms`, "success");
      } else {
        props.onLog?.("Flash", `✗ Flash failed: ${flashRes.message}`, "error");
      }
    } catch (e) {
      props.onLog?.("Flash", `Flash error: ${e}`, "error");
    }
    
    setIsFlashing(false);
  };

  const resetTarget = async (mode: string) => {
    if (!probeConnected()) return;
    
    try {
      await invoke("probe_reset", { mode });
      props.onLog?.("Probe", `Target reset (${mode})`, "info");
    } catch (e) {
      props.onLog?.("Probe", `Reset failed: ${e}`, "error");
    }
  };

  const startRtt = async () => {
    if (!probeConnected()) {
      props.onLog?.("RTT", "Connect probe first", "warning");
      return;
    }
    
    try {
      await invoke("rtt_start", { channel: 0 });
      setRttActive(true);
      setRttMessages([]);
      
      // Poll for RTT messages
      rttInterval = setInterval(async () => {
        try {
          const msgs = await invoke("rtt_read") as RttMessage[];
          if (msgs.length > 0) {
            setRttMessages(prev => [...prev, ...msgs].slice(-1000)); // Keep last 1000
          }
        } catch (e) {
          // Silent - RTT might not have data
        }
      }, 100);
      
      props.onLog?.("RTT", "Started RTT streaming", "success");
    } catch (e) {
      props.onLog?.("RTT", `RTT start failed: ${e}`, "error");
    }
  };

  const stopRtt = async () => {
    if (rttInterval) {
      clearInterval(rttInterval);
      rttInterval = null;
    }
    
    try {
      await invoke("rtt_stop");
      setRttActive(false);
      props.onLog?.("RTT", "Stopped RTT streaming", "info");
    } catch (e) {
      // Silent
    }
  };

  onMount(() => {
    discoverToolchains();
    listProbes();
  });

  onCleanup(() => {
    if (rttInterval) {
      clearInterval(rttInterval);
    }
  });

  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  };

  return (
    <div class="build-panel">
      {/* Tab bar */}
      <div class="tab-bar">
        <button class={activeTab() === "build" ? "active" : ""} onClick={() => setActiveTab("build")}>
          🔨 Build
        </button>
        <button class={activeTab() === "flash" ? "active" : ""} onClick={() => setActiveTab("flash")}>
          ⚡ Flash
        </button>
        <button class={activeTab() === "rtt" ? "active" : ""} onClick={() => setActiveTab("rtt")}>
          📡 RTT
        </button>
        <button class={activeTab() === "diagnostics" ? "active" : ""} onClick={() => setActiveTab("diagnostics")}>
          🔍 Diagnostics
        </button>
      </div>

      {/* Build Tab */}
      <Show when={activeTab() === "build"}>
        <div class="tab-content">
          {/* Toolchain selector */}
          <div class="section">
            <label>Toolchain</label>
            <div class="toolchain-row">
              <select 
                value={selectedToolchain() || ""} 
                onChange={(e) => setSelectedToolchain(e.target.value)}
              >
                <For each={toolchains()}>
                  {(tc) => <option value={tc.id}>{tc.name} ({tc.version})</option>}
                </For>
              </select>
              <button class="icon-btn" onClick={discoverToolchains} title="Refresh">🔄</button>
            </div>
            <Show when={toolchains().length === 0}>
              <div class="warning-box">
                ⚠️ No toolchain found. Install ARM GCC or Rust embedded.
              </div>
            </Show>
          </div>

          {/* Optimization */}
          <div class="section">
            <label>Optimization</label>
            <select value={optimization()} onChange={(e) => setOptimization(e.target.value)}>
              <For each={optimizationOptions}>
                {(opt) => <option value={opt.value}>{opt.label}</option>}
              </For>
            </select>
          </div>

          {/* Build buttons */}
          <div class="button-row">
            <Show when={!isBuilding()}>
              <button class="primary-btn" onClick={build} disabled={toolchains().length === 0}>
                🔨 Build
              </button>
            </Show>
            <Show when={isBuilding()}>
              <button class="danger-btn" onClick={cancelBuild}>
                ⏹️ Cancel
              </button>
            </Show>
            <button class="secondary-btn" onClick={clean} disabled={isBuilding()}>🗑️ Clean</button>
          </div>

          {/* Progress bar during build */}
          <Show when={buildProgress()}>
            <div class="progress-section">
              <div class="progress-header">
                <span>{buildProgress()!.phase}</span>
                <span>{buildProgress()!.percent}%</span>
              </div>
              <div class="progress-bar">
                <div class="progress-fill" style={{ width: `${buildProgress()!.percent}%` }} />
              </div>
            </div>
          </Show>

          {/* Live build output */}
          <Show when={isBuilding() && buildOutput().length > 0}>
            <div class="build-output">
              <div class="output-header">Live Output</div>
              <div class="output-scroll">
                <For each={buildOutput().slice(-20)}>
                  {(line) => <div class="output-line">{line}</div>}
                </For>
              </div>
            </div>
          </Show>

          {/* Build result */}
          <Show when={buildResult()}>
            <div class={`result-box ${buildResult()!.success ? "success" : "error"}`}>
              <div class="result-header">
                {buildResult()!.success ? "✓ Build Successful" : "✗ Build Failed"}
                <span class="duration">{buildResult()!.duration_ms}ms</span>
              </div>
              <Show when={buildResult()!.errors.length > 0}>
                <div class="error-list">
                  <For each={buildResult()!.errors.slice(0, 5)}>
                    {(err) => (
                      <div class="error-item">
                        <span class="file">{err.file}:{err.line}</span>
                        <span class="msg">{err.message}</span>
                        <Show when={err.suggestion}>
                          <span class="suggestion">💡 {err.suggestion}</span>
                        </Show>
                      </div>
                    )}
                  </For>
                </div>
              </Show>
            </div>
          </Show>

          {/* Size report */}
          <Show when={sizeReport()}>
            <div class="size-report">
              <div class="size-header">📊 Memory Usage</div>
              <div class="size-bars">
                <div class="size-bar">
                  <label>Flash ({formatBytes(sizeReport()!.flash_used)} / {formatBytes(sizeReport()!.flash_total)})</label>
                  <div class="bar-bg">
                    <div class="bar-fill flash" style={{ width: `${sizeReport()!.flash_percent}%` }} />
                  </div>
                  <span class="percent">{sizeReport()!.flash_percent.toFixed(1)}%</span>
                </div>
                <div class="size-bar">
                  <label>RAM ({formatBytes(sizeReport()!.ram_used)} / {formatBytes(sizeReport()!.ram_total)})</label>
                  <div class="bar-bg">
                    <div class="bar-fill ram" style={{ width: `${sizeReport()!.ram_percent}%` }} />
                  </div>
                  <span class="percent">{sizeReport()!.ram_percent.toFixed(1)}%</span>
                </div>
              </div>
              <div class="size-details">
                <span>.text: {formatBytes(sizeReport()!.text)}</span>
                <span>.data: {formatBytes(sizeReport()!.data)}</span>
                <span>.bss: {formatBytes(sizeReport()!.bss)}</span>
              </div>
            </div>
          </Show>
        </div>
      </Show>

      {/* Flash Tab */}
      <Show when={activeTab() === "flash"}>
        <div class="tab-content">
          {/* Probe selector */}
          <div class="section">
            <label>Debug Probe</label>
            <div class="probe-row">
              <select 
                value={selectedProbe()} 
                onChange={(e) => setSelectedProbe(parseInt(e.target.value))}
                disabled={probeConnected()}
              >
                <For each={probes()}>
                  {(probe, idx) => <option value={idx()}>{probe.name} ({probe.probe_type})</option>}
                </For>
              </select>
              <button class="icon-btn" onClick={listProbes} title="Refresh" disabled={probeConnected()}>🔄</button>
            </div>
            <Show when={probes().length === 0}>
              <div class="warning-box">
                ⚠️ No debug probe detected. Connect ST-Link, J-Link, or CMSIS-DAP.
              </div>
            </Show>
          </div>

          {/* Connection */}
          <div class="button-row">
            <Show when={!probeConnected()}>
              <button class="primary-btn" onClick={connectProbe} disabled={probes().length === 0}>
                🔌 Connect
              </button>
            </Show>
            <Show when={probeConnected()}>
              <button class="danger-btn" onClick={disconnectProbe}>
                ❌ Disconnect
              </button>
            </Show>
          </div>

          {/* Flash controls */}
          <Show when={probeConnected()}>
            <div class="section">
              <label>Flash Firmware</label>
              <button 
                class="primary-btn large" 
                onClick={flash} 
                disabled={isFlashing() || !buildResult()?.elf_path}
              >
                {isFlashing() ? "⏳ Flashing..." : "⚡ Flash"}
              </button>
            </div>

            <div class="button-row reset-row">
              <button class="secondary-btn" onClick={() => resetTarget("software")}>🔄 Reset</button>
              <button class="secondary-btn" onClick={() => resetTarget("halt")}>⏸️ Halt</button>
              <button class="secondary-btn" onClick={() => invoke("probe_resume")}>▶️ Resume</button>
            </div>
          </Show>

          {/* Flash result */}
          <Show when={flashResult()}>
            <div class={`result-box ${flashResult()!.success ? "success" : "error"}`}>
              <div class="result-header">
                {flashResult()!.success ? "✓ Flash Successful" : "✗ Flash Failed"}
              </div>
              <div class="flash-details">
                <span>Wrote: {formatBytes(flashResult()!.bytes_written)}</span>
                <span>Time: {flashResult()!.duration_ms}ms</span>
                <span>Verified: {flashResult()!.verified ? "Yes" : "No"}</span>
              </div>
            </div>
          </Show>
        </div>
      </Show>

      {/* RTT Tab */}
      <Show when={activeTab() === "rtt"}>
        <div class="tab-content">
          <div class="rtt-controls">
            <Show when={!rttActive()}>
              <button class="primary-btn" onClick={startRtt} disabled={!probeConnected()}>
                ▶️ Start RTT
              </button>
            </Show>
            <Show when={rttActive()}>
              <button class="danger-btn" onClick={stopRtt}>
                ⏹️ Stop RTT
              </button>
              <span class="live-indicator">● LIVE</span>
            </Show>
            <button class="secondary-btn" onClick={() => setRttMessages([])}>🗑️ Clear</button>
          </div>

          <div class="rtt-output">
            <For each={rttMessages()}>
              {(msg) => (
                <div class="rtt-line">
                  <span class="rtt-time">[{msg.timestamp_ms}]</span>
                  <span class="rtt-data">{msg.data}</span>
                </div>
              )}
            </For>
            <Show when={rttMessages().length === 0}>
              <div class="rtt-empty">
                {rttActive() ? "Waiting for RTT data..." : "Start RTT to see device output"}
              </div>
            </Show>
          </div>
        </div>
      </Show>

      {/* Diagnostics Tab */}
      <Show when={activeTab() === "diagnostics"}>
        <div class="tab-content">
          <div class="section">
            <label>Build Diagnostics</label>
            <Show when={buildResult()}>
              <div class="diag-summary">
                <span class="diag-item errors">{buildResult()!.errors.length} Errors</span>
                <span class="diag-item warnings">{buildResult()!.warnings.length} Warnings</span>
              </div>
              <div class="diag-list">
                <For each={[...buildResult()!.errors, ...buildResult()!.warnings]}>
                  {(diag) => (
                    <div class={`diag-entry ${diag.severity}`}>
                      <div class="diag-loc">{diag.file}:{diag.line}{diag.column ? `:${diag.column}` : ""}</div>
                      <div class="diag-msg">{diag.message}</div>
                      <Show when={diag.suggestion}>
                        <div class="diag-sug">💡 {diag.suggestion}</div>
                      </Show>
                    </div>
                  )}
                </For>
              </div>
            </Show>
            <Show when={!buildResult()}>
              <div class="empty-state">Build project to see diagnostics</div>
            </Show>
          </div>
        </div>
      </Show>

      <style>{`
        .build-panel {
          background: var(--bg-secondary, #1a1a2e);
          border: 1px solid var(--border, #333);
          border-radius: 8px;
          overflow: hidden;
          height: 100%;
          display: flex;
          flex-direction: column;
        }

        .tab-bar {
          display: flex;
          background: rgba(0,0,0,0.3);
          border-bottom: 1px solid #333;
        }

        .tab-bar button {
          flex: 1;
          background: transparent;
          border: none;
          color: #888;
          padding: 10px 8px;
          font-size: 11px;
          cursor: pointer;
          border-bottom: 2px solid transparent;
          transition: all 0.2s;
        }

        .tab-bar button:hover { color: #fff; }
        .tab-bar button.active {
          color: #ef4444;
          border-bottom-color: #ef4444;
          background: rgba(239, 68, 68, 0.1);
        }

        .tab-content {
          flex: 1;
          padding: 12px;
          overflow-y: auto;
        }

        .section {
          margin-bottom: 16px;
        }

        .section label {
          display: block;
          font-size: 11px;
          color: #888;
          margin-bottom: 6px;
        }

        .section select {
          width: 100%;
          background: rgba(255,255,255,0.05);
          border: 1px solid #333;
          color: #fff;
          padding: 8px 12px;
          border-radius: 6px;
        }

        .toolchain-row, .probe-row {
          display: flex;
          gap: 8px;
        }

        .toolchain-row select, .probe-row select { flex: 1; }

        .icon-btn {
          background: rgba(255,255,255,0.1);
          border: none;
          color: #ccc;
          width: 36px;
          border-radius: 6px;
          cursor: pointer;
        }

        .button-row {
          display: flex;
          gap: 8px;
          margin-bottom: 12px;
        }

        .primary-btn, .secondary-btn, .danger-btn {
          flex: 1;
          padding: 10px;
          border: none;
          border-radius: 6px;
          font-weight: 600;
          cursor: pointer;
          transition: all 0.2s;
        }

        .primary-btn {
          background: linear-gradient(135deg, #ef4444, #dc2626);
          color: white;
        }
        .primary-btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .secondary-btn {
          background: rgba(255,255,255,0.1);
          color: #ccc;
        }

        .danger-btn {
          background: #dc2626;
          color: white;
        }

        .large { padding: 14px; font-size: 14px; }

        .warning-box {
          background: rgba(234, 179, 8, 0.1);
          border: 1px solid rgba(234, 179, 8, 0.3);
          color: #eab308;
          padding: 10px;
          border-radius: 6px;
          font-size: 11px;
          margin-top: 8px;
        }

        .result-box {
          border-radius: 6px;
          padding: 12px;
          margin-top: 12px;
        }
        .result-box.success {
          background: rgba(34, 197, 94, 0.1);
          border: 1px solid rgba(34, 197, 94, 0.3);
        }
        .result-box.error {
          background: rgba(239, 68, 68, 0.1);
          border: 1px solid rgba(239, 68, 68, 0.3);
        }

        .result-header {
          display: flex;
          justify-content: space-between;
          font-weight: 600;
          margin-bottom: 8px;
        }
        .result-box.success .result-header { color: #22c55e; }
        .result-box.error .result-header { color: #ef4444; }

        .duration { color: #888; font-weight: normal; font-size: 11px; }

        .error-list { font-size: 11px; }
        .error-item {
          padding: 6px 0;
          border-top: 1px solid rgba(255,255,255,0.05);
        }
        .error-item .file { color: #f97316; display: block; }
        .error-item .msg { color: #ef4444; }
        .error-item .suggestion { color: #22c55e; display: block; margin-top: 4px; }

        .size-report {
          background: rgba(0,0,0,0.2);
          border-radius: 6px;
          padding: 12px;
          margin-top: 12px;
        }
        .size-header {
          font-weight: 600;
          margin-bottom: 12px;
        }
        .size-bar {
          margin-bottom: 8px;
        }
        .size-bar label {
          font-size: 10px;
          color: #888;
          margin-bottom: 4px;
        }
        .size-bar .bar-bg {
          height: 8px;
          background: rgba(255,255,255,0.1);
          border-radius: 4px;
          overflow: hidden;
        }
        .size-bar .bar-fill {
          height: 100%;
          border-radius: 4px;
          transition: width 0.3s;
        }
        .bar-fill.flash { background: linear-gradient(90deg, #3b82f6, #6366f1); }
        .bar-fill.ram { background: linear-gradient(90deg, #22c55e, #10b981); }
        .size-bar .percent {
          font-size: 10px;
          color: #888;
          float: right;
          margin-top: -14px;
        }
        .size-details {
          display: flex;
          gap: 16px;
          font-size: 10px;
          color: #888;
          margin-top: 8px;
        }

        .flash-details {
          display: flex;
          gap: 16px;
          font-size: 11px;
          color: #888;
        }

        .reset-row {
          margin-top: 12px;
        }

        .rtt-controls {
          display: flex;
          gap: 8px;
          margin-bottom: 12px;
          align-items: center;
        }
        .live-indicator {
          color: #ef4444;
          font-size: 11px;
          animation: pulse 1s infinite;
        }
        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.5; }
        }

        .rtt-output {
          background: #000;
          border-radius: 6px;
          padding: 8px;
          height: 300px;
          overflow-y: auto;
          font-family: 'Fira Code', monospace;
          font-size: 11px;
        }
        .rtt-line {
          display: flex;
          gap: 8px;
        }
        .rtt-time { color: #666; }
        .rtt-data { color: #22c55e; }
        .rtt-empty {
          color: #666;
          text-align: center;
          padding: 40px;
        }

        .diag-summary {
          display: flex;
          gap: 16px;
          margin-bottom: 12px;
        }
        .diag-item {
          padding: 4px 12px;
          border-radius: 4px;
          font-size: 11px;
        }
        .diag-item.errors { background: rgba(239, 68, 68, 0.2); color: #ef4444; }
        .diag-item.warnings { background: rgba(234, 179, 8, 0.2); color: #eab308; }

        .diag-list {
          max-height: 300px;
          overflow-y: auto;
        }
        .diag-entry {
          padding: 8px;
          border-radius: 4px;
          margin-bottom: 6px;
          font-size: 11px;
        }
        .diag-entry.error { background: rgba(239, 68, 68, 0.1); }
        .diag-entry.warning { background: rgba(234, 179, 8, 0.1); }
        .diag-loc { color: #f97316; margin-bottom: 4px; }
        .diag-msg { color: #fff; }
        .diag-sug { color: #22c55e; margin-top: 4px; }

        .progress-section {
          background: rgba(0,0,0,0.2);
          border-radius: 6px;
          padding: 10px;
          margin-bottom: 12px;
        }
        .progress-header {
          display: flex;
          justify-content: space-between;
          font-size: 11px;
          color: #888;
          margin-bottom: 6px;
        }
        .progress-bar {
          height: 6px;
          background: rgba(255,255,255,0.1);
          border-radius: 3px;
          overflow: hidden;
        }
        .progress-fill {
          height: 100%;
          background: linear-gradient(90deg, #ef4444, #f97316);
          border-radius: 3px;
          transition: width 0.3s;
        }

        .build-output {
          background: #000;
          border-radius: 6px;
          margin-bottom: 12px;
          max-height: 150px;
          overflow: hidden;
        }
        .build-output .output-header {
          background: rgba(255,255,255,0.05);
          padding: 6px 10px;
          font-size: 10px;
          color: #888;
        }
        .output-scroll {
          padding: 8px;
          max-height: 120px;
          overflow-y: auto;
        }
        .output-line {
          font-family: 'Fira Code', monospace;
          font-size: 10px;
          color: #22c55e;
          white-space: nowrap;
        }

        .empty-state {
          color: #666;
          text-align: center;
          padding: 40px;
        }
      `}</style>
    </div>
  );
}
