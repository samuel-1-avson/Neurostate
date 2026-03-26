// PanelNavigator Component - Organized panel navigation with categories
import { createSignal, For, Show } from "solid-js";

interface PanelCategory {
  name: string;
  icon: string;
  panels: PanelItem[];
}

interface PanelItem {
  id: string;
  name: string;
  icon: string;
  description?: string;
}

interface PanelNavigatorProps {
  activePanel: string;
  onPanelChange: (panelId: string) => void;
  collapsed?: boolean;
  onToggleCollapse?: () => void;
}

const panelCategories: PanelCategory[] = [
  {
    name: "Design",
    icon: "🎨",
    panels: [
      { id: "nodes", name: "Node Palette", icon: "📦", description: "FSM nodes and components" },
      { id: "pins", name: "Pin Diagram", icon: "📍", description: "MCU pinout configuration" },
      { id: "chat", name: "AI Assistant", icon: "🤖", description: "AI-powered help" },
    ]
  },
  {
    name: "Hardware",
    icon: "🔧",
    panels: [
      { id: "hardware", name: "MCU Config", icon: "🎯", description: "Target MCU configuration" },
      { id: "drivers", name: "Drivers", icon: "🔌", description: "GPIO, UART, SPI, I2C" },
      { id: "peripherals", name: "Peripherals", icon: "⚙️", description: "Advanced peripherals" },
      { id: "timers", name: "Timers", icon: "⏱️", description: "Timer configuration" },
      { id: "analog", name: "Analog", icon: "📊", description: "ADC/DAC configuration" },
    ]
  },
  {
    name: "Communication",
    icon: "📡",
    panels: [
      { id: "wireless", name: "Wireless", icon: "📶", description: "BLE, WiFi, LoRa" },
      { id: "serial", name: "Serial", icon: "🔗", description: "Serial monitor" },
    ]
  },
  {
    name: "RTOS & DSP",
    icon: "⚡",
    panels: [
      { id: "rtos", name: "RTOS", icon: "🔄", description: "FreeRTOS/Zephyr tasks" },
      { id: "dsp", name: "DSP", icon: "📈", description: "Filters, FFT, PID" },
      { id: "scheduler", name: "Scheduler", icon: "📅", description: "Task scheduling" },
    ]
  },
  {
    name: "Development",
    icon: "💻",
    panels: [
      { id: "code", name: "Code", icon: "📝", description: "Generated code view" },
      { id: "build", name: "Build", icon: "🔨", description: "Build & flash" },
      { id: "validation", name: "Validation", icon: "✅", description: "Syntax & logic check" },
      { id: "git", name: "Git", icon: "🌿", description: "Version control" },
    ]
  },
  {
    name: "Debug & Analysis",
    icon: "🔍",
    panels: [
      { id: "debug", name: "Debug", icon: "🐛", description: "Debugger controls" },
      { id: "simulator", name: "Simulator", icon: "🎮", description: "FSM simulation" },
      { id: "memory", name: "Memory", icon: "💾", description: "Memory inspector" },
      { id: "profiler", name: "Profiler", icon: "📉", description: "Performance profiling" },
      { id: "performance", name: "Performance", icon: "⚡", description: "System performance" },
    ]
  },
  {
    name: "Security & Power",
    icon: "🛡️",
    panels: [
      { id: "security", name: "Security", icon: "🔐", description: "Bootloader, encryption" },
      { id: "power", name: "Power", icon: "🔋", description: "Power management" },
    ]
  },
  {
    name: "Other",
    icon: "📁",
    panels: [
      { id: "agents", name: "AI Agents", icon: "🧠", description: "Multi-agent system" },
      { id: "workflow", name: "Workflow", icon: "📊", description: "Visual workflows" },
      { id: "history", name: "History", icon: "📜", description: "Change history" },
    ]
  },
];

export function PanelNavigator(props: PanelNavigatorProps) {
  const [expandedCategories, setExpandedCategories] = createSignal<Set<string>>(
    new Set(["Design", "Hardware", "Development"])
  );

  const toggleCategory = (category: string) => {
    const current = new Set(expandedCategories());
    if (current.has(category)) {
      current.delete(category);
    } else {
      current.add(category);
    }
    setExpandedCategories(current);
  };

  return (
    <div class={`panel-navigator ${props.collapsed ? "collapsed" : ""}`}>
      <div class="panel-nav-header">
        <span class="panel-nav-title">Panels</span>
        <button 
          class="panel-nav-collapse-btn" 
          onClick={props.onToggleCollapse}
          title={props.collapsed ? "Expand" : "Collapse"}
        >
          {props.collapsed ? "▶" : "◀"}
        </button>
      </div>

      <Show when={!props.collapsed}>
        <div class="panel-nav-content">
          <For each={panelCategories}>
            {(category) => (
              <div class="panel-nav-category">
                <button 
                  class="panel-nav-category-header"
                  onClick={() => toggleCategory(category.name)}
                >
                  <span class="category-icon">{category.icon}</span>
                  <span class="category-name">{category.name}</span>
                  <span class="category-toggle">
                    {expandedCategories().has(category.name) ? "▼" : "▶"}
                  </span>
                </button>

                <Show when={expandedCategories().has(category.name)}>
                  <div class="panel-nav-items">
                    <For each={category.panels}>
                      {(panel) => (
                        <button
                          class={`panel-nav-item ${props.activePanel === panel.id ? "active" : ""}`}
                          onClick={() => props.onPanelChange(panel.id)}
                          title={panel.description}
                        >
                          <span class="panel-icon">{panel.icon}</span>
                          <span class="panel-name">{panel.name}</span>
                        </button>
                      )}
                    </For>
                  </div>
                </Show>
              </div>
            )}
          </For>
        </div>
      </Show>

      {/* Quick access when collapsed */}
      <Show when={props.collapsed}>
        <div class="panel-nav-collapsed-items">
          <For each={panelCategories.flatMap(c => c.panels).slice(0, 10)}>
            {(panel) => (
              <button
                class={`panel-nav-collapsed-item ${props.activePanel === panel.id ? "active" : ""}`}
                onClick={() => props.onPanelChange(panel.id)}
                title={panel.name}
              >
                {panel.icon}
              </button>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
}

export default PanelNavigator;
