import { createSignal, onMount, onCleanup, For, Show, createMemo } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { Icons } from "./AppIcons";
import "./DebugPanel.css";

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
            <button class="stop-btn" onClick={stopSession}>{Icons.stop()} Stop</button>
            <div class="sep" />
            <Show when={isHalted()}>
              <button class="control-btn" onClick={resume}>{Icons.resume()} Continue</button>
              <button class="control-btn" onClick={() => step("into")}>{Icons.stepInto()} Step In</button>
              <button class="control-btn" onClick={() => step("over")}>{Icons.stepOver()} Step Over</button>
              <button class="control-btn" onClick={() => step("out")}>{Icons.stepOut()} Step Out</button>
            </Show>
            <Show when={isRunning()}>
              <button class="control-btn pause" onClick={pause}>{Icons.pause()} Pause</button>
            </Show>
          </>
        }>
          <button class="start-btn" onClick={startSession} disabled={loading()}>
            {loading() ? 'Starting...' : <>{Icons.debug()} Start Debug</>}
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
                    <button onClick={() => removeBreakpoint(bp.id)}>{Icons.x()}</button>
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


    </div>
  );
}
