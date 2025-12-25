import { createSignal, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface RTOSPanelProps {
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function RTOSPanel(props: RTOSPanelProps) {
  // RTOS selection
  const [rtos, setRtos] = createSignal<"freertos" | "zephyr">("freertos");
  
  // Tab selection
  const [activeTab, setActiveTab] = createSignal<"task" | "semaphore" | "mutex" | "queue" | "timer">("task");
  
  // Task config
  const [taskName, setTaskName] = createSignal("Task1");
  const [taskStack, setTaskStack] = createSignal(1024);
  const [taskPriority, setTaskPriority] = createSignal("normal");
  const [taskEntry, setTaskEntry] = createSignal("vTask1");
  const [taskAutoStart, setTaskAutoStart] = createSignal(true);
  
  // Semaphore config
  const [semName, setSemName] = createSignal("Sem1");
  const [semType, setSemType] = createSignal<"binary" | "counting">("binary");
  const [semMaxCount, setSemMaxCount] = createSignal(10);
  const [semInitCount, setSemInitCount] = createSignal(0);
  
  // Mutex config
  const [mutexName, setMutexName] = createSignal("Mutex1");
  const [mutexRecursive, setMutexRecursive] = createSignal(false);
  
  // Queue config
  const [queueName, setQueueName] = createSignal("Queue1");
  const [queueLength, setQueueLength] = createSignal(10);
  const [queueItemSize, setQueueItemSize] = createSignal(4);
  
  // Timer config
  const [timerName, setTimerName] = createSignal("Timer1");
  const [timerPeriod, setTimerPeriod] = createSignal(1000);
  const [timerAutoReload, setTimerAutoReload] = createSignal(true);
  const [timerCallback, setTimerCallback] = createSignal("vTimerCallback");
  
  // Generated code
  const [generatedCode, setGeneratedCode] = createSignal("");
  const [isGenerating, setIsGenerating] = createSignal(false);

  const addLog = (source: string, message: string, type: "info" | "success" | "warning" | "error" = "info") => {
    props.onLog?.(source, message, type);
  };

  const generateTask = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_rtos_task", {
        rtos: rtos(),
        name: taskName(),
        stackSize: taskStack(),
        priority: taskPriority(),
        entryFunction: taskEntry(),
        autoStart: taskAutoStart(),
      }) as any;
      setGeneratedCode(result.code);
      addLog("RTOS", `Generated ${rtos().toUpperCase()} task: ${taskName()}`, "success");
    } catch (e) {
      addLog("ERROR", `Failed to generate task: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generateSemaphore = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_rtos_semaphore", {
        rtos: rtos(),
        name: semName(),
        semType: semType(),
        maxCount: semMaxCount(),
        initialCount: semInitCount(),
      }) as any;
      setGeneratedCode(result.code);
      addLog("RTOS", `Generated ${rtos().toUpperCase()} semaphore: ${semName()}`, "success");
    } catch (e) {
      addLog("ERROR", `Failed to generate semaphore: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generateMutex = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_rtos_mutex", {
        rtos: rtos(),
        name: mutexName(),
        recursive: mutexRecursive(),
      }) as any;
      setGeneratedCode(result.code);
      addLog("RTOS", `Generated ${rtos().toUpperCase()} mutex: ${mutexName()}`, "success");
    } catch (e) {
      addLog("ERROR", `Failed to generate mutex: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generateQueue = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_rtos_queue", {
        rtos: rtos(),
        name: queueName(),
        length: queueLength(),
        itemSize: queueItemSize(),
      }) as any;
      setGeneratedCode(result.code);
      addLog("RTOS", `Generated ${rtos().toUpperCase()} queue: ${queueName()}`, "success");
    } catch (e) {
      addLog("ERROR", `Failed to generate queue: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generateTimer = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_rtos_timer", {
        rtos: rtos(),
        name: timerName(),
        periodMs: timerPeriod(),
        autoReload: timerAutoReload(),
        callback: timerCallback(),
      }) as any;
      setGeneratedCode(result.code);
      addLog("RTOS", `Generated ${rtos().toUpperCase()} timer: ${timerName()}`, "success");
    } catch (e) {
      addLog("ERROR", `Failed to generate timer: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generateConfig = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_rtos_config", {
        rtos: rtos(),
      }) as any;
      setGeneratedCode(result.code);
      addLog("RTOS", `Generated ${rtos().toUpperCase()} config: ${result.filename}`, "success");
    } catch (e) {
      addLog("ERROR", `Failed to generate config: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const copyToClipboard = () => {
    navigator.clipboard.writeText(generatedCode());
    addLog("RTOS", "Code copied to clipboard", "info");
  };

  return (
    <div class="rtos-panel">
      <div class="rtos-header">
        <h3>🔄 RTOS Configuration</h3>
        <div class="rtos-selector">
          <button 
            class={`rtos-btn ${rtos() === "freertos" ? "active" : ""}`}
            onClick={() => setRtos("freertos")}
          >
            FreeRTOS
          </button>
          <button 
            class={`rtos-btn ${rtos() === "zephyr" ? "active" : ""}`}
            onClick={() => setRtos("zephyr")}
          >
            Zephyr
          </button>
        </div>
      </div>

      {/* Tabs */}
      <div class="rtos-tabs">
        <button class={`tab ${activeTab() === "task" ? "active" : ""}`} onClick={() => setActiveTab("task")}>
          📋 Tasks
        </button>
        <button class={`tab ${activeTab() === "semaphore" ? "active" : ""}`} onClick={() => setActiveTab("semaphore")}>
          🚦 Semaphore
        </button>
        <button class={`tab ${activeTab() === "mutex" ? "active" : ""}`} onClick={() => setActiveTab("mutex")}>
          🔒 Mutex
        </button>
        <button class={`tab ${activeTab() === "queue" ? "active" : ""}`} onClick={() => setActiveTab("queue")}>
          📬 Queue
        </button>
        <button class={`tab ${activeTab() === "timer" ? "active" : ""}`} onClick={() => setActiveTab("timer")}>
          ⏱️ Timer
        </button>
      </div>

      {/* Task Config */}
      <Show when={activeTab() === "task"}>
        <div class="config-section">
          <div class="config-row">
            <label>Task Name</label>
            <input type="text" value={taskName()} onInput={(e) => setTaskName(e.target.value)} />
          </div>
          <div class="config-row">
            <label>Stack Size (bytes)</label>
            <input type="number" value={taskStack()} onInput={(e) => setTaskStack(parseInt(e.target.value) || 1024)} />
          </div>
          <div class="config-row">
            <label>Priority</label>
            <select value={taskPriority()} onChange={(e) => setTaskPriority(e.target.value)}>
              <option value="idle">Idle</option>
              <option value="low">Low</option>
              <option value="normal">Normal</option>
              <option value="high">High</option>
              <option value="realtime">Realtime</option>
            </select>
          </div>
          <div class="config-row">
            <label>Entry Function</label>
            <input type="text" value={taskEntry()} onInput={(e) => setTaskEntry(e.target.value)} />
          </div>
          <div class="config-row checkbox">
            <label>
              <input type="checkbox" checked={taskAutoStart()} onChange={(e) => setTaskAutoStart(e.target.checked)} />
              Auto Start
            </label>
          </div>
          <button class="generate-btn" onClick={generateTask} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate Task"}
          </button>
        </div>
      </Show>

      {/* Semaphore Config */}
      <Show when={activeTab() === "semaphore"}>
        <div class="config-section">
          <div class="config-row">
            <label>Semaphore Name</label>
            <input type="text" value={semName()} onInput={(e) => setSemName(e.target.value)} />
          </div>
          <div class="config-row">
            <label>Type</label>
            <select value={semType()} onChange={(e) => setSemType(e.target.value as any)}>
              <option value="binary">Binary</option>
              <option value="counting">Counting</option>
            </select>
          </div>
          <Show when={semType() === "counting"}>
            <div class="config-row">
              <label>Max Count</label>
              <input type="number" value={semMaxCount()} onInput={(e) => setSemMaxCount(parseInt(e.target.value) || 10)} />
            </div>
          </Show>
          <div class="config-row">
            <label>Initial Count</label>
            <input type="number" value={semInitCount()} onInput={(e) => setSemInitCount(parseInt(e.target.value) || 0)} />
          </div>
          <button class="generate-btn" onClick={generateSemaphore} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate Semaphore"}
          </button>
        </div>
      </Show>

      {/* Mutex Config */}
      <Show when={activeTab() === "mutex"}>
        <div class="config-section">
          <div class="config-row">
            <label>Mutex Name</label>
            <input type="text" value={mutexName()} onInput={(e) => setMutexName(e.target.value)} />
          </div>
          <div class="config-row checkbox">
            <label>
              <input type="checkbox" checked={mutexRecursive()} onChange={(e) => setMutexRecursive(e.target.checked)} />
              Recursive Mutex
            </label>
          </div>
          <button class="generate-btn" onClick={generateMutex} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate Mutex"}
          </button>
        </div>
      </Show>

      {/* Queue Config */}
      <Show when={activeTab() === "queue"}>
        <div class="config-section">
          <div class="config-row">
            <label>Queue Name</label>
            <input type="text" value={queueName()} onInput={(e) => setQueueName(e.target.value)} />
          </div>
          <div class="config-row">
            <label>Length</label>
            <input type="number" value={queueLength()} onInput={(e) => setQueueLength(parseInt(e.target.value) || 10)} />
          </div>
          <div class="config-row">
            <label>Item Size (bytes)</label>
            <input type="number" value={queueItemSize()} onInput={(e) => setQueueItemSize(parseInt(e.target.value) || 4)} />
          </div>
          <button class="generate-btn" onClick={generateQueue} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate Queue"}
          </button>
        </div>
      </Show>

      {/* Timer Config */}
      <Show when={activeTab() === "timer"}>
        <div class="config-section">
          <div class="config-row">
            <label>Timer Name</label>
            <input type="text" value={timerName()} onInput={(e) => setTimerName(e.target.value)} />
          </div>
          <div class="config-row">
            <label>Period (ms)</label>
            <input type="number" value={timerPeriod()} onInput={(e) => setTimerPeriod(parseInt(e.target.value) || 1000)} />
          </div>
          <div class="config-row">
            <label>Callback Function</label>
            <input type="text" value={timerCallback()} onInput={(e) => setTimerCallback(e.target.value)} />
          </div>
          <div class="config-row checkbox">
            <label>
              <input type="checkbox" checked={timerAutoReload()} onChange={(e) => setTimerAutoReload(e.target.checked)} />
              Auto Reload (Periodic)
            </label>
          </div>
          <button class="generate-btn" onClick={generateTimer} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate Timer"}
          </button>
        </div>
      </Show>

      {/* Config File Button */}
      <div class="config-actions">
        <button class="config-btn" onClick={generateConfig}>
          📄 Generate {rtos() === "freertos" ? "FreeRTOSConfig.h" : "prj.conf"}
        </button>
      </div>

      {/* Generated Code */}
      <Show when={generatedCode()}>
        <div class="code-output">
          <div class="code-header">
            <span>Generated Code</span>
            <button class="copy-btn" onClick={copyToClipboard}>📋 Copy</button>
          </div>
          <pre class="code-block">{generatedCode()}</pre>
        </div>
      </Show>
    </div>
  );
}
