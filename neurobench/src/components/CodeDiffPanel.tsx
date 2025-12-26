import { createSignal, For, Show, createMemo } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

// Types
interface FileDiff {
  path: string;
  old_content: string | null;
  new_content: string;
  additions: number;
  deletions: number;
}

interface Props {
  projectPath: string;
  onAccept?: (files: string[]) => void;
  onReject?: () => void;
}

export default function CodeDiffPanel(props: Props) {
  const [files, setFiles] = createSignal<FileDiff[]>([]);
  const [selectedFile, setSelectedFile] = createSignal<FileDiff | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [viewMode, setViewMode] = createSignal<"split" | "unified">("split");
  const [selectedPaths, setSelectedPaths] = createSignal<Set<string>>(new Set());

  const loadPreview = async () => {
    if (!props.projectPath) return;
    
    setLoading(true);
    try {
      const result = await invoke<FileDiff[]>("code_preview", {
        projectPath: props.projectPath
      });
      setFiles(result);
      if (result.length > 0) {
        setSelectedFile(result[0]);
        // Auto-select all by default
        setSelectedPaths(new Set<string>(result.map(f => f.path)));
      }
      setError(null);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const toggleFile = (path: string) => {
    const current = selectedPaths();
    const newSet = new Set(current);
    if (newSet.has(path)) {
      newSet.delete(path);
    } else {
      newSet.add(path);
    }
    setSelectedPaths(newSet);
  };

  const selectAll = () => {
    setSelectedPaths(new Set<string>(files().map(f => f.path)));
  };

  const selectNone = () => {
    setSelectedPaths(new Set<string>());
  };

  const handleAccept = async () => {
    const paths = Array.from(selectedPaths());
    if (paths.length === 0) return;
    
    try {
      await invoke("code_write", {
        projectPath: props.projectPath,
        paths
      });
      props.onAccept?.(paths);
    } catch (err) {
      setError(`Write failed: ${err}`);
    }
  };

  // Compute diff lines for display
  const computeDiffLines = createMemo(() => {
    const file = selectedFile();
    if (!file) return { old: [], new: [], unified: [] };
    
    const oldLines = (file.old_content || "").split("\n");
    const newLines = file.new_content.split("\n");
    
    // Simple diff: show added/removed lines
    const unified: { type: "add" | "remove" | "same"; content: string; lineNum: number }[] = [];
    
    const maxLines = Math.max(oldLines.length, newLines.length);
    for (let i = 0; i < maxLines; i++) {
      const oldLine = oldLines[i];
      const newLine = newLines[i];
      
      if (oldLine === newLine) {
        unified.push({ type: "same", content: oldLine || "", lineNum: i + 1 });
      } else if (oldLine === undefined) {
        unified.push({ type: "add", content: newLine, lineNum: i + 1 });
      } else if (newLine === undefined) {
        unified.push({ type: "remove", content: oldLine, lineNum: i + 1 });
      } else {
        unified.push({ type: "remove", content: oldLine, lineNum: i + 1 });
        unified.push({ type: "add", content: newLine, lineNum: i + 1 });
      }
    }
    
    return { 
      old: oldLines, 
      new: newLines, 
      unified 
    };
  });

  const stats = createMemo(() => {
    let adds = 0, dels = 0;
    for (const f of files()) {
      adds += f.additions;
      dels += f.deletions;
    }
    return { additions: adds, deletions: dels, files: files().length };
  });

  return (
    <div class="code-diff-panel">
      <div class="toolbar">
        <button class="refresh-btn" onClick={loadPreview} disabled={loading()}>
          {loading() ? "Loading..." : "⟳ Preview Code"}
        </button>
        
        <div class="view-toggle">
          <button 
            class={viewMode() === "split" ? "active" : ""} 
            onClick={() => setViewMode("split")}
          >Split</button>
          <button 
            class={viewMode() === "unified" ? "active" : ""} 
            onClick={() => setViewMode("unified")}
          >Unified</button>
        </div>
        
        <div class="stats">
          <span class="stat-files">{stats().files} files</span>
          <span class="stat-add">+{stats().additions}</span>
          <span class="stat-del">-{stats().deletions}</span>
        </div>
      </div>
      
      <Show when={error()}>
        <div class="error">{error()}</div>
      </Show>
      
      <div class="content">
        {/* File list */}
        <div class="file-list">
          <div class="file-list-header">
            <span>Changed Files</span>
            <div class="select-btns">
              <button onClick={selectAll}>All</button>
              <button onClick={selectNone}>None</button>
            </div>
          </div>
          <For each={files()}>
            {(file) => (
              <div 
                class={`file-item ${selectedFile()?.path === file.path ? 'selected' : ''}`}
                onClick={() => setSelectedFile(file)}
              >
                <input 
                  type="checkbox" 
                  checked={selectedPaths().has(file.path)}
                  onChange={() => toggleFile(file.path)}
                  onClick={(e) => e.stopPropagation()}
                />
                <span class="file-path">{file.path}</span>
                <span class="file-stats">
                  <span class="add">+{file.additions}</span>
                  <span class="del">-{file.deletions}</span>
                </span>
              </div>
            )}
          </For>
        </div>
        
        {/* Diff view */}
        <div class="diff-view">
          <Show when={selectedFile()} fallback={
            <div class="empty">Select a file to view diff</div>
          }>
            <div class="diff-header">
              <span class="file-name">{selectedFile()?.path}</span>
              <span class={`badge ${selectedFile()?.old_content ? 'modified' : 'new'}`}>
                {selectedFile()?.old_content ? 'Modified' : 'New File'}
              </span>
            </div>
            
            <Show when={viewMode() === "split"}>
              <div class="split-view">
                <div class="split-pane old">
                  <div class="pane-header">Old</div>
                  <pre class="code">
                    <For each={computeDiffLines().old}>
                      {(line, i) => (
                        <div class="line">
                          <span class="line-num">{i() + 1}</span>
                          <span class="line-content">{line}</span>
                        </div>
                      )}
                    </For>
                  </pre>
                </div>
                <div class="split-pane new">
                  <div class="pane-header">New</div>
                  <pre class="code">
                    <For each={computeDiffLines().new}>
                      {(line, i) => (
                        <div class="line">
                          <span class="line-num">{i() + 1}</span>
                          <span class="line-content">{line}</span>
                        </div>
                      )}
                    </For>
                  </pre>
                </div>
              </div>
            </Show>
            
            <Show when={viewMode() === "unified"}>
              <pre class="unified-view">
                <For each={computeDiffLines().unified}>
                  {(line) => (
                    <div class={`line ${line.type}`}>
                      <span class="line-num">{line.lineNum}</span>
                      <span class="line-marker">
                        {line.type === "add" ? "+" : line.type === "remove" ? "-" : " "}
                      </span>
                      <span class="line-content">{line.content}</span>
                    </div>
                  )}
                </For>
              </pre>
            </Show>
          </Show>
        </div>
      </div>
      
      {/* Action buttons */}
      <div class="actions">
        <button 
          class="accept-btn" 
          onClick={handleAccept}
          disabled={selectedPaths().size === 0}
        >
          ✓ Accept {selectedPaths().size > 0 ? `(${selectedPaths().size} files)` : ''}
        </button>
        <button class="reject-btn" onClick={props.onReject}>
          ✗ Reject All
        </button>
      </div>

      <style>{`
        .code-diff-panel {
          display: flex;
          flex-direction: column;
          height: 100%;
          background: var(--bg-base, #0d0d0d);
          color: var(--text-primary, #e8e8e8);
          font-family: var(--font-mono, 'IBM Plex Mono', monospace);
        }
        
        .toolbar {
          display: flex;
          align-items: center;
          gap: 16px;
          padding: 14px 16px;
          background: var(--bg-panel, #141414);
          border-bottom: 1px solid var(--border, #2a2a2a);
        }
        
        .refresh-btn {
          background: var(--accent, #e49b0f);
          color: #000;
          border: none;
          padding: 8px 18px;
          border-radius: 3px;
          font-family: inherit;
          font-size: 11px;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 0.5px;
          cursor: pointer;
          transition: all 0.15s;
        }
        
        .refresh-btn:hover {
          box-shadow: 0 0 12px var(--accent-glow, rgba(228, 155, 15, 0.4));
        }
        
        .view-toggle {
          display: flex;
          border: 1px solid var(--border-strong, #3a3a3a);
          border-radius: 3px;
          overflow: hidden;
        }
        
        .view-toggle button {
          background: var(--bg-surface, #1a1a1a);
          border: none;
          color: var(--text-dim, #606060);
          padding: 6px 14px;
          font-family: inherit;
          font-size: 10px;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 0.5px;
          cursor: pointer;
          transition: all 0.15s;
        }
        
        .view-toggle button:hover {
          color: var(--text-primary, #e8e8e8);
        }
        
        .view-toggle button.active {
          background: var(--accent, #e49b0f);
          color: #000;
        }
        
        .stats {
          margin-left: auto;
          display: flex;
          gap: 14px;
          font-size: 11px;
        }
        
        .stat-files { color: var(--text-secondary, #a0a0a0); }
        .stat-add { color: var(--status-success, #32cd32); }
        .stat-del { color: var(--status-error, #dc3545); }
        
        .content {
          display: flex;
          flex: 1;
          overflow: hidden;
        }
        
        .file-list {
          width: 260px;
          background: var(--bg-panel, #141414);
          border-right: 1px solid var(--border, #2a2a2a);
          overflow-y: auto;
        }
        
        .file-list-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 10px 14px;
          background: var(--bg-surface, #1a1a1a);
          border-bottom: 1px solid var(--border, #2a2a2a);
          font-size: 10px;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 1px;
          color: var(--text-dim, #606060);
        }
        
        .select-btns button {
          background: transparent;
          border: none;
          color: var(--text-dim, #606060);
          font-family: inherit;
          font-size: 9px;
          text-transform: uppercase;
          cursor: pointer;
          padding: 3px 8px;
        }
        
        .select-btns button:hover {
          color: var(--accent, #e49b0f);
        }
        
        .file-item {
          display: flex;
          align-items: center;
          gap: 10px;
          padding: 10px 14px;
          cursor: pointer;
          border-bottom: 1px solid var(--border, #2a2a2a);
          transition: all 0.15s;
        }
        
        .file-item:hover { 
          background: var(--bg-elevated, #222);
        }
        
        .file-item.selected { 
          background: var(--accent-subtle, rgba(228, 155, 15, 0.1));
          border-left: 3px solid var(--accent, #e49b0f);
        }
        
        .file-item input[type="checkbox"] {
          accent-color: var(--accent, #e49b0f);
        }
        
        .file-path {
          flex: 1;
          font-size: 11px;
          overflow: hidden;
          text-overflow: ellipsis;
        }
        
        .file-stats {
          font-size: 10px;
          display: flex;
          gap: 6px;
        }
        
        .file-stats .add { color: var(--status-success, #32cd32); }
        .file-stats .del { color: var(--status-error, #dc3545); }
        
        .diff-view {
          flex: 1;
          overflow: auto;
          display: flex;
          flex-direction: column;
          background: var(--bg-base, #0d0d0d);
        }
        
        .diff-header {
          display: flex;
          align-items: center;
          gap: 14px;
          padding: 12px 18px;
          background: var(--bg-panel, #141414);
          border-bottom: 1px solid var(--border, #2a2a2a);
        }
        
        .file-name {
          font-weight: 600;
          font-size: 12px;
        }
        
        .badge {
          font-size: 9px;
          font-weight: 600;
          padding: 3px 10px;
          border-radius: 2px;
          text-transform: uppercase;
          letter-spacing: 0.5px;
        }
        
        .badge.new { 
          background: rgba(50, 205, 50, 0.15);
          color: var(--status-success, #32cd32);
        }
        .badge.modified { 
          background: var(--accent-subtle, rgba(228, 155, 15, 0.15));
          color: var(--accent, #e49b0f);
        }
        
        .split-view {
          display: flex;
          flex: 1;
        }
        
        .split-pane {
          flex: 1;
          overflow: auto;
        }
        
        .split-pane.old { 
          background: rgba(220, 53, 69, 0.05);
          border-right: 1px solid var(--border, #2a2a2a);
        }
        .split-pane.new { 
          background: rgba(50, 205, 50, 0.05);
        }
        
        .pane-header {
          padding: 8px 14px;
          font-size: 10px;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 0.5px;
          color: var(--text-dim, #606060);
          background: var(--bg-panel, #141414);
          border-bottom: 1px solid var(--border, #2a2a2a);
        }
        
        .code {
          margin: 0;
          padding: 10px 0;
          font-size: 11px;
        }
        
        .line {
          display: flex;
          padding: 0 14px;
          line-height: 1.6;
        }
        
        .line-num {
          width: 44px;
          color: var(--text-dim, #606060);
          text-align: right;
          padding-right: 14px;
          user-select: none;
        }
        
        .line-marker {
          width: 18px;
          text-align: center;
          font-weight: bold;
        }
        
        .unified-view {
          flex: 1;
          margin: 0;
          padding: 10px 0;
          font-size: 11px;
          overflow: auto;
          background: var(--bg-base, #0d0d0d);
        }
        
        .unified-view .line.add { 
          background: rgba(50, 205, 50, 0.1);
        }
        .unified-view .line.remove { 
          background: rgba(220, 53, 69, 0.1);
        }
        .unified-view .line.add .line-marker { 
          color: var(--status-success, #32cd32);
        }
        .unified-view .line.remove .line-marker { 
          color: var(--status-error, #dc3545);
        }
        
        .empty {
          display: flex;
          align-items: center;
          justify-content: center;
          flex: 1;
          color: var(--text-dim, #606060);
          font-size: 11px;
          text-transform: uppercase;
          letter-spacing: 0.5px;
        }
        
        .actions {
          display: flex;
          gap: 14px;
          padding: 14px 16px;
          background: var(--bg-panel, #141414);
          border-top: 1px solid var(--border, #2a2a2a);
        }
        
        .accept-btn {
          background: var(--status-success, #32cd32);
          color: #000;
          border: none;
          padding: 12px 28px;
          border-radius: 3px;
          font-family: inherit;
          font-size: 11px;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 0.5px;
          cursor: pointer;
          transition: all 0.15s;
        }
        
        .accept-btn:hover {
          box-shadow: 0 0 12px rgba(50, 205, 50, 0.4);
        }
        
        .accept-btn:disabled {
          background: var(--border-strong, #3a3a3a);
          color: var(--text-dim, #606060);
          cursor: not-allowed;
          box-shadow: none;
        }
        
        .reject-btn {
          background: var(--status-error, #dc3545);
          color: white;
          border: none;
          padding: 12px 28px;
          border-radius: 3px;
          font-family: inherit;
          font-size: 11px;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 0.5px;
          cursor: pointer;
          transition: all 0.15s;
        }
        
        .reject-btn:hover {
          box-shadow: 0 0 12px rgba(220, 53, 69, 0.4);
        }
        
        .error {
          background: rgba(220, 53, 69, 0.1);
          color: var(--status-error, #dc3545);
          padding: 10px 16px;
          border-left: 3px solid var(--status-error, #dc3545);
          font-size: 11px;
        }
      `}</style>
    </div>
  );
}
