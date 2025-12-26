/**
 * FSMCanvas - Main canvas component with node palette and property panel
 * 
 * Integrates:
 * - NodePalette (left sidebar)
 * - Canvas (center)
 * - PropertyPanel (right sidebar)
 */

import { Component, createSignal, createEffect, onMount, For, Show } from "solid-js";
import { useCanvasEngine, CanvasNode } from "../hooks/useCanvasEngine";
import { useNodeEngine, NodeTypeInfo, Node, CATEGORY_INFO } from "../hooks/useNodeEngine";
import NodePalette from "./NodePalette";
import PropertyPanel from "./PropertyPanel";
import "./FSMCanvas.css";

interface FSMCanvasProps {
  showPalette?: boolean;
  showProperties?: boolean;
}

const FSMCanvas: Component<FSMCanvasProps> = (props) => {
  const canvasEngine = useCanvasEngine();
  const nodeEngine = useNodeEngine();
  
  const [zoom, setZoom] = createSignal(1);
  const [pan] = createSignal({ x: 0, y: 0 });
  const [selectedNodeData, setSelectedNodeData] = createSignal<Node | null>(null);
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [_isDragging, _setIsDragging] = createSignal(false);
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [_isConnecting, _setIsConnecting] = createSignal(false);
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [_connectStart, _setConnectStart] = createSignal<{ nodeId: string; port: string } | null>(null);
  
  let canvasRef: HTMLDivElement | undefined;
  let svgRef: SVGSVGElement | undefined;
  void svgRef; // Silence unused warning

  onMount(async () => {
    // Initialize canvas
    await canvasEngine.init([], []);
    // Load node palette
    await nodeEngine.loadPalette();
    await nodeEngine.loadAllTypes();
  });

  // Update selected node data when selection changes
  createEffect(() => {
    const state = canvasEngine.state();
    if (state.selection.length === 1) {
      const selectedId = state.selection[0];
      const node = state.nodes.find(n => n.id === selectedId);
      if (node) {
        // Convert CanvasNode to Node format for PropertyPanel
        const nodeInfo = nodeEngine.allTypes().find(t => t.node_type === node.node_type);
        setSelectedNodeData({
          id: node.id,
          label: node.label,
          node_type: node.node_type,
          x: node.x,
          y: node.y,
          width: node.width,
          height: node.height,
          ports: nodeInfo?.ports || { inputs: [], outputs: [] },
          properties: {},
          entry_action: node.entry_action,
          exit_action: node.exit_action,
          description: node.description,
        });
      }
    } else {
      setSelectedNodeData(null);
    }
  });

  // Handle drag-and-drop from palette
  const handleDrop = async (e: DragEvent) => {
    e.preventDefault();
    const data = e.dataTransfer?.getData("application/node-type");
    if (!data || !canvasRef) return;

    const nodeInfo: NodeTypeInfo = JSON.parse(data);
    const rect = canvasRef.getBoundingClientRect();
    const x = (e.clientX - rect.left - pan().x) / zoom();
    const y = (e.clientY - rect.top - pan().y) / zoom();

    // Create node using canvas engine (old system for now)
    const newNode: CanvasNode = {
      id: `node_${Date.now()}`,
      label: nodeInfo.name,
      node_type: mapNodeType(nodeInfo.node_type),
      x,
      y,
      width: 160,
      height: 80,
    };
    
    await canvasEngine.addNode(newNode);
  };

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
    e.dataTransfer!.dropEffect = "copy";
  };

  // Map new node types to old NodeType (for backwards compatibility)
  const mapNodeType = (newType: string): any => {
    const typeMap: Record<string, string> = {
      "state": "process",
      "initial": "input",
      "final": "output",
      "decision": "decision",
      "gpio": "hardware",
      "adc": "hardware",
      "pwm": "hardware",
      "timer": "delay",
      "interrupt": "interrupt",
    };
    return typeMap[newType] || "process";
  };

  // Get node color based on type/category
  const getNodeColor = (nodeType: string): string => {
    const info = nodeEngine.allTypes().find(t => t.node_type === nodeType);
    if (info) {
      return CATEGORY_INFO[info.category]?.color || "#4f46e5";
    }
    // Fallback for old types
    const colors: Record<string, string> = {
      input: "#4CAF50",
      output: "#F44336",
      process: "#2196F3",
      decision: "#FF9800",
      error: "#E91E63",
      hardware: "#9C27B0",
      delay: "#607D8B",
      interrupt: "#FF5722",
    };
    return colors[nodeType] || "#4f46e5";
  };

  // Get node icon
  const getNodeIcon = (nodeType: string): string => {
    const info = nodeEngine.allTypes().find(t => t.node_type === nodeType);
    return info?.icon || "●";
  };

  // Handle node click
  const handleNodeClick = async (e: MouseEvent, nodeId: string) => {
    e.stopPropagation();
    await canvasEngine.select([nodeId]);
  };

  // Handle canvas click (deselect)
  const handleCanvasClick = async () => {
    await canvasEngine.clearSelection();
  };

  // Handle keyboard shortcuts
  const handleKeyDown = async (e: KeyboardEvent) => {
    if (e.key === "Delete" || e.key === "Backspace") {
      const selection = canvasEngine.state().selection;
      if (selection.length > 0) {
        await canvasEngine.deleteNodes(selection);
      }
    }
    if (e.ctrlKey && e.key === "z") {
      await canvasEngine.undo();
    }
    if (e.ctrlKey && e.key === "y") {
      await canvasEngine.redo();
    }
    if (e.ctrlKey && e.key === "a") {
      e.preventDefault();
      await canvasEngine.selectAll();
    }
  };

  // Zoom handlers
  const handleWheel = (e: WheelEvent) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    setZoom(z => Math.max(0.25, Math.min(3, z * delta)));
  };

  const handlePropertyChange = async (nodeId: string, key: string, value: any) => {
    await nodeEngine.updateProperty(nodeId, key, value);
  };

  const handleLabelChange = async (nodeId: string, label: string) => {
    await canvasEngine.updateNode(nodeId, { label });
  };

  return (
    <div class="fsm-canvas-container" tabIndex={0} onKeyDown={handleKeyDown}>
      {/* Left Sidebar - Node Palette */}
      <Show when={props.showPalette !== false}>
        <NodePalette />
      </Show>

      {/* Main Canvas */}
      <div 
        class="canvas-viewport"
        ref={canvasRef}
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        onClick={handleCanvasClick}
        onWheel={handleWheel}
      >
        {/* Toolbar */}
        <div class="canvas-toolbar">
          <button onClick={() => canvasEngine.autoLayout("hierarchical")} title="Auto Layout">
            📐
          </button>
          <button onClick={() => canvasEngine.validate()} title="Validate">
            ✓
          </button>
          <button onClick={() => canvasEngine.undo()} title="Undo">
            ↶
          </button>
          <button onClick={() => canvasEngine.redo()} title="Redo">
            ↷
          </button>
          <div class="zoom-controls">
            <button onClick={() => setZoom(z => Math.max(0.25, z - 0.1))}>−</button>
            <span>{Math.round(zoom() * 100)}%</span>
            <button onClick={() => setZoom(z => Math.min(3, z + 0.1))}>+</button>
          </div>
        </div>

        {/* SVG Canvas */}
        <svg 
          ref={svgRef}
          class="canvas-svg"
          style={{
            transform: `scale(${zoom()}) translate(${pan().x}px, ${pan().y}px)`,
          }}
        >
          {/* Grid */}
          <defs>
            <pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">
              <path d="M 20 0 L 0 0 0 20" fill="none" stroke="rgba(255,255,255,0.05)" stroke-width="1"/>
            </pattern>
          </defs>
          <rect width="100%" height="100%" fill="url(#grid)"/>

          {/* Edges */}
          <g class="edges">
            <For each={canvasEngine.state().edges}>
              {(edge) => {
                const path = canvasEngine.edgePaths()[edge.id];
                return (
                  <g class="edge">
                    <path
                      d={path || ""}
                      fill="none"
                      stroke="#4f46e5"
                      stroke-width="2"
                      marker-end="url(#arrowhead)"
                    />
                    <Show when={edge.label}>
                      <text class="edge-label" dy="-5">
                        <textPath href={`#edge-${edge.id}`} startOffset="50%">
                          {edge.label}
                        </textPath>
                      </text>
                    </Show>
                  </g>
                );
              }}
            </For>
          </g>

          {/* Arrow marker */}
          <defs>
            <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
              <polygon points="0 0, 10 3.5, 0 7" fill="#4f46e5"/>
            </marker>
          </defs>

          {/* Nodes */}
          <g class="nodes">
            <For each={canvasEngine.state().nodes}>
              {(node) => {
                const isSelected = canvasEngine.state().selection.includes(node.id);
                const color = getNodeColor(node.node_type);
                const icon = getNodeIcon(node.node_type);
                
                return (
                  <g 
                    class={`node ${isSelected ? "selected" : ""}`}
                    transform={`translate(${node.x}, ${node.y})`}
                    onClick={(e) => handleNodeClick(e, node.id)}
                  >
                    {/* Node body */}
                    <rect
                      width={node.width}
                      height={node.height}
                      rx="8"
                      fill="var(--node-bg, #252538)"
                      stroke={isSelected ? "#fff" : color}
                      stroke-width={isSelected ? "2" : "1"}
                    />
                    
                    {/* Header bar */}
                    <rect
                      width={node.width}
                      height="24"
                      rx="8"
                      ry="8"
                      fill={color}
                    />
                    <rect
                      y="16"
                      width={node.width}
                      height="8"
                      fill={color}
                    />
                    
                    {/* Icon and label */}
                    <text x="10" y="17" fill="white" font-size="12">{icon}</text>
                    <text x="28" y="17" fill="white" font-size="11" font-weight="600">
                      {node.node_type.toUpperCase()}
                    </text>
                    
                    {/* Node label */}
                    <text 
                      x={node.width / 2} 
                      y={node.height / 2 + 10} 
                      fill="var(--text-color, #e0e0e0)"
                      font-size="12"
                      text-anchor="middle"
                    >
                      {node.label}
                    </text>

                    {/* Input port */}
                    <circle
                      cx="0"
                      cy={node.height / 2}
                      r="6"
                      fill="#4CAF50"
                      stroke="white"
                      stroke-width="2"
                      class="port input-port"
                    />

                    {/* Output port */}
                    <circle
                      cx={node.width}
                      cy={node.height / 2}
                      r="6"
                      fill="#2196F3"
                      stroke="white"
                      stroke-width="2"
                      class="port output-port"
                    />
                  </g>
                );
              }}
            </For>
          </g>
        </svg>

        {/* Validation Display */}
        <Show when={canvasEngine.validation()}>
          {(result) => (
            <div class={`validation-badge ${result().valid ? "valid" : "invalid"}`}>
              {result().valid ? "✓ Valid" : `✗ ${result().errors.length} errors`}
            </div>
          )}
        </Show>
      </div>

      {/* Right Sidebar - Property Panel */}
      <Show when={props.showProperties !== false}>
        <PropertyPanel
          selectedNode={selectedNodeData()}
          onPropertyChange={handlePropertyChange}
          onLabelChange={handleLabelChange}
        />
      </Show>
    </div>
  );
};

export default FSMCanvas;
