import { createSignal, Show, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface PeripheralsPanelProps {
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

// Common I2C devices for quick setup
const I2C_DEVICES = [
  { name: "MPU6050", addr: 0x68, desc: "Accelerometer/Gyroscope" },
  { name: "SSD1306", addr: 0x3C, desc: "OLED Display 128x64" },
  { name: "BMP280", addr: 0x76, desc: "Pressure/Temperature" },
  { name: "AHT20", addr: 0x38, desc: "Temperature/Humidity" },
  { name: "AT24C02", addr: 0x50, desc: "EEPROM 256 bytes" },
  { name: "PCF8574", addr: 0x20, desc: "I/O Expander" },
  { name: "TCS34725", addr: 0x29, desc: "RGB Color Sensor" },
  { name: "VL53L0X", addr: 0x29, desc: "ToF Distance Sensor" },
];

// SPI devices
const SPI_DEVICES = [
  { name: "W25Q64", desc: "Flash Memory 64Mbit" },
  { name: "MAX7219", desc: "LED Matrix Driver" },
  { name: "MCP3008", desc: "8-ch 10-bit ADC" },
  { name: "nRF24L01", desc: "2.4GHz Radio" },
  { name: "ILI9341", desc: "TFT Display 320x240" },
  { name: "SD Card", desc: "SD Card via SPI" },
];

export function PeripheralsPanel(props: PeripheralsPanelProps) {
  // Tab state
  const [activeTab, setActiveTab] = createSignal<"spi" | "i2c" | "uart">("spi");
  
  // SPI configuration
  const [spiInstance, setSpiInstance] = createSignal("SPI1");
  const [spiClock, setSpiClock] = createSignal(1000000);
  const [spiMode, setSpiMode] = createSignal(0);
  const [spiDataSize, setSpiDataSize] = createSignal(8);
  const [spiDma, setSpiDma] = createSignal(false);
  const [spiLanguage, setSpiLanguage] = createSignal("C");
  
  // I2C configuration
  const [i2cInstance, setI2cInstance] = createSignal("I2C1");
  const [i2cSpeed, setI2cSpeed] = createSignal("fast");
  const [i2cDevice, setI2cDevice] = createSignal("");
  const [i2cAddress, setI2cAddress] = createSignal(0x68);
  const [i2cLanguage, setI2cLanguage] = createSignal("C");
  
  // UART configuration
  const [uartInstance, setUartInstance] = createSignal("USART1");
  const [uartBaud, setUartBaud] = createSignal(115200);
  const [uartDataBits, setUartDataBits] = createSignal(8);
  const [uartParity, setUartParity] = createSignal("none");
  const [uartStopBits, setUartStopBits] = createSignal(1);
  const [uartFlowControl, setUartFlowControl] = createSignal(false);
  const [uartDma, setUartDma] = createSignal(false);
  const [uartLanguage, setUartLanguage] = createSignal("C");
  
  // Generated code
  const [generatedCode, setGeneratedCode] = createSignal("");
  const [isGenerating, setIsGenerating] = createSignal(false);

  const addLog = (source: string, message: string, type: "info" | "success" | "warning" | "error") => {
    props.onLog?.(source, message, type);
  };

  const generateSPI = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_spi_driver", {
        instance: spiInstance(),
        clockHz: spiClock(),
        mode: spiMode(),
        dataSize: spiDataSize(),
        dma: spiDma(),
        language: spiLanguage(),
      }) as any;
      
      let code = "";
      if (result.header_file) code += "// === HEADER FILE ===\n" + result.header_file + "\n\n";
      code += "// === SOURCE FILE ===\n" + result.source_file;
      if (result.example_file) code += "\n\n// === EXAMPLE ===\n" + result.example_file;
      
      setGeneratedCode(code);
      addLog("SPI", `Generated ${spiInstance()} driver @ ${(spiClock() / 1000000).toFixed(1)}MHz`, "success");
    } catch (e) {
      addLog("ERROR", `SPI generation failed: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generateI2C = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_i2c_driver", {
        instance: i2cInstance(),
        speed: i2cSpeed(),
        address: i2cAddress(),
        language: i2cLanguage(),
      }) as any;
      
      let code = "";
      if (result.header_file) code += "// === HEADER FILE ===\n" + result.header_file + "\n\n";
      code += "// === SOURCE FILE ===\n" + result.source_file;
      if (result.example_file) code += "\n\n// === EXAMPLE ===\n" + result.example_file;
      
      setGeneratedCode(code);
      addLog("I2C", `Generated ${i2cInstance()} driver (${i2cSpeed()})`, "success");
    } catch (e) {
      addLog("ERROR", `I2C generation failed: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generateUART = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_uart_driver", {
        instance: uartInstance(),
        baudRate: uartBaud(),
        dataBits: uartDataBits(),
        parity: uartParity(),
        stopBits: uartStopBits(),
        flowControl: uartFlowControl(),
        dma: uartDma(),
        language: uartLanguage(),
      }) as any;
      
      let code = "";
      if (result.header_file) code += "// === HEADER FILE ===\n" + result.header_file + "\n\n";
      code += "// === SOURCE FILE ===\n" + result.source_file;
      if (result.example_file) code += "\n\n// === EXAMPLE ===\n" + result.example_file;
      
      setGeneratedCode(code);
      addLog("UART", `Generated ${uartInstance()} @ ${uartBaud()} baud`, "success");
    } catch (e) {
      addLog("ERROR", `UART generation failed: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const selectI2CDevice = (device: typeof I2C_DEVICES[0]) => {
    setI2cDevice(device.name);
    setI2cAddress(device.addr);
    addLog("I2C", `Selected ${device.name} (0x${device.addr.toString(16)})`, "info");
  };

  const copyCode = () => {
    navigator.clipboard.writeText(generatedCode());
    addLog("SYSTEM", "Code copied to clipboard", "success");
  };

  return (
    <div class="peripherals-panel">
      <div class="panel-header">
        <h3>🔌 Serial Peripherals</h3>
      </div>
      
      {/* Tabs */}
      <div class="periph-tabs">
        <button 
          class={`tab ${activeTab() === "spi" ? "active" : ""}`}
          onClick={() => setActiveTab("spi")}
        >
          📡 SPI
        </button>
        <button 
          class={`tab ${activeTab() === "i2c" ? "active" : ""}`}
          onClick={() => setActiveTab("i2c")}
        >
          🔗 I²C
        </button>
        <button 
          class={`tab ${activeTab() === "uart" ? "active" : ""}`}
          onClick={() => setActiveTab("uart")}
        >
          📟 UART
        </button>
      </div>

      {/* SPI Config */}
      <Show when={activeTab() === "spi"}>
        <div class="config-section">
          <div class="config-row">
            <label>Instance</label>
            <select value={spiInstance()} onChange={(e) => setSpiInstance(e.currentTarget.value)}>
              <option value="SPI1">SPI1</option>
              <option value="SPI2">SPI2</option>
              <option value="SPI3">SPI3</option>
            </select>
          </div>
          <div class="config-row">
            <label>Clock (Hz)</label>
            <input 
              type="number" 
              value={spiClock()} 
              onInput={(e) => setSpiClock(parseInt(e.currentTarget.value))}
            />
          </div>
          <div class="config-row">
            <label>Mode</label>
            <select value={spiMode()} onChange={(e) => setSpiMode(parseInt(e.currentTarget.value))}>
              <option value={0}>Mode 0 (CPOL=0, CPHA=0)</option>
              <option value={1}>Mode 1 (CPOL=0, CPHA=1)</option>
              <option value={2}>Mode 2 (CPOL=1, CPHA=0)</option>
              <option value={3}>Mode 3 (CPOL=1, CPHA=1)</option>
            </select>
          </div>
          <div class="config-row">
            <label>Data Size</label>
            <select value={spiDataSize()} onChange={(e) => setSpiDataSize(parseInt(e.currentTarget.value))}>
              <option value={8}>8-bit</option>
              <option value={16}>16-bit</option>
            </select>
          </div>
          <div class="config-row">
            <label>DMA</label>
            <input type="checkbox" checked={spiDma()} onChange={(e) => setSpiDma(e.currentTarget.checked)} />
          </div>
          <div class="config-row">
            <label>Language</label>
            <select value={spiLanguage()} onChange={(e) => setSpiLanguage(e.currentTarget.value)}>
              <option value="C">C</option>
              <option value="Cpp">C++</option>
              <option value="Rust">Rust</option>
            </select>
          </div>
          
          <div class="device-library">
            <label>Quick Device Templates:</label>
            <div class="device-chips">
              <For each={SPI_DEVICES}>
                {(device) => (
                  <span class="device-chip" title={device.desc}>
                    {device.name}
                  </span>
                )}
              </For>
            </div>
          </div>
          
          <button class="generate-btn" onClick={generateSPI} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate SPI Driver"}
          </button>
        </div>
      </Show>

      {/* I2C Config */}
      <Show when={activeTab() === "i2c"}>
        <div class="config-section">
          <div class="config-row">
            <label>Instance</label>
            <select value={i2cInstance()} onChange={(e) => setI2cInstance(e.currentTarget.value)}>
              <option value="I2C1">I2C1</option>
              <option value="I2C2">I2C2</option>
              <option value="I2C3">I2C3</option>
            </select>
          </div>
          <div class="config-row">
            <label>Speed</label>
            <select value={i2cSpeed()} onChange={(e) => setI2cSpeed(e.currentTarget.value)}>
              <option value="standard">Standard (100kHz)</option>
              <option value="fast">Fast (400kHz)</option>
              <option value="fast_plus">Fast+ (1MHz)</option>
            </select>
          </div>
          <div class="config-row">
            <label>Address</label>
            <input 
              type="text" 
              value={`0x${i2cAddress().toString(16).toUpperCase()}`}
              onInput={(e) => {
                const val = parseInt(e.currentTarget.value, 16);
                if (!isNaN(val)) setI2cAddress(val);
              }}
            />
          </div>
          <div class="config-row">
            <label>Language</label>
            <select value={i2cLanguage()} onChange={(e) => setI2cLanguage(e.currentTarget.value)}>
              <option value="C">C</option>
              <option value="Cpp">C++</option>
              <option value="Rust">Rust</option>
            </select>
          </div>
          
          <div class="device-library">
            <label>Common I²C Devices:</label>
            <div class="device-chips">
              <For each={I2C_DEVICES}>
                {(device) => (
                  <span 
                    class={`device-chip ${i2cDevice() === device.name ? "selected" : ""}`}
                    title={`${device.desc} @ 0x${device.addr.toString(16)}`}
                    onClick={() => selectI2CDevice(device)}
                  >
                    {device.name}
                  </span>
                )}
              </For>
            </div>
          </div>
          
          <button class="generate-btn" onClick={generateI2C} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate I²C Driver"}
          </button>
        </div>
      </Show>

      {/* UART Config */}
      <Show when={activeTab() === "uart"}>
        <div class="config-section">
          <div class="config-row">
            <label>Instance</label>
            <select value={uartInstance()} onChange={(e) => setUartInstance(e.currentTarget.value)}>
              <option value="USART1">USART1</option>
              <option value="USART2">USART2</option>
              <option value="USART6">USART6</option>
              <option value="UART4">UART4</option>
              <option value="UART5">UART5</option>
            </select>
          </div>
          <div class="config-row">
            <label>Baud Rate</label>
            <select value={uartBaud()} onChange={(e) => setUartBaud(parseInt(e.currentTarget.value))}>
              <option value={9600}>9600</option>
              <option value={19200}>19200</option>
              <option value={38400}>38400</option>
              <option value={57600}>57600</option>
              <option value={115200}>115200</option>
              <option value={230400}>230400</option>
              <option value={460800}>460800</option>
              <option value={921600}>921600</option>
            </select>
          </div>
          <div class="config-row">
            <label>Data Bits</label>
            <select value={uartDataBits()} onChange={(e) => setUartDataBits(parseInt(e.currentTarget.value))}>
              <option value={8}>8</option>
              <option value={9}>9</option>
            </select>
          </div>
          <div class="config-row">
            <label>Parity</label>
            <select value={uartParity()} onChange={(e) => setUartParity(e.currentTarget.value)}>
              <option value="none">None</option>
              <option value="even">Even</option>
              <option value="odd">Odd</option>
            </select>
          </div>
          <div class="config-row">
            <label>Stop Bits</label>
            <select value={uartStopBits()} onChange={(e) => setUartStopBits(parseInt(e.currentTarget.value))}>
              <option value={1}>1</option>
              <option value={2}>2</option>
            </select>
          </div>
          <div class="config-row">
            <label>Flow Control</label>
            <input type="checkbox" checked={uartFlowControl()} onChange={(e) => setUartFlowControl(e.currentTarget.checked)} />
          </div>
          <div class="config-row">
            <label>DMA</label>
            <input type="checkbox" checked={uartDma()} onChange={(e) => setUartDma(e.currentTarget.checked)} />
          </div>
          <div class="config-row">
            <label>Language</label>
            <select value={uartLanguage()} onChange={(e) => setUartLanguage(e.currentTarget.value)}>
              <option value="C">C</option>
              <option value="Cpp">C++</option>
              <option value="Rust">Rust</option>
            </select>
          </div>
          
          <button class="generate-btn" onClick={generateUART} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate UART Driver"}
          </button>
        </div>
      </Show>

      {/* Generated Code Output */}
      <Show when={generatedCode()}>
        <div class="code-output">
          <div class="code-header">
            <span>Generated Driver</span>
            <button class="copy-btn" onClick={copyCode}>📋 Copy</button>
          </div>
          <pre><code>{generatedCode()}</code></pre>
        </div>
      </Show>
    </div>
  );
}
