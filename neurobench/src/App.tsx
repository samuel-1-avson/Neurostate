import { createSignal, For, Show, onMount, createEffect } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { PinDiagram } from "./components/PinDiagram";
import "./components/PinDiagram.css";
import { Terminal } from "./components/Terminal";
import "./components/Terminal.css";
import { AgentPanel } from "./components/AgentPanel";
import "./components/AgentPanel.css";
import { TimersPanel } from "./components/TimersPanel";
import "./components/TimersPanel.css";
import { PeripheralsPanel } from "./components/PeripheralsPanel";
import "./components/PeripheralsPanel.css";
import { ClockPanel } from "./components/ClockPanel";
import "./components/ClockPanel.css";
import { AnalogPanel } from "./components/AnalogPanel";
import "./components/AnalogPanel.css";
import { McuSelector } from "./components/McuSelector";
import "./components/McuSelector.css";
import { RTOSPanel } from "./components/RTOSPanel";
import "./components/RTOSPanel.css";
import { WirelessPanel } from "./components/WirelessPanel";
import "./components/WirelessPanel.css";
import { DSPPanel } from "./components/DSPPanel";
import "./components/DSPPanel.css";
import { SecurityPanel } from "./components/SecurityPanel";
import "./components/SecurityPanel.css";
import { SettingsPanel } from "./components/SettingsPanel";
import UnifiedCanvas from "./components/UnifiedCanvas";

// NEW: Import additional panels that exist but were not accessible
import { SimulatorPanel } from "./components/SimulatorPanel";
import DebugPanel from "./components/DebugPanel";
import { PerformancePanel } from "./components/PerformancePanel";
import { BuildPanel } from "./components/BuildPanel";
import { GitPanel } from "./components/GitPanel";
import { ValidationPanel } from "./components/ValidationPanel";
import { PowerPanel } from "./components/PowerPanel";
import { SerialPanel } from "./components/SerialPanel";
import { MemoryPanel } from "./components/MemoryPanel";
import { ProfilerPanel } from "./components/ProfilerPanel";
import WorkflowPanel from "./components/WorkflowPanel";
import { HistoryPanel } from "./components/HistoryPanel";
import "./components/HistoryPanel.css";
import { SchedulerPanel } from "./components/SchedulerPanel";
import "./components/SchedulerPanel.css";

// --- Icons (inline SVG for simplicity) ---
const Icons = {
  brain: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M12 2a4 4 0 0 0-4 4c0 1.1.9 2 2 2h.5" />
      <path d="M8 6a4 4 0 0 0-4 4c0 2.2 1.8 4 4 4h1" />
      <path d="M12 22a4 4 0 0 0 4-4c0-1.1-.9-2-2-2h-.5" />
      <path d="M16 18a4 4 0 0 0 4-4c0-2.2-1.8-4-4-4h-1" />
      <path d="M12 2a4 4 0 0 1 4 4c0 1.1-.9 2-2 2h-.5" />
      <path d="M16 6a4 4 0 0 1 4 4c0 2.2-1.8 4-4 4h-1" />
      <path d="M12 22a4 4 0 0 1-4-4c0-1.1.9-2 2-2h.5" />
      <path d="M8 18a4 4 0 0 1-4-4c0-2.2 1.8-4 4-4h1" />
      <circle cx="12" cy="12" r="2" />
    </svg>
  ),
  newFile: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
      <polyline points="14,2 14,8 20,8" />
      <line x1="12" y1="18" x2="12" y2="12" />
      <line x1="9" y1="15" x2="15" y2="15" />
    </svg>
  ),
  save: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
      <polyline points="17 21 17 13 7 13 7 21" />
      <polyline points="7 3 7 8 15 8" />
    </svg>
  ),
  folder: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
    </svg>
  ),
  play: () => (
    <svg viewBox="0 0 24 24" fill="currentColor">
      <polygon points="5 3 19 12 5 21 5 3" />
    </svg>
  ),
  pause: () => (
    <svg viewBox="0 0 24 24" fill="currentColor">
      <rect x="6" y="4" width="4" height="16" />
      <rect x="14" y="4" width="4" height="16" />
    </svg>
  ),
  step: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <polygon points="5 4 15 12 5 20 5 4" fill="currentColor" />
      <line x1="19" y1="5" x2="19" y2="19" />
    </svg>
  ),
  stop: () => (
    <svg viewBox="0 0 24 24" fill="currentColor">
      <rect x="4" y="4" width="16" height="16" rx="2" />
    </svg>
  ),
  code: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <polyline points="16 18 22 12 16 6" />
      <polyline points="8 6 2 12 8 18" />
    </svg>
  ),
  chip: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="4" y="4" width="16" height="16" rx="2" />
      <rect x="9" y="9" width="6" height="6" />
      <line x1="9" y1="1" x2="9" y2="4" />
      <line x1="15" y1="1" x2="15" y2="4" />
      <line x1="9" y1="20" x2="9" y2="23" />
      <line x1="15" y1="20" x2="15" y2="23" />
      <line x1="20" y1="9" x2="23" y2="9" />
      <line x1="20" y1="14" x2="23" y2="14" />
      <line x1="1" y1="9" x2="4" y2="9" />
      <line x1="1" y1="14" x2="4" y2="14" />
    </svg>
  ),
  layers: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <polygon points="12 2 2 7 12 12 22 7 12 2" />
      <polyline points="2 17 12 22 22 17" />
      <polyline points="2 12 12 17 22 12" />
    </svg>
  ),
  settings: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </svg>
  ),
  x: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <line x1="18" y1="6" x2="6" y2="18" />
      <line x1="6" y1="6" x2="18" y2="18" />
    </svg>
  ),
  message: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
    </svg>
  ),
  send: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <line x1="22" y1="2" x2="11" y2="13" />
      <polygon points="22 2 15 22 11 13 2 9 22 2" />
    </svg>
  ),
  zoomIn: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="11" cy="11" r="8" />
      <line x1="21" y1="21" x2="16.65" y2="16.65" />
      <line x1="11" y1="8" x2="11" y2="14" />
      <line x1="8" y1="11" x2="14" y2="11" />
    </svg>
  ),
  zoomOut: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="11" cy="11" r="8" />
      <line x1="21" y1="21" x2="16.65" y2="16.65" />
      <line x1="8" y1="11" x2="14" y2="11" />
    </svg>
  ),
  fitView: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3" />
    </svg>
  ),
  plug: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M12 2v10" />
      <path d="M18.4 6.6a9 9 0 1 1-12.8 0" />
      <circle cx="12" cy="12" r="2" />
    </svg>
  ),
  cpu: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="4" y="4" width="16" height="16" rx="2" />
      <rect x="9" y="9" width="6" height="6" />
      <path d="M9 1v3M15 1v3M9 20v3M15 20v3M20 9h3M20 14h3M1 9h3M1 14h3" />
    </svg>
  ),
  tasks: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M9 11l3 3L22 4" />
      <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" />
    </svg>
  ),
  wifi: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M5 12.55a11 11 0 0 1 14.08 0" />
      <path d="M1.42 9a16 16 0 0 1 21.16 0" />
      <path d="M8.53 16.11a6 6 0 0 1 6.95 0" />
      <circle cx="12" cy="20" r="1" fill="currentColor" />
    </svg>
  ),
  activity: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
    </svg>
  ),
  lock: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
      <path d="M7 11V7a5 5 0 0 1 10 0v4" />
    </svg>
  ),
  grid: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="3" y="3" width="7" height="7" />
      <rect x="14" y="3" width="7" height="7" />
      <rect x="14" y="14" width="7" height="7" />
      <rect x="3" y="14" width="7" height="7" />
    </svg>
  ),
  minimap: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="3" y="3" width="18" height="18" rx="2" />
      <rect x="14" y="14" width="5" height="5" fill="currentColor" opacity="0.5" />
      <circle cx="8" cy="8" r="2" fill="currentColor" />
      <circle cx="13" cy="10" r="1.5" fill="currentColor" />
    </svg>
  ),
  layout: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="3" y="3" width="6" height="5" rx="1" />
      <rect x="15" y="3" width="6" height="5" rx="1" />
      <rect x="9" y="16" width="6" height="5" rx="1" />
      <path d="M6 8v3a3 3 0 0 0 3 3h6a3 3 0 0 0 3-3V8" />
      <path d="M12 14v2" />
    </svg>
  ),
  validate: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M9 12l2 2 4-4" />
      <circle cx="12" cy="12" r="10" />
    </svg>
  ),
  // NEW ICONS for additional panels
  debug: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M12 2a3 3 0 0 0-3 3v2a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
      <path d="M18 12h-3M9 12H6M18 8h-3M9 8H6M18 16h-3M9 16H6" />
      <rect x="9" y="9" width="6" height="13" rx="2" />
    </svg>
  ),
  performance: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" />
      <circle cx="12" cy="12" r="3" />
    </svg>
  ),
  build: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z" />
    </svg>
  ),
  gitBranch: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <line x1="6" y1="3" x2="6" y2="15" />
      <circle cx="18" cy="6" r="3" />
      <circle cx="6" cy="18" r="3" />
      <path d="M18 9a9 9 0 0 1-9 9" />
    </svg>
  ),
  power: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M18.36 6.64a9 9 0 1 1-12.73 0" />
      <line x1="12" y1="2" x2="12" y2="12" />
    </svg>
  ),
  serial: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="4" y="4" width="16" height="6" rx="1" />
      <rect x="4" y="14" width="16" height="6" rx="1" />
      <path d="M8 10v4M12 10v4M16 10v4" />
    </svg>
  ),
  memory: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="2" y="6" width="20" height="12" rx="2" />
      <path d="M6 6V4M10 6V4M14 6V4M18 6V4M6 18v2M10 18v2M14 18v2M18 18v2" />
      <line x1="6" y1="10" x2="6" y2="14" />
      <line x1="18" y1="10" x2="18" y2="14" />
    </svg>
  ),
  profiler: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <line x1="18" y1="20" x2="18" y2="10" />
      <line x1="12" y1="20" x2="12" y2="4" />
      <line x1="6" y1="20" x2="6" y2="14" />
    </svg>
  ),
  workflow: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="3" y="4" width="6" height="6" rx="1" />
      <rect x="15" y="4" width="6" height="6" rx="1" />
      <rect x="9" y="14" width="6" height="6" rx="1" />
      <path d="M6 10v2a2 2 0 0 0 2 2h2M18 10v2a2 2 0 0 1-2 2h-2" />
    </svg>
  ),
  simulator: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="2" y="3" width="20" height="14" rx="2" />
      <line x1="8" y1="21" x2="16" y2="21" />
      <line x1="12" y1="17" x2="12" y2="21" />
    </svg>
  ),
};

// --- Types ---
interface FSMNode {
  id: string;
  label: string;
  type: "input" | "process" | "output" | "decision" | "error" | "hardware" | "delay" | "interrupt";
  x: number;
  y: number;
  entryAction?: string;
}

interface FSMEdge {
  id: string;
  source: string;
  target: string;
  label?: string;
}

interface LogEntry {
  time: string;
  source: string;
  message: string;
  type: "info" | "success" | "warning" | "error";
}

interface ChatMessage {
  role: "user" | "assistant";
  content: string;
}

// Context menu state
interface ContextMenuState {
  visible: boolean;
  x: number;
  y: number;
  type: "canvas" | "node" | "edge";
  targetId?: string;
}

// Selection box for marquee selection
interface SelectionBox {
  startX: number;
  startY: number;
  endX: number;
  endY: number;
}

// Connection state for edge creation
interface ConnectionState {
  sourceId: string;
  sourcePort: "input" | "output";
  mouseX: number;
  mouseY: number;
}

// Node dimensions for edge calculations
const NODE_WIDTH = 140;
const NODE_HEIGHT = 50;
const GRID_SIZE = 24;
const SNAP_THRESHOLD = 12;

// --- App Component ---
function App() {
  // State
  const [projectName, setProjectName] = createSignal("Untitled Project");
  const [targetMcu, setTargetMcu] = createSignal("STM32F401");
  const [simStatus, setSimStatus] = createSignal<"idle" | "running" | "paused">("idle");
  
  const [nodes, setNodes] = createSignal<FSMNode[]>([
    { id: "1", label: "START", type: "input", x: 300, y: 80 },
    { id: "2", label: "INIT", type: "process", x: 300, y: 200, entryAction: "HAL.init();\nGPIO.setup(13, OUTPUT);" },
    { id: "3", label: "RUNNING", type: "process", x: 300, y: 320, entryAction: "ledOn = true;" },
    { id: "4", label: "END", type: "output", x: 300, y: 440 },
  ]);
  
  const [edges, setEdges] = createSignal<FSMEdge[]>([
    { id: "e1", source: "1", target: "2", label: "init" },
    { id: "e2", source: "2", target: "3", label: "ready" },
    { id: "e3", source: "3", target: "4", label: "done" },
  ]);
  
  const [selectedNode, setSelectedNode] = createSignal<string | null>(null);
  const [logs, setLogs] = createSignal<LogEntry[]>([
    { time: "00:00:00", source: "SYSTEM", message: "NeuroBench initialized", type: "info" },
    { time: "00:00:01", source: "MCU", message: "Target: STM32F401 BlackPill", type: "success" },
  ]);
  
  // Canvas state
  const [zoom, setZoom] = createSignal(1);
  const [panX, setPanX] = createSignal(0);
  const [panY, setPanY] = createSignal(0);
  const [isPanning, setIsPanning] = createSignal(false);
  const [panStart, setPanStart] = createSignal({ x: 0, y: 0 });
  
  // Drag state
  const [draggingNode, setDraggingNode] = createSignal<string | null>(null);
  const [dragOffset, setDragOffset] = createSignal({ x: 0, y: 0 });
  
  // Multi-selection state
  const [selectedNodes, setSelectedNodes] = createSignal<Set<string>>(new Set());
  const [isMarqueeSelecting, setIsMarqueeSelecting] = createSignal(false);
  const [selectionBox, setSelectionBox] = createSignal<SelectionBox | null>(null);
  
  // Context menu state
  const [contextMenu, setContextMenu] = createSignal<ContextMenuState>({ 
    visible: false, x: 0, y: 0, type: "canvas" 
  });
  
  // Edge creation state (drag from port to port)
  const [connecting, setConnecting] = createSignal<ConnectionState | null>(null);
  
  // Canvas options
  const [snapToGrid, setSnapToGrid] = createSignal(true);
  const [showMinimap, setShowMinimap] = createSignal(true);
  const [useEnhancedCanvas, setUseEnhancedCanvas] = createSignal(false); // Toggle for new FSMCanvas with node palette
  
  // Panel state
  const [activePanel, setActivePanel] = createSignal("nodes");
  const [activeBottomTab, setActiveBottomTab] = createSignal("console");
  
  // AI Chat state
  const [chatMessages, setChatMessages] = createSignal<ChatMessage[]>([]);
  const [chatInput, setChatInput] = createSignal("");
  const [isAiLoading, setIsAiLoading] = createSignal(false);
  
  // Hardware state
  const [serialPorts, setSerialPorts] = createSignal<any[]>([]);
  const [mcuList, setMcuList] = createSignal<any[]>([]);
  
  // Code generation state
  const [generatedCode, setGeneratedCode] = createSignal<string>("");
  const [codeLanguage, setCodeLanguage] = createSignal<string>("C");
  const [isGenerating, setIsGenerating] = createSignal(false);
  
  // FSM from description state
  const [showDescriptionModal, setShowDescriptionModal] = createSignal(false);
  const [fsmDescription, setFsmDescription] = createSignal("");
  const [isParsingFsm, setIsParsingFsm] = createSignal(false);
  
  // Settings modal state
  const [showSettingsModal, setShowSettingsModal] = createSignal(false);
  
  // Enhanced Editor modal state
  const [showEnhancedEditor, setShowEnhancedEditor] = createSignal(false);
  
  // Unified Canvas mode (default true - shows new unified canvas)
  const [useUnifiedCanvas, setUseUnifiedCanvas] = createSignal(true);
  
  // Driver generation state
  const [driverType, setDriverType] = createSignal<"GPIO" | "UART" | "SPI" | "I2C" | "CAN" | "Modbus">("GPIO");
  const [gpioPort, setGpioPort] = createSignal("A");
  const [gpioPin, setGpioPin] = createSignal(13);
  const [gpioMode, setGpioMode] = createSignal("output");
  const [uartInstance, setUartInstance] = createSignal("USART1");
  const [uartBaud, setUartBaud] = createSignal(115200);
  const [uartDma, setUartDma] = createSignal(false);
  const [spiInstance, setSpiInstance] = createSignal("SPI1");
  const [spiClock, setSpiClock] = createSignal(1000000);
  const [spiMode, setSpiMode] = createSignal(0);
  const [i2cInstance, setI2cInstance] = createSignal("I2C1");
  const [i2cSpeed, setI2cSpeed] = createSignal("fast");
  const [canInstance, setCanInstance] = createSignal("CAN1");
  const [canBitrate, setCanBitrate] = createSignal(500000);
  const [modbusAddress, setModbusAddress] = createSignal(1);
  const [modbusMode, setModbusMode] = createSignal("master");
  const [driverLanguage, setDriverLanguage] = createSignal("C");
  const [generatedDriver, setGeneratedDriver] = createSignal<any>(null);
  const [isGeneratingDriver, setIsGeneratingDriver] = createSignal(false);

  // Undo/Redo history
  interface HistoryState {
    nodes: FSMNode[];
    edges: FSMEdge[];
  }
  const [history, setHistory] = createSignal<HistoryState[]>([]);
  const [historyIndex, setHistoryIndex] = createSignal(-1);
  const [isUndoing, setIsUndoing] = createSignal(false);

  // Save current state to history
  function pushHistory() {
    if (isUndoing()) return;
    const currentState = { nodes: [...nodes()], edges: [...edges()] };
    const newHistory = history().slice(0, historyIndex() + 1);
    newHistory.push(currentState);
    // Keep max 50 history items
    if (newHistory.length > 50) newHistory.shift();
    setHistory(newHistory);
    setHistoryIndex(newHistory.length - 1);
  }

  function undo() {
    if (historyIndex() <= 0) return;
    setIsUndoing(true);
    const newIndex = historyIndex() - 1;
    const state = history()[newIndex];
    setNodes([...state.nodes]);
    setEdges([...state.edges]);
    setHistoryIndex(newIndex);
    setIsUndoing(false);
    addLog("SYSTEM", "Undo", "info");
  }

  function redo() {
    if (historyIndex() >= history().length - 1) return;
    setIsUndoing(true);
    const newIndex = historyIndex() + 1;
    const state = history()[newIndex];
    setNodes([...state.nodes]);
    setEdges([...state.edges]);
    setHistoryIndex(newIndex);
    setIsUndoing(false);
    addLog("SYSTEM", "Redo", "info");
  }

  // Sync FSM context to agents when nodes/edges change
  createEffect(() => {
    const currentNodes = nodes();
    const currentEdges = edges();
    const selected = selectedNode();
    
    // Convert to context format
    const contextNodes = currentNodes.map(n => ({
      id: n.id,
      label: n.label,
      node_type: n.type,
      x: n.x,
      y: n.y,
      entry_action: n.entryAction || null,
    }));
    
    const contextEdges = currentEdges.map(e => ({
      id: e.id,
      source: e.source,
      target: e.target,
      label: e.label || null,
    }));
    
    // Send to backend (fire and forget)
    invoke("update_fsm_context", { 
      nodes: contextNodes, 
      edges: contextEdges,
      selectedNode: selected 
    }).catch(e => console.warn("Failed to sync FSM context:", e));
  });

  // Save project to file
  async function saveProject() {
    const projectData = {
      name: projectName(),
      nodes: nodes().map(n => ({ ...n })),
      edges: edges().map(e => ({ ...e })),
      mcu: "STM32F401",
      language: codeLanguage(),
    };
    const path = `${projectName().replace(/\s+/g, '_')}.neurobench.json`;
    try {
      await invoke("save_project_file", { path, project: projectData });
      addLog("PROJECT", `Saved: ${path}`, "success");
    } catch (e) {
      addLog("ERROR", `Save failed: ${e}`, "error");
    }
  }

  // Load project from file (would need file dialog in real app)
  async function loadProject(path: string) {
    try {
      const project = await invoke("load_project_file", { path }) as any;
      setProjectName(project.name);
      setNodes(project.nodes);
      setEdges(project.edges);
      setCodeLanguage(project.language);
      addLog("PROJECT", `Loaded: ${path}`, "success");
    } catch (e) {
      addLog("ERROR", `Load failed: ${e}`, "error");
    }
  }

  // Keyboard shortcuts
  function handleKeyDown(e: KeyboardEvent) {
    // Don't handle shortcuts when typing in inputs
    const target = e.target as HTMLElement;
    if (target.tagName === "INPUT" || target.tagName === "TEXTAREA") return;
    
    if (e.ctrlKey || e.metaKey) {
      // Ctrl+Z - Undo
      if (e.key === "z" && !e.shiftKey) {
        e.preventDefault();
        undo();
      }
      // Ctrl+Shift+Z or Ctrl+Y - Redo
      else if ((e.key === "z" && e.shiftKey) || e.key === "y") {
        e.preventDefault();
        redo();
      }
      // Ctrl+S - Save
      else if (e.key === "s") {
        e.preventDefault();
        saveProject();
      }
      // Ctrl+A - Select All Nodes
      else if (e.key === "a") {
        e.preventDefault();
        const allIds = new Set<string>(nodes().map(n => n.id));
        setSelectedNodes(allIds);
        addLog("CANVAS", `Selected all ${nodes().length} nodes`, "info");
      }
      // Ctrl+D - Duplicate selected node
      else if (e.key === "d" && selectedNode()) {
        e.preventDefault();
        const node = getNodeById(selectedNode()!);
        if (node) {
          pushHistory();
          const newNode: FSMNode = {
            ...node,
            id: `node_${Date.now()}`,
            x: node.x + 50,
            y: node.y + 50,
          };
          setNodes([...nodes(), newNode]);
          setSelectedNode(newNode.id);
          addLog("CANVAS", `Duplicated node: ${node.label}`, "success");
        }
      }
    }
    
    // Escape - Deselect all
    if (e.key === "Escape") {
      setSelectedNode(null);
      setSelectedNodes(new Set<string>());
      setConnecting(null);
      setContextMenu({ visible: false, x: 0, y: 0, type: "canvas" });
    }
    
    // Delete - Delete selected node(s)
    if (e.key === "Delete") {
      const multiSelected = selectedNodes();
      
      // Delete multiple selected nodes
      if (multiSelected.size > 0) {
        pushHistory();
        setNodes(nodes().filter(n => !multiSelected.has(n.id)));
        setEdges(edges().filter(e => !multiSelected.has(e.source) && !multiSelected.has(e.target)));
        setSelectedNodes(new Set<string>());
        setSelectedNode(null);
        addLog("CANVAS", `Deleted ${multiSelected.size} nodes`, "info");
      }
      // Delete single selected node
      else if (selectedNode()) {
        const nodeId = selectedNode()!;
        const node = getNodeById(nodeId);
        pushHistory();
        setNodes(nodes().filter(n => n.id !== nodeId));
        setEdges(edges().filter(e => e.source !== nodeId && e.target !== nodeId));
        setSelectedNode(null);
        addLog("CANVAS", `Deleted node: ${node?.label || nodeId}`, "info");
      }
    }
  }


  // Load system info on mount
  onMount(async () => {
    // Register keyboard shortcuts
    window.addEventListener("keydown", handleKeyDown);
    // Initialize history with current state
    pushHistory();
    
    try {
      const info = await invoke("get_system_info");
      addLog("SYSTEM", `Backend: ${(info as any).name} v${(info as any).version}`, "success");
      if ((info as any).ai_available) {
        addLog("AI", "Gemini AI ready", "success");
      } else {
        addLog("AI", "Set GEMINI_API_KEY for AI features", "warning");
      }
    } catch (e) {
      addLog("ERROR", `Failed to connect to backend: ${e}`, "error");
    }
  });

  // Helper functions
  const addLog = (source: string, message: string, type: LogEntry["type"] = "info") => {
    const now = new Date();
    const time = now.toTimeString().split(" ")[0];
    setLogs(prev => [...prev.slice(-99), { time, source, message, type }]);
  };

  const getNodeById = (id: string) => nodes().find(n => n.id === id);
  
  // Snap position to grid
  const snapPosition = (x: number, y: number): { x: number; y: number } => {
    if (!snapToGrid()) return { x, y };
    return {
      x: Math.round(x / GRID_SIZE) * GRID_SIZE,
      y: Math.round(y / GRID_SIZE) * GRID_SIZE,
    };
  };
  
  // Get connection preview line path
  const getConnectionPreviewPath = () => {
    const conn = connecting();
    if (!conn) return "";
    const sourceNode = getNodeById(conn.sourceId);
    if (!sourceNode) return "";
    
    const sx = sourceNode.x + NODE_WIDTH / 2;
    const sy = conn.sourcePort === "output" ? sourceNode.y + NODE_HEIGHT : sourceNode.y;
    const tx = (conn.mouseX - panX()) / zoom();
    const ty = (conn.mouseY - panY()) / zoom();
    
    const midY = (sy + ty) / 2;
    return `M ${sx} ${sy} C ${sx} ${midY}, ${tx} ${midY}, ${tx} ${ty}`;
  };
  
  // Check if a node is within marquee selection box
  const isNodeInSelectionBox = (node: FSMNode, box: SelectionBox) => {
    const boxLeft = Math.min(box.startX, box.endX);
    const boxRight = Math.max(box.startX, box.endX);
    const boxTop = Math.min(box.startY, box.endY);
    const boxBottom = Math.max(box.startY, box.endY);
    
    return (
      node.x < boxRight &&
      node.x + NODE_WIDTH > boxLeft &&
      node.y < boxBottom &&
      node.y + NODE_HEIGHT > boxTop
    );
  };
  
  // Get edge path between two nodes
  const getEdgePath = (edge: FSMEdge) => {
    const source = getNodeById(edge.source);
    const target = getNodeById(edge.target);
    if (!source || !target) return "";
    
    const sx = source.x + NODE_WIDTH / 2;
    const sy = source.y + NODE_HEIGHT;
    const tx = target.x + NODE_WIDTH / 2;
    const ty = target.y;
    
    // Bezier curve control points
    const midY = (sy + ty) / 2;
    return `M ${sx} ${sy} C ${sx} ${midY}, ${tx} ${midY}, ${tx} ${ty}`;
  };
  
  // Get edge label position
  const getEdgeLabelPos = (edge: FSMEdge) => {
    const source = getNodeById(edge.source);
    const target = getNodeById(edge.target);
    if (!source || !target) return { x: 0, y: 0 };
    
    return {
      x: (source.x + target.x) / 2 + NODE_WIDTH / 2,
      y: (source.y + NODE_HEIGHT + target.y) / 2,
    };
  };

  // Canvas event handlers
  const handleCanvasWheel = (e: WheelEvent) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    const newZoom = Math.min(3, Math.max(0.25, zoom() * delta));
    setZoom(newZoom);
  };
  
  const handleCanvasMouseDown = (e: MouseEvent) => {
    // Close context menu on any click
    if (contextMenu().visible) {
      setContextMenu({ visible: false, x: 0, y: 0, type: "canvas" });
    }
    
    if (e.button === 1 || (e.button === 0 && e.shiftKey)) {
      // Middle click or Shift+Left click for panning
      setIsPanning(true);
      setPanStart({ x: e.clientX - panX(), y: e.clientY - panY() });
      e.preventDefault();
    } else if (e.button === 0 && !e.ctrlKey) {
      // Left click - start marquee selection if not on a node
      const target = e.target as HTMLElement;
      if (target.classList.contains('canvas-grid') || target.classList.contains('fsm-canvas')) {
        const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
        const startX = (e.clientX - rect.left - panX()) / zoom();
        const startY = (e.clientY - rect.top - panY()) / zoom();
        setIsMarqueeSelecting(true);
        setSelectionBox({ startX, startY, endX: startX, endY: startY });
        // Clear selection if not holding Ctrl
        setSelectedNode(null);
        setSelectedNodes(new Set<string>());
      }
    }
  };
  
  const handleCanvasMouseMove = (e: MouseEvent) => {
    if (isPanning()) {
      setPanX(e.clientX - panStart().x);
      setPanY(e.clientY - panStart().y);
    } else if (isMarqueeSelecting()) {
      // Update marquee selection box
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      const endX = (e.clientX - rect.left - panX()) / zoom();
      const endY = (e.clientY - rect.top - panY()) / zoom();
      setSelectionBox(prev => prev ? { ...prev, endX, endY } : null);
      
      // Update selected nodes within marquee
      const box = selectionBox();
      if (box) {
        const newSelection = new Set<string>();
        nodes().forEach(node => {
          if (isNodeInSelectionBox(node, { ...box, endX, endY })) {
            newSelection.add(node.id);
          }
        });
        setSelectedNodes(newSelection);
      }
    } else if (connecting()) {
      // Update connection preview line
      setConnecting(prev => prev ? { ...prev, mouseX: e.clientX, mouseY: e.clientY } : null);
    } else if (draggingNode()) {
      const nodeId = draggingNode()!;
      const z = zoom();
      const rawX = (e.clientX - dragOffset().x) / z - panX() / z;
      const rawY = (e.clientY - dragOffset().y) / z - panY() / z;
      const snapped = snapPosition(rawX, rawY);
      
      setNodes(prev => prev.map(n => {
        if (n.id === nodeId) {
          return { ...n, x: snapped.x, y: snapped.y };
        }
        return n;
      }));
    }
  };
  
  const handleCanvasMouseUp = (e: MouseEvent) => {
    // Finalize marquee selection
    if (isMarqueeSelecting()) {
      const box = selectionBox();
      if (box) {
        const finalSelection = new Set<string>();
        nodes().forEach(node => {
          if (isNodeInSelectionBox(node, box)) {
            finalSelection.add(node.id);
          }
        });
        setSelectedNodes(finalSelection);
        if (finalSelection.size === 1) {
          setSelectedNode([...finalSelection][0]);
        }
      }
    }
    
    // Finalize edge creation
    if (connecting()) {
      // Check if mouse is over a target node port
      const conn = connecting()!;
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      const mx = (e.clientX - rect.left - panX()) / zoom();
      const my = (e.clientY - rect.top - panY()) / zoom();
      
      // Find target node at mouse position (expanded hit area for entire node)
      const targetNode = nodes().find(n => 
        mx >= n.x - 10 && mx <= n.x + NODE_WIDTH + 10 &&
        my >= n.y - 20 && my <= n.y + NODE_HEIGHT + 20
      );
      
      if (targetNode) {
        // Prevent self-loops
        if (targetNode.id === conn.sourceId) {
          addLog("CANVAS", "Cannot create self-loop connection", "warning");
        } else {
          // Determine source and target based on port direction
          const sourceId = conn.sourcePort === "output" ? conn.sourceId : targetNode.id;
          const targetId = conn.sourcePort === "output" ? targetNode.id : conn.sourceId;
          
          // Check for duplicate edge
          const isDuplicate = edges().some(e => e.source === sourceId && e.target === targetId);
          if (isDuplicate) {
            addLog("CANVAS", "Edge already exists between these nodes", "warning");
          } else {
            // Create new edge
            const newEdge: FSMEdge = {
              id: `e${Date.now()}`,
              source: sourceId,
              target: targetId,
            };
            pushHistory();
            setEdges([...edges(), newEdge]);
            addLog("CANVAS", `Created edge: ${getNodeById(sourceId)?.label || sourceId} → ${getNodeById(targetId)?.label || targetId}`, "success");
          }
        }
      }
    }
    
    setIsPanning(false);
    setDraggingNode(null);
    setIsMarqueeSelecting(false);
    setSelectionBox(null);
    setConnecting(null);
  };
  
  // Context menu handler
  const handleCanvasContextMenu = (e: MouseEvent) => {
    e.preventDefault();
    const target = e.target as HTMLElement;
    
    // Check if right-clicking on a node
    const nodeElement = target.closest('.fsm-node');
    if (nodeElement) {
      const nodeId = nodeElement.getAttribute('data-node-id');
      if (nodeId) {
        setContextMenu({ visible: true, x: e.clientX, y: e.clientY, type: "node", targetId: nodeId });
        return;
      }
    }
    
    // Check if right-clicking on an edge (check nearby edges)
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const mx = (e.clientX - rect.left - panX()) / zoom();
    const my = (e.clientY - rect.top - panY()) / zoom();
    
    // Find nearest edge to click position
    const nearestEdge = edges().find(edge => {
      const source = getNodeById(edge.source);
      const target = getNodeById(edge.target);
      if (!source || !target) return false;
      
      const sx = source.x + NODE_WIDTH / 2;
      const sy = source.y + NODE_HEIGHT;
      const tx = target.x + NODE_WIDTH / 2;
      const ty = target.y;
      
      // Simple distance check to the line midpoint and path
      const midX = (sx + tx) / 2;
      const midY = (sy + ty) / 2;
      const dist = Math.sqrt((mx - midX) ** 2 + (my - midY) ** 2);
      
      return dist < 30; // Click within 30px of midpoint
    });
    
    if (nearestEdge) {
      setContextMenu({ visible: true, x: e.clientX, y: e.clientY, type: "edge", targetId: nearestEdge.id });
      return;
    }
    
    // Right-click on canvas
    setContextMenu({ visible: true, x: e.clientX, y: e.clientY, type: "canvas" });
  };
  
  // Context menu actions
  const handleContextMenuAction = (action: string) => {
    const menu = contextMenu();
    setContextMenu({ visible: false, x: 0, y: 0, type: "canvas" });
    
    if (menu.type === "node" && menu.targetId) {
      const node = getNodeById(menu.targetId);
      if (!node) return;
      
      switch (action) {
        case "delete":
          pushHistory();
          setNodes(nodes().filter(n => n.id !== menu.targetId));
          setEdges(edges().filter(e => e.source !== menu.targetId && e.target !== menu.targetId));
          setSelectedNode(null);
          addLog("CANVAS", `Deleted node: ${node.label}`, "info");
          break;
        case "duplicate":
          pushHistory();
          const newNode: FSMNode = {
            ...node,
            id: `node_${Date.now()}`,
            x: node.x + 50,
            y: node.y + 50,
          };
          setNodes([...nodes(), newNode]);
          setSelectedNode(newNode.id);
          addLog("CANVAS", `Duplicated node: ${node.label}`, "success");
          break;
      }
    } else if (menu.type === "canvas") {
      switch (action) {
        case "add_input":
          handleAddNode("input");
          break;
        case "add_process":
          handleAddNode("process");
          break;
        case "add_output":
          handleAddNode("output");
          break;
        case "add_decision":
          handleAddNode("decision");
          break;
      }
    } else if (menu.type === "edge" && menu.targetId) {
      const edge = edges().find(e => e.id === menu.targetId);
      if (!edge) return;
      
      switch (action) {
        case "delete":
          pushHistory();
          setEdges(edges().filter(e => e.id !== menu.targetId));
          const sourceNode = getNodeById(edge.source);
          const targetNode = getNodeById(edge.target);
          addLog("CANVAS", `Deleted edge: ${sourceNode?.label || edge.source} → ${targetNode?.label || edge.target}`, "info");
          break;
      }
    }
  };
  
  // Add node at a position
  const handleAddNode = (type: FSMNode["type"]) => {
    pushHistory();
    const existingCount = nodes().filter(n => n.type === type).length;
    const labels: Record<FSMNode["type"], string> = {
      input: "START",
      process: "STATE",
      output: "END",
      decision: "CHECK",
      error: "ERROR",
      hardware: "HW",
      delay: "DELAY",
      interrupt: "IRQ",
    };
    const snapped = snapPosition(300 + existingCount * 50, 100 + nodes().length * 100);
    const newNode: FSMNode = {
      id: `node_${Date.now()}`,
      label: `${labels[type]}${existingCount > 0 ? existingCount + 1 : ""}`,
      type,
      x: snapped.x,
      y: snapped.y,
    };
    setNodes([...nodes(), newNode]);
    setSelectedNode(newNode.id);
    addLog("CANVAS", `Added ${type} node: ${newNode.label}`, "success");
  };
  
  // Auto-layout using Rust engine
  const handleAutoLayout = async (algorithm: "hierarchical" | "force_directed" | "grid" = "hierarchical") => {
    try {
      addLog("CANVAS", `Applying ${algorithm} layout...`, "info");
      
      // Convert nodes to Rust format and initialize engine
      const canvasNodes = nodes().map(n => ({
        id: n.id,
        label: n.label,
        node_type: n.type,
        x: n.x,
        y: n.y,
        width: 160,
        height: 80,
        entry_action: n.entryAction || undefined,
        exit_action: (n as any).exitAction || undefined,
      }));
      
      const canvasEdges = edges().map(e => ({
        id: e.id,
        source: e.source,
        target: e.target,
        label: e.label || undefined,
        condition: (e as any).condition || undefined,
      }));
      
      // Initialize Rust engine with current state
      await invoke("canvas_init", { nodes: canvasNodes, edges: canvasEdges });
      
      // Apply layout
      const newState = await invoke<{ nodes: any[], edges: any[] }>("canvas_auto_layout", { algorithm });
      
      // Update local state with new positions
      pushHistory();
      const updatedNodes = nodes().map(node => {
        const newNode = newState.nodes.find((n: any) => n.id === node.id);
        if (newNode) {
          return { ...node, x: newNode.x, y: newNode.y };
        }
        return node;
      });
      setNodes(updatedNodes);
      
      addLog("CANVAS", `Applied ${algorithm} layout successfully!`, "success");
    } catch (e) {
      addLog("ERROR", `Auto-layout failed: ${e}`, "error");
    }
  };
  
  // Validate graph using Rust engine
  const handleValidate = async () => {
    try {
      const canvasNodes = nodes().map(n => ({
        id: n.id,
        label: n.label,
        node_type: n.type,
        x: n.x,
        y: n.y,
        width: 160,
        height: 80,
      }));
      
      const canvasEdges = edges().map(e => ({
        id: e.id,
        source: e.source,
        target: e.target,
      }));
      
      await invoke("canvas_init", { nodes: canvasNodes, edges: canvasEdges });
      const result = await invoke<{ valid: boolean, errors: any[], warnings: any[] }>("canvas_validate");
      
      if (result.valid) {
        addLog("VALIDATE", `✓ Graph is valid (${result.warnings.length} warnings)`, "success");
      } else {
        addLog("VALIDATE", `✗ ${result.errors.length} errors found`, "error");
        result.errors.forEach((err: any) => {
          addLog("VALIDATE", `  ${err.message}`, "error");
        });
      }
      result.warnings.forEach((warn: any) => {
        addLog("VALIDATE", `  ⚠ ${warn.message}`, "warning");
      });
    } catch (e) {
      addLog("ERROR", `Validation failed: ${e}`, "error");
    }
  };
  
  // Node drag handlers
  const handleNodeMouseDown = (e: MouseEvent, nodeId: string) => {
    e.stopPropagation();
    const node = getNodeById(nodeId);
    if (!node) return;
    
    setDraggingNode(nodeId);
    setSelectedNode(nodeId);
    
    const z = zoom();
    setDragOffset({
      x: e.clientX - (node.x * z + panX()),
      y: e.clientY - (node.y * z + panY()),
    });
  };

  // Handle tool actions from AI agents
  const handleToolAction = (action: any) => {
    console.log("[AgentTool]", action);
    
    switch (action.action) {
      case "add_node": {
        const node = action.node;
        // Generate unique ID if not provided or already exists
        const newId = node.id || `node_${Date.now()}`;
        const existingIds = nodes().map(n => n.id);
        const finalId = existingIds.includes(newId) ? `${newId}_${Date.now()}` : newId;
        
        const newNode: FSMNode = {
          id: finalId,
          label: node.label || "NewState",
          type: node.type || "process",
          x: node.x || 300 + (nodes().length * 30) % 200,
          y: node.y || 100 + nodes().length * 120,
          entryAction: node.entryAction,
        };
        
        setNodes([...nodes(), newNode]);
        addLog("AGENT", `Added node: ${newNode.label}`, "success");
        break;
      }
      
      case "remove_node": {
        const nodeId = action.nodeId;
        const node = nodes().find(n => n.id === nodeId);
        if (node) {
          setNodes(nodes().filter(n => n.id !== nodeId));
          // Also remove connected edges
          setEdges(edges().filter(e => e.source !== nodeId && e.target !== nodeId));
          addLog("AGENT", `Removed node: ${node.label}`, "info");
        }
        break;
      }
      
      case "update_node": {
        const { nodeId, updates } = action;
        setNodes(nodes().map(n => {
          if (n.id === nodeId) {
            return {
              ...n,
              label: updates.label ?? n.label,
              type: updates.type ?? n.type,
              entryAction: updates.entryAction ?? n.entryAction,
            };
          }
          return n;
        }));
        addLog("AGENT", `Updated node: ${nodeId}`, "info");
        break;
      }
      
      case "add_edge": {
        const edge = action.edge;
        const newId = edge.id || `edge_${Date.now()}`;
        const existingIds = edges().map(e => e.id);
        const finalId = existingIds.includes(newId) ? `${newId}_${Date.now()}` : newId;
        
        // Verify source and target exist
        const sourceExists = nodes().some(n => n.id === edge.source);
        const targetExists = nodes().some(n => n.id === edge.target);
        
        if (!sourceExists || !targetExists) {
          addLog("AGENT", `Cannot add edge: source or target node not found`, "error");
          break;
        }
        
        const newEdge: FSMEdge = {
          id: finalId,
          source: edge.source,
          target: edge.target,
          label: edge.label,
        };
        
        setEdges([...edges(), newEdge]);
        addLog("AGENT", `Added transition: ${edge.source} → ${edge.target}`, "success");
        break;
      }
      
      case "remove_edge": {
        const edgeId = action.edgeId;
        const edge = edges().find(e => e.id === edgeId);
        if (edge) {
          setEdges(edges().filter(e => e.id !== edgeId));
          addLog("AGENT", `Removed transition: ${edge.source} → ${edge.target}`, "info");
        }
        break;
      }
      
      case "validate_fsm": {
        // Run FSM validation
        const issues: string[] = [];
        const nodeIds = nodes().map(n => n.id);
        
        // Check for unreachable states
        const targetedNodes = new Set(edges().map(e => e.target));
        const startNodes = nodes().filter(n => n.type === "input");
        
        nodes().forEach(n => {
          if (n.type !== "input" && !targetedNodes.has(n.id)) {
            issues.push(`State "${n.label}" is unreachable`);
          }
        });
        
        // Check for dead ends
        const sourceNodes = new Set(edges().map(e => e.source));
        nodes().forEach(n => {
          if (n.type !== "output" && !sourceNodes.has(n.id)) {
            issues.push(`State "${n.label}" has no outgoing transitions`);
          }
        });
        
        if (issues.length === 0) {
          addLog("AGENT", "✅ FSM validation passed - no issues found", "success");
        } else {
          issues.forEach(issue => addLog("AGENT", `⚠️ ${issue}`, "warning"));
        }
        break;
      }
      
      default:
        addLog("AGENT", `Unknown action: ${action.action}`, "warning");
    }
  };

  // Button handlers
  const handleNewProject = async () => {
    try {
      const project = await invoke("create_project", { name: "New Project", targetMcu: "stm32f401" });
      setProjectName((project as any).name);
      addLog("PROJECT", "Created new project", "success");
    } catch (e) {
      addLog("ERROR", `${e}`, "error");
    }
  };

  const handleSimulate = () => {
    if (simStatus() === "running") {
      setSimStatus("paused");
      addLog("SIM", "Simulation paused", "warning");
    } else {
      setSimStatus("running");
      addLog("SIM", "Simulation started", "success");
    }
  };

  const handleStep = () => addLog("SIM", "Step executed", "info");
  const handleStop = () => { setSimStatus("idle"); addLog("SIM", "Simulation stopped", "info"); };

  const handleGenerateCode = async () => {
    if (isGenerating()) return;
    setIsGenerating(true);
    addLog("CODEGEN", `Generating ${codeLanguage()} code with AI...`, "info");
    
    try {
      // Convert frontend nodes/edges to backend format
      const backendNodes = nodes().map(n => ({
        id: n.id,
        label: n.label,
        node_type: n.type.charAt(0).toUpperCase() + n.type.slice(1),
        position: { x: n.x, y: n.y },
        entry_action: n.entryAction || null,
        exit_action: null,
        description: null,
      }));
      const backendEdges = edges().map(e => ({
        id: e.id,
        source: e.source,
        target: e.target,
        label: e.label || null,
        guard: null,
        action: null,
      }));
      
      const code = await invoke("ai_generate_code", {
        nodes: backendNodes,
        edges: backendEdges,
        language: codeLanguage(),
      });
      setGeneratedCode(code as string);
      setActivePanel("code");
      addLog("CODEGEN", `Generated ${codeLanguage()} code (${(code as string).length} chars)`, "success");
    } catch (e) {
      addLog("ERROR", `Code generation failed: ${e}`, "error");
      // Fallback to template-based generation
      try {
        const fallback = await invoke("generate_code", {
          project: {
            id: "00000000-0000-0000-0000-000000000000",
            name: projectName(),
            nodes: nodes(),
            edges: edges(),
            target_mcu: targetMcu().toLowerCase(),
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
          target: codeLanguage().toLowerCase(),
        });
        setGeneratedCode((fallback as any).code);
        setActivePanel("code");
        addLog("CODEGEN", "Used template generation (AI unavailable)", "warning");
      } catch (e2) {
        addLog("ERROR", `Template generation also failed: ${e2}`, "error");
      }
    }
    setIsGenerating(false);
  };

  const handleDetectDevices = async () => {
    addLog("HW", "Scanning for serial ports...", "info");
    try {
      const ports = await invoke("list_serial_ports");
      setSerialPorts(ports as any[]);
      addLog("HW", `Found ${(ports as any[]).length} serial port(s)`, "success");
      (ports as any[]).forEach((p: any) => {
        const info = p.info;
        if (info.type === "USB") {
          addLog("HW", `  → ${p.name}: ${info.product || info.manufacturer || 'USB Device'} [${info.vid}:${info.pid}]`, "info");
        } else {
          addLog("HW", `  → ${p.name}: ${info.type}`, "info");
        }
      });
      setActivePanel("hardware");
    } catch (e) {
      addLog("ERROR", `${e}`, "error");
    }
  };
  
  const loadMcuList = async () => {
    try {
      const mcus = await invoke("get_mcu_list");
      setMcuList(mcus as any[]);
    } catch (e) {
      addLog("ERROR", `Failed to load MCU list: ${e}`, "error");
    }
  };
  
  const handleZoom = (delta: number) => {
    setZoom(Math.min(3, Math.max(0.25, zoom() + delta)));
  };
  
  const handleFitView = () => {
    setZoom(1);
    setPanX(0);
    setPanY(0);
  };
  
  // AI Chat
  const handleSendChat = async () => {
    const msg = chatInput().trim();
    if (!msg || isAiLoading()) return;
    
    setChatMessages(prev => [...prev, { role: "user", content: msg }]);
    setChatInput("");
    setIsAiLoading(true);
    
    try {
      const response = await invoke("ai_chat", { message: msg });
      setChatMessages(prev => [...prev, { role: "assistant", content: response as string }]);
    } catch (e) {
      setChatMessages(prev => [...prev, { role: "assistant", content: `Error: ${e}` }]);
    }
    setIsAiLoading(false);
  };
  
  // Create FSM from natural language description
  const handleCreateFromDescription = async () => {
    const desc = fsmDescription().trim();
    if (!desc || isParsingFsm()) return;
    
    setIsParsingFsm(true);
    addLog("AI", "Parsing FSM description...", "info");
    
    try {
      const jsonStr = await invoke("ai_parse_fsm", { description: desc });
      // Parse the JSON response
      const data = JSON.parse(jsonStr as string);
      
      if (data.nodes && data.edges) {
        setNodes(data.nodes);
        setEdges(data.edges);
        setShowDescriptionModal(false);
        setFsmDescription("");
        addLog("AI", `Created FSM with ${data.nodes.length} nodes and ${data.edges.length} edges`, "success");
      } else {
        throw new Error("Invalid FSM structure returned");
      }
    } catch (e) {
      addLog("ERROR", `Failed to parse FSM: ${e}`, "error");
    }
    setIsParsingFsm(false);
  };
  
  // Generate peripheral driver
  const handleGenerateDriver = async () => {
    if (isGeneratingDriver()) return;
    setIsGeneratingDriver(true);
    
    try {
      let result;
      if (driverType() === "GPIO") {
        addLog("DRIVER", `Generating GPIO driver for P${gpioPort()}${gpioPin()}...`, "info");
        result = await invoke("generate_gpio_driver", {
          port: gpioPort(),
          pin: gpioPin(),
          mode: gpioMode(),
          language: driverLanguage(),
        });
      } else if (driverType() === "UART") {
        addLog("DRIVER", `Generating UART driver for ${uartInstance()}...`, "info");
        result = await invoke("generate_uart_driver", {
          instance: uartInstance(),
          baudRate: uartBaud(),
          useDma: uartDma(),
          language: driverLanguage(),
        });
      } else if (driverType() === "SPI") {
        addLog("DRIVER", `Generating SPI driver for ${spiInstance()}...`, "info");
        result = await invoke("generate_spi_driver", {
          instance: spiInstance(),
          clockHz: spiClock(),
          mode: spiMode(),
          language: driverLanguage(),
        });
      } else if (driverType() === "I2C") {
        addLog("DRIVER", `Generating I2C driver for ${i2cInstance()}...`, "info");
        result = await invoke("generate_i2c_driver", {
          instance: i2cInstance(),
          speed: i2cSpeed(),
          language: driverLanguage(),
        });
      } else if (driverType() === "CAN") {
        addLog("DRIVER", `Generating CAN driver for ${canInstance()}...`, "info");
        result = await invoke("generate_can_driver", {
          instance: canInstance(),
          bitrate: canBitrate(),
          mode: "normal",
          language: driverLanguage(),
        });
      } else if (driverType() === "Modbus") {
        addLog("DRIVER", `Generating Modbus RTU driver...`, "info");
        result = await invoke("generate_modbus_driver", {
          uartInstance: uartInstance(),
          baudRate: uartBaud(),
          address: modbusAddress(),
          mode: modbusMode(),
          language: driverLanguage(),
        });
      }
      
      setGeneratedDriver(result);
      addLog("DRIVER", `Generated ${driverType()} driver successfully!`, "success");
    } catch (e) {
      addLog("ERROR", `Driver generation failed: ${e}`, "error");
    }
    setIsGeneratingDriver(false);
  };

  return (
    <div class="app">
      {/* Header */}
      <header class="header">
        <div class="header-logo">
          <Icons.brain />
          <span>NeuroBench</span>
        </div>
        
        <div class="header-divider" />
        
        <div class="header-toolbar">
          <button class="toolbar-btn" onClick={handleNewProject} title="New Project">
            <Icons.newFile />
            <span>New</span>
          </button>
          <button class="toolbar-btn" title="Save Project">
            <Icons.save />
            <span>Save</span>
          </button>
          <button class="toolbar-btn" title="Open Project">
            <Icons.folder />
            <span>Open</span>
          </button>
        </div>
        
        <div class="header-divider" />
        
        <div class="header-toolbar">
          <button 
            class={`toolbar-btn ${simStatus() === "running" ? "active" : "success"}`}
            onClick={handleSimulate}
          >
            <Show when={simStatus() === "running"} fallback={<Icons.play />}>
              <Icons.pause />
            </Show>
            <span>{simStatus() === "running" ? "Pause" : "Run"}</span>
          </button>
          <button class="toolbar-btn" onClick={handleStep}><Icons.step /></button>
          <button class="toolbar-btn danger" onClick={handleStop}><Icons.stop /></button>
        </div>
        
        <div class="header-divider" />
        
        <div class="header-toolbar">
          <button class="toolbar-btn" onClick={handleGenerateCode}><Icons.code /><span>Generate</span></button>
          <button class="toolbar-btn" onClick={handleDetectDevices}><Icons.chip /><span>Devices</span></button>
          <button class="toolbar-btn success" onClick={() => setShowDescriptionModal(true)}><Icons.brain /><span>AI Magic</span></button>
        </div>
        
        <div class="header-divider" />
        
        {/* Canvas Tools */}
        <div class="header-toolbar">
          <div class="toolbar-dropdown">
            <button class="toolbar-btn" title="Auto-Layout">
              <Icons.layout />
              <span>Layout</span>
            </button>
            <div class="dropdown-menu">
              <button class="dropdown-item" onClick={() => handleAutoLayout("hierarchical")}>
                Hierarchical (Top-Down)
              </button>
              <button class="dropdown-item" onClick={() => handleAutoLayout("force_directed")}>
                Force-Directed (Spring)
              </button>
              <button class="dropdown-item" onClick={() => handleAutoLayout("grid")}>
                Grid Layout
              </button>
            </div>
          </div>
          <button class="toolbar-btn" onClick={handleValidate} title="Validate Graph">
            <Icons.validate />
            <span>Validate</span>
          </button>
        </div>
        
        <div class="header-spacer" />
        
        <div class="header-status">
          {/* IDE Selector */}
          <select class="ide-selector" title="IDE Target">
            <option value="stm32cubeide">STM32CubeIDE</option>
            <option value="keil">Keil MDK</option>
            <option value="iar">IAR Workbench</option>
            <option value="arduino">Arduino IDE</option>
            <option value="platformio">PlatformIO</option>
            <option value="vscode">VS Code</option>
          </select>
          
          {/* Status Indicator */}
          <div class="status-item">
            <span class={`status-dot ${simStatus() === "running" ? "active" : simStatus() === "paused" ? "warning" : ""}`} />
            <span>{simStatus().toUpperCase()}</span>
          </div>
          
          {/* MCU Badge */}
          <select class="mcu-selector" value={targetMcu()} onChange={(e) => setTargetMcu(e.currentTarget.value)}>
            <option value="STM32F401">STM32F401</option>
            <option value="STM32F103">STM32F103</option>
            <option value="ATMega328P">ATMega328P</option>
            <option value="ESP32">ESP32</option>
            <option value="RP2040">RP2040</option>
            <option value="nRF52840">nRF52840</option>
          </select>
        </div>
      </header>
      
      {/* Main Content */}
      <div class="main-content">
        {/* Sidebar */}
        <nav class="sidebar">
          <button class={`sidebar-btn ${activePanel() === "nodes" ? "active" : ""}`} onClick={() => setActivePanel("nodes")}><Icons.layers /></button>
          <button class={`sidebar-btn ${activePanel() === "chat" ? "active" : ""}`} onClick={() => setActivePanel("chat")}><Icons.message /></button>
          <button class={`sidebar-btn ${activePanel() === "hardware" ? "active" : ""}`} onClick={() => setActivePanel("hardware")}><Icons.chip /></button>
          <button class={`sidebar-btn ${activePanel() === "code" ? "active" : ""}`} onClick={() => setActivePanel("code")}><Icons.code /></button>
          <button class={`sidebar-btn ${activePanel() === "drivers" ? "active" : ""}`} onClick={() => setActivePanel("drivers")}><Icons.plug /></button>
          <button class={`sidebar-btn ${activePanel() === "pins" ? "active" : ""}`} onClick={() => setActivePanel("pins")}><Icons.cpu /></button>
          <button class={`sidebar-btn ${activePanel() === "rtos" ? "active" : ""}`} onClick={() => setActivePanel("rtos")} title="RTOS"><Icons.tasks /></button>
          <button class={`sidebar-btn ${activePanel() === "wireless" ? "active" : ""}`} onClick={() => setActivePanel("wireless")} title="Wireless"><Icons.wifi /></button>
          <button class={`sidebar-btn ${activePanel() === "dsp" ? "active" : ""}`} onClick={() => setActivePanel("dsp")} title="DSP"><Icons.activity /></button>
          <button class={`sidebar-btn ${activePanel() === "security" ? "active" : ""}`} onClick={() => setActivePanel("security")} title="Security"><Icons.lock /></button>
          <button class={`sidebar-btn ${activePanel() === "agents" ? "active" : ""}`} onClick={() => setActivePanel("agents")} title="AI Agents"><Icons.brain /></button>
          {/* NEW: Additional panel buttons */}
          <button class={`sidebar-btn ${activePanel() === "simulator" ? "active" : ""}`} onClick={() => setActivePanel("simulator")} title="Simulator"><Icons.simulator /></button>
          <button class={`sidebar-btn ${activePanel() === "debug" ? "active" : ""}`} onClick={() => setActivePanel("debug")} title="Debug"><Icons.debug /></button>
          <button class={`sidebar-btn ${activePanel() === "performance" ? "active" : ""}`} onClick={() => setActivePanel("performance")} title="Performance"><Icons.performance /></button>
          <button class={`sidebar-btn ${activePanel() === "build" ? "active" : ""}`} onClick={() => setActivePanel("build")} title="Build"><Icons.build /></button>
          <button class={`sidebar-btn ${activePanel() === "workflow" ? "active" : ""}`} onClick={() => setActivePanel("workflow")} title="Workflow"><Icons.workflow /></button>
          <button class={`sidebar-btn ${activePanel() === "git" ? "active" : ""}`} onClick={() => setActivePanel("git")} title="Git"><Icons.gitBranch /></button>
          <button class={`sidebar-btn ${activePanel() === "serial" ? "active" : ""}`} onClick={() => setActivePanel("serial")} title="Serial"><Icons.serial /></button>
          <button class={`sidebar-btn ${activePanel() === "memory" ? "active" : ""}`} onClick={() => setActivePanel("memory")} title="Memory"><Icons.memory /></button>
          <button class={`sidebar-btn ${activePanel() === "profiler" ? "active" : ""}`} onClick={() => setActivePanel("profiler")} title="Profiler"><Icons.profiler /></button>
          <button class={`sidebar-btn ${activePanel() === "power" ? "active" : ""}`} onClick={() => setActivePanel("power")} title="Power"><Icons.power /></button>
          <button class={`sidebar-btn ${activePanel() === "validation" ? "active" : ""}`} onClick={() => setActivePanel("validation")} title="Validation"><Icons.validate /></button>
          <button class={`sidebar-btn ${activePanel() === "history" ? "active" : ""}`} onClick={() => setActivePanel("history")} title="History">📜</button>
          <button class={`sidebar-btn ${activePanel() === "scheduler" ? "active" : ""}`} onClick={() => setActivePanel("scheduler")} title="Scheduler">⚡</button>
          <div class="sidebar-spacer" />
          <button class="sidebar-btn" onClick={() => setShowSettingsModal(true)} title="Settings"><Icons.settings /></button>
        </nav>
        
        {/* Canvas Area - Unified Canvas */}
        <UnifiedCanvas 
          projectName={projectName()} 
        />

        {/* Right Panel */}
        <aside class="right-panel">
          {/* NOTE: Properties Panel removed - handled by UnifiedCanvas */}
          
          {/* AI Chat Panel */}
          <Show when={activePanel() === "chat"}>
            <div class="panel-header"><Icons.message /><span>AI Assistant</span></div>
            <div class="panel-content" style="display:flex;flex-direction:column;padding:0;">
              <div style="flex:1;overflow-y:auto;padding:12px;">
                <For each={chatMessages()}>
                  {(msg) => (
                    <div style={`margin-bottom:12px;padding:8px 10px;border-radius:8px;${msg.role === "user" ? "background:#0f3460;margin-left:20px;" : "background:#1f1f3a;margin-right:20px;"}`}>
                      <div style={`font-size:10px;font-weight:600;margin-bottom:4px;color:${msg.role === "user" ? "#00d4ff" : "#a0a0a0"}`}>
                        {msg.role === "user" ? "You" : "NeuroBench AI"}
                      </div>
                      <div style="font-size:12px;white-space:pre-wrap;">{msg.content}</div>
                    </div>
                  )}
                </For>
                <Show when={isAiLoading()}>
                  <div style="color:#666;font-size:12px;">Thinking...</div>
                </Show>
              </div>
              <div style="padding:12px;border-top:1px solid #2a2a4a;display:flex;gap:8px;">
                <input 
                  class="panel-input" 
                  style="flex:1" 
                  placeholder="Ask about FSM design..." 
                  value={chatInput()} 
                  onInput={(e) => setChatInput(e.currentTarget.value)}
                  onKeyDown={(e) => e.key === "Enter" && handleSendChat()}
                />
                <button class="btn-primary" onClick={handleSendChat} disabled={isAiLoading()}>
                  <Icons.send />
                </button>
              </div>
            </div>
          </Show>
          
          {/* Hardware Panel */}
          <Show when={activePanel() === "hardware"}>
            <div class="panel-header"><Icons.chip /><span>Hardware</span></div>
            <div class="panel-content">
              <div class="panel-section">
                <div class="panel-section-title">Target MCU</div>
                <select class="panel-input" value={targetMcu()} onChange={(e) => setTargetMcu(e.currentTarget.value)}>
                  <option value="STM32F401">STM32F401 BlackPill</option>
                  <option value="STM32F103">STM32F103 BluePill</option>
                  <option value="ESP32">ESP32-WROOM</option>
                  <option value="RP2040">Raspberry Pi Pico</option>
                  <option value="ATmega328P">Arduino Uno</option>
                </select>
              </div>
              <div class="panel-section">
                <div class="panel-section-title">Serial Ports</div>
                <button class="btn-primary" style="width:100%;margin-bottom:8px;" onClick={handleDetectDevices}>
                  Scan Ports
                </button>
                <Show when={serialPorts().length > 0} fallback={
                  <div style="color:#666;font-size:11px;">No ports detected. Click Scan.</div>
                }>
                  <For each={serialPorts()}>
                    {(port) => (
                      <div style="padding:8px;background:#1f1f3a;border-radius:4px;margin-bottom:4px;">
                        <div style="font-weight:600;font-size:12px;color:#00d4ff;">{port.name}</div>
                        <div style="font-size:10px;color:#666;">
                          {port.info.type === "USB" ? `${port.info.product || 'USB'} [${port.info.vid}:${port.info.pid}]` : port.info.type}
                        </div>
                      </div>
                    )}
                  </For>
                </Show>
              </div>
            </div>
          </Show>
          
          {/* Code Panel */}
          <Show when={activePanel() === "code"}>
            <div class="panel-header"><Icons.code /><span>Generated Code</span></div>
            <div class="panel-content" style="padding:0;display:flex;flex-direction:column;">
              <div style="padding:8px 12px;border-bottom:1px solid #2a2a4a;display:flex;gap:8px;align-items:center;">
                <select class="panel-input" style="flex:1" value={codeLanguage()} onChange={(e) => setCodeLanguage(e.currentTarget.value)}>
                  <option value="C">C</option>
                  <option value="Cpp">C++</option>
                  <option value="Rust">Rust</option>
                  <option value="Python">Python</option>
                </select>
                <button class="btn-primary" onClick={handleGenerateCode} disabled={isGenerating()}>
                  {isGenerating() ? "..." : "Generate"}
                </button>
              </div>
              <div style="flex:1;overflow:auto;padding:8px;">
                <Show when={generatedCode()} fallback={
                  <div style="color:#666;font-size:11px;text-align:center;padding:20px;">
                    Click Generate to create code from your FSM
                  </div>
                }>
                  <pre style="font-family:var(--font-mono);font-size:10px;white-space:pre-wrap;color:#eaeaea;margin:0;">
                    {generatedCode()}
                  </pre>
                </Show>
              </div>
            </div>
          </Show>
          
          {/* Drivers Panel */}
          <Show when={activePanel() === "drivers"}>
            <div class="panel-header"><Icons.plug /><span>Driver Generator</span></div>
            <div class="panel-content">
              <div class="panel-section">
                <div class="panel-section-title">Peripheral Type</div>
                <select class="panel-input" value={driverType()} onChange={(e) => setDriverType(e.currentTarget.value as "GPIO" | "UART" | "SPI" | "I2C" | "CAN" | "Modbus")}>
                  <option value="GPIO">GPIO</option>
                  <option value="UART">UART</option>
                  <option value="SPI">SPI</option>
                  <option value="I2C">I2C</option>
                  <option value="CAN">CAN Bus</option>
                  <option value="Modbus">Modbus RTU</option>
                </select>
              </div>
              
              <Show when={driverType() === "GPIO"}>
                <div class="panel-section">
                  <div class="panel-section-title">Port</div>
                  <select class="panel-input" value={gpioPort()} onChange={(e) => setGpioPort(e.currentTarget.value)}>
                    <option value="A">Port A</option>
                    <option value="B">Port B</option>
                    <option value="C">Port C</option>
                    <option value="D">Port D</option>
                  </select>
                </div>
                <div class="panel-section">
                  <div class="panel-section-title">Pin</div>
                  <input class="panel-input" type="number" min="0" max="15" value={gpioPin()} onInput={(e) => setGpioPin(parseInt(e.currentTarget.value) || 0)} />
                </div>
                <div class="panel-section">
                  <div class="panel-section-title">Mode</div>
                  <select class="panel-input" value={gpioMode()} onChange={(e) => setGpioMode(e.currentTarget.value)}>
                    <option value="output">Output</option>
                    <option value="input">Input</option>
                    <option value="analog">Analog</option>
                    <option value="alternate">Alternate Function</option>
                  </select>
                </div>
              </Show>
              
              <Show when={driverType() === "UART"}>
                <div class="panel-section">
                  <div class="panel-section-title">Instance</div>
                  <select class="panel-input" value={uartInstance()} onChange={(e) => setUartInstance(e.currentTarget.value)}>
                    <option value="USART1">USART1</option>
                    <option value="USART2">USART2</option>
                    <option value="USART3">USART3</option>
                    <option value="UART4">UART4</option>
                  </select>
                </div>
                <div class="panel-section">
                  <div class="panel-section-title">Baud Rate</div>
                  <select class="panel-input" value={uartBaud()} onChange={(e) => setUartBaud(parseInt(e.currentTarget.value))}>
                    <option value="9600">9600</option>
                    <option value="19200">19200</option>
                    <option value="38400">38400</option>
                    <option value="57600">57600</option>
                    <option value="115200">115200</option>
                    <option value="230400">230400</option>
                    <option value="460800">460800</option>
                  </select>
                </div>
                <div class="panel-section">
                  <label style="display:flex;align-items:center;gap:8px;font-size:12px;">
                    <input type="checkbox" checked={uartDma()} onChange={(e) => setUartDma(e.currentTarget.checked)} />
                    Enable DMA
                  </label>
                </div>
              </Show>
              
              <Show when={driverType() === "SPI"}>
                <div class="panel-section">
                  <div class="panel-section-title">Instance</div>
                  <select class="panel-input" value={spiInstance()} onChange={(e) => setSpiInstance(e.currentTarget.value)}>
                    <option value="SPI1">SPI1</option>
                    <option value="SPI2">SPI2</option>
                    <option value="SPI3">SPI3</option>
                  </select>
                </div>
                <div class="panel-section">
                  <div class="panel-section-title">Clock Speed</div>
                  <select class="panel-input" value={spiClock()} onChange={(e) => setSpiClock(parseInt(e.currentTarget.value))}>
                    <option value="1000000">1 MHz</option>
                    <option value="2000000">2 MHz</option>
                    <option value="4000000">4 MHz</option>
                    <option value="8000000">8 MHz</option>
                    <option value="16000000">16 MHz</option>
                  </select>
                </div>
                <div class="panel-section">
                  <div class="panel-section-title">SPI Mode</div>
                  <select class="panel-input" value={spiMode()} onChange={(e) => setSpiMode(parseInt(e.currentTarget.value))}>
                    <option value="0">Mode 0 (CPOL=0, CPHA=0)</option>
                    <option value="1">Mode 1 (CPOL=0, CPHA=1)</option>
                    <option value="2">Mode 2 (CPOL=1, CPHA=0)</option>
                    <option value="3">Mode 3 (CPOL=1, CPHA=1)</option>
                  </select>
                </div>
              </Show>
              
              <Show when={driverType() === "I2C"}>
                <div class="panel-section">
                  <div class="panel-section-title">Instance</div>
                  <select class="panel-input" value={i2cInstance()} onChange={(e) => setI2cInstance(e.currentTarget.value)}>
                    <option value="I2C1">I2C1</option>
                    <option value="I2C2">I2C2</option>
                    <option value="I2C3">I2C3</option>
                  </select>
                </div>
                <div class="panel-section">
                  <div class="panel-section-title">Speed</div>
                  <select class="panel-input" value={i2cSpeed()} onChange={(e) => setI2cSpeed(e.currentTarget.value)}>
                    <option value="standard">Standard (100 kHz)</option>
                    <option value="fast">Fast (400 kHz)</option>
                    <option value="fastplus">Fast+ (1 MHz)</option>
                  </select>
                </div>
              </Show>
              
              <Show when={driverType() === "CAN"}>
                <div class="panel-section">
                  <div class="panel-section-title">Instance</div>
                  <select class="panel-input" value={canInstance()} onChange={(e) => setCanInstance(e.currentTarget.value)}>
                    <option value="CAN1">CAN1</option>
                    <option value="CAN2">CAN2</option>
                  </select>
                </div>
                <div class="panel-section">
                  <div class="panel-section-title">Bitrate</div>
                  <select class="panel-input" value={canBitrate()} onChange={(e) => setCanBitrate(parseInt(e.currentTarget.value))}>
                    <option value="125000">125 kbps</option>
                    <option value="250000">250 kbps</option>
                    <option value="500000">500 kbps</option>
                    <option value="1000000">1 Mbps</option>
                  </select>
                </div>
              </Show>
              
              <Show when={driverType() === "Modbus"}>
                <div class="panel-section">
                  <div class="panel-section-title">Mode</div>
                  <select class="panel-input" value={modbusMode()} onChange={(e) => setModbusMode(e.currentTarget.value)}>
                    <option value="master">RTU Master</option>
                    <option value="slave">RTU Slave</option>
                  </select>
                </div>
                <div class="panel-section">
                  <div class="panel-section-title">Slave Address</div>
                  <input class="panel-input" type="number" min="1" max="247" value={modbusAddress()} onInput={(e) => setModbusAddress(parseInt(e.currentTarget.value) || 1)} />
                </div>
                <div class="panel-section">
                  <div class="panel-section-title">UART</div>
                  <select class="panel-input" value={uartInstance()} onChange={(e) => setUartInstance(e.currentTarget.value)}>
                    <option value="USART1">USART1</option>
                    <option value="USART2">USART2</option>
                    <option value="USART3">USART3</option>
                  </select>
                </div>
              </Show>
              
              <div class="panel-section">
                <div class="panel-section-title">Language</div>
                <select class="panel-input" value={driverLanguage()} onChange={(e) => setDriverLanguage(e.currentTarget.value)}>
                  <option value="C">C</option>
                  <option value="Cpp">C++</option>
                  <option value="Rust">Rust</option>
                </select>
              </div>
              
              <button class="btn-primary" style="width:100%;margin-top:12px;" onClick={handleGenerateDriver} disabled={isGeneratingDriver()}>
                {isGeneratingDriver() ? "Generating..." : "Generate Driver"}
              </button>
              
              <Show when={generatedDriver()}>
                <div style="margin-top:12px;padding:8px;background:#1a1a2e;border-radius:4px;max-height:200px;overflow:auto;">
                  <div style="font-size:10px;color:#00d4ff;margin-bottom:4px;">Generated Code:</div>
                  <pre style="font-family:var(--font-mono);font-size:9px;color:#eaeaea;margin:0;white-space:pre-wrap;">
                    {generatedDriver()?.source || ""}
                  </pre>
                </div>
              </Show>
            </div>
          </Show>
          
          {/* Pins Panel - Visual MCU Pin Configurator */}
          <Show when={activePanel() === "pins"}>
            <div class="panel-header"><Icons.cpu /><span>MCU Pins</span></div>
            <div class="panel-content" style="padding:4px;">
              <PinDiagram 
                mcuId="STM32F401"
                onPinSelect={(pin, func) => {
                  addLog("PINS", `Configured ${pin.name} as ${func}`, "success");
                }}
              />
            </div>
          </Show>
          
          {/* RTOS Panel */}
          <Show when={activePanel() === "rtos"}>
            <div class="panel-header"><Icons.tasks /><span>RTOS Tasks</span></div>
            <div class="panel-content">
              <div class="panel-section">
                <div class="panel-section-title">FreeRTOS Configuration</div>
                <div style="font-size:11px;color:#aaa;margin-bottom:8px;">
                  Generate RTOS task scaffolding with priorities and scheduling.
                </div>
              </div>
              
              <div class="panel-section">
                <div class="panel-section-title">Example Tasks</div>
                <div style="font-size:10px;color:#666;padding:8px;background:#1a1a2e;border-radius:4px;">
                  <div style="margin-bottom:4px;">• <span style="color:#4CAF50;">LED_Task</span> - Priority 1, 256 words</div>
                  <div style="margin-bottom:4px;">• <span style="color:#2196F3;">UART_Task</span> - Priority 2, 512 words</div>
                  <div>• <span style="color:#FF9800;">Sensor_Task</span> - Priority 3, 256 words</div>
                </div>
              </div>
              
              <div class="panel-section">
                <div class="panel-section-title">Heap Size</div>
                <select class="panel-input">
                  <option value="8">8 KB</option>
                  <option value="16" selected>16 KB</option>
                  <option value="32">32 KB</option>
                  <option value="64">64 KB</option>
                </select>
              </div>
              
              <div class="panel-section">
                <div class="panel-section-title">Language</div>
                <select class="panel-input" value={driverLanguage()} onChange={(e) => setDriverLanguage(e.currentTarget.value)}>
                  <option value="C">C</option>
                  <option value="Cpp">C++</option>
                  <option value="Rust">Rust</option>
                </select>
              </div>
              
              <button class="btn-primary" style="width:100%;margin-top:12px;" onClick={async () => {
                try {
                  addLog("RTOS", "Generating FreeRTOS code...", "info");
                  const result = await invoke("generate_rtos_code", {
                    tasks: [
                      { name: "LED", priority: 1, stackSize: 256, periodMs: 500, handler: "LED_Handler" },
                      { name: "UART", priority: 2, stackSize: 512, periodMs: 100, handler: "UART_Handler" },
                      { name: "Sensor", priority: 3, stackSize: 256, periodMs: 50, handler: "Sensor_Handler" },
                    ],
                    heapSizeKb: 16,
                    language: driverLanguage(),
                  });
                  setGeneratedDriver(result);
                  addLog("RTOS", "Generated FreeRTOS code successfully!", "success");
                } catch (e) {
                  addLog("ERROR", `RTOS generation failed: ${e}`, "error");
                }
              }}>
                Generate RTOS Code
              </button>
              
              <Show when={generatedDriver()?.peripheral === "RTOS"}>
                <div style="margin-top:12px;padding:8px;background:#1a1a2e;border-radius:4px;max-height:200px;overflow:auto;">
                  <div style="font-size:10px;color:#00d4ff;margin-bottom:4px;">Generated RTOS Code:</div>
                  <pre style="font-family:var(--font-mono);font-size:9px;color:#eaeaea;margin:0;white-space:pre-wrap;">
                    {generatedDriver()?.source || ""}
                  </pre>
                </div>
              </Show>
            </div>
          </Show>
          
          {/* AI Agents Panel */}
          <Show when={activePanel() === "agents"}>
            <AgentPanel onToolAction={handleToolAction} />
          </Show>
          
          {/* Timers & Interrupts Panel */}
          <Show when={activePanel() === "timers"}>
            <TimersPanel onLog={addLog} />
          </Show>
          
          {/* Serial Peripherals Panel */}
          <Show when={activePanel() === "peripherals"}>
            <PeripheralsPanel onLog={addLog} />
          </Show>
          
          {/* Clock & Power Panel */}
          <Show when={activePanel() === "clock"}>
            <ClockPanel onLog={addLog} />
          </Show>
          
          {/* Analog I/O Panel */}
          <Show when={activePanel() === "analog"}>
            <AnalogPanel onLog={addLog} />
          </Show>
          
          {/* MCU Selector Panel */}
          <Show when={activePanel() === "mcu"}>
            <McuSelector onLog={addLog} />
          </Show>
          
          {/* RTOS Panel */}
          <Show when={activePanel() === "rtos"}>
            <RTOSPanel onLog={addLog} />
          </Show>
          
          {/* Wireless Panel */}
          <Show when={activePanel() === "wireless"}>
            <WirelessPanel onLog={addLog} />
          </Show>
          
          {/* DSP Panel */}
          <Show when={activePanel() === "dsp"}>
            <DSPPanel onLog={addLog} />
          </Show>
          
          {/* Security Panel */}
          <Show when={activePanel() === "security"}>
            <SecurityPanel onLog={addLog} />
          </Show>
          
          {/* NEW: Additional Panels */}
          <Show when={activePanel() === "simulator"}>
            <SimulatorPanel onLog={addLog} />
          </Show>
          
          <Show when={activePanel() === "debug"}>
            <DebugPanel />
          </Show>
          
          <Show when={activePanel() === "performance"}>
            <PerformancePanel onLog={addLog} />
          </Show>
          
          <Show when={activePanel() === "build"}>
            <BuildPanel />
          </Show>
          
          <Show when={activePanel() === "workflow"}>
            <WorkflowPanel />
          </Show>
          
          <Show when={activePanel() === "git"}>
            <GitPanel projectPath="." onLog={addLog} />
          </Show>
          
          <Show when={activePanel() === "serial"}>
            <SerialPanel onLog={addLog} />
          </Show>
          
          <Show when={activePanel() === "memory"}>
            <MemoryPanel />
          </Show>
          
          <Show when={activePanel() === "profiler"}>
            <ProfilerPanel />
          </Show>
          
          <Show when={activePanel() === "power"}>
            <PowerPanel onLog={addLog} />
          </Show>
          
          <Show when={activePanel() === "validation"}>
            <ValidationPanel code={generatedCode() || ""} language={codeLanguage()} onLog={addLog} />
          </Show>
          
          <Show when={activePanel() === "history"}>
            <HistoryPanel />
          </Show>
          
          <Show when={activePanel() === "scheduler"}>
            <SchedulerPanel />
          </Show>
        </aside>
      </div>
      
      {/* Bottom Panel */}
      <div class="bottom-panel">
        <div class="bottom-panel-tabs">
          <button class={`bottom-panel-tab ${activeBottomTab() === "terminal" ? "active" : ""}`} onClick={() => setActiveBottomTab("terminal")}>Terminal</button>
          <button class={`bottom-panel-tab ${activeBottomTab() === "console" ? "active" : ""}`} onClick={() => setActiveBottomTab("console")}>Console</button>
          <button class={`bottom-panel-tab ${activeBottomTab() === "problems" ? "active" : ""}`} onClick={() => setActiveBottomTab("problems")}>Problems</button>
          <button class={`bottom-panel-tab ${activeBottomTab() === "output" ? "active" : ""}`} onClick={() => setActiveBottomTab("output")}>Output</button>
        </div>
        
        <Show when={activeBottomTab() === "terminal"}>
          <div class="terminal-panel">
            <Terminal onCommand={(cmd, output) => {
              addLog("TERMINAL", `> ${cmd}`, "info");
            }} />
          </div>
        </Show>
        
        <Show when={activeBottomTab() === "console"}>
          <div class="console-content">
            <For each={logs()}>
              {(log) => (
                <div class="console-line">
                  <span class="console-time">{log.time}</span>
                  <span class={`console-tag ${log.type}`}>{log.source}</span>
                  <span class="console-message">{log.message}</span>
                </div>
              )}
            </For>
          </div>
        </Show>
        
        <Show when={activeBottomTab() === "problems"}>
          <div class="console-content">
            <div class="console-line info">
              <span class="console-message" style="color:#888;">No problems detected</span>
            </div>
          </div>
        </Show>
        
        <Show when={activeBottomTab() === "output"}>
          <div class="console-content">
            <div class="console-line">
              <span class="console-message" style="color:#888;">Build output will appear here...</span>
            </div>
          </div>
        </Show>
      </div>
      
      {/* FSM from Description Modal */}
      <Show when={showDescriptionModal()}>
        <div class="modal-overlay" onClick={() => setShowDescriptionModal(false)}>
          <div class="modal" onClick={(e) => e.stopPropagation()}>
            <div class="modal-header">
              <Icons.brain />
              <span>Create FSM from Description</span>
              <button class="modal-close" onClick={() => setShowDescriptionModal(false)}><Icons.x /></button>
            </div>
            <div class="modal-body">
              <p style="color:#a0a0a0;font-size:12px;margin-bottom:12px;">
                Describe your state machine in plain English. AI will generate the nodes and transitions.
              </p>
              <textarea 
                class="panel-textarea" 
                style="min-height:150px;width:100%;" 
                placeholder="Example: A traffic light controller with RED, YELLOW, and GREEN states. It starts in RED, transitions to GREEN after 30 seconds, then to YELLOW after 25 seconds, then back to RED after 5 seconds."
                value={fsmDescription()}
                onInput={(e) => setFsmDescription(e.currentTarget.value)}
              />
              <div style="display:flex;gap:8px;margin-top:12px;justify-content:flex-end;">
                <button class="btn-secondary" onClick={() => setShowDescriptionModal(false)}>Cancel</button>
                <button class="btn-primary" onClick={handleCreateFromDescription} disabled={isParsingFsm()}>
                  {isParsingFsm() ? "Creating..." : "Create FSM"}
                </button>
              </div>
            </div>
          </div>
        </div>
      </Show>
      
      {/* Settings Modal */}
      <Show when={showSettingsModal()}>
        <div class="modal-overlay" onClick={() => setShowSettingsModal(false)} />
        <SettingsPanel 
          onClose={() => setShowSettingsModal(false)}
          onLog={addLog}
        />
      </Show>
      

    </div>
  );
}

export default App;
