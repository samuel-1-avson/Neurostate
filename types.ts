
import { Node, Edge } from 'reactflow';

export type FSMDomain = 'EMBEDDED' | 'SOFTWARE' | 'NETWORKING' | 'MECHANICS';

export type Theme = 'NEURO' | 'CYBERPUNK' | 'BLUEPRINT' | 'TERMINAL';

export interface FSMNodeData {
  label: string;
  // Expanded types for multi-domain support
  type?: 
    // Embedded
    | 'input' | 'process' | 'output' | 'error' | 'listener' | 'decision' | 'hardware' | 'uart' | 'interrupt' | 'timer' | 'peripheral' | 'queue' | 'mutex' | 'critical' 
    // Software
    | 'function' | 'class' | 'interface' | 'database' | 'api_endpoint' | 'ui_view' | 'service' | 'state'
    // Networking
    | 'server' | 'client' | 'router' | 'firewall' | 'packet' | 'websocket' | 'cloud'
    // Mechanics
    | 'actuator' | 'piston' | 'valve' | 'motor' | 'sensor_analog' | 'sensor_digital' | 'controller'
    // Generic
    | 'math' | 'wireless' | 'storage' | 'logger' | 'display' | 'network' | 'sensor' | 'group';

  entryAction?: string; // JavaScript code string
  exitAction?: string; // JavaScript code string
  description?: string;
  aiReasoning?: string; // Explanation of the generated logic
  tags?: string[];
  isBreakpoint?: boolean;
  active?: boolean;
  error?: boolean;
  // Animation States
  executionState?: 'entry' | 'exit' | 'idle';
  executionLog?: string; // The specific line of code being executed
}

export interface FSMEdgeData {
  condition?: string; // JavaScript expression returning boolean
  // Animation States
  isTraversing?: boolean;
  guardResult?: 'pass' | 'fail' | null;
}

export interface ChatEntry {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
}

export interface FSMProject {
  id: string;
  name: string;
  domain: FSMDomain; // New field to track the domain
  description?: string;
  version?: string;
  tags?: string[];
  nodes: Node[];
  edges: Edge[];
  chatHistory: ChatEntry[];
  updatedAt: number;
}

export interface GhostIssue {
  id: string;
  severity: 'CRITICAL' | 'WARNING' | 'INFO';
  title: string;
  description: string;
  nodeId?: string;
}

export interface LogEntry {
  id: string;
  timestamp: string;
  source: 'SYSTEM' | 'GEMINI' | 'GHOST' | 'EXEC' | 'USER';
  message: string;
  type: 'info' | 'error' | 'success' | 'warning';
}

export enum SimulationStatus {
  IDLE = 'IDLE',
  RUNNING = 'RUNNING',
  PAUSED = 'PAUSED',
  ERROR = 'ERROR'
}

export type AgentState = 'IDLE' | 'LISTENING' | 'THINKING' | 'MODIFYING' | 'CREATING' | 'SPEAKING';

export interface TestCase {
  id: string;
  name: string;
  sequence: string[]; // List of events to trigger
  expectedState: string;
  description: string;
}

export interface ValidationReport {
  timestamp: number;
  critique: string[]; // List of AI observations (e.g., "Missing error handling")
  suggestions: string[]; // Actionable improvements
  testCases: TestCase[];
}

export interface ResourceMetrics {
  timestamp: number;
  lutUsage: number; // Percent 0-100
  ffUsage: number; // Absolute count
  memoryKB: number; // Estimated memory in KB
  powermW: number; // Estimated power in mW
  maxFreqMHz: number; // Estimated max frequency
  summary: string; // Brief AI comment
}

export interface SimTelemetry {
  uptimeMs: number;
  cpuLoad: number; // 0-100%
  ramUsageBytes: number;
  stackDepth: number;
  interruptCount: number;
  powerDrawMW: number;
  activeStateTimeMs: number;
  transitionsPerSec: number;
}

export type WorkspaceTemplate = 'ARCHITECT' | 'ENGINEER' | 'AUDITOR' | 'HACKER' | 'ZEN' | 'FULL_SUITE' | 'DEBUG_FOCUS' | 'AI_PAIR' | 'HARDWARE_LAB';

// --- HARDWARE TARGETS ---

export type FlashMethod = 'WEB_SERIAL' | 'USB_MSD' | 'JLINK_WEB' | 'DOWNLOAD_BIN';

export interface McuDefinition {
  id: string;
  name: string;
  family: 'STM32' | 'ESP32' | 'AVR' | 'RP2040' | 'NRF52' | 'PIC' | 'RISC-V' | 'OTHER';
  arch: string; // e.g. "Cortex-M4", "Xtensa LX6"
  flashMethod: FlashMethod;
  description: string;
  specs: {
    flashKB: number;
    ramKB: number;
    freqMHz: number;
    voltage: number;
  };
}