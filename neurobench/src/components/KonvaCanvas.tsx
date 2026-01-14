/**
 * KonvaCanvas - High-performance canvas using Konva.js
 * 
 * Features:
 * - Native drag-and-drop (no HTML5 drag events)
 * - Hardware-accelerated rendering
 * - Built-in zoom and pan
 * - Port-based connections with bezier curves
 */

import { Component, createSignal, onMount, onCleanup, createEffect, For, Show } from 'solid-js';
import Konva from 'konva';
import { Icons } from './AppIcons';
import { useCanvasEngine } from '../hooks/useCanvasEngine';
import { useNodeEngine, NodeTypeInfo, CATEGORY_INFO } from '../hooks/useNodeEngine';
import { EVENT_TYPE_COLORS, EVENT_TYPE_ICONS } from '../hooks/useTransitionEngine';
import type { CanvasNode, CanvasEdge, NodeType } from '../types/canvas';
import TransitionEditor, { TransitionData } from './TransitionEditor';
import NodePropertiesPanel, { NodeData } from './NodePropertiesPanel';
import ContextMenu, { MenuItem } from './ContextMenu';
import './KonvaCanvas.css';

// Constants
const GRID_SIZE = 20;
const MIN_ZOOM = 0.1;
const MAX_ZOOM = 3;
const NODE_COLORS: Record<string, string> = {
  state: '#4CAF50',
  initial: '#2196F3',
  final: '#9C27B0',
  decision: '#FF9800',
  junction: '#00BCD4',
  gpio: '#607D8B',
  timer: '#795548',
  uart: '#3F51B5',
  adc: '#E91E63',
  default: '#4CAF50',
};

// Context menu types
type ContextMenuType = 'canvas' | 'node' | 'edge' | null;

const KonvaCanvas: Component = () => {
  // Refs
  let containerRef: HTMLDivElement | undefined;
  let stage: Konva.Stage | null = null;
  let layer: Konva.Layer | null = null;
  let edgeLayer: Konva.Layer | null = null;
  let gridLayer: Konva.Layer | null = null;

  // State
  const [zoom, setZoom] = createSignal(1);
  const [pan, setPan] = createSignal({ x: 0, y: 0 });
  const [showPalette, setShowPalette] = createSignal(true);
  const [selectedNodeId, setSelectedNodeId] = createSignal<string | null>(null);
  const [selectedEdgeId, setSelectedEdgeId] = createSignal<string | null>(null);
  const [showNodePanel, setShowNodePanel] = createSignal(false);
  const [showEdgePanel, setShowEdgePanel] = createSignal(false);
  const [connecting, setConnecting] = createSignal<{ nodeId: string; isOutput: boolean } | null>(null);
  const [connectionLine, setConnectionLine] = createSignal<Konva.Line | null>(null);
  const [draggedPaletteNode, setDraggedPaletteNode] = createSignal<NodeTypeInfo | null>(null);
  const [ghostNode, setGhostNode] = createSignal<Konva.Group | null>(null);
  const [paletteWidth, setPaletteWidth] = createSignal(260); // Default wider
  const [isResizingPalette, setIsResizingPalette] = createSignal(false);
  
  // Context menu state
  const [contextMenu, setContextMenu] = createSignal<{
    type: ContextMenuType;
    x: number;
    y: number;
    targetId?: string;
  } | null>(null);
  
  // Clipboard for copy/paste
  const [clipboard, setClipboard] = createSignal<CanvasNode | null>(null);

  // Hooks
  const canvasEngine = useCanvasEngine();
  const nodeEngine = useNodeEngine();

  // Node shape cache for performance
  const nodeShapes = new Map<string, Konva.Group>();

  // Get node color based on type
  const getNodeColor = (nodeType: string): string => {
    const type = nodeType.toLowerCase();
    return NODE_COLORS[type] || NODE_COLORS.default;
  };

  const getNodeIcon = (nodeType: string): any => {
    const type = nodeType.toLowerCase();
    const map: Record<string, any> = {
      // Hardware
      gpio: Icons.pin(),
      adc: Icons.chart(),
      dac: Icons.trendUp(),
      pwm: Icons.activity(),
      uart: Icons.terminal(),
      spi: Icons.cpu(),
      i2c: Icons.cpu(),
      can: Icons.car(),
      usb: Icons.plug(),
      
      // RTOS
      task: Icons.clipboard(),
      "rtos task": Icons.clipboard(),
      timer: Icons.timer(),
      event: Icons.bell(),
      semaphore: Icons.trafficLight(),
      mutex: Icons.lock(),
      interrupt: Icons.flash(),
      "critical section": Icons.shield(),
      "message queue": Icons.message(),
      "event flags": Icons.flag(),
      
      // Logic
      state: Icons.layers(),
      initial: Icons.play(),
      final: Icons.stop(),
      decision: Icons.gitBranch(),
      junction: Icons.circleFilled(),
    };

    if (map[type]) return map[type];
    if (type.includes("hardware") || type.includes("driver")) return Icons.cpu();
    
    return Icons.package();
  };

  // Snap to grid
  const snapToGrid = (value: number): number => {
    return Math.round(value / GRID_SIZE) * GRID_SIZE;
  };

  // Get selected node data for the properties panel
  const getSelectedNodeData = (): NodeData | null => {
    const nodeId = selectedNodeId();
    if (!nodeId) return null;
    const state = canvasEngine.state();
    const node = state?.nodes.find(n => n.id === nodeId);
    if (!node) return null;
    return {
      id: node.id,
      label: node.label,
      node_type: String(node.node_type),
      x: node.x,
      y: node.y,
      width: node.width || 180,
      height: node.height || 80,
      entry_action: (node as any).entry_action,
      exit_action: (node as any).exit_action,
      description: (node as any).description,
      properties: (node as any).properties,
    };
  };

  // Get selected edge data for the transition editor
  const getSelectedEdgeData = (): TransitionData | null => {
    const edgeId = selectedEdgeId();
    if (!edgeId) return null;
    const state = canvasEngine.state();
    const edge = state?.edges.find(e => e.id === edgeId);
    if (!edge) return null;
    return {
      id: edge.id,
      source: edge.source,
      target: edge.target,
      label: edge.label,
      condition: (edge as any).condition,
      guard: (edge as any).guard,
      action: (edge as any).action,
      priority: (edge as any).priority || 0,
      sourcePort: (edge as any).source_port,
      targetPort: (edge as any).target_port,
    };
  };

  // Get edge source node name
  const getEdgeSourceName = (): string => {
    const edge = getSelectedEdgeData();
    if (!edge) return '';
    const state = canvasEngine.state();
    const node = state?.nodes.find(n => n.id === edge.source);
    return node?.label || edge.source;
  };

  // Get edge target node name
  const getEdgeTargetName = (): string => {
    const edge = getSelectedEdgeData();
    if (!edge) return '';
    const state = canvasEngine.state();
    const node = state?.nodes.find(n => n.id === edge.target);
    return node?.label || edge.target;
  };

  // Handle node updates from properties panel
  const handleNodeUpdate = async (id: string, data: Partial<NodeData>) => {
    try {
      await canvasEngine.updateNode(id, data as any);
      console.log('[KonvaCanvas] Node updated:', id);
    } catch (err) {
      console.error('[KonvaCanvas] Failed to update node:', err);
    }
  };

  // Handle node delete from properties panel
  const handleNodeDelete = async (id: string) => {
    try {
      await canvasEngine.deleteNodes([id]);
      setShowNodePanel(false);
      setSelectedNodeId(null);
      console.log('[KonvaCanvas] Node deleted:', id);
    } catch (err) {
      console.error('[KonvaCanvas] Failed to delete node:', err);
    }
  };

  // Handle edge updates from transition editor
  const handleEdgeUpdate = async (id: string, data: Partial<TransitionData>) => {
    try {
      // Use canvas engine to update edge
      // For now, log the update - we'll add proper IPC later
      console.log('[KonvaCanvas] Edge update requested:', id, data);
      // await canvasEngine.updateEdge(id, data);
    } catch (err) {
      console.error('[KonvaCanvas] Failed to update edge:', err);
    }
  };

  // Handle edge delete from transition editor
  const handleEdgeDelete = async (id: string) => {
    try {
      // Delete edge via canvas engine
      console.log('[KonvaCanvas] Edge delete requested:', id);
      setShowEdgePanel(false);
      setSelectedEdgeId(null);
    } catch (err) {
      console.error('[KonvaCanvas] Failed to delete edge:', err);
    }
  };

  // ==================== CONTEXT MENU FUNCTIONS ====================

  // Show context menu at position
  const showContextMenu = (type: ContextMenuType, x: number, y: number, targetId?: string) => {
    setContextMenu({ type, x, y, targetId });
  };

  // Hide context menu
  const hideContextMenu = () => {
    setContextMenu(null);
  };

  // Add node at position
  const addNodeAtPosition = async (nodeType: string, screenX: number, screenY: number) => {
    if (!stage) return;
    
    const worldPos = {
      x: (screenX - pan().x) / zoom(),
      y: (screenY - pan().y) / zoom(),
    };
    
    // Format label from node type
    const label = nodeType.charAt(0).toUpperCase() + nodeType.slice(1).replace(/_/g, ' ');
    
    try {
      await canvasEngine.addNode({
        id: `node_${Date.now()}`,
        x: snapToGrid(worldPos.x),
        y: snapToGrid(worldPos.y),
        label: label,
        node_type: nodeType as NodeType,
        width: 180,
        height: 80,
      });
      
      // Canvas syncs automatically via createEffect on canvasEngine.state()
    } catch (err) {
      console.error('[KonvaCanvas] Failed to add node:', err);
    }
    
    hideContextMenu();
  };

  // Copy selected node
  const copySelectedNode = () => {
    const nodeId = selectedNodeId();
    if (!nodeId) return;
    
    const state = canvasEngine.state();
    const node = state?.nodes.find(n => n.id === nodeId);
    if (node) {
      setClipboard({ ...node });
      console.log('[KonvaCanvas] Node copied:', node.id);
    }
  };

  // Paste node from clipboard
  const pasteNode = async (screenX: number, screenY: number) => {
    const node = clipboard();
    if (!node || !stage) return;
    
    const worldPos = {
      x: (screenX - pan().x) / zoom(),
      y: (screenY - pan().y) / zoom(),
    };
    
    try {
      await canvasEngine.addNode({
        ...node,
        id: `node_${Date.now()}`,
        x: snapToGrid(worldPos.x),
        y: snapToGrid(worldPos.y),
        label: node.label + ' (copy)',
      });
      // Canvas syncs automatically via createEffect
    } catch (err) {
      console.error('[KonvaCanvas] Failed to paste node:', err);
    }
    
    hideContextMenu();
  };

  // Duplicate selected node
  const duplicateSelectedNode = async () => {
    const nodeId = selectedNodeId();
    if (!nodeId) return;
    
    const state = canvasEngine.state();
    const node = state?.nodes.find(n => n.id === nodeId);
    if (!node) return;
    
    try {
      await canvasEngine.addNode({
        ...node,
        id: `node_${Date.now()}`,
        x: node.x + 40,
        y: node.y + 40,
        label: node.label + ' (copy)',
      });
      // Canvas syncs automatically via createEffect
    } catch (err) {
      console.error('[KonvaCanvas] Failed to duplicate node:', err);
    }
    
    hideContextMenu();
  };

  // Delete selected node or edge
  const deleteSelected = async () => {
    const nodeId = selectedNodeId();
    const edgeId = selectedEdgeId();
    
    if (nodeId) {
      await handleNodeDelete(nodeId);
    } else if (edgeId) {
      await handleEdgeDelete(edgeId);
    }
    
    hideContextMenu();
  };

  // Select all nodes
  const selectAllNodes = () => {
    // For now, just log - multi-select will be added later
    console.log('[KonvaCanvas] Select all nodes');
    hideContextMenu();
  };

  // Get canvas context menu items
  const getCanvasMenuItems = (x: number, y: number): MenuItem[] => [
    { label: 'Add State', icon: Icons.layers(), onClick: () => addNodeAtPosition('state', x, y) },
    { label: 'Add Initial State', icon: Icons.play(), onClick: () => addNodeAtPosition('initial_state', x, y) },
    { label: 'Add Final State', icon: Icons.stop(), onClick: () => addNodeAtPosition('final_state', x, y) },
    { label: 'Add Decision', icon: Icons.gitBranch(), onClick: () => addNodeAtPosition('decision', x, y) },
    { label: 'Add Junction', icon: Icons.circleFilled(), onClick: () => addNodeAtPosition('junction', x, y) },
    { divider: true, label: '', onClick: () => {} },
    { label: 'Paste', icon: Icons.clipboard(), shortcut: 'Ctrl+V', onClick: () => pasteNode(x, y), disabled: !clipboard() },
    { label: 'Select All', icon: Icons.check(), shortcut: 'Ctrl+A', onClick: selectAllNodes },
  ];

  // Get node context menu items
  const getNodeMenuItems = (nodeId: string): MenuItem[] => [
    { label: 'Edit Properties', icon: Icons.edit(), onClick: () => { setSelectedNodeId(nodeId); setShowNodePanel(true); hideContextMenu(); } },
    { divider: true, label: '', onClick: () => {} },
    { label: 'Copy', icon: Icons.clipboard(), shortcut: 'Ctrl+C', onClick: copySelectedNode },
    { label: 'Duplicate', icon: Icons.clipboard(), shortcut: 'Ctrl+D', onClick: duplicateSelectedNode },
    { label: 'Cut', icon: Icons.cut(), shortcut: 'Ctrl+X', onClick: () => { copySelectedNode(); deleteSelected(); } },
    { divider: true, label: '', onClick: () => {} },
    { label: 'Delete', icon: Icons.trash(), shortcut: 'Del', onClick: deleteSelected },
  ];

  // Get edge context menu items
  const getEdgeMenuItems = (edgeId: string): MenuItem[] => [
    { label: 'Edit Transition', icon: Icons.edit(), onClick: () => { setSelectedEdgeId(edgeId); setShowEdgePanel(true); hideContextMenu(); } },
    { divider: true, label: '', onClick: () => {} },
    { label: 'Delete', icon: Icons.trash(), shortcut: 'Del', onClick: deleteSelected },
  ];

  // Draw grid pattern
  const drawGrid = () => {
    if (!gridLayer || !stage) return;
    gridLayer.destroyChildren();

    const width = stage.width();
    const height = stage.height();
    const scale = zoom();
    const offset = pan();

    // Calculate visible area
    const startX = Math.floor(-offset.x / scale / GRID_SIZE) * GRID_SIZE;
    const startY = Math.floor(-offset.y / scale / GRID_SIZE) * GRID_SIZE;
    const endX = Math.ceil((width - offset.x) / scale / GRID_SIZE) * GRID_SIZE;
    const endY = Math.ceil((height - offset.y) / scale / GRID_SIZE) * GRID_SIZE;

    // Draw grid lines
    for (let x = startX; x <= endX; x += GRID_SIZE) {
      const line = new Konva.Line({
        points: [x, startY, x, endY],
        stroke: '#1e1e1e',
        strokeWidth: 1 / scale,
        listening: false,
      });
      gridLayer.add(line);
    }

    for (let y = startY; y <= endY; y += GRID_SIZE) {
      const line = new Konva.Line({
        points: [startX, y, endX, y],
        stroke: '#1e1e1e',
        strokeWidth: 1 / scale,
        listening: false,
      });
      gridLayer.add(line);
    }

    gridLayer.batchDraw();
  };

  // Create node shape
  const createNodeShape = (node: CanvasNode): Konva.Group => {
    const color = getNodeColor(String(node.node_type));
    const isSelected = selectedNodeId() === node.id;

    const group = new Konva.Group({
      x: node.x,
      y: node.y,
      draggable: true,
      id: node.id,
    });

    // Node body
    const rect = new Konva.Rect({
      width: node.width || 180,
      height: node.height || 80,
      fill: '#1a1a2e',
      stroke: isSelected ? '#e49b0f' : color,
      strokeWidth: isSelected ? 3 : 2,
      cornerRadius: 8,
      shadowColor: color,
      shadowBlur: isSelected ? 15 : 5,
      shadowOpacity: isSelected ? 0.5 : 0.3,
    });
    group.add(rect);

    // Header bar
    const header = new Konva.Rect({
      width: node.width || 180,
      height: 28,
      fill: color,
      cornerRadius: [8, 8, 0, 0],
    });
    group.add(header);

    // Label
    const label = new Konva.Text({
      text: node.label,
      fontSize: 13,
      fontFamily: 'Inter, system-ui, sans-serif',
      fill: '#ffffff',
      x: 10,
      y: 6,
      width: (node.width || 180) - 20,
      ellipsis: true,
    });
    group.add(label);

    // Type indicator
    const typeText = new Konva.Text({
      text: String(node.node_type).replace(/_/g, ' '),
      fontSize: 11,
      fontFamily: 'Inter, system-ui, sans-serif',
      fill: '#888',
      x: 10,
      y: 38,
    });
    group.add(typeText);

    // Input port (left) - larger hit area for easier connections
    const inputPort = new Konva.Circle({
      x: 0,
      y: (node.height || 80) / 2,
      radius: 10,
      fill: '#1a1a2e',
      stroke: '#4CAF50',
      strokeWidth: 2,
      hitStrokeWidth: 20,
      name: 'input-port',
    });
    group.add(inputPort);

    // Output port (right) - larger hit area for easier connections
    const outputPort = new Konva.Circle({
      x: node.width || 180,
      y: (node.height || 80) / 2,
      radius: 10,
      fill: '#1a1a2e',
      stroke: '#4CAF50',
      strokeWidth: 2,
      hitStrokeWidth: 20,
      name: 'output-port',
    });
    group.add(outputPort);

    // Event handlers
    group.on('dragstart', () => {
      group.moveToTop();
    });

    group.on('dragmove', () => {
      // Snap to grid while dragging
      const pos = group.position();
      group.position({
        x: snapToGrid(pos.x),
        y: snapToGrid(pos.y),
      });
      // Update edge positions
      updateEdges();
    });

    group.on('dragend', async () => {
      const pos = group.position();
      try {
        await canvasEngine.moveNodes([{
          id: node.id,
          x: pos.x,
          y: pos.y,
        }]);
      } catch (err) {
        console.error('Failed to move node:', err);
      }
    });

    group.on('click tap', (e) => {
      e.cancelBubble = true;
      setSelectedNodeId(node.id);
      setSelectedEdgeId(null);
      setShowEdgePanel(false);
      updateNodeSelection();
    });

    // Double-click to open properties panel
    group.on('dblclick dbltap', (e) => {
      e.cancelBubble = true;
      setSelectedNodeId(node.id);
      setShowNodePanel(true);
      setShowEdgePanel(false);
    });

    // Right-click context menu for nodes
    group.on('contextmenu', (e) => {
      e.cancelBubble = true;
      e.evt.preventDefault();
      setSelectedNodeId(node.id);
      setSelectedEdgeId(null);
      showContextMenu('node', e.evt.clientX, e.evt.clientY, node.id);
    });

    // Port connection handling - start connection on mousedown
    inputPort.on('mousedown touchstart', (e) => {
      e.cancelBubble = true;
      startConnection(node.id, false);
    });

    outputPort.on('mousedown touchstart', (e) => {
      e.cancelBubble = true;
      startConnection(node.id, true);
    });

    // Complete connection when mouseup on a port
    inputPort.on('mouseup touchend', (e) => {
      e.cancelBubble = true;
      if (connecting()) {
        endConnection(node.id, false); // Input port
      }
    });

    outputPort.on('mouseup touchend', (e) => {
      e.cancelBubble = true;
      if (connecting()) {
        endConnection(node.id, true); // Output port
      }
    });

    return group;
  };

  // Start connection from port
  const startConnection = (nodeId: string, isOutput: boolean) => {
    setConnecting({ nodeId, isOutput });
    
    if (!layer || !stage) return;
    
    const nodeShape = nodeShapes.get(nodeId);
    if (!nodeShape) return;

    const port = isOutput 
      ? nodeShape.findOne('.output-port') as Konva.Circle
      : nodeShape.findOne('.input-port') as Konva.Circle;
    
    if (!port) return;

    // Calculate port position relative to layer (node position + port offset)
    const nodePos = nodeShape.position();
    const portOffset = port.position();
    const portX = nodePos.x + portOffset.x;
    const portY = nodePos.y + portOffset.y;
    
    // Create a curved path for the connection preview
    const path = new Konva.Path({
      data: `M ${portX} ${portY} L ${portX} ${portY}`,
      stroke: '#e49b0f',
      strokeWidth: 2,
      lineCap: 'round',
      lineJoin: 'round',
      dash: [8, 4],
      opacity: 0.8,
    });
    
    // Add to edge layer so it transforms correctly with zoom/pan
    edgeLayer?.add(path);
    setConnectionLine(path as unknown as Konva.Line);
  };

  // Update connection line during drag - creates smooth bezier curve
  const updateConnectionLine = (mousePos: { x: number; y: number }) => {
    const line = connectionLine();
    const conn = connecting();
    if (!line || !conn) return;

    const nodeShape = nodeShapes.get(conn.nodeId);
    if (!nodeShape) return;

    const port = conn.isOutput 
      ? nodeShape.findOne('.output-port') as Konva.Circle
      : nodeShape.findOne('.input-port') as Konva.Circle;
    
    if (!port) return;

    // Calculate port position relative to layer (node position + port offset)
    const nodePos = nodeShape.position();
    const portOffset = port.position();
    const startX = nodePos.x + portOffset.x;
    const startY = nodePos.y + portOffset.y;
    
    // Convert mouse position from screen to world coordinates
    const endX = (mousePos.x - pan().x) / zoom();
    const endY = (mousePos.y - pan().y) / zoom();
    
    // Calculate control points for smooth curve
    const dx = endX - startX;
    const offset = Math.max(50, Math.abs(dx) * 0.4);
    
    const cp1x = conn.isOutput ? startX + offset : startX - offset;
    const cp2x = conn.isOutput ? endX - offset : endX + offset;
    
    // Update path data for bezier curve
    const pathData = `M ${startX} ${startY} C ${cp1x} ${startY}, ${cp2x} ${endY}, ${endX} ${endY}`;
    (line as unknown as Konva.Path).data(pathData);
    
    edgeLayer?.batchDraw();
  };

  // End connection
  const endConnection = async (targetNodeId: string | null, isTargetOutput: boolean) => {
    const conn = connecting();
    const line = connectionLine();
    
    if (line) {
      line.destroy();
      setConnectionLine(null);
    }
    
    if (!conn) {
      setConnecting(null);
      return;
    }

    // Don't connect to same node
    if (targetNodeId === conn.nodeId) {
      setConnecting(null);
      return;
    }

    // Must connect output to input
    if (targetNodeId && conn.isOutput !== isTargetOutput) {
      const source = conn.isOutput ? conn.nodeId : targetNodeId;
      const target = conn.isOutput ? targetNodeId : conn.nodeId;
      
      try {
        console.log('[KonvaCanvas] Connecting:', source, '->', target);
        await canvasEngine.connect(source, target);
        console.log('[KonvaCanvas] Connection successful, updating edges');
        updateEdges();
      } catch (err) {
        console.error('[KonvaCanvas] Failed to add edge:', err);
      }
    }
    
    setConnecting(null);
    layer?.batchDraw();
  };

  // Create edge shape with arrow, label, and selection support
  const createEdgeShape = (edge: CanvasEdge): Konva.Group | null => {
    // Get actual Konva shapes for source and target nodes
    const sourceShape = nodeShapes.get(edge.source);
    const targetShape = nodeShapes.get(edge.target);
    
    if (!sourceShape || !targetShape) {
      console.warn('[KonvaCanvas] Edge missing Konva shape:', edge.id, 'source:', edge.source, 'target:', edge.target);
      return null;
    }

    // Get port circles from the node groups
    const sourcePort = sourceShape.findOne('.output-port') as Konva.Circle;
    const targetPort = targetShape.findOne('.input-port') as Konva.Circle;
    
    if (!sourcePort || !targetPort) {
      console.warn('[KonvaCanvas] Edge missing ports:', edge.id);
      return null;
    }

    // Calculate port positions relative to the layer (not screen)
    // Port position = node group position + port position within group
    const sourceNodePos = sourceShape.position();
    const targetNodePos = targetShape.position();
    const sourcePortOffset = sourcePort.position();
    const targetPortOffset = targetPort.position();
    
    const startX = sourceNodePos.x + sourcePortOffset.x;
    const startY = sourceNodePos.y + sourcePortOffset.y;
    const endX = targetNodePos.x + targetPortOffset.x;
    const endY = targetNodePos.y + targetPortOffset.y;

    // Create orthogonal (right-angle) path points
    // Style: horizontal -> vertical -> horizontal
    const dx = endX - startX;
    const dy = endY - startY;
    
    // Minimum horizontal offset from ports
    const minHorizontalOffset = 30;
    
    let pathPoints: number[] = [];
    
    if (dx >= minHorizontalOffset * 2) {
      // Simple case: target is to the right, use midpoint for vertical segment
      const midX = startX + dx / 2;
      pathPoints = [
        startX, startY,           // Start at source port
        midX, startY,             // Go horizontal to midpoint
        midX, endY,               // Go vertical to target Y
        endX, endY                // Go horizontal to target port
      ];
    } else {
      // Target is to the left or very close - need to route around
      // Go right, then down/up, then left to target
      const routeOffset = minHorizontalOffset;
      const verticalMidY = (startY + endY) / 2;
      
      pathPoints = [
        startX, startY,                    // Start at source port
        startX + routeOffset, startY,      // Go right a bit
        startX + routeOffset, verticalMidY, // Go to middle height
        endX - routeOffset, verticalMidY,   // Go horizontal across
        endX - routeOffset, endY,           // Go to target height
        endX, endY                          // Go to target port
      ];
    }

    // Determine color based on selection and event type
    const isSelected = selectedEdgeId() === edge.id;
    
    // Get event-based color (if edge has event_type property)
    const eventType = (edge as unknown as { event_type?: string }).event_type;
    const isDisabled = (edge as unknown as { enabled?: boolean }).enabled === false;
    
    // Priority: selected > disabled > event color > default
    let baseColor = '#4CAF50'; // Default green
    if (eventType && EVENT_TYPE_COLORS[eventType]) {
      baseColor = EVENT_TYPE_COLORS[eventType];
    }
    if (isDisabled) {
      baseColor = '#666666'; // Gray for disabled
    }
    if (isSelected) {
      baseColor = '#e49b0f'; // Orange for selected
    }
    
    const strokeWidth = isSelected ? 3 : 2;
    const lineDash = isDisabled ? [8, 4] : undefined; // Dashed for disabled

    // Create group for edge components
    const group = new Konva.Group({ id: edge.id });

    // Main orthogonal polyline
    const curve = new Konva.Line({
      points: pathPoints,
      stroke: baseColor,
      strokeWidth: strokeWidth,
      lineCap: 'round',
      lineJoin: 'round',
      hitStrokeWidth: 15,
      dash: lineDash,
    });

    // Arrow head at end - pointing left (towards input port)
    const arrowSize = 10;
    const arrowHead = new Konva.Line({
      points: [
        endX - arrowSize, endY - arrowSize / 2,
        endX, endY,
        endX - arrowSize, endY + arrowSize / 2,
      ],
      fill: baseColor,
      stroke: baseColor,
      strokeWidth: strokeWidth,
      lineCap: 'round',
      lineJoin: 'round',
      closed: true,
    });

    group.add(curve);
    group.add(arrowHead);

    // Add event type icon badge
    if (eventType) {
      const icon = EVENT_TYPE_ICONS[eventType] || '?';
      const iconX = startX + 25;
      const iconY = startY - 12;
      
      // Icon background circle
      const iconBg = new Konva.Circle({
        x: iconX,
        y: iconY,
        radius: 10,
        fill: baseColor,
        stroke: '#1a1a1a',
        strokeWidth: 1,
      });
      
      // Icon text
      const iconText = new Konva.Text({
        x: iconX - 6,
        y: iconY - 6,
        text: icon,
        fontSize: 11,
        fill: '#ffffff',
      });
      
      group.add(iconBg);
      group.add(iconText);
    }

    // Add guard indicator (? icon)
    const hasGuard = (edge as unknown as { guard?: unknown }).guard;
    const hasAction = (edge as unknown as { action?: unknown }).action || 
                      (edge as unknown as { actions?: unknown }).actions;
    
    if (hasGuard || hasAction) {
      const indicatorX = endX - 30;
      const indicatorY = endY - 12;
      
      const indicators: string[] = [];
      if (hasGuard) indicators.push('?');
      if (hasAction) indicators.push('!');
      
      const indicatorBg = new Konva.Rect({
        x: indicatorX - 8,
        y: indicatorY - 8,
        width: indicators.length * 10 + 6,
        height: 16,
        fill: '#2a2a3e',
        cornerRadius: 4,
      });
      
      const indicatorText = new Konva.Text({
        x: indicatorX - 5,
        y: indicatorY - 5,
        text: indicators.join(''),
        fontSize: 10,
        fill: '#e0e0e0',
        fontStyle: 'bold',
      });
      
      group.add(indicatorBg);
      group.add(indicatorText);
    }

    // Add label if exists
    if (edge.label) {
      // Calculate midpoint for label placement (use path midpoint)
      const midX = (startX + endX) / 2;
      const midY = (startY + endY) / 2 - 15;
      
      const labelText = edge.label.length > 10 ? edge.label.slice(0, 10) + '…' : edge.label;
      const labelWidth = labelText.length * 7 + 12;
      
      const labelBg = new Konva.Rect({
        x: midX - labelWidth / 2,
        y: midY - 10,
        width: labelWidth,
        height: 20,
        fill: '#1a1a1a',
        stroke: '#2a2a2a',
        strokeWidth: 1,
        cornerRadius: 4,
      });
      
      const labelTextShape = new Konva.Text({
        x: midX - labelWidth / 2 + 6,
        y: midY - 6,
        text: labelText,
        fontSize: 11,
        fill: '#e8e8e8',
        fontFamily: 'IBM Plex Mono, monospace',
      });
      
      group.add(labelBg);
      group.add(labelTextShape);
    }

    // Click handler for selection
    group.on('click tap', (e) => {
      e.cancelBubble = true;
      setSelectedEdgeId(edge.id);
      setSelectedNodeId(null);
      setShowNodePanel(false);
      updateEdges();
    });

    // Double-click to open editor
    group.on('dblclick dbltap', (e) => {
      e.cancelBubble = true;
      setSelectedEdgeId(edge.id);
      setShowEdgePanel(true);
      setShowNodePanel(false);
    });

    // Right-click context menu for edges
    group.on('contextmenu', (e) => {
      e.cancelBubble = true;
      e.evt.preventDefault();
      setSelectedEdgeId(edge.id);
      setSelectedNodeId(null);
      showContextMenu('edge', e.evt.clientX, e.evt.clientY, edge.id);
    });

    // Hover effects
    group.on('mouseenter', () => {
      if (selectedEdgeId() !== edge.id) {
        curve.stroke('#6FCF97');
        arrowHead.stroke('#6FCF97');
        arrowHead.fill('#6FCF97');
        edgeLayer?.batchDraw();
      }
      document.body.style.cursor = 'pointer';
    });

    group.on('mouseleave', () => {
      if (selectedEdgeId() !== edge.id) {
        curve.stroke(baseColor);
        arrowHead.stroke(baseColor);
        arrowHead.fill(baseColor);
        edgeLayer?.batchDraw();
      }
      document.body.style.cursor = 'default';
    });

    return group;
  };

  // Update all edges
  const updateEdges = () => {
    if (!edgeLayer) return;
    
    edgeLayer.destroyChildren();
    const state = canvasEngine.state();
    if (!state) return;

    state.edges.forEach(edge => {
      const edgeShape = createEdgeShape(edge);
      if (edgeShape && edgeLayer) {
        edgeLayer.add(edgeShape);
      }
    });

    edgeLayer.batchDraw();
  };

  // Update node selection visuals
  const updateNodeSelection = () => {
    const selected = selectedNodeId();
    
    nodeShapes.forEach((shape, id) => {
      const rect = shape.findOne('Rect') as Konva.Rect;
      if (rect) {
        const node = canvasEngine.state()?.nodes.find(n => n.id === id);
        const color = node ? getNodeColor(String(node.node_type)) : NODE_COLORS.default;
        rect.stroke(id === selected ? '#e49b0f' : color);
        rect.strokeWidth(id === selected ? 3 : 2);
        rect.shadowOpacity(id === selected ? 0.5 : 0.3);
        rect.shadowBlur(id === selected ? 15 : 5);
      }
    });
    
    layer?.batchDraw();
  };

  // Sync nodes with canvas engine state
  const syncNodes = () => {
    if (!layer) return;

    const state = canvasEngine.state();
    if (!state) return;

    const existingIds = new Set(nodeShapes.keys());
    const currentIds = new Set(state.nodes.map(n => n.id));

    // Remove deleted nodes
    existingIds.forEach(id => {
      if (!currentIds.has(id)) {
        const shape = nodeShapes.get(id);
        shape?.destroy();
        nodeShapes.delete(id);
      }
    });

    // Add new nodes or update existing
    state.nodes.forEach(node => {
      if (!nodeShapes.has(node.id)) {
        const shape = createNodeShape(node);
        nodeShapes.set(node.id, shape);
        layer!.add(shape);
      } else {
        // Update position if changed externally
        const shape = nodeShapes.get(node.id)!;
        const pos = shape.position();
        if (pos.x !== node.x || pos.y !== node.y) {
          shape.position({ x: node.x, y: node.y });
        }
      }
    });

    layer.batchDraw();
    updateEdges();
  };

  // Handle stage wheel for zoom
  const handleWheel = (e: Konva.KonvaEventObject<WheelEvent>) => {
    e.evt.preventDefault();
    
    if (!stage) return;

    const oldScale = zoom();
    const pointer = stage.getPointerPosition();
    if (!pointer) return;

    const mousePointTo = {
      x: (pointer.x - pan().x) / oldScale,
      y: (pointer.y - pan().y) / oldScale,
    };

    const direction = e.evt.deltaY > 0 ? -1 : 1;
    const scaleBy = 1.1;
    let newScale = direction > 0 ? oldScale * scaleBy : oldScale / scaleBy;
    newScale = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, newScale));

    setZoom(newScale);

    const newPos = {
      x: pointer.x - mousePointTo.x * newScale,
      y: pointer.y - mousePointTo.y * newScale,
    };
    setPan(newPos);

    stage.scale({ x: newScale, y: newScale });
    stage.position(newPos);
    stage.batchDraw();
    drawGrid();
  };

  // Handle stage drag for panning
  const handleStageDragMove = () => {
    if (!stage) return;
    setPan(stage.position());
    drawGrid();
  };

  // Handle click on empty stage area
  const handleStageClick = (e: Konva.KonvaEventObject<MouseEvent>) => {
    // Only if clicking on stage, not on a node
    if (e.target === stage) {
      setSelectedNodeId(null);
      updateNodeSelection();
      
      // If we have a dragged palette node, create it here
      const paletteNode = draggedPaletteNode();
      if (paletteNode && stage) {
        const pos = stage.getPointerPosition();
        if (pos) {
          const worldPos = {
            x: (pos.x - pan().x) / zoom(),
            y: (pos.y - pan().y) / zoom(),
          };
          addNodeFromPalette(paletteNode, worldPos.x, worldPos.y);
          setDraggedPaletteNode(null);
          removeGhostNode();
        }
      }
    }
    
    // End any active connection with proximity detection
    const conn = connecting();
    if (conn && stage) {
      const pos = stage.getPointerPosition();
      if (pos) {
        // Convert screen position to canvas position
        const worldPos = {
          x: (pos.x - pan().x) / zoom(),
          y: (pos.y - pan().y) / zoom(),
        };
        
        // Find nearest port within snap distance
        const snapDistance = 40;
        let nearestPort: { nodeId: string; isOutput: boolean; distance: number } | null = null;
        
        nodeShapes.forEach((shape, nodeId) => {
          if (nodeId === conn.nodeId) return; // Skip source node
          
          const nodePos = shape.position();
          const inputPort = shape.findOne('.input-port') as Konva.Circle;
          const outputPort = shape.findOne('.output-port') as Konva.Circle;
          
          if (inputPort) {
            const portOffset = inputPort.position();
            const portX = nodePos.x + portOffset.x;
            const portY = nodePos.y + portOffset.y;
            const dist = Math.sqrt(Math.pow(worldPos.x - portX, 2) + Math.pow(worldPos.y - portY, 2));
            if (dist < snapDistance && (!nearestPort || dist < nearestPort.distance)) {
              nearestPort = { nodeId, isOutput: false, distance: dist };
            }
          }
          
          if (outputPort) {
            const portOffset = outputPort.position();
            const portX = nodePos.x + portOffset.x;
            const portY = nodePos.y + portOffset.y;
            const dist = Math.sqrt(Math.pow(worldPos.x - portX, 2) + Math.pow(worldPos.y - portY, 2));
            if (dist < snapDistance && (!nearestPort || dist < nearestPort.distance)) {
              nearestPort = { nodeId, isOutput: true, distance: dist };
            }
          }
        });
        
        if (nearestPort) {
          // Found a nearby port, make the connection
          const port = nearestPort as { nodeId: string; isOutput: boolean; distance: number };
          endConnection(port.nodeId, port.isOutput);
        } else {
          // No port found, cancel connection
          endConnection(null, false);
        }
      } else {
        endConnection(null, false);
      }
    }
  };

  // Handle mouse move on stage
  const handleStageMouseMove = (_e: Konva.KonvaEventObject<MouseEvent>) => {
    if (!stage) return;
    
    const pos = stage.getPointerPosition();
    if (!pos) return;

    // Update connection line
    if (connecting()) {
      updateConnectionLine(pos);
    }

    // Update ghost node position
    const ghost = ghostNode();
    if (ghost && draggedPaletteNode()) {
      const worldPos = {
        x: (pos.x - pan().x) / zoom(),
        y: (pos.y - pan().y) / zoom(),
      };
      ghost.position({
        x: snapToGrid(worldPos.x - 90),
        y: snapToGrid(worldPos.y - 40),
      });
      layer?.batchDraw();
    }
  };

  // Add node from palette
  const addNodeFromPalette = async (nodeInfo: NodeTypeInfo, x: number, y: number) => {
    const newNode: CanvasNode = {
      id: `node_${Date.now()}`,
      label: nodeInfo.name,
      node_type: nodeInfo.node_type as NodeType,
      x: snapToGrid(x),
      y: snapToGrid(y),
      width: 180,
      height: 80,
    };

    try {
      await canvasEngine.addNode(newNode);
      setSelectedNodeId(newNode.id);
    } catch (err) {
      console.error('Failed to add node:', err);
    }
  };

  // Create ghost node for palette drag preview
  const createGhostNode = (nodeInfo: NodeTypeInfo) => {
    if (!layer || ghostNode()) return;

    const color = getNodeColor(nodeInfo.node_type);
    const ghost = new Konva.Group({
      opacity: 0.6,
      listening: false,
    });

    const rect = new Konva.Rect({
      width: 180,
      height: 80,
      fill: '#1a1a2e',
      stroke: color,
      strokeWidth: 2,
      cornerRadius: 8,
      dash: [5, 5],
    });
    ghost.add(rect);

    const header = new Konva.Rect({
      width: 180,
      height: 28,
      fill: color,
      cornerRadius: [8, 8, 0, 0],
      opacity: 0.8,
    });
    ghost.add(header);

    const label = new Konva.Text({
      text: nodeInfo.name,
      fontSize: 13,
      fontFamily: 'Inter, system-ui, sans-serif',
      fill: '#ffffff',
      x: 10,
      y: 6,
    });
    ghost.add(label);

    layer.add(ghost);
    setGhostNode(ghost);
  };

  // Remove ghost node
  const removeGhostNode = () => {
    const ghost = ghostNode();
    if (ghost) {
      ghost.destroy();
      setGhostNode(null);
      layer?.batchDraw();
    }
  };

  // Handle palette node click (for click-to-place)
  const handlePaletteNodeClick = (nodeInfo: NodeTypeInfo) => {
    if (draggedPaletteNode()?.node_type === nodeInfo.node_type) {
      // Deselect
      setDraggedPaletteNode(null);
      removeGhostNode();
    } else {
      // Select for placement
      setDraggedPaletteNode(nodeInfo);
      createGhostNode(nodeInfo);
    }
  };

  // Handle palette node mousedown for drag
  const handlePaletteNodeMouseDown = (e: MouseEvent, nodeInfo: NodeTypeInfo) => {
    e.preventDefault();
    setDraggedPaletteNode(nodeInfo);
    createGhostNode(nodeInfo);
  };

  // Initialize Konva stage
  onMount(async () => {
    // Initialize engines
    try {
      await canvasEngine.init([], []);
      console.log('[KonvaCanvas] Canvas engine initialized');
    } catch (err) {
      console.error('[KonvaCanvas] Canvas engine init failed:', err);
    }
    
    try {
      await nodeEngine.loadPalette();
      console.log('[KonvaCanvas] Palette loaded, items:', nodeEngine.palette().length);
    } catch (err) {
      console.error('[KonvaCanvas] Palette load failed:', err);
    }

    if (!containerRef) return;

    const width = containerRef.clientWidth;
    const height = containerRef.clientHeight;

    stage = new Konva.Stage({
      container: containerRef,
      width,
      height,
      draggable: true,
    });

    // Grid layer (bottom)
    gridLayer = new Konva.Layer({ listening: false });
    stage.add(gridLayer);

    // Edge layer (middle)
    edgeLayer = new Konva.Layer();
    stage.add(edgeLayer);

    // Node layer (top)
    layer = new Konva.Layer();
    stage.add(layer);

    // Event handlers
    stage.on('wheel', handleWheel);
    stage.on('dragmove', handleStageDragMove);
    stage.on('click tap', handleStageClick);
    stage.on('mousemove', handleStageMouseMove);

    // Handle mouse up for ending connections
    stage.on('mouseup touchend', (e) => {
      if (connecting()) {
        const target = e.target;
        if (target instanceof Konva.Circle) {
          const parentGroup = target.getParent();
          if (parentGroup) {
            const nodeId = parentGroup.id();
            const isOutput = target.name() === 'output-port';
            endConnection(nodeId, isOutput);
          }
        } else {
          endConnection(null, false);
        }
      }
    });

    // Initial draw
    drawGrid();

    // Handle resize
    const resizeObserver = new ResizeObserver(() => {
      if (stage && containerRef) {
        stage.width(containerRef.clientWidth);
        stage.height(containerRef.clientHeight);
        drawGrid();
      }
    });
    resizeObserver.observe(containerRef);

    // Keyboard shortcuts
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ignore if typing in input/textarea
      const target = e.target as HTMLElement;
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return;

      // Delete - remove selected node/edge
      if (e.key === 'Delete' || e.key === 'Backspace') {
        if (selectedNodeId() || selectedEdgeId()) {
          e.preventDefault();
          deleteSelected();
        }
      }

      // Escape - deselect all
      if (e.key === 'Escape') {
        setSelectedNodeId(null);
        setSelectedEdgeId(null);
        hideContextMenu();
        updateNodeSelection();
      }

      // Ctrl+C - Copy
      if (e.ctrlKey && e.key === 'c') {
        if (selectedNodeId()) {
          e.preventDefault();
          copySelectedNode();
        }
      }

      // Ctrl+V - Paste
      if (e.ctrlKey && e.key === 'v') {
        if (clipboard() && stage) {
          e.preventDefault();
          const center = stage.getPointerPosition() || { x: 400, y: 300 };
          pasteNode(center.x, center.y);
        }
      }

      // Ctrl+D - Duplicate
      if (e.ctrlKey && e.key === 'd') {
        if (selectedNodeId()) {
          e.preventDefault();
          duplicateSelectedNode();
        }
      }

      // Ctrl+A - Select all (placeholder for multi-select)
      if (e.ctrlKey && e.key === 'a') {
        e.preventDefault();
        selectAllNodes();
      }
    };

    document.addEventListener('keydown', handleKeyDown);

    onCleanup(() => {
      document.removeEventListener('keydown', handleKeyDown);
      resizeObserver.disconnect();
      stage?.destroy();
    });
  });

  // Sync with canvas engine state
  createEffect(() => {
    const state = canvasEngine.state();
    if (state && layer) {
      syncNodes();
    }
  });

  // Handle global mouse move/up for resizing
  onMount(() => {
    const handleGlobalMouseMove = (e: MouseEvent) => {
      if (isResizingPalette()) {
        const newWidth = Math.max(200, Math.min(600, e.clientX));
        setPaletteWidth(newWidth);
      }
    };

    const handleGlobalMouseUp = () => {
      if (isResizingPalette()) {
        setIsResizingPalette(false);
        document.body.style.cursor = 'default';
      }
    };

    document.addEventListener('mousemove', handleGlobalMouseMove);
    document.addEventListener('mouseup', handleGlobalMouseUp);

    onCleanup(() => {
      document.removeEventListener('mousemove', handleGlobalMouseMove);
      document.removeEventListener('mouseup', handleGlobalMouseUp);
    });
  });

  // Zoom controls
  const handleZoomIn = () => {
    const newZoom = Math.min(MAX_ZOOM, zoom() * 1.2);
    setZoom(newZoom);
    stage?.scale({ x: newZoom, y: newZoom });
    stage?.batchDraw();
    drawGrid();
  };

  const handleZoomOut = () => {
    const newZoom = Math.max(MIN_ZOOM, zoom() / 1.2);
    setZoom(newZoom);
    stage?.scale({ x: newZoom, y: newZoom });
    stage?.batchDraw();
    drawGrid();
  };

  const handleZoomReset = () => {
    setZoom(1);
    setPan({ x: 0, y: 0 });
    stage?.scale({ x: 1, y: 1 });
    stage?.position({ x: 0, y: 0 });
    stage?.batchDraw();
    drawGrid();
  };

  return (
    <div class="konva-canvas-container">
      {/* Palette */}
      <Show when={showPalette()}>
        <div 
          class="canvas-palette"
          style={{ width: `${paletteWidth()}px` }}
        >
          <div class="palette-header">
            <span>NODES</span>
            <button class="close-btn" onClick={() => setShowPalette(false)}>◀</button>
          </div>
          <input type="text" class="palette-search" placeholder="Search nodes..." />
          <div class="palette-nodes">
            <For each={nodeEngine.palette()}>
              {([categoryName, nodes]) => (
                <>
                  <div class="palette-category">{categoryName}</div>
                  <For each={nodes}>
                    {(nodeInfo) => (
                      <div 
                        class={`palette-node ${draggedPaletteNode()?.node_type === nodeInfo.node_type ? 'selected' : ''}`}
                        onClick={() => handlePaletteNodeClick(nodeInfo)}
                        onMouseDown={(e) => handlePaletteNodeMouseDown(e, nodeInfo)}
                        style={{ "border-left-color": CATEGORY_INFO[nodeInfo.category]?.color }}
                      >
                        <span class="node-icon">{getNodeIcon(nodeInfo.node_type)}</span>
                        <span class="node-name">{nodeInfo.name}</span>
                      </div>
                    )}
                  </For>
                </>
              )}
            </For>
          </div>
          
          {/* Resize Handle */}
          <div 
            class="palette-resize-handle"
            onMouseDown={(e) => {
              e.preventDefault();
              setIsResizingPalette(true);
              document.body.style.cursor = 'col-resize';
            }}
          />
        </div>
      </Show>

      {/* Collapsed palette toggle */}
      <Show when={!showPalette()}>
        <button class="palette-toggle" onClick={() => setShowPalette(true)}>▶</button>
      </Show>

      {/* Canvas container */}
      <div 
        ref={containerRef} 
        class="konva-stage-container"
        onContextMenu={(e) => {
          e.preventDefault();
          showContextMenu('canvas', e.clientX, e.clientY);
        }}
      />

      {/* Toolbar */}
      <div class="canvas-toolbar">
        <button onClick={handleZoomOut} title="Zoom Out">−</button>
        <span class="zoom-level">{Math.round(zoom() * 100)}%</span>
        <button onClick={handleZoomIn} title="Zoom In">+</button>
        <button onClick={handleZoomReset} title="Reset View">⟲</button>
      </div>

      {/* Status hint */}
      <Show when={draggedPaletteNode()}>
        <div class="placement-hint">
          Click on canvas to place {draggedPaletteNode()?.name}
        </div>
      </Show>

      {/* Node Properties Panel */}
      <Show when={showNodePanel() && selectedNodeId()}>
        <div class="properties-overlay">
          <NodePropertiesPanel
            node={getSelectedNodeData()}
            onUpdate={handleNodeUpdate}
            onDelete={handleNodeDelete}
            onClose={() => { setShowNodePanel(false); setSelectedNodeId(null); }}
          />
        </div>
      </Show>

      {/* Transition Editor Panel */}
      <Show when={showEdgePanel() && selectedEdgeId()}>
        <div class="properties-overlay">
          <TransitionEditor
            transition={getSelectedEdgeData()}
            sourceNodeName={getEdgeSourceName()}
            targetNodeName={getEdgeTargetName()}
            onUpdate={handleEdgeUpdate}
            onDelete={handleEdgeDelete}
            onClose={() => { setShowEdgePanel(false); setSelectedEdgeId(null); }}
          />
        </div>
      </Show>

      {/* Context Menu */}
      <Show when={contextMenu()}>
        {(menu) => {
          const m = menu();
          const items = m.type === 'canvas' 
            ? getCanvasMenuItems(m.x, m.y)
            : m.type === 'node' && m.targetId
              ? getNodeMenuItems(m.targetId)
              : m.type === 'edge' && m.targetId
                ? getEdgeMenuItems(m.targetId)
                : [];
          
          return (
            <ContextMenu
              x={m.x}
              y={m.y}
              items={items}
              onClose={hideContextMenu}
            />
          );
        }}
      </Show>
    </div>
  );
};

export default KonvaCanvas;
