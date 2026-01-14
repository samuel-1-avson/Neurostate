/**
 * useTransitionEngine - Enhanced Hook for Transition Engine IPC
 * 
 * Provides comprehensive interface to the Rust transition engine for:
 * - Creating transitions between nodes
 * - Updating transition properties (guards, actions, events)
 * - Deleting transitions
 * - Querying transitions by node
 * 
 * Supports 40+ embedded system event types across categories:
 * - Hardware: GPIO, ADC, PWM, DAC, DMA
 * - Timer: Timeout, Periodic, Watchdog, RTC
 * - Communication: UART, SPI, I2C, CAN, Ethernet, USB, Bluetooth
 * - System: Power, Interrupt, Error, Boot
 * - FSM: Signal, Condition, Immediate
 */

import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

// =============================================================================
// EVENT TYPE DEFINITIONS
// =============================================================================

// GPIO Event Types
export interface GpioEventConfig {
  pin: string;                    // GPIO pin identifier (e.g., "PA0", "PB5")
  edge: "rising" | "falling" | "both" | "level_high" | "level_low";
  debounce_ms?: number;           // Optional debounce time
  pull?: "up" | "down" | "none";  // Pull resistor config
}

// ADC Event Types  
export interface AdcEventConfig {
  channel: string;                   // ADC channel identifier
  threshold_type: "above" | "below" | "within_range" | "outside_range" | "change";
  threshold_value?: number;          // Single threshold or lower bound
  threshold_high?: number;           // Upper bound for range
  hysteresis?: number;               // Prevent oscillation
  sample_rate_hz?: number;           // Sampling frequency
}

// PWM Event Types
export interface PwmEventConfig {
  channel: string;                   // PWM channel
  event: "cycle_complete" | "duty_reached" | "period_start" | "period_end";
  duty_threshold?: number;           // For duty_reached event
}

// Timer Event Types
export interface TimerEventConfig {
  timer_id?: string;                 // Optional timer identifier
  duration_ms: number;               // Duration in milliseconds
  mode?: "oneshot" | "periodic" | "countdown";
  auto_restart?: boolean;
}

// Watchdog Event Types
export interface WatchdogEventConfig {
  level: "warning" | "imminent" | "reset";
  remaining_ms?: number;
}

// RTC Event Types
export interface RtcEventConfig {
  alarm_id?: string;
  type: "alarm" | "periodic" | "wakeup";
  time?: string;                     // ISO time string for alarm
  interval_seconds?: number;         // For periodic wakeup
}

// UART Event Types
export interface UartEventConfig {
  port: string;                      // UART port identifier
  event: "rx_complete" | "tx_complete" | "rx_idle" | "rx_timeout" | "error" | "pattern_match";
  pattern?: string;                  // Pattern to match (for pattern_match)
  timeout_ms?: number;               // For rx_timeout
  min_bytes?: number;                // Minimum bytes before trigger
}

// SPI Event Types
export interface SpiEventConfig {
  bus: string;                       // SPI bus identifier
  event: "transfer_complete" | "rx_ready" | "tx_empty" | "error";
  chip_select?: string;              // CS pin
}

// I2C Event Types
export interface I2cEventConfig {
  bus: string;                       // I2C bus identifier
  event: "transfer_complete" | "nack" | "error" | "arbitration_lost";
  address?: number;                  // I2C address
}

// CAN Event Types
export interface CanEventConfig {
  bus: string;                       // CAN bus identifier
  event: "message_received" | "message_sent" | "error" | "bus_off" | "wakeup";
  id_filter?: number;                // CAN ID to filter
  id_mask?: number;                  // Mask for filtering
  is_extended?: boolean;             // Extended frame format
}

// Ethernet Event Types
export interface EthernetEventConfig {
  interface: string;                 // Network interface
  event: "link_up" | "link_down" | "packet_received" | "packet_sent" | "error";
  port?: number;                     // UDP/TCP port filter
  protocol?: "tcp" | "udp" | "icmp" | "any";
}

// USB Event Types
export interface UsbEventConfig {
  event: "connected" | "disconnected" | "suspended" | "resumed" | "data_received" | "data_sent" | "error";
  endpoint?: number;                 // USB endpoint
  interface?: number;                // USB interface number
}

// Bluetooth Event Types
export interface BluetoothEventConfig {
  event: "connected" | "disconnected" | "paired" | "data_received" | "data_sent" | "scan_result" | "error";
  device_address?: string;           // BLE device address
  service_uuid?: string;             // GATT service UUID
  characteristic_uuid?: string;      // GATT characteristic UUID
}

// WiFi Event Types
export interface WifiEventConfig {
  event: "connected" | "disconnected" | "scan_complete" | "ip_acquired" | "error";
  ssid?: string;                     // Network SSID
}

// Power Event Types
export interface PowerEventConfig {
  event: "sleep_enter" | "sleep_exit" | "standby_enter" | "standby_exit" | "low_battery" | "critical_battery" | "charging" | "charged";
  battery_level?: number;            // Battery threshold percentage
}

// Interrupt Event Types
export interface InterruptEventConfig {
  irq: number | string;              // IRQ number or name
  priority?: number;                 // Interrupt priority
  edge?: "rising" | "falling" | "both";
}

// DMA Event Types
export interface DmaEventConfig {
  channel: string;                   // DMA channel
  event: "transfer_complete" | "half_transfer" | "error";
  size?: number;                     // Expected transfer size
}

// Sensor Event Types
export interface SensorEventConfig {
  sensor_id: string;                 // Sensor identifier
  type: "threshold" | "change" | "sample_ready";
  threshold?: number;
  comparison?: "gt" | "lt" | "eq" | "ge" | "le";
}

// System Event Types
export interface SystemEventConfig {
  event: "boot_complete" | "shutdown" | "fault" | "reset" | "memory_low" | "stack_overflow" | "watchdog_reset";
  fault_code?: number;
}

// Motor Event Types
export interface MotorEventConfig {
  motor_id: string;
  event: "target_reached" | "stall_detected" | "overcurrent" | "limit_switch" | "home_found";
  position?: number;                 // Target position
}

// FSM Event Types (for state machine logic)
export interface FsmSignalConfig {
  signal: string;                    // Signal name
}

export interface FsmConditionConfig {
  expression: string;                // Boolean expression
  variables?: Record<string, string>; // Variable bindings
}

// =============================================================================
// UNIFIED TRANSITION EVENT TYPE
// =============================================================================

export type TransitionEventType = 
  // Hardware Events
  | { type: "gpio"; config: GpioEventConfig }
  | { type: "adc"; config: AdcEventConfig }
  | { type: "pwm"; config: PwmEventConfig }
  | { type: "dma"; config: DmaEventConfig }
  
  // Timer Events
  | { type: "timer"; config: TimerEventConfig }
  | { type: "watchdog"; config: WatchdogEventConfig }
  | { type: "rtc"; config: RtcEventConfig }
  
  // Communication Events
  | { type: "uart"; config: UartEventConfig }
  | { type: "spi"; config: SpiEventConfig }
  | { type: "i2c"; config: I2cEventConfig }
  | { type: "can"; config: CanEventConfig }
  | { type: "ethernet"; config: EthernetEventConfig }
  | { type: "usb"; config: UsbEventConfig }
  | { type: "bluetooth"; config: BluetoothEventConfig }
  | { type: "wifi"; config: WifiEventConfig }
  
  // System Events
  | { type: "power"; config: PowerEventConfig }
  | { type: "interrupt"; config: InterruptEventConfig }
  | { type: "system"; config: SystemEventConfig }
  | { type: "sensor"; config: SensorEventConfig }
  | { type: "motor"; config: MotorEventConfig }
  
  // FSM Events
  | { type: "signal"; config: FsmSignalConfig }
  | { type: "condition"; config: FsmConditionConfig }
  | { type: "immediate" }
  
  // Legacy/Custom Events
  | { type: "custom"; config: { name: string; data?: Record<string, unknown> } };

// Event type categories for UI grouping
export const EVENT_CATEGORIES = {
  hardware: ["gpio", "adc", "pwm", "dma"],
  timer: ["timer", "watchdog", "rtc"],
  communication: ["uart", "spi", "i2c", "can", "ethernet", "usb", "bluetooth", "wifi"],
  system: ["power", "interrupt", "system", "sensor", "motor"],
  fsm: ["signal", "condition", "immediate"],
  custom: ["custom"],
} as const;

// Human readable event type names
export const EVENT_TYPE_NAMES: Record<string, string> = {
  gpio: "GPIO Pin Change",
  adc: "ADC Threshold",
  pwm: "PWM Event",
  dma: "DMA Transfer",
  timer: "Timer Event",
  watchdog: "Watchdog",
  rtc: "RTC Alarm",
  uart: "UART Data",
  spi: "SPI Transfer",
  i2c: "I2C Transaction",
  can: "CAN Message",
  ethernet: "Ethernet Packet",
  usb: "USB Event",
  bluetooth: "Bluetooth Event",
  wifi: "WiFi Event",
  power: "Power State",
  interrupt: "Hardware Interrupt",
  system: "System Event",
  sensor: "Sensor Reading",
  motor: "Motor Event",
  signal: "Signal",
  condition: "Condition",
  immediate: "Immediate",
  custom: "Custom Event",
};

// Event type icons (emoji or text)
export const EVENT_TYPE_ICONS: Record<string, string> = {
  gpio: "⚡",
  adc: "📊",
  pwm: "〰️",
  dma: "↔️",
  timer: "⏱️",
  watchdog: "🐕",
  rtc: "⏰",
  uart: "📡",
  spi: "🔄",
  i2c: "🔗",
  can: "🚗",
  ethernet: "🌐",
  usb: "🔌",
  bluetooth: "📶",
  wifi: "📻",
  power: "🔋",
  interrupt: "⚠️",
  system: "⚙️",
  sensor: "📏",
  motor: "🔧",
  signal: "📨",
  condition: "❓",
  immediate: "⚡",
  custom: "🔧",
};

// Event type colors for edge visualization
export const EVENT_TYPE_COLORS: Record<string, string> = {
  gpio: "#FF6B6B",      // Red - hardware I/O
  adc: "#FFE66D",       // Yellow - analog
  pwm: "#FF9F43",       // Orange - PWM
  dma: "#A29BFE",       // Purple - DMA
  timer: "#00B894",     // Green - timer
  watchdog: "#D63031",  // Dark red - watchdog
  rtc: "#00CEC9",       // Teal - RTC
  uart: "#6C5CE7",      // Purple - UART
  spi: "#0984E3",       // Blue - SPI
  i2c: "#00B894",       // Green - I2C
  can: "#FDCB6E",       // Yellow - CAN
  ethernet: "#74B9FF",  // Light blue - Ethernet
  usb: "#FD79A8",       // Pink - USB
  bluetooth: "#0984E3", // Blue - Bluetooth
  wifi: "#00B894",      // Green - WiFi
  power: "#E17055",     // Coral - power
  interrupt: "#D63031", // Red - interrupt
  system: "#636E72",    // Gray - system
  sensor: "#FDCB6E",    // Yellow - sensor
  motor: "#E17055",     // Coral - motor
  signal: "#4CAF50",    // Green - signal (default)
  condition: "#9B59B6", // Purple - condition
  immediate: "#3498DB", // Blue - immediate
  custom: "#95A5A6",    // Gray - custom
};

// =============================================================================
// TRANSITION TIMING CONFIGURATION
// =============================================================================

export interface TransitionTiming {
  min_duration_ms?: number;          // Minimum time in source state
  max_duration_ms?: number;          // Maximum time before auto-transition
  timeout_action?: "transition" | "error" | "retry" | "ignore";
  retry_count?: number;              // Retries before giving up
  retry_delay_ms?: number;           // Delay between retries
}

// =============================================================================
// TRANSITION ACTIONS
// =============================================================================

export interface TransitionAction {
  id?: string;                       // Unique action identifier
  type: "code" | "function" | "signal" | "set_variable" | "log" | "delay";
  code?: string;                     // For type "code"
  function_name?: string;            // For type "function"
  signal_name?: string;              // For type "signal"
  variable?: string;                 // For type "set_variable"
  value?: unknown;                   // Value to set
  message?: string;                  // For type "log"
  delay_ms?: number;                 // For type "delay"
}

export interface TransitionActions {
  on_exit?: TransitionAction[];      // Actions before leaving source state
  on_transit?: TransitionAction[];   // Actions during transition
  on_enter?: TransitionAction[];     // Actions after entering target state
}

// =============================================================================
// GUARD EXPRESSION
// =============================================================================

export interface TransitionGuard {
  expression: string;                // Boolean expression
  language?: "c" | "python" | "javascript" | "expression";
  description?: string;              // Human readable description
  variables?: string[];              // Required variables
}

// =============================================================================
// CONNECTION POINT & WAYPOINT
// =============================================================================

export interface ConnectionPoint {
  node_id: string;
  port_name?: string;
  port_index?: number;
}

export interface Waypoint {
  x: number;
  y: number;
}

// =============================================================================
// TRANSITION INTERFACE
// =============================================================================

export interface Transition {
  id: string;
  source: ConnectionPoint;
  target: ConnectionPoint;
  
  // Basic properties
  label?: string;
  description?: string;
  
  // Event trigger
  event?: TransitionEventType;
  
  // Guard condition
  guard?: TransitionGuard;
  
  // Actions
  actions?: TransitionActions;
  
  // Timing
  timing?: TransitionTiming;
  
  // Priority & ordering
  priority: number;
  is_default: boolean;
  
  // State
  enabled: boolean;
  
  // Visual
  waypoints: Waypoint[];
  color?: string;
  style?: "solid" | "dashed" | "dotted";
  
  // Metadata
  created_at?: string;
  updated_at?: string;
  tags?: string[];
}

// Lighter weight for lists
export interface TransitionInfo {
  id: string;
  source_node: string;
  source_port?: string;
  target_node: string;
  target_port?: string;
  label?: string;
  event_type?: string;
  has_guard: boolean;
  has_action: boolean;
  priority: number;
  enabled: boolean;
}

// Update payload
export interface TransitionUpdate {
  label?: string;
  description?: string;
  event?: TransitionEventType;
  guard?: TransitionGuard;
  actions?: TransitionActions;
  timing?: TransitionTiming;
  priority?: number;
  is_default?: boolean;
  enabled?: boolean;
  waypoints?: Waypoint[];
  color?: string;
  style?: "solid" | "dashed" | "dotted";
  tags?: string[];
}

// Validation result
export interface ValidationResult {
  valid: boolean;
  errors: Array<{ code: string; message: string; transition_id?: string }>;
  warnings: Array<{ code: string; message: string; transition_id?: string }>;
  transition_count: number;
}

// =============================================================================
// TRANSITION ENGINE HOOK
// =============================================================================

export function useTransitionEngine() {
  const [transitions, setTransitions] = createSignal<Transition[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  // Create a new transition
  const create = async (
    sourceNode: string,
    targetNode: string,
    sourcePort?: string,
    targetPort?: string,
  ): Promise<Transition | null> => {
    try {
      setLoading(true);
      const transition = await invoke<Transition>("transitions_create", {
        sourceNode,
        targetNode,
        sourcePort,
        targetPort,
      });
      await refresh();
      return transition;
    } catch (e) {
      setError(String(e));
      return null;
    } finally {
      setLoading(false);
    }
  };

  // Update a transition
  const update = async (id: string, updateData: TransitionUpdate): Promise<Transition | null> => {
    try {
      setLoading(true);
      const transition = await invoke<Transition>("transitions_update", { id, update: updateData });
      await refresh();
      return transition;
    } catch (e) {
      setError(String(e));
      return null;
    } finally {
      setLoading(false);
    }
  };

  // Delete a transition
  const remove = async (id: string): Promise<boolean> => {
    try {
      setLoading(true);
      await invoke<Transition>("transitions_delete", { id });
      await refresh();
      return true;
    } catch (e) {
      setError(String(e));
      return false;
    } finally {
      setLoading(false);
    }
  };

  // Get a single transition
  const get = async (id: string): Promise<Transition | null> => {
    try {
      return await invoke<Transition | null>("transitions_get", { id });
    } catch (e) {
      setError(String(e));
      return null;
    }
  };

  // Get all transitions
  const getAll = async (): Promise<Transition[]> => {
    try {
      return await invoke<Transition[]>("transitions_get_all");
    } catch (e) {
      setError(String(e));
      return [];
    }
  };

  // Get transitions by node
  const getByNode = async (nodeId: string): Promise<TransitionInfo[]> => {
    try {
      return await invoke<TransitionInfo[]>("transitions_get_by_node", { nodeId });
    } catch (e) {
      setError(String(e));
      return [];
    }
  };

  // Validate all transitions
  const validate = async (): Promise<ValidationResult> => {
    try {
      return await invoke<ValidationResult>("transitions_validate");
    } catch (e) {
      setError(String(e));
      return { valid: false, errors: [], warnings: [], transition_count: 0 };
    }
  };

  // Clear all transitions
  const clear = async (): Promise<void> => {
    try {
      setLoading(true);
      await invoke("transitions_clear");
      setTransitions([]);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  // Refresh transitions from backend
  const refresh = async (): Promise<void> => {
    const all = await getAll();
    setTransitions(all);
  };

  // Quick connect helper
  const connect = async (sourceNode: string, targetNode: string): Promise<Transition | null> => {
    return create(sourceNode, targetNode);
  };

  // Disconnect (delete by source/target)
  const disconnect = async (sourceNode: string, targetNode: string): Promise<void> => {
    const all = transitions();
    const toRemove = all.filter(
      t => t.source.node_id === sourceNode && t.target.node_id === targetNode
    );
    for (const t of toRemove) {
      await remove(t.id);
    }
  };

  // Set transition event
  const setEvent = async (id: string, event: TransitionEventType): Promise<Transition | null> => {
    return update(id, { event });
  };

  // Set transition guard
  const setGuard = async (id: string, guard: TransitionGuard): Promise<Transition | null> => {
    return update(id, { guard });
  };

  // Set transition actions
  const setActions = async (id: string, actions: TransitionActions): Promise<Transition | null> => {
    return update(id, { actions });
  };

  // Set transition timing
  const setTiming = async (id: string, timing: TransitionTiming): Promise<Transition | null> => {
    return update(id, { timing });
  };

  // Enable/disable transition
  const setEnabled = async (id: string, enabled: boolean): Promise<Transition | null> => {
    return update(id, { enabled });
  };

  // Set priority
  const setPriority = async (id: string, priority: number): Promise<Transition | null> => {
    return update(id, { priority });
  };

  // Get transitions by event type
  const getByEventType = (eventType: string): Transition[] => {
    return transitions().filter(t => t.event?.type === eventType);
  };

  // Get enabled transitions only
  const getEnabled = (): Transition[] => {
    return transitions().filter(t => t.enabled);
  };

  // Get default transitions
  const getDefaults = (): Transition[] => {
    return transitions().filter(t => t.is_default);
  };

  return {
    // State
    transitions,
    loading,
    error,
    
    // CRUD
    create,
    update,
    remove,
    get,
    getAll,
    getByNode,
    
    // Helpers
    connect,
    disconnect,
    validate,
    clear,
    refresh,
    
    // Property setters
    setEvent,
    setGuard,
    setActions,
    setTiming,
    setEnabled,
    setPriority,
    
    // Query helpers
    getByEventType,
    getEnabled,
    getDefaults,
    
    // Constants
    EVENT_CATEGORIES,
    EVENT_TYPE_NAMES,
  };
}

export default useTransitionEngine;
