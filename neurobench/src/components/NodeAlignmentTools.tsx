/**
 * NodeAlignmentTools - Canvas node alignment toolbar
 * Professional design tools for node positioning
 */

import { Component, createSignal, Show } from "solid-js";
import "./NodeAlignmentTools.css";

interface Node {
  id: string;
  x: number;
  y: number;
  width?: number;
  height?: number;
}

interface NodeAlignmentToolsProps {
  selectedNodes: Node[];
  onAlign: (nodes: Node[]) => void;
  onDistribute: (nodes: Node[]) => void;
  visible?: boolean;
}

export const NodeAlignmentTools: Component<NodeAlignmentToolsProps> = (props) => {
  const [showTooltip, setShowTooltip] = createSignal<string | null>(null);
  
  const hasSelection = () => props.selectedNodes.length > 0;
  const hasMultiple = () => props.selectedNodes.length > 1;
  
  const getNodeWidth = (node: Node) => node.width || 120;
  const getNodeHeight = (node: Node) => node.height || 60;
  
  // Alignment functions
  const alignLeft = () => {
    if (!hasMultiple()) return;
    const minX = Math.min(...props.selectedNodes.map(n => n.x));
    const aligned = props.selectedNodes.map(n => ({ ...n, x: minX }));
    props.onAlign(aligned);
  };
  
  const alignCenter = () => {
    if (!hasMultiple()) return;
    const centers = props.selectedNodes.map(n => n.x + getNodeWidth(n) / 2);
    const avgCenter = centers.reduce((a, b) => a + b, 0) / centers.length;
    const aligned = props.selectedNodes.map(n => ({ 
      ...n, 
      x: avgCenter - getNodeWidth(n) / 2 
    }));
    props.onAlign(aligned);
  };
  
  const alignRight = () => {
    if (!hasMultiple()) return;
    const maxRight = Math.max(...props.selectedNodes.map(n => n.x + getNodeWidth(n)));
    const aligned = props.selectedNodes.map(n => ({ 
      ...n, 
      x: maxRight - getNodeWidth(n) 
    }));
    props.onAlign(aligned);
  };
  
  const alignTop = () => {
    if (!hasMultiple()) return;
    const minY = Math.min(...props.selectedNodes.map(n => n.y));
    const aligned = props.selectedNodes.map(n => ({ ...n, y: minY }));
    props.onAlign(aligned);
  };
  
  const alignMiddle = () => {
    if (!hasMultiple()) return;
    const middles = props.selectedNodes.map(n => n.y + getNodeHeight(n) / 2);
    const avgMiddle = middles.reduce((a, b) => a + b, 0) / middles.length;
    const aligned = props.selectedNodes.map(n => ({ 
      ...n, 
      y: avgMiddle - getNodeHeight(n) / 2 
    }));
    props.onAlign(aligned);
  };
  
  const alignBottom = () => {
    if (!hasMultiple()) return;
    const maxBottom = Math.max(...props.selectedNodes.map(n => n.y + getNodeHeight(n)));
    const aligned = props.selectedNodes.map(n => ({ 
      ...n, 
      y: maxBottom - getNodeHeight(n) 
    }));
    props.onAlign(aligned);
  };
  
  // Distribution functions
  const distributeHorizontally = () => {
    if (props.selectedNodes.length < 3) return;
    const sorted = [...props.selectedNodes].sort((a, b) => a.x - b.x);
    const leftMost = sorted[0].x;
    const rightMost = sorted[sorted.length - 1].x + getNodeWidth(sorted[sorted.length - 1]);
    const totalWidth = sorted.reduce((sum, n) => sum + getNodeWidth(n), 0);
    const spacing = (rightMost - leftMost - totalWidth) / (sorted.length - 1);
    
    let currentX = leftMost;
    const distributed = sorted.map(node => {
      const newNode = { ...node, x: currentX };
      currentX += getNodeWidth(node) + spacing;
      return newNode;
    });
    
    props.onDistribute(distributed);
  };
  
  const distributeVertically = () => {
    if (props.selectedNodes.length < 3) return;
    const sorted = [...props.selectedNodes].sort((a, b) => a.y - b.y);
    const topMost = sorted[0].y;
    const bottomMost = sorted[sorted.length - 1].y + getNodeHeight(sorted[sorted.length - 1]);
    const totalHeight = sorted.reduce((sum, n) => sum + getNodeHeight(n), 0);
    const spacing = (bottomMost - topMost - totalHeight) / (sorted.length - 1);
    
    let currentY = topMost;
    const distributed = sorted.map(node => {
      const newNode = { ...node, y: currentY };
      currentY += getNodeHeight(node) + spacing;
      return newNode;
    });
    
    props.onDistribute(distributed);
  };
  
  const buttons = [
    { id: "align-left", icon: "⫷", label: "Align Left", action: alignLeft, requiresMultiple: true },
    { id: "align-center", icon: "⫼", label: "Align Center", action: alignCenter, requiresMultiple: true },
    { id: "align-right", icon: "⫸", label: "Align Right", action: alignRight, requiresMultiple: true },
    { id: "divider-1", divider: true },
    { id: "align-top", icon: "⫯", label: "Align Top", action: alignTop, requiresMultiple: true },
    { id: "align-middle", icon: "⫰", label: "Align Middle", action: alignMiddle, requiresMultiple: true },
    { id: "align-bottom", icon: "⫱", label: "Align Bottom", action: alignBottom, requiresMultiple: true },
    { id: "divider-2", divider: true },
    { id: "dist-h", icon: "⇹", label: "Distribute Horizontally", action: distributeHorizontally, requiresThree: true },
    { id: "dist-v", icon: "⇵", label: "Distribute Vertically", action: distributeVertically, requiresThree: true },
  ];

  return (
    <Show when={props.visible !== false && hasSelection()}>
      <div class="alignment-toolbar">
        {buttons.map((btn) => (
          btn.divider ? (
            <div class="toolbar-divider" />
          ) : (
            <button
              class="alignment-btn"
              classList={{ 
                disabled: btn.requiresMultiple && !hasMultiple() || 
                          btn.requiresThree && props.selectedNodes.length < 3 
              }}
              onClick={btn.action}
              onMouseEnter={() => setShowTooltip(btn.id)}
              onMouseLeave={() => setShowTooltip(null)}
              disabled={btn.requiresMultiple && !hasMultiple() || 
                        btn.requiresThree && props.selectedNodes.length < 3}
              title={btn.label}
            >
              <span class="btn-icon">{btn.icon}</span>
              <Show when={showTooltip() === btn.id}>
                <div class="btn-tooltip">{btn.label}</div>
              </Show>
            </button>
          )
        ))}
        
        <div class="toolbar-divider" />
        
        <div class="selection-info">
          <span class="selection-count">{props.selectedNodes.length}</span>
          <span class="selection-label">selected</span>
        </div>
      </div>
    </Show>
  );
};

export default NodeAlignmentTools;
