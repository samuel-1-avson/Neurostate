import { createSignal, For, createEffect } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./MemoryMapPanel.css";

interface MemoryRegion {
  name: string;
  start: number;
  end: number;
  type: "flash" | "ram" | "peripheral" | "reserved" | "stack" | "heap";
  usage?: number; // 0-100 percentage used
}

interface MemoryMapProps {
  mcu?: string;
}

const DEFAULT_STM32_REGIONS: MemoryRegion[] = [
  { name: "Flash", start: 0x08000000, end: 0x0807FFFF, type: "flash", usage: 45 },
  { name: "SRAM1", start: 0x20000000, end: 0x2001FFFF, type: "ram", usage: 62 },
  { name: "SRAM2", start: 0x20020000, end: 0x2002FFFF, type: "ram", usage: 30 },
  { name: "Stack", start: 0x2001F000, end: 0x2001FFFF, type: "stack", usage: 25 },
  { name: "Heap", start: 0x2001A000, end: 0x2001EFFF, type: "heap", usage: 40 },
  { name: "Peripherals", start: 0x40000000, end: 0x5FFFFFFF, type: "peripheral" },
  { name: "System", start: 0xE0000000, end: 0xFFFFFFFF, type: "reserved" },
];

const TYPE_COLORS: Record<string, string> = {
  flash: "#3b82f6",
  ram: "#10b981",
  peripheral: "#f59e0b",
  reserved: "#6b7280",
  stack: "#ef4444",
  heap: "#8b5cf6",
};

export function MemoryMapPanel(props: MemoryMapProps) {
  const [regions, setRegions] = createSignal<MemoryRegion[]>(DEFAULT_STM32_REGIONS);
  const [selectedRegion, setSelectedRegion] = createSignal<MemoryRegion | null>(null);
  const [viewScale, setViewScale] = createSignal(1);
  const [flashUsed, setFlashUsed] = createSignal(45);
  const [ramUsed, setRamUsed] = createSignal(62);

  const formatAddress = (addr: number) => `0x${addr.toString(16).toUpperCase().padStart(8, "0")}`;
  const formatSize = (bytes: number) => {
    if (bytes >= 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${bytes} B`;
  };

  const totalFlash = () => regions().filter(r => r.type === "flash").reduce((sum, r) => sum + (r.end - r.start + 1), 0);
  const totalRam = () => regions().filter(r => r.type === "ram").reduce((sum, r) => sum + (r.end - r.start + 1), 0);

  return (
    <div class="memory-map-panel">
      <div class="memory-header">
        <h2>Memory Map</h2>
        <div class="memory-stats">
          <div class="stat-item">
            <span class="stat-label">Flash</span>
            <div class="stat-bar">
              <div class="stat-fill flash" style={{ width: `${flashUsed()}%` }}></div>
            </div>
            <span class="stat-value">{flashUsed()}% used</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">RAM</span>
            <div class="stat-bar">
              <div class="stat-fill ram" style={{ width: `${ramUsed()}%` }}></div>
            </div>
            <span class="stat-value">{ramUsed()}% used</span>
          </div>
        </div>
      </div>

      <div class="memory-content">
        <div class="memory-visual">
          <div class="memory-legend">
            <For each={Object.entries(TYPE_COLORS)}>
              {([type, color]) => (
                <div class="legend-item">
                  <span class="legend-color" style={{ background: color }}></span>
                  <span class="legend-label">{type.charAt(0).toUpperCase() + type.slice(1)}</span>
                </div>
              )}
            </For>
          </div>

          <div class="memory-bars">
            <For each={regions()}>
              {(region) => {
                const size = region.end - region.start + 1;
                const width = Math.max(Math.log10(size) * 15, 30);
                return (
                  <div
                    class={`memory-region ${selectedRegion() === region ? "selected" : ""}`}
                    style={{
                      background: TYPE_COLORS[region.type],
                      width: `${width}px`,
                    }}
                    onClick={() => setSelectedRegion(region)}
                    title={`${region.name}: ${formatAddress(region.start)} - ${formatAddress(region.end)}`}
                  >
                    <span class="region-name">{region.name}</span>
                    {region.usage !== undefined && (
                      <div class="usage-overlay" style={{ height: `${100 - region.usage}%` }}></div>
                    )}
                  </div>
                );
              }}
            </For>
          </div>
        </div>

        <div class="memory-details">
          {selectedRegion() ? (
            <div class="region-info">
              <h3>{selectedRegion()!.name}</h3>
              <div class="info-grid">
                <div class="info-row">
                  <span class="info-label">Start</span>
                  <span class="info-value">{formatAddress(selectedRegion()!.start)}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">End</span>
                  <span class="info-value">{formatAddress(selectedRegion()!.end)}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">Size</span>
                  <span class="info-value">{formatSize(selectedRegion()!.end - selectedRegion()!.start + 1)}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">Type</span>
                  <span class="info-value type" style={{ color: TYPE_COLORS[selectedRegion()!.type] }}>
                    {selectedRegion()!.type.toUpperCase()}
                  </span>
                </div>
                {selectedRegion()!.usage !== undefined && (
                  <div class="info-row">
                    <span class="info-label">Usage</span>
                    <span class="info-value">{selectedRegion()!.usage}%</span>
                  </div>
                )}
              </div>
            </div>
          ) : (
            <div class="no-selection">
              <p>Click a memory region to view details</p>
            </div>
          )}
        </div>
      </div>

      <div class="memory-table">
        <table>
          <thead>
            <tr>
              <th>Region</th>
              <th>Start</th>
              <th>End</th>
              <th>Size</th>
              <th>Type</th>
            </tr>
          </thead>
          <tbody>
            <For each={regions()}>
              {(region) => (
                <tr class={selectedRegion() === region ? "selected" : ""} onClick={() => setSelectedRegion(region)}>
                  <td>{region.name}</td>
                  <td class="mono">{formatAddress(region.start)}</td>
                  <td class="mono">{formatAddress(region.end)}</td>
                  <td class="mono">{formatSize(region.end - region.start + 1)}</td>
                  <td>
                    <span class="type-badge" style={{ background: TYPE_COLORS[region.type] }}>
                      {region.type}
                    </span>
                  </td>
                </tr>
              )}
            </For>
          </tbody>
        </table>
      </div>
    </div>
  );
}

export default MemoryMapPanel;
