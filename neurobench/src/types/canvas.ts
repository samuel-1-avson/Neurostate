/**
 * Canvas Types - Complete type definitions matching Rust backend
 * 
 * Auto-sync with src-tauri/src/canvas/types.rs
 */

// =============================================================================
// NODE TYPES (55+ types across 11 categories)
// =============================================================================

export type NodeCategory = 
  | "fsm" 
  | "gpio" 
  | "communication" 
  | "timer" 
  | "rtos" 
  | "analog" 
  | "motor" 
  | "system" 
  | "sensor" 
  | "wireless" 
  | "general"
  | "custom";

// FSM Core
export type FsmNodeType = 
  | "initial" 
  | "state" 
  | "final" 
  | "choice" 
  | "fork" 
  | "join" 
  | "history" 
  | "deep_history";

// GPIO & Digital I/O
export type GpioNodeType = 
  | "gpio" 
  | "digital_input" 
  | "digital_output" 
  | "exti" 
  | "button" 
  | "led" 
  | "relay";

// Communication Interfaces
export type CommunicationNodeType = 
  | "uart" 
  | "spi" 
  | "i2c" 
  | "can" 
  | "usb" 
  | "ethernet" 
  | "rs485" 
  | "modbus";

// Timers & Delays
export type TimerNodeType = 
  | "timer" 
  | "delay" 
  | "pwm" 
  | "input_capture" 
  | "output_compare" 
  | "rtc" 
  | "watchdog";

// RTOS Components
export type RtosNodeType = 
  | "task" 
  | "semaphore" 
  | "mutex" 
  | "queue" 
  | "event_group" 
  | "sw_timer" 
  | "mem_pool";

// Analog
export type AnalogNodeType = 
  | "adc" 
  | "dac" 
  | "comparator" 
  | "op_amp";

// Motor Control
export type MotorNodeType = 
  | "motor_dc" 
  | "motor_stepper" 
  | "motor_servo" 
  | "h_bridge" 
  | "encoder";

// Power & System
export type SystemNodeType = 
  | "power_mode" 
  | "clock" 
  | "dma" 
  | "flash" 
  | "reset";

// Sensors
export type SensorNodeType = 
  | "sensor_temp" 
  | "sensor_imu" 
  | "sensor_proximity" 
  | "sensor";

// Wireless
export type WirelessNodeType = 
  | "wifi" 
  | "bluetooth" 
  | "lora" 
  | "zigbee";

// Legacy (backward compatibility)
export type LegacyNodeType = 
  | "input" 
  | "output" 
  | "process" 
  | "decision" 
  | "error" 
  | "hardware" 
  | "interrupt";

// Custom node type
export interface CustomNodeType {
  category: string;
  icon?: string;
}

// Complete NodeType union
export type NodeType = 
  | FsmNodeType 
  | GpioNodeType 
  | CommunicationNodeType 
  | TimerNodeType 
  | RtosNodeType 
  | AnalogNodeType 
  | MotorNodeType 
  | SystemNodeType 
  | SensorNodeType 
  | WirelessNodeType 
  | LegacyNodeType
  | { custom: CustomNodeType };

// =============================================================================
// PORT TYPES
// =============================================================================

export type PortDirection = "input" | "output" | "bidirectional";

export type PortDataType = 
  | "digital" 
  | "analog" 
  | "serial" 
  | "parallel" 
  | "event" 
  | "integer" 
  | "float" 
  | "boolean" 
  | "string" 
  | "buffer" 
  | "pwm" 
  | "clock" 
  | "power" 
  | "ground" 
  | "any"
  | { custom: string };

export type PortPosition = "top" | "bottom" | "left" | "right";

export interface Port {
  id: string;
  name: string;
  direction: PortDirection;
  data_type: PortDataType;
  position: PortPosition;
  offset: number;
  required: boolean;
  max_connections: number;
  description?: string;
}

// =============================================================================
// EDGE TYPES
// =============================================================================

export type EdgeRoutingStyle = "bezier" | "orthogonal" | "orthogonal_rounded" | "straight";

export interface EdgeWaypoint {
  x: number;
  y: number;
}

export interface CanvasEdge {
  id: string;
  source: string;
  target: string;
  label?: string;
  condition?: string;
  waypoints: EdgeWaypoint[];
  routing: EdgeRoutingStyle;
  z_order: number;
  source_port?: string;
  target_port?: string;
}

// =============================================================================
// NODE TYPES
// =============================================================================

export interface CanvasNode {
  id: string;
  label: string;
  node_type: NodeType;
  x: number;
  y: number;
  width: number;
  height: number;
  entry_action?: string;
  exit_action?: string;
  description?: string;
  ports?: Port[];
  group_id?: string;
  layer_id?: string;
  metadata?: NodeMetadata;
}

export interface NodeMetadata {
  created_at?: string;
  updated_at?: string;
  author?: string;
  version?: string;
  tags?: string[];
}

// =============================================================================
// LAYERS & GROUPS
// =============================================================================

export interface Layer {
  id: string;
  name: string;
  visible: boolean;
  locked: boolean;
  opacity: number;
  color?: string;
  z_index: number;
}

export interface NodeGroup {
  id: string;
  label: string;
  children: string[];
  child_groups: string[];
  parent?: string;
  collapsed: boolean;
  color?: string;
}

// =============================================================================
// VALIDATION
// =============================================================================

export type IssueSeverity = "error" | "warning" | "info" | "hint";
export type IssueCategory = 
  | "fsm_structure" 
  | "connectivity" 
  | "resource" 
  | "naming" 
  | "performance" 
  | "rtos" 
  | "dead_code" 
  | "timing";

export interface ValidationIssue {
  id: string;
  severity: IssueSeverity;
  category: IssueCategory;
  node_ids: string[];
  edge_ids: string[];
  message: string;
  description?: string;
  suggestion?: string;
  fixable: boolean;
}

export interface ValidationResult {
  issues: ValidationIssue[];
  valid: boolean;
  error_count: number;
  warning_count: number;
}

// =============================================================================
// CANVAS STATE
// =============================================================================

export interface CanvasState {
  nodes: CanvasNode[];
  edges: CanvasEdge[];
  selection: string[];
  layers?: Layer[];
  groups?: NodeGroup[];
}

// =============================================================================
// OPERATIONS
// =============================================================================

export interface NodeMove {
  id: string;
  x: number;
  y: number;
}

export interface NodeUpdate {
  label?: string;
  node_type?: NodeType;
  entry_action?: string | null;
  exit_action?: string | null;
  description?: string | null;
  width?: number;
  height?: number;
}

export interface DeleteResult {
  deleted_nodes: string[];
  deleted_edges: string[];
}

export interface BatchOperation {
  add_nodes?: CanvasNode[];
  delete_node_ids?: string[];
  add_edges?: { source: string; target: string; label?: string }[];
  delete_edge_ids?: string[];
  move_nodes?: NodeMove[];
  update_nodes?: { id: string; update: NodeUpdate }[];
}

export interface BatchResult {
  added_nodes: string[];
  added_edges: string[];
  deleted_nodes: string[];
  deleted_edges: string[];
  moved_nodes: string[];
  updated_nodes: string[];
  errors: string[];
}

// =============================================================================
// LAYOUT
// =============================================================================

export type LayoutAlgorithm = 
  | "force_directed" 
  | "hierarchical" 
  | "tree" 
  | "grid" 
  | "circular" 
  | "radial";

export type Alignment = 
  | "left" 
  | "right" 
  | "top" 
  | "bottom" 
  | "center_horizontal" 
  | "center_vertical" 
  | "distribute_horizontal" 
  | "distribute_vertical";

// =============================================================================
// VIEWPORT
// =============================================================================

export interface Viewport {
  center_x: number;
  center_y: number;
  zoom: number;
  width: number;
  height: number;
}

export interface ViewportTransform {
  translate_x: number;
  translate_y: number;
  scale: number;
}

// =============================================================================
// HELPERS
// =============================================================================

export const NODE_CATEGORIES: Record<NodeCategory, string[]> = {
  fsm: ["initial", "state", "final", "choice", "fork", "join", "history", "deep_history"],
  gpio: ["gpio", "digital_input", "digital_output", "exti", "button", "led", "relay"],
  communication: ["uart", "spi", "i2c", "can", "usb", "ethernet", "rs485", "modbus"],
  timer: ["timer", "delay", "pwm", "input_capture", "output_compare", "rtc", "watchdog"],
  rtos: ["task", "semaphore", "mutex", "queue", "event_group", "sw_timer", "mem_pool"],
  analog: ["adc", "dac", "comparator", "op_amp"],
  motor: ["motor_dc", "motor_stepper", "motor_servo", "h_bridge", "encoder"],
  system: ["power_mode", "clock", "dma", "flash", "reset"],
  sensor: ["sensor_temp", "sensor_imu", "sensor_proximity", "sensor"],
  wireless: ["wifi", "bluetooth", "lora", "zigbee"],
  general: ["input", "output", "process", "decision", "error", "hardware", "interrupt"],
  custom: [],
};

export function getNodeCategory(nodeType: NodeType): NodeCategory {
  if (typeof nodeType === "object" && "custom" in nodeType) return "custom";
  
  for (const [category, types] of Object.entries(NODE_CATEGORIES)) {
    if (types.includes(nodeType as string)) {
      return category as NodeCategory;
    }
  }
  return "general";
}

export function isNodeTypeFsm(nodeType: NodeType): boolean {
  return getNodeCategory(nodeType) === "fsm";
}

export function isNodeTypePeripheral(nodeType: NodeType): boolean {
  const cat = getNodeCategory(nodeType);
  return ["gpio", "communication", "timer", "analog", "motor", "sensor", "wireless"].includes(cat);
}

export function isNodeTypeRtos(nodeType: NodeType): boolean {
  return getNodeCategory(nodeType) === "rtos";
}
