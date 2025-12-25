import { createSignal, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface PowerBreakdown {
  component: string;
  current_ma: number;
  percent: number;
}

interface PowerEstimation {
  mcu: string;
  total_current_ma: number;
  power_mw: number;
  battery_life_hours: number | null;
  breakdown: PowerBreakdown[];
  recommendations: string[];
}

interface PowerPanelProps {
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function PowerPanel(props: PowerPanelProps) {
  const [selectedMcu, setSelectedMcu] = createSignal("STM32F407");
  const [peripherals, setPeripherals] = createSignal<string[]>([]);
  const [dutyCycle, setDutyCycle] = createSignal(50);
  const [batteryMah, setBatteryMah] = createSignal(1000);
  const [estimation, setEstimation] = createSignal<PowerEstimation | null>(null);
  const [isLoading, setIsLoading] = createSignal(false);

  const availablePeripherals = [
    "UART", "SPI", "I2C", "ADC", "DAC", "Timer", "DMA", "USB", "WiFi", "BLE"
  ];

  const mcuOptions = [
    "STM32F407", "STM32F103", "STM32L476", "ESP32", "nRF52832"
  ];

  const togglePeripheral = (periph: string) => {
    if (peripherals().includes(periph)) {
      setPeripherals(peripherals().filter(p => p !== periph));
    } else {
      setPeripherals([...peripherals(), periph]);
    }
  };

  const estimatePower = async () => {
    setIsLoading(true);
    try {
      const result = await invoke("power_estimate", {
        mcu: selectedMcu(),
        peripherals: peripherals(),
        dutyCycle: dutyCycle(),
        batteryMah: batteryMah(),
      }) as PowerEstimation;
      
      setEstimation(result);
      props.onLog?.("Power", `Estimated: ${result.total_current_ma.toFixed(2)}mA`, "info");
    } catch (e) {
      props.onLog?.("Power", `Estimation failed: ${e}`, "error");
    }
    setIsLoading(false);
  };

  const formatHours = (hours: number) => {
    if (hours >= 24) {
      const days = Math.floor(hours / 24);
      const remainingHours = hours % 24;
      return `${days}d ${remainingHours.toFixed(0)}h`;
    }
    return `${hours.toFixed(1)}h`;
  };

  return (
    <div class="power-panel">
      <div class="panel-header">
        <h3>⚡ Power Estimator</h3>
      </div>

      {/* MCU Selection */}
      <div class="config-section">
        <label>MCU</label>
        <select value={selectedMcu()} onChange={(e) => setSelectedMcu(e.target.value)}>
          <For each={mcuOptions}>
            {(mcu) => <option value={mcu}>{mcu}</option>}
          </For>
        </select>
      </div>

      {/* Peripherals */}
      <div class="config-section">
        <label>Active Peripherals</label>
        <div class="periph-grid">
          <For each={availablePeripherals}>
            {(periph) => (
              <button 
                class={`periph-btn ${peripherals().includes(periph) ? "active" : ""}`}
                onClick={() => togglePeripheral(periph)}
              >
                {periph}
              </button>
            )}
          </For>
        </div>
      </div>

      {/* Duty Cycle */}
      <div class="config-section">
        <label>Active Duty Cycle: {dutyCycle()}%</label>
        <input 
          type="range" 
          min="1" 
          max="100" 
          value={dutyCycle()}
          onInput={(e) => setDutyCycle(parseInt(e.target.value))}
        />
      </div>

      {/* Battery */}
      <div class="config-section">
        <label>Battery Capacity (mAh)</label>
        <input 
          type="number" 
          value={batteryMah()}
          onInput={(e) => setBatteryMah(parseInt(e.target.value) || 1000)}
        />
      </div>

      <button class="estimate-btn" onClick={estimatePower} disabled={isLoading()}>
        {isLoading() ? "Calculating..." : "Calculate Power"}
      </button>

      {/* Results */}
      <Show when={estimation()}>
        <div class="results">
          <div class="summary">
            <div class="stat">
              <span class="stat-value">{estimation()!.total_current_ma.toFixed(2)}</span>
              <span class="stat-label">mA</span>
            </div>
            <div class="stat">
              <span class="stat-value">{estimation()!.power_mw.toFixed(1)}</span>
              <span class="stat-label">mW</span>
            </div>
            <Show when={estimation()!.battery_life_hours}>
              <div class="stat life">
                <span class="stat-value">{formatHours(estimation()!.battery_life_hours!)}</span>
                <span class="stat-label">Battery Life</span>
              </div>
            </Show>
          </div>

          {/* Breakdown */}
          <div class="breakdown">
            <h4>Power Breakdown</h4>
            <For each={estimation()!.breakdown}>
              {(item) => (
                <div class="breakdown-row">
                  <span class="comp-name">{item.component}</span>
                  <div class="bar-container">
                    <div class="bar" style={{ width: `${item.percent}%` }}></div>
                  </div>
                  <span class="comp-value">{item.current_ma.toFixed(2)}mA</span>
                </div>
              )}
            </For>
          </div>

          {/* Recommendations */}
          <Show when={estimation()!.recommendations.length > 0}>
            <div class="recommendations">
              <h4>💡 Recommendations</h4>
              <For each={estimation()!.recommendations}>
                {(rec) => <p class="rec">{rec}</p>}
              </For>
            </div>
          </Show>
        </div>
      </Show>

      <style>{`
        .power-panel {
          background: var(--bg-secondary, #1a1a2e);
          border: 1px solid var(--border, #333);
          border-radius: 8px;
          padding: 12px;
        }

        .panel-header { margin-bottom: 12px; }
        .panel-header h3 { margin: 0; font-size: 14px; }

        .config-section { margin-bottom: 12px; }
        .config-section label {
          display: block;
          color: #888;
          font-size: 11px;
          margin-bottom: 6px;
        }

        .config-section select,
        .config-section input[type="number"] {
          width: 100%;
          background: rgba(255,255,255,0.05);
          border: 1px solid #333;
          color: #fff;
          padding: 8px 12px;
          border-radius: 6px;
        }

        .config-section input[type="range"] { width: 100%; }

        .periph-grid {
          display: grid;
          grid-template-columns: repeat(5, 1fr);
          gap: 4px;
        }

        .periph-btn {
          padding: 6px;
          background: rgba(255,255,255,0.05);
          border: 1px solid #333;
          color: #888;
          border-radius: 4px;
          cursor: pointer;
          font-size: 10px;
        }

        .periph-btn.active {
          background: rgba(251, 191, 36, 0.2);
          border-color: #fbbf24;
          color: #fbbf24;
        }

        .estimate-btn {
          width: 100%;
          background: linear-gradient(135deg, #fbbf24, #f59e0b);
          color: #000;
          border: none;
          padding: 10px;
          border-radius: 6px;
          cursor: pointer;
          font-weight: 600;
          margin-bottom: 12px;
        }

        .results {
          background: rgba(0,0,0,0.2);
          border-radius: 6px;
          padding: 12px;
        }

        .summary {
          display: flex;
          justify-content: space-around;
          margin-bottom: 12px;
        }

        .stat { text-align: center; }
        .stat-value {
          display: block;
          font-size: 20px;
          font-weight: 600;
          color: #fbbf24;
        }
        .stat-label { font-size: 10px; color: #888; }
        .stat.life .stat-value { color: #4ade80; }

        .breakdown h4,
        .recommendations h4 {
          margin: 0 0 8px 0;
          font-size: 12px;
          color: #888;
        }

        .breakdown-row {
          display: flex;
          align-items: center;
          gap: 8px;
          margin-bottom: 6px;
        }

        .comp-name { width: 80px; font-size: 11px; color: #ccc; }
        .bar-container {
          flex: 1;
          height: 8px;
          background: rgba(255,255,255,0.1);
          border-radius: 4px;
          overflow: hidden;
        }
        .bar {
          height: 100%;
          background: linear-gradient(90deg, #fbbf24, #f59e0b);
          border-radius: 4px;
        }
        .comp-value { width: 60px; font-size: 10px; color: #888; text-align: right; }

        .recommendations { margin-top: 12px; }
        .rec {
          font-size: 11px;
          color: #888;
          margin: 4px 0;
          padding-left: 12px;
          border-left: 2px solid #fbbf24;
        }
      `}</style>
    </div>
  );
}
