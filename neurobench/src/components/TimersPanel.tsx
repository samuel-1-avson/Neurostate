import { createSignal, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface TimersPanelProps {
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function TimersPanel(props: TimersPanelProps) {
  // Tab state
  const [activeTab, setActiveTab] = createSignal<"interrupts" | "timers" | "ticker">("interrupts");
  
  // Interrupt configuration
  const [intPin, setIntPin] = createSignal("PA0");
  const [intEdge, setIntEdge] = createSignal("rising");
  const [intPriority, setIntPriority] = createSignal(5);
  const [intDebounce, setIntDebounce] = createSignal(50);
  const [intHandler, setIntHandler] = createSignal("EXTI0_IRQHandler");
  const [intCode, setIntCode] = createSignal("fsm_trigger_event(EVT_BUTTON);");
  
  // Timer configuration
  const [timerInstance, setTimerInstance] = createSignal("TIM2");
  const [timerPrescaler, setTimerPrescaler] = createSignal(8400);
  const [timerPeriod, setTimerPeriod] = createSignal(10000);
  const [timerAutoReload, setTimerAutoReload] = createSignal(true);
  const [timerInterrupt, setTimerInterrupt] = createSignal(true);
  const [timerHandler, setTimerHandler] = createSignal("TIM2_IRQHandler");
  const [timerCode, setTimerCode] = createSignal("system_tick++;");
  
  // Ticker configuration
  const [tickerName, setTickerName] = createSignal("main_ticker");
  const [tickerInterval, setTickerInterval] = createSignal(1);
  const [tickerCode, setTickerCode] = createSignal("// 1ms periodic task");
  
  // Generated code
  const [generatedCode, setGeneratedCode] = createSignal("");
  const [isGenerating, setIsGenerating] = createSignal(false);

  const addLog = (source: string, message: string, type: "info" | "success" | "warning" | "error") => {
    props.onLog?.(source, message, type);
  };

  const generateInterrupt = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_interrupt_code", {
        pin: intPin(),
        edge: intEdge(),
        priority: intPriority(),
        debounceMs: intDebounce(),
        handlerName: intHandler(),
        handlerCode: intCode(),
      }) as any;
      setGeneratedCode(result.code);
      addLog("INTERRUPT", `Generated EXTI code for ${intPin()}`, "success");
    } catch (e) {
      addLog("ERROR", `Failed: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generateTimer = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_timer_code", {
        instance: timerInstance(),
        prescaler: timerPrescaler(),
        period: timerPeriod(),
        autoReload: timerAutoReload(),
        interruptEnabled: timerInterrupt(),
        handlerName: timerHandler(),
        handlerCode: timerCode(),
        clockHz: 84000000,
      }) as any;
      setGeneratedCode(result.code);
      addLog("TIMER", `Generated ${timerInstance()} @ ${result.frequency_hz.toFixed(2)}Hz`, "success");
    } catch (e) {
      addLog("ERROR", `Failed: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generateTicker = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_ticker_code", {
        name: tickerName(),
        intervalMs: tickerInterval(),
        callbackCode: tickerCode(),
      }) as any;
      setGeneratedCode(result.code);
      addLog("TICKER", `Generated ${tickerName()} @ ${tickerInterval()}ms`, "success");
    } catch (e) {
      addLog("ERROR", `Failed: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const copyCode = () => {
    navigator.clipboard.writeText(generatedCode());
    addLog("SYSTEM", "Code copied to clipboard", "success");
  };

  return (
    <div class="timers-panel">
      <div class="timers-header">
        <h3>⏱️ Timers & Interrupts</h3>
      </div>
      
      {/* Tabs */}
      <div class="timers-tabs">
        <button 
          class={`tab ${activeTab() === "interrupts" ? "active" : ""}`}
          onClick={() => setActiveTab("interrupts")}
        >
          ⚡ Interrupts
        </button>
        <button 
          class={`tab ${activeTab() === "timers" ? "active" : ""}`}
          onClick={() => setActiveTab("timers")}
        >
          ⏲️ Timers
        </button>
        <button 
          class={`tab ${activeTab() === "ticker" ? "active" : ""}`}
          onClick={() => setActiveTab("ticker")}
        >
          🔄 Ticker
        </button>
      </div>

      {/* Interrupt Config */}
      <Show when={activeTab() === "interrupts"}>
        <div class="config-section">
          <div class="config-row">
            <label>Pin</label>
            <input 
              type="text" 
              value={intPin()} 
              onInput={(e) => setIntPin(e.currentTarget.value)}
              placeholder="PA0"
            />
          </div>
          <div class="config-row">
            <label>Edge</label>
            <select value={intEdge()} onChange={(e) => setIntEdge(e.currentTarget.value)}>
              <option value="rising">Rising ↑</option>
              <option value="falling">Falling ↓</option>
              <option value="both">Both ↕</option>
            </select>
          </div>
          <div class="config-row">
            <label>Priority</label>
            <input 
              type="number" 
              min="0" 
              max="15" 
              value={intPriority()} 
              onInput={(e) => setIntPriority(parseInt(e.currentTarget.value))}
            />
          </div>
          <div class="config-row">
            <label>Debounce (ms)</label>
            <input 
              type="number" 
              min="0" 
              value={intDebounce()} 
              onInput={(e) => setIntDebounce(parseInt(e.currentTarget.value))}
            />
          </div>
          <div class="config-row">
            <label>Handler Name</label>
            <input 
              type="text" 
              value={intHandler()} 
              onInput={(e) => setIntHandler(e.currentTarget.value)}
            />
          </div>
          <div class="config-row full">
            <label>Handler Code</label>
            <textarea 
              value={intCode()} 
              onInput={(e) => setIntCode(e.currentTarget.value)}
              rows={3}
            />
          </div>
          <button class="generate-btn" onClick={generateInterrupt} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate Interrupt Code"}
          </button>
        </div>
      </Show>

      {/* Timer Config */}
      <Show when={activeTab() === "timers"}>
        <div class="config-section">
          <div class="config-row">
            <label>Timer</label>
            <select value={timerInstance()} onChange={(e) => setTimerInstance(e.currentTarget.value)}>
              <option value="TIM1">TIM1</option>
              <option value="TIM2">TIM2</option>
              <option value="TIM3">TIM3</option>
              <option value="TIM4">TIM4</option>
              <option value="TIM5">TIM5</option>
            </select>
          </div>
          <div class="config-row">
            <label>Prescaler</label>
            <input 
              type="number" 
              value={timerPrescaler()} 
              onInput={(e) => setTimerPrescaler(parseInt(e.currentTarget.value))}
            />
          </div>
          <div class="config-row">
            <label>Period</label>
            <input 
              type="number" 
              value={timerPeriod()} 
              onInput={(e) => setTimerPeriod(parseInt(e.currentTarget.value))}
            />
          </div>
          <div class="config-row">
            <label>Frequency</label>
            <span class="calc-value">
              {(84000000 / (timerPrescaler() * timerPeriod())).toFixed(2)} Hz
            </span>
          </div>
          <div class="config-row">
            <label>Auto-Reload</label>
            <input 
              type="checkbox" 
              checked={timerAutoReload()} 
              onChange={(e) => setTimerAutoReload(e.currentTarget.checked)}
            />
          </div>
          <div class="config-row">
            <label>Interrupt</label>
            <input 
              type="checkbox" 
              checked={timerInterrupt()} 
              onChange={(e) => setTimerInterrupt(e.currentTarget.checked)}
            />
          </div>
          <div class="config-row full">
            <label>Handler Code</label>
            <textarea 
              value={timerCode()} 
              onInput={(e) => setTimerCode(e.currentTarget.value)}
              rows={3}
            />
          </div>
          <button class="generate-btn" onClick={generateTimer} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate Timer Code"}
          </button>
        </div>
      </Show>

      {/* Ticker Config */}
      <Show when={activeTab() === "ticker"}>
        <div class="config-section">
          <div class="config-row">
            <label>Name</label>
            <input 
              type="text" 
              value={tickerName()} 
              onInput={(e) => setTickerName(e.currentTarget.value)}
            />
          </div>
          <div class="config-row">
            <label>Interval (ms)</label>
            <input 
              type="number" 
              min="1" 
              value={tickerInterval()} 
              onInput={(e) => setTickerInterval(parseInt(e.currentTarget.value))}
            />
          </div>
          <div class="config-row full">
            <label>Callback Code</label>
            <textarea 
              value={tickerCode()} 
              onInput={(e) => setTickerCode(e.currentTarget.value)}
              rows={4}
            />
          </div>
          <button class="generate-btn" onClick={generateTicker} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate Ticker Code"}
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
