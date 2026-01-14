import { Component, For } from "solid-js";
import { Icons } from "./AppIcons";
import "./EditorTabBar.css";

export interface EditorTab {
  id: string;
  name: string;
  modified?: boolean;
  icon?: any; // Optional custom icon, otherwise derived from extension
}

interface EditorTabBarProps {
  tabs: EditorTab[];
  activeTabId: string;
  onTabClick: (id: string) => void;
  onTabClose: (id: string) => void;
  onAddTab?: () => void;
}

export const EditorTabBar: Component<EditorTabBarProps> = (props) => {
  
  const getIconForFile = (filename: string) => {
    if (filename.endsWith(".ts")) return <span class="file-icon ts">TS</span>;
    if (filename.endsWith(".tsx")) return <span class="file-icon tsx">TSX</span>;
    if (filename.endsWith(".js")) return <span class="file-icon js">JS</span>;
    if (filename.endsWith(".jsx")) return <span class="file-icon js">JSX</span>;
    if (filename.endsWith(".css")) return <span class="file-icon css">#</span>;
    if (filename.endsWith(".json")) return <span class="file-icon json">{"{}"}</span>;
    if (filename.endsWith(".md")) return <span class="file-icon md">MD</span>;
    return <Icons.file />;
  };

  return (
    <div class="editor-tab-bar">
      <div class="tabs-container">
        <For each={props.tabs}>
          {(tab) => (
            <div 
              class="editor-tab" 
              classList={{ 
                active: props.activeTabId === tab.id,
                modified: tab.modified
              }}
              onClick={() => props.onTabClick(tab.id)}
              title={tab.name}
            >
              <span class="tab-icon">
                {tab.icon || getIconForFile(tab.name)}
              </span>
              <span class="tab-label">{tab.name}</span>
              <span class="tab-status">
                {tab.modified && <span class="modified-dot">●</span>}
              </span>
              <button 
                class="tab-close" 
                onClick={(e) => {
                  e.stopPropagation();
                  props.onTabClose(tab.id);
                }}
              >
                <Icons.x />
              </button>
            </div>
          )}
        </For>
      </div>
      <div class="tab-bar-tools">
        {props.onAddTab && (
          <button class="tool-btn" onClick={props.onAddTab} title="New Tab">
            <Icons.plus />
          </button>
        )}
      </div>
    </div>
  );
};
