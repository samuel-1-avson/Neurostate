/**
 * IndustrialMenuBar - Professional industrial-grade application menubar
 * Includes File, Edit, View, Build, Simulation, Tools, and Help menus
 */

import { Component, createSignal, For, Show, onMount, onCleanup } from "solid-js";
import "./IndustrialMenuBar.css";

interface MenuItem {
  label: string;
  shortcut?: string;
  action?: () => void;
  disabled?: boolean;
  divider?: boolean;
  submenu?: MenuItem[];
  icon?: string;
  checked?: boolean;
}

interface MenuGroup {
  label: string;
  items: MenuItem[];
}

interface IndustrialMenuBarProps {
  projectName: string;
  targetMcu: string;
  simStatus: "idle" | "running" | "paused" | "stopped";
  onNewProject: () => void;
  onOpenProject: () => void;
  onSaveProject: () => void;
  onSaveAs?: () => void;
  onExport?: () => void;
  onUndo?: () => void;
  onRedo?: () => void;
  onCut?: () => void;
  onCopy?: () => void;
  onPaste?: () => void;
  onDelete?: () => void;
  onSelectAll?: () => void;
  onZoomIn?: () => void;
  onZoomOut?: () => void;
  onZoomReset?: () => void;
  onToggleGrid?: () => void;
  onToggleMinimap?: () => void;
  onToggleRulers?: () => void;
  onBuild?: () => void;
  onClean?: () => void;
  onFlash?: () => void;
  onSimStart?: () => void;
  onSimPause?: () => void;
  onSimStop?: () => void;
  onSimStep?: () => void;
  onSimReset?: () => void;
  onGenerateCode?: () => void;
  onValidate?: () => void;
  onAutoLayout?: (type: string) => void;
  onDetectDevices?: () => void;
  onSettings?: () => void;
  onHelp?: () => void;
  onAbout?: () => void;
  onTargetChange?: (mcu: string) => void;
}

export const IndustrialMenuBar: Component<IndustrialMenuBarProps> = (props) => {
  const [activeMenu, setActiveMenu] = createSignal<string | null>(null);
  
  let menuBarRef: HTMLDivElement | undefined;

  // Close menu when clicking outside
  const handleClickOutside = (e: MouseEvent) => {
    if (menuBarRef && !menuBarRef.contains(e.target as Node)) {
      setActiveMenu(null);
    }
  };

  onMount(() => {
    document.addEventListener("click", handleClickOutside);
  });

  onCleanup(() => {
    document.removeEventListener("click", handleClickOutside);
  });

  const handleMenuClick = (menuId: string) => {
    setActiveMenu(activeMenu() === menuId ? null : menuId);
  };

  const handleMenuHover = (menuId: string) => {
    if (activeMenu() !== null) {
      setActiveMenu(menuId);
    }
  };

  const handleItemClick = (item: MenuItem) => {
    if (item.action && !item.disabled) {
      item.action();
      setActiveMenu(null);
    }
  };

  const menus: MenuGroup[] = [
    {
      label: "File",
      items: [
        { label: "New Project", shortcut: "Ctrl+N", icon: "📄", action: props.onNewProject },
        { label: "Open Project...", shortcut: "Ctrl+O", icon: "📂", action: props.onOpenProject },
        { divider: true, label: "" },
        { label: "Save", shortcut: "Ctrl+S", icon: "💾", action: props.onSaveProject },
        { label: "Save As...", shortcut: "Ctrl+Shift+S", icon: "📑", action: props.onSaveAs },
        { divider: true, label: "" },
        { label: "Export Code...", shortcut: "Ctrl+E", icon: "📤", action: props.onExport },
        { divider: true, label: "" },
        { label: "Settings", shortcut: "Ctrl+,", icon: "⚙️", action: props.onSettings },
      ],
    },
    {
      label: "Edit",
      items: [
        { label: "Undo", shortcut: "Ctrl+Z", icon: "↩️", action: props.onUndo },
        { label: "Redo", shortcut: "Ctrl+Y", icon: "↪️", action: props.onRedo },
        { divider: true, label: "" },
        { label: "Cut", shortcut: "Ctrl+X", icon: "✂️", action: props.onCut },
        { label: "Copy", shortcut: "Ctrl+C", icon: "📋", action: props.onCopy },
        { label: "Paste", shortcut: "Ctrl+V", icon: "📥", action: props.onPaste },
        { label: "Delete", shortcut: "Del", icon: "🗑️", action: props.onDelete },
        { divider: true, label: "" },
        { label: "Select All", shortcut: "Ctrl+A", icon: "☑️", action: props.onSelectAll },
      ],
    },
    {
      label: "View",
      items: [
        { label: "Zoom In", shortcut: "Ctrl++", icon: "🔍", action: props.onZoomIn },
        { label: "Zoom Out", shortcut: "Ctrl+-", icon: "🔎", action: props.onZoomOut },
        { label: "Reset Zoom", shortcut: "Ctrl+0", icon: "🔄", action: props.onZoomReset },
        { divider: true, label: "" },
        { label: "Toggle Grid", shortcut: "Ctrl+G", icon: "⊞", action: props.onToggleGrid },
        { label: "Toggle Minimap", shortcut: "Ctrl+M", icon: "🗺️", action: props.onToggleMinimap },
        { label: "Toggle Rulers", shortcut: "Ctrl+R", icon: "📏", action: props.onToggleRulers },
        { divider: true, label: "" },
        { label: "Auto-Layout: Hierarchical", icon: "📊", action: () => props.onAutoLayout?.("hierarchical") },
        { label: "Auto-Layout: Force-Directed", icon: "🕸️", action: () => props.onAutoLayout?.("force_directed") },
        { label: "Auto-Layout: Grid", icon: "▦", action: () => props.onAutoLayout?.("grid") },
      ],
    },
    {
      label: "Build",
      items: [
        { label: "Generate Code", shortcut: "F5", icon: "⚡", action: props.onGenerateCode },
        { label: "Validate Graph", shortcut: "F6", icon: "✅", action: props.onValidate },
        { divider: true, label: "" },
        { label: "Build Project", shortcut: "Ctrl+B", icon: "🔨", action: props.onBuild },
        { label: "Clean Build", icon: "🧹", action: props.onClean },
        { divider: true, label: "" },
        { label: "Flash to Device", shortcut: "Ctrl+F", icon: "⚡", action: props.onFlash },
        { label: "Detect Devices", icon: "🔌", action: props.onDetectDevices },
      ],
    },
    {
      label: "Simulation",
      items: [
        { 
          label: props.simStatus === "running" ? "Pause" : "Start", 
          shortcut: "F9", 
          icon: props.simStatus === "running" ? "⏸️" : "▶️", 
          action: props.simStatus === "running" ? props.onSimPause : props.onSimStart 
        },
        { label: "Stop", shortcut: "Shift+F9", icon: "⏹️", action: props.onSimStop, disabled: props.simStatus === "idle" },
        { label: "Step", shortcut: "F10", icon: "⏭️", action: props.onSimStep },
        { label: "Reset", shortcut: "Ctrl+Shift+R", icon: "🔄", action: props.onSimReset },
      ],
    },
    {
      label: "Help",
      items: [
        { label: "Documentation", shortcut: "F1", icon: "📖", action: props.onHelp },
        { label: "Keyboard Shortcuts", icon: "⌨️", action: props.onHelp },
        { divider: true, label: "" },
        { label: "About NeuroBench", icon: "ℹ️", action: props.onAbout },
      ],
    },
  ];

  return (
    <div class="industrial-menubar" ref={menuBarRef}>
      {/* Logo */}
      <div class="menubar-logo">
        <span class="logo-icon">⬡</span>
        <span class="logo-text">NEUROBENCH</span>
      </div>

      <div class="menubar-divider" />

      {/* Menu Items */}
      <div class="menubar-menus">
        <For each={menus}>
          {(menu) => (
            <div 
              class="menu-trigger"
              classList={{ active: activeMenu() === menu.label }}
              onClick={() => handleMenuClick(menu.label)}
              onMouseEnter={() => handleMenuHover(menu.label)}
            >
              {menu.label}
              <Show when={activeMenu() === menu.label}>
                <div class="menu-dropdown">
                  <For each={menu.items}>
                    {(item) => (
                      item.divider ? (
                        <div class="menu-divider" />
                      ) : (
                        <div 
                          class="menu-item"
                          classList={{ disabled: item.disabled }}
                          onClick={(e) => { e.stopPropagation(); handleItemClick(item); }}
                        >
                          <span class="item-icon">{item.icon}</span>
                          <span class="item-label">{item.label}</span>
                          <Show when={item.shortcut}>
                            <span class="item-shortcut">{item.shortcut}</span>
                          </Show>
                        </div>
                      )
                    )}
                  </For>
                </div>
              </Show>
            </div>
          )}
        </For>
      </div>

      <div class="menubar-spacer" />

      {/* Quick Action Buttons */}
      <div class="menubar-actions">
        <button class="action-btn" onClick={props.onSaveProject} title="Save (Ctrl+S)">
          <span>💾</span>
        </button>
        <button class="action-btn" onClick={props.onGenerateCode} title="Generate Code (F5)">
          <span>⚡</span>
        </button>
        <button 
          class="action-btn sim-btn"
          classList={{ 
            running: props.simStatus === "running",
            paused: props.simStatus === "paused"
          }}
          onClick={props.simStatus === "running" ? props.onSimPause : props.onSimStart}
          title={props.simStatus === "running" ? "Pause (F9)" : "Start Simulation (F9)"}
        >
          <span>{props.simStatus === "running" ? "⏸️" : "▶️"}</span>
        </button>
        <button 
          class="action-btn danger"
          onClick={props.onSimStop}
          disabled={props.simStatus === "idle"}
          title="Stop Simulation"
        >
          <span>⏹️</span>
        </button>
      </div>

      <div class="menubar-divider" />

      {/* Status Indicators */}
      <div class="menubar-status">
        <div class="status-indicator">
          <span class={`status-led ${props.simStatus}`} />
          <span class="status-text">{props.simStatus.toUpperCase()}</span>
        </div>
      </div>

      <div class="menubar-divider" />

      {/* MCU Selector */}
      <div class="menubar-mcu">
        <select 
          class="mcu-select" 
          value={props.targetMcu}
          onChange={(e) => props.onTargetChange?.(e.currentTarget.value)}
        >
          <option value="STM32F401">STM32F401</option>
          <option value="STM32F103">STM32F103</option>
          <option value="STM32F407">STM32F407</option>
          <option value="STM32H743">STM32H743</option>
          <option value="ESP32">ESP32</option>
          <option value="ESP32-S3">ESP32-S3</option>
          <option value="RP2040">RP2040</option>
          <option value="nRF52840">nRF52840</option>
          <option value="ATmega328P">ATmega328P</option>
          <option value="ATmega2560">ATmega2560</option>
        </select>
      </div>
    </div>
  );
};

export default IndustrialMenuBar;
