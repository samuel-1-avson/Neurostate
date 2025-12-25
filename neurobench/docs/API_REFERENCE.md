# NeuroBench API Reference

## Tauri IPC Commands

All commands are invoked from the frontend using `invoke()` from `@tauri-apps/api/core`.

---

## Core Commands

### Project Management

```typescript
// Create a new project
invoke("create_project", { name: string, description?: string })
  -> Result<ProjectInfo, string>

// Save current project
invoke("save_project", { project: ProjectData })
  -> Result<(), string>

// Load project from file
invoke("load_project", { path: string })
  -> Result<ProjectData, string>

// List recent projects
invoke("list_projects")
  -> Result<ProjectInfo[], string>
```

### FSM Operations

```typescript
// Add a node to the FSM
invoke("add_node", {
  label: string,
  node_type: "input" | "output" | "process" | "decision",
  x: number,
  y: number
}) -> Result<FSMNode, string>

// Remove a node
invoke("remove_node", { node_id: string })
  -> Result<boolean, string>

// Add an edge (transition)
invoke("add_edge", {
  source_id: string,
  target_id: string,
  label?: string,
  guard?: string
}) -> Result<FSMEdge, string>

// Simulation
invoke("simulate_step") -> Result<SimulationResult, string>
invoke("simulate_run") -> Result<SimulationStatus, string>
invoke("simulate_stop") -> Result<SimulationStatus, string>
```

---

## Driver Generation

### GPIO

```typescript
invoke("generate_gpio_driver", {
  port: string,      // "A" - "K"
  pin: number,       // 0 - 15
  mode: string,      // "input", "output", "alternate", "analog"
  pull: string,      // "none", "up", "down"
  speed: string,     // "low", "medium", "high", "very_high"
  language: string   // "C", "Cpp", "Rust"
}) -> Result<{header: string, source: string}, string>
```

### UART

```typescript
invoke("generate_uart_driver", {
  instance: number,   // 1, 2, 3...
  baudRate: number,   // 9600, 115200, etc.
  dataBits: number,   // 7, 8, 9
  parity: string,     // "none", "even", "odd"
  stopBits: string,   // "one", "one_half", "two"
  useDma: boolean,
  useInterrupt: boolean,
  language: string
}) -> Result<{header: string, source: string}, string>
```

### SPI

```typescript
invoke("generate_spi_driver", {
  instance: string,   // "SPI1", "SPI2"
  mode: string,       // "master", "slave"
  clockHz: number,
  cpol: number,       // 0 or 1
  cpha: number,       // 0 or 1
  language: string
}) -> Result<{header: string, source: string}, string>
```

### I2C

```typescript
invoke("generate_i2c_driver", {
  instance: number,
  speed: string,      // "standard", "fast", "fast_plus"
  address: number,    // 7-bit address
  language: string
}) -> Result<{header: string, source: string}, string>
```

---

## RTOS Generation

### Task

```typescript
invoke("generate_rtos_task", {
  rtos: string,           // "freertos", "zephyr"
  name: string,
  priority: string,       // "idle", "low", "normal", "high", "realtime"
  stackSize: number,
  entryFunction: string,
  autoStart: boolean
}) -> Result<string, string>
```

### Semaphore

```typescript
invoke("generate_rtos_semaphore", {
  rtos: string,
  name: string,
  semType: string,        // "binary", "counting"
  maxCount: number,
  initialCount: number
}) -> Result<string, string>
```

### Mutex

```typescript
invoke("generate_rtos_mutex", {
  rtos: string,
  name: string,
  recursive: boolean
}) -> Result<string, string>
```

### Queue

```typescript
invoke("generate_rtos_queue", {
  rtos: string,
  name: string,
  length: number,
  itemSize: number
}) -> Result<string, string>
```

---

## DSP Generation

### FIR Filter

```typescript
invoke("generate_fir_filter", {
  name: string,
  order: number,
  sampleRate: number,
  cutoffFreq: number,
  filterType: string,     // "lowpass", "highpass", "bandpass"
  windowType: string      // "rectangular", "hamming", "hanning", "blackman"
}) -> Result<string, string>
```

### IIR Filter

```typescript
invoke("generate_iir_filter", {
  name: string,
  order: number,
  sampleRate: number,
  cutoffFreq: number,
  qFactor: number,
  filterType: string
}) -> Result<string, string>
```

### FFT

```typescript
invoke("generate_fft_block", {
  name: string,
  size: number,           // 64, 128, 256, 512, 1024, 2048, 4096
  useCmsis: boolean,
  windowType?: string
}) -> Result<string, string>
```

### PID Controller

```typescript
invoke("generate_pid_controller", {
  name: string,
  kp: number,
  ki: number,
  kd: number,
  outputMin: number,
  outputMax: number,
  sampleTimeMs: number,
  antiWindup: boolean
}) -> Result<string, string>
```

---

## Wireless

### BLE

```typescript
invoke("generate_ble_service", {
  platform: string,       // "nrf52", "esp32"
  deviceName: string,
  serviceUuid: string,
  serviceName: string,
  characteristics: Array<{
    uuid: string,
    name: string,
    read: boolean,
    write: boolean,
    notify: boolean,
    max_length: number
  }>
}) -> Result<{code: string}, string>
```

### WiFi

```typescript
invoke("generate_wifi_config", {
  mode: string,           // "station", "ap"
  ssid: string,
  password: string,
  security: string,       // "open", "wpa2", "wpa3"
  channel?: number
}) -> Result<{code: string}, string>
```

### LoRa

```typescript
invoke("generate_lora_config", {
  frequencyMhz: number,   // 433, 868, 915, 923
  spreadingFactor: number,// 7-12
  bandwidth: string,      // "125", "250", "500"
  codingRate: number,     // 5-8
  txPower: number         // 2-20 dBm
}) -> Result<{code: string}, string>
```

---

## Security

```typescript
invoke("generate_bootloader", {
  flashSize: number,
  bootloaderSize: number,
  appOffset: number,
  verifySignature: boolean
}) -> Result<string, string>

invoke("generate_ota_client", {
  serverUrl: string,
  useTls: boolean,
  firmwareVersion: string
}) -> Result<string, string>

invoke("generate_secure_boot", {
  keyType: string,        // "ecdsa", "rsa"
  keySize: number
}) -> Result<string, string>

invoke("generate_crypto_utils", {
  algorithm: string       // "aes128", "aes256", "chacha20"
}) -> Result<string, string>
```

---

## AI Agent Commands

```typescript
// Chat with the active agent
invoke("agent_chat", {
  message: string
}) -> Result<AgentResponse, string>

// List available agents
invoke("list_agents")
  -> Result<AgentInfo[], string>

// Get active agent
invoke("get_active_agent")
  -> Result<AgentInfo | null, string>

// Set active agent
invoke("set_active_agent", {
  agentId: string         // "fsm", "code", "hardware"
}) -> Result<(), string>
```

---

## Hardware Commands

```typescript
// Detect connected devices
invoke("detect_devices")
  -> Result<DeviceInfo[], string>

// Connect to device
invoke("connect_device", {
  port: string,
  baudRate: number
}) -> Result<(), string>

// Flash firmware
invoke("flash_firmware", {
  file: string,
  target: string
}) -> Result<(), string>

// Read telemetry
invoke("read_telemetry")
  -> Result<TelemetryData, string>
```

---

## System

```typescript
// Get system info
invoke("get_system_info")
  -> Result<SystemInfo, string>

// Get MCU list
invoke("get_mcu_list")
  -> Result<McuInfo[], string>

// List serial ports
invoke("list_serial_ports")
  -> Result<PortInfo[], string>
```
