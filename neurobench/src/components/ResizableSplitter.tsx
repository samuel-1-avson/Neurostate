import { createSignal, onCleanup } from "solid-js";
import "./ResizableSplitter.css";

interface ResizableSplitterProps {
  direction?: "horizontal" | "vertical";
  onResize?: (newSize: number) => void;
  minSize?: number;
  initialSize?: number; // Needed if we want the splitter to control state internally or hint
}

export function ResizableSplitter(props: ResizableSplitterProps) {
  const [isDragging, setIsDragging] = createSignal(false);
  const direction = props.direction || "horizontal";

  const handleMouseDown = (e: MouseEvent) => {
    e.preventDefault();
    setIsDragging(true);
    document.body.style.cursor = direction === "horizontal" ? "col-resize" : "row-resize";
    document.body.style.userSelect = "none";
    
    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (!isDragging()) return;
    
    // We emit the new position/size. 
    // Ideally, the parent component handles the actual sizing logic based on mouse position.
    // But a simpler way for a "Splitter" is to just emit the raw movement or absolute position.
    // Let's assume the parent passes a callback that accepts the clientX/Y or delta.
    
    // However, a standard pattern is:
    // The parent has `[width, setWidth]`.
    // The splitter just reports the `e.clientX` or `e.clientY`.
    
    // Alternatively, we can calculate the delta if we tracked start position.
    // For now, let's just pass `e.clientX` or `e.clientY` to the onResize handler, 
    // and let the parent calculate the specific width based on its own context.
    
    if (props.onResize) {
      props.onResize(direction === "horizontal" ? e.clientX : e.clientY);
    }
  };

  const handleMouseUp = () => {
    setIsDragging(false);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    window.removeEventListener("mousemove", handleMouseMove);
    window.removeEventListener("mouseup", handleMouseUp);
  };

  onCleanup(() => {
    window.removeEventListener("mousemove", handleMouseMove);
    window.removeEventListener("mouseup", handleMouseUp);
  });

  return (
    <div
      class={`resizable-splitter ${direction} ${isDragging() ? "dragging" : ""}`}
      onMouseDown={handleMouseDown}
    >
      <div class="splitter-handle" />
    </div>
  );
}
