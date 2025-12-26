// EnhancedEditor - Dedicated view for the enhanced node editor with code generation
import { Component, createSignal, Show, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import FSMCanvas from "./FSMCanvas";
import "./EnhancedEditor.css";

interface GeneratedCode {
  includes: string[];
  defines: string[];
  types: string[];
  globals: string[];
  init_code: string[];
  handler_code: string[];
  main_loop: string[];
}

interface CodeFile {
  name: string;
  content: string;
  language: string;
}

interface NodeData {
  id: string;
  node_type: string;
  properties: Record<string, unknown>;
}

const EnhancedEditor: Component<{
  onClose?: () => void;
  showFullscreen?: boolean;
}> = (props) => {
  const [nodes, setNodes] = createSignal<NodeData[]>([]);
  const [generatedCode, setGeneratedCode] = createSignal<GeneratedCode | null>(null);
  const [generatedFiles, setGeneratedFiles] = createSignal<CodeFile[]>([]);
  const [isGenerating, setIsGenerating] = createSignal(false);
  const [showCodePanel, setShowCodePanel] = createSignal(false);
  const [activeCodeTab, setActiveCodeTab] = createSignal<"preview" | "files">("preview");
  const [selectedFile, setSelectedFile] = createSignal<string | null>(null);

  // Generate code from current nodes (both single and multi-file)
  const handleGenerateCode = async () => {
    setIsGenerating(true);
    try {
      const currentNodes = nodes();
      const nodeData = currentNodes.map(n => ({
        node_type: n.node_type,
        properties: n.properties
      }));
      
      // Generate single file view
      const codeResult = await invoke<GeneratedCode>("nodes_generate_code", {
        nodesData: nodeData
      });
      setGeneratedCode(codeResult);
      
      // Generate multi-file export
      const filesResult = await invoke<CodeFile[]>("nodes_generate_files", {
        nodesData: nodeData
      });
      setGeneratedFiles(filesResult);
      setSelectedFile(filesResult.length > 0 ? filesResult[0].name : null);
      
      setShowCodePanel(true);
    } catch (error) {
      console.error("Code generation failed:", error);
    } finally {
      setIsGenerating(false);
    }
  };

  // Render full code output (single file view)
  const renderFullCode = () => {
    const code = generatedCode();
    if (!code) return "";

    let output = "";
    const uniqueIncludes = [...new Set(code.includes)].sort();
    for (const inc of uniqueIncludes) {
      output += `#include ${inc}\n`;
    }
    output += "\n";
    for (const def of code.defines) {
      output += `#define ${def}\n`;
    }
    if (code.defines.length > 0) output += "\n";
    for (const typ of code.types) {
      output += `${typ}\n\n`;
    }
    for (const glob of code.globals) {
      output += `${glob}\n`;
    }
    if (code.globals.length > 0) output += "\n";
    for (const handler of code.handler_code) {
      output += `${handler}\n\n`;
    }
    output += "void System_Init(void) {\n";
    for (const init of code.init_code) {
      output += `    ${init}\n`;
    }
    output += "}\n\nvoid System_Update(void) {\n";
    for (const main of code.main_loop) {
      output += `    ${main}\n`;
    }
    output += "}\n";
    return output;
  };

  // Get selected file content
  const getSelectedFileContent = () => {
    const files = generatedFiles();
    const selected = selectedFile();
    if (!selected) return "";
    const file = files.find(f => f.name === selected);
    return file?.content || "";
  };

  // Copy code to clipboard
  const copyToClipboard = (content?: string) => {
    navigator.clipboard.writeText(content || renderFullCode());
  };

  // Download single file
  const downloadFile = (name: string, content: string) => {
    const blob = new Blob([content], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = name;
    a.click();
    URL.revokeObjectURL(url);
  };

  // Download all files as ZIP
  const downloadAllFiles = async () => {
    const files = generatedFiles();
    // Simple approach: download each file
    for (const file of files) {
      downloadFile(file.name, file.content);
      await new Promise(r => setTimeout(r, 200)); // Small delay between downloads
    }
  };

  return (
    <div class={`enhanced-editor ${props.showFullscreen ? "fullscreen" : ""}`}>
      {/* Header */}
      <header class="enhanced-editor-header">
        <div class="enhanced-editor-title">
          <span class="title-icon">✨</span>
          <h1>Enhanced Node Editor</h1>
          <span class="badge">40+ Node Types</span>
        </div>
        
        <div class="enhanced-editor-actions">
          <button 
            class="generate-btn"
            onClick={handleGenerateCode}
            disabled={isGenerating()}
          >
            {isGenerating() ? (
              <>
                <span class="spinner" />
                Generating...
              </>
            ) : (
              <>
                <span class="icon">⚡</span>
                Generate Code
              </>
            )}
          </button>
          
          <Show when={generatedCode()}>
            <button class="action-btn" onClick={() => setShowCodePanel(!showCodePanel())}>
              {showCodePanel() ? "Hide Code" : "Show Code"}
            </button>
          </Show>
          
          <Show when={props.onClose}>
            <button class="close-btn" onClick={props.onClose}>
              ✕
            </button>
          </Show>
        </div>
      </header>
      
      {/* Main Content */}
      <div class="enhanced-editor-content">
        {/* Canvas Section */}
        <div class={`canvas-section ${showCodePanel() ? "with-code" : ""}`}>
          <FSMCanvas 
            showPalette={true} 
            showProperties={true}
          />
        </div>
        
        {/* Code Panel */}
        <Show when={showCodePanel() && generatedCode()}>
          <div class="code-panel">
            <div class="code-panel-header">
              <div class="code-tabs">
                <button 
                  class={activeCodeTab() === "preview" ? "active" : ""}
                  onClick={() => setActiveCodeTab("preview")}
                >
                  Preview
                </button>
                <button 
                  class={activeCodeTab() === "files" ? "active" : ""}
                  onClick={() => setActiveCodeTab("files")}
                >
                  Files ({generatedFiles().length})
                </button>
              </div>
              
              <div class="code-actions">
                <Show when={activeCodeTab() === "preview"}>
                  <button onClick={() => copyToClipboard()} title="Copy to clipboard">📋</button>
                  <button onClick={() => downloadFile("generated_code.c", renderFullCode())} title="Download">⬇️</button>
                </Show>
                <Show when={activeCodeTab() === "files"}>
                  <button onClick={downloadAllFiles} title="Download all files">📦</button>
                </Show>
              </div>
            </div>
            
            <div class="code-content">
              <Show when={activeCodeTab() === "preview"}>
                <pre><code>{renderFullCode()}</code></pre>
              </Show>
              
              <Show when={activeCodeTab() === "files"}>
                <div class="files-container">
                  {/* File List Sidebar */}
                  <div class="file-list">
                    <For each={generatedFiles()}>
                      {(file) => (
                        <div 
                          class={`file-item ${selectedFile() === file.name ? "selected" : ""}`}
                          onClick={() => setSelectedFile(file.name)}
                        >
                          <span class="file-icon">
                            {file.name.endsWith(".h") ? "📘" : "📄"}
                          </span>
                          <span class="file-name">{file.name}</span>
                          <button 
                            class="file-download" 
                            onClick={(e) => {
                              e.stopPropagation();
                              downloadFile(file.name, file.content);
                            }}
                          >
                            ⬇️
                          </button>
                        </div>
                      )}
                    </For>
                  </div>
                  
                  {/* File Content Preview */}
                  <div class="file-preview">
                    <div class="file-preview-header">
                      <span>{selectedFile() || "No file selected"}</span>
                      <Show when={selectedFile()}>
                        <button onClick={() => copyToClipboard(getSelectedFileContent())}>
                          📋 Copy
                        </button>
                      </Show>
                    </div>
                    <pre><code>{getSelectedFileContent()}</code></pre>
                  </div>
                </div>
              </Show>
            </div>
            
            {/* Stats */}
            <div class="code-stats">
              <span>{generatedCode()?.includes.length || 0} includes</span>
              <span>{generatedCode()?.globals.length || 0} globals</span>
              <span>{generatedCode()?.handler_code.length || 0} handlers</span>
              <span>{generatedFiles().length} files</span>
            </div>
          </div>
        </Show>
      </div>
      
      {/* Status Bar */}
      <footer class="enhanced-editor-footer">
        <div class="status-left">
          <span class="node-count">{nodes().length} nodes</span>
        </div>
        <div class="status-right">
          <span class="hint">Drag nodes from palette • Connect ports • Generate code</span>
        </div>
      </footer>
    </div>
  );
};

export default EnhancedEditor;
