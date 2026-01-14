import { createSignal, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { Icons } from "./AppIcons";
import "./SerialPanel.css";

interface SerialPanelProps {
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

interface PortInfo {
  name: string;
  description: string;
  port_type: string;
}

export function SerialPanel(props: SerialPanelProps) {
  const [ports, setPorts] = createSignal<PortInfo[]>([]);
  const [selectedPort, setSelectedPort] = createSignal("");
  const [baudRate, setBaudRate] = createSignal(115200);
  const [baudRates, setBaudRates] = createSignal<number[]>([]);
  const [isConnected, setIsConnected] = createSignal(false);
  const [outputData, setOutputData] = createSignal<string[]>([]);
  const [inputText, setInputText] = createSignal("");
  const [displayFormat, setDisplayFormat] = createSignal("ascii");

  const scanPorts = async () => {
    try {
      const result = await invoke("serial_list_ports") as PortInfo[];
      setPorts(result);
      
      const rates = await invoke("serial_get_baud_rates") as { baudRates: number[] };
      setBaudRates(rates.baudRates);
      
      if (result.length > 0 && !selectedPort()) {
        setSelectedPort(result[0].name);
      }
      
      props.onLog?.("Serial", `Found ${result.length} ports`, "info");
    } catch (e) {
      props.onLog?.("Serial", `Scan failed: ${e}`, "error");
    }
  };

  const toggleConnection = () => {
    if (isConnected()) {
      setIsConnected(false);
      props.onLog?.("Serial", "Disconnected", "info");
    } else {
      setIsConnected(true);
      props.onLog?.("Serial", `Connected to ${selectedPort()} @ ${baudRate()}`, "success");
    }
  };

  const sendData = async () => {
    if (!inputText().trim()) return;
    
    try {
      const result = await invoke("serial_parse_escape", { input: inputText() }) as { bytes: number[] };
      const formatted = await invoke("serial_format_data", {
        data: result.bytes,
        format: "hex",
      }) as { formatted: string };
      
      setOutputData([...outputData(), `TX: ${inputText()} [${formatted.formatted}]`]);
      setInputText("");
    } catch (e) {
      props.onLog?.("Serial", `Send failed: ${e}`, "error");
    }
  };

  const clearOutput = () => {
    setOutputData([]);
  };

  // Scan on mount
  if (ports().length === 0) {
    scanPorts();
  }

  return (
    <div class="serial-panel">
      <div class="panel-header">
        <h3><span class="header-icon">{Icons.serial()}</span> Serial Monitor</h3>
        <button class="scan-btn" onClick={scanPorts}>{Icons.refresh()} Scan</button>
      </div>

      <div class="config-section">
        <div class="config-col">
          <div class="config-row">
            <label>Port</label>
            <select value={selectedPort()} onChange={(e) => setSelectedPort(e.target.value)}>
              <For each={ports()}>
                {(port) => (
                  <option value={port.name}>{port.name} - {port.description}</option>
                )}
              </For>
            </select>
          </div>
        </div>
        
        <div class="config-col" style="max-width: 120px;">
          <div class="config-row">
            <label>Baud Rate</label>
            <select value={baudRate()} onChange={(e) => setBaudRate(parseInt(e.target.value))}>
              <For each={baudRates()}>
                {(rate) => <option value={rate}>{rate}</option>}
              </For>
            </select>
          </div>
        </div>

        <div class="connect-wrapper">
          <button 
            class={`connect-btn ${isConnected() ? "connected" : ""}`}
            onClick={toggleConnection}
          >
            {isConnected() ? <>{Icons.plug()} Disconnect</> : <>{Icons.sync()} Connect</>}
          </button>
        </div>
      </div>

      <div class="output-container">
        <div class="output-header">
          <span>Output ({displayFormat()})</span>
          <div class="output-controls">
            <select value={displayFormat()} onChange={(e) => setDisplayFormat(e.target.value)}>
              <option value="ascii">ASCII</option>
              <option value="hex">HEX</option>
              <option value="decimal">DEC</option>
            </select>
            <button onClick={clearOutput}>Clear Output</button>
          </div>
        </div>
        <div class="output-area">
          <For each={outputData()}>
            {(line) => <div class="output-line">{line}</div>}
          </For>
          <Show when={outputData().length === 0}>
            <div class="output-placeholder">No data received</div>
          </Show>
        </div>
      </div>

      <div class="input-container">
        <input 
          type="text" 
          placeholder="Enter data to send..."
          value={inputText()}
          onInput={(e) => setInputText(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && sendData()}
          disabled={!isConnected()}
        />
        <button onClick={sendData} disabled={!isConnected()}>Send</button>
      </div>


    </div>
  );
}
