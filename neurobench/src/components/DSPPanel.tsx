import { createSignal, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { ValidationPanel } from "./ValidationPanel";
import { CodePreview } from "./CodePreview";

interface DSPPanelProps {
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function DSPPanel(props: DSPPanelProps) {
  // DSP type selection
  const [dspType, setDspType] = createSignal<"fir" | "iir" | "fft" | "pid" | "buffer">("fir");
  
  // FIR config
  const [firName, setFirName] = createSignal("lowpass_fir");
  const [firType, setFirType] = createSignal("lowpass");
  const [firOrder, setFirOrder] = createSignal(31);
  const [firSampleRate, setFirSampleRate] = createSignal(48000);
  const [firCutoff, setFirCutoff] = createSignal(1000);
  const [firWindow, setFirWindow] = createSignal("hamming");
  
  // IIR config
  const [iirName, setIirName] = createSignal("biquad_lp");
  const [iirType, setIirType] = createSignal("lowpass");
  const [iirSampleRate, setIirSampleRate] = createSignal(48000);
  const [iirCutoff, setIirCutoff] = createSignal(1000);
  const [iirQ, setIirQ] = createSignal(0.707);
  
  // FFT config
  const [fftName, setFftName] = createSignal("audio_fft");
  const [fftSize, setFftSize] = createSignal(256);
  const [fftWindow, setFftWindow] = createSignal(true);
  const [fftWindowType, setFftWindowType] = createSignal("hanning");
  
  // PID config
  const [pidName, setPidName] = createSignal("motor_pid");
  const [pidKp, setPidKp] = createSignal(1.0);
  const [pidKi, setPidKi] = createSignal(0.1);
  const [pidKd, setPidKd] = createSignal(0.01);
  const [pidOutMin, setPidOutMin] = createSignal(-100);
  const [pidOutMax, setPidOutMax] = createSignal(100);
  const [pidSampleTime, setPidSampleTime] = createSignal(10);
  const [pidAntiWindup, setPidAntiWindup] = createSignal(true);
  
  // Buffer config
  const [bufName, setBufName] = createSignal("audio_buffer");
  const [bufSize, setBufSize] = createSignal(1024);
  const [bufType, setBufType] = createSignal("int16_t");
  const [bufThreadSafe, setBufThreadSafe] = createSignal(false);
  
  // Generated code
  const [generatedCode, setGeneratedCode] = createSignal("");
  const [isGenerating, setIsGenerating] = createSignal(false);

  const addLog = (source: string, message: string, type: "info" | "success" | "warning" | "error" = "info") => {
    props.onLog?.(source, message, type);
  };

  const generateFir = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_fir_filter", {
        name: firName(),
        filterType: firType(),
        order: firOrder(),
        sampleRate: firSampleRate(),
        cutoffFreq: firCutoff(),
        window: firWindow(),
      }) as any;
      setGeneratedCode(result.code);
      addLog("DSP", `Generated FIR filter: ${firName()}, Order ${firOrder()}`, "success");
    } catch (e) {
      addLog("ERROR", `Failed to generate FIR: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generateIir = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_iir_filter", {
        name: iirName(),
        filterType: iirType(),
        sampleRate: iirSampleRate(),
        cutoffFreq: iirCutoff(),
        qFactor: iirQ(),
      }) as any;
      setGeneratedCode(result.code);
      addLog("DSP", `Generated IIR biquad: ${iirName()}, Fc ${iirCutoff()}Hz`, "success");
    } catch (e) {
      addLog("ERROR", `Failed to generate IIR: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generateFft = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_fft_block", {
        name: fftName(),
        size: fftSize(),
        useWindow: fftWindow(),
        window: fftWindowType(),
      }) as any;
      setGeneratedCode(result.code);
      addLog("DSP", `Generated FFT: ${fftName()}, ${fftSize()} points`, "success");
    } catch (e) {
      addLog("ERROR", `Failed to generate FFT: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generatePid = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_pid_controller", {
        name: pidName(),
        kp: pidKp(),
        ki: pidKi(),
        kd: pidKd(),
        outputMin: pidOutMin(),
        outputMax: pidOutMax(),
        sampleTimeMs: pidSampleTime(),
        antiWindup: pidAntiWindup(),
      }) as any;
      setGeneratedCode(result.code);
      addLog("DSP", `Generated PID: ${pidName()}, Kp=${pidKp()}`, "success");
    } catch (e) {
      addLog("ERROR", `Failed to generate PID: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generateBuffer = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_circular_buffer", {
        name: bufName(),
        size: bufSize(),
        elementType: bufType(),
        threadSafe: bufThreadSafe(),
      }) as any;
      setGeneratedCode(result.code);
      addLog("DSP", `Generated buffer: ${bufName()}, ${bufSize()} elements`, "success");
    } catch (e) {
      addLog("ERROR", `Failed to generate buffer: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const copyToClipboard = () => {
    navigator.clipboard.writeText(generatedCode());
    addLog("DSP", "Code copied to clipboard", "info");
  };

  return (
    <div class="dsp-panel">
      <div class="dsp-header">
        <h3>📊 DSP Configuration</h3>
      </div>

      {/* DSP Type Tabs */}
      <div class="dsp-tabs">
        <button class={`tab ${dspType() === "fir" ? "active" : ""}`} onClick={() => setDspType("fir")}>FIR</button>
        <button class={`tab ${dspType() === "iir" ? "active" : ""}`} onClick={() => setDspType("iir")}>IIR</button>
        <button class={`tab ${dspType() === "fft" ? "active" : ""}`} onClick={() => setDspType("fft")}>FFT</button>
        <button class={`tab ${dspType() === "pid" ? "active" : ""}`} onClick={() => setDspType("pid")}>PID</button>
        <button class={`tab ${dspType() === "buffer" ? "active" : ""}`} onClick={() => setDspType("buffer")}>Buffer</button>
      </div>

      {/* FIR Config */}
      <Show when={dspType() === "fir"}>
        <div class="config-section">
          <div class="config-row">
            <label>Filter Name</label>
            <input type="text" value={firName()} onInput={(e) => setFirName(e.target.value)} />
          </div>
          <div class="config-row">
            <label>Type</label>
            <select value={firType()} onChange={(e) => setFirType(e.target.value)}>
              <option value="lowpass">Lowpass</option>
              <option value="highpass">Highpass</option>
              <option value="bandpass">Bandpass</option>
            </select>
          </div>
          <div class="config-row">
            <label>Order (Taps - 1)</label>
            <input type="number" value={firOrder()} onInput={(e) => setFirOrder(parseInt(e.target.value) || 31)} />
          </div>
          <div class="config-row">
            <label>Sample Rate (Hz)</label>
            <input type="number" value={firSampleRate()} onInput={(e) => setFirSampleRate(parseInt(e.target.value) || 48000)} />
          </div>
          <div class="config-row">
            <label>Cutoff Frequency (Hz)</label>
            <input type="number" value={firCutoff()} onInput={(e) => setFirCutoff(parseInt(e.target.value) || 1000)} />
          </div>
          <div class="config-row">
            <label>Window</label>
            <select value={firWindow()} onChange={(e) => setFirWindow(e.target.value)}>
              <option value="hamming">Hamming</option>
              <option value="hanning">Hanning</option>
              <option value="blackman">Blackman</option>
              <option value="rectangular">Rectangular</option>
            </select>
          </div>
          <button class="generate-btn" onClick={generateFir} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate FIR Filter"}
          </button>
        </div>
      </Show>

      {/* IIR Config */}
      <Show when={dspType() === "iir"}>
        <div class="config-section">
          <div class="config-row">
            <label>Filter Name</label>
            <input type="text" value={iirName()} onInput={(e) => setIirName(e.target.value)} />
          </div>
          <div class="config-row">
            <label>Type</label>
            <select value={iirType()} onChange={(e) => setIirType(e.target.value)}>
              <option value="lowpass">Lowpass</option>
              <option value="highpass">Highpass</option>
              <option value="bandpass">Bandpass</option>
            </select>
          </div>
          <div class="config-row">
            <label>Sample Rate (Hz)</label>
            <input type="number" value={iirSampleRate()} onInput={(e) => setIirSampleRate(parseInt(e.target.value) || 48000)} />
          </div>
          <div class="config-row">
            <label>Cutoff Frequency (Hz)</label>
            <input type="number" value={iirCutoff()} onInput={(e) => setIirCutoff(parseInt(e.target.value) || 1000)} />
          </div>
          <div class="config-row">
            <label>Q Factor</label>
            <input type="number" step="0.01" value={iirQ()} onInput={(e) => setIirQ(parseFloat(e.target.value) || 0.707)} />
          </div>
          <button class="generate-btn" onClick={generateIir} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate IIR Filter"}
          </button>
        </div>
      </Show>

      {/* FFT Config */}
      <Show when={dspType() === "fft"}>
        <div class="config-section">
          <div class="config-row">
            <label>FFT Name</label>
            <input type="text" value={fftName()} onInput={(e) => setFftName(e.target.value)} />
          </div>
          <div class="config-row">
            <label>Size (Power of 2)</label>
            <select value={fftSize()} onChange={(e) => setFftSize(parseInt(e.target.value))}>
              <option value={64}>64</option>
              <option value={128}>128</option>
              <option value={256}>256</option>
              <option value={512}>512</option>
              <option value={1024}>1024</option>
              <option value={2048}>2048</option>
              <option value={4096}>4096</option>
            </select>
          </div>
          <div class="config-row checkbox">
            <label>
              <input type="checkbox" checked={fftWindow()} onChange={(e) => setFftWindow(e.target.checked)} />
              Apply Window
            </label>
          </div>
          <Show when={fftWindow()}>
            <div class="config-row">
              <label>Window Type</label>
              <select value={fftWindowType()} onChange={(e) => setFftWindowType(e.target.value)}>
                <option value="hanning">Hanning</option>
                <option value="hamming">Hamming</option>
                <option value="blackman">Blackman</option>
              </select>
            </div>
          </Show>
          <button class="generate-btn" onClick={generateFft} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate FFT Block"}
          </button>
        </div>
      </Show>

      {/* PID Config */}
      <Show when={dspType() === "pid"}>
        <div class="config-section">
          <div class="config-row">
            <label>Controller Name</label>
            <input type="text" value={pidName()} onInput={(e) => setPidName(e.target.value)} />
          </div>
          <div class="config-row-group">
            <div class="config-row">
              <label>Kp</label>
              <input type="number" step="0.1" value={pidKp()} onInput={(e) => setPidKp(parseFloat(e.target.value) || 1)} />
            </div>
            <div class="config-row">
              <label>Ki</label>
              <input type="number" step="0.01" value={pidKi()} onInput={(e) => setPidKi(parseFloat(e.target.value) || 0.1)} />
            </div>
            <div class="config-row">
              <label>Kd</label>
              <input type="number" step="0.001" value={pidKd()} onInput={(e) => setPidKd(parseFloat(e.target.value) || 0.01)} />
            </div>
          </div>
          <div class="config-row-group">
            <div class="config-row">
              <label>Output Min</label>
              <input type="number" value={pidOutMin()} onInput={(e) => setPidOutMin(parseFloat(e.target.value) || -100)} />
            </div>
            <div class="config-row">
              <label>Output Max</label>
              <input type="number" value={pidOutMax()} onInput={(e) => setPidOutMax(parseFloat(e.target.value) || 100)} />
            </div>
          </div>
          <div class="config-row">
            <label>Sample Time (ms)</label>
            <input type="number" value={pidSampleTime()} onInput={(e) => setPidSampleTime(parseInt(e.target.value) || 10)} />
          </div>
          <div class="config-row checkbox">
            <label>
              <input type="checkbox" checked={pidAntiWindup()} onChange={(e) => setPidAntiWindup(e.target.checked)} />
              Anti-Windup
            </label>
          </div>
          <button class="generate-btn" onClick={generatePid} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate PID Controller"}
          </button>
        </div>
      </Show>

      {/* Buffer Config */}
      <Show when={dspType() === "buffer"}>
        <div class="config-section">
          <div class="config-row">
            <label>Buffer Name</label>
            <input type="text" value={bufName()} onInput={(e) => setBufName(e.target.value)} />
          </div>
          <div class="config-row">
            <label>Size (elements)</label>
            <input type="number" value={bufSize()} onInput={(e) => setBufSize(parseInt(e.target.value) || 1024)} />
          </div>
          <div class="config-row">
            <label>Element Type</label>
            <select value={bufType()} onChange={(e) => setBufType(e.target.value)}>
              <option value="int16_t">int16_t</option>
              <option value="int32_t">int32_t</option>
              <option value="float">float</option>
              <option value="uint8_t">uint8_t</option>
            </select>
          </div>
          <div class="config-row checkbox">
            <label>
              <input type="checkbox" checked={bufThreadSafe()} onChange={(e) => setBufThreadSafe(e.target.checked)} />
              Thread-Safe (RTOS mutex)
            </label>
          </div>
          <button class="generate-btn" onClick={generateBuffer} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate Circular Buffer"}
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
            onCopy={() => addLog("DSP", "Code copied to clipboard", "info")}
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
