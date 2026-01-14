/**
 * ActivityBar - VS Code-style vertical icon rail
 * Industrial-grade sidebar navigation with SVG icons
 */

import { Component, For, Show, JSX } from "solid-js";
import { Icons } from "./AppIcons";
import "./ActivityBar.css";

interface ActivityItem {
  id: string;
  icon: () => JSX.Element;
  label: string;
  badge?: number;
}

interface ActivityBarProps {
  items: ActivityItem[];
  activeId: string | null;
  onItemClick: (id: string) => void;
  bottomItems?: ActivityItem[];
}

// Default navigation items with SVG icons
export const DEFAULT_ACTIVITY_ITEMS: ActivityItem[] = [
  { id: "explorer", icon: Icons.explorer, label: "Explorer" },
  { id: "nodes", icon: Icons.layers, label: "Node Palette" },
  { id: "simulation", icon: Icons.flash, label: "Simulation" },
  { id: "peripherals", icon: Icons.plug, label: "Peripherals" },
  { id: "debug", icon: Icons.debug, label: "Debug" },
  { id: "build", icon: Icons.build, label: "Build" },
  { id: "git", icon: Icons.gitBranch, label: "Source Control" },
  { id: "ai", icon: Icons.brain, label: "AI Assistant" },
];

export const DEFAULT_BOTTOM_ITEMS: ActivityItem[] = [
  { id: "performance", icon: Icons.chart, label: "Performance" },
  { id: "settings", icon: Icons.gear, label: "Settings" },
];

export const ActivityBar: Component<ActivityBarProps> = (props) => {
  return (
    <aside class="activity-bar">
      {/* Top section - main navigation */}
      <div class="activity-section activity-top">
        <For each={props.items}>
          {(item) => (
            <button
              class="activity-item"
              classList={{ active: props.activeId === item.id }}
              onClick={() => props.onItemClick(item.id)}
              title={item.label}
              data-testid={`activity-${item.id}`}
            >
              <span class="activity-icon">{item.icon()}</span>
              <Show when={item.badge && item.badge > 0}>
                <span class="activity-badge">{item.badge}</span>
              </Show>
              <div class="activity-indicator" />
              
              {/* Tooltip */}
              <div class="activity-tooltip">
                <span>{item.label}</span>
              </div>
            </button>
          )}
        </For>
      </div>
      
      {/* Bottom section - settings, etc */}
      <Show when={props.bottomItems && props.bottomItems.length > 0}>
        <div class="activity-section activity-bottom">
          <For each={props.bottomItems}>
            {(item) => (
              <button
                class="activity-item"
                classList={{ active: props.activeId === item.id }}
                onClick={() => props.onItemClick(item.id)}
                title={item.label}
                data-testid={`activity-${item.id}`}
              >
                <span class="activity-icon">{item.icon()}</span>
                <Show when={item.badge && item.badge > 0}>
                  <span class="activity-badge">{item.badge}</span>
                </Show>
                <div class="activity-indicator" />
                
                {/* Tooltip */}
                <div class="activity-tooltip">
                  <span>{item.label}</span>
                </div>
              </button>
            )}
          </For>
        </div>
      </Show>
    </aside>
  );
};

export default ActivityBar;
