# NeuroBench 🧠⚡

> **Professional Embedded Systems Development Environment**

A visual FSM designer, code generator, and AI-assisted development platform for microcontrollers.

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust)
![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?style=flat&logo=typescript&logoColor=white)
![Tauri](https://img.shields.io/badge/Tauri-24C8DB?style=flat&logo=tauri&logoColor=white)
![SolidJS](https://img.shields.io/badge/SolidJS-2C4F7C?style=flat&logo=solid&logoColor=white)

---

## ✨ Features

### 🎨 Visual FSM Design
- Drag-and-drop state machine editor
- 50+ node types (FSM, Hardware, Processing, I/O, Protocols)
- Undo/Redo, zoom/pan, grid snap

### 💻 Multi-Language Code Generation
| Language | Drivers |
|----------|---------|
| C | GPIO, UART, SPI, I2C, CAN |
| C++ | GPIO, UART, SPI, I2C, CAN |
| Rust | GPIO, UART, SPI, I2C, CAN |
| Ada/SPARK | GPIO, UART, SPI, I2C |
| ARM Assembly | GPIO, UART, SPI, I2C |
| MicroPython | GPIO |
| Zig | GPIO |

### 🔧 Supported MCUs (24+)
| Family | Models |
|--------|--------|
| **STM32** | F103, F401, H743, L476, G474, WL55 |
| **Nordic** | nRF52832, nRF52840, nRF5340 |
| **ESP32** | ESP32, C3, C6, S3, H2 |
| **Raspberry Pi** | RP2040 |
| **Microchip** | SAMD21, SAMD51 |
| **AVR** | ATmega328P, ATmega2560 |
| **NXP** | LPC1768, LPC55S69 |
| **RISC-V** | CH32V003, GD32VF103 |

### 🌐 Node Types
| Category | Examples |
|----------|----------|
| FSM | State, Decision, Junction, History |
| Hardware | GPIO, ADC, DAC, PWM, UART, SPI, I2C, CAN |
| Wireless | WiFi, BLE, LoRa, Zigbee |
| Protocols | MQTT, HTTP, WebSocket, Modbus |
| RTOS | Task, Semaphore, Mutex, Queue |
| System | Watchdog, PowerMgmt, ClockConfig, DebugLog |

### 🤖 AI Integration
- Gemini-powered code assistance
- Natural language FSM generation
- Context-aware suggestions

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+S` | Save project |
| `Ctrl+Z` / `Ctrl+Y` | Undo / Redo |
| `Ctrl+A` | Select all nodes |
| `Ctrl+D` | Duplicate node |
| `Ctrl+G` | Generate code |
| `Ctrl+B` | Build panel |
| `Ctrl+,` | Settings |
| `Ctrl+1-9` | Quick panel switch |
| `F1` | Show shortcuts help |
| `Delete` | Delete selected |
| `Arrow keys` | Nudge nodes |
| `Escape` | Deselect / Close modals |

---

## 🚀 Quick Start

```bash
# Clone the repository
git clone https://github.com/your-username/neurobench.git
cd neurobench

# Install dependencies
npm install

# Run development server
npm run tauri dev

# Build for production
npm run tauri build
```

---

## 📁 Project Structure

```
neurobench/
├── src/                    # Frontend (SolidJS)
│   ├── App.tsx            # Main application
│   └── components/        # 66 UI components
├── src-tauri/             # Backend (Rust)
│   └── src/
│       ├── drivers/       # Code generators
│       ├── agents/        # AI system
│       ├── nodes/         # Node types
│       └── mcu/           # MCU registry
└── docs/                  # Documentation
```

---

## 📊 Tech Stack

- **Frontend**: SolidJS, TypeScript, Vite
- **Backend**: Rust, Tauri
- **AI**: Google Gemini API
- **Targets**: ARM Cortex-M, RISC-V, AVR, Xtensa

---

## 📝 License

MIT License - see [LICENSE](LICENSE) for details.

---

*Built with ❤️ for embedded developers*
