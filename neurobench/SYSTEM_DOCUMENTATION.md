# NeuroBench System Documentation

## 📋 Executive Summary

**NeuroBench** is a professional-grade, AI-native embedded systems development environment. It has evolved into a comprehensive platform combining a Rust-based high-performance backend with a reactive SolidJS frontend. It features a cycle-accurate simulation engine, a multi-agent AI system, and extensive support for modern embedded protocols and security standards.

| Metric | Value |
|--------|-------|
| **Backend** | Rust (Tauri) ~200+ source files, ~5,600 lines in lib.rs |
| **Frontend** | SolidJS/TypeScript ~150+ components |
| **Total Modules** | 30+ Rust modules (Simulation, Agents, Canvas, Drivers...) |
| **IPC Commands** | **285+ commands** (up from 90) |
| **Target Platform** | STM32, nRF52, ESP32, RP2040, Custom Cores |

---

## 🏗️ System Architecture v2.0

The system has grown into a modular "operating system" for embedded development.

```mermaid
graph TB
    subgraph Frontend["Frontend (SolidJS)"]
        UI[App.tsx - Shell]
        subgraph Views
            Canvas[Unified Infinite Canvas]
            SimDash[Simulation Dashboard]
            Term[Intelligent Terminal]
            Panels[30+ Config Panels]
        end
    end
    
    subgraph Backend["Backend (Rust/Tauri)"]
        IPC[IPC Dispatcher (~285 cmds)]
        
        subgraph Engines
            CoreFSM[FSM Core & Graph Engine]
            SimEngine[Simulation Engine (QEMU/Unicorn)]
            Scheduler[Job & Build Scheduler]
            EventStore[Event Sourcing / History]
        end
        
        subgraph AI_Layer["Agentic Core"]
            Router[AI Router]
            Orchestrator[Multi-Agent Orchestrator]
            Specialists[Coder/Hardware/QA Agents]
            RAG[Vector Store & Semantic Cache]
        end
        
        subgraph Hardware_Stack
            Drivers[Driver Generators (30+)]
            HAL[Hardware Abstraction]
            Toolchain[ARM/RISC-V Toolchains]
            Probe[Debug/Flash (probe-rs)]
        end
    end
    
    UI <--> IPC
    IPC <--> Engines
    IPC <--> AI_Layer
    IPC <--> Hardware_Stack
    
    Engines --> Hardware_Stack
    AI_Layer --> Engines
```

### Key Backend Modules (`src-tauri/src/`)

| Module | Files | Purpose |
|--------|-------|---------|
| `simulation/` | 27 | **Cycle-accurate simulation**, CPU/Memory/Peripheral state, Breakpoints, Waveforms |
| `agents/` | 33 | **Multi-Agent System**: Director, Coder, Hardware, Planner, Critic, Voice, Self-Reflection |
| `canvas/` | 31 | **Graph Theory Engine**: Spatial indexing, auto-layout, signal flow, undo/redo, validation |
| `drivers/` | 60+ | Massive driver library including Wireless, Security, DSP, RTOS |
| `jobs/` | 7 | Priority Job Scheduler, Async Build Pipeline, Resource Management |
| `ai/` | 13 | LLM integration, Guardrails, Prompts, RAG Pipeline |
| `toolchain/` | 6 | GCC/Clang management, Streaming Builds, Output Parsing |
| `nodes/` | 4 | Node behavior engine, Code generation definitions |
| `transitions/` | 4 | Transition logic, Edge validation, Guard evaluation |

### Key Frontend Components (`src/components/`)

| Category | Components | Examples |
|----------|------------|----------|
| **Core UI** | ~20 | `UnifiedCanvas`, `IndustrialMenuBar`, `ActivityBar`, `BottomDock` |
| **Simulation** | ~5 | `SimulationDashboard`, `SimulatorPanel`, `WaveformViewer`, `RegisterView` |
| **Config Panels** | ~25 | `ClockPanel`, `PinDiagram`, `PeripheralsPanel`, `McuSelector` |
| **Development** | ~10 | `Terminal`, `CodeDiffPanel`, `GitPanel`, `WorkflowPanel`, `SnippetPanel` |
| **Domain Specific** | ~15 | `WirelessPanel`, `SecurityPanel`, `DSPPanel`, `RTOSPanel`, `PowerMonitor` |

---

## ✨ Feature Inventory

### 1. Simulation & Digital Twin (NEW)
- [x] **Cycle-Accurate Simulation**: Run firmware virtually before hardware is ready.
- [x] **Peripheral Simulation**: GPIO, UART, Timer injection/monitoring.
- [x] **Debug Controls**: Step, Pause, Resume, Reset, Breakpoints.
- [x] **State Inspection**: Live view of CPU Registers (R0-R15, PC, SP, Flags) and Memory.
- [x] **Behavioral Bridge**: Link FSM states to simulated hardware events.

### 2. Advanced AI Agents (NEW)
- [x] **Multi-Agent Orchestrator**: "Director" agent delegates tasks to specialized sub-agents.
- [x] **Agent Roles**:
  - `CodeAgent`: Writes and refactors driver code.
  - `HardwareAgent`: Validates electrical constraints and pinouts.
  - `DocsAgent`: Maintains documentation and explains code.
  - `CanvasAgent`: Manipulates the visual graph directly.
- [x] **Self-Reflection**: agents critique their own outputs before showing user.
- [x] **Tool Use**: Agents have access to 50+ typed tools (file ops, git, grep, build).

### 3. Expanded Peripherals & Drivers
The driver generation engine has expanded significantly:

*   **Wireless**: WiFi, BLE, LoRa, Thread, Matter, Zigbee, NFC, UWB, LTE-M.
*   **Security**: Secure Boot, OTA Updates, TrustZone/HSM/TPM, Crypto (AES/ECC/SHA), Attestation.
*   **DSP & Math**: FFT, FIR/IIR Filters, PID Controllers, Matrix Operations, Neural Networks (TinyML).
*   **RTOS Support**: FreeRTOS, Zephyr, ThreadX task/queue/mutex generation.

### 4. Professional Development Workflow
- [x] **Streaming Build Pipeline**: Real-time GCC output parsed with error highlighting.
- [x] **Git Integration**: Built-in diff viewer (`CodeDiffPanel`), commit interface, branch management.
- [x] **Event Sourcing**: Infinite undo/redo and "Time Travel" debugging of the project state.
- [x] **Project Management**: Project browser, templates, manifest versioning.

### 5. Visual Enhancement
- [x] **Unified Canvas**: High-performance rendering for 1000+ nodes.
- [x] **Interactive Pinout**: `PinDiagram` with conflict detection.
- [x] **Performance Monitor**: `PerformancePanel` for both Host and Target (via RTT).

---

## 📈 System Stats (Updated)

| Metric | Count |
|--------|-------|
| **IPC Commands** | ~285 |
| **Driver Generators** | 45+ Types |
| **Supported MCUs** | ~50+ Families (STM32, nRF, ESP, RP2040) |
| **Theme Support** | 8 Professional Themes |
| **AI Tools** | 60+ Available to Agents |

---

## 🔄 Job Scheduler & Execution

The system now runs a **Background Job Scheduler** handling:
1.  **AI Tasks**: Long-running generation and reasoning (backgrounded).
2.  **Compilation**: Streaming builds do not block the UI.
3.  **Simulation**: Runs in a separate thread/process with shared memory state.
4.  **Device I/O**: Flash/Debug operations via `probe-rs`.

## 💡 v2.0 Roadmap Status

*   **Completed**:
    *   Simulation Engine Core
    *   Multi-Agent Architecture
    *   Wireless & Security Modules
    *   Job Schedulers
*   **In Progress**:
    *   Advanced Waveform Analysis
    *   PCB Layout Integration
    *   Formal Verification of FSMs
*   **Planned**:
    *   Cloud Team Collaboration
    *   Marketplace for Drivers/Agents

---

*Generated by NeuroBench System*
