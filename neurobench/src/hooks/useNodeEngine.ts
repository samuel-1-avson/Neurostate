/**
 * useNodeEngine - Hook for Enhanced Node Engine IPC
 * 
 * Provides interface to the 40+ node type system
 */

import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

// Node categories
export type NodeCategory = "fsm" | "hardware" | "processing" | "control" | "io" | "data";

// Port types
export type PortType = "flow" | "digital" | "analog" | "integer" | "float" | "data" | "event" | { bus: { width: number } };

// Port direction
export type PortDirection = "input" | "output" | "bidirectional";

// Port definition
export interface PortDef {
  name: string;
  port_type: PortType;
  direction: PortDirection;
  required: boolean;
}

// Node ports
export interface NodePorts {
  inputs: PortDef[];
  outputs: PortDef[];
}

// Property value types
export type PropertyValue = 
  | string 
  | number 
  | boolean 
  | { value: string; options: string[] }
  | { port: string; pin: number };

// Node type info
export interface NodeTypeInfo {
  node_type: string;
  category: NodeCategory;
  name: string;
  icon: string;
  ports: NodePorts;
}

// Full node instance
export interface Node {
  id: string;
  label: string;
  node_type: string;
  x: number;
  y: number;
  width: number;
  height: number;
  ports: NodePorts;
  properties: Record<string, PropertyValue>;
  entry_action?: string;
  exit_action?: string;
  description?: string;
}

// Connection
export interface Connection {
  id: string;
  source_node: string;
  source_port: string;
  target_node: string;
  target_port: string;
  label?: string;
  condition?: string;
}

// Design state
export interface DesignState {
  nodes: Node[];
  connections: Connection[];
}

// Validation
export interface ValidationResult {
  valid: boolean;
  errors: Array<{ code: string; message: string; node_ids: string[] }>;
  warnings: Array<{ code: string; message: string; node_ids: string[] }>;
}

// Category info for UI
export const CATEGORY_INFO: Record<NodeCategory, { name: string; icon: string; color: string }> = {
  fsm: { name: "State Machine", icon: "🔀", color: "#4CAF50" },
  hardware: { name: "Hardware", icon: "🔧", color: "#2196F3" },
  processing: { name: "Processing", icon: "⚙️", color: "#FF9800" },
  control: { name: "Control/RTOS", icon: "📋", color: "#9C27B0" },
  io: { name: "I/O Devices", icon: "📡", color: "#E91E63" },
  data: { name: "Data", icon: "📦", color: "#607D8B" },
};

// Port type colors
export const PORT_COLORS: Record<string, string> = {
  flow: "#4CAF50",
  digital: "#2196F3",
  analog: "#FF9800",
  integer: "#9C27B0",
  float: "#E91E63",
  data: "#607D8B",
  event: "#F44336",
  bus: "#795548",
};

/**
 * Node Engine Hook
 */
export function useNodeEngine() {
  const [palette, setPalette] = createSignal<[string, NodeTypeInfo[]][]>([]);
  const [allTypes, setAllTypes] = createSignal<NodeTypeInfo[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  // Load node palette (grouped by category)
  const loadPalette = async () => {
    try {
      setLoading(true);
      const result = await invoke<[string, NodeTypeInfo[]][]>("nodes_get_palette");
      setPalette(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  // Load all node types
  const loadAllTypes = async () => {
    try {
      const result = await invoke<NodeTypeInfo[]>("nodes_get_all_types");
      setAllTypes(result);
    } catch (e) {
      setError(String(e));
    }
  };

  // Get type info for specific node type
  const getTypeInfo = async (nodeType: string): Promise<NodeTypeInfo | null> => {
    try {
      return await invoke<NodeTypeInfo>("nodes_get_type_info", { nodeType });
    } catch (e) {
      setError(String(e));
      return null;
    }
  };

  // Create a new node
  const createNode = async (
    nodeType: string, 
    x: number, 
    y: number, 
    label?: string
  ): Promise<Node | null> => {
    try {
      return await invoke<Node>("nodes_create", { nodeType, x, y, label });
    } catch (e) {
      setError(String(e));
      return null;
    }
  };

  // Delete a node
  const deleteNode = async (nodeId: string): Promise<boolean> => {
    try {
      return await invoke<boolean>("nodes_delete", { nodeId });
    } catch (e) {
      setError(String(e));
      return false;
    }
  };

  // Connect two nodes
  const connect = async (
    sourceNode: string,
    sourcePort: string,
    targetNode: string,
    targetPort: string
  ): Promise<string | null> => {
    try {
      return await invoke<string>("nodes_connect", { 
        sourceNode, sourcePort, targetNode, targetPort 
      });
    } catch (e) {
      setError(String(e));
      return null;
    }
  };

  // Disconnect
  const disconnect = async (connectionId: string): Promise<boolean> => {
    try {
      return await invoke<boolean>("nodes_disconnect", { connectionId });
    } catch (e) {
      setError(String(e));
      return false;
    }
  };

  // Update property
  const updateProperty = async (
    nodeId: string,
    key: string,
    value: PropertyValue
  ): Promise<void> => {
    try {
      await invoke("nodes_update_property", { nodeId, key, value });
    } catch (e) {
      setError(String(e));
    }
  };

  // Validate design
  const validate = async (): Promise<ValidationResult | null> => {
    try {
      return await invoke<ValidationResult>("nodes_validate");
    } catch (e) {
      setError(String(e));
      return null;
    }
  };

  // Export design
  const exportDesign = async (): Promise<DesignState | null> => {
    try {
      return await invoke<DesignState>("nodes_export");
    } catch (e) {
      setError(String(e));
      return null;
    }
  };

  // Import design
  const importDesign = async (state: DesignState): Promise<void> => {
    try {
      await invoke("nodes_import", { state });
    } catch (e) {
      setError(String(e));
    }
  };

  return {
    // State
    palette,
    allTypes,
    loading,
    error,

    // Load
    loadPalette,
    loadAllTypes,
    getTypeInfo,

    // CRUD
    createNode,
    deleteNode,
    connect,
    disconnect,
    updateProperty,

    // Design
    validate,
    exportDesign,
    importDesign,
  };
}

export type NodeEngineHook = ReturnType<typeof useNodeEngine>;
