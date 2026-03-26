import { Component, createSignal, For, Show, onMount } from "solid-js";
import { Icons } from "./AppIcons";
import { FileSystem, FileSystemEntry } from "../services/FileSystem";
import "./FileExplorer.css";

interface FileExplorerProps {
  workspacePath: string;
  onFileSelect: (path: string, name: string) => void;
  className?: string; // Allow passing class for width manipulation
}

const FileTreeNode: Component<{ 
  entry: FileSystemEntry; 
  level: number;
  onSelect: (path: string, name: string) => void;
}> = (props) => {
  const [expanded, setExpanded] = createSignal(false);
  const [children, setChildren] = createSignal<FileSystemEntry[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [loaded, setLoaded] = createSignal(false);

  const handleToggle = async (e: MouseEvent) => {
    e.stopPropagation();
    if (!props.entry.isDirectory) {
      props.onSelect(props.entry.path, props.entry.name);
      return;
    }

    setExpanded(!expanded());
    
    if (expanded() && !loaded()) {
      setLoading(true);
      const entries = await FileSystem.readDirectory(props.entry.path);
      setChildren(entries);
      setLoaded(true);
      setLoading(false);
    }
  };

  const getIcon = () => {
    if (props.entry.isDirectory) {
      return expanded() ? <Icons.folderOpen /> : <Icons.folder />;
    }
    
    // Simple file extension check for icon
    const name = props.entry.name.toLowerCase();
    if (name.endsWith('.ts')) return <span class="file-icon ts">TS</span>;
    if (name.endsWith('.tsx')) return <span class="file-icon tsx">TSX</span>;
    if (name.endsWith('.js')) return <span class="file-icon js">JS</span>;
    if (name.endsWith('.jsx')) return <span class="file-icon js">JSX</span>;
    if (name.endsWith('.css')) return <span class="file-icon css">#</span>;
    if (name.endsWith('.json')) return <span class="file-icon json">{"{}"}</span>;
    if (name.endsWith('.md')) return <span class="file-icon md">MD</span>;
    if (name.endsWith('.rs')) return <span class="file-icon rs">RS</span>;
    return <Icons.file />;
  };

  return (
    <div class="file-tree-node">
      <div 
        class="file-node-content" 
        style={{ "padding-left": `${props.level * 12 + 10}px` }}
        onClick={handleToggle}
        title={props.entry.path}
      >
        <span class="file-arrow" style={{ opacity: props.entry.isDirectory ? 1 : 0, transform: expanded() ? "rotate(90deg)" : "none" }}>
          <Icons.chevronRight />
        </span>
        <span class="file-icon-wrapper">
          {getIcon()}
        </span>
        <span class="file-name">{props.entry.name}</span>
      </div>
      
      <Show when={expanded()}>
        <div class="file-node-children">
          <For each={children()}>
            {(child) => (
              <FileTreeNode 
                entry={child} 
                level={props.level + 1} 
                onSelect={props.onSelect}
              />
            )}
          </For>
          <Show when={children().length === 0 && !loading() && loaded()}>
             <div class="empty-folder">Empty</div>
          </Show>
        </div>
      </Show>
    </div>
  );
};

export const FileExplorer: Component<FileExplorerProps> = (props) => {
  const [rootFiles, setRootFiles] = createSignal<FileSystemEntry[]>([]);
  const [loading, setLoading] = createSignal(false);
  
  const loadRoot = async () => {
    if (!props.workspacePath) return; // Add check back
    setLoading(true);
    const entries = await FileSystem.readDirectory(props.workspacePath);
    setRootFiles(entries);
    setLoading(false);
  };

  // Reload when workspace path changes
  createSignal(() => {
     loadRoot(); 
  }); 
  
  // Use effect to trigger load on mount or prop change
  onMount(loadRoot);

  return (
    <div class={`file-explorer ${props.className || ''}`}>
      <div class="explorer-header">
        <span class="explorer-title">EXPLORER</span>
      </div>
      
      <div class="explorer-content">
        <Show when={!props.workspacePath}>
            <div class="explorer-empty-state">
                No Folder Opened
            </div>
        </Show>

        <Show when={props.workspacePath}>
            <div class="explorer-root-header">
                <span class="root-arrow"><Icons.chevronDown /></span>
                <span class="root-name">{props.workspacePath.split(/[\/\\]/).pop()}</span>
            </div>
            
            <Show when={loading()}>
                <div class="explorer-loading" style={{padding: "10px", "font-style": "italic"}}>Loading...</div>
            </Show>

            <For each={rootFiles()}>
            {(entry) => (
                <FileTreeNode 
                entry={entry} 
                level={0} 
                onSelect={props.onFileSelect}
                />
            )}
            </For>
        </Show>
      </div>
    </div>
  );
};
