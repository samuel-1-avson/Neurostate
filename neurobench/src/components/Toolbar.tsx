// Toolbar Component - Main action toolbar below menubar
import { Show } from "solid-js";
<<<<<<< HEAD
=======
import { Icons } from "./AppIcons";
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b

interface ToolbarProps {
  // File actions
  onNew?: () => void;
  onOpen?: () => void;
  onSave?: () => void;
  
  // Edit actions
  onUndo?: () => void;
  onRedo?: () => void;
  canUndo?: boolean;
  canRedo?: boolean;
  
  // Simulation
  simStatus?: "idle" | "running" | "paused";
  onPlay?: () => void;
  onPause?: () => void;
  onStep?: () => void;
  onStop?: () => void;
  
  // Build
  onBuild?: () => void;
  onFlash?: () => void;
  isBuilding?: boolean;
  
  // Code generation
  onGenerate?: () => void;
  onAIMagic?: () => void;
  
  // Canvas
  onZoomIn?: () => void;
  onZoomOut?: () => void;
  onFitView?: () => void;
  onAutoLayout?: () => void;
  
  // View toggles
  showGrid?: boolean;
  onToggleGrid?: () => void;
  showMinimap?: boolean;
  onToggleMinimap?: () => void;
}

export function Toolbar(props: ToolbarProps) {
  return (
    <div class="toolbar">
      {/* File Group */}
      <div class="toolbar-group">
        <button class="toolbar-icon-btn" onClick={props.onNew} title="New Project (Ctrl+N)">
<<<<<<< HEAD
          📄
        </button>
        <button class="toolbar-icon-btn" onClick={props.onOpen} title="Open Project (Ctrl+O)">
          📂
        </button>
        <button class="toolbar-icon-btn" onClick={props.onSave} title="Save Project (Ctrl+S)">
          💾
=======
          <span class="btn-icon">{Icons.newFile()}</span>
        </button>
        <button class="toolbar-icon-btn" onClick={props.onOpen} title="Open Project (Ctrl+O)">
          <span class="btn-icon">{Icons.folder()}</span>
        </button>
        <button class="toolbar-icon-btn" onClick={props.onSave} title="Save Project (Ctrl+S)">
          <span class="btn-icon">{Icons.save()}</span>
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
        </button>
      </div>

      <div class="toolbar-separator" />

      {/* Edit Group */}
      <div class="toolbar-group">
        <button 
          class={`toolbar-icon-btn ${!props.canUndo ? "disabled" : ""}`} 
          onClick={props.onUndo} 
          title="Undo (Ctrl+Z)"
          disabled={!props.canUndo}
        >
<<<<<<< HEAD
          ↩️
=======
          <span class="btn-icon">{Icons.reset()}</span>
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
        </button>
        <button 
          class={`toolbar-icon-btn ${!props.canRedo ? "disabled" : ""}`} 
          onClick={props.onRedo} 
          title="Redo (Ctrl+Y)"
          disabled={!props.canRedo}
        >
<<<<<<< HEAD
          ↪️
=======
          <span class="btn-icon">{Icons.refresh()}</span>
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
        </button>
      </div>

      <div class="toolbar-separator" />

      {/* Simulation Group */}
      <div class="toolbar-group">
        <Show when={props.simStatus !== "running"} fallback={
          <button class="toolbar-icon-btn active" onClick={props.onPause} title="Pause Simulation">
<<<<<<< HEAD
            ⏸️
          </button>
        }>
          <button class="toolbar-icon-btn success" onClick={props.onPlay} title="Run Simulation (F5)">
            ▶️
          </button>
        </Show>
        <button class="toolbar-icon-btn" onClick={props.onStep} title="Step (F10)">
          ⏭️
        </button>
        <button class="toolbar-icon-btn danger" onClick={props.onStop} title="Stop (Shift+F5)">
          ⏹️
=======
            <span class="btn-icon">{Icons.halt()}</span>
          </button>
        }>
          <button class="toolbar-icon-btn success" onClick={props.onPlay} title="Run Simulation (F5)">
            <span class="btn-icon">{Icons.resume()}</span>
          </button>
        </Show>
        <button class="toolbar-icon-btn" onClick={props.onStep} title="Step (F10)">
          <span class="btn-icon">{Icons.step()}</span>
        </button>
        <button class="toolbar-icon-btn danger" onClick={props.onStop} title="Stop (Shift+F5)">
          <span class="btn-icon">{Icons.stop()}</span>
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
        </button>
      </div>

      <div class="toolbar-separator" />

      {/* Build Group */}
      <div class="toolbar-group">
        <button 
          class={`toolbar-icon-btn ${props.isBuilding ? "building" : ""}`} 
          onClick={props.onBuild} 
          title="Build Project (Ctrl+B)"
          disabled={props.isBuilding}
        >
<<<<<<< HEAD
          🔨
        </button>
        <button class="toolbar-icon-btn" onClick={props.onFlash} title="Flash to Device (Ctrl+F)">
          ⚡
=======
          <span class="btn-icon">{Icons.build()}</span>
        </button>
        <button class="toolbar-icon-btn" onClick={props.onFlash} title="Flash to Device (Ctrl+F)">
          <span class="btn-icon">{Icons.flash()}</span>
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
        </button>
      </div>

      <div class="toolbar-separator" />

      {/* Code Generation Group */}
      <div class="toolbar-group">
        <button class="toolbar-icon-btn" onClick={props.onGenerate} title="Generate Code (Ctrl+G)">
<<<<<<< HEAD
          📝
        </button>
        <button class="toolbar-icon-btn magic" onClick={props.onAIMagic} title="AI Magic - Generate FSM from Description">
          🪄
=======
          <span class="btn-icon">{Icons.code()}</span>
        </button>
        <button class="toolbar-icon-btn magic" onClick={props.onAIMagic} title="AI Magic - Generate FSM from Description">
          <span class="btn-icon">{Icons.brain()}</span>
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
        </button>
      </div>

      <div class="toolbar-separator" />

      {/* Canvas Tools Group */}
      <div class="toolbar-group">
        <button class="toolbar-icon-btn" onClick={props.onZoomIn} title="Zoom In (Ctrl++)">
<<<<<<< HEAD
          🔍+
        </button>
        <button class="toolbar-icon-btn" onClick={props.onZoomOut} title="Zoom Out (Ctrl+-)">
          🔍-
        </button>
        <button class="toolbar-icon-btn" onClick={props.onFitView} title="Fit View (Ctrl+0)">
          ⛶
        </button>
        <button class="toolbar-icon-btn" onClick={props.onAutoLayout} title="Auto Layout">
          📐
=======
          <span class="btn-icon">{Icons.zoomIn()}</span>
        </button>
        <button class="toolbar-icon-btn" onClick={props.onZoomOut} title="Zoom Out (Ctrl+-)">
          <span class="btn-icon">{Icons.zoomOut()}</span>
        </button>
        <button class="toolbar-icon-btn" onClick={props.onFitView} title="Fit View (Ctrl+0)">
          <span class="btn-icon">{Icons.fitView()}</span>
        </button>
        <button class="toolbar-icon-btn" onClick={props.onAutoLayout} title="Auto Layout">
          <span class="btn-icon">{Icons.layout()}</span>
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
        </button>
      </div>

      <div class="toolbar-separator" />

      {/* View Toggles */}
      <div class="toolbar-group">
        <button 
          class={`toolbar-icon-btn toggle ${props.showGrid ? "active" : ""}`} 
          onClick={props.onToggleGrid} 
          title="Toggle Grid"
        >
<<<<<<< HEAD
          #
=======
          <span class="btn-icon">{Icons.grid()}</span>
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
        </button>
        <button 
          class={`toolbar-icon-btn toggle ${props.showMinimap ? "active" : ""}`} 
          onClick={props.onToggleMinimap} 
          title="Toggle Minimap"
        >
<<<<<<< HEAD
          🗺️
=======
          <span class="btn-icon">{Icons.minimap()}</span>
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
        </button>
      </div>

      <div class="toolbar-spacer" />

      {/* Quick Actions (right side) */}
      <div class="toolbar-group">
        <button class="toolbar-text-btn" onClick={props.onAIMagic}>
<<<<<<< HEAD
          ✨ AI Assistant
=======
          <span class="btn-icon">{Icons.brain()}</span> AI Assistant
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
        </button>
      </div>
    </div>
  );
}

export default Toolbar;
