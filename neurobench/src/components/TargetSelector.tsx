import { createSignal, onMount, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface TargetProfile {
  id: string;
  name: string;
  mcu: string;
  chip: string;
  enabled: boolean;
  output_dir: string;
  defines: Record<string, string>;
  link_script: string | null;
}

interface Props {
  projectPath: string;
  onBuildAll?: () => void;
}

export default function TargetSelector(props: Props) {
  const [targets, setTargets] = createSignal<TargetProfile[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [showAdd, setShowAdd] = createSignal(false);
  const [building, setBuilding] = createSignal(false);
  
  // Add form
  const [newId, setNewId] = createSignal("");
  const [newName, setNewName] = createSignal("");
  const [newMcu, setNewMcu] = createSignal("STM32F407VG");

  const loadTargets = async () => {
    if (!props.projectPath) return;
    
    setLoading(true);
    try {
      const result = await invoke<TargetProfile[]>("targets_list", {
        projectPath: props.projectPath
      });
      setTargets(result);
    } catch (err) {
      console.error("Failed to load targets:", err);
    } finally {
      setLoading(false);
    }
  };

  const addTarget = async () => {
    if (!newId() || !newName()) return;
    
    try {
      await invoke("targets_add", {
        projectPath: props.projectPath,
        id: newId(),
        name: newName(),
        mcu: newMcu(),
        chip: newMcu(),
      });
      
      setShowAdd(false);
      setNewId("");
      setNewName("");
      await loadTargets();
    } catch (err) {
      console.error("Failed to add target:", err);
    }
  };

  const removeTarget = async (id: string) => {
    try {
      await invoke("targets_remove", {
        projectPath: props.projectPath,
        targetId: id,
      });
      await loadTargets();
    } catch (err) {
      console.error("Failed to remove target:", err);
    }
  };

  const buildAll = async () => {
    setBuilding(true);
    try {
      await invoke("targets_build_all", {
        projectPath: props.projectPath,
      });
      props.onBuildAll?.();
    } catch (err) {
      console.error("Build failed:", err);
    } finally {
      setBuilding(false);
    }
  };

  onMount(() => {
    loadTargets();
  });

  return (
    <div class="target-selector">
      <div class="header">
        <span class="title">Build Targets</span>
        <button class="add-btn" onClick={() => setShowAdd(true)}>+ Add</button>
      </div>
      
      <Show when={showAdd()}>
        <div class="add-form">
          <input 
            type="text" 
            placeholder="ID (e.g., nrf52)"
            value={newId()}
            onInput={(e) => setNewId(e.currentTarget.value)}
          />
          <input 
            type="text" 
            placeholder="Name"
            value={newName()}
            onInput={(e) => setNewName(e.currentTarget.value)}
          />
          <select value={newMcu()} onChange={(e) => setNewMcu(e.currentTarget.value)}>
            <option value="STM32F407VG">STM32F407VG</option>
            <option value="STM32F103C8">STM32F103C8</option>
            <option value="nRF52840_xxAA">nRF52840</option>
            <option value="ESP32">ESP32</option>
          </select>
          <div class="form-btns">
            <button onClick={addTarget}>Add</button>
            <button class="cancel" onClick={() => setShowAdd(false)}>Cancel</button>
          </div>
        </div>
      </Show>
      
      <div class="targets-list">
        <For each={targets()}>
          {(target) => (
            <div class={`target-item ${target.enabled ? '' : 'disabled'}`}>
              <div class="target-info">
                <span class="name">{target.name}</span>
                <span class="mcu">{target.mcu}</span>
              </div>
              <div class="target-actions">
                <span class="output">{target.output_dir}</span>
                <button class="remove" onClick={() => removeTarget(target.id)}>×</button>
              </div>
            </div>
          )}
        </For>
      </div>
      
      <Show when={targets().length === 0 && !loading()}>
        <div class="empty">No targets configured</div>
      </Show>
      
      <Show when={targets().length > 0}>
        <button 
          class="build-all-btn" 
          onClick={buildAll}
          disabled={building()}
        >
          {building() ? 'Building...' : `Build All (${targets().filter(t => t.enabled).length})`}
        </button>
      </Show>

      <style>{`
        .target-selector {
          background: var(--bg-panel, #141414);
          border: 1px solid var(--border, #2a2a2a);
          border-radius: 4px;
          padding: 14px;
          font-family: var(--font-mono, 'IBM Plex Mono', monospace);
          font-size: 12px;
        }
        
        .header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 14px;
          padding-bottom: 10px;
          border-bottom: 1px solid var(--border, #2a2a2a);
        }
        
        .title { 
          font-size: 11px;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 1px;
          color: var(--accent, #e49b0f);
        }
        
        .add-btn {
          background: transparent;
          border: 1px solid var(--border-strong, #3a3a3a);
          color: var(--text-secondary, #a0a0a0);
          padding: 5px 12px;
          border-radius: 3px;
          cursor: pointer;
          font-family: inherit;
          font-size: 11px;
          transition: all 0.15s;
        }
        
        .add-btn:hover {
          border-color: var(--accent, #e49b0f);
          color: var(--accent, #e49b0f);
        }
        
        .add-form {
          display: flex;
          flex-direction: column;
          gap: 10px;
          margin-bottom: 14px;
          padding: 14px;
          background: var(--bg-surface, #1a1a1a);
          border: 1px solid var(--border, #2a2a2a);
          border-radius: 4px;
        }
        
        .add-form input, .add-form select {
          padding: 8px 12px;
          background: var(--bg-input, #0f0f0f);
          border: 1px solid var(--border, #2a2a2a);
          border-radius: 3px;
          color: var(--text-primary, #e8e8e8);
          font-family: inherit;
          font-size: 11px;
        }
        
        .add-form input:focus, .add-form select:focus {
          border-color: var(--accent, #e49b0f);
          outline: none;
        }
        
        .form-btns {
          display: flex;
          gap: 10px;
        }
        
        .form-btns button {
          flex: 1;
          padding: 8px;
          border: none;
          border-radius: 3px;
          font-family: inherit;
          font-size: 11px;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 0.5px;
          cursor: pointer;
          transition: all 0.15s;
        }
        
        .form-btns button:first-child { 
          background: var(--status-success, #32cd32);
          color: #000;
        }
        
        .form-btns button:first-child:hover {
          box-shadow: 0 0 10px rgba(50, 205, 50, 0.4);
        }
        
        .form-btns button.cancel { 
          background: var(--bg-elevated, #222);
          color: var(--text-secondary, #a0a0a0);
          border: 1px solid var(--border, #2a2a2a);
        }
        
        .targets-list {
          display: flex;
          flex-direction: column;
          gap: 6px;
        }
        
        .target-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 10px 12px;
          background: var(--bg-surface, #1a1a1a);
          border-radius: 3px;
          border-left: 3px solid var(--status-success, #32cd32);
          transition: all 0.15s;
        }
        
        .target-item:hover {
          background: var(--bg-elevated, #222);
        }
        
        .target-item.disabled {
          border-left-color: var(--text-dim, #606060);
          opacity: 0.6;
        }
        
        .target-info {
          display: flex;
          flex-direction: column;
          gap: 3px;
        }
        
        .name { 
          font-weight: 600;
          color: var(--text-primary, #e8e8e8);
        }
        .mcu { 
          font-size: 10px; 
          color: var(--status-info, #5dade2);
        }
        
        .target-actions {
          display: flex;
          align-items: center;
          gap: 10px;
        }
        
        .output { 
          font-size: 10px; 
          color: var(--text-dim, #606060);
        }
        
        .remove {
          background: transparent;
          border: none;
          color: var(--text-dim, #606060);
          font-size: 16px;
          cursor: pointer;
          padding: 2px 6px;
          border-radius: 3px;
          transition: all 0.15s;
        }
        
        .remove:hover { 
          color: var(--status-error, #dc3545);
          background: rgba(220, 53, 69, 0.1);
        }
        
        .empty {
          text-align: center;
          color: var(--text-dim, #606060);
          padding: 20px;
          font-size: 11px;
          text-transform: uppercase;
          letter-spacing: 0.5px;
        }
        
        .build-all-btn {
          width: 100%;
          margin-top: 14px;
          padding: 12px;
          background: var(--accent, #e49b0f);
          color: #000;
          border: none;
          border-radius: 3px;
          font-family: inherit;
          font-size: 11px;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 0.5px;
          cursor: pointer;
          transition: all 0.15s;
        }
        
        .build-all-btn:hover {
          box-shadow: 0 0 12px var(--accent-glow, rgba(228, 155, 15, 0.4));
        }
        
        .build-all-btn:disabled {
          background: var(--border-strong, #3a3a3a);
          color: var(--text-dim, #606060);
          cursor: not-allowed;
          box-shadow: none;
        }
      `}</style>
    </div>
  );
}
