import { createSignal, For, Show, onMount, onCleanup, createEffect } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./SimulationDashboard.css";
import { Icons } from "./AppIcons";

interface McuInfo {
  id: string;
  name: string;
  family: string;
  arch: string;
}

interface SimulationState {
  state: string;
  cycles: number;
  pc: number;
}

interface RegisterState {
  registers: Record<string, number>;
}

interface GpioState {
  port: string;
  pins: number[];
}

interface SimulationDashboardProps {
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function SimulationDashboard(props: SimulationDashboardProps) {
  // MCU selection
  const [supportedMcus, setSupportedMcus] = createSignal<McuInfo[]>([]);
  const [selectedMcu, setSelectedMcu] = createSignal<string>("STM32F407");
  const [simulationCreated, setSimulationCreated] = createSignal(false);
  
  // Simulation state
  const [simState, setSimState] = createSignal<string>("Idle");
  const [cycles, setCycles] = createSignal<number>(0);
  const [pc, setPc] = createSignal<number>(0);
  const [isRunning, setIsRunning] = createSignal(false);
  
  // CPU Registers
  const [registers, setRegisters] = createSignal<Record<string, number>>({});
  
  // GPIO State (16 pins per port)
  const [gpioA, setGpioA] = createSignal<boolean[]>(Array(16).fill(false));
  const [gpioB, setGpioB] = createSignal<boolean[]>(Array(16).fill(false));
  
  // Memory viewer
  const [memoryAddress, setMemoryAddress] = createSignal<number>(0x20000000);
  const [memoryData, setMemoryData] = createSignal<number[]>([]);
  
  // Status
  const [statusMessage, setStatusMessage] = createSignal("");
  const [isLoading, setIsLoading] = createSignal(false);
  
  // Polling interval
  let pollInterval: number | undefined;

  const addLog = (source: string, message: string, type: "info" | "success" | "warning" | "error" = "info") => {
    props.onLog?.(source, message, type);
    setStatusMessage(message);
  };

  // Load supported MCUs
  const loadMcus = async () => {
    try {
      const result = await invoke("simulation_get_supported_mcus") as McuInfo[];
      setSupportedMcus(result);
      addLog("Simulator", `Loaded ${result.length} MCU configurations`, "success");
    } catch (e) {
      addLog("Simulator", `Failed to load MCUs: ${e}`, "error");
    }
  };

  // Create simulation
  const createSimulation = async () => {
    setIsLoading(true);
    try {
      const result = await invoke("simulation_create", { mcu: selectedMcu() }) as any;
      if (result.success) {
        setSimulationCreated(true);
        setSimState("Idle");
        setCycles(0);
        setPc(0);
        addLog("Simulator", `Created ${selectedMcu()} simulation`, "success");
        await refreshRegisters();
      }
    } catch (e) {
      addLog("Simulator", `Failed to create simulation: ${e}`, "error");
    }
    setIsLoading(false);
  };

  // Start simulation
  const startSimulation = async () => {
    try {
      await invoke("simulation_start");
      setIsRunning(true);
      setSimState("Running");
      addLog("Simulator", "Simulation started", "success");
      startPolling();
    } catch (e) {
      addLog("Simulator", `Failed to start: ${e}`, "error");
    }
  };

  // Stop simulation
  const stopSimulation = async () => {
    try {
      await invoke("simulation_stop");
      setIsRunning(false);
      setSimState("Stopped");
      addLog("Simulator", "Simulation stopped", "info");
      stopPolling();
    } catch (e) {
      addLog("Simulator", `Failed to stop: ${e}`, "error");
    }
  };

  // Pause simulation
  const pauseSimulation = async () => {
    try {
      await invoke("simulation_pause");
      setIsRunning(false);
      setSimState("Paused");
      addLog("Simulator", "Simulation paused", "info");
      stopPolling();
    } catch (e) {
      addLog("Simulator", `Failed to pause: ${e}`, "error");
    }
  };

  // Step single instruction
  const stepSimulation = async () => {
    try {
      const result = await invoke("simulation_step") as any;
      setCycles(result.cycles);
      await refreshState();
      addLog("Simulator", `Step: ${result.result}`, "info");
    } catch (e) {
      addLog("Simulator", `Step failed: ${e}`, "error");
    }
  };

  // Reset simulation
  const resetSimulation = async () => {
    try {
      await invoke("simulation_reset");
      setIsRunning(false);
      setSimState("Idle");
      setCycles(0);
      setPc(0);
      await refreshRegisters();
      addLog("Simulator", "Simulation reset", "info");
    } catch (e) {
      addLog("Simulator", `Reset failed: ${e}`, "error");
    }
  };

  // Refresh state
  const refreshState = async () => {
    try {
      const result = await invoke("simulation_get_state") as SimulationState;
      setSimState(result.state);
      setCycles(result.cycles);
      setPc(result.pc);
    } catch (e) {
      // Ignore polling errors
    }
  };

  // Refresh registers
  const refreshRegisters = async () => {
    try {
      const result = await invoke("simulation_get_registers") as RegisterState;
      setRegisters(result.registers);
    } catch (e) {
      // Ignore
    }
  };

  // Read memory
  const readMemory = async () => {
    try {
      const result = await invoke("simulation_get_memory", { 
        address: memoryAddress(), 
        size: 64 
      }) as any;
      setMemoryData(result.data);
    } catch (e) {
      addLog("Simulator", `Memory read failed: ${e}`, "error");
    }
  };

  // Inject GPIO
  const toggleGpioPin = async (port: string, pin: number) => {
    const currentState = port === 'A' ? gpioA() : gpioB();
    const newState = !currentState[pin];
    
    try {
      await invoke("simulation_inject_gpio", { port, pin, state: newState });
      
      if (port === 'A') {
        const updated = [...gpioA()];
        updated[pin] = newState;
        setGpioA(updated);
      } else {
        const updated = [...gpioB()];
        updated[pin] = newState;
        setGpioB(updated);
      }
      
      addLog("Simulator", `GPIO ${port}${pin} = ${newState ? '1' : '0'}`, "info");
    } catch (e) {
      addLog("Simulator", `GPIO inject failed: ${e}`, "error");
    }
  };

  // Polling for state updates
  const startPolling = () => {
    pollInterval = window.setInterval(async () => {
      await refreshState();
      await refreshRegisters();
    }, 100);
  };

  const stopPolling = () => {
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = undefined;
    }
  };

  onMount(() => {
    loadMcus();
  });

  onCleanup(() => {
    stopPolling();
  });

  // Group MCUs by family
  const mcusByFamily = () => {
    const grouped: Record<string, McuInfo[]> = {};
    for (const mcu of supportedMcus()) {
      if (!grouped[mcu.family]) {
        grouped[mcu.family] = [];
      }
      grouped[mcu.family].push(mcu);
    }
    return grouped;
  };

  return (
    <div class="simulation-dashboard">
      {/* Header */}
      <div class="dashboard-header">
        <h3><span class="header-icon">{Icons.flash()}</span> Simulation Engine</h3>
        <div class="status-indicator" classList={{ running: isRunning(), idle: !isRunning() }}>
          {simState()}
        </div>
      </div>

      {/* MCU Selection */}
      <Show when={!simulationCreated()}>
        <div class="mcu-selector-section">
          <h4>Select MCU</h4>
          <div class="mcu-grid">
            <For each={Object.entries(mcusByFamily())}>
              {([family, mcus]) => (
                <div class="mcu-family">
                  <div class="family-name">{family}</div>
                  <div class="mcu-chips">
                    <For each={mcus}>
                      {(mcu) => (
                        <button
                          class={`mcu-chip ${selectedMcu() === mcu.id ? 'selected' : ''}`}
                          onClick={() => setSelectedMcu(mcu.id)}
                        >
                          <span class="mcu-id">{mcu.id}</span>
                          <span class="mcu-arch">{mcu.arch}</span>
                        </button>
                      )}
                    </For>
                  </div>
                </div>
              )}
            </For>
          </div>
          <button 
            class="create-sim-btn" 
            onClick={createSimulation}
            disabled={isLoading()}
          >
            {isLoading() ? "Creating..." : `🚀 Create ${selectedMcu()} Simulation`}
          </button>
        </div>
      </Show>

      {/* Simulation Dashboard */}
      <Show when={simulationCreated()}>
        {/* Controls */}
        <div class="controls-section">
          <button class="ctrl-btn start" onClick={startSimulation} disabled={isRunning()}>
            {Icons.play()} Start
          </button>
          <button class="ctrl-btn pause" onClick={pauseSimulation} disabled={!isRunning()}>
            {Icons.pause()} Pause
          </button>
          <button class="ctrl-btn stop" onClick={stopSimulation}>
            {Icons.stop()} Stop
          </button>
          <button class="ctrl-btn step" onClick={stepSimulation} disabled={isRunning()}>
            {Icons.step()} Step
          </button>
          <button class="ctrl-btn reset" onClick={resetSimulation}>
            {Icons.refresh()} Reset
          </button>
        </div>

        {/* Stats Row */}
        <div class="stats-row">
          <div class="stat-card">
            <span class="stat-label">Cycles</span>
            <span class="stat-value">{cycles().toLocaleString()}</span>
          </div>
          <div class="stat-card">
            <span class="stat-label">PC</span>
            <span class="stat-value mono">0x{pc().toString(16).toUpperCase().padStart(8, '0')}</span>
          </div>
          <div class="stat-card">
            <span class="stat-label">MCU</span>
            <span class="stat-value">{selectedMcu()}</span>
          </div>
        </div>

        {/* Two Column Layout */}
        <div class="dashboard-grid">
          {/* Registers */}
          <div class="panel registers-panel">
            <h4>CPU Registers</h4>
            <div class="register-grid">
              <For each={['r0', 'r1', 'r2', 'r3', 'r4', 'r5', 'r6', 'r7']}>
                {(reg) => (
                  <div class="register-row">
                    <span class="reg-name">{reg.toUpperCase()}</span>
                    <span class="reg-value">
                      {(registers()[reg] ?? 0).toString(16).toUpperCase().padStart(8, '0')}
                    </span>
                  </div>
                )}
              </For>
              <For each={['r8', 'r9', 'r10', 'r11', 'r12', 'sp', 'lr', 'pc']}>
                {(reg) => (
                  <div class="register-row">
                    <span class="reg-name">{reg.toUpperCase()}</span>
                    <span class="reg-value">
                      {(registers()[reg] ?? 0).toString(16).toUpperCase().padStart(8, '0')}
                    </span>
                  </div>
                )}
              </For>
            </div>
          </div>

          {/* GPIO */}
          <div class="panel gpio-panel">
            <h4>GPIO Input Injection</h4>
            <div class="gpio-port">
              <span class="port-name">Port A</span>
              <div class="gpio-pins">
                <For each={[0, 1, 2, 3, 4, 5, 6, 7]}>
                  {(pin) => (
                    <button
                      class={`gpio-pin ${gpioA()[pin] ? 'high' : 'low'}`}
                      onClick={() => toggleGpioPin('A', pin)}
                      title={`PA${pin}`}
                    >
                      {pin}
                    </button>
                  )}
                </For>
              </div>
            </div>
            <div class="gpio-port">
              <span class="port-name">Port B</span>
              <div class="gpio-pins">
                <For each={[0, 1, 2, 3, 4, 5, 6, 7]}>
                  {(pin) => (
                    <button
                      class={`gpio-pin ${gpioB()[pin] ? 'high' : 'low'}`}
                      onClick={() => toggleGpioPin('B', pin)}
                      title={`PB${pin}`}
                    >
                      {pin}
                    </button>
                  )}
                </For>
              </div>
            </div>
          </div>
        </div>

        {/* Memory Viewer */}
        <div class="panel memory-panel">
          <div class="memory-header">
            <h4>Memory Viewer</h4>
            <div class="memory-controls">
              <input
                type="text"
                class="address-input"
                value={`0x${memoryAddress().toString(16).toUpperCase()}`}
                onInput={(e) => {
                  const val = parseInt(e.target.value.replace('0x', ''), 16);
                  if (!isNaN(val)) setMemoryAddress(val);
                }}
              />
              <button class="read-btn" onClick={readMemory}>Read</button>
            </div>
          </div>
          <div class="memory-view">
            <Show when={memoryData().length > 0} fallback={<span class="no-data">Press Read to view memory</span>}>
              <div class="hex-dump">
                <For each={Array(4).fill(0).map((_, i) => i * 16)}>
                  {(offset) => (
                    <div class="hex-row">
                      <span class="hex-address">
                        {`${(memoryAddress() + offset).toString(16).toUpperCase().padStart(8, '0')}`}
                      </span>
                      <span class="hex-bytes">
                        <For each={memoryData().slice(offset, offset + 16)}>
                          {(byte) => <span class="hex-byte">{byte.toString(16).toUpperCase().padStart(2, '0')}</span>}
                        </For>
                      </span>
                    </div>
                  )}
                </For>
              </div>
            </Show>
          </div>
        </div>

        {/* Status */}
        <div class="status-bar">
          <span class="status-message">{statusMessage()}</span>
          <button class="new-sim-btn" onClick={() => setSimulationCreated(false)}>
            New Simulation
          </button>
        </div>
      </Show>
    </div>
  );
}
