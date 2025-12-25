import { createSignal, Show, createEffect } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface ClockPanelProps {
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function ClockPanel(props: ClockPanelProps) {
  // Tab state
  const [activeTab, setActiveTab] = createSignal<"clock" | "power">("clock");
  
  // PLL Configuration
  const [pllSource, setPllSource] = createSignal("HSE");
  const [hseFreq, setHseFreq] = createSignal(8);
  const [pllm, setPllm] = createSignal(8);
  const [plln, setPlln] = createSignal(336);
  const [pllp, setPllp] = createSignal(2);
  const [pllq, setPllq] = createSignal(7);
  
  // Bus prescalers
  const [ahbDiv, setAhbDiv] = createSignal(1);
  const [apb1Div, setApb1Div] = createSignal(4);
  const [apb2Div, setApb2Div] = createSignal(2);
  
  // Calculated frequencies
  const [sysclk, setSysclk] = createSignal(168);
  const [hclk, setHclk] = createSignal(168);
  const [pclk1, setPclk1] = createSignal(42);
  const [pclk2, setPclk2] = createSignal(84);
  const [pll48, setPll48] = createSignal(48);
  const [usbValid, setUsbValid] = createSignal(true);
  
  // Low power configuration
  const [powerMode, setPowerMode] = createSignal("sleep");
  const [wakeupPin, setWakeupPin] = createSignal(true);
  const [rtcAlarm, setRtcAlarm] = createSignal(false);
  const [rtcWakeup, setRtcWakeup] = createSignal(false);
  
  // Power estimates
  const [runMa, setRunMa] = createSignal(0);
  const [sleepMa, setSleepMa] = createSignal(0);
  const [stopUa, setStopUa] = createSignal(0);
  const [standbyUa, setStandbyUa] = createSignal(0);
  
  // Generated code  
  const [generatedCode, setGeneratedCode] = createSignal("");
  const [isGenerating, setIsGenerating] = createSignal(false);

  const addLog = (source: string, message: string, type: "info" | "success" | "warning" | "error") => {
    props.onLog?.(source, message, type);
  };

  // Calculate frequencies when PLL params change
  createEffect(async () => {
    try {
      const result = await invoke("calculate_clock_frequencies", {
        pllSource: pllSource(),
        hseFreqHz: pllSource() === "HSE" ? hseFreq() * 1_000_000 : null,
        pllm: pllm(),
        plln: plln(),
        pllp: pllp(),
        pllq: pllq(),
        ahbPrescaler: ahbDiv(),
        apb1Prescaler: apb1Div(),
        apb2Prescaler: apb2Div(),
      }) as any;
      
      setSysclk(result.sysclk_mhz);
      setHclk(result.hclk_mhz);
      setPclk1(result.pclk1_mhz);
      setPclk2(result.pclk2_mhz);
      setPll48(result.pll_48_mhz);
      setUsbValid(result.usb_valid);
    } catch (e) {
      // Ignore calculation errors during typing
    }
  });

  const generateClockCode = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_clock_config", {
        sysclkSource: "PLL",
        hseFreqHz: pllSource() === "HSE" ? hseFreq() * 1_000_000 : null,
        pllm: pllm(),
        plln: plln(),
        pllp: pllp(),
        pllq: pllq(),
        ahbPrescaler: ahbDiv(),
        apb1Prescaler: apb1Div(),
        apb2Prescaler: apb2Div(),
      }) as any;
      
      setGeneratedCode(result.code);
      addLog("CLOCK", `Generated clock config @ ${result.frequencies.sysclk_mhz}MHz`, "success");
    } catch (e) {
      addLog("ERROR", `Clock generation failed: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generatePowerCode = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_low_power_code", {
        mode: powerMode(),
        wakeupPin: wakeupPin(),
        rtcAlarm: rtcAlarm(),
        rtcWakeup: rtcWakeup(),
        externalInterrupt: null,
      }) as any;
      
      setGeneratedCode(result.code);
      setRunMa(result.power_estimate.run_mode_ma);
      setSleepMa(result.power_estimate.sleep_mode_ma);
      setStopUa(result.power_estimate.stop_mode_ua);
      setStandbyUa(result.power_estimate.standby_mode_ua);
      addLog("POWER", `Generated ${powerMode()} mode code`, "success");
    } catch (e) {
      addLog("ERROR", `Power code generation failed: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const copyCode = () => {
    navigator.clipboard.writeText(generatedCode());
    addLog("SYSTEM", "Code copied to clipboard", "success");
  };

  return (
    <div class="clock-panel">
      <div class="panel-header">
        <h3>⚙️ Clock & Power</h3>
      </div>
      
      {/* Tabs */}
      <div class="clock-tabs">
        <button 
          class={`tab ${activeTab() === "clock" ? "active" : ""}`}
          onClick={() => setActiveTab("clock")}
        >
          🕐 Clock Tree
        </button>
        <button 
          class={`tab ${activeTab() === "power" ? "active" : ""}`}
          onClick={() => setActiveTab("power")}
        >
          ⚡ Low Power
        </button>
      </div>

      {/* Clock Configuration */}
      <Show when={activeTab() === "clock"}>
        <div class="config-section">
          {/* PLL Source */}
          <div class="config-row">
            <label>PLL Source</label>
            <select value={pllSource()} onChange={(e) => setPllSource(e.currentTarget.value)}>
              <option value="HSI">HSI (16MHz)</option>
              <option value="HSE">HSE (External)</option>
            </select>
          </div>
          
          <Show when={pllSource() === "HSE"}>
            <div class="config-row">
              <label>HSE Freq (MHz)</label>
              <input 
                type="number" 
                value={hseFreq()} 
                onInput={(e) => setHseFreq(parseInt(e.currentTarget.value))}
              />
            </div>
          </Show>
          
          {/* PLL Parameters */}
          <div class="pll-config">
            <div class="config-row">
              <label>PLLM (/M)</label>
              <input 
                type="number" 
                min="2" max="63"
                value={pllm()} 
                onInput={(e) => setPllm(parseInt(e.currentTarget.value))}
              />
            </div>
            <div class="config-row">
              <label>PLLN (×N)</label>
              <input 
                type="number"
                min="50" max="432"
                value={plln()} 
                onInput={(e) => setPlln(parseInt(e.currentTarget.value))}
              />
            </div>
            <div class="config-row">
              <label>PLLP (/P)</label>
              <select value={pllp()} onChange={(e) => setPllp(parseInt(e.currentTarget.value))}>
                <option value={2}>÷2</option>
                <option value={4}>÷4</option>
                <option value={6}>÷6</option>
                <option value={8}>÷8</option>
              </select>
            </div>
            <div class="config-row">
              <label>PLLQ (/Q)</label>
              <input 
                type="number"
                min="2" max="15"
                value={pllq()} 
                onInput={(e) => setPllq(parseInt(e.currentTarget.value))}
              />
            </div>
          </div>
          
          {/* Bus Prescalers */}
          <div class="bus-config">
            <div class="config-row">
              <label>AHB (/)</label>
              <select value={ahbDiv()} onChange={(e) => setAhbDiv(parseInt(e.currentTarget.value))}>
                <option value={1}>÷1</option>
                <option value={2}>÷2</option>
                <option value={4}>÷4</option>
                <option value={8}>÷8</option>
                <option value={16}>÷16</option>
              </select>
            </div>
            <div class="config-row">
              <label>APB1 (/)</label>
              <select value={apb1Div()} onChange={(e) => setApb1Div(parseInt(e.currentTarget.value))}>
                <option value={1}>÷1</option>
                <option value={2}>÷2</option>
                <option value={4}>÷4</option>
                <option value={8}>÷8</option>
                <option value={16}>÷16</option>
              </select>
            </div>
            <div class="config-row">
              <label>APB2 (/)</label>
              <select value={apb2Div()} onChange={(e) => setApb2Div(parseInt(e.currentTarget.value))}>
                <option value={1}>÷1</option>
                <option value={2}>÷2</option>
                <option value={4}>÷4</option>
                <option value={8}>÷8</option>
                <option value={16}>÷16</option>
              </select>
            </div>
          </div>
          
          {/* Calculated Frequencies Display */}
          <div class="freq-display">
            <div class="freq-item">
              <span class="freq-label">SYSCLK</span>
              <span class="freq-value">{sysclk()} MHz</span>
            </div>
            <div class="freq-item">
              <span class="freq-label">HCLK (AHB)</span>
              <span class="freq-value">{hclk()} MHz</span>
            </div>
            <div class="freq-item">
              <span class="freq-label">PCLK1 (APB1)</span>
              <span class="freq-value">{pclk1()} MHz</span>
            </div>
            <div class="freq-item">
              <span class="freq-label">PCLK2 (APB2)</span>
              <span class="freq-value">{pclk2()} MHz</span>
            </div>
            <div class="freq-item">
              <span class="freq-label">USB/SDIO</span>
              <span class={`freq-value ${usbValid() ? "valid" : "invalid"}`}>
                {pll48()} MHz {usbValid() ? "✓" : "✗"}
              </span>
            </div>
          </div>
          
          <button class="generate-btn" onClick={generateClockCode} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate Clock Config"}
          </button>
        </div>
      </Show>

      {/* Power Configuration */}
      <Show when={activeTab() === "power"}>
        <div class="config-section">
          <div class="config-row">
            <label>Mode</label>
            <select value={powerMode()} onChange={(e) => setPowerMode(e.currentTarget.value)}>
              <option value="sleep">Sleep (~2mA)</option>
              <option value="stop">Stop (~12µA)</option>
              <option value="standby">Standby (~2.5µA)</option>
            </select>
          </div>
          
          <div class="wakeup-sources">
            <label>Wake Sources:</label>
            <div class="checkbox-row">
              <input type="checkbox" checked={wakeupPin()} onChange={(e) => setWakeupPin(e.currentTarget.checked)} />
              <span>WKUP Pin</span>
            </div>
            <div class="checkbox-row">
              <input type="checkbox" checked={rtcAlarm()} onChange={(e) => setRtcAlarm(e.currentTarget.checked)} />
              <span>RTC Alarm</span>
            </div>
            <div class="checkbox-row">
              <input type="checkbox" checked={rtcWakeup()} onChange={(e) => setRtcWakeup(e.currentTarget.checked)} />
              <span>RTC Wakeup Timer</span>
            </div>
          </div>
          
          <Show when={runMa() > 0}>
            <div class="power-estimate">
              <div class="power-item">
                <span>Run Mode</span>
                <span class="power-value">{runMa().toFixed(1)} mA</span>
              </div>
              <div class="power-item">
                <span>Sleep Mode</span>
                <span class="power-value">{sleepMa().toFixed(1)} mA</span>
              </div>
              <div class="power-item">
                <span>Stop Mode</span>
                <span class="power-value">{stopUa().toFixed(1)} µA</span>
              </div>
              <div class="power-item">
                <span>Standby Mode</span>
                <span class="power-value">{standbyUa().toFixed(1)} µA</span>
              </div>
            </div>
          </Show>
          
          <button class="generate-btn" onClick={generatePowerCode} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate Low-Power Code"}
          </button>
        </div>
      </Show>

      {/* Generated Code Output */}
      <Show when={generatedCode()}>
        <div class="code-output">
          <div class="code-header">
            <span>Generated Code</span>
            <button class="copy-btn" onClick={copyCode}>📋 Copy</button>
          </div>
          <pre><code>{generatedCode()}</code></pre>
        </div>
      </Show>
    </div>
  );
}
