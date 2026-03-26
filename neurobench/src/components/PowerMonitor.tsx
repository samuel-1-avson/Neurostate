<<<<<<< HEAD
import { createSignal, For, Show, createEffect, onMount, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
=======
import { createSignal, Show, onMount, onCleanup } from "solid-js";
import { Icons } from "./AppIcons";
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
import "./PowerMonitor.css";

interface PowerMonitorProps {
  simulationActive: boolean;
  mcuType: string;
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

interface PowerReading {
  timestamp: number;
  current_ua: number;
  power_uw: number;
}

export function PowerMonitor(props: PowerMonitorProps) {
  const [powerMode, setPowerMode] = createSignal<"run" | "sleep" | "stop" | "standby">("run");
  const [currentDraw, setCurrentDraw] = createSignal<number>(0);
  const [powerConsumption, setPowerConsumption] = createSignal<number>(0);
  const [batteryLife, setBatteryLife] = createSignal<number | null>(null);
  const [batteryCapacity, setBatteryCapacity] = createSignal<number>(1000); // mAh
  const [clockFrequency, setClockFrequency] = createSignal<number>(168); // MHz
  
  // Power history for graph
  const [powerHistory, setPowerHistory] = createSignal<PowerReading[]>([]);
  const maxHistorySize = 60;
  
  // Energy accumulator
  const [totalEnergy, setTotalEnergy] = createSignal<number>(0);
  
  let updateInterval: number | undefined;

  // Estimate power based on mode and clock
  const estimatePower = () => {
    const baseCurrentByMcu: Record<string, number> = {
      "STM32F407": 250,
      "STM32F401": 200,
      "STM32F411": 220,
      "ESP32": 300,
      "nRF52840": 50,
      "RP2040": 100,
      "LPC55S69": 150,
    };
    
    const baseCurrent = baseCurrentByMcu[props.mcuType] ?? 200;
    const clockMhz = clockFrequency();
    
    let modeFactor = 1.0;
    switch (powerMode()) {
      case "run": modeFactor = 1.0; break;
      case "sleep": modeFactor = 0.1; break;
      case "stop": modeFactor = 0.01; break;
      case "standby": modeFactor = 0.0001; break;
    }
    
    const current = baseCurrent * (clockMhz / 168) * modeFactor * 1000; // in µA
    setCurrentDraw(Math.round(current));
    setPowerConsumption(Math.round(current * 3.3)); // in µW
    
    // Battery life
    const currentMa = current / 1000;
    if (currentMa > 0) {
      setBatteryLife(batteryCapacity() / currentMa);
    }
    
    // Add to history
    const reading: PowerReading = {
      timestamp: Date.now(),
      current_ua: current,
      power_uw: current * 3.3,
    };
    
    setPowerHistory(prev => {
      const updated = [...prev, reading];
      if (updated.length > maxHistorySize) {
        updated.shift();
      }
      return updated;
    });
    
    // Accumulate energy (assuming 1 second interval)
    setTotalEnergy(prev => prev + (current * 3.3 / 1000000)); // mJ
  };

  onMount(() => {
    // Update power estimate every second
    updateInterval = window.setInterval(estimatePower, 1000);
    estimatePower();
  });

  onCleanup(() => {
    if (updateInterval) {
      clearInterval(updateInterval);
    }
  });

  // Format battery life
  const formatBatteryLife = (hours: number | null) => {
    if (hours === null) return "N/A";
    if (hours > 24 * 365) return ">1 year";
    if (hours > 24 * 30) return `${Math.round(hours / (24 * 30))} months`;
    if (hours > 24) return `${Math.round(hours / 24)} days`;
    return `${Math.round(hours)} hours`;
  };

  // Get max power in history for scaling
  const maxPower = () => {
    const readings = powerHistory();
    if (readings.length === 0) return 1000000;
    return Math.max(...readings.map(r => r.power_uw), 1000);
  };

  return (
    <div class="power-monitor">
      <div class="monitor-header">
<<<<<<< HEAD
        <h3>⚡ Power Monitor</h3>
=======
        <h3><span class="header-icon">{Icons.flash()}</span> Power Monitor</h3>
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
        <span class="mcu-label">{props.mcuType}</span>
      </div>

      {/* Power Mode Selection */}
      <div class="power-modes">
        <label>Power Mode:</label>
        <div class="mode-buttons">
          <button 
            class={`mode-btn ${powerMode() === "run" ? "active" : ""}`}
            onClick={() => setPowerMode("run")}
          >
            Run
          </button>
          <button 
            class={`mode-btn ${powerMode() === "sleep" ? "active" : ""}`}
            onClick={() => setPowerMode("sleep")}
          >
            Sleep
          </button>
          <button 
            class={`mode-btn ${powerMode() === "stop" ? "active" : ""}`}
            onClick={() => setPowerMode("stop")}
          >
            Stop
          </button>
          <button 
            class={`mode-btn ${powerMode() === "standby" ? "active" : ""}`}
            onClick={() => setPowerMode("standby")}
          >
            Standby
          </button>
        </div>
      </div>

      {/* Clock Frequency */}
      <div class="clock-setting">
        <label>Clock: {clockFrequency()} MHz</label>
        <input
          type="range"
          min="1"
          max="240"
          value={clockFrequency()}
          onInput={(e) => setClockFrequency(parseInt(e.target.value))}
        />
      </div>

      {/* Battery Capacity */}
      <div class="battery-setting">
        <label>Battery: {batteryCapacity()} mAh</label>
        <input
          type="range"
          min="50"
          max="5000"
          step="50"
          value={batteryCapacity()}
          onInput={(e) => setBatteryCapacity(parseInt(e.target.value))}
        />
      </div>

      {/* Stats Cards */}
      <div class="power-stats">
        <div class="stat-card current">
<<<<<<< HEAD
          <span class="stat-icon">🔌</span>
=======
          <span class="stat-icon">{Icons.plug()}</span>
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
          <div class="stat-info">
            <span class="stat-value">{(currentDraw() / 1000).toFixed(2)}</span>
            <span class="stat-unit">mA</span>
          </div>
          <span class="stat-label">Current</span>
        </div>
        
        <div class="stat-card power">
<<<<<<< HEAD
          <span class="stat-icon">💡</span>
=======
          <span class="stat-icon">{Icons.lightbulb()}</span>
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
          <div class="stat-info">
            <span class="stat-value">{(powerConsumption() / 1000).toFixed(2)}</span>
            <span class="stat-unit">mW</span>
          </div>
          <span class="stat-label">Power</span>
        </div>
        
        <div class="stat-card battery">
<<<<<<< HEAD
          <span class="stat-icon">🔋</span>
=======
          <span class="stat-icon">{Icons.battery()}</span>
>>>>>>> 3e03cd4273f400c8cf131df2c0e623136fc8ea8b
          <div class="stat-info">
            <span class="stat-value">{formatBatteryLife(batteryLife())}</span>
          </div>
          <span class="stat-label">Battery Life</span>
        </div>
      </div>

      {/* Power Graph */}
      <div class="power-graph">
        <div class="graph-header">
          <span>Power Over Time</span>
          <span class="energy-total">Total: {totalEnergy().toFixed(3)} mJ</span>
        </div>
        <div class="graph-container">
          <svg viewBox="0 0 300 80" preserveAspectRatio="none">
            <Show when={powerHistory().length > 1}>
              <polyline
                fill="none"
                stroke="url(#powerGradient)"
                stroke-width="2"
                points={powerHistory().map((r, i) => 
                  `${(i / (maxHistorySize - 1)) * 300},${80 - (r.power_uw / maxPower()) * 70}`
                ).join(' ')}
              />
              <defs>
                <linearGradient id="powerGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                  <stop offset="0%" stop-color="#3b82f6" />
                  <stop offset="100%" stop-color="#8b5cf6" />
                </linearGradient>
              </defs>
            </Show>
          </svg>
          <div class="graph-labels">
            <span>0</span>
            <span>{(maxPower() / 1000).toFixed(0)} mW</span>
          </div>
        </div>
      </div>

      {/* Power Breakdown */}
      <div class="power-breakdown">
        <h4>Estimated Breakdown</h4>
        <div class="breakdown-row">
          <span class="component">CPU Core</span>
          <div class="bar-container">
            <div class="bar" style={{ width: `${powerMode() === 'run' ? 60 : 10}%` }} />
          </div>
          <span class="percent">{powerMode() === 'run' ? 60 : 10}%</span>
        </div>
        <div class="breakdown-row">
          <span class="component">Peripherals</span>
          <div class="bar-container">
            <div class="bar" style={{ width: "20%" }} />
          </div>
          <span class="percent">20%</span>
        </div>
        <div class="breakdown-row">
          <span class="component">Flash/RAM</span>
          <div class="bar-container">
            <div class="bar" style={{ width: `${powerMode() === 'standby' ? 0 : 15}%` }} />
          </div>
          <span class="percent">{powerMode() === 'standby' ? 0 : 15}%</span>
        </div>
        <div class="breakdown-row">
          <span class="component">Leakage</span>
          <div class="bar-container">
            <div class="bar" style={{ width: "5%" }} />
          </div>
          <span class="percent">5%</span>
        </div>
      </div>
    </div>
  );
}
