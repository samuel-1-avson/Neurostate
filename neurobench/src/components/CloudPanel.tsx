import { createSignal, Show, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface ExportedFile {
  path: string;
  content: string;
  language: string;
  generated: boolean;
}

interface ProjectExport {
  version: string;
  name: string;
  description: string;
  created_at: string;
  mcu_target: string;
  files: ExportedFile[];
  config: any;
  metadata: Record<string, string>;
}

interface CloudPanelProps {
  projectName?: string;
  generatedFiles?: { path: string; content: string }[];
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function CloudPanel(props: CloudPanelProps) {
  const [activeTab, setActiveTab] = createSignal<"export" | "import" | "share">("export");
  const [projectName, setProjectName] = createSignal(props.projectName || "My NeuroBench Project");
  const [description, setDescription] = createSignal("Generated embedded code");
  const [mcuTarget, setMcuTarget] = createSignal("STM32F407");
  const [exportJson, setExportJson] = createSignal("");
  const [importJson, setImportJson] = createSignal("");
  const [importedProject, setImportedProject] = createSignal<ProjectExport | null>(null);
  const [shareId, setShareId] = createSignal("");
  const [isLoading, setIsLoading] = createSignal(false);

  const addLog = (source: string, message: string, type: "info" | "success" | "warning" | "error" = "info") => {
    props.onLog?.(source, message, type);
  };

  const exportProject = async () => {
    setIsLoading(true);
    try {
      // Convert generated files to export format
      const files = (props.generatedFiles || []).map(f => ({
        path: f.path,
        content: f.content,
        language: f.path.endsWith(".c") ? "c" : f.path.endsWith(".h") ? "c" : "text",
        generated: true,
      }));

      const result = await invoke("cloud_export_project", {
        name: projectName(),
        description: description(),
        mcuTarget: mcuTarget(),
        files,
      }) as { success: boolean; json: string };

      setExportJson(result.json);
      addLog("Cloud", "Project exported successfully", "success");
    } catch (e) {
      addLog("Cloud", `Export failed: ${e}`, "error");
    }
    setIsLoading(false);
  };

  const importProject = async () => {
    if (!importJson().trim()) {
      addLog("Cloud", "Please paste project JSON", "warning");
      return;
    }

    setIsLoading(true);
    try {
      const result = await invoke("cloud_import_project", {
        json: importJson(),
      }) as ProjectExport;

      setImportedProject(result);
      addLog("Cloud", `Imported: ${result.name} (${result.files.length} files)`, "success");
    } catch (e) {
      addLog("Cloud", `Import failed: ${e}`, "error");
    }
    setIsLoading(false);
  };

  const generateShareLink = async () => {
    try {
      const result = await invoke("cloud_generate_share_id") as { shareId: string };
      setShareId(result.shareId);
      addLog("Cloud", `Share ID generated: ${result.shareId}`, "info");
    } catch (e) {
      addLog("Cloud", `Failed to generate share ID: ${e}`, "error");
    }
  };

  const copyToClipboard = (text: string, label: string) => {
    navigator.clipboard.writeText(text);
    addLog("Cloud", `${label} copied to clipboard`, "info");
  };

  const downloadExport = () => {
    const blob = new Blob([exportJson()], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${projectName().replace(/\s+/g, "_")}.neurobench.json`;
    a.click();
    URL.revokeObjectURL(url);
    addLog("Cloud", "Project file downloaded", "success");
  };

  return (
    <div class="cloud-panel">
      <div class="cloud-header">
        <h3>☁️ Cloud Sync</h3>
      </div>

      {/* Tabs */}
      <div class="cloud-tabs">
        <button 
          class={`tab ${activeTab() === "export" ? "active" : ""}`}
          onClick={() => setActiveTab("export")}
        >
          📤 Export
        </button>
        <button 
          class={`tab ${activeTab() === "import" ? "active" : ""}`}
          onClick={() => setActiveTab("import")}
        >
          📥 Import
        </button>
        <button 
          class={`tab ${activeTab() === "share" ? "active" : ""}`}
          onClick={() => setActiveTab("share")}
        >
          🔗 Share
        </button>
      </div>

      {/* Export Tab */}
      <Show when={activeTab() === "export"}>
        <div class="export-section">
          <div class="config-row">
            <label>Project Name</label>
            <input 
              type="text" 
              value={projectName()}
              onInput={(e) => setProjectName(e.target.value)}
            />
          </div>
          <div class="config-row">
            <label>Description</label>
            <input 
              type="text" 
              value={description()}
              onInput={(e) => setDescription(e.target.value)}
            />
          </div>
          <div class="config-row">
            <label>MCU Target</label>
            <select value={mcuTarget()} onChange={(e) => setMcuTarget(e.target.value)}>
              <option value="STM32F407">STM32F407</option>
              <option value="STM32F103">STM32F103</option>
              <option value="STM32F411">STM32F411</option>
              <option value="ESP32">ESP32</option>
              <option value="nRF52832">nRF52832</option>
              <option value="LM3S6965">LM3S6965</option>
            </select>
          </div>

          <div class="file-count">
            {props.generatedFiles?.length || 0} files ready for export
          </div>

          <button 
            class="export-btn"
            onClick={exportProject}
            disabled={isLoading()}
          >
            {isLoading() ? "Exporting..." : "📦 Export Project"}
          </button>

          <Show when={exportJson()}>
            <div class="export-result">
              <div class="result-header">
                <span>Export JSON</span>
                <div class="result-actions">
                  <button onClick={() => copyToClipboard(exportJson(), "JSON")}>📋 Copy</button>
                  <button onClick={downloadExport}>💾 Download</button>
                </div>
              </div>
              <pre class="json-preview">{exportJson().substring(0, 500)}...</pre>
            </div>
          </Show>
        </div>
      </Show>

      {/* Import Tab */}
      <Show when={activeTab() === "import"}>
        <div class="import-section">
          <div class="config-row">
            <label>Paste Project JSON</label>
          </div>
          <textarea 
            class="json-input"
            placeholder="Paste exported project JSON here..."
            value={importJson()}
            onInput={(e) => setImportJson(e.target.value)}
          />
          
          <button 
            class="import-btn"
            onClick={importProject}
            disabled={isLoading() || !importJson().trim()}
          >
            {isLoading() ? "Importing..." : "📥 Import Project"}
          </button>

          <Show when={importedProject()}>
            <div class="import-result">
              <h4>✅ Project Imported</h4>
              <div class="project-info">
                <div class="info-row">
                  <span>Name:</span>
                  <strong>{importedProject()!.name}</strong>
                </div>
                <div class="info-row">
                  <span>MCU:</span>
                  <strong>{importedProject()!.mcu_target}</strong>
                </div>
                <div class="info-row">
                  <span>Files:</span>
                  <strong>{importedProject()!.files.length}</strong>
                </div>
              </div>
              <div class="file-list">
                <For each={importedProject()!.files}>
                  {(file) => (
                    <div class="file-item">
                      <span class="file-icon">📄</span>
                      <span class="file-name">{file.path}</span>
                      <span class="file-lang">{file.language}</span>
                    </div>
                  )}
                </For>
              </div>
            </div>
          </Show>
        </div>
      </Show>

      {/* Share Tab */}
      <Show when={activeTab() === "share"}>
        <div class="share-section">
          <div class="share-info">
            <h4>🔗 Share Your Project</h4>
            <p>Generate a unique share ID to share your configuration with others.</p>
          </div>

          <button 
            class="share-btn"
            onClick={generateShareLink}
          >
            🎲 Generate Share ID
          </button>

          <Show when={shareId()}>
            <div class="share-result">
              <div class="share-id-display">
                <span class="share-label">Share ID:</span>
                <code class="share-id">{shareId()}</code>
                <button onClick={() => copyToClipboard(shareId(), "Share ID")}>
                  📋
                </button>
              </div>
            </div>
          </Show>

          <div class="share-methods">
            <h4>Other Sharing Options</h4>
            <div class="method-list">
              <div class="method">
                <span class="method-icon">📋</span>
                <span>Copy exported JSON and share directly</span>
              </div>
              <div class="method">
                <span class="method-icon">💾</span>
                <span>Download .neurobench.json file</span>
              </div>
              <div class="method">
                <span class="method-icon">🐙</span>
                <span>Create a GitHub Gist (manual)</span>
              </div>
            </div>
          </div>
        </div>
      </Show>

      <style>{`
        .cloud-panel {
          background: var(--bg-secondary, #1a1a2e);
          border: 1px solid var(--border, #333);
          border-radius: 8px;
          padding: 12px;
        }

        .cloud-header {
          margin-bottom: 12px;
        }

        .cloud-header h3 {
          margin: 0;
          font-size: 14px;
        }

        .cloud-tabs {
          display: flex;
          gap: 4px;
          margin-bottom: 16px;
        }

        .tab {
          flex: 1;
          padding: 10px;
          background: transparent;
          border: 1px solid #333;
          color: #888;
          cursor: pointer;
          border-radius: 6px;
          font-size: 12px;
          transition: all 0.2s;
        }

        .tab.active {
          background: linear-gradient(135deg, #8b5cf6, #7c3aed);
          color: white;
          border-color: #8b5cf6;
        }

        .config-row {
          margin-bottom: 12px;
        }

        .config-row label {
          display: block;
          color: #888;
          font-size: 12px;
          margin-bottom: 6px;
        }

        .config-row input,
        .config-row select {
          width: 100%;
          background: rgba(255,255,255,0.05);
          border: 1px solid #333;
          color: #fff;
          padding: 10px 12px;
          border-radius: 6px;
          font-size: 13px;
        }

        .file-count {
          text-align: center;
          color: #888;
          font-size: 12px;
          margin: 12px 0;
        }

        .export-btn,
        .import-btn,
        .share-btn {
          width: 100%;
          background: linear-gradient(135deg, #8b5cf6, #7c3aed);
          color: white;
          border: none;
          padding: 12px 20px;
          border-radius: 6px;
          cursor: pointer;
          font-weight: 600;
          font-size: 14px;
          margin-bottom: 16px;
        }

        .export-btn:disabled,
        .import-btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .export-result,
        .import-result {
          background: rgba(0,0,0,0.2);
          border-radius: 6px;
          padding: 12px;
        }

        .result-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 10px;
          font-size: 12px;
          color: #888;
        }

        .result-actions {
          display: flex;
          gap: 8px;
        }

        .result-actions button {
          background: rgba(255,255,255,0.1);
          border: 1px solid #333;
          color: #ccc;
          padding: 4px 10px;
          border-radius: 4px;
          cursor: pointer;
          font-size: 11px;
        }

        .json-preview {
          background: #11111b;
          padding: 10px;
          border-radius: 4px;
          font-size: 11px;
          color: #6c7086;
          overflow: hidden;
          max-height: 150px;
          font-family: 'Fira Code', monospace;
        }

        .json-input {
          width: 100%;
          min-height: 150px;
          background: rgba(255,255,255,0.05);
          border: 1px solid #333;
          color: #fff;
          padding: 12px;
          border-radius: 6px;
          font-size: 12px;
          font-family: 'Fira Code', monospace;
          resize: vertical;
          margin-bottom: 12px;
        }

        .import-result h4 {
          margin: 0 0 12px 0;
          color: #4ade80;
        }

        .project-info {
          margin-bottom: 12px;
        }

        .info-row {
          display: flex;
          justify-content: space-between;
          padding: 6px 0;
          border-bottom: 1px solid #333;
          font-size: 12px;
        }

        .file-list {
          max-height: 150px;
          overflow-y: auto;
        }

        .file-item {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 6px 0;
          font-size: 12px;
        }

        .file-name {
          flex: 1;
          color: #ccc;
        }

        .file-lang {
          color: #888;
          font-size: 10px;
          text-transform: uppercase;
        }

        .share-info {
          text-align: center;
          margin-bottom: 16px;
        }

        .share-info h4 {
          margin: 0 0 8px 0;
        }

        .share-info p {
          color: #888;
          font-size: 12px;
          margin: 0;
        }

        .share-result {
          background: rgba(139, 92, 246, 0.1);
          border: 1px solid rgba(139, 92, 246, 0.3);
          border-radius: 6px;
          padding: 16px;
          margin-bottom: 16px;
        }

        .share-id-display {
          display: flex;
          align-items: center;
          gap: 12px;
          justify-content: center;
        }

        .share-label {
          color: #888;
          font-size: 12px;
        }

        .share-id {
          background: #11111b;
          padding: 8px 16px;
          border-radius: 4px;
          font-family: 'Fira Code', monospace;
          color: #8b5cf6;
          font-size: 14px;
        }

        .share-id-display button {
          background: transparent;
          border: none;
          cursor: pointer;
          font-size: 16px;
        }

        .share-methods h4 {
          margin: 0 0 12px 0;
          font-size: 12px;
          color: #888;
          text-transform: uppercase;
        }

        .method-list {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .method {
          display: flex;
          align-items: center;
          gap: 10px;
          padding: 10px;
          background: rgba(255,255,255,0.03);
          border-radius: 6px;
          font-size: 12px;
          color: #888;
        }

        .method-icon {
          font-size: 16px;
        }
      `}</style>
    </div>
  );
}
