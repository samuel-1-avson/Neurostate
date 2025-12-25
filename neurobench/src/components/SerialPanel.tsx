import { createSignal, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

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
        <h3>📡 Serial Monitor</h3>
        <button class="scan-btn" onClick={scanPorts}>🔄 Scan</button>
      </div>

      {/* Port selection */}
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

      {/* Baud rate */}
      <div class="config-row">
        <label>Baud Rate</label>
        <select value={baudRate()} onChange={(e) => setBaudRate(parseInt(e.target.value))}>
          <For each={baudRates()}>
            {(rate) => <option value={rate}>{rate}</option>}
          </For>
        </select>
      </div>

      {/* Connect button */}
      <button 
        class={`connect-btn ${isConnected() ? "connected" : ""}`}
        onClick={toggleConnection}
      >
        {isConnected() ? "🔌 Disconnect" : "🔗 Connect"}
      </button>

      {/* Output display */}
      <div class="output-container">
        <div class="output-header">
          <span>Output</span>
          <div class="output-controls">
            <select value={displayFormat()} onChange={(e) => setDisplayFormat(e.target.value)}>
              <option value="ascii">ASCII</option>
              <option value="hex">HEX</option>
              <option value="decimal">DEC</option>
            </select>
            <button onClick={clearOutput}>Clear</button>
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

      {/* Input */}
      <div class="input-container">
        <input 
          type="text" 
          placeholder="Enter data to send (supports \n, \r, \xNN)"
          value={inputText()}
          onInput={(e) => setInputText(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && sendData()}
          disabled={!isConnected()}
        />
        <button onClick={sendData} disabled={!isConnected()}>Send</button>
      </div>

      <style>{`
        .serial-panel {
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

        .scan-btn {
          background: rgba(255,255,255,0.1);
          border: none;
          color: #ccc;
          padding: 4px 10px;
          border-radius: 4px;
          cursor: pointer;
          font-size: 11px;
        }

        .config-row { margin-bottom: 10px; }
        .config-row label {
          display: block;
          color: #888;
          font-size: 11px;
          margin-bottom: 4px;
        }

        .config-row select {
          width: 100%;
          background: rgba(255,255,255,0.05);
          border: 1px solid #333;
          color: #fff;
          padding: 8px 12px;
          border-radius: 6px;
        }

        .connect-btn {
          width: 100%;
          background: linear-gradient(135deg, #22c55e, #16a34a);
          color: white;
          border: none;
          padding: 10px;
          border-radius: 6px;
          cursor: pointer;
          font-weight: 600;
          margin-bottom: 12px;
        }

        .connect-btn.connected {
          background: linear-gradient(135deg, #ef4444, #dc2626);
        }

        .output-container {
          background: #11111b;
          border-radius: 6px;
          margin-bottom: 10px;
          overflow: hidden;
        }

        .output-header {
          display: flex;
          justify-content: space-between;
          padding: 6px 10px;
          background: rgba(255,255,255,0.05);
          font-size: 11px;
        }

        .output-controls {
          display: flex;
          gap: 6px;
        }

        .output-controls select,
        .output-controls button {
          background: rgba(255,255,255,0.1);
          border: none;
          color: #ccc;
          padding: 2px 8px;
          border-radius: 4px;
          font-size: 10px;
          cursor: pointer;
        }

        .output-area {
          height: 150px;
          overflow-y: auto;
          padding: 8px;
          font-family: 'Fira Code', monospace;
          font-size: 11px;
        }

        .output-line { color: #4ade80; margin-bottom: 2px; }
        .output-placeholder { color: #555; font-style: italic; }

        .input-container {
          display: flex;
          gap: 8px;
        }

        .input-container input {
          flex: 1;
          background: rgba(255,255,255,0.05);
          border: 1px solid #333;
          color: #fff;
          padding: 8px 12px;
          border-radius: 6px;
          font-size: 12px;
        }

        .input-container button {
          background: linear-gradient(135deg, #3b82f6, #2563eb);
          color: white;
          border: none;
          padding: 8px 16px;
          border-radius: 6px;
          cursor: pointer;
        }

        .input-container button:disabled,
        .input-container input:disabled {
          opacity: 0.5;
        }
      `}</style>
    </div>
  );
}
