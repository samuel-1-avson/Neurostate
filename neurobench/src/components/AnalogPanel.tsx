import { createSignal, Show, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface AnalogPanelProps {
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function AnalogPanel(props: AnalogPanelProps) {
  // Tab state
  const [activeTab, setActiveTab] = createSignal<"adc" | "dac" | "pwm">("adc");
  
  // ADC configuration
  const [adcInstance, setAdcInstance] = createSignal("ADC1");
  const [adcResolution, setAdcResolution] = createSignal(12);
  const [adcContinuous, setAdcContinuous] = createSignal(false);
  const [adcDma, setAdcDma] = createSignal(false);
  const [adcChannels, setAdcChannels] = createSignal([
    { channel: 0, pin: "PA0" }
  ]);
  
  // DAC configuration
  const [dacChannel, setDacChannel] = createSignal(1);
  const [dacBuffer, setDacBuffer] = createSignal(true);
  const [dacTrigger, setDacTrigger] = createSignal(false);
  const [dacWaveform, setDacWaveform] = createSignal("none");
  
  // PWM configuration
  const [pwmTimer, setPwmTimer] = createSignal("TIM2");
  const [pwmFrequency, setPwmFrequency] = createSignal(1000);
  const [pwmCenterAligned, setPwmCenterAligned] = createSignal(false);
  const [pwmChannels, setPwmChannels] = createSignal([
    { channel: 1, duty: 50, pin: "PA0" }
  ]);
  
  // Generated code
  const [generatedCode, setGeneratedCode] = createSignal("");
  const [isGenerating, setIsGenerating] = createSignal(false);

  const addLog = (source: string, message: string, type: "info" | "success" | "warning" | "error") => {
    props.onLog?.(source, message, type);
  };

  const addAdcChannel = () => {
    const channels = adcChannels();
    const nextChannel = channels.length;
    setAdcChannels([...channels, { channel: nextChannel, pin: `PA${nextChannel}` }]);
  };

  const removeAdcChannel = (index: number) => {
    setAdcChannels(adcChannels().filter((_, i) => i !== index));
  };

  const addPwmChannel = () => {
    const channels = pwmChannels();
    const nextChannel = channels.length + 1;
    setPwmChannels([...channels, { channel: nextChannel, duty: 50, pin: `PA${nextChannel}` }]);
  };

  const removePwmChannel = (index: number) => {
    setPwmChannels(pwmChannels().filter((_, i) => i !== index));
  };

  const generateADC = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_adc_code", {
        instance: adcInstance(),
        resolution: adcResolution(),
        channels: adcChannels(),
        continuousMode: adcContinuous(),
        dmaEnabled: adcDma(),
      }) as any;
      
      setGeneratedCode(result.code);
      addLog("ADC", `Generated ${adcInstance()} (${adcResolution()}-bit, ${result.num_channels} channels)`, "success");
    } catch (e) {
      addLog("ERROR", `ADC generation failed: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generateDAC = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_dac_code", {
        channel: dacChannel(),
        outputBuffer: dacBuffer(),
        triggerEnabled: dacTrigger(),
        waveform: dacWaveform(),
      }) as any;
      
      setGeneratedCode(result.code);
      addLog("DAC", `Generated DAC CH${result.channel} (${result.output_pin})`, "success");
    } catch (e) {
      addLog("ERROR", `DAC generation failed: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generatePWM = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_pwm_code", {
        timer: pwmTimer(),
        frequencyHz: pwmFrequency(),
        channels: pwmChannels(),
        centerAligned: pwmCenterAligned(),
      }) as any;
      
      setGeneratedCode(result.code);
      addLog("PWM", `Generated ${result.timer} @ ${result.frequency_hz}Hz`, "success");
    } catch (e) {
      addLog("ERROR", `PWM generation failed: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const copyCode = () => {
    navigator.clipboard.writeText(generatedCode());
    addLog("SYSTEM", "Code copied to clipboard", "success");
  };

  return (
    <div class="analog-panel">
      <div class="panel-header">
        <h3>📊 Analog I/O</h3>
      </div>
      
      {/* Tabs */}
      <div class="analog-tabs">
        <button 
          class={`tab ${activeTab() === "adc" ? "active" : ""}`}
          onClick={() => setActiveTab("adc")}
        >
          📈 ADC
        </button>
        <button 
          class={`tab ${activeTab() === "dac" ? "active" : ""}`}
          onClick={() => setActiveTab("dac")}
        >
          📉 DAC
        </button>
        <button 
          class={`tab ${activeTab() === "pwm" ? "active" : ""}`}
          onClick={() => setActiveTab("pwm")}
        >
          〰️ PWM
        </button>
      </div>

      {/* ADC Config */}
      <Show when={activeTab() === "adc"}>
        <div class="config-section">
          <div class="config-row">
            <label>Instance</label>
            <select value={adcInstance()} onChange={(e) => setAdcInstance(e.currentTarget.value)}>
              <option value="ADC1">ADC1</option>
              <option value="ADC2">ADC2</option>
              <option value="ADC3">ADC3</option>
            </select>
          </div>
          <div class="config-row">
            <label>Resolution</label>
            <select value={adcResolution()} onChange={(e) => setAdcResolution(parseInt(e.currentTarget.value))}>
              <option value={12}>12-bit (0-4095)</option>
              <option value={10}>10-bit (0-1023)</option>
              <option value={8}>8-bit (0-255)</option>
            </select>
          </div>
          <div class="config-row">
            <label>Continuous</label>
            <input type="checkbox" checked={adcContinuous()} onChange={(e) => setAdcContinuous(e.currentTarget.checked)} />
          </div>
          <div class="config-row">
            <label>DMA</label>
            <input type="checkbox" checked={adcDma()} onChange={(e) => setAdcDma(e.currentTarget.checked)} />
          </div>
          
          <div class="channels-section">
            <div class="channels-header">
              <span>Channels ({adcChannels().length})</span>
              <button class="add-btn" onClick={addAdcChannel}>+ Add</button>
            </div>
            <For each={adcChannels()}>
              {(ch, index) => (
                <div class="channel-row">
                  <span>CH{ch.channel}</span>
                  <input type="text" value={ch.pin} disabled />
                  <button class="remove-btn" onClick={() => removeAdcChannel(index())}>×</button>
                </div>
              )}
            </For>
          </div>
          
          <button class="generate-btn" onClick={generateADC} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate ADC Code"}
          </button>
        </div>
      </Show>

      {/* DAC Config */}
      <Show when={activeTab() === "dac"}>
        <div class="config-section">
          <div class="config-row">
            <label>Channel</label>
            <select value={dacChannel()} onChange={(e) => setDacChannel(parseInt(e.currentTarget.value))}>
              <option value={1}>Channel 1 (PA4)</option>
              <option value={2}>Channel 2 (PA5)</option>
            </select>
          </div>
          <div class="config-row">
            <label>Output Buffer</label>
            <input type="checkbox" checked={dacBuffer()} onChange={(e) => setDacBuffer(e.currentTarget.checked)} />
          </div>
          <div class="config-row">
            <label>Trigger</label>
            <input type="checkbox" checked={dacTrigger()} onChange={(e) => setDacTrigger(e.currentTarget.checked)} />
          </div>
          <div class="config-row">
            <label>Waveform</label>
            <select value={dacWaveform()} onChange={(e) => setDacWaveform(e.currentTarget.value)}>
              <option value="none">None</option>
              <option value="noise">Noise</option>
              <option value="triangle">Triangle</option>
            </select>
          </div>
          
          <div class="info-box">
            <p>📌 DAC Output: 0-4095 → 0-3.3V</p>
            <p>🔧 Resolution: 12-bit</p>
          </div>
          
          <button class="generate-btn" onClick={generateDAC} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate DAC Code"}
          </button>
        </div>
      </Show>

      {/* PWM Config */}
      <Show when={activeTab() === "pwm"}>
        <div class="config-section">
          <div class="config-row">
            <label>Timer</label>
            <select value={pwmTimer()} onChange={(e) => setPwmTimer(e.currentTarget.value)}>
              <option value="TIM1">TIM1 (Advanced)</option>
              <option value="TIM2">TIM2</option>
              <option value="TIM3">TIM3</option>
              <option value="TIM4">TIM4</option>
            </select>
          </div>
          <div class="config-row">
            <label>Frequency</label>
            <input 
              type="number" 
              value={pwmFrequency()} 
              onInput={(e) => setPwmFrequency(parseInt(e.currentTarget.value))}
            />
            <span class="unit">Hz</span>
          </div>
          <div class="config-row">
            <label>Center Aligned</label>
            <input type="checkbox" checked={pwmCenterAligned()} onChange={(e) => setPwmCenterAligned(e.currentTarget.checked)} />
          </div>
          
          <div class="channels-section">
            <div class="channels-header">
              <span>PWM Channels ({pwmChannels().length})</span>
              <button class="add-btn" onClick={addPwmChannel}>+ Add</button>
            </div>
            <For each={pwmChannels()}>
              {(ch, index) => (
                <div class="channel-row pwm-channel">
                  <span>CH{ch.channel}</span>
                  <input 
                    type="number" 
                    value={ch.duty} 
                    min="0" 
                    max="100"
                    class="duty-input"
                    onInput={(e) => {
                      const updated = [...pwmChannels()];
                      updated[index()].duty = parseInt(e.currentTarget.value);
                      setPwmChannels(updated);
                    }}
                  />
                  <span class="unit">%</span>
                  <button class="remove-btn" onClick={() => removePwmChannel(index())}>×</button>
                </div>
              )}
            </For>
          </div>
          
          <button class="generate-btn" onClick={generatePWM} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate PWM Code"}
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
