import { createSignal, For, Show, createEffect } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./PeripheralViewer.css";

interface PeripheralViewerProps {
  simulationActive: boolean;
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function PeripheralViewer(props: PeripheralViewerProps) {
  const [activeTab, setActiveTab] = createSignal<"uart" | "timer" | "adc">("uart");
  
  // UART state
  const [uartData, setUartData] = createSignal<string>("");
  const [uartInstance, setUartInstance] = createSignal<number>(1);
  const [uartInput, setUartInput] = createSignal<string>("");
  
  // Timer state
  const [timerValues, setTimerValues] = createSignal<Record<number, number>>({});
  
  // ADC state
  const [adcChannels, setAdcChannels] = createSignal<number[]>(Array(8).fill(2048));

  const addLog = (source: string, message: string, type: "info" | "success" | "warning" | "error" = "info") => {
    props.onLog?.(source, message, type);
  };

  // Send UART data
  const sendUart = async () => {
    if (!props.simulationActive || !uartInput()) return;
    
    try {
      const data = Array.from(uartInput()).map(c => c.charCodeAt(0));
      await invoke("simulation_inject_uart", { 
        instance: uartInstance(), 
        data 
      });
      setUartData(prev => prev + uartInput());
      addLog("UART", `TX: "${uartInput()}"`, "info");
      setUartInput("");
    } catch (e) {
      addLog("UART", `Send failed: ${e}`, "error");
    }
  };

  // Update ADC channel value
  const updateAdcChannel = (channel: number, value: number) => {
    const channels = [...adcChannels()];
    channels[channel] = value;
    setAdcChannels(channels);
    addLog("ADC", `Channel ${channel} = ${value}`, "info");
  };

  return (
    <div class="peripheral-viewer">
      <div class="viewer-header">
        <h3>📟 Peripheral Monitor</h3>
      </div>

      {/* Tab Navigation */}
      <div class="tab-nav">
        <button 
          class={`tab-btn ${activeTab() === "uart" ? "active" : ""}`}
          onClick={() => setActiveTab("uart")}
        >
          UART
        </button>
        <button 
          class={`tab-btn ${activeTab() === "timer" ? "active" : ""}`}
          onClick={() => setActiveTab("timer")}
        >
          Timers
        </button>
        <button 
          class={`tab-btn ${activeTab() === "adc" ? "active" : ""}`}
          onClick={() => setActiveTab("adc")}
        >
          ADC
        </button>
      </div>

      {/* UART Tab */}
      <Show when={activeTab() === "uart"}>
        <div class="uart-section">
          <div class="uart-header">
            <label>Instance:</label>
            <select 
              value={uartInstance()} 
              onChange={(e) => setUartInstance(parseInt(e.target.value))}
            >
              <For each={[1, 2, 3, 4, 5, 6]}>
                {(n) => <option value={n}>USART{n}</option>}
              </For>
            </select>
          </div>
          
          <div class="uart-terminal">
            <div class="terminal-output">
              <Show when={uartData()} fallback={<span class="empty">No data</span>}>
                <pre>{uartData()}</pre>
              </Show>
            </div>
            <div class="terminal-input">
              <input
                type="text"
                placeholder="Type to send..."
                value={uartInput()}
                onInput={(e) => setUartInput(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && sendUart()}
                disabled={!props.simulationActive}
              />
              <button 
                onClick={sendUart} 
                disabled={!props.simulationActive || !uartInput()}
              >
                Send
              </button>
            </div>
          </div>
        </div>
      </Show>

      {/* Timer Tab */}
      <Show when={activeTab() === "timer"}>
        <div class="timer-section">
          <For each={[1, 2, 3, 4]}>
            {(n) => (
              <div class="timer-card">
                <div class="timer-header">
                  <span class="timer-name">TIM{n}</span>
                  <span class="timer-status">
                    {timerValues()[n] ? "Running" : "Stopped"}
                  </span>
                </div>
                <div class="timer-bar">
                  <div 
                    class="timer-fill" 
                    style={{ width: `${((timerValues()[n] ?? 0) / 65535) * 100}%` }}
                  />
                </div>
                <div class="timer-value">
                  CNT: {(timerValues()[n] ?? 0).toString(16).toUpperCase().padStart(4, '0')}
                </div>
              </div>
            )}
          </For>
        </div>
      </Show>

      {/* ADC Tab */}
      <Show when={activeTab() === "adc"}>
        <div class="adc-section">
          <For each={[0, 1, 2, 3, 4, 5, 6, 7]}>
            {(ch) => (
              <div class="adc-channel">
                <span class="channel-name">CH{ch}</span>
                <input
                  type="range"
                  min="0"
                  max="4095"
                  value={adcChannels()[ch]}
                  onInput={(e) => updateAdcChannel(ch, parseInt(e.target.value))}
                  disabled={!props.simulationActive}
                />
                <span class="channel-value">{adcChannels()[ch]}</span>
                <span class="channel-voltage">
                  {((adcChannels()[ch] / 4095) * 3.3).toFixed(2)}V
                </span>
              </div>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
}
