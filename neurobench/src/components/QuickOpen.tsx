import { Component, createSignal, createEffect, For, Show } from "solid-js";
import { Icons } from "./AppIcons";
import { FileSystem } from "../services/FileSystem";
import "./QuickOpen.css";

interface QuickOpenProps {
  onClose: () => void;
  isOpen: boolean;
  workspacePath?: string; // Add workspace path prop
  files?: { name: string; path: string; icon?: any }[];
  onSelectFile?: (path: string) => void;
  onRunCommand?: (commandId: string) => void;
}

interface QuickOpenItem {
  id: string;
  type: "command" | "file" | "separator" | "header";
  label?: string;
  description?: string;
  detail?: string;
  icon?: any;
  shortcut?: string;
  action?: () => void;
}

export const QuickOpen: Component<QuickOpenProps> = (props) => {
  const [input, setInput] = createSignal("");
  const [selectedIndex, setSelectedIndex] = createSignal(0);
  let inputRef: HTMLInputElement | undefined;

  // Mock data for commands - mimicking the reference image
  const commands: QuickOpenItem[] = [
    {
      id: "goto_file",
      type: "command",
      label: "Go to File",
      shortcut: "Ctrl + P",
      action: () => console.log("Go to File"),
    },
    {
      id: "show_commands",
      type: "command",
      label: "Show and Run Commands >",
      shortcut: "Ctrl + Shift + P",
      action: () => console.log("Show Commands"),
    },
    {
      id: "search_text",
      type: "command",
      label: "Search for Text %",
      action: () => console.log("Search Text"),
    },
    {
      id: "goto_symbol",
      type: "command",
      label: "Go to Symbol in Editor @",
      shortcut: "Ctrl + Shift + O",
      action: () => console.log("Go to Symbol"),
    },
    {
      id: "start_debug",
      type: "command",
      label: "Start Debugging",
      detail: "debug",
      action: () => console.log("Start Debugging"),
    },
    {
      id: "run_task",
      type: "command",
      label: "Run Task",
      detail: "task",
      action: () => console.log("Run Task"),
    },
  ];

  const [recentFiles, setRecentFiles] = createSignal<QuickOpenItem[]>([]);

  // Load files when opened
  // Load files when opened
  createEffect(() => {
    if (props.isOpen && props.workspacePath) {
       // Deep read for quick open might be expensive, for now just read root or use a recursive function if we had one that returned a flat list
       // For this MVP, let's just show root files + maybe a flattened "all files" if we implement a crawler
       
       // Just read root for now to avoid freezing UI
       (async () => {
           try {
            const rootEntries = await FileSystem.readDirectory(props.workspacePath!);
            const fileItems = rootEntries
                .filter(e => !e.isDirectory)
                .map(e => ({
                    id: e.path,
                    type: "file" as const,
                    label: e.name,
                    description: "root",
                    icon: <Icons.file />,
                    action: () => props.onSelectFile?.(e.path)
                }));
            setRecentFiles(fileItems);
           } catch (e) {
               console.error("Failed to load files for quick open", e);
           }
       })();
    }
  });

  const getFilteredItems = (): QuickOpenItem[] => {
    const searchText = input().toLowerCase();
    
    // If empty, show default list similar to screenshot
    if (!searchText) {
      return [
        ...commands,
        { id: "more_sep", type: "header", label: "More ?" },
        { id: "recent_header", type: "header", label: "files in workspace", description: "" }, // Header handling styling
        ...recentFiles()
      ];
    }

    // Filter logic
    const filteredCommands = commands.filter(c => 
      c.label?.toLowerCase().includes(searchText)
    );
    
    const filteredFiles = recentFiles().filter(f => 
      f.label?.toLowerCase().includes(searchText) || 
      f.description?.toLowerCase().includes(searchText)
    );

    return [...filteredCommands, ...filteredFiles];
  };

  const filteredItems = () => getFilteredItems();

  // Focus input on mount
  createEffect(() => {
    if (props.isOpen) {
      setTimeout(() => inputRef?.focus(), 50);
      setInput("");
      setSelectedIndex(0);
    }
  });

  const handleKeyDown = (e: KeyboardEvent) => {
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        setSelectedIndex(prev => Math.min(prev + 1, filteredItems().length - 1));
        break;
      case "ArrowUp":
        e.preventDefault();
        setSelectedIndex(prev => Math.max(prev - 1, 0));
        break;
      case "Enter":
        e.preventDefault();
        const item = filteredItems()[selectedIndex()];
        if (item && item.action) {
          item.action();
          props.onClose();
        }
        break;
      case "Escape":
        e.preventDefault();
        props.onClose();
        break;
    }
  };

  return (
    <Show when={props.isOpen}>
      <div class="quick-open-overlay" onClick={(e) => {
        if (e.target === e.currentTarget) props.onClose();
      }}>
        <div class="quick-open-container">
          <div class="quick-open-input-wrapper">
            <input
              ref={inputRef}
              type="text"
              class="quick-open-input"
              placeholder="Search files by name (append : to go to line or @ to go to symbol)"
              value={input()}
              onInput={(e) => {
                setInput(e.currentTarget.value);
                setSelectedIndex(0);
              }}
              onKeyDown={handleKeyDown}
            />
          </div>
          
          <div class="quick-open-list">
            <For each={filteredItems()}>
              {(item, index) => (
                <div 
                  class={`quick-open-item ${item.type} ${index() === selectedIndex() ? "selected" : ""}`}
                  onClick={() => {
                    if (item.action) {
                      item.action();
                      props.onClose();
                    }
                  }}
                  onMouseEnter={() => setSelectedIndex(index())}
                >
                  <Show when={item.type !== "header" && item.type !== "separator"}>
                    <div class="quick-open-item-content">
                      <Show when={item.icon}>
                        <span class="item-icon">{item.icon}</span>
                      </Show>
                      <div class="item-details">
                        <div class="item-row-main">
                          <span class="item-label">{item.label}</span>
                          <Show when={item.detail}>
                            <span class="item-detail-tag">{item.detail}</span>
                          </Show>
                        </div>
                        <Show when={item.description}>
                          <span class="item-description">{item.description}</span>
                        </Show>
                      </div>
                      <Show when={item.shortcut}>
                        <span class="item-shortcut">{item.shortcut}</span>
                      </Show>
                    </div>
                  </Show>
                  
                  <Show when={item.type === "header"}>
                    <div class="quick-open-header">
                      <span>{item.label}</span>
                    </div>
                  </Show>
                </div>
              )}
            </For>
            
            <Show when={filteredItems().length === 0}>
              <div class="quick-open-empty">No results found</div>
            </Show>
          </div>
        </div>
      </div>
    </Show>
  );
};
