// Menubar Component - Application Menu with File, Edit, View, etc.
import { createSignal, Show, For } from "solid-js";
import { Icons } from "./AppIcons";

interface MenubarProps {
  onNewProject?: () => void;
  onOpenProject?: () => void;
  onSaveProject?: () => void;
  onExport?: (format: string) => void;
  onUndo?: () => void;
  onRedo?: () => void;
  onCut?: () => void;
  onCopy?: () => void;
  onPaste?: () => void;
  onZoomIn?: () => void;
  onZoomOut?: () => void;
  onResetZoom?: () => void;
  onToggleFullscreen?: () => void;
  onShowSettings?: () => void;
  onShowAbout?: () => void;
}

interface MenuItem {
  label: string;
  shortcut?: string;
  action?: () => void;
  divider?: boolean;
  submenu?: MenuItem[];
  disabled?: boolean;
}

export function Menubar(props: MenubarProps) {
  const [activeMenu, setActiveMenu] = createSignal<string | null>(null);

  const menus: Record<string, MenuItem[]> = {
    File: [
      { label: "New Project", shortcut: "Ctrl+N", action: props.onNewProject },
      { label: "Open Project...", shortcut: "Ctrl+O", action: props.onOpenProject },
      { label: "Open Recent", submenu: [
        { label: "project1.nbp" },
        { label: "demo_fsm.nbp" },
        { label: "Clear Recent" },
      ]},
      { label: "", divider: true },
      { label: "Save", shortcut: "Ctrl+S", action: props.onSaveProject },
      { label: "Save As...", shortcut: "Ctrl+Shift+S" },
      { label: "", divider: true },
      { label: "Export", submenu: [
        { label: "Export to C/C++", action: () => props.onExport?.("c") },
        { label: "Export to Rust", action: () => props.onExport?.("rust") },
        { label: "Export to STM32CubeIDE" },
        { label: "Export to PlatformIO" },
      ]},
      { label: "Import...", shortcut: "Ctrl+I" },
      { label: "", divider: true },
      { label: "Exit" },
    ],
    Edit: [
      { label: "Undo", shortcut: "Ctrl+Z", action: props.onUndo },
      { label: "Redo", shortcut: "Ctrl+Y", action: props.onRedo },
      { label: "", divider: true },
      { label: "Cut", shortcut: "Ctrl+X", action: props.onCut },
      { label: "Copy", shortcut: "Ctrl+C", action: props.onCopy },
      { label: "Paste", shortcut: "Ctrl+V", action: props.onPaste },
      { label: "Duplicate", shortcut: "Ctrl+D" },
      { label: "", divider: true },
      { label: "Select All", shortcut: "Ctrl+A" },
      { label: "Delete Selected", shortcut: "Del" },
    ],
    View: [
      { label: "Zoom In", shortcut: "Ctrl++", action: props.onZoomIn },
      { label: "Zoom Out", shortcut: "Ctrl+-", action: props.onZoomOut },
      { label: "Reset Zoom", shortcut: "Ctrl+0", action: props.onResetZoom },
      { label: "", divider: true },
      { label: "Toggle Grid" },
      { label: "Toggle Minimap" },
      { label: "Toggle Snap to Grid" },
      { label: "", divider: true },
      { label: "Show Left Panel" },
      { label: "Show Right Panel" },
      { label: "Show Bottom Panel" },
      { label: "", divider: true },
      { label: "Fullscreen", shortcut: "F11", action: props.onToggleFullscreen },
    ],
    Canvas: [
      { label: "Add State Node" },
      { label: "Add GPIO Node" },
      { label: "Add Timer Node" },
      { label: "", divider: true },
      { label: "Auto Layout", submenu: [
        { label: "Hierarchical" },
        { label: "Force-Directed" },
        { label: "Grid" },
      ]},
      { label: "Validate Graph" },
      { label: "", divider: true },
      { label: "Clear Canvas" },
    ],
    Build: [
      { label: "Build Project", shortcut: "Ctrl+B" },
      { label: "Clean Build" },
      { label: "Rebuild All" },
      { label: "", divider: true },
      { label: "Flash to Device", shortcut: "Ctrl+F" },
      { label: "Debug", shortcut: "F5" },
      { label: "", divider: true },
      { label: "Build Settings..." },
    ],
    Tools: [
      { label: "AI Assistant", shortcut: "Ctrl+Shift+A" },
      { label: "Code Generator" },
      { label: "Driver Generator" },
      { label: "", divider: true },
      { label: "Serial Monitor" },
      { label: "Logic Analyzer" },
      { label: "Memory Inspector" },
      { label: "", divider: true },
      { label: "Settings...", action: props.onShowSettings },
    ],
    Help: [
      { label: "Documentation" },
      { label: "Keyboard Shortcuts", shortcut: "Ctrl+K" },
      { label: "Getting Started Tutorial" },
      { label: "", divider: true },
      { label: "Check for Updates" },
      { label: "Report Issue" },
      { label: "", divider: true },
      { label: "About NeuroBench", action: props.onShowAbout },
    ],
  };

  const handleMenuClick = (menuName: string) => {
    setActiveMenu(activeMenu() === menuName ? null : menuName);
  };

  const handleMenuItemClick = (item: MenuItem) => {
    if (item.action && !item.disabled) {
      item.action();
    }
    setActiveMenu(null);
  };

  const handleMouseLeave = () => {
    setActiveMenu(null);
  };

  return (
    <div class="menubar" onMouseLeave={handleMouseLeave}>
      <For each={Object.keys(menus)}>
        {(menuName) => (
          <div class="menu-container">
            <button 
              class={`menu-trigger ${activeMenu() === menuName ? "active" : ""}`}
              onClick={() => handleMenuClick(menuName)}
              onMouseEnter={() => activeMenu() && setActiveMenu(menuName)}
            >
              {menuName}
            </button>
            <Show when={activeMenu() === menuName}>
              <div class="menu-dropdown">
                <For each={menus[menuName]}>
                  {(item) => (
                    <Show when={!item.divider} fallback={<div class="menu-divider" />}>
                      <button 
                        class={`menu-item ${item.disabled ? "disabled" : ""}`}
                        onClick={() => handleMenuItemClick(item)}
                      >
                        <span class="menu-item-label">{item.label}</span>
                        <Show when={item.submenu}>
                          <span class="menu-item-arrow">{Icons.chevronRight()}</span>
                        </Show>
                        <Show when={item.shortcut}>
                          <span class="menu-item-shortcut">{item.shortcut}</span>
                        </Show>
                      </button>
                    </Show>
                  )}
                </For>
              </div>
            </Show>
          </div>
        )}
      </For>
    </div>
  );
}

export default Menubar;
