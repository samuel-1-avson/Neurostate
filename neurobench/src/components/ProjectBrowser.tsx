import { createSignal, onMount, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

// Types matching Rust backend
interface ProjectManifest {
  id: string;
  name: string;
  mcu_target: string;
  chip: string | null;
  created_at: string;
  modified_at: string;
  config_hash: string;
  settings: ProjectSettings;
}

interface ProjectSettings {
  clock_speed_hz: number | null;
  flash_size_kb: number | null;
  ram_size_kb: number | null;
  rtos: string | null;
  optimization: string | null;
  defines: Record<string, string>;
}

interface ProjectInfo {
  id: string;
  name: string;
  mcu_target: string;
  path: string;
  modified_at: string;
}

interface Props {
  projectsDir?: string;
  onProjectOpen?: (project: ProjectManifest, path: string) => void;
}

export default function ProjectBrowser(props: Props) {
  const [projects, setProjects] = createSignal<ProjectInfo[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [showCreate, setShowCreate] = createSignal(false);
  
  // Create form
  const [newName, setNewName] = createSignal("");
  const [newMcu, setNewMcu] = createSignal("STM32F407VG");
  const [newPath, setNewPath] = createSignal("");

  const loadProjects = async () => {
    if (!props.projectsDir) return;
    
    setLoading(true);
    try {
      const result = await invoke<ProjectInfo[]>("project_list", {
        dir: props.projectsDir
      });
      setProjects(result);
      setError(null);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const openProject = async (info: ProjectInfo) => {
    try {
      const manifest = await invoke<ProjectManifest>("project_load", {
        path: info.path
      });
      props.onProjectOpen?.(manifest, info.path);
    } catch (err) {
      setError(`Failed to open: ${err}`);
    }
  };

  const createProject = async () => {
    if (!newName() || !newPath()) {
      setError("Name and path required");
      return;
    }
    
    try {
      const manifest = await invoke<ProjectManifest>("project_create", {
        path: newPath(),
        name: newName(),
        mcuTarget: newMcu()
      });
      
      setShowCreate(false);
      setNewName("");
      setNewPath("");
      
      // Refresh and open new project
      await loadProjects();
      props.onProjectOpen?.(manifest, newPath());
    } catch (err) {
      setError(`Create failed: ${err}`);
    }
  };

  const deleteProject = async (path: string) => {
    if (!confirm("Delete this project?")) return;
    
    try {
      await invoke("project_delete", { path });
      await loadProjects();
    } catch (err) {
      setError(`Delete failed: ${err}`);
    }
  };
  
  const formatDate = (iso: string) => {
    return new Date(iso).toLocaleDateString();
  };

  onMount(() => {
    loadProjects();
  });

  return (
    <div class="project-browser">
      <div class="header">
        <h3>Projects</h3>
        <button class="create-btn" onClick={() => setShowCreate(true)}>+ New</button>
      </div>
      
      <Show when={error()}>
        <div class="error">{error()}</div>
      </Show>
      
      <Show when={showCreate()}>
        <div class="create-form">
          <input 
            type="text" 
            placeholder="Project Name" 
            value={newName()} 
            onInput={(e) => setNewName(e.currentTarget.value)}
          />
          <input 
            type="text" 
            placeholder="Path" 
            value={newPath()} 
            onInput={(e) => setNewPath(e.currentTarget.value)}
          />
          <select value={newMcu()} onChange={(e) => setNewMcu(e.currentTarget.value)}>
            <option value="STM32F407VG">STM32F407VG</option>
            <option value="STM32F103C8">STM32F103C8</option>
            <option value="STM32H743ZI">STM32H743ZI</option>
            <option value="nRF52840_xxAA">nRF52840</option>
          </select>
          <div class="form-buttons">
            <button onClick={createProject}>Create</button>
            <button class="cancel" onClick={() => setShowCreate(false)}>Cancel</button>
          </div>
        </div>
      </Show>
      
      <Show when={loading()}>
        <div class="loading">Loading...</div>
      </Show>
      
      <div class="project-list">
        <For each={projects()}>
          {(project) => (
            <div class="project-item" onClick={() => openProject(project)}>
              <div class="project-info">
                <span class="name">{project.name}</span>
                <span class="mcu">{project.mcu_target}</span>
              </div>
              <div class="project-meta">
                <span class="date">{formatDate(project.modified_at)}</span>
                <button 
                  class="delete-btn" 
                  onClick={(e) => { e.stopPropagation(); deleteProject(project.path); }}
                >×</button>
              </div>
            </div>
          )}
        </For>
      </div>
      
      <Show when={projects().length === 0 && !loading()}>
        <div class="empty">No projects found</div>
      </Show>

      <style>{`
        .project-browser {
          display: flex;
          flex-direction: column;
          gap: 12px;
          padding: 16px;
          background: #1e1e1e;
          border-radius: 8px;
          color: #e0e0e0;
        }
        
        .header {
          display: flex;
          justify-content: space-between;
          align-items: center;
        }
        
        .header h3 {
          margin: 0;
          font-size: 16px;
        }
        
        .create-btn {
          background: #4CAF50;
          color: white;
          border: none;
          padding: 6px 12px;
          border-radius: 4px;
          cursor: pointer;
        }
        
        .create-form {
          display: flex;
          flex-direction: column;
          gap: 8px;
          padding: 12px;
          background: #2d2d2d;
          border-radius: 4px;
        }
        
        .create-form input, .create-form select {
          padding: 8px;
          background: #333;
          border: 1px solid #444;
          border-radius: 4px;
          color: #e0e0e0;
        }
        
        .form-buttons {
          display: flex;
          gap: 8px;
        }
        
        .form-buttons button {
          flex: 1;
          padding: 8px;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        }
        
        .form-buttons button:first-child {
          background: #4CAF50;
          color: white;
        }
        
        .form-buttons button.cancel {
          background: #666;
          color: white;
        }
        
        .project-list {
          display: flex;
          flex-direction: column;
          gap: 4px;
        }
        
        .project-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 10px 12px;
          background: #2d2d2d;
          border-radius: 4px;
          cursor: pointer;
          transition: background 0.2s;
        }
        
        .project-item:hover {
          background: #383838;
        }
        
        .project-info {
          display: flex;
          flex-direction: column;
          gap: 2px;
        }
        
        .name {
          font-weight: 500;
        }
        
        .mcu {
          font-size: 12px;
          color: #888;
        }
        
        .project-meta {
          display: flex;
          align-items: center;
          gap: 8px;
        }
        
        .date {
          font-size: 12px;
          color: #666;
        }
        
        .delete-btn {
          width: 24px;
          height: 24px;
          background: transparent;
          border: 1px solid #666;
          border-radius: 4px;
          color: #888;
          cursor: pointer;
          opacity: 0;
          transition: opacity 0.2s;
        }
        
        .project-item:hover .delete-btn {
          opacity: 1;
        }
        
        .delete-btn:hover {
          background: #f44336;
          border-color: #f44336;
          color: white;
        }
        
        .error {
          background: rgba(244, 67, 54, 0.1);
          color: #f44336;
          padding: 8px;
          border-radius: 4px;
          font-size: 13px;
        }
        
        .loading, .empty {
          text-align: center;
          color: #666;
          padding: 20px;
        }
      `}</style>
    </div>
  );
}
