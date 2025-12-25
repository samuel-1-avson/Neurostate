// Performance Monitor Panel
// Task Manager-style system metrics visualizer with real-time graphs

import { createSignal, For, Show, onMount, onCleanup, createEffect } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface SystemMetrics {
  cpu: CpuMetrics;
  memory: MemoryMetrics;
  disks: DiskMetrics[];
  network: NetworkMetrics;
  uptime: number;
  timestamp: number;
}

interface CpuMetrics {
  usage_percent: number;
  core_count: number;
  per_core_usage: number[];
  frequency_mhz: number;
  name: string;
}

interface MemoryMetrics {
  total_bytes: number;
  used_bytes: number;
  available_bytes: number;
  usage_percent: number;
  swap_total: number;
  swap_used: number;
}

interface DiskMetrics {
  name: string;
  mount_point: string;
  total_bytes: number;
  available_bytes: number;
  used_bytes: number;
  usage_percent: number;
  file_system: string;
  is_removable: boolean;
}

interface NetworkMetrics {
  interfaces: NetworkInterface[];
  total_received: number;
  total_transmitted: number;
  receive_speed_bps: number;
  transmit_speed_bps: number;
}

interface NetworkInterface {
  name: string;
  received_bytes: number;
  transmitted_bytes: number;
  receive_speed_bps: number;
  transmit_speed_bps: number;
}

interface ProcessInfo {
  pid: number;
  name: string;
  cpu_percent: number;
  memory_bytes: number;
  memory_percent: number;
  status: string;
  start_time: number;
}

interface EmbeddedMetrics {
  device_name: string;
  port: string;
  connected: boolean;
  power_mw: number;
  voltage_v: number;
  current_ma: number;
  temperature_c: number;
  clock_mhz: number;
  flash_used_kb: number;
  ram_used_kb: number;
  flash_total_kb: number;
  ram_total_kb: number;
}

interface PerformancePanelProps {
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

// Chart colors
const COLORS = {
  cpu: "#58a6ff",
  memory: "#bc8cff",
  disk: "#3fb950",
  network: "#f0883e",
  embedded_power: "#ff7b72",
  embedded_temp: "#d29922",
  grid: "#30363d",
  text: "#c9d1d9",
  bg: "#161b22",
};

// Format bytes to human readable
function formatBytes(bytes: number): string {
  if (bytes >= 1099511627776) return `${(bytes / 1099511627776).toFixed(1)} TB`;
  if (bytes >= 1073741824) return `${(bytes / 1073741824).toFixed(1)} GB`;
  if (bytes >= 1048576) return `${(bytes / 1048576).toFixed(1)} MB`;
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${bytes} B`;
}

// Format speed (bytes/sec to human readable)
function formatSpeed(bps: number): string {
  if (bps >= 1073741824) return `${(bps / 1073741824).toFixed(1)} GB/s`;
  if (bps >= 1048576) return `${(bps / 1048576).toFixed(1)} MB/s`;
  if (bps >= 1024) return `${(bps / 1024).toFixed(1)} KB/s`;
  return `${bps} B/s`;
}

// Format uptime
function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  if (days > 0) return `${days}d ${hours}h ${mins}m`;
  if (hours > 0) return `${hours}h ${mins}m`;
  return `${mins}m`;
}

// Mini sparkline chart component
function SparklineChart(props: { data: number[], color: string, height?: number, maxValue?: number }) {
  const height = props.height || 40;
  const maxVal = props.maxValue || Math.max(...props.data, 1);
  
  const points = () => {
    const data = props.data;
    if (data.length === 0) return "";
    
    const width = 100;
    const step = width / Math.max(data.length - 1, 1);
    
    return data.map((val, i) => {
      const x = i * step;
      const y = height - (val / maxVal) * height;
      return `${x},${y}`;
    }).join(" ");
  };

  return (
    <svg viewBox={`0 0 100 ${height}`} class="sparkline-chart" preserveAspectRatio="none">
      <polyline
        points={points()}
        fill="none"
        stroke={props.color}
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <polyline
        points={`0,${height} ${points()} 100,${height}`}
        fill={`${props.color}20`}
        stroke="none"
      />
    </svg>
  );
}

// Circular progress component
function CircularProgress(props: { value: number, color: string, label: string, sublabel?: string }) {
  const radius = 40;
  const circumference = 2 * Math.PI * radius;
  const offset = circumference - (props.value / 100) * circumference;

  return (
    <div class="circular-progress">
      <svg viewBox="0 0 100 100">
        <circle
          cx="50" cy="50" r={radius}
          fill="none"
          stroke={COLORS.grid}
          stroke-width="8"
        />
        <circle
          cx="50" cy="50" r={radius}
          fill="none"
          stroke={props.color}
          stroke-width="8"
          stroke-dasharray={circumference}
          stroke-dashoffset={offset}
          stroke-linecap="round"
          transform="rotate(-90 50 50)"
          class="progress-ring"
        />
        <text x="50" y="48" text-anchor="middle" fill={COLORS.text} font-size="16" font-weight="bold">
          {props.value.toFixed(0)}%
        </text>
        <text x="50" y="65" text-anchor="middle" fill={COLORS.text} font-size="8">
          {props.label}
        </text>
      </svg>
      <Show when={props.sublabel}>
        <span class="circular-sublabel">{props.sublabel}</span>
      </Show>
    </div>
  );
}

export function PerformancePanel(props: PerformancePanelProps) {
  const [metrics, setMetrics] = createSignal<SystemMetrics | null>(null);
  const [processes, setProcesses] = createSignal<ProcessInfo[]>([]);
  const [embedded, setEmbedded] = createSignal<EmbeddedMetrics | null>(null);
  const [cpuHistory, setCpuHistory] = createSignal<number[]>([]);
  const [memoryHistory, setMemoryHistory] = createSignal<number[]>([]);
  const [networkRxHistory, setNetworkRxHistory] = createSignal<number[]>([]);
  const [networkTxHistory, setNetworkTxHistory] = createSignal<number[]>([]);
  const [powerHistory, setPowerHistory] = createSignal<number[]>([]);
  const [tempHistory, setTempHistory] = createSignal<number[]>([]);
  const [activeTab, setActiveTab] = createSignal<"overview" | "processes" | "embedded">("overview");
  const [updateInterval, setUpdateInterval] = createSignal(1000);
  const [isLoading, setIsLoading] = createSignal(true);
  
  let intervalId: number | undefined;

  // Fetch system metrics
  async function fetchMetrics() {
    try {
      const result = await invoke("performance_get_system_metrics") as SystemMetrics;
      setMetrics(result);
      
      // Update history (keep last 60 points)
      setCpuHistory(prev => [...prev.slice(-59), result.cpu.usage_percent]);
      setMemoryHistory(prev => [...prev.slice(-59), result.memory.usage_percent]);
      
      // Calculate network speed from interfaces
      const totalRx = result.network.interfaces.reduce((sum, iface) => sum + iface.receive_speed_bps, 0);
      const totalTx = result.network.interfaces.reduce((sum, iface) => sum + iface.transmit_speed_bps, 0);
      setNetworkRxHistory(prev => [...prev.slice(-59), totalRx]);
      setNetworkTxHistory(prev => [...prev.slice(-59), totalTx]);
      
      setIsLoading(false);
    } catch (e) {
      console.error("Failed to fetch metrics:", e);
      props.onLog?.("Performance", `Failed to fetch metrics: ${e}`, "error");
    }
  }

  // Fetch process list
  async function fetchProcesses() {
    try {
      const result = await invoke("performance_get_process_list", { limit: 15 }) as ProcessInfo[];
      setProcesses(result);
    } catch (e) {
      console.error("Failed to fetch processes:", e);
    }
  }

  // Fetch embedded device metrics
  async function fetchEmbedded() {
    try {
      const result = await invoke("performance_get_embedded_metrics", { port: null }) as EmbeddedMetrics;
      setEmbedded(result);
      
      // Update embedded history
      setPowerHistory(prev => [...prev.slice(-59), result.power_mw]);
      setTempHistory(prev => [...prev.slice(-59), result.temperature_c]);
    } catch (e) {
      console.error("Failed to fetch embedded metrics:", e);
    }
  }

  onMount(() => {
    fetchMetrics();
    fetchProcesses();
    fetchEmbedded();
    
    // Start polling
    intervalId = setInterval(() => {
      fetchMetrics();
      if (activeTab() === "processes") fetchProcesses();
      if (activeTab() === "embedded") fetchEmbedded();
    }, updateInterval()) as unknown as number;
  });

  onCleanup(() => {
    if (intervalId) clearInterval(intervalId);
  });

  return (
    <div class="performance-panel">
      <div class="performance-header">
        <h2>⚡ Performance Monitor</h2>
        <div class="performance-actions">
          <select 
            class="interval-select"
            value={updateInterval()}
            onChange={(e) => setUpdateInterval(parseInt(e.target.value))}
          >
            <option value="500">0.5s</option>
            <option value="1000">1s</option>
            <option value="2000">2s</option>
            <option value="5000">5s</option>
          </select>
          <button class="refresh-btn" onClick={() => { fetchMetrics(); fetchProcesses(); fetchEmbedded(); }}>
            🔄 Refresh
          </button>
        </div>
      </div>

      <div class="performance-tabs">
        <button 
          class={`tab-btn ${activeTab() === "overview" ? "active" : ""}`}
          onClick={() => setActiveTab("overview")}
        >
          📊 Overview
        </button>
        <button 
          class={`tab-btn ${activeTab() === "processes" ? "active" : ""}`}
          onClick={() => { setActiveTab("processes"); fetchProcesses(); }}
        >
          📋 Processes
        </button>
        <button 
          class={`tab-btn ${activeTab() === "embedded" ? "active" : ""}`}
          onClick={() => { setActiveTab("embedded"); fetchEmbedded(); }}
        >
          🔌 Embedded
        </button>
      </div>

      <Show when={isLoading()}>
        <div class="loading-state">Loading system metrics...</div>
      </Show>

      <Show when={!isLoading() && metrics()}>
        {/* Overview Tab */}
        <Show when={activeTab() === "overview"}>
          <div class="metrics-grid">
            {/* CPU Card */}
            <div class="metric-card cpu">
              <div class="metric-header">
                <span class="metric-icon">🖥️</span>
                <span class="metric-title">CPU</span>
                <span class="metric-value">{metrics()!.cpu.usage_percent.toFixed(1)}%</span>
              </div>
              <div class="metric-chart">
                <SparklineChart data={cpuHistory()} color={COLORS.cpu} maxValue={100} />
              </div>
              <div class="metric-details">
                <span>{metrics()!.cpu.name}</span>
                <span>{metrics()!.cpu.core_count} cores @ {metrics()!.cpu.frequency_mhz} MHz</span>
              </div>
              <div class="core-usage">
                <For each={metrics()!.cpu.per_core_usage.slice(0, 8)}>
                  {(usage, i) => (
                    <div class="core-bar">
                      <div class="core-fill" style={{ height: `${usage}%`, background: COLORS.cpu }} />
                      <span class="core-label">{i()}</span>
                    </div>
                  )}
                </For>
              </div>
            </div>

            {/* Memory Card */}
            <div class="metric-card memory">
              <div class="metric-header">
                <span class="metric-icon">💾</span>
                <span class="metric-title">Memory</span>
                <span class="metric-value">{formatBytes(metrics()!.memory.used_bytes)}</span>
              </div>
              <div class="metric-chart">
                <SparklineChart data={memoryHistory()} color={COLORS.memory} maxValue={100} />
              </div>
              <div class="metric-details">
                <div class="memory-bar">
                  <div 
                    class="memory-fill" 
                    style={{ width: `${metrics()!.memory.usage_percent}%`, background: COLORS.memory }} 
                  />
                </div>
                <span>{formatBytes(metrics()!.memory.used_bytes)} / {formatBytes(metrics()!.memory.total_bytes)}</span>
                <span>Available: {formatBytes(metrics()!.memory.available_bytes)}</span>
              </div>
            </div>

            {/* Disk Card */}
            <div class="metric-card disk">
              <div class="metric-header">
                <span class="metric-icon">💿</span>
                <span class="metric-title">Storage</span>
              </div>
              <div class="disk-list">
                <For each={metrics()!.disks.slice(0, 4)}>
                  {(disk) => (
                    <div class="disk-item">
                      <div class="disk-info">
                        <span class="disk-name">{disk.mount_point}</span>
                        <span class="disk-size">{formatBytes(disk.used_bytes)} / {formatBytes(disk.total_bytes)}</span>
                      </div>
                      <div class="disk-bar">
                        <div 
                          class="disk-fill" 
                          style={{ 
                            width: `${disk.usage_percent}%`, 
                            background: disk.usage_percent > 90 ? "#f85149" : COLORS.disk 
                          }} 
                        />
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </div>

            {/* Network Card */}
            <div class="metric-card network">
              <div class="metric-header">
                <span class="metric-icon">🌐</span>
                <span class="metric-title">Network</span>
              </div>
              <div class="network-speeds">
                <div class="speed-item">
                  <span class="speed-label">↓ Download</span>
                  <span class="speed-value">{formatSpeed(networkRxHistory().at(-1) || 0)}</span>
                </div>
                <div class="speed-item">
                  <span class="speed-label">↑ Upload</span>
                  <span class="speed-value">{formatSpeed(networkTxHistory().at(-1) || 0)}</span>
                </div>
              </div>
              <div class="metric-chart">
                <SparklineChart data={networkRxHistory()} color={COLORS.network} />
              </div>
              <div class="metric-details">
                <span>Total: ↓{formatBytes(metrics()!.network.total_received)} ↑{formatBytes(metrics()!.network.total_transmitted)}</span>
              </div>
            </div>
          </div>

          {/* System Info */}
          <div class="system-info">
            <span>⏱️ Uptime: {formatUptime(metrics()!.uptime)}</span>
          </div>
        </Show>

        {/* Processes Tab */}
        <Show when={activeTab() === "processes"}>
          <div class="process-list">
            <div class="process-header">
              <span class="col-name">Name</span>
              <span class="col-pid">PID</span>
              <span class="col-cpu">CPU %</span>
              <span class="col-memory">Memory</span>
              <span class="col-status">Status</span>
            </div>
            <For each={processes()}>
              {(proc) => (
                <div class="process-row">
                  <span class="col-name">{proc.name}</span>
                  <span class="col-pid">{proc.pid}</span>
                  <span class="col-cpu" classList={{ high: proc.cpu_percent > 50 }}>
                    {proc.cpu_percent.toFixed(1)}%
                  </span>
                  <span class="col-memory">{formatBytes(proc.memory_bytes)}</span>
                  <span class="col-status">{proc.status}</span>
                </div>
              )}
            </For>
          </div>
        </Show>

        {/* Embedded Tab */}
        <Show when={activeTab() === "embedded"}>
          <Show when={embedded()} fallback={<div class="no-device">No embedded device connected</div>}>
            <div class="embedded-section">
              <div class="embedded-header">
                <span class="device-icon">🔌</span>
                <span class="device-name">{embedded()!.device_name}</span>
                <span class="device-port">{embedded()!.port}</span>
                <span class={`device-status ${embedded()!.connected ? "connected" : "disconnected"}`}>
                  {embedded()!.connected ? "● Connected" : "○ Disconnected"}
                </span>
              </div>

              <div class="embedded-metrics">
                {/* Power */}
                <div class="embedded-card">
                  <div class="embedded-card-header">
                    <span>⚡ Power</span>
                    <span class="embedded-value">{embedded()!.power_mw.toFixed(1)} mW</span>
                  </div>
                  <div class="metric-chart">
                    <SparklineChart data={powerHistory()} color={COLORS.embedded_power} />
                  </div>
                  <div class="embedded-details">
                    <span>{embedded()!.voltage_v}V @ {embedded()!.current_ma.toFixed(1)}mA</span>
                  </div>
                </div>

                {/* Temperature */}
                <div class="embedded-card">
                  <div class="embedded-card-header">
                    <span>🌡️ Temperature</span>
                    <span class="embedded-value">{embedded()!.temperature_c.toFixed(1)}°C</span>
                  </div>
                  <div class="metric-chart">
                    <SparklineChart data={tempHistory()} color={COLORS.embedded_temp} />
                  </div>
                </div>

                {/* Clock */}
                <div class="embedded-card">
                  <div class="embedded-card-header">
                    <span>⏱️ Clock</span>
                    <span class="embedded-value">{embedded()!.clock_mhz} MHz</span>
                  </div>
                  <CircularProgress value={100} color={COLORS.cpu} label="Active" />
                </div>

                {/* Memory */}
                <div class="embedded-card">
                  <div class="embedded-card-header">
                    <span>💾 Memory</span>
                  </div>
                  <div class="embedded-memory">
                    <div class="emem-item">
                      <span>Flash</span>
                      <div class="emem-bar">
                        <div 
                          class="emem-fill" 
                          style={{ 
                            width: `${(embedded()!.flash_used_kb / embedded()!.flash_total_kb) * 100}%`,
                            background: COLORS.disk
                          }} 
                        />
                      </div>
                      <span>{embedded()!.flash_used_kb} / {embedded()!.flash_total_kb} KB</span>
                    </div>
                    <div class="emem-item">
                      <span>RAM</span>
                      <div class="emem-bar">
                        <div 
                          class="emem-fill" 
                          style={{ 
                            width: `${(embedded()!.ram_used_kb / embedded()!.ram_total_kb) * 100}%`,
                            background: COLORS.memory
                          }} 
                        />
                      </div>
                      <span>{embedded()!.ram_used_kb} / {embedded()!.ram_total_kb} KB</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </Show>
        </Show>
      </Show>

      <style>{`
        .performance-panel {
          display: flex;
          flex-direction: column;
          height: 100%;
          background: ${COLORS.bg};
          color: ${COLORS.text};
          font-family: 'Segoe UI', system-ui, sans-serif;
          overflow: auto;
        }

        .performance-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 12px 16px;
          border-bottom: 1px solid ${COLORS.grid};
        }

        .performance-header h2 {
          margin: 0;
          font-size: 16px;
          font-weight: 600;
        }

        .performance-actions {
          display: flex;
          gap: 8px;
        }

        .interval-select, .refresh-btn {
          background: ${COLORS.grid};
          border: 1px solid #444;
          color: ${COLORS.text};
          padding: 4px 8px;
          border-radius: 4px;
          cursor: pointer;
          font-size: 12px;
        }

        .refresh-btn:hover {
          background: #3d444d;
        }

        .performance-tabs {
          display: flex;
          gap: 4px;
          padding: 8px 16px;
          border-bottom: 1px solid ${COLORS.grid};
        }

        .tab-btn {
          background: transparent;
          border: none;
          color: #8b949e;
          padding: 6px 12px;
          border-radius: 4px;
          cursor: pointer;
          font-size: 13px;
        }

        .tab-btn:hover {
          background: ${COLORS.grid};
        }

        .tab-btn.active {
          background: ${COLORS.grid};
          color: ${COLORS.text};
        }

        .loading-state, .no-device {
          display: flex;
          justify-content: center;
          align-items: center;
          height: 200px;
          color: #8b949e;
        }

        .metrics-grid {
          display: grid;
          grid-template-columns: repeat(2, 1fr);
          gap: 12px;
          padding: 16px;
        }

        .metric-card {
          background: #21262d;
          border: 1px solid ${COLORS.grid};
          border-radius: 8px;
          padding: 12px;
        }

        .metric-header {
          display: flex;
          align-items: center;
          gap: 8px;
          margin-bottom: 8px;
        }

        .metric-icon {
          font-size: 16px;
        }

        .metric-title {
          font-weight: 600;
          flex: 1;
        }

        .metric-value {
          font-size: 18px;
          font-weight: bold;
          color: ${COLORS.cpu};
        }

        .metric-chart {
          height: 40px;
          margin: 8px 0;
        }

        .sparkline-chart {
          width: 100%;
          height: 100%;
        }

        .metric-details {
          display: flex;
          flex-direction: column;
          gap: 4px;
          font-size: 11px;
          color: #8b949e;
        }

        .core-usage {
          display: flex;
          gap: 4px;
          margin-top: 8px;
        }

        .core-bar {
          flex: 1;
          height: 30px;
          background: ${COLORS.grid};
          border-radius: 2px;
          position: relative;
          display: flex;
          flex-direction: column;
          justify-content: flex-end;
        }

        .core-fill {
          border-radius: 2px;
          transition: height 0.3s ease;
        }

        .core-label {
          position: absolute;
          bottom: -14px;
          left: 50%;
          transform: translateX(-50%);
          font-size: 9px;
          color: #8b949e;
        }

        .memory-bar, .disk-bar, .emem-bar {
          height: 8px;
          background: ${COLORS.grid};
          border-radius: 4px;
          overflow: hidden;
        }

        .memory-fill, .disk-fill, .emem-fill {
          height: 100%;
          transition: width 0.3s ease;
        }

        .disk-list {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .disk-item {
          display: flex;
          flex-direction: column;
          gap: 4px;
        }

        .disk-info {
          display: flex;
          justify-content: space-between;
          font-size: 11px;
        }

        .disk-name {
          font-weight: 500;
        }

        .disk-size {
          color: #8b949e;
        }

        .network-speeds {
          display: flex;
          gap: 16px;
          margin-bottom: 8px;
        }

        .speed-item {
          display: flex;
          flex-direction: column;
        }

        .speed-label {
          font-size: 10px;
          color: #8b949e;
        }

        .speed-value {
          font-size: 14px;
          font-weight: 600;
          color: ${COLORS.network};
        }

        .system-info {
          padding: 8px 16px;
          font-size: 12px;
          color: #8b949e;
          border-top: 1px solid ${COLORS.grid};
        }

        /* Process List */
        .process-list {
          padding: 8px 16px;
        }

        .process-header, .process-row {
          display: grid;
          grid-template-columns: 2fr 70px 70px 90px 80px;
          gap: 8px;
          padding: 6px 8px;
          font-size: 12px;
        }

        .process-header {
          font-weight: 600;
          color: #8b949e;
          border-bottom: 1px solid ${COLORS.grid};
        }

        .process-row {
          border-radius: 4px;
        }

        .process-row:hover {
          background: ${COLORS.grid};
        }

        .col-cpu.high {
          color: #f85149;
          font-weight: 600;
        }

        /* Embedded Section */
        .embedded-section {
          padding: 16px;
        }

        .embedded-header {
          display: flex;
          align-items: center;
          gap: 12px;
          margin-bottom: 16px;
          padding: 12px;
          background: #21262d;
          border-radius: 8px;
        }

        .device-icon {
          font-size: 24px;
        }

        .device-name {
          font-weight: 600;
          font-size: 16px;
        }

        .device-port {
          color: #8b949e;
          font-size: 13px;
        }

        .device-status {
          margin-left: auto;
          font-size: 12px;
        }

        .device-status.connected {
          color: ${COLORS.disk};
        }

        .device-status.disconnected {
          color: #8b949e;
        }

        .embedded-metrics {
          display: grid;
          grid-template-columns: repeat(2, 1fr);
          gap: 12px;
        }

        .embedded-card {
          background: #21262d;
          border: 1px solid ${COLORS.grid};
          border-radius: 8px;
          padding: 12px;
        }

        .embedded-card-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 8px;
        }

        .embedded-value {
          font-size: 18px;
          font-weight: bold;
        }

        .embedded-details {
          font-size: 11px;
          color: #8b949e;
        }

        .embedded-memory {
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .emem-item {
          display: flex;
          flex-direction: column;
          gap: 4px;
        }

        .emem-item > span:first-child {
          font-size: 11px;
          font-weight: 500;
        }

        .emem-item > span:last-child {
          font-size: 10px;
          color: #8b949e;
        }

        /* Circular Progress */
        .circular-progress {
          display: flex;
          flex-direction: column;
          align-items: center;
        }

        .circular-progress svg {
          width: 80px;
          height: 80px;
        }

        .progress-ring {
          transition: stroke-dashoffset 0.3s ease;
        }
      `}</style>
    </div>
  );
}

export default PerformancePanel;
