/**
 * IndustrialMenuBar - Professional industrial-grade application menubar
 * Includes File, Edit, View, Build, Simulation, Tools, and Help menus
 */

import { Component, createSignal, For, Show, onMount, onCleanup } from "solid-js";
<<<<<<< HEAD
=======
import { Icons } from "./AppIcons";
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
import "./IndustrialMenuBar.css";

interface MenuItem {
  label: string;
  shortcut?: string;
  action?: () => void;
  disabled?: boolean;
  divider?: boolean;
  submenu?: MenuItem[];
<<<<<<< HEAD
  icon?: string;
=======
  icon?: any;
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
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
<<<<<<< HEAD
=======
  // Layout customization
  onCustomizeLayout?: () => void;
  onToggleActivityBar?: () => void;
  onTogglePrimarySideBar?: () => void;
  onTogglePanel?: () => void;
  onToggleStatusBar?: () => void;
  showActivityBar?: boolean;
  showPrimarySideBar?: boolean;
  showPanel?: boolean;
  showStatusBar?: boolean;
  // New Layout & Tools
  onToggleRightPanel?: () => void;
  showRightPanel?: boolean;
  onOpenAgentManager?: () => void;
  onQuickOpen?: () => void;
  onToggleLineComment?: () => void;
  onToggleBlockComment?: () => void;
  onFind?: () => void;
  onReplace?: () => void;
  onFindInFiles?: () => void;
  onReplaceInFiles?: () => void;
  onEmmetExpand?: () => void;
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
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

<<<<<<< HEAD
=======
// Revised Menu Item Component for recursion
  const MenuDropdownItem: Component<{ item: MenuItem }> = (props) => {
    const item = props.item;
    
    if (item.divider) {
      return <div class="menu-divider" />;
    }
    
    return (
      <div 
        class="menu-item"
        classList={{ disabled: item.disabled, checked: item.checked }}
        onClick={(e) => { 
          e.stopPropagation(); 
          if (!item.submenu) handleItemClick(item); 
        }}
      >
        <span class="item-toggle">
          <Show when={item.checked !== undefined}>
            <span class={`toggle-indicator ${item.checked ? 'active' : ''}`} />
          </Show>
        </span>
        <span class="item-icon">{item.icon}</span>
        <span class="item-label">{item.label}</span>
        <Show when={item.shortcut}>
          <span class="item-shortcut">{item.shortcut}</span>
        </Show>
        <Show when={item.submenu}>
          <span class="submenu-arrow">▶</span>
          <div class="menu-submenu">
             <For each={item.submenu}>
               {(subItem) => <MenuDropdownItem item={subItem} />}
             </For>
          </div>
        </Show>
      </div>
    );
  };

>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
  const menus: MenuGroup[] = [
    {
      label: "File",
      items: [
<<<<<<< HEAD
        { label: "New Project", shortcut: "Ctrl+N", icon: "📄", action: props.onNewProject },
        { label: "Open Project...", shortcut: "Ctrl+O", icon: "📂", action: props.onOpenProject },
        { divider: true, label: "" },
        { label: "Save", shortcut: "Ctrl+S", icon: "💾", action: props.onSaveProject },
        { label: "Save As...", shortcut: "Ctrl+Shift+S", icon: "📑", action: props.onSaveAs },
        { divider: true, label: "" },
        { label: "Export Code...", shortcut: "Ctrl+E", icon: "📤", action: props.onExport },
        { divider: true, label: "" },
        { label: "Settings", shortcut: "Ctrl+,", icon: "⚙️", action: props.onSettings },
=======
        { label: "New Project", shortcut: "Ctrl+N", icon: Icons.newFile(), action: props.onNewProject },
        { label: "Open Project...", shortcut: "Ctrl+O", icon: Icons.folder(), action: props.onOpenProject },
        { divider: true, label: "" },
        { label: "Save", shortcut: "Ctrl+S", icon: Icons.save(), action: props.onSaveProject },
        { label: "Save As...", shortcut: "Ctrl+Shift+S", icon: Icons.save(), action: props.onSaveAs },
        { divider: true, label: "" },
        { label: "Export Code...", shortcut: "Ctrl+E", icon: Icons.exportFile(), action: props.onExport },
        { divider: true, label: "" },
        { label: "Settings", shortcut: "Ctrl+,", icon: Icons.settings(), action: props.onSettings },
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
      ],
    },
    {
      label: "Edit",
      items: [
<<<<<<< HEAD
        { label: "Undo", shortcut: "Ctrl+Z", icon: "↩️", action: props.onUndo },
        { label: "Redo", shortcut: "Ctrl+Y", icon: "↪️", action: props.onRedo },
        { divider: true, label: "" },
        { label: "Cut", shortcut: "Ctrl+X", icon: "✂️", action: props.onCut },
        { label: "Copy", shortcut: "Ctrl+C", icon: "📋", action: props.onCopy },
        { label: "Paste", shortcut: "Ctrl+V", icon: "📥", action: props.onPaste },
        { label: "Delete", shortcut: "Del", icon: "🗑️", action: props.onDelete },
        { divider: true, label: "" },
        { label: "Select All", shortcut: "Ctrl+A", icon: "☑️", action: props.onSelectAll },
=======
        { label: "Undo", shortcut: "Ctrl+Z", action: props.onUndo },
        { label: "Redo", shortcut: "Ctrl+Y", action: props.onRedo },
        { divider: true, label: "" },
        { label: "Cut", shortcut: "Ctrl+X", action: props.onCut },
        { label: "Copy", shortcut: "Ctrl+C", action: props.onCopy },
        { label: "Paste", shortcut: "Ctrl+V", action: props.onPaste },
        { divider: true, label: "" },
        { label: "Find", shortcut: "Ctrl+F", action: props.onFind },
        { label: "Replace", shortcut: "Ctrl+H", action: props.onReplace },
        { divider: true, label: "" },
        { label: "Find in Files", shortcut: "Ctrl+Shift+F", action: props.onFindInFiles },
        { label: "Replace in Files", shortcut: "Ctrl+Shift+H", action: props.onReplaceInFiles },
        { divider: true, label: "" },
        { label: "Toggle Line Comment", shortcut: "Ctrl+/", action: props.onToggleLineComment },
        { label: "Toggle Block Comment", shortcut: "Shift+Alt+A", action: props.onToggleBlockComment },
        { label: "Emmet: Expand Abbreviation", shortcut: "Tab", action: props.onEmmetExpand },
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
      ],
    },
    {
      label: "View",
      items: [
<<<<<<< HEAD
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
=======
        { label: "Command Palette...", shortcut: "Ctrl+Shift+P", icon: Icons.terminal(), action: props.onQuickOpen },
        { label: "Open View...", icon: Icons.layout() },
        { divider: true, label: "" },
        { 
            label: "Appearance", 
            icon: Icons.layout(),
            submenu: [
                { label: "Full Screen", shortcut: "F11", action: () => document.documentElement.requestFullscreen().catch(() => document.exitFullscreen()) },
                { label: "Zen Mode", shortcut: "Ctrl+K Z" },
                { label: "Centered Layout" },
                { divider: true, label: "" },
                { label: "Menu Bar", checked: true },
                { label: "Primary Side Bar", shortcut: "Ctrl+B", checked: props.showPrimarySideBar, action: props.onTogglePrimarySideBar },
                { label: "Secondary Side Bar", checked: props.showRightPanel, action: props.onToggleRightPanel },
                { label: "Status Bar", checked: props.showStatusBar, action: props.onToggleStatusBar },
                { label: "Panel", shortcut: "Ctrl+J", checked: props.showPanel, action: props.onTogglePanel },
                { divider: true, label: "" },
                { label: "Move Primary Side Bar Right" },
                { label: "Activity Bar Position", submenu: [{ label: "Left" }, { label: "Right" }, { label: "Hidden" }] },
                { label: "Panel Position", submenu: [{ label: "Bottom" }, { label: "Left" }, { label: "Right" }] },
                { divider: true, label: "" },
                { label: "Minimap", checked: props.onToggleMinimap ? true : false, action: props.onToggleMinimap }, // Assuming state logic handles check status, passing generic true for visualization if no direct state prop
                { label: "Breadcrumbs", checked: true },
                { label: "Sticky Scroll", checked: true },
                { divider: true, label: "" },
                { label: "Zoom In", shortcut: "Ctrl++", icon: Icons.zoomIn(), action: props.onZoomIn },
                { label: "Zoom Out", shortcut: "Ctrl+-", icon: Icons.zoomOut(), action: props.onZoomOut },
                { label: "Reset Zoom", shortcut: "Ctrl+0", icon: Icons.fitView(), action: props.onZoomReset },
            ]
        },
        { 
            label: "Editor Layout", 
            icon: Icons.layout(),
            submenu: [
                { label: "Split Up", action: () => props.onAutoLayout?.("hierarchical") }, // Temporary mapping to layout actions
                { label: "Split Down" },
                { label: "Split Left" },
                { label: "Split Right" },
                { divider: true, label: "" },
                { label: "Single" },
                { label: "Two Columns" },
                { label: "Grid (2x2)" },
            ]
        },
        { divider: true, label: "" },
        { label: "Explorer", shortcut: "Ctrl+Shift+E", icon: Icons.folder(), action: () => props.onTogglePrimarySideBar?.() },
        { label: "Search", shortcut: "Ctrl+Shift+F", icon: Icons.search(), action: props.onQuickOpen },
        { label: "Source Control", shortcut: "Ctrl+Shift+G", icon: Icons.gitBranch() },
        { label: "Run", shortcut: "Ctrl+Shift+D", icon: Icons.debug() },
        { label: "Extensions", shortcut: "Ctrl+Shift+X", icon: Icons.plug() },
        { label: "Testing", icon: Icons.check() },
        { divider: true, label: "" },
        { label: "Problems", shortcut: "Ctrl+Shift+M", icon: Icons.warning() },
        { label: "Output", shortcut: "Ctrl+Shift+U", icon: Icons.terminal() },
        { label: "Debug Console", shortcut: "Ctrl+Shift+Y", icon: Icons.debug() },
        { label: "Terminal", shortcut: "Ctrl+`", icon: Icons.terminal(), action: props.onTogglePanel },
        { divider: true, label: "" },
        { label: "Word Wrap", shortcut: "Alt+Z", checked: true },
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
      ],
    },
    {
      label: "Build",
      items: [
<<<<<<< HEAD
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
=======
        { label: "Generate Code", shortcut: "F5", icon: Icons.code(), action: props.onGenerateCode },
        { label: "Validate Graph", shortcut: "F6", icon: Icons.check(), action: props.onValidate },
        { divider: true, label: "" },
        { label: "Build Project", shortcut: "Ctrl+B", icon: Icons.build(), action: props.onBuild },
        { label: "Clean Build", icon: Icons.clean(), action: props.onClean },
        { divider: true, label: "" },
        { label: "Flash to Device", shortcut: "Ctrl+F", icon: Icons.flash(), action: props.onFlash },
        { label: "Detect Devices", icon: Icons.plug(), action: props.onDetectDevices },
      ],
    },

    {
      label: "Help",
      items: [
        { label: "Documentation", shortcut: "F1", icon: Icons.docs(), action: props.onHelp },
        { label: "Keyboard Shortcuts", icon: Icons.keyboard(), action: props.onHelp },
        { divider: true, label: "" },
        { label: "About NeuroBench", icon: Icons.info(), action: props.onAbout },
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
      ],
    },
  ];

  return (
    <div class="industrial-menubar" ref={menuBarRef}>
<<<<<<< HEAD
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
=======
      {/* Left Section: Menu Items */}
      <div class="menubar-left">
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
                      {(item) => <MenuDropdownItem item={item} />}
                    </For>
                  </div>
                </Show>
              </div>
            )}
          </For>
        </div>
      </div>

      {/* Center Section: Logo Removed */}
      <div class="menubar-center">
      </div>

      {/* Right Section: Actions, Status, MCU */}
      <div class="menubar-right">
        {/* Quick Action Buttons */}
        <div class="menubar-actions">
          {/* Layout Toggles */}
          <button 
            class={`action-btn ${props.showPrimarySideBar ? 'active' : ''}`} 
            onClick={props.onTogglePrimarySideBar} 
            title="Toggle Primary Side Bar (Ctrl+B)"
          >
            <span>{Icons.panelLeft()}</span>
          </button>
          
          <button 
            class={`action-btn ${props.showPanel ? 'active' : ''}`} 
            onClick={props.onTogglePanel} 
            title="Toggle Panel (Ctrl+J)"
          >
            <span>{Icons.terminal()}</span>
          </button>

          <button 
            class={`action-btn ${props.showRightPanel ? 'active' : ''}`} 
            onClick={props.onToggleRightPanel} 
            title="Toggle AI Assistant"
          >
            <span>{Icons.panelRight()}</span>
          </button>

          <button 
            class="action-btn" 
            onClick={props.onCustomizeLayout} 
            title="Customize Layout..."
          >
            <span>{Icons.layout()}</span>
          </button>

          <div class="menubar-divider-small" />

          {/* Tools */}
          <button 
            class="action-btn" 
            onClick={props.onQuickOpen} 
            title="Quick Open (Ctrl+P)"
          >
            <span>{Icons.search()}</span>
          </button>
          
          <button 
            class="action-btn" 
            onClick={props.onOpenAgentManager} 
            title="Open Agent Manager"
          >
            <span>{Icons.brain()}</span>
          </button>

          <button 
            class="action-btn" 
            onClick={props.onSettings} 
            title="Settings"
          >
            <span>{Icons.settings()}</span>
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
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
      </div>
    </div>
  );
};

export default IndustrialMenuBar;
