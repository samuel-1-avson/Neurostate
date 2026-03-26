// StatusBar Component - Bottom status information bar
import { createSignal, onMount, onCleanup } from "solid-js";

interface StatusBarProps {
  projectName?: string;
  targetMcu?: string;
  connectionStatus?: "connected" | "disconnected" | "connecting";
  buildStatus?: "idle" | "building" | "success" | "error";
  nodeCount?: number;
  edgeCount?: number;
  zoom?: number;
  cursorPosition?: { x: number; y: number };
}

export function StatusBar(props: StatusBarProps) {
  const [memoryUsage, setMemoryUsage] = createSignal<number>(0);
  const [cpuUsage, setCpuUsage] = createSignal<number>(0);
  const [currentTime, setCurrentTime] = createSignal<string>("");

  onMount(() => {
    const updateTime = () => {
      const now = new Date();
      setCurrentTime(now.toLocaleTimeString('en-US', { hour12: false }));
    };
    updateTime();
    const interval = setInterval(updateTime, 1000);
    onCleanup(() => clearInterval(interval));

    // Simulate memory/CPU for demo
    const perfInterval = setInterval(() => {
      setMemoryUsage(Math.floor(Math.random() * 30 + 50));
      setCpuUsage(Math.floor(Math.random() * 20 + 5));
    }, 3000);
    onCleanup(() => clearInterval(perfInterval));
  });

  const getConnectionIcon = () => {
    switch (props.connectionStatus) {
      case "connected": return "🟢";
      case "connecting": return "🟡";
      default: return "🔴";
    }
  };

  const getBuildIcon = () => {
    switch (props.buildStatus) {
      case "building": return "🔄";
      case "success": return "✅";
      case "error": return "❌";
      default: return "⚪";
    }
  };

  return (
    <div class="statusbar">
      {/* Left section - Project info */}
      <div class="statusbar-section statusbar-left">
        <span class="status-item" title="Project">
          📁 {props.projectName || "Untitled Project"}
        </span>
        <span class="status-divider" />
        <span class="status-item" title="Target MCU">
          🎯 {props.targetMcu || "STM32F401"}
        </span>
        <span class="status-divider" />
        <span class="status-item" title="Connection Status">
          {getConnectionIcon()} {props.connectionStatus || "Disconnected"}
        </span>
      </div>

      {/* Center section - Build status */}
      <div class="statusbar-section statusbar-center">
        <span class="status-item" title="Build Status">
          {getBuildIcon()} {props.buildStatus === "building" ? "Building..." : 
                           props.buildStatus === "success" ? "Build Successful" :
                           props.buildStatus === "error" ? "Build Failed" : "Ready"}
        </span>
      </div>

      {/* Right section - Canvas info */}
      <div class="statusbar-section statusbar-right">
        <span class="status-item" title="Graph Statistics">
          🔷 {props.nodeCount || 0} nodes • 🔗 {props.edgeCount || 0} edges
        </span>
        <span class="status-divider" />
        <span class="status-item" title="Zoom Level">
          🔍 {Math.round((props.zoom || 1) * 100)}%
        </span>
        <span class="status-divider" />
        <span class="status-item" title="Cursor Position">
          📍 {props.cursorPosition?.x || 0}, {props.cursorPosition?.y || 0}
        </span>
        <span class="status-divider" />
        <span class="status-item perf" title="Performance">
          💾 {memoryUsage()}MB • ⚡ {cpuUsage()}%
        </span>
        <span class="status-divider" />
        <span class="status-item" title="Time">
          🕐 {currentTime()}
        </span>
      </div>
    </div>
  );
}

export default StatusBar;
