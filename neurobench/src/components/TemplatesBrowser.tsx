import { createSignal, onMount, For, Show, createMemo } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

// Types matching Rust backend
interface ProjectTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  mcu: string;
  complexity: string;
  tags: string[];
  files: TemplateFile[];
}

interface TemplateFile {
  path: string;
  content: string;
}

interface Props {
  onApply?: (template: ProjectTemplate, path: string) => void;
  onCancel?: () => void;
}

export default function TemplatesBrowser(props: Props) {
  const [templates, setTemplates] = createSignal<ProjectTemplate[]>([]);
  const [categories, setCategories] = createSignal<string[]>([]);
  const [selectedCategory, setSelectedCategory] = createSignal<string | null>(null);
  const [selectedTemplate, setSelectedTemplate] = createSignal<ProjectTemplate | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  
  // Apply form
  const [showApply, setShowApply] = createSignal(false);
  const [projectName, setProjectName] = createSignal("");
  const [projectPath, setProjectPath] = createSignal("");
  const [applying, setApplying] = createSignal(false);

  const loadTemplates = async () => {
    setLoading(true);
    try {
      const result = await invoke<{ templates: ProjectTemplate[]; categories: string[] }>(
        "templates_list",
        { category: selectedCategory() }
      );
      setTemplates(result.templates);
      setCategories(result.categories);
      setError(null);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const filteredTemplates = createMemo(() => {
    const cat = selectedCategory();
    if (!cat) return templates();
    return templates().filter(t => t.category === cat);
  });

  const applyTemplate = async () => {
    const template = selectedTemplate();
    if (!template || !projectName() || !projectPath()) return;
    
    setApplying(true);
    try {
      await invoke("templates_apply", {
        templateId: template.id,
        projectPath: projectPath(),
        projectName: projectName(),
      });
      
      setShowApply(false);
      props.onApply?.(template, projectPath());
    } catch (err) {
      setError(`Apply failed: ${err}`);
    } finally {
      setApplying(false);
    }
  };

  onMount(() => {
    loadTemplates();
  });

  return (
    <div class="templates-browser">
      <div class="header">
        <h2>Project Templates</h2>
        <button class="close-btn" onClick={props.onCancel}>×</button>
      </div>
      
      <Show when={error()}>
        <div class="error">{error()}</div>
      </Show>
      
      <div class="content">
        {/* Categories sidebar */}
        <div class="sidebar">
          <div class="section-title">Categories</div>
          <div 
            class={`category-item ${!selectedCategory() ? 'active' : ''}`}
            onClick={() => { setSelectedCategory(null); loadTemplates(); }}
          >
            All Templates
          </div>
          <For each={categories()}>
            {(cat) => (
              <div 
                class={`category-item ${selectedCategory() === cat ? 'active' : ''}`}
                onClick={() => { setSelectedCategory(cat); loadTemplates(); }}
              >
                {cat}
              </div>
            )}
          </For>
        </div>
        
        {/* Templates grid */}
        <div class="templates-grid">
          <Show when={loading()}>
            <div class="loading">Loading templates...</div>
          </Show>
          
          <For each={filteredTemplates()}>
            {(template) => (
              <div 
                class={`template-card ${selectedTemplate()?.id === template.id ? 'selected' : ''}`}
                onClick={() => setSelectedTemplate(template)}
              >
                <div class="card-header">
                  <span class="name">{template.name}</span>
                  <span class={`complexity ${template.complexity.toLowerCase()}`}>
                    {template.complexity}
                  </span>
                </div>
                <div class="description">{template.description}</div>
                <div class="meta">
                  <span class="mcu">{template.mcu}</span>
                  <span class="files">{template.files.length} files</span>
                </div>
                <div class="tags">
                  <For each={template.tags.slice(0, 3)}>
                    {(tag) => <span class="tag">{tag}</span>}
                  </For>
                </div>
              </div>
            )}
          </For>
        </div>
        
        {/* Preview pane */}
        <Show when={selectedTemplate()}>
          <div class="preview-pane">
            <h3>{selectedTemplate()?.name}</h3>
            <p class="desc">{selectedTemplate()?.description}</p>
            
            <div class="info-row">
              <span class="label">MCU:</span>
              <span class="value">{selectedTemplate()?.mcu}</span>
            </div>
            <div class="info-row">
              <span class="label">Category:</span>
              <span class="value">{selectedTemplate()?.category}</span>
            </div>
            
            <div class="files-list">
              <div class="section-title">Files</div>
              <For each={selectedTemplate()?.files}>
                {(file) => (
                  <div class="file-item">{file.path}</div>
                )}
              </For>
            </div>
            
            <button class="use-btn" onClick={() => setShowApply(true)}>
              Use This Template
            </button>
          </div>
        </Show>
      </div>
      
      {/* Apply dialog */}
      <Show when={showApply()}>
        <div class="dialog-overlay">
          <div class="apply-dialog">
            <h3>Create Project from Template</h3>
            <div class="form-group">
              <label>Project Name</label>
              <input 
                type="text" 
                value={projectName()} 
                onInput={(e) => setProjectName(e.currentTarget.value)}
                placeholder="My Project"
              />
            </div>
            <div class="form-group">
              <label>Project Path</label>
              <input 
                type="text" 
                value={projectPath()} 
                onInput={(e) => setProjectPath(e.currentTarget.value)}
                placeholder="C:/projects/my-project"
              />
            </div>
            <div class="dialog-buttons">
              <button 
                class="create-btn" 
                onClick={applyTemplate}
                disabled={applying() || !projectName() || !projectPath()}
              >
                {applying() ? 'Creating...' : 'Create Project'}
              </button>
              <button class="cancel-btn" onClick={() => setShowApply(false)}>
                Cancel
              </button>
            </div>
          </div>
        </div>
      </Show>

      <style>{`
        .templates-browser {
          display: flex;
          flex-direction: column;
          height: 100%;
          background: var(--bg-base, #0d0d0d);
          color: var(--text-primary, #e8e8e8);
          font-family: var(--font-mono, 'IBM Plex Mono', monospace);
        }
        
        .header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 16px 20px;
          background: var(--bg-panel, #141414);
          border-bottom: 1px solid var(--border, #2a2a2a);
        }
        
        .header h2 { 
          margin: 0; 
          font-size: 14px;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 1px;
          color: var(--accent, #e49b0f);
        }
        
        .close-btn {
          background: transparent;
          border: none;
          color: var(--text-dim, #606060);
          font-size: 20px;
          cursor: pointer;
          width: 32px;
          height: 32px;
          border-radius: 3px;
        }
        
        .close-btn:hover {
          background: var(--bg-elevated, #222);
          color: var(--text-primary, #e8e8e8);
        }
        
        .content {
          display: flex;
          flex: 1;
          overflow: hidden;
        }
        
        .sidebar {
          width: 180px;
          background: var(--bg-panel, #141414);
          border-right: 1px solid var(--border, #2a2a2a);
          padding: 14px;
        }
        
        .section-title {
          font-size: 10px;
          font-weight: 600;
          color: var(--text-dim, #606060);
          text-transform: uppercase;
          letter-spacing: 1px;
          margin-bottom: 10px;
          padding-bottom: 6px;
          border-bottom: 1px solid var(--border, #2a2a2a);
        }
        
        .category-item {
          padding: 10px 12px;
          border-radius: 3px;
          cursor: pointer;
          font-size: 12px;
          color: var(--text-secondary, #a0a0a0);
          transition: all 0.15s;
          margin-bottom: 2px;
        }
        
        .category-item:hover { 
          background: var(--bg-elevated, #222);
          color: var(--text-primary, #e8e8e8);
        }
        
        .category-item.active { 
          background: var(--accent-subtle, rgba(228, 155, 15, 0.1));
          color: var(--accent, #e49b0f);
          border-left: 3px solid var(--accent, #e49b0f);
        }
        
        .templates-grid {
          flex: 1;
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
          gap: 14px;
          padding: 16px;
          overflow-y: auto;
          background: var(--bg-base, #0d0d0d);
        }
        
        .template-card {
          background: var(--bg-surface, #1a1a1a);
          border: 1px solid var(--border, #2a2a2a);
          border-radius: 4px;
          padding: 16px;
          cursor: pointer;
          transition: all 0.15s;
        }
        
        .template-card:hover { 
          border-color: var(--border-strong, #3a3a3a);
          background: var(--bg-elevated, #222);
        }
        
        .template-card.selected { 
          border-color: var(--accent, #e49b0f);
          box-shadow: 0 0 0 1px var(--accent-subtle, rgba(228, 155, 15, 0.2));
        }
        
        .card-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 10px;
        }
        
        .name { 
          font-weight: 600;
          font-size: 13px;
        }
        
        .complexity {
          font-size: 9px;
          font-weight: 600;
          padding: 3px 8px;
          border-radius: 2px;
          text-transform: uppercase;
          letter-spacing: 0.5px;
        }
        
        .complexity.beginner { 
          background: rgba(50, 205, 50, 0.15);
          color: var(--status-success, #32cd32);
        }
        .complexity.intermediate { 
          background: var(--accent-subtle, rgba(228, 155, 15, 0.15));
          color: var(--accent, #e49b0f);
        }
        .complexity.advanced { 
          background: rgba(220, 53, 69, 0.15);
          color: var(--status-error, #dc3545);
        }
        
        .description {
          font-size: 11px;
          color: var(--text-dim, #606060);
          margin-bottom: 10px;
          line-height: 1.5;
        }
        
        .meta {
          display: flex;
          gap: 14px;
          font-size: 10px;
          color: var(--text-dim, #606060);
          margin-bottom: 10px;
        }
        
        .mcu { color: var(--status-info, #5dade2); }
        
        .tags {
          display: flex;
          gap: 6px;
          flex-wrap: wrap;
        }
        
        .tag {
          font-size: 9px;
          background: var(--bg-panel, #141414);
          padding: 3px 8px;
          border-radius: 2px;
          border: 1px solid var(--border, #2a2a2a);
        }
        
        .preview-pane {
          width: 300px;
          background: var(--bg-panel, #141414);
          border-left: 1px solid var(--border, #2a2a2a);
          padding: 18px;
          overflow-y: auto;
        }
        
        .preview-pane h3 { 
          margin: 0 0 10px;
          font-size: 14px;
          color: var(--accent, #e49b0f);
        }
        
        .preview-pane .desc { 
          font-size: 12px; 
          color: var(--text-dim, #606060);
          margin-bottom: 18px;
          line-height: 1.5;
        }
        
        .info-row {
          display: flex;
          justify-content: space-between;
          font-size: 11px;
          margin-bottom: 6px;
          padding: 6px 0;
          border-bottom: 1px solid var(--border, #2a2a2a);
        }
        
        .info-row .label { color: var(--text-dim, #606060); }
        .info-row .value { color: var(--text-primary, #e8e8e8); }
        
        .files-list {
          margin-top: 18px;
        }
        
        .file-item {
          font-size: 11px;
          padding: 6px 10px;
          color: var(--text-secondary, #a0a0a0);
          background: var(--bg-surface, #1a1a1a);
          border-radius: 3px;
          margin-bottom: 4px;
        }
        
        .use-btn {
          margin-top: 18px;
          width: 100%;
          background: var(--accent, #e49b0f);
          color: #000;
          border: none;
          padding: 12px;
          border-radius: 3px;
          font-family: inherit;
          font-size: 11px;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 0.5px;
          cursor: pointer;
          transition: all 0.15s;
        }
        
        .use-btn:hover {
          box-shadow: 0 0 12px var(--accent-glow, rgba(228, 155, 15, 0.4));
        }
        
        .dialog-overlay {
          position: fixed;
          inset: 0;
          background: rgba(0, 0, 0, 0.8);
          display: flex;
          align-items: center;
          justify-content: center;
          backdrop-filter: blur(2px);
        }
        
        .apply-dialog {
          background: var(--bg-panel, #141414);
          border: 1px solid var(--border-strong, #3a3a3a);
          border-radius: 6px;
          padding: 24px;
          width: 420px;
        }
        
        .apply-dialog h3 { 
          margin: 0 0 18px;
          font-size: 14px;
          text-transform: uppercase;
          letter-spacing: 0.5px;
          color: var(--accent, #e49b0f);
        }
        
        .form-group {
          margin-bottom: 16px;
        }
        
        .form-group label {
          display: block;
          font-size: 10px;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 0.5px;
          color: var(--text-dim, #606060);
          margin-bottom: 6px;
        }
        
        .form-group input {
          width: 100%;
          padding: 10px 12px;
          background: var(--bg-input, #0f0f0f);
          border: 1px solid var(--border, #2a2a2a);
          border-radius: 3px;
          color: var(--text-primary, #e8e8e8);
          font-family: inherit;
          font-size: 12px;
        }
        
        .form-group input:focus {
          border-color: var(--accent, #e49b0f);
          outline: none;
        }
        
        .dialog-buttons {
          display: flex;
          gap: 10px;
          margin-top: 20px;
        }
        
        .create-btn {
          flex: 1;
          background: var(--status-success, #32cd32);
          color: #000;
          border: none;
          padding: 12px;
          border-radius: 3px;
          font-family: inherit;
          font-size: 11px;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 0.5px;
          cursor: pointer;
        }
        
        .create-btn:hover {
          box-shadow: 0 0 12px rgba(50, 205, 50, 0.4);
        }
        
        .create-btn:disabled { 
          background: var(--border-strong, #3a3a3a);
          color: var(--text-dim, #606060);
          cursor: not-allowed;
          box-shadow: none;
        }
        
        .cancel-btn {
          background: var(--bg-surface, #1a1a1a);
          color: var(--text-secondary, #a0a0a0);
          border: 1px solid var(--border, #2a2a2a);
          padding: 12px 20px;
          border-radius: 3px;
          font-family: inherit;
          font-size: 11px;
          font-weight: 500;
          text-transform: uppercase;
          letter-spacing: 0.5px;
          cursor: pointer;
        }
        
        .cancel-btn:hover {
          background: var(--bg-elevated, #222);
          border-color: var(--border-strong, #3a3a3a);
        }
        
        .error {
          background: rgba(220, 53, 69, 0.1);
          color: var(--status-error, #dc3545);
          padding: 10px 16px;
          border-left: 3px solid var(--status-error, #dc3545);
          font-size: 11px;
        }
        
        .loading {
          grid-column: 1 / -1;
          text-align: center;
          color: var(--text-dim, #606060);
          padding: 40px;
          font-size: 12px;
          text-transform: uppercase;
          letter-spacing: 0.5px;
        }
      `}</style>
    </div>
  );
}
