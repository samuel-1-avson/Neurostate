import { createSignal, For } from "solid-js";
import "./Minimap.css";

interface MinimapNode {
  id: string;
  x: number;
  y: number;
  type: string;
}

interface MinimapProps {
  nodes: MinimapNode[];
  edges: { source: string; target: string }[];
  selectedNodes: Set<string>;
  viewport: { x: number; y: number; zoom: number };
  canvasSize: { width: number; height: number };
  onViewportChange: (x: number, y: number) => void;
}

export function Minimap(props: MinimapProps) {
  const [isDragging, setIsDragging] = createSignal(false);
  
  // Minimap dimensions
  const MINIMAP_WIDTH = 180;
  const MINIMAP_HEIGHT = 120;
  const NODE_SIZE = 6;
  
  // Calculate bounds of all nodes
  const getBounds = () => {
    if (props.nodes.length === 0) {
      return { minX: 0, maxX: 1000, minY: 0, maxY: 800 };
    }
    
    let minX = Infinity, maxX = -Infinity;
    let minY = Infinity, maxY = -Infinity;
    
    props.nodes.forEach(node => {
      minX = Math.min(minX, node.x);
      maxX = Math.max(maxX, node.x + 160);
      minY = Math.min(minY, node.y);
      maxY = Math.max(maxY, node.y + 80);
    });
    
    // Add padding
    const padding = 100;
    return {
      minX: minX - padding,
      maxX: maxX + padding,
      minY: minY - padding,
      maxY: maxY + padding,
    };
  };
  
  // Scale factor for minimap
  const getScale = () => {
    const bounds = getBounds();
    const worldWidth = bounds.maxX - bounds.minX;
    const worldHeight = bounds.maxY - bounds.minY;
    return Math.min(MINIMAP_WIDTH / worldWidth, MINIMAP_HEIGHT / worldHeight);
  };
  
  // Convert world coordinates to minimap coordinates
  const toMinimap = (x: number, y: number) => {
    const bounds = getBounds();
    const scale = getScale();
    return {
      x: (x - bounds.minX) * scale,
      y: (y - bounds.minY) * scale,
    };
  };
  
  // Get viewport rectangle position and size
  const getViewportRect = () => {
    const bounds = getBounds();
    const scale = getScale();
    const vp = props.viewport;
    
    // Viewport in world coordinates
    const vpWorldX = -vp.x / vp.zoom;
    const vpWorldY = -vp.y / vp.zoom;
    const vpWorldWidth = props.canvasSize.width / vp.zoom;
    const vpWorldHeight = props.canvasSize.height / vp.zoom;
    
    return {
      x: (vpWorldX - bounds.minX) * scale,
      y: (vpWorldY - bounds.minY) * scale,
      width: vpWorldWidth * scale,
      height: vpWorldHeight * scale,
    };
  };
  
  // Handle click/drag on minimap
  const handleMinimapMouseDown = (e: MouseEvent) => {
    setIsDragging(true);
    navigateToPosition(e);
  };
  
  const handleMinimapMouseMove = (e: MouseEvent) => {
    if (isDragging()) {
      navigateToPosition(e);
    }
  };
  
  const handleMinimapMouseUp = () => {
    setIsDragging(false);
  };
  
  const navigateToPosition = (e: MouseEvent) => {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const clickX = e.clientX - rect.left;
    const clickY = e.clientY - rect.top;
    
    const bounds = getBounds();
    const scale = getScale();
    const vp = props.viewport;
    
    // Convert minimap position to world coordinates
    const worldX = clickX / scale + bounds.minX;
    const worldY = clickY / scale + bounds.minY;
    
    // Center viewport on clicked position
    const newPanX = -(worldX - props.canvasSize.width / vp.zoom / 2) * vp.zoom;
    const newPanY = -(worldY - props.canvasSize.height / vp.zoom / 2) * vp.zoom;
    
    props.onViewportChange(newPanX, newPanY);
  };
  
  // Get node color based on type
  const getNodeColor = (type: string) => {
    switch (type) {
      case "input": return "#2980b9";
      case "output": return "#27ae60";
      case "process": return "#8e44ad";
      case "decision": return "#e49b0f";
      case "error": return "#c0392b";
      default: return "#666";
    }
  };
  
  return (
    <div 
      class="minimap"
      onMouseDown={handleMinimapMouseDown}
      onMouseMove={handleMinimapMouseMove}
      onMouseUp={handleMinimapMouseUp}
      onMouseLeave={handleMinimapMouseUp}
    >
      <div class="minimap-header">
        <span>MINIMAP</span>
      </div>
      <svg 
        class="minimap-canvas"
        width={MINIMAP_WIDTH} 
        height={MINIMAP_HEIGHT}
        viewBox={`0 0 ${MINIMAP_WIDTH} ${MINIMAP_HEIGHT}`}
      >
        {/* Edges */}
        <For each={props.edges}>
          {(edge) => {
            const source = props.nodes.find(n => n.id === edge.source);
            const target = props.nodes.find(n => n.id === edge.target);
            if (!source || !target) return null;
            
            const from = toMinimap(source.x + 80, source.y + 40);
            const to = toMinimap(target.x + 80, target.y);
            
            return (
              <line 
                x1={from.x} y1={from.y}
                x2={to.x} y2={to.y}
                stroke="var(--accent-dim, #c48a0d)"
                stroke-width="1"
                opacity="0.5"
              />
            );
          }}
        </For>
        
        {/* Nodes */}
        <For each={props.nodes}>
          {(node) => {
            const pos = toMinimap(node.x, node.y);
            const isSelected = props.selectedNodes.has(node.id);
            
            return (
              <rect 
                x={pos.x}
                y={pos.y}
                width={NODE_SIZE}
                height={NODE_SIZE * 0.6}
                fill={getNodeColor(node.type)}
                rx="1"
                stroke={isSelected ? "var(--accent, #e49b0f)" : "none"}
                stroke-width={isSelected ? 1 : 0}
              />
            );
          }}
        </For>
        
        {/* Viewport indicator */}
        {(() => {
          const vp = getViewportRect();
          return (
            <rect 
              x={vp.x}
              y={vp.y}
              width={Math.max(vp.width, 10)}
              height={Math.max(vp.height, 8)}
              fill="rgba(228, 155, 15, 0.15)"
              stroke="var(--accent, #e49b0f)"
              stroke-width="1"
              rx="2"
            />
          );
        })()}
      </svg>
    </div>
  );
}
