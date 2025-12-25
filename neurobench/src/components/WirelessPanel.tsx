import { createSignal, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { ValidationPanel } from "./ValidationPanel";
import { CodePreview } from "./CodePreview";

interface WirelessPanelProps {
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function WirelessPanel(props: WirelessPanelProps) {
  // Protocol selection
  const [protocol, setProtocol] = createSignal<"ble" | "wifi" | "lora">("ble");
  
  // BLE config
  const [blePlatform, setBlePlatform] = createSignal("nrf52");
  const [bleDeviceName, setBleDeviceName] = createSignal("NeuroBench");
  const [bleServiceUuid, setBleServiceUuid] = createSignal("180D");
  const [bleServiceName, setBleServiceName] = createSignal("HeartRate");
  const [bleCharRead, setBleCharRead] = createSignal(true);
  const [bleCharNotify, setBleCharNotify] = createSignal(true);
  
  // WiFi config
  const [wifiMode, setWifiMode] = createSignal("station");
  const [wifiSsid, setWifiSsid] = createSignal("MyNetwork");
  const [wifiPassword, setWifiPassword] = createSignal("");
  const [wifiSecurity, setWifiSecurity] = createSignal("wpa2");
  const [wifiChannel, setWifiChannel] = createSignal(1);
  
  // LoRa config
  const [loraFrequency, setLoraFrequency] = createSignal(915);
  const [loraSF, setLoraSF] = createSignal(7);
  const [loraBW, setLoraBW] = createSignal("125");
  const [loraCR, setLoraCR] = createSignal(5);
  const [loraPower, setLoraPower] = createSignal(14);
  
  // Generated code
  const [generatedCode, setGeneratedCode] = createSignal("");
  const [isGenerating, setIsGenerating] = createSignal(false);

  const addLog = (source: string, message: string, type: "info" | "success" | "warning" | "error" = "info") => {
    props.onLog?.(source, message, type);
  };

  const generateBle = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_ble_service", {
        platform: blePlatform(),
        deviceName: bleDeviceName(),
        serviceUuid: bleServiceUuid(),
        serviceName: bleServiceName(),
        characteristics: [{
          uuid: "2A37",
          name: "HeartRateMeasurement",
          read: bleCharRead(),
          notify: bleCharNotify(),
          max_length: 20,
        }],
      }) as any;
      setGeneratedCode(result.code);
      addLog("BLE", `Generated ${blePlatform().toUpperCase()} GATT service: ${bleServiceName()}`, "success");
    } catch (e) {
      addLog("ERROR", `Failed to generate BLE: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generateWifi = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_wifi_config", {
        mode: wifiMode(),
        ssid: wifiSsid(),
        password: wifiPassword(),
        security: wifiSecurity(),
        channel: wifiChannel(),
      }) as any;
      setGeneratedCode(result.code);
      addLog("WiFi", `Generated ESP32 WiFi ${wifiMode()}: ${wifiSsid()}`, "success");
    } catch (e) {
      addLog("ERROR", `Failed to generate WiFi: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generateLora = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_lora_config", {
        frequencyMhz: loraFrequency(),
        spreadingFactor: loraSF(),
        bandwidth: loraBW(),
        codingRate: loraCR(),
        txPower: loraPower(),
      }) as any;
      setGeneratedCode(result.code);
      addLog("LoRa", `Generated SX127x config: ${loraFrequency()}MHz SF${loraSF()}`, "success");
    } catch (e) {
      addLog("ERROR", `Failed to generate LoRa: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const copyToClipboard = () => {
    navigator.clipboard.writeText(generatedCode());
    addLog("Wireless", "Code copied to clipboard", "info");
  };

  return (
    <div class="wireless-panel">
      <div class="wireless-header">
        <h3>📡 Wireless Configuration</h3>
      </div>

      {/* Protocol Tabs */}
      <div class="protocol-tabs">
        <button class={`tab ${protocol() === "ble" ? "active" : ""}`} onClick={() => setProtocol("ble")}>
          🔷 BLE
        </button>
        <button class={`tab ${protocol() === "wifi" ? "active" : ""}`} onClick={() => setProtocol("wifi")}>
          📶 WiFi
        </button>
        <button class={`tab ${protocol() === "lora" ? "active" : ""}`} onClick={() => setProtocol("lora")}>
          📻 LoRa
        </button>
      </div>

      {/* BLE Config */}
      <Show when={protocol() === "ble"}>
        <div class="config-section">
          <div class="config-row">
            <label>Platform</label>
            <select value={blePlatform()} onChange={(e) => setBlePlatform(e.target.value)}>
              <option value="nrf52">Nordic nRF52</option>
              <option value="esp32">ESP32</option>
            </select>
          </div>
          <div class="config-row">
            <label>Device Name</label>
            <input type="text" value={bleDeviceName()} onInput={(e) => setBleDeviceName(e.target.value)} />
          </div>
          <div class="config-row">
            <label>Service UUID</label>
            <input type="text" value={bleServiceUuid()} onInput={(e) => setBleServiceUuid(e.target.value)} />
          </div>
          <div class="config-row">
            <label>Service Name</label>
            <input type="text" value={bleServiceName()} onInput={(e) => setBleServiceName(e.target.value)} />
          </div>
          <div class="config-row checkbox">
            <label>
              <input type="checkbox" checked={bleCharRead()} onChange={(e) => setBleCharRead(e.target.checked)} />
              Characteristic: Read
            </label>
          </div>
          <div class="config-row checkbox">
            <label>
              <input type="checkbox" checked={bleCharNotify()} onChange={(e) => setBleCharNotify(e.target.checked)} />
              Characteristic: Notify
            </label>
          </div>
          <button class="generate-btn" onClick={generateBle} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate BLE Service"}
          </button>
        </div>
      </Show>

      {/* WiFi Config */}
      <Show when={protocol() === "wifi"}>
        <div class="config-section">
          <div class="config-row">
            <label>Mode</label>
            <select value={wifiMode()} onChange={(e) => setWifiMode(e.target.value)}>
              <option value="station">Station (Client)</option>
              <option value="ap">Access Point</option>
            </select>
          </div>
          <div class="config-row">
            <label>SSID</label>
            <input type="text" value={wifiSsid()} onInput={(e) => setWifiSsid(e.target.value)} />
          </div>
          <div class="config-row">
            <label>Password</label>
            <input type="password" value={wifiPassword()} onInput={(e) => setWifiPassword(e.target.value)} />
          </div>
          <div class="config-row">
            <label>Security</label>
            <select value={wifiSecurity()} onChange={(e) => setWifiSecurity(e.target.value)}>
              <option value="open">Open</option>
              <option value="wpa2">WPA2 Personal</option>
              <option value="wpa3">WPA3 Personal</option>
            </select>
          </div>
          <Show when={wifiMode() === "ap"}>
            <div class="config-row">
              <label>Channel</label>
              <input type="number" value={wifiChannel()} onInput={(e) => setWifiChannel(parseInt(e.target.value) || 1)} min={1} max={13} />
            </div>
          </Show>
          <button class="generate-btn" onClick={generateWifi} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate WiFi Config"}
          </button>
        </div>
      </Show>

      {/* LoRa Config */}
      <Show when={protocol() === "lora"}>
        <div class="config-section">
          <div class="config-row">
            <label>Frequency (MHz)</label>
            <select value={loraFrequency()} onChange={(e) => setLoraFrequency(parseInt(e.target.value))}>
              <option value={433}>433 MHz (EU/Asia)</option>
              <option value={868}>868 MHz (EU)</option>
              <option value={915}>915 MHz (US)</option>
              <option value={923}>923 MHz (Asia)</option>
            </select>
          </div>
          <div class="config-row">
            <label>Spreading Factor</label>
            <select value={loraSF()} onChange={(e) => setLoraSF(parseInt(e.target.value))}>
              <option value={7}>SF7 (Fastest)</option>
              <option value={8}>SF8</option>
              <option value={9}>SF9</option>
              <option value={10}>SF10</option>
              <option value={11}>SF11</option>
              <option value={12}>SF12 (Longest Range)</option>
            </select>
          </div>
          <div class="config-row">
            <label>Bandwidth (kHz)</label>
            <select value={loraBW()} onChange={(e) => setLoraBW(e.target.value)}>
              <option value="125">125 kHz</option>
              <option value="250">250 kHz</option>
              <option value="500">500 kHz</option>
            </select>
          </div>
          <div class="config-row">
            <label>TX Power (dBm)</label>
            <input type="number" value={loraPower()} onInput={(e) => setLoraPower(parseInt(e.target.value) || 14)} min={2} max={20} />
          </div>
          <button class="generate-btn" onClick={generateLora} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate LoRa Config"}
          </button>
        </div>
      </Show>

      {/* Generated Code */}
      <Show when={generatedCode()}>
        <div class="code-output">
          <CodePreview 
            code={generatedCode()} 
            language="c" 
            showLineNumbers={true}
            onCopy={() => addLog("Wireless", "Code copied to clipboard", "info")}
          />
          
          {/* Code Validation */}
          <ValidationPanel 
            code={generatedCode()} 
            language="c" 
            onLog={props.onLog} 
          />
        </div>
      </Show>
    </div>
  );
}
