// PanelNavigator Component - Organized panel navigation with categories
import { createSignal, For, Show, JSX } from "solid-js";
import { Icons } from "./AppIcons";

interface PanelCategory {
  name: string;
  icon: JSX.Element;
  panels: PanelItem[];
}

interface PanelItem {
  id: string;
  name: string;
  icon: JSX.Element;
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
    icon: Icons.palette(),
    panels: [
      { id: "nodes", name: "Node Palette", icon: Icons.package(), description: "FSM nodes and components" },
      { id: "pins", name: "Pin Diagram", icon: Icons.pin(), description: "MCU pinout configuration" },
      { id: "chat", name: "AI Assistant", icon: Icons.robot(), description: "AI-powered help" },
    ]
  },
  {
    name: "Hardware",
    icon: Icons.tool(),
    panels: [
      { id: "hardware", name: "MCU Config", icon: Icons.crosshair(), description: "Target MCU configuration" },
      { id: "drivers", name: "Drivers", icon: Icons.plug(), description: "GPIO, UART, SPI, I2C" },
      { id: "peripherals", name: "Peripherals", icon: Icons.gear(), description: "Advanced peripherals" },
      { id: "timers", name: "Timers", icon: Icons.timer(), description: "Timer configuration" },
      { id: "analog", name: "Analog", icon: Icons.chart(), description: "ADC/DAC configuration" },
    ]
  },
  {
    name: "Communication",
    icon: Icons.antenna(),
    panels: [
      { id: "wireless", name: "Wireless", icon: Icons.wifi(), description: "BLE, WiFi, LoRa" },
      { id: "serial", name: "Serial", icon: Icons.link(), description: "Serial monitor" },
    ]
  },
  {
    name: "RTOS & DSP",
    icon: Icons.flash(),
    panels: [
      { id: "rtos", name: "RTOS", icon: Icons.refresh(), description: "FreeRTOS/Zephyr tasks" },
      { id: "dsp", name: "DSP", icon: Icons.trendUp(), description: "Filters, FFT, PID" },
      { id: "scheduler", name: "Scheduler", icon: Icons.calendar(), description: "Task scheduling" },
    ]
  },
  {
    name: "Development",
    icon: Icons.monitor(),
    panels: [
      { id: "code", name: "Code", icon: Icons.docs(), description: "Generated code view" },
      { id: "build", name: "Build", icon: Icons.build(), description: "Build & flash" },
      { id: "validation", name: "Validation", icon: Icons.check(), description: "Syntax & logic check" },
      { id: "git", name: "Git", icon: Icons.gitBranch(), description: "Version control" },
    ]
  },
  {
    name: "Debug & Analysis",
    icon: Icons.search(),
    panels: [
      { id: "debug", name: "Debug", icon: Icons.debug(), description: "Debugger controls" },
      { id: "simulator", name: "Simulator", icon: Icons.simulator(), description: "FSM simulation" },
      { id: "memory", name: "Memory", icon: Icons.memory(), description: "Memory inspector" },
      { id: "profiler", name: "Profiler", icon: Icons.trendDown(), description: "Performance profiling" },
      { id: "performance", name: "Performance", icon: Icons.flash(), description: "System performance" },
    ]
  },
  {
    name: "Security & Power",
    icon: Icons.shield(),
    panels: [
      { id: "security", name: "Security", icon: Icons.lock(), description: "Bootloader, encryption" },
      { id: "power", name: "Power", icon: Icons.battery(), description: "Power management" },
    ]
  },
  {
    name: "Other",
    icon: Icons.folder(),
    panels: [
      { id: "agents", name: "AI Agents", icon: Icons.brain(), description: "Multi-agent system" },
      { id: "workflow", name: "Workflow", icon: Icons.workflow(), description: "Visual workflows" },
      { id: "history", name: "History", icon: Icons.history(), description: "Change history" },
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
          {props.collapsed ? Icons.chevronRight() : Icons.chevronLeft ? Icons.chevronLeft() : <span style={{transform: "rotate(180deg)", display: "inline-block"}}>{Icons.chevronRight()}</span>}
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
                    {expandedCategories().has(category.name) ? Icons.chevronDown() : Icons.chevronRight()}
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
