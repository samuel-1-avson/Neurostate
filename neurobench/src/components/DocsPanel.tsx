import { createSignal, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface FunctionDoc {
  name: string;
  brief: string;
  description: string;
  params: { name: string; param_type: string; description: string }[];
  returns: string | null;
}

interface DocsPanelProps {
  code?: string;
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function DocsPanel(props: DocsPanelProps) {
  const [filename, setFilename] = createSignal("main.c");
  const [author, setAuthor] = createSignal("NeuroBench");
  const [brief, setBrief] = createSignal("Auto-generated embedded firmware");
  const [generatedDocs, setGeneratedDocs] = createSignal("");
  const [functions, setFunctions] = createSignal<FunctionDoc[]>([]);
  const [isLoading, setIsLoading] = createSignal(false);

  const generateDocs = async () => {
    const code = props.code || "";
    if (!code.trim()) {
      props.onLog?.("Docs", "No code to document", "warning");
      return;
    }

    setIsLoading(true);
    try {
      const result = await invoke("docs_generate", {
        code,
        filename: filename(),
        author: author(),
        brief: brief(),
      }) as { documentation: string };
      
      setGeneratedDocs(result.documentation);
      
      const funcs = await invoke("docs_extract_functions", { code }) as FunctionDoc[];
      setFunctions(funcs);
      
      props.onLog?.("Docs", `Generated docs for ${funcs.length} functions`, "success");
    } catch (e) {
      props.onLog?.("Docs", `Generation failed: ${e}`, "error");
    }
    setIsLoading(false);
  };

  const copyDocs = () => {
    navigator.clipboard.writeText(generatedDocs());
    props.onLog?.("Docs", "Documentation copied to clipboard", "info");
  };

  return (
    <div class="docs-panel">
      <div class="panel-header">
        <h3>📚 Documentation Generator</h3>
      </div>

      <div class="config-row">
        <label>Filename</label>
        <input 
          type="text" 
          value={filename()}
          onInput={(e) => setFilename(e.target.value)}
        />
      </div>

      <div class="config-row">
        <label>Author</label>
        <input 
          type="text" 
          value={author()}
          onInput={(e) => setAuthor(e.target.value)}
        />
      </div>

      <div class="config-row">
        <label>Brief Description</label>
        <input 
          type="text" 
          value={brief()}
          onInput={(e) => setBrief(e.target.value)}
        />
      </div>

      <button class="generate-btn" onClick={generateDocs} disabled={isLoading()}>
        {isLoading() ? "Generating..." : "Generate Doxygen Docs"}
      </button>

      {/* Functions found */}
      <Show when={functions().length > 0}>
        <div class="functions-list">
          <h4>Functions Found ({functions().length})</h4>
          <For each={functions()}>
            {(func) => (
              <div class="func-item">
                <span class="func-name">{func.name}</span>
                <span class="func-params">({func.params.length} params)</span>
              </div>
            )}
          </For>
        </div>
      </Show>

      {/* Generated documentation */}
      <Show when={generatedDocs()}>
        <div class="docs-output">
          <div class="output-header">
            <span>Generated Documentation</span>
            <button onClick={copyDocs}>📋 Copy</button>
          </div>
          <pre class="docs-code">{generatedDocs().substring(0, 800)}...</pre>
        </div>
      </Show>

      <style>{`
        .docs-panel {
          background: var(--bg-secondary, #1a1a2e);
          border: 1px solid var(--border, #333);
          border-radius: 8px;
          padding: 12px;
        }

        .panel-header { margin-bottom: 12px; }
        .panel-header h3 { margin: 0; font-size: 14px; }

        .config-row { margin-bottom: 10px; }
        .config-row label {
          display: block;
          color: #888;
          font-size: 11px;
          margin-bottom: 4px;
        }

        .config-row input {
          width: 100%;
          background: rgba(255,255,255,0.05);
          border: 1px solid #333;
          color: #fff;
          padding: 8px 12px;
          border-radius: 6px;
          font-size: 12px;
        }

        .generate-btn {
          width: 100%;
          background: linear-gradient(135deg, #14b8a6, #0d9488);
          color: white;
          border: none;
          padding: 10px;
          border-radius: 6px;
          cursor: pointer;
          font-weight: 600;
          margin-bottom: 12px;
        }

        .functions-list {
          margin-bottom: 12px;
        }

        .functions-list h4 {
          margin: 0 0 8px 0;
          font-size: 12px;
          color: #888;
        }

        .func-item {
          display: flex;
          justify-content: space-between;
          padding: 6px;
          background: rgba(255,255,255,0.03);
          border-radius: 4px;
          margin-bottom: 4px;
          font-size: 12px;
        }

        .func-name { color: #60a5fa; }
        .func-params { color: #888; }

        .docs-output {
          background: rgba(0,0,0,0.3);
          border-radius: 6px;
          overflow: hidden;
        }

        .output-header {
          display: flex;
          justify-content: space-between;
          padding: 8px 12px;
          background: rgba(0,0,0,0.2);
          font-size: 12px;
        }

        .output-header button {
          background: transparent;
          border: none;
          color: #ccc;
          cursor: pointer;
          font-size: 11px;
        }

        .docs-code {
          padding: 12px;
          margin: 0;
          font-size: 10px;
          font-family: 'Fira Code', monospace;
          color: #a6e3a1;
          max-height: 200px;
          overflow-y: auto;
        }
      `}</style>
    </div>
  );
}
