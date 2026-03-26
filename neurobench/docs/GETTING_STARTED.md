# NeuroBench - Getting Started Tutorial

Welcome to NeuroBench! This tutorial will guide you through creating your first embedded project with FSM-based state machine design.

## Prerequisites

- Node.js 18+ and npm
- Rust toolchain (for Tauri backend)
- A modern web browser

## Installation

```bash
# Clone and install dependencies
cd neurobench
npm install

# Start development server
npm run tauri dev
```

---

## Tutorial: LED Blink State Machine

In this 10-minute tutorial, you'll create a complete LED blink project with:
- A 2-state FSM (ON/OFF)
- GPIO driver configuration
- Timer-based transitions
- Generated C code

### Step 1: Create a New Project

1. Launch NeuroBench with `npm run tauri dev`
2. The canvas opens automatically
3. Your project is saved in the current directory

### Step 2: Add States

1. **Open the Node Palette** (left sidebar or press `P`)
2. Search for "State" or find it in the FSM category
3. **Drag two State nodes** onto the canvas
4. Double-click each to rename:
   - First state: `LED_OFF`
   - Second state: `LED_ON`

### Step 3: Connect States with Transitions

1. **Click the output port** (right side) of `LED_OFF`
2. **Drag to the input port** (left side) of `LED_ON`
3. This creates a transition edge
4. Repeat for the reverse (`LED_ON` → `LED_OFF`)

### Step 4: Configure Transition Timing

1. Click on a transition edge
2. In the properties panel, set:
   - **Guard**: `timer_expired`
   - **Action**: `toggle_led(); reset_timer(500);`

### Step 5: Add GPIO Configuration

1. Click the **GPIO** button in the sidebar (or press `Ctrl+1`)
2. Configure the LED pin:
   - Port: **A**
   - Pin: **5**
   - Mode: **Output**
   - Initial: **Low**
3. Click **Generate GPIO Driver**

### Step 6: Generate Code

1. Press `Ctrl+G` or click **Generate Code** in the toolbar
2. Review the generated files:
   - `main.c` - State machine implementation
   - `gpio_driver.c` - GPIO initialization
   - `gpio_driver.h` - GPIO header

### Step 7: Build (Optional)

If you have ARM GCC installed:

1. Open the **Build Panel** (`Ctrl+B`)
2. Click **Discover** to find toolchains
3. Select your toolchain and click **Build**

---

## Next Steps

Now that you've completed your first project, explore these features:

### Driver Panels
- **UART**: Serial communication
- **SPI/I2C**: Sensor integration
- **Timers**: Hardware timer configuration

### RTOS Integration
- Add FreeRTOS or Zephyr support
- Create tasks with priorities
- Use semaphores and queues

### DSP Features
- Design FIR/IIR filters
- FFT for signal analysis
- PID controllers

### Wireless
- BLE service generation
- WiFi configuration
- LoRa radio setup

---

## Tips & Tricks

| Tip | How |
|-----|-----|
| Quick zoom | Scroll wheel or `+`/`-` |
| Pan canvas | Middle mouse button or Alt+drag |
| Duplicate node | `Ctrl+D` |
| Undo/Redo | `Ctrl+Z` / `Ctrl+Y` |
| Search palette | Type in palette search box |
| AI Assist | Click 🤖 for code suggestions |

---

## Troubleshooting

**Canvas is empty after reload?**
- Projects are saved to disk - check File → Recent Projects

**Toolchain not found?**
- Install ARM GCC: `arm-none-eabi-gcc`
- Or Rust: `rustup target add thumbv7em-none-eabihf`

**Build fails?**
- Check the Diagnostics tab for error details
- Ensure all required header files are present

---

## Support

- Documentation: See `USER_GUIDE.md` and `API_REFERENCE.md`
- Issues: Report on the project's issue tracker
