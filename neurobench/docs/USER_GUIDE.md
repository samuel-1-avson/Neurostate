# NeuroBench User Guide

## 🚀 Getting Started

NeuroBench is an industrial-grade embedded systems workbench for designing, simulating, and exporting FSM-based systems with integrated code generation.

### Quick Start

1. **Launch the application**: Run `npm run tauri dev` in the project root
2. **Create a new project**: File → New Project
3. **Design your FSM**: Use the canvas to add states and transitions
4. **Configure peripherals**: Use the driver panels (GPIO, UART, SPI, etc.)
5. **Generate code**: Click the Generate button to produce C/C++/Rust code

---

## 📋 Features Overview

### FSM Designer
- Visual state machine editor with drag-and-drop
- State types: Input, Output, Process, Decision
- Transition guards and actions
- Simulation mode for step-by-step execution

### Driver Generation
| Peripheral | Languages | MCUs Supported |
|------------|-----------|----------------|
| GPIO | C, C++, Rust, Ada, Asm, MicroPython, Zig | All |
| UART | C, C++, Rust, Ada, Asm | STM32, ESP32, nRF52 |
| SPI | C, C++, Rust, Ada, Asm | All |
| I2C | C, C++, Rust, Ada, Asm | All |
| CAN | C, C++ | STM32 |
| Modbus | C, C++ | STM32 |

### RTOS Support
- **FreeRTOS**: Tasks, semaphores, mutexes, queues, timers
- **Zephyr**: Threads, semaphores, mutexes, message queues

### DSP Features
- FIR/IIR filter design with CMSIS-DSP
- FFT with windowing functions
- PID controller with anti-windup

### Wireless
- BLE GATT service generation (nRF52, ESP32)
- WiFi configuration (ESP32)
- LoRa radio setup

### Security
- Secure bootloader generation
- OTA update client
- Firmware encryption utilities

---

## 🛠️ Panel Reference

### GPIO Panel
Configure digital I/O pins:
- **Port/Pin**: Select GPIO port (A-K) and pin (0-15)
- **Mode**: Input, Output, Alternate, Analog
- **Pull**: None, Up, Down
- **Speed**: Low, Medium, High, Very High

### UART Panel
Configure serial communication:
- **Instance**: USART1, USART2, etc.
- **Baud Rate**: 9600, 115200, etc.
- **Data Bits**: 7, 8, 9
- **Parity**: None, Even, Odd
- **Stop Bits**: 1, 1.5, 2

### RTOS Panel
Design real-time tasks:
1. Select RTOS (FreeRTOS or Zephyr)
2. Add tasks with name, priority, stack size
3. Configure synchronization primitives
4. Generate main.c with task creation

### DSP Panel
Design digital filters:
- **FIR**: Specify order, cutoff, window type
- **IIR**: Biquad filters with Q factor
- **PID**: Set Kp, Ki, Kd with anti-windup

---

## ⌨️ Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Save Project | Ctrl+S |
| Undo | Ctrl+Z |
| Redo | Ctrl+Y |
| Select All | Ctrl+A |
| Duplicate | Ctrl+D |
| Generate Code | Ctrl+G |
| Build Panel | Ctrl+B |
| Settings | Ctrl+, |
| Quick Panel 1-9 | Ctrl+1-9 |
| Show Help | F1 |
| Delete Node | Delete |
| Nudge Node | Arrow Keys |
| Deselect/Close | Escape |

---

## 🔧 Terminal Commands

The built-in terminal supports embedded development commands:

```bash
# List available serial ports
ports

# Build project (STM32)
build --target STM32F401

# Flash firmware
flash --port COM3 --file firmware.elf

# Monitor serial output
monitor --port COM3 --baud 115200
```

---

## 🤖 AI Assistant

NeuroBench includes AI agents for embedded development:

### FSM Architect
"Design a traffic light controller with pedestrian button"

### Code Generator
"Generate a UART driver for STM32F401 at 115200 baud"

### Available Agents
- **FSM Architect**: Design state machines from descriptions
- **Code Generator**: Generate peripheral drivers
- **Hardware Debugger**: Diagnose issues

---

## 📁 Project Structure

Generated projects follow this structure:

```
project/
├── src/
│   ├── main.c          # Entry point
│   ├── fsm.c           # State machine implementation
│   ├── gpio_driver.c   # GPIO peripheral driver
│   └── uart_driver.c   # UART peripheral driver
├── include/
│   ├── fsm.h           # FSM types and API
│   └── drivers.h       # Driver declarations
├── CMakeLists.txt      # CMake configuration
└── Makefile            # Alternative build
```

---

## 🔌 Supported MCUs (24+)

| Family | Chips | Features |
|--------|-------|----------|
| STM32 | F103, F401, L476, G474, H743, WL55 | Full HAL support |
| ESP32 | ESP32, C3, C6, S3, H2 | WiFi, BLE, Matter |
| nRF | nRF52832, nRF52840, nRF5340 | BLE 5, Thread, Zigbee |
| RP2040 | Pico | Dual-core, PIO |
| SAMD | SAMD21, SAMD51 | Arduino, USB |
| AVR | ATmega328P, ATmega2560 | Classic Arduino |
| NXP | LPC1768, LPC55S69 | Industrial, TrustZone |
| RISC-V | CH32V003, GD32VF103 | Low-cost, open arch |

---

## ❓ Troubleshooting

### Application won't start
- Ensure WebView2 runtime is installed (Windows)
- Check that `GEMINI_API_KEY` is set for AI features

### Code generation fails
- Verify peripheral configuration is complete
- Check that MCU is selected

### Build errors
- Ensure toolchain is installed (arm-none-eabi-gcc)
- Verify CMake version ≥ 3.16

---

## 📚 API Reference

### IPC Commands (Frontend → Backend)

```typescript
// Generate GPIO driver
invoke("generate_gpio_driver", {
  port: "A",
  pin: 5,
  mode: "output",
  language: "C"
});

// Generate RTOS task
invoke("generate_rtos_task", {
  rtos: "freertos",
  name: "MainTask",
  priority: "normal",
  stackSize: 2048
});

// Chat with AI agent
invoke("agent_chat", {
  message: "Design a traffic light FSM"
});
```

---

*NeuroBench v0.1.0 - Built with Tauri + SolidJS + Rust*
