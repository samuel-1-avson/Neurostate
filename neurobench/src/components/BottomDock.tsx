// BottomDock - VS Code-style bottom dock with tabbed panels
import { createSignal, Show, For, JSX } from "solid-js";

export interface DockTab {
  id: string;
  label: string;
  icon: string;
  content: () => JSX.Element;
}

interface BottomDockProps {
  tabs: DockTab[];
  activeTab: string | null;
  onTabChange: (tabId: string | null) => void;
  height?: number;
  onHeightChange?: (height: number) => void;
}

export function BottomDock(props: BottomDockProps) {
  const [isResizing, setIsResizing] = createSignal(false);
  const [dockHeight, setDockHeight] = createSignal(props.height || 200);
  const [isMinimized, setIsMinimized] = createSignal(false);

  const handleMouseDown = (e: MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
    
    const startY = e.clientY;
    const startHeight = dockHeight();
    
    const handleMouseMove = (e: MouseEvent) => {
      const delta = startY - e.clientY;
      const newHeight = Math.max(100, Math.min(500, startHeight + delta));
      setDockHeight(newHeight);
      props.onHeightChange?.(newHeight);
    };
    
    const handleMouseUp = () => {
      setIsResizing(false);
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
    
    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
  };

  const handleTabClick = (tabId: string) => {
    if (props.activeTab === tabId) {
      // Toggle minimize
      setIsMinimized(!isMinimized());
    } else {
      setIsMinimized(false);
      props.onTabChange(tabId);
    }
  };

  const activeTabContent = () => {
    const tab = props.tabs.find(t => t.id === props.activeTab);
    return tab?.content();
  };

  return (
    <div 
      class={`bottom-dock ${isMinimized() ? "minimized" : ""} ${isResizing() ? "resizing" : ""}`}
      style={{ height: isMinimized() ? "32px" : `${dockHeight()}px` }}
    >
      {/* Resize Handle */}
      <div class="dock-resize-handle" onMouseDown={handleMouseDown} />
      
      {/* Tab Bar */}
      <div class="dock-tabs">
        <For each={props.tabs}>
          {(tab) => (
            <button
              class={`dock-tab ${props.activeTab === tab.id ? "active" : ""}`}
              onClick={() => handleTabClick(tab.id)}
            >
              <span class="tab-icon">{tab.icon}</span>
              <span class="tab-label">{tab.label}</span>
            </button>
          )}
        </For>
        <div class="dock-tabs-spacer" />
        <button 
          class="dock-btn"
          onClick={() => setIsMinimized(!isMinimized())}
          title={isMinimized() ? "Expand" : "Minimize"}
        >
          {isMinimized() ? "▲" : "▼"}
        </button>
        <button 
          class="dock-btn"
          onClick={() => props.onTabChange(null)}
          title="Close All"
        >
          ✕
        </button>
      </div>
      
      {/* Content */}
      <Show when={!isMinimized() && props.activeTab}>
        <div class="dock-content">
          {activeTabContent()}
        </div>
      </Show>
    </div>
  );
}

export default BottomDock;
