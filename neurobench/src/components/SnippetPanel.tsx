import { createSignal, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface Snippet {
  id: string;
  name: string;
  description: string;
  category: string;
  language: string;
  code: string;
  tags: string[];
}

interface SnippetPanelProps {
  onInsert?: (code: string) => void;
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function SnippetPanel(props: SnippetPanelProps) {
  const [snippets, setSnippets] = createSignal<Snippet[]>([]);
  const [searchQuery, setSearchQuery] = createSignal("");
  const [selectedSnippet, setSelectedSnippet] = createSignal<Snippet | null>(null);
  const [isLoading, setIsLoading] = createSignal(false);

  const loadSnippets = async () => {
    setIsLoading(true);
    try {
      const result = await invoke("snippets_get_all") as Snippet[];
      setSnippets(result);
      props.onLog?.("Snippets", `Loaded ${result.length} snippets`, "info");
    } catch (e) {
      props.onLog?.("Snippets", `Failed to load: ${e}`, "error");
    }
    setIsLoading(false);
  };

  const searchSnippets = async () => {
    if (!searchQuery().trim()) {
      loadSnippets();
      return;
    }
    
    setIsLoading(true);
    try {
      const result = await invoke("snippets_search", { query: searchQuery() }) as Snippet[];
      setSnippets(result);
    } catch (e) {
      props.onLog?.("Snippets", `Search failed: ${e}`, "error");
    }
    setIsLoading(false);
  };

  const copyCode = (code: string) => {
    navigator.clipboard.writeText(code);
    props.onLog?.("Snippets", "Code copied to clipboard", "success");
  };

  // Load on mount
  if (snippets().length === 0) {
    loadSnippets();
  }

  return (
    <div class="snippet-panel">
      <div class="panel-header">
        <h3>📝 Code Snippets</h3>
      </div>

      {/* Search */}
      <div class="search-box">
        <input 
          type="text" 
          placeholder="Search snippets..."
          value={searchQuery()}
          onInput={(e) => setSearchQuery(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && searchSnippets()}
        />
        <button onClick={searchSnippets}>🔍</button>
      </div>

      {/* Snippet list */}
      <div class="snippet-list">
        <Show when={isLoading()}>
          <div class="loading">Loading...</div>
        </Show>
        
        <For each={snippets()}>
          {(snippet) => (
            <div 
              class={`snippet-card ${selectedSnippet()?.id === snippet.id ? "selected" : ""}`}
              onClick={() => setSelectedSnippet(snippet)}
            >
              <div class="card-header">
                <span class="snippet-name">{snippet.name}</span>
                <span class="lang-badge">{snippet.language}</span>
              </div>
              <p class="snippet-desc">{snippet.description}</p>
              <div class="tags">
                <For each={snippet.tags.slice(0, 3)}>
                  {(tag) => <span class="tag">{tag}</span>}
                </For>
              </div>
            </div>
          )}
        </For>
      </div>

      {/* Selected snippet preview */}
      <Show when={selectedSnippet()}>
        <div class="snippet-preview">
          <div class="preview-header">
            <span>{selectedSnippet()!.name}</span>
            <div class="preview-actions">
              <button onClick={() => copyCode(selectedSnippet()!.code)}>📋 Copy</button>
              <button onClick={() => props.onInsert?.(selectedSnippet()!.code)}>➕ Insert</button>
            </div>
          </div>
          <pre class="code-block"><code>{selectedSnippet()!.code}</code></pre>
        </div>
      </Show>

      <style>{`
        .snippet-panel {
          background: var(--bg-secondary, #1a1a2e);
          border: 1px solid var(--border, #333);
          border-radius: 8px;
          padding: 12px;
        }

        .panel-header { margin-bottom: 12px; }
        .panel-header h3 { margin: 0; font-size: 14px; }

        .search-box {
          display: flex;
          gap: 6px;
          margin-bottom: 12px;
        }

        .search-box input {
          flex: 1;
          background: rgba(255,255,255,0.05);
          border: 1px solid #333;
          color: #fff;
          padding: 8px 12px;
          border-radius: 6px;
          font-size: 12px;
        }

        .search-box button {
          background: rgba(59, 130, 246, 0.2);
          border: 1px solid rgba(59, 130, 246, 0.3);
          color: #60a5fa;
          padding: 8px 12px;
          border-radius: 6px;
          cursor: pointer;
        }

        .snippet-list {
          max-height: 200px;
          overflow-y: auto;
        }

        .snippet-card {
          padding: 10px;
          background: rgba(255,255,255,0.03);
          border: 1px solid transparent;
          border-radius: 6px;
          margin-bottom: 6px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .snippet-card:hover { background: rgba(59, 130, 246, 0.1); }
        .snippet-card.selected {
          border-color: #3b82f6;
          background: rgba(59, 130, 246, 0.15);
        }

        .card-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .snippet-name { font-weight: 600; font-size: 12px; }
        .lang-badge {
          background: rgba(139, 92, 246, 0.2);
          color: #a78bfa;
          padding: 2px 6px;
          border-radius: 4px;
          font-size: 10px;
        }

        .snippet-desc { color: #888; font-size: 11px; margin: 4px 0; }

        .tags { display: flex; gap: 4px; }
        .tag {
          background: rgba(255,255,255,0.05);
          color: #666;
          padding: 2px 6px;
          border-radius: 3px;
          font-size: 9px;
        }

        .snippet-preview {
          margin-top: 12px;
          background: rgba(0,0,0,0.3);
          border-radius: 6px;
          overflow: hidden;
        }

        .preview-header {
          display: flex;
          justify-content: space-between;
          padding: 8px 12px;
          background: rgba(0,0,0,0.2);
          font-size: 12px;
        }

        .preview-actions { display: flex; gap: 6px; }
        .preview-actions button {
          background: rgba(255,255,255,0.1);
          border: none;
          color: #ccc;
          padding: 4px 8px;
          border-radius: 4px;
          cursor: pointer;
          font-size: 10px;
        }

        .code-block {
          padding: 12px;
          margin: 0;
          font-size: 11px;
          font-family: 'Fira Code', monospace;
          color: #a6e3a1;
          overflow-x: auto;
          max-height: 150px;
        }

        .loading { text-align: center; color: #888; padding: 20px; }
      `}</style>
    </div>
  );
}
