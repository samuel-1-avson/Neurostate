// Visual MCU Pin Diagram Component
// Interactive pin configurator with STM32F401 BlackPill layout

import { createSignal, For, Show, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

// Pin function color mapping
const PIN_COLORS: Record<string, string> = {
  GPIO: "#4CAF50",
  UART_TX: "#2196F3",
  UART_RX: "#03A9F4",
  SPI_MOSI: "#9C27B0",
  SPI_MISO: "#AB47BC",
  SPI_SCK: "#7B1FA2",
  SPI_CS: "#6A1B9A",
  I2C_SDA: "#FF9800",
  I2C_SCL: "#FFA726",
  ADC: "#F44336",
  DAC: "#E91E63",
  PWM: "#00BCD4",
  CAN_TX: "#795548",
  CAN_RX: "#8D6E63",
  Timer: "#607D8B",
  EXTI: "#9E9E9E",
  USB_DM: "#673AB7",
  USB_DP: "#7E57C2",
  Power: "#F44336",
  Ground: "#212121",
  Reset: "#FF5722",
  Boot: "#FFEB3B",
  Clock: "#00E676",
};

interface McuPin {
  port: string;
  pin: number;
  name: string;
  functions: string[];
  currentFunction: string | null;
  x: number;
  y: number;
}

interface McuPinout {
  mcu_id: string;
  name: string;
  package: string;
  pins: McuPin[];
}

interface PinDiagramProps {
  mcuId?: string;
  onPinSelect?: (pin: McuPin, func: string) => void;
  onPinConfig?: (configs: Record<string, string>) => void;
}

export function PinDiagram(props: PinDiagramProps) {
  const [pinout, setPinout] = createSignal<McuPinout | null>(null);
  const [selectedPin, setSelectedPin] = createSignal<McuPin | null>(null);
  const [pinConfigs, setPinConfigs] = createSignal<Record<string, string>>({});
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);

  // Load MCU pinout on mount
  onMount(async () => {
    await loadPinout(props.mcuId || "STM32F401");
  });

  const loadPinout = async (mcuId: string) => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke("get_mcu_pinout", { mcuId }) as McuPinout;
      setPinout(data);
    } catch (e) {
      setError(`Failed to load pinout: ${e}`);
    }
    setLoading(false);
  };

  const handlePinClick = (pin: McuPin) => {
    setSelectedPin(pin);
  };

  const handleFunctionSelect = (pin: McuPin, func: string) => {
    const newConfigs = { ...pinConfigs(), [pin.name]: func };
    setPinConfigs(newConfigs);
    props.onPinSelect?.(pin, func);
    props.onPinConfig?.(newConfigs);
    setSelectedPin(null);
  };

  const getPinColor = (pin: McuPin): string => {
    const configuredFunc = pinConfigs()[pin.name];
    if (configuredFunc) {
      return PIN_COLORS[configuredFunc] || "#666";
    }
    if (pin.currentFunction) {
      return PIN_COLORS[pin.currentFunction] || "#666";
    }
    return "#444";
  };

  const getPinLabel = (pin: McuPin): string => {
    const configuredFunc = pinConfigs()[pin.name];
    if (configuredFunc && configuredFunc !== "GPIO") {
      return configuredFunc.replace("_", "");
    }
    return pin.name;
  };

  return (
    <div class="pin-diagram">
      <Show when={loading()}>
        <div class="pin-diagram-loading">Loading MCU pinout...</div>
      </Show>
      
      <Show when={error()}>
        <div class="pin-diagram-error">{error()}</div>
      </Show>
      
      <Show when={pinout() && !loading()}>
        <div class="pin-diagram-header">
          <span class="pin-diagram-mcu-name">{pinout()?.name}</span>
          <span class="pin-diagram-package">{pinout()?.package}</span>
        </div>
        
        <div class="pin-diagram-container">
          {/* MCU Chip Body */}
          <div class="mcu-chip">
            <div class="mcu-chip-label">{pinout()?.mcu_id}</div>
            
            {/* Left side pins */}
            <div class="pin-column left">
              <For each={pinout()?.pins.filter(p => p.x === 0)}>
                {(pin) => (
                  <div 
                    class={`pin ${selectedPin()?.name === pin.name ? "selected" : ""}`}
                    style={{ "background-color": getPinColor(pin) }}
                    onClick={() => handlePinClick(pin)}
                    title={`${pin.name}\nFunctions: ${pin.functions.join(", ")}`}
                  >
                    <span class="pin-label">{getPinLabel(pin)}</span>
                  </div>
                )}
              </For>
            </div>
            
            {/* Right side pins */}
            <div class="pin-column right">
              <For each={pinout()?.pins.filter(p => p.x === 1)}>
                {(pin) => (
                  <div 
                    class={`pin ${selectedPin()?.name === pin.name ? "selected" : ""}`}
                    style={{ "background-color": getPinColor(pin) }}
                    onClick={() => handlePinClick(pin)}
                    title={`${pin.name}\nFunctions: ${pin.functions.join(", ")}`}
                  >
                    <span class="pin-label">{getPinLabel(pin)}</span>
                  </div>
                )}
              </For>
            </div>
          </div>
        </div>
        
        {/* Pin Function Selector Popup */}
        <Show when={selectedPin()}>
          <div class="pin-function-popup">
            <div class="pin-function-header">
              <span>{selectedPin()?.name}</span>
              <button class="close-btn" onClick={() => setSelectedPin(null)}>×</button>
            </div>
            <div class="pin-function-list">
              <For each={selectedPin()?.functions}>
                {(func) => (
                  <button 
                    class="pin-function-btn"
                    style={{ "border-left": `4px solid ${PIN_COLORS[func] || "#666"}` }}
                    onClick={() => handleFunctionSelect(selectedPin()!, func)}
                  >
                    {func.replace("_", " ")}
                  </button>
                )}
              </For>
            </div>
          </div>
        </Show>
        
        {/* Legend */}
        <div class="pin-legend">
          <div class="legend-title">Pin Functions</div>
          <div class="legend-items">
            <div class="legend-item"><span class="legend-color" style={{ background: PIN_COLORS.GPIO }} /> GPIO</div>
            <div class="legend-item"><span class="legend-color" style={{ background: PIN_COLORS.UART_TX }} /> UART</div>
            <div class="legend-item"><span class="legend-color" style={{ background: PIN_COLORS.SPI_MOSI }} /> SPI</div>
            <div class="legend-item"><span class="legend-color" style={{ background: PIN_COLORS.I2C_SDA }} /> I2C</div>
            <div class="legend-item"><span class="legend-color" style={{ background: PIN_COLORS.ADC }} /> ADC</div>
            <div class="legend-item"><span class="legend-color" style={{ background: PIN_COLORS.PWM }} /> PWM</div>
            <div class="legend-item"><span class="legend-color" style={{ background: PIN_COLORS.CAN_TX }} /> CAN</div>
          </div>
        </div>
        
        {/* Configured Pins Summary */}
        <Show when={Object.keys(pinConfigs()).length > 0}>
          <div class="pin-config-summary">
            <div class="summary-title">Configured Pins</div>
            <For each={Object.entries(pinConfigs())}>
              {([pinName, func]) => (
                <div class="config-item">
                  <span class="config-pin">{pinName}</span>
                  <span class="config-arrow">→</span>
                  <span class="config-func" style={{ color: PIN_COLORS[func] }}>{func}</span>
                </div>
              )}
            </For>
          </div>
        </Show>
      </Show>
    </div>
  );
}

export default PinDiagram;
