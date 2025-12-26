/**
 * useCanvasEngine - Hook for Rust Canvas Engine IPC
 * 
 * Provides a reactive interface to the high-performance Rust canvas engine
 */

import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

// Import centralized types
import type {
  NodeType,
  CanvasNode,
  CanvasEdge,
  CanvasState,
  NodeMove,
  NodeUpdate,
  DeleteResult,
  ValidationResult,
  BatchOperation,
  BatchResult,
  LayoutAlgorithm,
  Alignment,
  Port,
  Layer,
  NodeGroup,
  Viewport,
  ViewportTransform,
} from "../types/canvas";

// Re-export types for convenience
export type {
  NodeType,
  CanvasNode,
  CanvasEdge,
  CanvasState,
  NodeMove,
  NodeUpdate,
  DeleteResult,
  ValidationResult,
  BatchOperation,
  BatchResult,
  LayoutAlgorithm,
  Alignment,
  Port,
  Layer,
  NodeGroup,
  Viewport,
  ViewportTransform,
};

// Re-export helpers
export { getNodeCategory, isNodeTypeFsm, isNodeTypePeripheral, isNodeTypeRtos } from "../types/canvas";

/**
 * Canvas Engine Hook
 */
export function useCanvasEngine() {
  const [state, setState] = createSignal<CanvasState>({ nodes: [], edges: [], selection: [] });
  const [edgePaths, setEdgePaths] = createSignal<Record<string, string>>({});
  const [validation, setValidation] = createSignal<ValidationResult | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  // Initialize engine with state
  const init = async (nodes: CanvasNode[], edges: CanvasEdge[]) => {
    try {
      setLoading(true);
      await invoke("canvas_init", { nodes, edges });
      await refreshState();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  // Refresh state from Rust engine
  const refreshState = async () => {
    try {
      const newState = await invoke<CanvasState>("canvas_get_state");
      setState(newState);
      await refreshEdgePaths();
    } catch (e) {
      setError(String(e));
    }
  };

  // Refresh edge paths
  const refreshEdgePaths = async () => {
    try {
      const paths = await invoke<Record<string, string>>("canvas_get_edge_paths");
      setEdgePaths(paths);
    } catch (e) {
      console.error("Failed to get edge paths:", e);
    }
  };

  // Add a node
  const addNode = async (node: CanvasNode) => {
    try {
      await invoke("canvas_add_node", { node });
      await refreshState();
    } catch (e) {
      setError(String(e));
      throw e;
    }
  };

  // Move nodes (batch)
  const moveNodes = async (moves: NodeMove[]) => {
    try {
      await invoke("canvas_move_nodes", { moves });
      await refreshState();
    } catch (e) {
      setError(String(e));
    }
  };

  // Move single node
  const moveNode = async (id: string, x: number, y: number) => {
    await moveNodes([{ id, x, y }]);
  };

  // Delete nodes
  const deleteNodes = async (ids: string[]): Promise<DeleteResult | null> => {
    try {
      const result = await invoke<DeleteResult>("canvas_delete_nodes", { ids });
      await refreshState();
      return result;
    } catch (e) {
      setError(String(e));
      return null;
    }
  };

  // Update node
  const updateNode = async (id: string, update: NodeUpdate) => {
    try {
      await invoke("canvas_update_node", { id, update });
      await refreshState();
    } catch (e) {
      setError(String(e));
    }
  };

  // Connect nodes (with validation)
  const connect = async (source: string, target: string, label?: string): Promise<CanvasEdge | null> => {
    try {
      const edge = await invoke<CanvasEdge>("canvas_connect", { source, target, label });
      await refreshState();
      return edge;
    } catch (e) {
      setError(String(e));
      return null;
    }
  };

  // Delete edges
  const deleteEdges = async (ids: string[]) => {
    try {
      await invoke("canvas_delete_edges", { ids });
      await refreshState();
    } catch (e) {
      setError(String(e));
    }
  };

  // Query node at point
  const queryAt = async (x: number, y: number): Promise<CanvasNode | null> => {
    try {
      return await invoke<CanvasNode | null>("canvas_query_at", { x, y });
    } catch (e) {
      return null;
    }
  };

  // Query nodes in rectangle (marquee selection)
  const queryRect = async (x: number, y: number, width: number, height: number): Promise<string[]> => {
    try {
      return await invoke<string[]>("canvas_query_rect", { x, y, width, height });
    } catch (e) {
      return [];
    }
  };

  // Selection
  const select = async (ids: string[]) => {
    try {
      await invoke("canvas_select", { ids });
      await refreshState();
    } catch (e) {
      setError(String(e));
    }
  };

  const selectAll = async () => {
    try {
      await invoke("canvas_select_all");
      await refreshState();
    } catch (e) {
      setError(String(e));
    }
  };

  const clearSelection = async () => {
    try {
      await invoke("canvas_clear_selection");
      await refreshState();
    } catch (e) {
      setError(String(e));
    }
  };

  // Validate graph
  const validate = async (): Promise<ValidationResult | null> => {
    try {
      const result = await invoke<ValidationResult>("canvas_validate");
      setValidation(result);
      return result;
    } catch (e) {
      setError(String(e));
      return null;
    }
  };

  // Auto-layout
  const autoLayout = async (algorithm: LayoutAlgorithm = "hierarchical") => {
    try {
      setLoading(true);
      const newState = await invoke<CanvasState>("canvas_auto_layout", { algorithm });
      setState(newState);
      await refreshEdgePaths();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  // Align nodes
  const align = async (alignment: Alignment) => {
    try {
      const newState = await invoke<CanvasState>("canvas_align", { alignment });
      setState(newState);
      await refreshEdgePaths();
    } catch (e) {
      setError(String(e));
    }
  };

  // Undo/Redo
  const undo = async () => {
    try {
      const newState = await invoke<CanvasState>("canvas_undo");
      setState(newState);
      await refreshEdgePaths();
    } catch (e) {
      setError(String(e));
    }
  };

  const redo = async () => {
    try {
      const newState = await invoke<CanvasState>("canvas_redo");
      setState(newState);
      await refreshEdgePaths();
    } catch (e) {
      setError(String(e));
    }
  };

  // === Batch Operations ===
  const executeBatch = async (batch: BatchOperation): Promise<BatchResult | null> => {
    try {
      setLoading(true);
      const result = await invoke<BatchResult>("canvas_execute_batch", { batch });
      await refreshState();
      return result;
    } catch (e) {
      setError(String(e));
      return null;
    } finally {
      setLoading(false);
    }
  };

  // === Viewport Operations ===
  const getViewport = async (): Promise<Viewport | null> => {
    try {
      return await invoke<Viewport>("canvas_get_viewport");
    } catch (e) {
      return null;
    }
  };

  const setViewport = async (viewport: Partial<Viewport>) => {
    try {
      await invoke("canvas_set_viewport", { viewport });
    } catch (e) {
      setError(String(e));
    }
  };

  const zoomTo = async (zoom: number) => {
    try {
      await invoke("canvas_zoom_to", { zoom });
    } catch (e) {
      setError(String(e));
    }
  };

  const fitToContent = async (padding?: number) => {
    try {
      await invoke("canvas_fit_to_content", { padding: padding ?? 50 });
    } catch (e) {
      setError(String(e));
    }
  };

  // === Layer Operations ===
  const createLayer = async (name: string): Promise<string | null> => {
    try {
      const id = await invoke<string>("canvas_create_layer", { name });
      await refreshState();
      return id;
    } catch (e) {
      setError(String(e));
      return null;
    }
  };

  const deleteLayer = async (id: string) => {
    try {
      await invoke("canvas_delete_layer", { id });
      await refreshState();
    } catch (e) {
      setError(String(e));
    }
  };

  const setLayerVisible = async (id: string, visible: boolean) => {
    try {
      await invoke("canvas_set_layer_visible", { id, visible });
      await refreshState();
    } catch (e) {
      setError(String(e));
    }
  };

  const setLayerLocked = async (id: string, locked: boolean) => {
    try {
      await invoke("canvas_set_layer_locked", { id, locked });
      await refreshState();
    } catch (e) {
      setError(String(e));
    }
  };

  // === Group Operations ===
  const createGroup = async (nodeIds: string[], label: string): Promise<string | null> => {
    try {
      const id = await invoke<string>("canvas_create_group", { nodeIds, label });
      await refreshState();
      return id;
    } catch (e) {
      setError(String(e));
      return null;
    }
  };

  const ungroup = async (groupId: string) => {
    try {
      await invoke("canvas_ungroup", { groupId });
      await refreshState();
    } catch (e) {
      setError(String(e));
    }
  };

  const collapseGroup = async (groupId: string) => {
    try {
      await invoke("canvas_collapse_group", { groupId });
      await refreshState();
    } catch (e) {
      setError(String(e));
    }
  };

  const expandGroup = async (groupId: string) => {
    try {
      await invoke("canvas_expand_group", { groupId });
      await refreshState();
    } catch (e) {
      setError(String(e));
    }
  };

  return {
    // State
    state,
    edgePaths,
    validation,
    loading,
    error,
    
    // Lifecycle
    init,
    refreshState,
    
    // Node operations
    addNode,
    moveNode,
    moveNodes,
    deleteNodes,
    updateNode,
    
    // Edge operations
    connect,
    deleteEdges,
    
    // Queries
    queryAt,
    queryRect,
    
    // Selection
    select,
    selectAll,
    clearSelection,
    
    // Validation & Layout
    validate,
    autoLayout,
    align,
    
    // History
    undo,
    redo,

    // Batch
    executeBatch,

    // Viewport
    getViewport,
    setViewport,
    zoomTo,
    fitToContent,

    // Layers
    createLayer,
    deleteLayer,
    setLayerVisible,
    setLayerLocked,

    // Groups
    createGroup,
    ungroup,
    collapseGroup,
    expandGroup,
  };
}

export type CanvasEngine = ReturnType<typeof useCanvasEngine>;
