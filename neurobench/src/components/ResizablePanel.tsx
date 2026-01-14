/**
 * ResizablePanel - Draggable resizable panel system
 * Professional IDE-style panel layout
 */

import { Component, createSignal, onMount, onCleanup, JSX } from "solid-js";
import "./ResizablePanel.css";

interface ResizablePanelProps {
  children: JSX.Element;
  defaultWidth?: number;
  minWidth?: number;
  maxWidth?: number;
  position?: "left" | "right";
  onResize?: (width: number) => void;
  collapsed?: boolean;
  onCollapse?: () => void;
}

export const ResizablePanel: Component<ResizablePanelProps> = (props) => {
  const [width, setWidth] = createSignal(props.defaultWidth || 300);
  const [isResizing, setIsResizing] = createSignal(false);
  
  let panelRef: HTMLDivElement | undefined;
  let startX = 0;
  let startWidth = 0;
  
  const minWidth = props.minWidth || 200;
  const maxWidth = props.maxWidth || 600;
  
  const handleMouseDown = (e: MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
    startX = e.clientX;
    startWidth = width();
    
    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
  };
  
  const handleMouseMove = (e: MouseEvent) => {
    if (!isResizing()) return;
    
    const diff = props.position === "right" 
      ? startX - e.clientX 
      : e.clientX - startX;
    
    let newWidth = startWidth + diff;
    newWidth = Math.max(minWidth, Math.min(maxWidth, newWidth));
    
    setWidth(newWidth);
    props.onResize?.(newWidth);
  };
  
  const handleMouseUp = () => {
    setIsResizing(false);
    document.removeEventListener("mousemove", handleMouseMove);
    document.removeEventListener("mouseup", handleMouseUp);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
  };
  
  onCleanup(() => {
    document.removeEventListener("mousemove", handleMouseMove);
    document.removeEventListener("mouseup", handleMouseUp);
  });

  return (
    <div 
      ref={panelRef}
      class="resizable-panel"
      classList={{ 
        resizing: isResizing(),
        collapsed: props.collapsed,
        left: props.position === "left",
        right: props.position === "right"
      }}
      style={{ width: props.collapsed ? "0px" : `${width()}px` }}
    >
      <div class="panel-content">
        {props.children}
      </div>
      
      {/* Resize Handle */}
      <div 
        class="resize-handle"
        classList={{ left: props.position === "right", right: props.position === "left" }}
        onMouseDown={handleMouseDown}
      >
        <div class="resize-grip" />
      </div>
    </div>
  );
};

// Split Panel Container
interface SplitPanelProps {
  children: JSX.Element[];
  direction?: "horizontal" | "vertical";
  defaultSizes?: number[];
  minSizes?: number[];
}

export const SplitPanel: Component<SplitPanelProps> = (props) => {
  const [sizes, setSizes] = createSignal<number[]>(props.defaultSizes || [50, 50]);
  const [isDragging, setIsDragging] = createSignal(false);
  const [dragIndex, setDragIndex] = createSignal(0);
  
  let containerRef: HTMLDivElement | undefined;
  
  const handleDividerMouseDown = (index: number) => (e: MouseEvent) => {
    e.preventDefault();
    setIsDragging(true);
    setDragIndex(index);
    
    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
    document.body.style.cursor = props.direction === "horizontal" ? "col-resize" : "row-resize";
    document.body.style.userSelect = "none";
  };
  
  const handleMouseMove = (e: MouseEvent) => {
    if (!isDragging() || !containerRef) return;
    
    const rect = containerRef.getBoundingClientRect();
    const isHorizontal = props.direction !== "vertical";
    const containerSize = isHorizontal ? rect.width : rect.height;
    const mousePos = isHorizontal ? e.clientX - rect.left : e.clientY - rect.top;
    
    const percentage = (mousePos / containerSize) * 100;
    const minSize = props.minSizes?.[dragIndex()] || 10;
    
    const newSizes = [...sizes()];
    newSizes[dragIndex()] = Math.max(minSize, Math.min(100 - minSize, percentage));
    newSizes[dragIndex() + 1] = 100 - newSizes[dragIndex()];
    
    setSizes(newSizes);
  };
  
  const handleMouseUp = () => {
    setIsDragging(false);
    document.removeEventListener("mousemove", handleMouseMove);
    document.removeEventListener("mouseup", handleMouseUp);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
  };

  return (
    <div 
      ref={containerRef}
      class="split-panel"
      classList={{ horizontal: props.direction !== "vertical", vertical: props.direction === "vertical" }}
    >
      {props.children.map((child, index) => (
        <>
          <div 
            class="split-pane"
            style={{ 
              [props.direction === "vertical" ? "height" : "width"]: `${sizes()[index]}%` 
            }}
          >
            {child}
          </div>
          {index < props.children.length - 1 && (
            <div 
              class="split-divider"
              classList={{ dragging: isDragging() && dragIndex() === index }}
              onMouseDown={handleDividerMouseDown(index)}
            />
          )}
        </>
      ))}
    </div>
  );
};

export default ResizablePanel;
