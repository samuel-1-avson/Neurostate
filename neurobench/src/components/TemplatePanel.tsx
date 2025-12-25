import { createSignal, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface Template {
  id: string;
  name: string;
  description: string;
  category: string;
  mcu_targets: string[];
  difficulty: string;
  files: { path: string; content: string; description: string }[];
}

interface TemplatePanelProps {
  onSelect?: (template: Template) => void;
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function TemplatePanel(props: TemplatePanelProps) {
  const [templates, setTemplates] = createSignal<Template[]>([]);
  const [categories, setCategories] = createSignal<string[]>([]);
  const [selectedCategory, setSelectedCategory] = createSignal("all");
  const [selectedTemplate, setSelectedTemplate] = createSignal<Template | null>(null);
  const [isLoading, setIsLoading] = createSignal(false);

  const loadTemplates = async () => {
    setIsLoading(true);
    try {
      const result = await invoke("templates_get_all") as Template[];
      setTemplates(result);
      
      const cats = await invoke("templates_get_categories") as { categories: string[] };
      setCategories(["all", ...cats.categories]);
      
      props.onLog?.("Templates", `Loaded ${result.length} templates`, "success");
    } catch (e) {
      props.onLog?.("Templates", `Failed to load: ${e}`, "error");
    }
    setIsLoading(false);
  };

  const filteredTemplates = () => {
    if (selectedCategory() === "all") return templates();
    return templates().filter(t => t.category === selectedCategory());
  };

  const getDifficultyColor = (difficulty: string) => {
    switch (difficulty) {
      case "beginner": return "#4ade80";
      case "intermediate": return "#fbbf24";
      case "advanced": return "#ef4444";
      default: return "#888";
    }
  };

  // Load on mount
  if (templates().length === 0) {
    loadTemplates();
  }

  return (
    <div class="template-panel">
      <div class="panel-header">
        <h3>📋 Project Templates</h3>
        <button class="refresh-btn" onClick={loadTemplates}>🔄</button>
      </div>

      {/* Category filter */}
      <div class="category-filter">
        <For each={categories()}>
          {(cat) => (
            <button 
              class={`cat-btn ${selectedCategory() === cat ? "active" : ""}`}
              onClick={() => setSelectedCategory(cat)}
            >
              {cat === "all" ? "All" : cat}
            </button>
          )}
        </For>
      </div>

      {/* Template list */}
      <div class="template-list">
        <Show when={isLoading()}>
          <div class="loading">Loading templates...</div>
        </Show>
        
        <For each={filteredTemplates()}>
          {(template) => (
            <div 
              class={`template-card ${selectedTemplate()?.id === template.id ? "selected" : ""}`}
              onClick={() => setSelectedTemplate(template)}
            >
              <div class="card-header">
                <span class="template-name">{template.name}</span>
                <span 
                  class="difficulty-badge" 
                  style={{ color: getDifficultyColor(template.difficulty) }}
                >
                  {template.difficulty}
                </span>
              </div>
              <p class="template-desc">{template.description}</p>
              <div class="template-meta">
                <span class="category">{template.category}</span>
                <span class="mcu-count">{template.mcu_targets?.length || 0} MCUs</span>
              </div>
            </div>
          )}
        </For>
      </div>

      {/* Selected template details */}
      <Show when={selectedTemplate()}>
        <div class="template-details">
          <h4>{selectedTemplate()!.name}</h4>
          <div class="files-list">
            <For each={selectedTemplate()!.files}>
              {(file) => (
                <div class="file-item">
                  <span>📄 {file.path}</span>
                </div>
              )}
            </For>
          </div>
          <button 
            class="use-btn"
            onClick={() => props.onSelect?.(selectedTemplate()!)}
          >
            Use Template
          </button>
        </div>
      </Show>

      <style>{`
        .template-panel {
          background: var(--bg-secondary, #1a1a2e);
          border: 1px solid var(--border, #333);
          border-radius: 8px;
          padding: 12px;
        }

        .panel-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 12px;
        }

        .panel-header h3 { margin: 0; font-size: 14px; }
        .refresh-btn { background: transparent; border: none; cursor: pointer; }

        .category-filter {
          display: flex;
          gap: 4px;
          flex-wrap: wrap;
          margin-bottom: 12px;
        }

        .cat-btn {
          padding: 6px 12px;
          background: rgba(255,255,255,0.05);
          border: 1px solid #333;
          color: #888;
          border-radius: 4px;
          cursor: pointer;
          font-size: 11px;
        }

        .cat-btn.active {
          background: linear-gradient(135deg, #3b82f6, #2563eb);
          color: white;
          border-color: #3b82f6;
        }

        .template-list {
          max-height: 250px;
          overflow-y: auto;
        }

        .template-card {
          padding: 10px;
          background: rgba(255,255,255,0.03);
          border: 1px solid transparent;
          border-radius: 6px;
          margin-bottom: 8px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .template-card:hover {
          background: rgba(59, 130, 246, 0.1);
        }

        .template-card.selected {
          border-color: #3b82f6;
          background: rgba(59, 130, 246, 0.15);
        }

        .card-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .template-name { font-weight: 600; font-size: 13px; }
        .difficulty-badge { font-size: 10px; text-transform: uppercase; }
        .template-desc { color: #888; font-size: 11px; margin: 6px 0; }

        .template-meta {
          display: flex;
          gap: 12px;
          font-size: 10px;
          color: #666;
        }

        .template-details {
          margin-top: 12px;
          padding: 12px;
          background: rgba(0,0,0,0.2);
          border-radius: 6px;
        }

        .template-details h4 { margin: 0 0 10px 0; font-size: 13px; }
        .files-list { margin-bottom: 12px; }
        .file-item { font-size: 12px; color: #888; padding: 4px 0; }

        .use-btn {
          width: 100%;
          background: linear-gradient(135deg, #4ade80, #22c55e);
          color: #000;
          border: none;
          padding: 10px;
          border-radius: 6px;
          cursor: pointer;
          font-weight: 600;
        }

        .loading { text-align: center; color: #888; padding: 20px; }
      `}</style>
    </div>
  );
}
