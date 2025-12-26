/**
 * UnifiedCanvas - The single powerful canvas editor
 * 
 * Combines all features from:
 * - Classic App.tsx canvas
 * - Enhanced Node Editor
 * - Code Generation Panel
 * 
 * Features:
 * - 40+ node types with typed ports
 * - Node palette with search
 * - Property panel
 * - Code generation (single file + multi-file)
 * - Snap-to-grid
 * - Context menus
 * - Minimap
 * - Keyboard shortcuts
 */

import { Component, createSignal, createEffect, onMount, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { useCanvasEngine, CanvasNode, CanvasEdge, NodeType } from "../hooks/useCanvasEngine";
import { useNodeEngine, NodeTypeInfo, CATEGORY_INFO } from "../hooks/useNodeEngine";
import "./UnifiedCanvas.css";

// ============================================================================
// Types
// ============================================================================

interface GeneratedCode {
  includes: string[];
  defines: string[];
  types: string[];
  globals: string[];
  init_code: string[];
  handler_code: string[];
  main_loop: string[];
}

interface CodeFile {
  name: string;
  content: string;
  language: string;
}

interface UnifiedCanvasProps {
  onNodesChange?: (nodes: CanvasNode[]) => void;
  onEdgesChange?: (edges: CanvasEdge[]) => void;
  projectName?: string;
  onToggleMode?: () => void;
}

// ============================================================================
// Component
// ============================================================================

const UnifiedCanvas: Component<UnifiedCanvasProps> = (props) => {
  // Engines
  const canvasEngine = useCanvasEngine();
  const nodeEngine = useNodeEngine();
  
  // Canvas state
  const [zoom, setZoom] = createSignal(1);
  const [pan, setPan] = createSignal({ x: 0, y: 0 });
  const [isPanning, setIsPanning] = createSignal(false);
  const [panStart, setPanStart] = createSignal({ x: 0, y: 0 });
  const [snapToGrid, setSnapToGrid] = createSignal(true);
  const [gridSize] = createSignal(20);
  
  // UI state
  const [showPalette, setShowPalette] = createSignal(true);
  const [showProperties, setShowProperties] = createSignal(true);
  const [showCodePanel, setShowCodePanel] = createSignal(false);
  const [showMinimap, setShowMinimap] = createSignal(true);
  const [paletteSearch, setPaletteSearch] = createSignal("");
  const [activeCategory, setActiveCategory] = createSignal<string | null>(null);
  
  // Selection and interaction
  const [selectedNodeId, setSelectedNodeId] = createSignal<string | null>(null);
  const [contextMenu, setContextMenu] = createSignal<{ x: number; y: number; nodeId?: string } | null>(null);
  const [isDragging, setIsDragging] = createSignal(false);
  const [dragNodeId, setDragNodeId] = createSignal<string | null>(null);
  const [dragOffset, setDragOffset] = createSignal({ x: 0, y: 0 });
  
  // Connection drawing
  const [isConnecting, setIsConnecting] = createSignal(false);
  const [connectionStart, setConnectionStart] = createSignal<{ nodeId: string; port: string; isOutput: boolean } | null>(null);
  const [connectionEnd, setConnectionEnd] = createSignal({ x: 0, y: 0 });
  
  // Clipboard for copy/paste
  const [clipboard, setClipboard] = createSignal<{ nodes: CanvasNode[]; edges: CanvasEdge[] } | null>(null);
  
  // Multi-selection (using engine's selection, but tracking marquee locally)
  const [isMarqueeSelecting, setIsMarqueeSelecting] = createSignal(false);
  const [marqueeStart, setMarqueeStart] = createSignal({ x: 0, y: 0 });
  const [marqueeEnd, setMarqueeEnd] = createSignal({ x: 0, y: 0 });
  
  // AI-assisted prompt
  const [showAIModal, setShowAIModal] = createSignal(false);
  const [aiPrompt, setAIPrompt] = createSignal("");
  const [isAIGenerating, setIsAIGenerating] = createSignal(false);
  
  // Code generation
  const [generatedCode, setGeneratedCode] = createSignal<GeneratedCode | null>(null);
  const [generatedFiles, setGeneratedFiles] = createSignal<CodeFile[]>([]);
  const [isGenerating, setIsGenerating] = createSignal(false);
  const [codeTab, setCodeTab] = createSignal<"preview" | "files">("preview");
  const [selectedFile, setSelectedFile] = createSignal<string | null>(null);
  
  // Refs
  let canvasRef: HTMLDivElement | undefined;
  let svgRef: SVGSVGElement | undefined;

  // ============================================================================
  // Initialization
  // ============================================================================

  onMount(async () => {
    await canvasEngine.init([], []);
    await nodeEngine.loadPalette();
    await nodeEngine.loadAllTypes();
  });

  // Notify parent of changes
  createEffect(() => {
    const state = canvasEngine.state();
    props.onNodesChange?.(state.nodes);
    props.onEdgesChange?.(state.edges);
  });

  // ============================================================================
  // Node Helpers
  // ============================================================================

  const getNodeColor = (nodeType: string): string => {
    const info = nodeEngine.allTypes().find(t => t.node_type === nodeType);
    if (info) {
      return CATEGORY_INFO[info.category]?.color || "#4f46e5";
    }
    const colors: Record<string, string> = {
      input: "#4CAF50", output: "#F44336", process: "#2196F3",
      decision: "#FF9800", error: "#E91E63", hardware: "#9C27B0",
    };
    return colors[nodeType] || "#4f46e5";
  };

  const getNodeIcon = (nodeType: string): string => {
    const info = nodeEngine.allTypes().find(t => t.node_type === nodeType);
    return info?.icon || "●";
  };

  const mapNodeType = (newType: string): NodeType => {
    const typeMap: Record<string, NodeType> = {
      "state": "process", "initial": "input", "final": "output",
      "decision": "decision", "gpio": "hardware", "adc": "hardware",
      "pwm": "hardware", "timer": "delay", "interrupt": "interrupt",
    };
    return typeMap[newType] || "process";
  };

  const snapToGridValue = (value: number): number => {
    if (!snapToGrid()) return value;
    return Math.round(value / gridSize()) * gridSize();
  };

  // ============================================================================
  // Palette Filtering
  // ============================================================================

  const filteredPalette = () => {
    const search = paletteSearch().toLowerCase();
    const category = activeCategory();
    
    return nodeEngine.allTypes().filter(t => {
      const matchesSearch = !search || 
        t.name.toLowerCase().includes(search) || 
        t.node_type.toLowerCase().includes(search);
      const matchesCategory = !category || t.category === category;
      return matchesSearch && matchesCategory;
    });
  };

  const categories = () => {
    const cats = new Set(nodeEngine.allTypes().map(t => t.category));
    return Array.from(cats);
  };

  // ============================================================================
  // Drag and Drop from Palette
  // ============================================================================

  const handlePaletteDragStart = (e: DragEvent, nodeInfo: NodeTypeInfo) => {
    e.dataTransfer?.setData("application/node-type", JSON.stringify(nodeInfo));
    e.dataTransfer!.effectAllowed = "copy";
  };

  const handleCanvasDrop = async (e: DragEvent) => {
    e.preventDefault();
    const data = e.dataTransfer?.getData("application/node-type");
    if (!data || !canvasRef) return;

    const nodeInfo: NodeTypeInfo = JSON.parse(data);
    const rect = canvasRef.getBoundingClientRect();
    let x = (e.clientX - rect.left - pan().x) / zoom();
    let y = (e.clientY - rect.top - pan().y) / zoom();
    
    x = snapToGridValue(x);
    y = snapToGridValue(y);

    const newNode: CanvasNode = {
      id: `node_${Date.now()}`,
      label: nodeInfo.name,
      node_type: mapNodeType(nodeInfo.node_type),
      x, y,
      width: 180,
      height: 100,
    };
    
    await canvasEngine.addNode(newNode);
    setSelectedNodeId(newNode.id);
  };

  const handleCanvasDragOver = (e: DragEvent) => {
    e.preventDefault();
    e.dataTransfer!.dropEffect = "copy";
  };

  // ============================================================================
  // Canvas Interaction
  // ============================================================================

  const handleCanvasClick = () => {
    setSelectedNodeId(null);
    setContextMenu(null);
  };

  const handleNodeClick = (e: MouseEvent, nodeId: string) => {
    e.stopPropagation();
    setSelectedNodeId(nodeId);
    setContextMenu(null);
  };

  const handleNodeRightClick = (e: MouseEvent, nodeId: string) => {
    e.preventDefault();
    e.stopPropagation();
    setSelectedNodeId(nodeId);
    setContextMenu({ x: e.clientX, y: e.clientY, nodeId });
  };

  const handleCanvasRightClick = (e: MouseEvent) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY });
  };

  // ============================================================================
  // Node Dragging
  // ============================================================================

  const handleNodeMouseDown = (e: MouseEvent, nodeId: string) => {
    if (e.button !== 0) return;
    e.stopPropagation();
    
    const node = canvasEngine.state().nodes.find(n => n.id === nodeId);
    if (!node) return;
    
    setIsDragging(true);
    setDragNodeId(nodeId);
    setDragOffset({
      x: e.clientX / zoom() - node.x,
      y: e.clientY / zoom() - node.y,
    });
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (isDragging() && dragNodeId()) {
      let newX = e.clientX / zoom() - dragOffset().x;
      let newY = e.clientY / zoom() - dragOffset().y;
      
      newX = snapToGridValue(newX);
      newY = snapToGridValue(newY);
      
      canvasEngine.moveNode(dragNodeId()!, newX, newY);
    } else if (isPanning()) {
      setPan({
        x: e.clientX - panStart().x,
        y: e.clientY - panStart().y,
      });
    } else if (isConnecting()) {
      if (!canvasRef) return;
      const rect = canvasRef.getBoundingClientRect();
      setConnectionEnd({
        x: (e.clientX - rect.left - pan().x) / zoom(),
        y: (e.clientY - rect.top - pan().y) / zoom(),
      });
    }
  };

  const handleMouseUp = async () => {
    if (isDragging()) {
      setIsDragging(false);
      setDragNodeId(null);
    }
    if (isPanning()) {
      setIsPanning(false);
    }
    if (isConnecting()) {
      // TODO: Complete connection if over valid port
      setIsConnecting(false);
      setConnectionStart(null);
    }
  };

  // ============================================================================
  // Panning
  // ============================================================================

  const handleCanvasMouseDown = (e: MouseEvent) => {
    if (e.button === 1 || (e.button === 0 && e.altKey)) {
      setIsPanning(true);
      setPanStart({
        x: e.clientX - pan().x,
        y: e.clientY - pan().y,
      });
    }
  };

  const handleWheel = (e: WheelEvent) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    setZoom(z => Math.max(0.25, Math.min(3, z * delta)));
  };

  // ============================================================================
  // Port Connection
  // ============================================================================

  const handlePortMouseDown = (e: MouseEvent, nodeId: string, portName: string, isOutput: boolean) => {
    e.stopPropagation();
    setIsConnecting(true);
    setConnectionStart({ nodeId, port: portName, isOutput });
    
    const node = canvasEngine.state().nodes.find(n => n.id === nodeId);
    if (node) {
      setConnectionEnd({
        x: node.x + (isOutput ? node.width : 0),
        y: node.y + node.height / 2,
      });
    }
  };

  const handlePortMouseUp = async (e: MouseEvent, nodeId: string, _portName: string, isOutput: boolean) => {
    e.stopPropagation();
    const start = connectionStart();
    if (!start || start.nodeId === nodeId || start.isOutput === isOutput) {
      setIsConnecting(false);
      setConnectionStart(null);
      return;
    }
    
    // Create connection
    const sourceId = start.isOutput ? start.nodeId : nodeId;
    const targetId = start.isOutput ? nodeId : start.nodeId;
    
    await canvasEngine.connect(sourceId, targetId, "");
    setIsConnecting(false);
    setConnectionStart(null);
  };

  // ============================================================================
  // Keyboard Shortcuts
  // ============================================================================

  const handleKeyDown = async (e: KeyboardEvent) => {
    // Delete selected nodes
    if (e.key === "Delete" || e.key === "Backspace") {
      const selection = canvasEngine.state().selection;
      if (selection.length > 0) {
        await canvasEngine.deleteNodes(selection);
        setSelectedNodeId(null);
      } else if (selectedNodeId()) {
        await canvasEngine.deleteNodes([selectedNodeId()!]);
        setSelectedNodeId(null);
      }
    }
    
    // Undo/Redo
    if (e.ctrlKey && e.key === "z") {
      e.preventDefault();
      await canvasEngine.undo();
    }
    if (e.ctrlKey && e.key === "y") {
      e.preventDefault();
      await canvasEngine.redo();
    }
    
    // Select All
    if (e.ctrlKey && e.key === "a") {
      e.preventDefault();
      await canvasEngine.selectAll();
    }
    
    // Copy - store selected nodes and their edges
    if (e.ctrlKey && e.key === "c") {
      e.preventDefault();
      const selection = canvasEngine.state().selection;
      const nodesToCopy = selection.length > 0 
        ? selection 
        : (selectedNodeId() ? [selectedNodeId()!] : []);
      
      if (nodesToCopy.length > 0) {
        const allNodes = canvasEngine.state().nodes;
        const allEdges = canvasEngine.state().edges;
        const copiedNodes = allNodes.filter(n => nodesToCopy.includes(n.id));
        // Copy edges that are between copied nodes
        const copiedEdges = allEdges.filter(e => 
          nodesToCopy.includes(e.source) && nodesToCopy.includes(e.target)
        );
        setClipboard({ nodes: copiedNodes, edges: copiedEdges });
      }
    }
    
    // Paste - clone nodes at offset
    if (e.ctrlKey && e.key === "v") {
      e.preventDefault();
      const clip = clipboard();
      if (clip && clip.nodes.length > 0) {
        const offset = 50;
        const idMap: Record<string, string> = {};
        
        // Create new nodes with new IDs
        for (const node of clip.nodes) {
          const newId = `node_${Date.now()}_${Math.random().toString(36).substr(2, 5)}`;
          idMap[node.id] = newId;
          const newNode: CanvasNode = {
            ...node,
            id: newId,
            x: node.x + offset,
            y: node.y + offset,
          };
          await canvasEngine.addNode(newNode);
        }
        
        // Recreate edges with new IDs
        for (const edge of clip.edges) {
          if (idMap[edge.source] && idMap[edge.target]) {
            await canvasEngine.connect(idMap[edge.source], idMap[edge.target], edge.label);
          }
        }
        
        // Select newly pasted nodes
        await canvasEngine.select(Object.values(idMap));
      }
    }
    
    // Escape - cancel operations
    if (e.key === "Escape") {
      setContextMenu(null);
      setIsConnecting(false);
      setConnectionStart(null);
      setIsMarqueeSelecting(false);
      await canvasEngine.clearSelection();
    }
  };

  // ============================================================================
  // Context Menu Actions
  // ============================================================================

  const duplicateNode = async () => {
    const ctx = contextMenu();
    if (!ctx?.nodeId) return;
    
    const node = canvasEngine.state().nodes.find(n => n.id === ctx.nodeId);
    if (!node) return;
    
    const newNode: CanvasNode = {
      ...node,
      id: `node_${Date.now()}`,
      x: node.x + 40,
      y: node.y + 40,
    };
    
    await canvasEngine.addNode(newNode);
    setContextMenu(null);
    setSelectedNodeId(newNode.id);
  };

  const deleteNode = async () => {
    const ctx = contextMenu();
    if (!ctx?.nodeId) return;
    
    await canvasEngine.deleteNodes([ctx.nodeId]);
    setContextMenu(null);
    setSelectedNodeId(null);
  };

  // ============================================================================
  // Code Generation
  // ============================================================================

  const handleGenerateCode = async () => {
    setIsGenerating(true);
    try {
      const nodes = canvasEngine.state().nodes;
      const nodeData = nodes.map(n => ({
        node_type: n.node_type,
        properties: {},
      }));
      
      const codeResult = await invoke<GeneratedCode>("nodes_generate_code", { nodesData: nodeData });
      setGeneratedCode(codeResult);
      
      const filesResult = await invoke<CodeFile[]>("nodes_generate_files", { nodesData: nodeData });
      setGeneratedFiles(filesResult);
      setSelectedFile(filesResult.length > 0 ? filesResult[0].name : null);
      
      setShowCodePanel(true);
    } catch (error) {
      console.error("Code generation failed:", error);
    } finally {
      setIsGenerating(false);
    }
  };

  const renderFullCode = () => {
    const code = generatedCode();
    if (!code) return "// No code generated yet\n// Add nodes and click Generate Code";
    
    let output = "";
    const uniqueIncludes = [...new Set(code.includes)].sort();
    for (const inc of uniqueIncludes) output += `#include ${inc}\n`;
    output += "\n";
    for (const def of code.defines) output += `#define ${def}\n`;
    if (code.defines.length > 0) output += "\n";
    for (const glob of code.globals) output += `${glob}\n`;
    if (code.globals.length > 0) output += "\n";
    for (const handler of code.handler_code) output += `${handler}\n\n`;
    output += "void System_Init(void) {\n";
    for (const init of code.init_code) output += `    ${init}\n`;
    output += "}\n\nvoid System_Update(void) {\n";
    for (const main of code.main_loop) output += `    ${main}\n`;
    output += "}\n";
    return output;
  };

  const getSelectedFileContent = () => {
    const file = generatedFiles().find(f => f.name === selectedFile());
    return file?.content || "";
  };

  const copyToClipboard = (content: string) => {
    navigator.clipboard.writeText(content);
  };

  const downloadFile = (name: string, content: string) => {
    const blob = new Blob([content], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = name;
    a.click();
    URL.revokeObjectURL(url);
  };

  // ============================================================================
  // Selected Node Data
  // ============================================================================

  const selectedNode = () => {
    const id = selectedNodeId();
    if (!id) return null;
    return canvasEngine.state().nodes.find(n => n.id === id) || null;
  };

  // ============================================================================
  // Render
  // ============================================================================

  return (
    <div 
      class="unified-canvas"
      tabIndex={0}
      onKeyDown={handleKeyDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
    >
      {/* Left Panel - Node Palette */}
      <Show when={showPalette()}>
        <div class="palette-panel">
          <div class="palette-header">
            <h3>Nodes</h3>
            <button class="toggle-btn" onClick={() => setShowPalette(false)}>◀</button>
          </div>
          
          {/* Search */}
          <div class="palette-search">
            <input 
              type="text" 
              placeholder="Search nodes..."
              value={paletteSearch()}
              onInput={(e) => setPaletteSearch(e.currentTarget.value)}
            />
          </div>
          
          {/* Categories */}
          <div class="palette-categories">
            <button 
              class={!activeCategory() ? "active" : ""}
              onClick={() => setActiveCategory(null)}
            >
              All
            </button>
            <For each={categories()}>
              {(cat) => (
                <button 
                  class={activeCategory() === cat ? "active" : ""}
                  onClick={() => setActiveCategory(cat)}
                  style={{ "border-color": CATEGORY_INFO[cat]?.color }}
                >
                  {cat}
                </button>
              )}
            </For>
          </div>
          
          {/* Node List */}
          <div class="palette-nodes">
            <For each={filteredPalette()}>
              {(nodeInfo) => (
                <div 
                  class="palette-node"
                  draggable={true}
                  onDragStart={(e) => handlePaletteDragStart(e, nodeInfo)}
                  style={{ "border-left-color": CATEGORY_INFO[nodeInfo.category]?.color }}
                >
                  <span class="node-icon">{nodeInfo.icon}</span>
                  <span class="node-name">{nodeInfo.name}</span>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>
      
      {/* Collapsed Palette Toggle */}
      <Show when={!showPalette()}>
        <button class="collapsed-toggle left" onClick={() => setShowPalette(true)}>▶</button>
      </Show>

      {/* Main Canvas Area */}
      <div class="canvas-area">
        {/* Simplified Toolbar */}
        <div class="canvas-toolbar">
          <div class="toolbar-left">
            <span class="project-name">{props.projectName || "Project"}</span>
            <span class="unified-badge">UNIFIED</span>
            <span class="node-count">{canvasEngine.state().nodes.length} nodes</span>
          </div>
          
          <div class="toolbar-center">
            <button onClick={() => canvasEngine.undo()} title="Undo">↶</button>
            <button onClick={() => canvasEngine.redo()} title="Redo">↷</button>
            <div class="toolbar-divider" />
            <button onClick={() => setZoom(z => Math.max(0.25, z - 0.1))}>−</button>
            <span class="zoom-level">{Math.round(zoom() * 100)}%</span>
            <button onClick={() => setZoom(z => Math.min(3, z + 0.1))}>+</button>
            <button onClick={() => { setZoom(1); setPan({ x: 0, y: 0 }); }} title="Fit">⊙</button>
          </div>
          
          <div class="toolbar-right">
            <button onClick={() => setSnapToGrid(!snapToGrid())} class={snapToGrid() ? "active" : ""} title="Snap">⊞</button>
            <button onClick={() => canvasEngine.autoLayout("hierarchical")} title="Layout">📐</button>
            <button onClick={() => setShowMinimap(!showMinimap())} class={showMinimap() ? "active" : ""} title="Minimap">🗺️</button>
            <div class="toolbar-divider" />
            <Show when={props.onToggleMode}>
              <button onClick={props.onToggleMode} title="Switch to Classic Layout">
                 ↩ Classic
              </button>
            </Show>
            <div class="toolbar-divider" />
            <button 
              class="ai-btn"
              onClick={() => setShowAIModal(true)}
              title="AI Generate Nodes"
            >
              🪄 AI
            </button>
            <button 
              class="generate-btn"
              onClick={handleGenerateCode}
              disabled={isGenerating()}
            >
              ⚡ Generate
            </button>
            <button 
              class={showCodePanel() ? "active" : ""}
              onClick={() => setShowCodePanel(!showCodePanel())}
            >
              📄
            </button>
          </div>
        </div>

        {/* Canvas Container */}
        <div 
          class="canvas-viewport"
          ref={canvasRef}
          onDrop={handleCanvasDrop}
          onDragOver={handleCanvasDragOver}
          onClick={handleCanvasClick}
          onContextMenu={handleCanvasRightClick}
          onMouseDown={handleCanvasMouseDown}
          onWheel={handleWheel}
        >
          <svg 
            ref={svgRef}
            class="canvas-svg"
            style={{
              transform: `translate(${pan().x}px, ${pan().y}px) scale(${zoom()})`,
            }}
          >
            {/* Grid Pattern */}
            <defs>
              <pattern id="grid-pattern" width={gridSize()} height={gridSize()} patternUnits="userSpaceOnUse">
                <path 
                  d={`M ${gridSize()} 0 L 0 0 0 ${gridSize()}`} 
                  fill="none" 
                  stroke="rgba(255,255,255,0.05)" 
                  stroke-width="1"
                />
              </pattern>
              <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                <polygon points="0 0, 10 3.5, 0 7" fill="#58a6ff"/>
              </marker>
            </defs>
            <rect x="-5000" y="-5000" width="10000" height="10000" fill="url(#grid-pattern)"/>

            {/* Edges */}
            <g class="edges">
              <For each={canvasEngine.state().edges}>
                {(edge) => {
                  const source = canvasEngine.state().nodes.find(n => n.id === edge.source);
                  const target = canvasEngine.state().nodes.find(n => n.id === edge.target);
                  if (!source || !target) return null;
                  
                  const sx = source.x + source.width;
                  const sy = source.y + source.height / 2;
                  const tx = target.x;
                  const ty = target.y + target.height / 2;
                  const cx = (sx + tx) / 2;
                  
                  return (
                    <g class="edge">
                      <path
                        d={`M ${sx} ${sy} C ${cx} ${sy}, ${cx} ${ty}, ${tx} ${ty}`}
                        fill="none"
                        stroke="#58a6ff"
                        stroke-width="2"
                        marker-end="url(#arrowhead)"
                      />
                      <Show when={edge.label}>
                        <text x={cx} y={(sy + ty) / 2 - 8} class="edge-label">{edge.label}</text>
                      </Show>
                    </g>
                  );
                }}
              </For>
              
              {/* Connection preview line */}
              <Show when={isConnecting() && connectionStart()}>
                {(() => {
                  const start = connectionStart()!;
                  const node = canvasEngine.state().nodes.find(n => n.id === start.nodeId);
                  if (!node) return null;
                  
                  const sx = node.x + (start.isOutput ? node.width : 0);
                  const sy = node.y + node.height / 2;
                  const end = connectionEnd();
                  
                  return (
                    <path
                      d={`M ${sx} ${sy} L ${end.x} ${end.y}`}
                      fill="none"
                      stroke="#58a6ff"
                      stroke-width="2"
                      stroke-dasharray="5,5"
                    />
                  );
                })()}
              </Show>
            </g>

            {/* Nodes */}
            <g class="nodes">
              <For each={canvasEngine.state().nodes}>
                {(node) => {
                  const isSelected = selectedNodeId() === node.id;
                  const color = getNodeColor(node.node_type);
                  const icon = getNodeIcon(node.node_type);
                  
                  return (
                    <g 
                      class={`node ${isSelected ? "selected" : ""}`}
                      transform={`translate(${node.x}, ${node.y})`}
                      onClick={(e) => handleNodeClick(e, node.id)}
                      onContextMenu={(e) => handleNodeRightClick(e, node.id)}
                      onMouseDown={(e) => handleNodeMouseDown(e, node.id)}
                    >
                      {/* Shadow */}
                      <rect
                        x="4" y="4"
                        width={node.width}
                        height={node.height}
                        rx="10"
                        fill="rgba(0,0,0,0.3)"
                      />
                      
                      {/* Body */}
                      <rect
                        width={node.width}
                        height={node.height}
                        rx="10"
                        fill="var(--node-bg, #1c1c2e)"
                        stroke={isSelected ? "#fff" : color}
                        stroke-width={isSelected ? "3" : "2"}
                      />
                      
                      {/* Header */}
                      <rect
                        width={node.width}
                        height="28"
                        rx="10"
                        fill={color}
                      />
                      <rect
                        y="18"
                        width={node.width}
                        height="10"
                        fill={color}
                      />
                      
                      {/* Icon and Type */}
                      <text x="12" y="20" fill="white" font-size="14">{icon}</text>
                      <text x="32" y="20" fill="white" font-size="12" font-weight="600">
                        {node.node_type.toUpperCase()}
                      </text>
                      
                      {/* Label */}
                      <text 
                        x={node.width / 2} 
                        y={node.height / 2 + 15} 
                        fill="#e0e0e0"
                        font-size="13"
                        text-anchor="middle"
                      >
                        {node.label}
                      </text>

                      {/* Input Port */}
                      <circle
                        cx="0"
                        cy={node.height / 2}
                        r="8"
                        fill="#10b981"
                        stroke="white"
                        stroke-width="2"
                        class="port input-port"
                        onMouseDown={(e) => handlePortMouseDown(e, node.id, "in", false)}
                        onMouseUp={(e) => handlePortMouseUp(e, node.id, "in", false)}
                      />
                      <text x="-4" y={node.height / 2 + 4} fill="white" font-size="10">◀</text>

                      {/* Output Port */}
                      <circle
                        cx={node.width}
                        cy={node.height / 2}
                        r="8"
                        fill="#3b82f6"
                        stroke="white"
                        stroke-width="2"
                        class="port output-port"
                        onMouseDown={(e) => handlePortMouseDown(e, node.id, "out", true)}
                        onMouseUp={(e) => handlePortMouseUp(e, node.id, "out", true)}
                      />
                      <text x={node.width - 4} y={node.height / 2 + 4} fill="white" font-size="10">▶</text>
                    </g>
                  );
                }}
              </For>
            </g>
          </svg>

          {/* Minimap */}
          <Show when={showMinimap()}>
            <div class="minimap">
              <svg viewBox="-500 -500 1000 1000">
                <rect x="-500" y="-500" width="1000" height="1000" fill="#0d1117"/>
                <For each={canvasEngine.state().nodes}>
                  {(node) => (
                    <rect 
                      x={node.x / 10} 
                      y={node.y / 10} 
                      width={node.width / 10} 
                      height={node.height / 10}
                      fill={getNodeColor(node.node_type)}
                    />
                  )}
                </For>
                <rect 
                  x={-pan().x / zoom() / 10 - 50} 
                  y={-pan().y / zoom() / 10 - 50}
                  width={100 / zoom()}
                  height={80 / zoom()}
                  fill="none"
                  stroke="#58a6ff"
                  stroke-width="2"
                />
              </svg>
            </div>
          </Show>

          {/* Validation Badge */}
          <Show when={canvasEngine.validation()}>
            {(result) => (
              <div class={`validation-badge ${result().valid ? "valid" : "invalid"}`}>
                {result().valid ? "✓ Valid" : `✗ ${result().errors.length} errors`}
              </div>
            )}
          </Show>
        </div>
      </div>

      {/* Right Panel - Properties + Code */}
      <Show when={showProperties() || showCodePanel()}>
        <div class="right-panel">
          {/* Tabs */}
          <div class="right-panel-tabs">
            <button 
              class={showProperties() && !showCodePanel() ? "active" : ""}
              onClick={() => { setShowProperties(true); setShowCodePanel(false); }}
            >
              Properties
            </button>
            <button 
              class={showCodePanel() ? "active" : ""}
              onClick={() => { setShowCodePanel(true); }}
            >
              Code
            </button>
            <button class="close-btn" onClick={() => { setShowProperties(false); setShowCodePanel(false); }}>✕</button>
          </div>

          {/* Properties Content */}
          <Show when={showProperties() && !showCodePanel()}>
            <div class="properties-content enhanced">
              <Show when={selectedNode()} fallback={
                <div class="no-selection">
                  <div class="no-selection-icon">📋</div>
                  <h4>SELECT A NODE TO EDIT</h4>
                  <p>Click on any node in the canvas to view and modify its properties</p>
                </div>
              }>
                {(node) => (
                  <div class="property-form enhanced">
                    {/* Node Header */}
                    <div class="node-header">
                      <div class="node-icon-large" style={{ background: getNodeColor(node().node_type) }}>
                        {getNodeIcon(node().node_type)}
                      </div>
                      <div class="node-header-info">
                        <input 
                          class="node-title-input"
                          type="text" 
                          value={node().label}
                          onInput={(e) => canvasEngine.updateNode(node().id, { label: e.currentTarget.value })}
                        />
                        <span class="node-type-badge">{node().node_type.toUpperCase()}</span>
                      </div>
                    </div>

                    {/* ID & Dimensions */}
                    <div class="property-section">
                      <div class="section-title">📍 Position & Size</div>
                      <div class="property-grid">
                        <div class="property-item">
                          <label>X</label>
                          <input type="number" value={Math.round(node().x)} readonly />
                        </div>
                        <div class="property-item">
                          <label>Y</label>
                          <input type="number" value={Math.round(node().y)} readonly />
                        </div>
                        <div class="property-item">
                          <label>W</label>
                          <input type="number" value={node().width} readonly />
                        </div>
                        <div class="property-item">
                          <label>H</label>
                          <input type="number" value={node().height} readonly />
                        </div>
                      </div>
                    </div>

                    {/* Entry Action */}
                    <div class="property-section">
                      <div class="section-title">▶️ Entry Action</div>
                      <textarea 
                        class="code-textarea"
                        placeholder="// Code runs when entering this state"
                        value={node().entry_action || ""}
                        onInput={(e) => canvasEngine.updateNode(node().id, { entry_action: e.currentTarget.value || null })}
                        rows={3}
                      />
                    </div>

                    {/* Exit Action */}
                    <div class="property-section">
                      <div class="section-title">⏹️ Exit Action</div>
                      <textarea 
                        class="code-textarea"
                        placeholder="// Code runs when exiting this state"
                        value={node().exit_action || ""}
                        onInput={(e) => canvasEngine.updateNode(node().id, { exit_action: e.currentTarget.value || null })}
                        rows={3}
                      />
                    </div>

                    {/* Description */}
                    <div class="property-section">
                      <div class="section-title">📝 Description</div>
                      <textarea 
                        class="description-textarea"
                        placeholder="Optional notes about this node..."
                        value={node().description || ""}
                        onInput={(e) => canvasEngine.updateNode(node().id, { description: e.currentTarget.value || null })}
                        rows={2}
                      />
                    </div>

                    {/* Connections */}
                    <div class="property-section">
                      <div class="section-title">🔗 Connections</div>
                      <div class="connections-list">
                        <For each={canvasEngine.state().edges.filter(e => e.source === node().id)}>
                          {(edge) => {
                            const target = canvasEngine.state().nodes.find(n => n.id === edge.target);
                            return (
                              <div class="connection-item outgoing">
                                <span class="conn-icon">→</span>
                                <span class="conn-label">{target?.label || edge.target}</span>
                                <button class="conn-delete" onClick={() => canvasEngine.deleteEdges([edge.id])}>✕</button>
                              </div>
                            );
                          }}
                        </For>
                        <For each={canvasEngine.state().edges.filter(e => e.target === node().id)}>
                          {(edge) => {
                            const source = canvasEngine.state().nodes.find(n => n.id === edge.source);
                            return (
                              <div class="connection-item incoming">
                                <span class="conn-icon">←</span>
                                <span class="conn-label">{source?.label || edge.source}</span>
                                <button class="conn-delete" onClick={() => canvasEngine.deleteEdges([edge.id])}>✕</button>
                              </div>
                            );
                          }}
                        </For>
                        <Show when={canvasEngine.state().edges.filter(e => e.source === node().id || e.target === node().id).length === 0}>
                          <div class="no-connections">No connections yet</div>
                        </Show>
                      </div>
                    </div>

                    {/* Quick Actions */}
                    <div class="property-section actions">
                      <div class="section-title">⚡ Quick Actions</div>
                      <div class="action-buttons">
                        <button class="action-btn duplicate" onClick={async () => {
                          const newNode: CanvasNode = {
                            ...node(),
                            id: `node_${Date.now()}`,
                            x: node().x + 60,
                            y: node().y + 60,
                            label: `${node().label}_copy`,
                          };
                          await canvasEngine.addNode(newNode);
                        }}>
                          📋 Duplicate
                        </button>
                        <button class="action-btn delete" onClick={async () => {
                          await canvasEngine.deleteNodes([node().id]);
                          setSelectedNodeId(null);
                        }}>
                          🗑️ Delete
                        </button>
                      </div>
                    </div>
                  </div>
                )}
              </Show>
            </div>
          </Show>

          {/* Code Panel Content */}
          <Show when={showCodePanel()}>
            <div class="code-panel-content">
              <div class="code-tabs">
                <button class={codeTab() === "preview" ? "active" : ""} onClick={() => setCodeTab("preview")}>
                  Preview
                </button>
                <button class={codeTab() === "files" ? "active" : ""} onClick={() => setCodeTab("files")}>
                  Files ({generatedFiles().length})
                </button>
              </div>
              
              <Show when={codeTab() === "preview"}>
                <div class="code-actions">
                  <button onClick={() => copyToClipboard(renderFullCode())}>📋 Copy</button>
                  <button onClick={() => downloadFile("generated.c", renderFullCode())}>⬇️ Download</button>
                </div>
                <pre class="code-preview"><code>{renderFullCode()}</code></pre>
              </Show>
              
              <Show when={codeTab() === "files"}>
                <div class="files-view">
                  <div class="file-list">
                    <For each={generatedFiles()}>
                      {(file) => (
                        <div 
                          class={`file-item ${selectedFile() === file.name ? "selected" : ""}`}
                          onClick={() => setSelectedFile(file.name)}
                        >
                          {file.name.endsWith(".h") ? "📘" : "📄"} {file.name}
                        </div>
                      )}
                    </For>
                  </div>
                  <Show when={selectedFile()}>
                    <div class="file-content">
                      <div class="file-header">
                        <span>{selectedFile()}</span>
                        <button onClick={() => copyToClipboard(getSelectedFileContent())}>📋</button>
                        <button onClick={() => downloadFile(selectedFile()!, getSelectedFileContent())}>⬇️</button>
                      </div>
                      <pre><code>{getSelectedFileContent()}</code></pre>
                    </div>
                  </Show>
                </div>
              </Show>
            </div>
          </Show>
        </div>
      </Show>

      {/* Context Menu */}
      <Show when={contextMenu()}>
        {(ctx) => (
          <div 
            class="context-menu"
            style={{ left: `${ctx().x}px`, top: `${ctx().y}px` }}
          >
            <Show when={ctx().nodeId}>
              <button onClick={duplicateNode}>📋 Duplicate</button>
              <button onClick={deleteNode}>🗑️ Delete</button>
              <div class="context-divider" />
            </Show>
            <button onClick={() => canvasEngine.undo()}>↶ Undo</button>
            <button onClick={() => canvasEngine.redo()}>↷ Redo</button>
            <div class="context-divider" />
            <button onClick={handleGenerateCode}>⚡ Generate Code</button>
            <button onClick={() => setContextMenu(null)}>✕ Close</button>
          </div>
        )}
      </Show>

      {/* AI Generate Modal */}
      <Show when={showAIModal()}>
        <div class="ai-modal-overlay" onClick={() => setShowAIModal(false)}>
          <div class="ai-modal" onClick={(e) => e.stopPropagation()}>
            <div class="ai-modal-header">
              <h3>🪄 AI Node Generator</h3>
              <button class="close-btn" onClick={() => setShowAIModal(false)}>✕</button>
            </div>
            <div class="ai-modal-content">
              <label>Describe what you want to build:</label>
              <textarea
                placeholder="E.g., Create an LED blink FSM with 500ms on/off timing..."
                value={aiPrompt()}
                onInput={(e) => setAIPrompt(e.currentTarget.value)}
                rows={4}
              />
              <div class="ai-examples">
                <span>Examples:</span>
                <button onClick={() => setAIPrompt("LED blink with button control")}>💡 LED Blink</button>
                <button onClick={() => setAIPrompt("Temperature monitor with thresholds")}>🌡️ Temp Monitor</button>
                <button onClick={() => setAIPrompt("UART communication state machine")}>📡 UART FSM</button>
              </div>
            </div>
            <div class="ai-modal-footer">
              <button class="cancel-btn" onClick={() => setShowAIModal(false)}>Cancel</button>
              <button 
                class="generate-ai-btn"
                onClick={async () => {
                  if (!aiPrompt().trim()) return;
                  setIsAIGenerating(true);
                  try {
                    // Call backend AI to generate nodes
                    const result = await invoke<{ nodes: CanvasNode[]; edges: CanvasEdge[] }>("ai_generate_fsm", {
                      prompt: aiPrompt()
                    });
                    // Add generated nodes to canvas
                    for (const node of result.nodes) {
                      await canvasEngine.addNode(node);
                    }
                    // Add edges
                    for (const edge of result.edges) {
                      await canvasEngine.connect(edge.source, edge.target, edge.label);
                    }
                    setShowAIModal(false);
                    setAIPrompt("");
                  } catch (e) {
                    console.error("AI generation failed:", e);
                    // Fallback: create a simple placeholder node
                    await canvasEngine.addNode({
                      id: `ai_${Date.now()}`,
                      label: "AI Generated",
                      node_type: "process",
                      x: 200,
                      y: 200,
                      width: 140,
                      height: 60,
                    });
                  } finally {
                    setIsAIGenerating(false);
                  }
                }}
                disabled={isAIGenerating() || !aiPrompt().trim()}
              >
                {isAIGenerating() ? "Generating..." : "🪄 Generate Nodes"}
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};

export default UnifiedCanvas;
