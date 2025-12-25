import { createSignal, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface SectionSize {
  name: string;
  size: number;
  percent: number;
}

interface MemoryAnalysis {
  total_flash: number;
  used_flash: number;
  total_ram: number;
  used_ram: number;
  section_sizes: SectionSize[];
}

interface MemoryPanelProps {
  code?: string;
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function MemoryPanel(props: MemoryPanelProps) {
  const [selectedMcu, setSelectedMcu] = createSignal("STM32F407VG");
  const [analysis, setAnalysis] = createSignal<MemoryAnalysis | null>(null);
  const [isLoading, setIsLoading] = createSignal(false);

  const mcuOptions = [
    { name: "STM32F407VG", flash: "1MB", ram: "128KB" },
    { name: "STM32F103C8", flash: "64KB", ram: "20KB" },
    { name: "STM32F411RE", flash: "512KB", ram: "128KB" },
    { name: "ESP32", flash: "4MB", ram: "520KB" },
    { name: "nRF52832", flash: "512KB", ram: "64KB" },
  ];

  const analyzeMemory = async () => {
    const code = props.code || "";
    if (!code.trim()) {
      props.onLog?.("Memory", "No code to analyze", "warning");
      return;
    }

    setIsLoading(true);
    try {
      const result = await invoke("memory_estimate", {
        code,
        mcu: selectedMcu(),
      }) as MemoryAnalysis;
      
      setAnalysis(result);
      props.onLog?.("Memory", `Analysis complete for ${selectedMcu()}`, "success");
    } catch (e) {
      props.onLog?.("Memory", `Analysis failed: ${e}`, "error");
    }
    setIsLoading(false);
  };

  const formatBytes = (bytes: number) => {
    if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(2)} KB`;
    return `${bytes} B`;
  };

  const getUsageColor = (percent: number) => {
    if (percent > 80) return "#ef4444";
    if (percent > 50) return "#fbbf24";
    return "#4ade80";
  };

  return (
    <div class="memory-panel">
      <div class="panel-header">
        <h3>💾 Memory Analyzer</h3>
      </div>

      {/* MCU Selection */}
      <div class="mcu-select">
        <label>Target MCU</label>
        <select value={selectedMcu()} onChange={(e) => setSelectedMcu(e.target.value)}>
          <For each={mcuOptions}>
            {(mcu) => (
              <option value={mcu.name}>
                {mcu.name} ({mcu.flash} Flash, {mcu.ram} RAM)
              </option>
            )}
          </For>
        </select>
      </div>

      <button class="analyze-btn" onClick={analyzeMemory} disabled={isLoading()}>
        {isLoading() ? "Analyzing..." : "Analyze Memory"}
      </button>

      {/* Results */}
      <Show when={analysis()}>
        <div class="results">
          {/* Flash Usage */}
          <div class="usage-block">
            <div class="usage-header">
              <span class="usage-label">Flash</span>
              <span class="usage-value">
                {formatBytes(analysis()!.used_flash)} / {formatBytes(analysis()!.total_flash)}
              </span>
            </div>
            <div class="usage-bar">
              <div 
                class="usage-fill" 
                style={{ 
                  width: `${(analysis()!.used_flash / analysis()!.total_flash) * 100}%`,
                  background: getUsageColor((analysis()!.used_flash / analysis()!.total_flash) * 100)
                }}
              ></div>
            </div>
            <span class="usage-percent">
              {((analysis()!.used_flash / analysis()!.total_flash) * 100).toFixed(1)}%
            </span>
          </div>

          {/* RAM Usage */}
          <div class="usage-block">
            <div class="usage-header">
              <span class="usage-label">RAM</span>
              <span class="usage-value">
                {formatBytes(analysis()!.used_ram)} / {formatBytes(analysis()!.total_ram)}
              </span>
            </div>
            <div class="usage-bar">
              <div 
                class="usage-fill" 
                style={{ 
                  width: `${(analysis()!.used_ram / analysis()!.total_ram) * 100}%`,
                  background: getUsageColor((analysis()!.used_ram / analysis()!.total_ram) * 100)
                }}
              ></div>
            </div>
            <span class="usage-percent">
              {((analysis()!.used_ram / analysis()!.total_ram) * 100).toFixed(1)}%
            </span>
          </div>

          {/* Section breakdown */}
          <div class="sections">
            <h4>Section Sizes</h4>
            <For each={analysis()!.section_sizes}>
              {(section) => (
                <div class="section-row">
                  <span class="section-name">{section.name}</span>
                  <span class="section-size">{formatBytes(section.size)}</span>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>

      <style>{`
        .memory-panel {
          background: var(--bg-secondary, #1a1a2e);
          border: 1px solid var(--border, #333);
          border-radius: 8px;
          padding: 12px;
        }

        .panel-header { margin-bottom: 12px; }
        .panel-header h3 { margin: 0; font-size: 14px; }

        .mcu-select { margin-bottom: 12px; }
        .mcu-select label {
          display: block;
          color: #888;
          font-size: 11px;
          margin-bottom: 6px;
        }

        .mcu-select select {
          width: 100%;
          background: rgba(255,255,255,0.05);
          border: 1px solid #333;
          color: #fff;
          padding: 8px 12px;
          border-radius: 6px;
        }

        .analyze-btn {
          width: 100%;
          background: linear-gradient(135deg, #8b5cf6, #7c3aed);
          color: white;
          border: none;
          padding: 10px;
          border-radius: 6px;
          cursor: pointer;
          font-weight: 600;
          margin-bottom: 12px;
        }

        .results {
          background: rgba(0,0,0,0.2);
          border-radius: 6px;
          padding: 12px;
        }

        .usage-block { margin-bottom: 16px; }

        .usage-header {
          display: flex;
          justify-content: space-between;
          margin-bottom: 6px;
        }

        .usage-label {
          font-weight: 600;
          font-size: 12px;
        }

        .usage-value {
          font-size: 11px;
          color: #888;
        }

        .usage-bar {
          height: 12px;
          background: rgba(255,255,255,0.1);
          border-radius: 6px;
          overflow: hidden;
        }

        .usage-fill {
          height: 100%;
          transition: width 0.3s;
          border-radius: 6px;
        }

        .usage-percent {
          display: block;
          text-align: right;
          font-size: 10px;
          color: #888;
          margin-top: 4px;
        }

        .sections h4 {
          margin: 0 0 8px 0;
          font-size: 12px;
          color: #888;
        }

        .section-row {
          display: flex;
          justify-content: space-between;
          padding: 4px 0;
          font-size: 11px;
        }

        .section-name { color: #ccc; }
        .section-size { color: #888; font-family: monospace; }
      `}</style>
    </div>
  );
}
