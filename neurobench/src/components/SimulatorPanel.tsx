import { createSignal, For, Show, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface QemuPreset {
  machine: string;
  cpu: string | null;
  firmware_path: string;
  gdb_port: number | null;
  serial_output: boolean;
  nographic: boolean;
  memory_mb: number | null;
  extra_args: string[];
}

interface SimulatorPanelProps {
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function SimulatorPanel(props: SimulatorPanelProps) {
  const [qemuAvailable, setQemuAvailable] = createSignal<boolean | null>(null);
  const [qemuVersion, setQemuVersion] = createSignal("");
  const [machines, setMachines] = createSignal<string[]>([]);
  const [presets, setPresets] = createSignal<Record<string, QemuPreset>>({});
  
  // Selected configuration
  const [selectedPreset, setSelectedPreset] = createSignal("");
  const [selectedMachine, setSelectedMachine] = createSignal("lm3s6965evb");
  const [firmwarePath, setFirmwarePath] = createSignal("");
  const [enableGdb, setEnableGdb] = createSignal(false);
  const [gdbPort, setGdbPort] = createSignal(1234);
  
  const [isLoading, setIsLoading] = createSignal(false);

  const addLog = (source: string, message: string, type: "info" | "success" | "warning" | "error" = "info") => {
    props.onLog?.(source, message, type);
  };

  const checkQemu = async () => {
    try {
      const result = await invoke("qemu_check") as { available: boolean };
      setQemuAvailable(result.available);
      
      if (result.available) {
        // Get version
        const versionResult = await invoke("qemu_version") as { version: string };
        setQemuVersion(versionResult.version);
        
        // Get presets
        const presetsResult = await invoke("qemu_get_presets") as Record<string, QemuPreset>;
        setPresets(presetsResult);
        
        addLog("QEMU", `Found: ${versionResult.version}`, "success");
      } else {
        addLog("QEMU", "QEMU not found. Please install and add to PATH.", "warning");
      }
    } catch (e) {
      setQemuAvailable(false);
      addLog("QEMU", `Error checking QEMU: ${e}`, "error");
    }
  };

  const loadMachines = async () => {
    setIsLoading(true);
    try {
      const result = await invoke("qemu_list_machines") as { machines: string[] };
      setMachines(result.machines);
      addLog("QEMU", `Loaded ${result.machines.length} machine types`, "info");
    } catch (e) {
      addLog("QEMU", `Failed to list machines: ${e}`, "error");
    }
    setIsLoading(false);
  };

  const applyPreset = (name: string) => {
    const preset = presets()[name];
    if (preset) {
      setSelectedPreset(name);
      setSelectedMachine(preset.machine);
    }
  };

  onMount(() => {
    checkQemu();
  });

  return (
    <div class="simulator-panel">
      <div class="simulator-header">
        <h3>🎮 Hardware Simulator (QEMU)</h3>
        <button class="refresh-btn" onClick={checkQemu}>
          🔄
        </button>
      </div>

      {/* Loading state */}
      <Show when={qemuAvailable() === null}>
        <div class="loading-state">
          Checking for QEMU...
        </div>
      </Show>

      {/* QEMU not available */}
      <Show when={qemuAvailable() === false}>
        <div class="not-available">
          <div class="warning-icon">⚠️</div>
          <h4>QEMU Not Found</h4>
          <p>To enable hardware simulation, install QEMU:</p>
          <div class="install-steps">
            <div class="step">
              <span class="step-os">Windows:</span>
              <code>winget install QEMU.QEMU</code>
            </div>
            <div class="step">
              <span class="step-os">macOS:</span>
              <code>brew install qemu</code>
            </div>
            <div class="step">
              <span class="step-os">Linux:</span>
              <code>sudo apt install qemu-system-arm</code>
            </div>
          </div>
          <button class="retry-btn" onClick={checkQemu}>
            Retry Detection
          </button>
        </div>
      </Show>

      {/* QEMU available */}
      <Show when={qemuAvailable() === true}>
        <div class="qemu-info">
          <span class="status-badge available">✓ QEMU Available</span>
          <span class="version">{qemuVersion()}</span>
        </div>

        {/* Presets */}
        <div class="config-section">
          <h4>Quick Presets</h4>
          <div class="preset-buttons">
            <For each={Object.keys(presets())}>
              {(name) => (
                <button 
                  class={`preset-btn ${selectedPreset() === name ? "active" : ""}`}
                  onClick={() => applyPreset(name)}
                >
                  {name}
                </button>
              )}
            </For>
          </div>
        </div>

        {/* Machine selection */}
        <div class="config-section">
          <h4>Configuration</h4>
          
          <div class="config-row">
            <label>Machine Type</label>
            <div class="machine-select">
              <select 
                value={selectedMachine()} 
                onChange={(e) => setSelectedMachine(e.target.value)}
              >
                <option value="lm3s6965evb">LM3S6965 EVB (Cortex-M3)</option>
                <option value="lm3s811evb">LM3S811 EVB (Cortex-M3)</option>
                <option value="stm32vldiscovery">STM32 VL Discovery (Cortex-M3)</option>
                <option value="netduino2">Netduino 2 (Cortex-M4)</option>
                <option value="microbit">micro:bit (Cortex-M0)</option>
              </select>
              <button 
                class="load-machines-btn" 
                onClick={loadMachines}
                disabled={isLoading()}
              >
                {isLoading() ? "..." : "All"}
              </button>
            </div>
          </div>

          <div class="config-row">
            <label>Firmware (ELF/BIN)</label>
            <input 
              type="text" 
              placeholder="Path to firmware file..."
              value={firmwarePath()}
              onInput={(e) => setFirmwarePath(e.target.value)}
            />
          </div>

          <div class="config-row checkbox">
            <label>
              <input 
                type="checkbox" 
                checked={enableGdb()}
                onChange={(e) => setEnableGdb(e.target.checked)}
              />
              Enable GDB Server
            </label>
            <Show when={enableGdb()}>
              <input 
                type="number" 
                class="port-input"
                value={gdbPort()}
                onInput={(e) => setGdbPort(parseInt(e.target.value) || 1234)}
                placeholder="Port"
              />
            </Show>
          </div>
        </div>

        {/* Simulation controls */}
        <div class="simulation-controls">
          <button 
            class="run-btn"
            disabled={!firmwarePath()}
            onClick={() => addLog("QEMU", `Would start: ${selectedMachine()} with ${firmwarePath()}`, "info")}
          >
            ▶️ Run Simulation
          </button>
          <button class="debug-btn" disabled={!firmwarePath() || !enableGdb()}>
            🔧 Debug (GDB)
          </button>
        </div>

        {/* Available machines list */}
        <Show when={machines().length > 0}>
          <div class="machines-list">
            <h4>Available Machines ({machines().length})</h4>
            <div class="machines-scroll">
              <For each={machines().slice(0, 50)}>
                {(machine) => (
                  <div 
                    class={`machine-item ${selectedMachine() === machine ? "selected" : ""}`}
                    onClick={() => setSelectedMachine(machine)}
                  >
                    {machine}
                  </div>
                )}
              </For>
            </div>
          </div>
        </Show>
      </Show>

      <style>{`
        .simulator-panel {
          background: var(--bg-secondary, #1a1a2e);
          border: 1px solid var(--border, #333);
          border-radius: 8px;
          padding: 12px;
        }

        .simulator-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 12px;
        }

        .simulator-header h3 {
          margin: 0;
          font-size: 14px;
        }

        .refresh-btn {
          background: transparent;
          border: none;
          cursor: pointer;
          font-size: 16px;
          padding: 4px 8px;
        }

        .loading-state {
          text-align: center;
          padding: 20px;
          color: #888;
        }

        .not-available {
          text-align: center;
          padding: 20px;
        }

        .warning-icon {
          font-size: 40px;
          margin-bottom: 10px;
        }

        .not-available h4 {
          margin: 0 0 10px 0;
          color: #fbbf24;
        }

        .install-steps {
          background: rgba(0,0,0,0.2);
          border-radius: 6px;
          padding: 12px;
          margin: 12px 0;
          text-align: left;
        }

        .step {
          display: flex;
          align-items: center;
          gap: 8px;
          margin-bottom: 8px;
        }

        .step:last-child {
          margin-bottom: 0;
        }

        .step-os {
          color: #888;
          min-width: 70px;
          font-size: 12px;
        }

        .step code {
          background: rgba(59, 130, 246, 0.2);
          padding: 4px 8px;
          border-radius: 4px;
          font-size: 12px;
          color: #60a5fa;
        }

        .retry-btn {
          background: linear-gradient(135deg, #f59e0b, #d97706);
          color: white;
          border: none;
          padding: 10px 20px;
          border-radius: 6px;
          cursor: pointer;
          font-weight: 600;
        }

        .qemu-info {
          display: flex;
          align-items: center;
          gap: 12px;
          margin-bottom: 12px;
        }

        .status-badge {
          padding: 4px 10px;
          border-radius: 4px;
          font-size: 12px;
          font-weight: 600;
        }

        .status-badge.available {
          background: rgba(74, 222, 128, 0.2);
          color: #4ade80;
        }

        .version {
          color: #888;
          font-size: 11px;
        }

        .config-section {
          margin-bottom: 16px;
        }

        .config-section h4 {
          margin: 0 0 10px 0;
          font-size: 12px;
          color: #888;
          text-transform: uppercase;
          letter-spacing: 0.5px;
        }

        .preset-buttons {
          display: flex;
          flex-wrap: wrap;
          gap: 6px;
        }

        .preset-btn {
          background: rgba(255,255,255,0.05);
          border: 1px solid #333;
          color: #ccc;
          padding: 6px 12px;
          border-radius: 4px;
          cursor: pointer;
          font-size: 12px;
          transition: all 0.2s;
        }

        .preset-btn:hover {
          background: rgba(59, 130, 246, 0.2);
        }

        .preset-btn.active {
          background: linear-gradient(135deg, #3b82f6, #2563eb);
          color: white;
          border-color: #3b82f6;
        }

        .config-row {
          display: flex;
          align-items: center;
          gap: 10px;
          margin-bottom: 10px;
        }

        .config-row label {
          color: #888;
          font-size: 12px;
          min-width: 120px;
        }

        .config-row.checkbox label {
          display: flex;
          align-items: center;
          gap: 6px;
          min-width: auto;
        }

        .config-row input[type="text"],
        .config-row select {
          flex: 1;
          background: rgba(255,255,255,0.05);
          border: 1px solid #333;
          color: #fff;
          padding: 8px 12px;
          border-radius: 6px;
          font-size: 12px;
        }

        .machine-select {
          display: flex;
          flex: 1;
          gap: 6px;
        }

        .machine-select select {
          flex: 1;
        }

        .load-machines-btn {
          background: rgba(59, 130, 246, 0.2);
          border: 1px solid rgba(59, 130, 246, 0.3);
          color: #60a5fa;
          padding: 8px 12px;
          border-radius: 6px;
          cursor: pointer;
          font-size: 12px;
        }

        .port-input {
          width: 80px !important;
          flex: none !important;
        }

        .simulation-controls {
          display: flex;
          gap: 10px;
          margin-top: 16px;
        }

        .run-btn {
          flex: 1;
          background: linear-gradient(135deg, #4ade80, #22c55e);
          color: #000;
          border: none;
          padding: 12px 20px;
          border-radius: 6px;
          cursor: pointer;
          font-weight: 600;
          font-size: 14px;
        }

        .debug-btn {
          background: linear-gradient(135deg, #f59e0b, #d97706);
          color: #000;
          border: none;
          padding: 12px 20px;
          border-radius: 6px;
          cursor: pointer;
          font-weight: 600;
          font-size: 14px;
        }

        .run-btn:disabled,
        .debug-btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .machines-list {
          margin-top: 16px;
        }

        .machines-list h4 {
          margin: 0 0 8px 0;
          font-size: 12px;
          color: #888;
        }

        .machines-scroll {
          max-height: 150px;
          overflow-y: auto;
          display: grid;
          grid-template-columns: repeat(3, 1fr);
          gap: 4px;
        }

        .machine-item {
          padding: 4px 8px;
          background: rgba(255,255,255,0.03);
          border-radius: 4px;
          font-size: 11px;
          cursor: pointer;
          color: #888;
          transition: all 0.2s;
        }

        .machine-item:hover {
          background: rgba(59, 130, 246, 0.2);
          color: #fff;
        }

        .machine-item.selected {
          background: rgba(74, 222, 128, 0.2);
          color: #4ade80;
        }
      `}</style>
    </div>
  );
}
