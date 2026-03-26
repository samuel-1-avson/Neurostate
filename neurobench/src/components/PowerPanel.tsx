import { createSignal, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { Icons } from "./AppIcons";
import "./PowerPanel.css";

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
        <h3><span class="header-icon">{Icons.flash()}</span> Power Estimator</h3>
      </div>

      <div class="power-content">
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
                <h4><span class="header-icon">{Icons.lightbulb()}</span> Recommendations</h4>
                <For each={estimation()!.recommendations}>
                  {(rec) => <p class="rec">{rec}</p>}
                </For>
              </div>
            </Show>
          </div>
        </Show>
      </div>

    </div>
  );
}
