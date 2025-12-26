import { createSignal, onMount, onCleanup, For, Show, createMemo } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

// Types matching Rust backend
interface DebugStatus {
  state: "stopped" | "running" | "halted" | "disconnected";
  chip: string;
  elf_path: string | null;
  current_address: number | null;
  current_file: string | null;
  current_line: number | null;
  breakpoints_hit: number[];
}

interface Breakpoint {
  id: number;
  file: string;
  line: number;
  address: number | null;
  enabled: boolean;
  hit_count: number;
  condition: string | null;
}

interface StackFrame {
  index: number;
  name: string;
  file: string | null;
  line: number | null;
  address: number;
}

interface Registers {
  pc: number;
  sp: number;
  lr: number;
  general: Record<string, number>;
}

interface Props {
  chip?: string;
  elfPath?: string;
}

export default function DebugPanel(props: Props) {
  const [status, setStatus] = createSignal<DebugStatus | null>(null);
  const [breakpoints, setBreakpoints] = createSignal<Breakpoint[]>([]);
  const [stack, setStack] = createSignal<StackFrame[]>([]);
  const [registers, setRegisters] = createSignal<Registers | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [loading, setLoading] = createSignal(false);
  
  // Breakpoint form
  const [bpFile, setBpFile] = createSignal("");
  const [bpLine, setBpLine] = createSignal("");
  
  let pollInterval: number | undefined;

  const isConnected = createMemo(() => 
    status() !== null && status()?.state !== "disconnected" && status()?.state !== "stopped"
  );

  const isHalted = createMemo(() => status()?.state === "halted");
  const isRunning = createMemo(() => status()?.state === "running");

  const refreshStatus = async () => {
    try {
      const s = await invoke<DebugStatus>("debug_status");
      setStatus(s);
      
      if (s.state === "halted") {
        // Auto-refresh stack and registers when halted
        await refreshStack();
        await refreshRegisters();
      }
    } catch {
      // No active session
      setStatus(null);
    }
  };

  const refreshBreakpoints = async () => {
    try {
      const bps = await invoke<Breakpoint[]>("debug_breakpoint_list");
      setBreakpoints(bps);
    } catch {
      setBreakpoints([]);
    }
  };

  const refreshStack = async () => {
    try {
      const s = await invoke<StackFrame[]>("debug_stack");
      setStack(s);
    } catch {
      setStack([]);
    }
  };

  const refreshRegisters = async () => {
    try {
      const r = await invoke<Registers>("debug_registers");
      setRegisters(r);
    } catch {
      setRegisters(null);
    }
  };

  const startSession = async () => {
    setLoading(true);
    try {
      const s = await invoke<DebugStatus>("debug_start", {
        chip: props.chip || "STM32F407VGTx",
        elfPath: props.elfPath,
      });
      setStatus(s);
      setError(null);
      await refreshBreakpoints();
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const stopSession = async () => {
    try {
      await invoke("debug_stop");
      setStatus(null);
      setStack([]);
      setRegisters(null);
    } catch (err) {
      setError(String(err));
    }
  };

  const resume = async () => {
    try {
      const s = await invoke<DebugStatus>("debug_resume");
      setStatus(s);
    } catch (err) {
      setError(String(err));
    }
  };

  const pause = async () => {
    try {
      const s = await invoke<DebugStatus>("debug_pause");
      setStatus(s);
    } catch (err) {
      setError(String(err));
    }
  };

  const step = async (type: string) => {
    try {
      const s = await invoke<DebugStatus>("debug_step", { stepType: type });
      setStatus(s);
      await refreshStack();
      await refreshRegisters();
    } catch (err) {
      setError(String(err));
    }
  };

  const addBreakpoint = async () => {
    if (!bpFile() || !bpLine()) return;
    
    try {
      await invoke("debug_breakpoint_add", {
        file: bpFile(),
        line: parseInt(bpLine()),
      });
      setBpFile("");
      setBpLine("");
      await refreshBreakpoints();
    } catch (err) {
      setError(String(err));
    }
  };

  const removeBreakpoint = async (id: number) => {
    try {
      await invoke("debug_breakpoint_remove", { id });
      await refreshBreakpoints();
    } catch (err) {
      setError(String(err));
    }
  };

  const formatAddress = (addr: number) => `0x${addr.toString(16).padStart(8, '0')}`;

  onMount(() => {
    refreshStatus();
    pollInterval = setInterval(refreshStatus, 1000) as unknown as number;
  });

  onCleanup(() => {
    if (pollInterval) clearInterval(pollInterval);
  });

  return (
    <div class="debug-panel">
      <div class="toolbar">
        <Show when={!isConnected()} fallback={
          <>
            <button class="stop-btn" onClick={stopSession}>⏹ Stop</button>
            <div class="sep" />
            <Show when={isHalted()}>
              <button class="control-btn" onClick={resume}>▶ Continue</button>
              <button class="control-btn" onClick={() => step("into")}>↓ Step In</button>
              <button class="control-btn" onClick={() => step("over")}>→ Step Over</button>
              <button class="control-btn" onClick={() => step("out")}>↑ Step Out</button>
            </Show>
            <Show when={isRunning()}>
              <button class="control-btn pause" onClick={pause}>⏸ Pause</button>
            </Show>
          </>
        }>
          <button class="start-btn" onClick={startSession} disabled={loading()}>
            {loading() ? 'Starting...' : '🐛 Start Debug'}
          </button>
        </Show>
        
        <div class="status-indicator">
          <span class={`dot ${status()?.state || 'disconnected'}`} />
          <span class="state">{status()?.state || 'disconnected'}</span>
        </div>
      </div>
      
      <Show when={error()}>
        <div class="error">{error()}</div>
      </Show>
      
      <Show when={isConnected()}>
        <div class="panels">
          {/* Breakpoints */}
          <div class="panel breakpoints">
            <div class="panel-header">Breakpoints</div>
            <div class="bp-form">
              <input 
                type="text" 
                placeholder="file.c" 
                value={bpFile()} 
                onInput={(e) => setBpFile(e.currentTarget.value)}
              />
              <input 
                type="number" 
                placeholder="line" 
                value={bpLine()} 
                onInput={(e) => setBpLine(e.currentTarget.value)}
              />
              <button onClick={addBreakpoint}>+</button>
            </div>
            <div class="bp-list">
              <For each={breakpoints()}>
                {(bp) => (
                  <div class="bp-item">
                    <span class="bp-loc">{bp.file}:{bp.line}</span>
                    <span class="bp-hits">{bp.hit_count}×</span>
                    <button onClick={() => removeBreakpoint(bp.id)}>×</button>
                  </div>
                )}
              </For>
            </div>
          </div>
          
          {/* Call Stack */}
          <div class="panel stack">
            <div class="panel-header">Call Stack</div>
            <div class="stack-list">
              <For each={stack()}>
                {(frame) => (
                  <div class={`stack-frame ${frame.index === 0 ? 'current' : ''}`}>
                    <span class="frame-name">{frame.name}</span>
                    <span class="frame-loc">
                      {frame.file ? `${frame.file}:${frame.line}` : formatAddress(frame.address)}
                    </span>
                  </div>
                )}
              </For>
            </div>
          </div>
          
          {/* Registers */}
          <div class="panel registers">
            <div class="panel-header">Registers</div>
            <Show when={registers()}>
              <div class="reg-list">
                <div class="reg-item special">
                  <span class="reg-name">PC</span>
                  <span class="reg-val">{formatAddress(registers()!.pc)}</span>
                </div>
                <div class="reg-item special">
                  <span class="reg-name">SP</span>
                  <span class="reg-val">{formatAddress(registers()!.sp)}</span>
                </div>
                <div class="reg-item special">
                  <span class="reg-name">LR</span>
                  <span class="reg-val">{formatAddress(registers()!.lr)}</span>
                </div>
              </div>
            </Show>
          </div>
        </div>
        
        {/* Current location */}
        <Show when={status()?.current_file}>
          <div class="current-loc">
            <span class="file">{status()?.current_file}</span>
            <span class="line">:{status()?.current_line}</span>
            <span class="addr">{formatAddress(status()?.current_address || 0)}</span>
          </div>
        </Show>
      </Show>

      <style>{`
        .debug-panel {
          display: flex;
          flex-direction: column;
          height: 100%;
          background: var(--bg-base, #0d0d0d);
          color: var(--text-primary, #e8e8e8);
          font-family: var(--font-mono, 'IBM Plex Mono', monospace);
          font-size: 12px;
        }
        
        .toolbar {
          display: flex;
          align-items: center;
          gap: 10px;
          padding: 12px 16px;
          background: var(--bg-panel, #141414);
          border-bottom: 1px solid var(--border, #2a2a2a);
        }
        
        .start-btn {
          background: var(--status-success, #32cd32);
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
        
        .start-btn:hover {
          box-shadow: 0 0 12px rgba(50, 205, 50, 0.4);
        }
        
        .stop-btn {
          background: var(--status-error, #dc3545);
          color: white;
          border: none;
          padding: 8px 18px;
          border-radius: 3px;
          font-family: inherit;
          font-size: 11px;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 0.5px;
          cursor: pointer;
        }
        
        .control-btn {
          background: var(--bg-surface, #1a1a1a);
          color: var(--text-primary, #e8e8e8);
          border: 1px solid var(--border-strong, #3a3a3a);
          padding: 6px 14px;
          border-radius: 3px;
          font-family: inherit;
          font-size: 11px;
          cursor: pointer;
          transition: all 0.15s;
        }
        
        .control-btn:hover { 
          border-color: var(--accent, #e49b0f);
          background: var(--bg-elevated, #222);
        }
        
        .control-btn.pause { 
          color: var(--accent, #e49b0f);
          border-color: var(--accent, #e49b0f);
        }
        
        .sep { 
          width: 1px; 
          height: 24px; 
          background: var(--border-strong, #3a3a3a);
        }
        
        .status-indicator {
          margin-left: auto;
          display: flex;
          align-items: center;
          gap: 8px;
          font-size: 11px;
          text-transform: uppercase;
          letter-spacing: 0.5px;
          color: var(--text-dim, #606060);
        }
        
        .dot {
          width: 10px;
          height: 10px;
          border-radius: 50%;
          background: var(--text-dim, #606060);
        }
        
        .dot.halted { 
          background: var(--accent, #e49b0f);
          box-shadow: 0 0 8px rgba(228, 155, 15, 0.5);
        }
        .dot.running { 
          background: var(--status-success, #32cd32);
          box-shadow: 0 0 8px rgba(50, 205, 50, 0.5);
          animation: led-pulse 1s infinite;
        }
        .dot.stopped, .dot.disconnected { 
          background: var(--text-dim, #606060);
        }
        
        @keyframes led-pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.4; }
        }
        
        .panels {
          display: flex;
          flex: 1;
          overflow: hidden;
        }
        
        .panel {
          flex: 1;
          border-right: 1px solid var(--border, #2a2a2a);
          display: flex;
          flex-direction: column;
          background: var(--bg-panel, #141414);
        }
        
        .panel:last-child { border-right: none; }
        
        .panel-header {
          padding: 10px 14px;
          background: var(--bg-surface, #1a1a1a);
          border-bottom: 1px solid var(--border, #2a2a2a);
          font-size: 10px;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 1px;
          color: var(--text-dim, #606060);
        }
        
        .bp-form {
          display: flex;
          gap: 6px;
          padding: 10px;
          background: var(--bg-surface, #1a1a1a);
        }
        
        .bp-form input {
          flex: 1;
          padding: 8px 10px;
          background: var(--bg-input, #0f0f0f);
          border: 1px solid var(--border, #2a2a2a);
          border-radius: 3px;
          color: var(--text-primary, #e8e8e8);
          font-family: inherit;
          font-size: 11px;
          min-width: 0;
        }
        
        .bp-form input:focus {
          border-color: var(--accent, #e49b0f);
          outline: none;
        }
        
        .bp-form input[type="number"] { width: 60px; flex: none; }
        
        .bp-form button {
          background: var(--accent, #e49b0f);
          color: #000;
          border: none;
          width: 32px;
          border-radius: 3px;
          font-weight: bold;
          cursor: pointer;
        }
        
        .bp-list, .stack-list, .reg-list {
          flex: 1;
          overflow-y: auto;
          padding: 8px;
        }
        
        .bp-item {
          display: flex;
          align-items: center;
          gap: 10px;
          padding: 8px 10px;
          border-radius: 3px;
          border-left: 3px solid var(--status-error, #dc3545);
          background: var(--bg-surface, #1a1a1a);
          margin-bottom: 4px;
        }
        
        .bp-item:hover { 
          background: var(--bg-elevated, #222);
        }
        
        .bp-loc { 
          flex: 1; 
          font-size: 11px;
        }
        .bp-hits { 
          color: var(--text-dim, #606060);
          font-size: 10px;
        }
        
        .bp-item button {
          background: transparent;
          border: none;
          color: var(--text-dim, #606060);
          cursor: pointer;
          font-size: 14px;
        }
        
        .bp-item button:hover { 
          color: var(--status-error, #dc3545);
        }
        
        .stack-frame {
          padding: 10px 12px;
          border-radius: 3px;
          margin-bottom: 4px;
          background: var(--bg-surface, #1a1a1a);
          border-left: 3px solid transparent;
        }
        
        .stack-frame.current { 
          border-left-color: var(--accent, #e49b0f);
          background: rgba(228, 155, 15, 0.05);
        }
        
        .frame-name { 
          display: block; 
          font-weight: 600;
          font-size: 12px;
        }
        .frame-loc { 
          display: block; 
          font-size: 10px; 
          color: var(--text-dim, #606060);
          margin-top: 2px;
        }
        
        .reg-item {
          display: flex;
          justify-content: space-between;
          padding: 6px 10px;
          border-radius: 3px;
          margin-bottom: 2px;
        }
        
        .reg-item.special { 
          background: var(--bg-surface, #1a1a1a);
        }
        .reg-name { 
          font-weight: 600;
          color: var(--text-secondary, #a0a0a0);
        }
        .reg-val { 
          color: var(--accent, #e49b0f);
        }
        
        .current-loc {
          padding: 10px 16px;
          background: var(--bg-surface, #1a1a1a);
          border-top: 1px solid var(--border, #2a2a2a);
          font-size: 11px;
        }
        
        .file { color: var(--status-info, #5dade2); }
        .line { color: var(--accent, #e49b0f); }
        .addr { 
          margin-left: 16px; 
          color: var(--text-dim, #606060);
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
