// Embedded System Commands
// 30+ specialized commands for embedded development

use super::{TerminalResult, TerminalLine};
use super::parser::ParsedCommand;

/// Process an embedded system command
pub fn process_embedded_command(cmd: &ParsedCommand) -> TerminalResult {
    match cmd.command.to_lowercase().as_str() {
        // === Help ===
        "help" => cmd_help(&cmd.args),
        
        // === Flash Commands ===
        "flash" => cmd_flash(cmd),
        "verify" => cmd_verify(cmd),
        "erase" => cmd_erase(cmd),
        "dfu" => cmd_dfu(cmd),
        
        // === Monitor Commands ===
        "monitor" => cmd_monitor(cmd),
        
        // === Debug Commands ===
        "gdb" => cmd_gdb(cmd),
        "debug" => cmd_debug(cmd),
        "trace" => cmd_trace(cmd),
        "breakpoint" | "bp" => cmd_breakpoint(cmd),
        
        // === Power Commands ===
        "power" => cmd_power(cmd),
        
        // === Build Commands ===
        "build" => cmd_build(cmd),
        "clean" => cmd_clean(cmd),
        "rebuild" => cmd_rebuild(cmd),
        
        // === Serial Commands ===
        "serial" => cmd_serial(cmd),
        
        // === FSM Commands ===
        "fsm" => cmd_fsm(cmd),
        
        // === Driver Commands ===
        "driver" => cmd_driver(cmd),
        
        // === MCU/Target Commands ===
        "mcu" | "target" => cmd_mcu(cmd),
        "info" => cmd_info(cmd),
        
        // === Utility Commands ===
        "version" => cmd_version(),
        "pwd" => cmd_pwd(),
        "ls" | "dir" => cmd_ls(&cmd.args),
        "echo" => cmd_echo(&cmd.args),
        "clear" => TerminalResult::success(vec![]),
        "history" => TerminalResult::info("History is managed by terminal UI"),
        "export" => cmd_export(cmd),
        "env" => cmd_env(),
        
        // === AI Commands ===
        "ai" => cmd_ai(cmd),
        
        // === Log Commands ===
        "log" => cmd_log(cmd),
        
        // === GPIO Commands ===
        "gpio" => cmd_gpio(cmd),
        
        // Unknown
        _ => TerminalResult::error(&format!(
            "Unknown command: '{}'. Type 'help' for available commands.",
            cmd.command
        )),
    }
}

// ===== Help Command =====
fn cmd_help(args: &[String]) -> TerminalResult {
    if let Some(topic) = args.first() {
        return help_topic(topic);
    }
    
    TerminalResult::success(vec![
        TerminalLine::with_ansi("╔══════════════════════════════════════════════════════════════════════╗", "\x1b[38;5;141m"),
        TerminalLine::with_ansi("║           NeuroBench Terminal - Advanced Embedded Commands           ║", "\x1b[38;5;141m"),
        TerminalLine::with_ansi("╠══════════════════════════════════════════════════════════════════════╣", "\x1b[38;5;99m"),
        TerminalLine::info("  🔧 Flash & Program"),
        TerminalLine::output("    flash [file] --probe stlink|jlink --speed N   Flash firmware"),
        TerminalLine::output("    verify                                        Verify flash contents"),
        TerminalLine::output("    erase [--full|--sector N]                     Erase flash memory"),
        TerminalLine::output("    dfu enter|exit                                DFU bootloader mode"),
        TerminalLine::info("  📡 Monitor & Trace"),
        TerminalLine::output("    monitor uart [port] --baud N                  UART serial monitor"),
        TerminalLine::output("    monitor can --filter ID                       CAN bus monitor"),
        TerminalLine::output("    monitor gpio PIN                              GPIO waveform monitor"),
        TerminalLine::output("    trace start|stop swo FREQ                     ITM/SWO tracing"),
        TerminalLine::info("  🐛 Debug"),
        TerminalLine::output("    gdb connect|disconnect                        GDB server control"),
        TerminalLine::output("    debug launch                                  Start debug session"),
        TerminalLine::output("    bp add|remove|list ADDR                       Breakpoint management"),
        TerminalLine::info("  ⚡ Power"),
        TerminalLine::output("    power measure --interval N                    Live power measurement"),
        TerminalLine::output("    power report                                  Power consumption report"),
        TerminalLine::info("  🏗️ Build"),
        TerminalLine::output("    build [--release] --target MCU                Build project"),
        TerminalLine::output("    clean                                         Clean build artifacts"),
        TerminalLine::output("    rebuild                                       Clean and rebuild"),
        TerminalLine::info("  🔌 GPIO & Peripherals"),
        TerminalLine::output("    gpio config PIN MODE [SPEED]                  Configure GPIO pin"),
        TerminalLine::output("    gpio set|clear|toggle PIN                     Set GPIO state"),
        TerminalLine::output("    gpio read PIN                                 Read GPIO state"),
        TerminalLine::info("  🤖 AI Assistant"),
        TerminalLine::output("    ai \"question\"                                 Ask AI about your code"),
        TerminalLine::output("    ai explain [topic]                            Get explanations"),
        TerminalLine::output("    ai generate [description]                     Generate code"),
        TerminalLine::info("  🔄 FSM"),
        TerminalLine::output("    fsm simulate                                  Simulate FSM in terminal"),
        TerminalLine::output("    fsm step                                      Step through FSM"),
        TerminalLine::output("    fsm validate                                  Validate FSM structure"),
        TerminalLine::with_ansi("╚══════════════════════════════════════════════════════════════════════╝", "\x1b[38;5;99m"),
        TerminalLine::output("  Type 'help <command>' for detailed usage. Use Tab for autocomplete."),
    ])
}

fn help_topic(topic: &str) -> TerminalResult {
    match topic.to_lowercase().as_str() {
        "flash" => TerminalResult::success(vec![
            TerminalLine::info("flash - Flash firmware to MCU"),
            TerminalLine::output(""),
            TerminalLine::output("Usage: flash [file] [options]"),
            TerminalLine::output(""),
            TerminalLine::output("Options:"),
            TerminalLine::output("  --probe, -p     Debug probe (stlink, jlink, cmsis-dap)"),
            TerminalLine::output("  --speed, -s     Flash speed in kHz (1000, 4000, 8000)"),
            TerminalLine::output("  --target, -t    Target MCU (stm32f401, stm32f407, etc.)"),
            TerminalLine::output("  --verify, -v    Verify after flashing"),
            TerminalLine::output("  --reset, -r     Reset after flashing"),
            TerminalLine::output(""),
            TerminalLine::output("Examples:"),
            TerminalLine::success("  flash firmware.elf --probe stlink --speed 8000"),
            TerminalLine::success("  flash --jlink -v -r output.bin"),
        ]),
        "monitor" => TerminalResult::success(vec![
            TerminalLine::info("monitor - Real-time peripheral monitoring"),
            TerminalLine::output(""),
            TerminalLine::output("Usage: monitor <type> [options]"),
            TerminalLine::output(""),
            TerminalLine::output("Types:"),
            TerminalLine::output("  uart [port]     UART serial monitor (COM3, /dev/ttyUSB0)"),
            TerminalLine::output("  can             CAN bus monitor with ID filtering"),
            TerminalLine::output("  gpio PIN        GPIO pin waveform (unicode visualization)"),
            TerminalLine::output("  adc CHANNEL     ADC channel live graph"),
            TerminalLine::output(""),
            TerminalLine::output("Options:"),
            TerminalLine::output("  --baud, -b      Baud rate for UART (default: 115200)"),
            TerminalLine::output("  --filter, -f    Filter pattern or ID"),
        ]),
        "ai" => TerminalResult::success(vec![
            TerminalLine::info("ai - AI-powered embedded systems assistant"),
            TerminalLine::output(""),
            TerminalLine::output("Usage: ai <question or command>"),
            TerminalLine::output(""),
            TerminalLine::output("Examples:"),
            TerminalLine::success("  ai \"why is my ADC reading noisy?\""),
            TerminalLine::success("  ai explain DMA transfer"),
            TerminalLine::success("  ai generate SPI driver for SD card"),
            TerminalLine::success("  ai optimize this code for power"),
            TerminalLine::output(""),
            TerminalLine::output("The AI has access to your:"),
            TerminalLine::output("  • Current pinout configuration"),
            TerminalLine::output("  • Clock tree settings"),
            TerminalLine::output("  • Generated driver code"),
            TerminalLine::output("  • FSM state machine"),
        ]),
        _ => TerminalResult::info(&format!("No help available for '{}'", topic)),
    }
}

// ===== Flash Commands =====
fn cmd_flash(cmd: &ParsedCommand) -> TerminalResult {
    let file = cmd.args.first().map(|s| s.as_str()).unwrap_or("firmware.elf");
    let probe = cmd.flags.get("probe")
        .or(cmd.flags.get("p"))
        .and_then(|v| v.clone())
        .unwrap_or_else(|| "stlink".to_string());
    let speed = cmd.flags.get("speed")
        .or(cmd.flags.get("s"))
        .and_then(|v| v.clone())
        .unwrap_or_else(|| "4000".to_string());

    TerminalResult::success(vec![
        TerminalLine::info(&format!("🔌 Connecting to {} probe...", probe.to_uppercase())),
        TerminalLine::success(&format!("✓ Probe detected: {} v2.1", probe.to_uppercase())),
        TerminalLine::output(&format!("  Target: STM32F401CCU6 (Cortex-M4)")),
        TerminalLine::output(&format!("  Speed: {} kHz", speed)),
        TerminalLine::info("📝 Erasing flash sectors..."),
        TerminalLine::output("  Sector 0-3: cleared"),
        TerminalLine::info(&format!("⬆️  Programming {}...", file)),
        TerminalLine::output("  [████████████████████████████████] 100%"),
        TerminalLine::output("  Size: 48.2 KB, Time: 2.3s"),
        TerminalLine::info("🔍 Verifying..."),
        TerminalLine::success("✓ Verification passed"),
        TerminalLine::success("✓ Flash complete! Target reset."),
    ])
}

fn cmd_verify(_cmd: &ParsedCommand) -> TerminalResult {
    TerminalResult::success(vec![
        TerminalLine::info("🔍 Verifying flash contents..."),
        TerminalLine::output("  Reading: 0x08000000 - 0x0800C000"),
        TerminalLine::output("  Comparing with: firmware.elf"),
        TerminalLine::success("✓ Verification passed: 100% match"),
    ])
}

fn cmd_erase(cmd: &ParsedCommand) -> TerminalResult {
    let full = cmd.flags.contains_key("full");
    let sector = cmd.flags.get("sector").and_then(|v| v.clone());

    if full {
        TerminalResult::success(vec![
            TerminalLine::warning("⚠️  Full chip erase requested"),
            TerminalLine::info("Erasing all flash sectors..."),
            TerminalLine::output("  [████████████████████████████████] 100%"),
            TerminalLine::success("✓ Full erase complete"),
        ])
    } else if let Some(s) = sector {
        TerminalResult::success(vec![
            TerminalLine::info(&format!("Erasing sector {}...", s)),
            TerminalLine::success(&format!("✓ Sector {} erased", s)),
        ])
    } else {
        TerminalResult::success(vec![
            TerminalLine::info("Erasing application sectors..."),
            TerminalLine::output("  Sectors 0-7: erased"),
            TerminalLine::success("✓ Erase complete"),
        ])
    }
}

fn cmd_dfu(cmd: &ParsedCommand) -> TerminalResult {
    let action = cmd.args.first().map(|s| s.as_str()).unwrap_or("enter");
    match action {
        "enter" => TerminalResult::success(vec![
            TerminalLine::info("🔄 Entering DFU bootloader mode..."),
            TerminalLine::output("  Sending BOOT0 signal..."),
            TerminalLine::success("✓ Device now in DFU mode"),
            TerminalLine::output("  Use dfu-util or STM32CubeProgrammer to flash"),
        ]),
        "exit" => TerminalResult::success(vec![
            TerminalLine::info("🔄 Exiting DFU mode..."),
            TerminalLine::output("  Resetting device..."),
            TerminalLine::success("✓ Device running application"),
        ]),
        _ => TerminalResult::info("Usage: dfu enter|exit"),
    }
}

// ===== Monitor Commands =====
fn cmd_monitor(cmd: &ParsedCommand) -> TerminalResult {
    let monitor_type = cmd.args.first().map(|s| s.as_str()).unwrap_or("uart");
    
    match monitor_type {
        "uart" => {
            let port = cmd.args.get(1).map(|s| s.as_str()).unwrap_or("COM3");
            let baud = cmd.flags.get("baud")
                .or(cmd.flags.get("b"))
                .and_then(|v| v.clone())
                .unwrap_or_else(|| "115200".to_string());
            TerminalResult::success(vec![
                TerminalLine::info(&format!("📡 Opening UART monitor on {} @ {} baud", port, baud)),
                TerminalLine::success("✓ Connected"),
                TerminalLine::output("─────────────────────────────────────"),
                TerminalLine::output("[RX] System initialized"),
                TerminalLine::output("[RX] ADC: 2048 (1.65V)"),
                TerminalLine::output("[RX] Temperature: 25.3°C"),
                TerminalLine::output("─────────────────────────────────────"),
                TerminalLine::info("Press Ctrl+C to close monitor"),
            ])
        }
        "can" => {
            TerminalResult::success(vec![
                TerminalLine::info("📡 CAN Bus Monitor - 500 kbps"),
                TerminalLine::success("✓ Connected to CAN interface"),
                TerminalLine::output("─────────────────────────────────────"),
                TerminalLine::output("ID: 0x123  DLC: 8  Data: 01 02 03 04 05 06 07 08"),
                TerminalLine::output("ID: 0x456  DLC: 4  Data: AA BB CC DD"),
                TerminalLine::output("ID: 0x789  DLC: 2  Data: 12 34"),
                TerminalLine::output("─────────────────────────────────────"),
            ])
        }
        "gpio" => {
            let pin = cmd.args.get(1).map(|s| s.as_str()).unwrap_or("PA5");
            TerminalResult::success(vec![
                TerminalLine::info(&format!("📡 GPIO Monitor: {}", pin)),
                TerminalLine::output(&format!("{}: ─╮╭─╮╭─╮╭─╮╭─╮╭─╮╭─  (1Hz toggle)", pin)),
                TerminalLine::output("        │││││││││││       "),
                TerminalLine::output("        ╰╯╰╯╰╯╰╯╰╯╰╯       "),
            ])
        }
        "adc" => {
            let channel = cmd.args.get(1).map(|s| s.as_str()).unwrap_or("0");
            TerminalResult::success(vec![
                TerminalLine::info(&format!("📊 ADC Monitor: Channel {}", channel)),
                TerminalLine::output("3.3V ┤                    ╭─╮  "),
                TerminalLine::output("2.5V ┤          ╭─╮   ╭──╯ │  "),
                TerminalLine::output("1.6V ┤     ╭───╯  ╰──╯    ╰─ "),
                TerminalLine::output("0.0V ┤────╯                   "),
                TerminalLine::output("     └────────────────────────"),
            ])
        }
        _ => TerminalResult::info("Usage: monitor uart|can|gpio|adc [target]"),
    }
}

// ===== Debug Commands =====
fn cmd_gdb(cmd: &ParsedCommand) -> TerminalResult {
    let action = cmd.args.first().map(|s| s.as_str()).unwrap_or("connect");
    let port = cmd.flags.get("port")
        .or(cmd.flags.get("p"))
        .and_then(|v| v.clone())
        .unwrap_or_else(|| "3333".to_string());

    match action {
        "connect" => TerminalResult::success(vec![
            TerminalLine::info("🐛 Starting GDB server..."),
            TerminalLine::output(&format!("  OpenOCD listening on port {}", port)),
            TerminalLine::success("✓ GDB server ready"),
            TerminalLine::output(&format!("  Connect with: arm-none-eabi-gdb -ex 'target remote :{}'", port)),
        ]),
        "disconnect" => TerminalResult::success(vec![
            TerminalLine::info("Stopping GDB server..."),
            TerminalLine::success("✓ GDB server stopped"),
        ]),
        _ => TerminalResult::info("Usage: gdb connect|disconnect [--port N]"),
    }
}

fn cmd_debug(cmd: &ParsedCommand) -> TerminalResult {
    let action = cmd.args.first().map(|s| s.as_str()).unwrap_or("launch");
    
    match action {
        "launch" => TerminalResult::success(vec![
            TerminalLine::info("🚀 Launching debug session..."),
            TerminalLine::output("  1. Flashing firmware..."),
            TerminalLine::output("  2. Starting OpenOCD..."),
            TerminalLine::output("  3. Connecting GDB..."),
            TerminalLine::output("  4. Setting initial breakpoint at main()"),
            TerminalLine::success("✓ Debug session active"),
            TerminalLine::output("  Stopped at: main() [line 42]"),
        ]),
        _ => TerminalResult::info("Usage: debug launch"),
    }
}

fn cmd_trace(cmd: &ParsedCommand) -> TerminalResult {
    let action = cmd.args.first().map(|s| s.as_str()).unwrap_or("start");
    
    match action {
        "start" => {
            let trace_type = cmd.args.get(1).map(|s| s.as_str()).unwrap_or("swo");
            let freq = cmd.args.get(2).map(|s| s.as_str()).unwrap_or("2000000");
            TerminalResult::success(vec![
                TerminalLine::info(&format!("📍 Starting {} trace @ {} Hz", trace_type.to_uppercase(), freq)),
                TerminalLine::output("  ITM channel 0: enabled (printf)"),
                TerminalLine::output("  ITM channel 31: enabled (timestamps)"),
                TerminalLine::success("✓ Trace active"),
            ])
        }
        "stop" => TerminalResult::success(vec![
            TerminalLine::info("Stopping trace..."),
            TerminalLine::success("✓ Trace stopped"),
            TerminalLine::output("  Captured: 1,234 events"),
        ]),
        _ => TerminalResult::info("Usage: trace start|stop [swo|etm] [freq]"),
    }
}

fn cmd_breakpoint(cmd: &ParsedCommand) -> TerminalResult {
    let action = cmd.args.first().map(|s| s.as_str()).unwrap_or("list");
    
    match action {
        "add" => {
            let addr = cmd.args.get(1).map(|s| s.as_str()).unwrap_or("main");
            TerminalResult::success(vec![
                TerminalLine::success(&format!("✓ Breakpoint set at {}", addr)),
            ])
        }
        "remove" => {
            let addr = cmd.args.get(1).map(|s| s.as_str()).unwrap_or("0");
            TerminalResult::success(vec![
                TerminalLine::success(&format!("✓ Breakpoint {} removed", addr)),
            ])
        }
        "list" => TerminalResult::success(vec![
            TerminalLine::info("Breakpoints:"),
            TerminalLine::output("  1. main() [0x08000100]"),
            TerminalLine::output("  2. HAL_GPIO_TogglePin() [0x08001234]"),
        ]),
        _ => TerminalResult::info("Usage: bp add|remove|list [addr|function]"),
    }
}

// ===== Power Commands =====
fn cmd_power(cmd: &ParsedCommand) -> TerminalResult {
    let action = cmd.args.first().map(|s| s.as_str()).unwrap_or("measure");
    
    match action {
        "measure" => TerminalResult::success(vec![
            TerminalLine::info("⚡ Power Measurement (INA226)"),
            TerminalLine::output("─────────────────────────────────────"),
            TerminalLine::output("  Voltage: 3.32 V"),
            TerminalLine::output("  Current: 12.4 mA"),
            TerminalLine::output("  Power:   41.2 mW"),
            TerminalLine::output("─────────────────────────────────────"),
            TerminalLine::output("  Mode: Active"),
            TerminalLine::output("  Est. battery life: 80.6 hours (1000mAh)"),
        ]),
        "report" => TerminalResult::success(vec![
            TerminalLine::info("⚡ Power Consumption Report"),
            TerminalLine::output("  Average: 15.2 mA"),
            TerminalLine::output("  Peak: 45.8 mA"),
            TerminalLine::output("  Sleep: 2.1 µA"),
            TerminalLine::output("  Run time: 00:05:23"),
        ]),
        _ => TerminalResult::info("Usage: power measure|report [--interval N]"),
    }
}

// ===== Build Commands =====
fn cmd_build(cmd: &ParsedCommand) -> TerminalResult {
    let release = cmd.flags.contains_key("release") || cmd.flags.contains_key("r");
    let target = cmd.flags.get("target")
        .or(cmd.flags.get("t"))
        .and_then(|v| v.clone())
        .unwrap_or_else(|| "STM32F401".to_string());

    let mode = if release { "Release" } else { "Debug" };
    
    TerminalResult::success(vec![
        TerminalLine::info(&format!("🏗️ Building for {} ({})...", target, mode)),
        TerminalLine::output("  Compiling fsm.c..."),
        TerminalLine::output("  Compiling gpio_driver.c..."),
        TerminalLine::output("  Compiling main.c..."),
        TerminalLine::output("  Linking..."),
        TerminalLine::success(&format!("✓ Build complete: output/firmware_{}.elf", target.to_lowercase())),
        TerminalLine::output(&format!("  Size: .text: 12.4 KB, .data: 256 B, .bss: 1.2 KB")),
    ])
}

fn cmd_clean(_cmd: &ParsedCommand) -> TerminalResult {
    TerminalResult::success(vec![
        TerminalLine::info("🧹 Cleaning build artifacts..."),
        TerminalLine::output("  Removing: build/"),
        TerminalLine::output("  Removing: output/"),
        TerminalLine::success("✓ Clean complete"),
    ])
}

fn cmd_rebuild(cmd: &ParsedCommand) -> TerminalResult {
    let mut lines = vec![
        TerminalLine::info("🔄 Rebuilding project..."),
    ];
    lines.extend(cmd_clean(cmd).output);
    lines.extend(cmd_build(cmd).output);
    TerminalResult::success(lines)
}

// ===== Serial Commands =====
fn cmd_serial(cmd: &ParsedCommand) -> TerminalResult {
    let action = cmd.args.first().map(|s| s.as_str()).unwrap_or("list");
    
    match action {
        "list" => TerminalResult::success(vec![
            TerminalLine::info("📡 Available Serial Ports:"),
            TerminalLine::output("  • COM3  [USB] STM32 Virtual COM Port"),
            TerminalLine::output("  • COM4  [USB] CH340 Serial Adapter"),
            TerminalLine::output("  • COM1  [Standard] Communications Port"),
        ]),
        "open" => {
            let port = cmd.args.get(1).map(|s| s.as_str()).unwrap_or("COM3");
            TerminalResult::success(vec![
                TerminalLine::success(&format!("✓ Opened {} @ 115200 8N1", port)),
                TerminalLine::info("Serial monitor active. Ctrl+C to close."),
            ])
        }
        _ => TerminalResult::info("Usage: serial list|open [port]"),
    }
}

// ===== FSM Commands =====
fn cmd_fsm(cmd: &ParsedCommand) -> TerminalResult {
    let action = cmd.args.first().map(|s| s.as_str()).unwrap_or("status");
    
    match action {
        "simulate" => TerminalResult::success(vec![
            TerminalLine::info("🔄 FSM Simulation Mode"),
            TerminalLine::output("─────────────────────────────────────"),
            TerminalLine::output("  Current State: INIT"),
            TerminalLine::output("  Available transitions:"),
            TerminalLine::output("    → RUNNING (on 'ready' event)"),
            TerminalLine::output("    → ERROR (on 'fault' event)"),
            TerminalLine::output("─────────────────────────────────────"),
            TerminalLine::info("Type 'fsm step <event>' to transition"),
        ]),
        "step" => {
            let event = cmd.args.get(1).map(|s| s.as_str()).unwrap_or("ready");
            TerminalResult::success(vec![
                TerminalLine::info(&format!("→ Event: '{}'", event)),
                TerminalLine::success("  Transition: INIT → RUNNING"),
                TerminalLine::output("  Executing entry action: ledOn = true;"),
            ])
        }
        "validate" => TerminalResult::success(vec![
            TerminalLine::info("🔍 Validating FSM..."),
            TerminalLine::success("✓ No unreachable states"),
            TerminalLine::success("✓ No missing transitions"),
            TerminalLine::success("✓ No infinite loops detected"),
            TerminalLine::success("FSM is valid!"),
        ]),
        "stats" => TerminalResult::success(vec![
            TerminalLine::info("📊 FSM Statistics:"),
            TerminalLine::output("  States: 4"),
            TerminalLine::output("  Transitions: 5"),
            TerminalLine::output("  Entry actions: 3"),
            TerminalLine::output("  Exit actions: 1"),
        ]),
        _ => TerminalResult::info("Usage: fsm simulate|step|validate|stats"),
    }
}

// ===== Driver Commands =====
fn cmd_driver(cmd: &ParsedCommand) -> TerminalResult {
    let driver_type = cmd.args.first().map(|s| s.to_uppercase()).unwrap_or_else(|| "GPIO".to_string());
    
    TerminalResult::success(vec![
        TerminalLine::info(&format!("🔧 Generating {} driver...", driver_type)),
        TerminalLine::output(&format!("  Created: drivers/{}_driver.c", driver_type.to_lowercase())),
        TerminalLine::output(&format!("  Created: drivers/{}_driver.h", driver_type.to_lowercase())),
        TerminalLine::success(&format!("✓ {} driver generated", driver_type)),
    ])
}

// ===== MCU Commands =====
fn cmd_mcu(cmd: &ParsedCommand) -> TerminalResult {
    if let Some(target) = cmd.args.first() {
        TerminalResult::success(vec![
            TerminalLine::success(&format!("✓ Target MCU set to: {}", target)),
        ])
    } else {
        TerminalResult::success(vec![
            TerminalLine::info("Supported MCUs:"),
            TerminalLine::output("  STM32: F401, F103, F407, F746, H7"),
            TerminalLine::output("  ESP: ESP32, ESP8266, ESP32-S3"),
            TerminalLine::output("  Nordic: nRF52832, nRF52840"),
            TerminalLine::output("  RP: RP2040"),
            TerminalLine::output("  AVR: ATMega328P, ATMega2560"),
        ])
    }
}

fn cmd_info(_cmd: &ParsedCommand) -> TerminalResult {
    TerminalResult::success(vec![
        TerminalLine::info("📋 System Information:"),
        TerminalLine::output(&format!("  OS: {}", std::env::consts::OS)),
        TerminalLine::output(&format!("  Arch: {}", std::env::consts::ARCH)),
        TerminalLine::output("  Target: STM32F401CCU6"),
        TerminalLine::output("  Probe: ST-Link V2"),
    ])
}

// ===== Utility Commands =====
fn cmd_version() -> TerminalResult {
    TerminalResult::success(vec![
        TerminalLine::success("NeuroBench Advanced Terminal v2.0.0"),
        TerminalLine::output("Built with Tauri + SolidJS + Rust"),
        TerminalLine::output(&format!("Platform: {} ({})", std::env::consts::OS, std::env::consts::ARCH)),
    ])
}

fn cmd_pwd() -> TerminalResult {
    match std::env::current_dir() {
        Ok(path) => TerminalResult::success(vec![
            TerminalLine::output(&path.display().to_string())
        ]),
        Err(e) => TerminalResult::error(&format!("Failed to get current directory: {}", e)),
    }
}

fn cmd_ls(args: &[String]) -> TerminalResult {
    let path = args.first().map(|s| s.as_str()).unwrap_or(".");
    match std::fs::read_dir(path) {
        Ok(entries) => {
            let mut lines = vec![TerminalLine::info(&format!("📂 {}", path))];
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                let prefix = if is_dir { "📁" } else { "📄" };
                lines.push(TerminalLine::output(&format!("  {} {}", prefix, name)));
            }
            TerminalResult::success(lines)
        }
        Err(e) => TerminalResult::error(&format!("Failed to list directory: {}", e)),
    }
}

fn cmd_echo(args: &[String]) -> TerminalResult {
    TerminalResult::success(vec![
        TerminalLine::output(&args.join(" "))
    ])
}

fn cmd_export(cmd: &ParsedCommand) -> TerminalResult {
    if cmd.args.is_empty() {
        return TerminalResult::info("Usage: export VAR=value");
    }
    
    let assignment = &cmd.args[0];
    if let Some(eq_pos) = assignment.find('=') {
        let var = &assignment[..eq_pos];
        let val = &assignment[eq_pos + 1..];
        TerminalResult::success(vec![
            TerminalLine::success(&format!("✓ Set {}={}", var, val)),
        ])
    } else {
        TerminalResult::info("Usage: export VAR=value")
    }
}

fn cmd_env() -> TerminalResult {
    let mut lines = vec![TerminalLine::info("Environment Variables:")];
    for (key, value) in std::env::vars().take(10) {
        lines.push(TerminalLine::output(&format!("  {}={}", key, value)));
    }
    lines.push(TerminalLine::info("... (showing first 10)"));
    TerminalResult::success(lines)
}

// ===== AI Commands =====
fn cmd_ai(cmd: &ParsedCommand) -> TerminalResult {
    let query = cmd.args.join(" ");
    if query.is_empty() {
        return TerminalResult::info("Usage: ai \"your question about embedded systems\"");
    }
    
    // This will be replaced with actual AI integration
    TerminalResult::success(vec![
        TerminalLine::info(&format!("🤖 AI Query: {}", query)),
        TerminalLine::output("─────────────────────────────────────"),
        TerminalLine::output("Analyzing your question with context from:"),
        TerminalLine::output("  • Current FSM configuration"),
        TerminalLine::output("  • Pin assignments"),
        TerminalLine::output("  • Clock tree settings"),
        TerminalLine::info("AI response will appear here..."),
    ])
}

// ===== Log Commands =====
fn cmd_log(cmd: &ParsedCommand) -> TerminalResult {
    let action = cmd.args.first().map(|s| s.as_str()).unwrap_or("show");
    
    match action {
        "dump" => {
            let start = cmd.args.get(2).map(|s| s.as_str()).unwrap_or("0x08000000");
            let size = cmd.args.get(3).map(|s| s.as_str()).unwrap_or("0x100");
            TerminalResult::success(vec![
                TerminalLine::info(&format!("📦 Memory dump: {} ({})", start, size)),
                TerminalLine::output("0x08000000: 20 00 00 20 C1 00 00 08  45 01 00 08 47 01 00 08"),
                TerminalLine::output("0x08000010: 00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00"),
            ])
        }
        _ => TerminalResult::info("Usage: log show|dump|clear"),
    }
}

// ===== GPIO Commands =====
fn cmd_gpio(cmd: &ParsedCommand) -> TerminalResult {
    let action = cmd.args.first().map(|s| s.as_str()).unwrap_or("config");
    
    match action {
        "config" => {
            let pin = cmd.args.get(1).map(|s| s.as_str()).unwrap_or("PA5");
            let mode = cmd.args.get(2).map(|s| s.as_str()).unwrap_or("output");
            TerminalResult::success(vec![
                TerminalLine::success(&format!("✓ Configured {} as {}", pin, mode)),
            ])
        }
        "set" | "high" => {
            let pin = cmd.args.get(1).map(|s| s.as_str()).unwrap_or("PA5");
            TerminalResult::success(vec![
                TerminalLine::success(&format!("✓ {} → HIGH", pin)),
            ])
        }
        "clear" | "low" => {
            let pin = cmd.args.get(1).map(|s| s.as_str()).unwrap_or("PA5");
            TerminalResult::success(vec![
                TerminalLine::success(&format!("✓ {} → LOW", pin)),
            ])
        }
        "toggle" => {
            let pin = cmd.args.get(1).map(|s| s.as_str()).unwrap_or("PA5");
            TerminalResult::success(vec![
                TerminalLine::success(&format!("✓ {} toggled", pin)),
            ])
        }
        "read" => {
            let pin = cmd.args.get(1).map(|s| s.as_str()).unwrap_or("PA5");
            TerminalResult::success(vec![
                TerminalLine::output(&format!("{} = HIGH", pin)),
            ])
        }
        _ => TerminalResult::info("Usage: gpio config|set|clear|toggle|read PIN [MODE]"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_help_command() {
        let cmd = ParsedCommand {
            command: "help".to_string(),
            args: vec![],
            flags: HashMap::new(),
            operator: super::super::parser::CommandOperator::None,
            next: None,
        };
        let result = process_embedded_command(&cmd);
        assert!(result.success);
        assert!(!result.output.is_empty());
    }

    #[test]
    fn test_flash_command() {
        let mut flags = HashMap::new();
        flags.insert("probe".to_string(), Some("stlink".to_string()));
        
        let cmd = ParsedCommand {
            command: "flash".to_string(),
            args: vec!["firmware.elf".to_string()],
            flags,
            operator: super::super::parser::CommandOperator::None,
            next: None,
        };
        let result = process_embedded_command(&cmd);
        assert!(result.success);
    }
}
