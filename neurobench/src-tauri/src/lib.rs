// NeuroBench - Industrial-Grade Embedded Systems Workbench
// Core FSM Engine and IPC Command Handlers

use tauri::Emitter;
pub mod core;
pub mod commands;
pub mod hal;
pub mod mcu;
pub mod ai;
pub mod drivers;
pub mod terminal;
pub mod agents;
pub mod jobs;
pub mod validation;
pub mod git;
pub mod qemu;
pub mod cloud;
pub mod templates;
pub mod snippets;
pub mod memory;
pub mod power;
pub mod pins;
pub mod build;
pub mod serial;
pub mod docs;
pub mod profiler;
pub mod registers;
pub mod performance;
pub mod toolchain;

#[cfg(test)]
mod tests;

use ai::AIService;
use core::*;
use mcu::registry;
use drivers::templates::*;
use terminal::{TerminalResult, TerminalLine};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;
use serde::{Serialize, Deserialize};

/// Application state - persists across IPC calls
pub struct AppState {
    pub orchestrator: Arc<Mutex<agents::Orchestrator>>,
    pub build_manager: Arc<toolchain::streaming_build::BuildManager>,
    pub job_manager: Arc<jobs::JobManager>,
    pub tool_registry: Arc<Mutex<agents::ToolRegistry>>,
    pub audit_log: Arc<Mutex<agents::AuditLog>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            orchestrator: Arc::new(Mutex::new(agents::Orchestrator::new())),
            build_manager: Arc::new(toolchain::streaming_build::BuildManager::new()),
            job_manager: Arc::new(jobs::JobManager::new()),
            tool_registry: Arc::new(Mutex::new(agents::create_default_registry())),
            audit_log: Arc::new(Mutex::new(agents::AuditLog::new())),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    env_logger::init();
    log::info!("NeuroBench starting...");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Project commands
            commands::project::create_project,
            commands::project::save_project,
            commands::project::load_project,
            commands::project::list_projects,
            
            // FSM commands
            commands::fsm::add_node,
            commands::fsm::remove_node,
            commands::fsm::update_node,
            commands::fsm::add_edge,
            commands::fsm::remove_edge,
            commands::fsm::update_edge,
            commands::fsm::simulate_step,
            commands::fsm::simulate_run,
            commands::fsm::simulate_stop,
            
            // Code generation
            commands::codegen::generate_code,
            commands::codegen::get_supported_targets,
            
            // Hardware commands
            commands::hardware::detect_devices,
            commands::hardware::connect_device,
            commands::hardware::disconnect_device,
            commands::hardware::flash_firmware,
            commands::hardware::read_telemetry,
            
            // AI commands
            ai_chat,
            ai_status,
            ai_generate_code,
            ai_parse_fsm,
            
            // Serial port & MCU
            list_serial_ports,
            get_mcu_list,
            
            // Driver generation
            generate_gpio_driver,
            generate_uart_driver,
            generate_spi_driver,
            generate_i2c_driver,
            generate_can_driver,
            generate_modbus_driver,
            generate_rtos_code,
            generate_driver_ai,
            get_peripherals_list,
            get_mcu_pinout,
            
            // Interrupt & Timer generation
            generate_interrupt_code,
            generate_timer_code,
            generate_ticker_code,
            
            // Clock & Power generation
            generate_clock_config,
            generate_low_power_code,
            calculate_clock_frequencies,
            
            // Analog I/O generation
            generate_adc_code,
            generate_dac_code,
            generate_pwm_code,
            
            // Multi-MCU support
            get_supported_mcus,
            get_mcu_info,
            generate_mcu_gpio,
            generate_mcu_peripheral,
            
            // RTOS generation
            generate_rtos_task,
            generate_rtos_semaphore,
            generate_rtos_mutex,
            generate_rtos_queue,
            generate_rtos_timer,
            generate_rtos_config,
            
            // Wireless generation
            generate_ble_service,
            generate_wifi_config,
            generate_lora_config,
            
            // DSP generation
            generate_fir_filter,
            generate_iir_filter,
            generate_fft_block,
            generate_pid_controller,
            generate_circular_buffer,
            
            // Security generation
            generate_bootloader,
            generate_ota_client,
            generate_secure_boot,
            generate_crypto_utils,
            
            // Export commands
            export_code_to_file,
            generate_project_cmake,
            generate_project_makefile,
            
            // Terminal commands
            execute_terminal_command,
            get_terminal_welcome,
            
            // Agent commands
            list_agents,
            get_active_agent,
            set_active_agent,
            agent_chat,
            execute_tool,
            update_fsm_context,
            
            // Project persistence
            save_project_file,
            load_project_file,
            
            // System info
            get_system_info,
            
            // Code validation
            validate_code,
            
            // Git integration
            git_init,
            git_status,
            git_stage_files,
            git_stage_all,
            git_commit,
            git_history,
            git_diff,
            
            // QEMU simulation
            qemu_check,
            qemu_version,
            qemu_list_machines,
            qemu_get_presets,
            
            // Cloud sync
            cloud_export_project,
            cloud_import_project,
            cloud_generate_share_id,
            cloud_collect_files,
            
            // Templates
            templates_get_all,
            templates_get_by_id,
            templates_get_categories,
            
            // Snippets
            snippets_get_all,
            snippets_search,
            snippets_get_by_id,
            
            // Memory analyzer
            memory_estimate,
            memory_get_mcu_configs,
            
            // Power estimator
            power_estimate,
            power_get_mcu_specs,
            
            // Pin configuration
            pins_get_packages,
            pins_generate_code,
            
            // Build system
            build_generate_makefile,
            build_generate_cmake,
            build_check_toolchain,
            
            // Serial monitor
            serial_list_ports,
            serial_get_baud_rates,
            serial_format_data,
            serial_parse_escape,
            serial_calculate_checksum,
            
            // Documentation generator
            docs_generate,
            docs_generate_doxyfile,
            docs_extract_functions,
            
            // Profiler
            profiler_analyze,
            profiler_estimate_timing,
            
            // Registers
            registers_get_peripherals,
            registers_get_gpio,
            registers_generate_code,
            
            // Advanced Terminal
            terminal_execute_advanced,
            terminal_get_completions,
            terminal_get_themes,
            terminal_get_welcome,
            terminal_parse_command,
            
            // Performance Monitor
            performance_get_system_metrics,
            performance_get_process_list,
            performance_get_embedded_metrics,
            
            // Toolchain & IDE Loop
            toolchain_discover,
            toolchain_build,
            toolchain_clean,
            toolchain_size_report,
            toolchain_parse_map,
            probe_list,
            probe_connect,
            probe_disconnect,
            probe_flash,
            probe_reset,
            probe_halt,
            probe_resume,
            probe_read_memory,
            probe_read_registers,
            rtt_start,
            rtt_read,
            rtt_stop,
            decode_hardfault,
            
            // Streaming Build (live output + cancel + logs + artifacts)
            streaming_build_start,
            streaming_build_cancel,
            streaming_build_list,
            streaming_build_get_log,
            streaming_build_get_diagnostics,
            streaming_build_get_latest_artifacts,
            streaming_build_get_artifacts,
            
            // Flash (live progress + cancel)
            flash_start,
            flash_cancel,
            
            // RTT Job Streaming (batched events + stop)
            rtt_stream_start,
            rtt_stream_stop,
            
            // Generic Job Management
            job_list,
            job_get_status,
            job_get_log,
            job_cancel,
            
            // Run Chain (build → flash → rtt)
            run_chain,
            
            // Device Status
            device_status_get,
            workflow_cancel,
            
            // Tool Registry
            tool_list,
            tool_execute,
            tool_get_schemas,
            
            // Patch/Audit System
            patch_propose,
            patch_apply,
            patch_reject,
            patch_get_pending,
            
            // AI Model Management
            ai_get_providers,
            ai_set_provider,
        ])
        .manage(AppState::new())
        .run(tauri::generate_context!())
        .expect("error while running NeuroBench");
}

/// Get system information
#[tauri::command]
fn get_system_info() -> serde_json::Value {
    let ai_service = AIService::new();
    serde_json::json!({
        "name": "NeuroBench",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Industrial-Grade Embedded Systems Workbench",
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "ai_available": ai_service.is_available(),
    })
}

/// Chat with AI assistant
#[tauri::command]
async fn ai_chat(message: String) -> Result<String, String> {
    let service = AIService::new();
    if !service.is_available() {
        return Err("AI not configured. Set GEMINI_API_KEY environment variable.".to_string());
    }
    service.chat(&message, None).await
}

/// Check AI status
#[tauri::command]
fn ai_status() -> serde_json::Value {
    let service = AIService::new();
    serde_json::json!({
        "available": service.is_available(),
        "provider": "Gemini",
    })
}

/// Generate FSM code using AI
#[tauri::command]
async fn ai_generate_code(
    nodes: Vec<FSMNode>,
    edges: Vec<FSMEdge>,
    language: String,
) -> Result<String, String> {
    let service = AIService::new();
    if !service.is_available() {
        return Err("AI not configured. Set GEMINI_API_KEY environment variable.".to_string());
    }
    service.generate_fsm_code(&nodes, &edges, &language).await
}

/// Parse natural language into FSM graph
#[tauri::command]
async fn ai_parse_fsm(description: String) -> Result<String, String> {
    let service = AIService::new();
    if !service.is_available() {
        return Err("AI not configured. Set GEMINI_API_KEY environment variable.".to_string());
    }
    service.parse_fsm_from_description(&description).await
}

/// List available serial ports
#[tauri::command]
fn list_serial_ports() -> Result<Vec<serde_json::Value>, String> {
    match serialport::available_ports() {
        Ok(ports) => {
            let result: Vec<serde_json::Value> = ports.iter().map(|p| {
                let port_type = match &p.port_type {
                    serialport::SerialPortType::UsbPort(info) => {
                        serde_json::json!({
                            "type": "USB",
                            "vid": format!("{:04X}", info.vid),
                            "pid": format!("{:04X}", info.pid),
                            "manufacturer": info.manufacturer.clone().unwrap_or_default(),
                            "product": info.product.clone().unwrap_or_default(),
                            "serial": info.serial_number.clone().unwrap_or_default(),
                        })
                    },
                    serialport::SerialPortType::PciPort => serde_json::json!({"type": "PCI"}),
                    serialport::SerialPortType::BluetoothPort => serde_json::json!({"type": "Bluetooth"}),
                    serialport::SerialPortType::Unknown => serde_json::json!({"type": "Unknown"}),
                };
                serde_json::json!({
                    "name": p.port_name,
                    "info": port_type,
                })
            }).collect();
            log::info!("Found {} serial ports", result.len());
            Ok(result)
        },
        Err(e) => Err(format!("Failed to list ports: {}", e)),
    }
}

/// Get list of supported MCUs
#[tauri::command]
fn get_mcu_list() -> Vec<serde_json::Value> {
    registry::get_all_mcus().iter().map(|mcu| {
        serde_json::json!({
            "id": mcu.id,
            "name": mcu.name,
            "family": format!("{:?}", mcu.family),
            "arch": format!("{:?}", mcu.arch),
            "flash_kb": mcu.specs.flash_kb,
            "ram_kb": mcu.specs.ram_kb,
            "freq_mhz": mcu.specs.freq_mhz,
        })
    }).collect()
}

// ==================== Driver Generation Commands ====================

/// Generate GPIO driver
#[tauri::command]
fn generate_gpio_driver(
    port: String,
    pin: u8,
    mode: String,
    language: String,
) -> Result<serde_json::Value, String> {
    let gpio_mode = match mode.to_lowercase().as_str() {
        "input" => GpioMode::Input,
        "output" => GpioMode::Output,
        "alternate" | "af" => GpioMode::AlternateFunction,
        "analog" => GpioMode::Analog,
        _ => GpioMode::Output,
    };
    
    let config = GpioConfig {
        port,
        pin,
        mode: gpio_mode,
        pull: GpioPull::None,
        speed: GpioSpeed::High,
        alternate_function: None,
        initial_state: None,
    };
    
    let lang = match language.to_lowercase().as_str() {
        "c" => DriverLanguage::C,
        "cpp" | "c++" => DriverLanguage::Cpp,
        "rust" => DriverLanguage::Rust,
        _ => DriverLanguage::C,
    };
    
    let output = drivers::gpio::generate_gpio_driver(&config, &McuArch::Stm32, &lang);
    
    Ok(serde_json::json!({
        "header": output.header_file,
        "source": output.source_file,
        "example": output.example_file,
        "peripheral": "GPIO",
    }))
}

/// Generate UART driver
#[tauri::command]
fn generate_uart_driver(
    instance: String,
    baud_rate: u32,
    use_dma: bool,
    language: String,
) -> Result<serde_json::Value, String> {
    let config = UartConfig {
        instance,
        baud_rate,
        data_bits: 8,
        stop_bits: StopBits::One,
        parity: Parity::None,
        flow_control: false,
        tx_pin: None,
        rx_pin: None,
        use_dma,
        use_interrupt: true,
    };
    
    let lang = match language.to_lowercase().as_str() {
        "c" => DriverLanguage::C,
        "cpp" | "c++" => DriverLanguage::Cpp,
        "rust" => DriverLanguage::Rust,
        _ => DriverLanguage::C,
    };
    
    let output = drivers::uart::generate_uart_driver(&config, &McuArch::Stm32, &lang);
    
    Ok(serde_json::json!({
        "header": output.header_file,
        "source": output.source_file,
        "example": output.example_file,
        "peripheral": "UART",
    }))
}

/// Generate SPI driver
#[tauri::command]
fn generate_spi_driver(
    instance: String,
    clock_hz: u32,
    mode: u8,
    language: String,
) -> Result<serde_json::Value, String> {
    use drivers::templates::SpiMode;
    
    let spi_mode = match mode {
        0 => SpiMode::Mode0,
        1 => SpiMode::Mode1,
        2 => SpiMode::Mode2,
        3 => SpiMode::Mode3,
        _ => SpiMode::Mode0,
    };
    
    let config = SpiConfig {
        instance,
        mode: spi_mode,
        clock_hz,
        data_bits: 8,
        msb_first: true,
        bit_order: BitOrder::MsbFirst,
        mosi_pin: None,
        miso_pin: None,
        sck_pin: None,
        cs_pin: None,
    };
    
    let lang = match language.to_lowercase().as_str() {
        "c" => DriverLanguage::C,
        "cpp" | "c++" => DriverLanguage::Cpp,
        "rust" => DriverLanguage::Rust,
        _ => DriverLanguage::C,
    };
    
    let output = drivers::spi::generate_spi_driver(&config, &McuArch::Stm32, &lang);
    
    Ok(serde_json::json!({
        "header": output.header_file,
        "source": output.source_file,
        "example": output.example_file,
        "peripheral": "SPI",
    }))
}

/// Generate I2C driver
#[tauri::command]
fn generate_i2c_driver(
    instance: String,
    speed: String,
    language: String,
) -> Result<serde_json::Value, String> {
    let i2c_speed = match speed.to_lowercase().as_str() {
        "standard" | "100k" => I2cSpeed::Standard,
        "fast" | "400k" => I2cSpeed::Fast,
        "fastplus" | "fast+" | "1m" => I2cSpeed::FastPlus,
        _ => I2cSpeed::Standard,
    };
    
    let config = I2cConfig {
        instance,
        speed: i2c_speed,
        address_bits: 7,
        address: None,
        sda_pin: None,
        scl_pin: None,
    };
    
    let lang = match language.to_lowercase().as_str() {
        "c" => DriverLanguage::C,
        "cpp" | "c++" => DriverLanguage::Cpp,
        "rust" => DriverLanguage::Rust,
        _ => DriverLanguage::C,
    };
    
    let output = drivers::i2c::generate_i2c_driver(&config, &McuArch::Stm32, &lang);
    
    Ok(serde_json::json!({
        "header": output.header_file,
        "source": output.source_file,
        "example": output.example_file,
        "peripheral": "I2C",
    }))
}

/// Generate driver using AI
#[tauri::command]
async fn generate_driver_ai(
    peripheral: String,
    description: String,
    mcu: String,
    language: String,
) -> Result<serde_json::Value, String> {
    let output = drivers::generate_driver_with_ai(&peripheral, &description, &mcu, &language).await?;
    
    Ok(serde_json::json!({
        "header": output.header_file,
        "source": output.source_file,
        "example": output.example_file,
        "peripheral": peripheral,
    }))
}

/// Get list of supported peripherals
#[tauri::command]
fn get_peripherals_list() -> serde_json::Value {
    serde_json::json!({
        "peripherals": drivers::get_supported_peripherals(),
        "architectures": drivers::get_supported_architectures(),
    })
}

/// Generate CAN driver
#[tauri::command]
fn generate_can_driver(
    instance: String,
    bitrate: u32,
    mode: String,
    language: String,
) -> Result<serde_json::Value, String> {
    use drivers::can::{CanConfig, CanMode, generate_can_driver as gen_can};
    
    let can_mode = match mode.to_lowercase().as_str() {
        "loopback" => CanMode::Loopback,
        "silent" => CanMode::Silent,
        _ => CanMode::Normal,
    };
    
    let config = CanConfig {
        instance,
        bitrate,
        mode: can_mode,
        tx_pin: None,
        rx_pin: None,
    };
    
    let lang = match language.to_lowercase().as_str() {
        "c" => DriverLanguage::C,
        "cpp" | "c++" => DriverLanguage::Cpp,
        "rust" => DriverLanguage::Rust,
        _ => DriverLanguage::C,
    };
    
    let output = gen_can(&config, &McuArch::Stm32, &lang);
    
    Ok(serde_json::json!({
        "header": output.header_file,
        "source": output.source_file,
        "example": output.example_file,
        "peripheral": "CAN",
    }))
}

/// Generate Modbus driver
#[tauri::command]
fn generate_modbus_driver(
    uart_instance: String,
    baud_rate: u32,
    address: u8,
    mode: String,
    language: String,
) -> Result<serde_json::Value, String> {
    use drivers::modbus::{ModbusConfig, ModbusMode, generate_modbus_driver as gen_modbus};
    
    let modbus_mode = match mode.to_lowercase().as_str() {
        "slave" => ModbusMode::RtuSlave,
        _ => ModbusMode::RtuMaster,
    };
    
    let config = ModbusConfig {
        mode: modbus_mode,
        address,
        uart_instance,
        baud_rate,
    };
    
    let lang = match language.to_lowercase().as_str() {
        "c" => DriverLanguage::C,
        "cpp" | "c++" => DriverLanguage::Cpp,
        "rust" => DriverLanguage::Rust,
        _ => DriverLanguage::C,
    };
    
    let output = gen_modbus(&config, &McuArch::Stm32, &lang);
    
    Ok(serde_json::json!({
        "header": output.header_file,
        "source": output.source_file,
        "example": output.example_file,
        "peripheral": "Modbus",
    }))
}

/// Get MCU pinout for visual configurator
#[tauri::command]
fn get_mcu_pinout(mcu_id: String) -> Result<serde_json::Value, String> {
    use drivers::pins::get_mcu_pinout as get_pinout;
    
    match get_pinout(&mcu_id) {
        Some(pinout) => Ok(serde_json::json!({
            "mcu_id": pinout.mcu_id,
            "name": pinout.name,
            "package": format!("{:?}", pinout.package),
            "pins": pinout.pins.iter().map(|p| {
                serde_json::json!({
                    "port": p.port,
                    "pin": p.pin,
                    "name": p.name,
                    "functions": p.functions.iter().map(|f| format!("{:?}", f)).collect::<Vec<_>>(),
                    "currentFunction": p.current_function.as_ref().map(|f| format!("{:?}", f)),
                    "x": p.x,
                    "y": p.y,
                })
            }).collect::<Vec<_>>(),
        })),
        None => Err(format!("Unknown MCU: {}", mcu_id)),
    }
}

/// Generate RTOS code
#[tauri::command]
fn generate_rtos_code(
    tasks: Vec<serde_json::Value>,
    heap_size_kb: u32,
    language: String,
) -> Result<serde_json::Value, String> {
    use drivers::rtos::{RtosConfig, RtosType, TaskConfig, generate_rtos_code as gen_rtos};
    
    let task_configs: Vec<TaskConfig> = tasks.iter().map(|t| {
        TaskConfig {
            name: t.get("name").and_then(|v| v.as_str()).unwrap_or("Task").to_string(),
            priority: t.get("priority").and_then(|v| v.as_u64()).unwrap_or(1) as u8,
            stack_size: t.get("stackSize").and_then(|v| v.as_u64()).unwrap_or(256) as u32,
            period_ms: t.get("periodMs").and_then(|v| v.as_u64()).map(|v| v as u32),
            handler_name: t.get("handler").and_then(|v| v.as_str())
                .unwrap_or(&format!("{}_Handler", t.get("name").and_then(|v| v.as_str()).unwrap_or("Task"))).to_string(),
        }
    }).collect();
    
    let config = RtosConfig {
        rtos_type: RtosType::FreeRtos,
        tasks: task_configs,
        use_queues: true,
        use_semaphores: true,
        use_mutexes: true,
        heap_size_kb,
    };
    
    let lang = match language.to_lowercase().as_str() {
        "c" => DriverLanguage::C,
        "cpp" | "c++" => DriverLanguage::Cpp,
        "rust" => DriverLanguage::Rust,
        _ => DriverLanguage::C,
    };
    
    let output = gen_rtos(&config, &lang);
    
    Ok(serde_json::json!({
        "header": output.header_file,
        "source": output.source_file,
        "example": output.example_file,
        "peripheral": "RTOS",
    }))
}

/// Execute a terminal command via the Rust backend (legacy API - kept for compatibility)
#[tauri::command]
fn execute_terminal_command(command: String, args: Vec<String>) -> terminal::TerminalResult {
    // Create a parsed command from legacy format
    let parsed = terminal::parser::ParsedCommand {
        command: command.clone(),
        args,
        flags: std::collections::HashMap::new(),
        operator: terminal::parser::CommandOperator::None,
        next: None,
    };
    terminal::commands::process_embedded_command(&parsed)
}

/// Get terminal welcome message
#[tauri::command]
fn get_terminal_welcome() -> Vec<terminal::TerminalLine> {
    terminal::get_welcome_message()
}

/// List available agents
#[tauri::command]
async fn list_agents(state: State<'_, AppState>) -> Result<Vec<agents::AgentInfo>, String> {
    let orchestrator = state.orchestrator.lock().await;
    Ok(orchestrator.list_agents())
}

/// Get currently active agent
#[tauri::command]
async fn get_active_agent(state: State<'_, AppState>) -> Result<Option<agents::AgentInfo>, String> {
    let orchestrator = state.orchestrator.lock().await;
    Ok(orchestrator.get_active_agent())
}

/// Set active agent
#[tauri::command]
async fn set_active_agent(state: State<'_, AppState>, agent_id: String) -> Result<(), String> {
    let mut orchestrator = state.orchestrator.lock().await;
    orchestrator.set_active_agent(&agent_id)
}

/// Chat with active agent
#[tauri::command]
async fn agent_chat(state: State<'_, AppState>, message: String) -> Result<agents::AgentResponse, String> {
    let orch = state.orchestrator.clone();
    // Drop the State reference before await
    drop(state);
    let orchestrator = orch.lock().await;
    orchestrator.process(&message).await
}

/// Execute a tool call from an agent
#[tauri::command]
fn execute_tool(tool: String, params: serde_json::Value) -> agents::ToolResult {
    agents::ToolExecutor::execute(&tool, &params)
}

/// Update FSM context in agent state (sync FSM canvas to agents)
#[tauri::command]
async fn update_fsm_context(
    state: State<'_, AppState>,
    nodes: Vec<agents::ContextNode>,
    edges: Vec<agents::ContextEdge>,
    selected_node: Option<String>,
) -> Result<(), String> {
    let mut orchestrator = state.orchestrator.lock().await;
    orchestrator.update_context(nodes, edges, selected_node).await;
    Ok(())
}

/// Project data for save/load
#[derive(Serialize, Deserialize)]
pub struct ProjectData {
    pub name: String,
    pub nodes: Vec<serde_json::Value>,
    pub edges: Vec<serde_json::Value>,
    pub mcu: String,
    pub language: String,
}

/// Save project to file
#[tauri::command]
fn save_project_file(path: String, project: ProjectData) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&project)
        .map_err(|e| format!("Serialization error: {}", e))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("File write error: {}", e))?;
    log::info!("Project saved to: {}", path);
    Ok(())
}

/// Load project from file
#[tauri::command]
fn load_project_file(path: String) -> Result<ProjectData, String> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("File read error: {}", e))?;
    let project: ProjectData = serde_json::from_str(&content)
        .map_err(|e| format!("Parse error: {}", e))?;
    log::info!("Project loaded from: {}", path);
    Ok(project)
}

/// Generate EXTI interrupt initialization code
#[tauri::command]
fn generate_interrupt_code(
    pin: String,
    edge: String,
    priority: u8,
    debounce_ms: u32,
    handler_name: String,
    handler_code: String,
) -> Result<serde_json::Value, String> {
    use drivers::interrupts::{InterruptConfig, InterruptEdge, generate_exti_init};
    
    let edge_type = match edge.to_lowercase().as_str() {
        "falling" => InterruptEdge::Falling,
        "both" => InterruptEdge::Both,
        _ => InterruptEdge::Rising,
    };
    
    let config = InterruptConfig {
        pin,
        edge: edge_type,
        priority,
        debounce_ms,
        handler_name,
        handler_code,
    };
    
    let code = generate_exti_init(&config, "STM32F4");
    
    Ok(serde_json::json!({
        "code": code,
        "config": config,
    }))
}

/// Generate Timer initialization code
#[tauri::command]
fn generate_timer_code(
    instance: String,
    prescaler: u32,
    period: u32,
    auto_reload: bool,
    interrupt_enabled: bool,
    handler_name: String,
    handler_code: String,
    clock_hz: Option<u32>,
) -> Result<serde_json::Value, String> {
    use drivers::interrupts::{TimerConfig, generate_timer_init};
    
    let config = TimerConfig {
        instance,
        prescaler,
        period,
        auto_reload,
        interrupt_enabled,
        handler_name,
        handler_code,
    };
    
    let clock = clock_hz.unwrap_or(84_000_000); // Default STM32F4 @ 84MHz
    let code = generate_timer_init(&config, clock);
    let frequency = clock as f64 / (prescaler as f64 * period as f64);
    
    Ok(serde_json::json!({
        "code": code,
        "config": config,
        "frequency_hz": frequency,
    }))
}

/// Generate Ticker (SysTick) code
#[tauri::command]
fn generate_ticker_code(
    name: String,
    interval_ms: u32,
    callback_code: String,
) -> Result<serde_json::Value, String> {
    use drivers::interrupts::{TickerConfig, generate_ticker};
    
    let config = TickerConfig {
        name,
        interval_ms,
        callback_code,
    };
    
    let code = generate_ticker(&config);
    
    Ok(serde_json::json!({
        "code": code,
        "config": config,
    }))
}

/// Generate clock configuration code
#[tauri::command]
fn generate_clock_config(
    sysclk_source: String,
    hse_freq_hz: Option<u32>,
    pllm: u32,
    plln: u32,
    pllp: u32,
    pllq: u32,
    ahb_prescaler: u32,
    apb1_prescaler: u32,
    apb2_prescaler: u32,
) -> Result<serde_json::Value, String> {
    use drivers::clock::{ClockConfig, ClockSource, PllConfig, BusClocksConfig, 
                         generate_clock_init, calculate_clocks};
    
    let source = match sysclk_source.to_lowercase().as_str() {
        "hsi" => ClockSource::HSI,
        "hse" => ClockSource::HSE,
        "pll" => ClockSource::PLL,
        _ => ClockSource::PLL,
    };
    
    let config = ClockConfig {
        sysclk_source: source,
        hse_freq_hz,
        pll: PllConfig {
            source: if hse_freq_hz.is_some() { ClockSource::HSE } else { ClockSource::HSI },
            pllm,
            plln,
            pllp,
            pllq,
        },
        bus_clocks: BusClocksConfig {
            ahb_prescaler,
            apb1_prescaler,
            apb2_prescaler,
        },
    };
    
    let freqs = calculate_clocks(&config);
    let code = generate_clock_init(&config);
    
    Ok(serde_json::json!({
        "code": code,
        "frequencies": {
            "sysclk_mhz": freqs.sysclk / 1_000_000,
            "hclk_mhz": freqs.hclk / 1_000_000,
            "pclk1_mhz": freqs.pclk1 / 1_000_000,
            "pclk2_mhz": freqs.pclk2 / 1_000_000,
            "pll_48_mhz": freqs.pll_48_clk / 1_000_000,
        },
    }))
}

/// Generate low-power mode code
#[tauri::command]
fn generate_low_power_code(
    mode: String,
    wakeup_pin: bool,
    rtc_alarm: bool,
    rtc_wakeup: bool,
    external_interrupt: Option<String>,
) -> Result<serde_json::Value, String> {
    use drivers::clock::{LowPowerMode, WakeupConfig, 
                         generate_low_power_code as gen_lp, estimate_power, ClockConfig};
    
    let lp_mode = match mode.to_lowercase().as_str() {
        "sleep" => LowPowerMode::Sleep,
        "stop" => LowPowerMode::Stop,
        "standby" => LowPowerMode::Standby,
        _ => LowPowerMode::Sleep,
    };
    
    let wakeup = WakeupConfig {
        wakeup_pin,
        rtc_alarm,
        rtc_wakeup,
        external_interrupt,
    };
    
    let code = gen_lp(lp_mode, &wakeup);
    let power = estimate_power(&ClockConfig::default(), lp_mode);
    
    Ok(serde_json::json!({
        "code": code,
        "mode": mode,
        "power_estimate": {
            "run_mode_ma": power.run_mode_ma,
            "sleep_mode_ma": power.sleep_mode_ma,
            "stop_mode_ua": power.stop_mode_ua,
            "standby_mode_ua": power.standby_mode_ua,
        },
    }))
}

/// Calculate clock frequencies from PLL parameters
#[tauri::command]
fn calculate_clock_frequencies(
    pll_source: String,
    hse_freq_hz: Option<u32>,
    pllm: u32,
    plln: u32,
    pllp: u32,
    pllq: u32,
    ahb_prescaler: u32,
    apb1_prescaler: u32,
    apb2_prescaler: u32,
) -> Result<serde_json::Value, String> {
    use drivers::clock::{ClockConfig, ClockSource, PllConfig, BusClocksConfig, calculate_clocks};
    
    let source = match pll_source.to_lowercase().as_str() {
        "hsi" => ClockSource::HSI,
        "hse" => ClockSource::HSE,
        _ => ClockSource::HSI,
    };
    
    let config = ClockConfig {
        sysclk_source: ClockSource::PLL,
        hse_freq_hz,
        pll: PllConfig {
            source,
            pllm,
            plln,
            pllp,
            pllq,
        },
        bus_clocks: BusClocksConfig {
            ahb_prescaler,
            apb1_prescaler,
            apb2_prescaler,
        },
    };
    
    let freqs = calculate_clocks(&config);
    
    Ok(serde_json::json!({
        "sysclk_hz": freqs.sysclk,
        "sysclk_mhz": freqs.sysclk / 1_000_000,
        "hclk_mhz": freqs.hclk / 1_000_000,
        "pclk1_mhz": freqs.pclk1 / 1_000_000,
        "pclk2_mhz": freqs.pclk2 / 1_000_000,
        "pll_48_mhz": freqs.pll_48_clk / 1_000_000,
        "usb_valid": freqs.pll_48_clk == 48_000_000,
    }))
}

/// Generate ADC initialization code
#[tauri::command]
fn generate_adc_code(
    instance: String,
    resolution: u8,
    channels: Vec<serde_json::Value>,
    continuous_mode: bool,
    dma_enabled: bool,
) -> Result<serde_json::Value, String> {
    use drivers::analog::{AdcConfig, AdcChannelConfig, AdcResolution, AdcSampleTime, generate_adc_init};
    
    let res = match resolution {
        8 => AdcResolution::Bits8,
        10 => AdcResolution::Bits10,
        _ => AdcResolution::Bits12,
    };
    
    let channel_configs: Vec<AdcChannelConfig> = channels.iter().map(|ch| {
        AdcChannelConfig {
            channel: ch.get("channel").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
            sample_time: AdcSampleTime::Cycles84,
            gpio_pin: ch.get("pin").and_then(|v| v.as_str()).unwrap_or("PA0").to_string(),
        }
    }).collect();
    
    let config = AdcConfig {
        instance: instance.clone(),
        resolution: res,
        channels: channel_configs,
        continuous_mode,
        dma_enabled,
        scan_mode: channels.len() > 1,
    };
    
    let code = generate_adc_init(&config, 84_000_000);
    
    Ok(serde_json::json!({
        "code": code,
        "instance": instance,
        "resolution_bits": resolution,
        "num_channels": channels.len(),
    }))
}

/// Generate DAC initialization code
#[tauri::command]
fn generate_dac_code(
    channel: u8,
    output_buffer: bool,
    trigger_enabled: bool,
    waveform: String,
) -> Result<serde_json::Value, String> {
    use drivers::analog::{DacConfig, DacWaveform, generate_dac_init};
    
    let wave = match waveform.to_lowercase().as_str() {
        "noise" => DacWaveform::Noise,
        "triangle" => DacWaveform::Triangle,
        _ => DacWaveform::None,
    };
    
    let config = DacConfig {
        channel,
        output_buffer,
        trigger_enabled,
        waveform: wave,
        amplitude: 0,
    };
    
    let code = generate_dac_init(&config);
    
    Ok(serde_json::json!({
        "code": code,
        "channel": channel,
        "output_pin": if channel == 1 { "PA4" } else { "PA5" },
    }))
}

/// Generate PWM initialization code
#[tauri::command]
fn generate_pwm_code(
    timer: String,
    frequency_hz: u32,
    channels: Vec<serde_json::Value>,
    center_aligned: bool,
) -> Result<serde_json::Value, String> {
    use drivers::analog::{PwmConfig, PwmChannelConfig, PwmMode, generate_pwm_init};
    
    let mode = if center_aligned { PwmMode::CenterAligned } else { PwmMode::EdgeAligned };
    
    let channel_configs: Vec<PwmChannelConfig> = channels.iter().map(|ch| {
        PwmChannelConfig {
            channel: ch.get("channel").and_then(|v| v.as_u64()).unwrap_or(1) as u8,
            duty_cycle_percent: ch.get("duty").and_then(|v| v.as_f64()).unwrap_or(50.0) as f32,
            gpio_pin: ch.get("pin").and_then(|v| v.as_str()).unwrap_or("PA0").to_string(),
            polarity_high: ch.get("polarity_high").and_then(|v| v.as_bool()).unwrap_or(true),
        }
    }).collect();
    
    let config = PwmConfig {
        timer: timer.clone(),
        frequency_hz,
        mode,
        channels: channel_configs,
        dead_time_ns: None,
    };
    
    let code = generate_pwm_init(&config, 84_000_000);
    
    Ok(serde_json::json!({
        "code": code,
        "timer": timer,
        "frequency_hz": frequency_hz,
        "num_channels": channels.len(),
    }))
}

/// Get all supported MCUs
#[tauri::command]
fn get_supported_mcus() -> Result<serde_json::Value, String> {
    use drivers::mcu::get_all_mcus;
    
    let mcus = get_all_mcus();
    let mcu_list: Vec<serde_json::Value> = mcus.iter().map(|m| {
        serde_json::json!({
            "family": format!("{:?}", m.family),
            "display_name": m.display_name,
            "vendor": m.vendor,
            "architecture": m.architecture,
            "max_freq_mhz": m.max_freq_mhz,
            "flash_kb": m.flash_kb,
            "ram_kb": m.ram_kb,
            "has_fpu": m.has_fpu,
            "has_dsp": m.has_dsp,
            "has_ble": m.has_ble,
            "has_wifi": m.has_wifi,
        })
    }).collect();
    
    Ok(serde_json::json!({
        "mcus": mcu_list,
        "count": mcu_list.len(),
    }))
}

/// Get detailed info for a specific MCU
#[tauri::command]
fn get_mcu_info(family: String) -> Result<serde_json::Value, String> {
    use drivers::mcu::{McuFamily, McuInfo};
    
    let mcu_family = match family.as_str() {
        "STM32F1" => McuFamily::STM32F1,
        "STM32F4" => McuFamily::STM32F4,
        "STM32H7" => McuFamily::STM32H7,
        "STM32L4" => McuFamily::STM32L4,
        "STM32G4" => McuFamily::STM32G4,
        "ESP32" => McuFamily::ESP32,
        "ESP32S3" => McuFamily::ESP32S3,
        "ESP32C3" => McuFamily::ESP32C3,
        "RP2040" => McuFamily::RP2040,
        "NRF52832" => McuFamily::NRF52832,
        "NRF52840" => McuFamily::NRF52840,
        "LPC1768" => McuFamily::LPC1768,
        "LPC5500" => McuFamily::LPC5500,
        _ => return Err(format!("Unknown MCU family: {}", family)),
    };
    
    let info: McuInfo = mcu_family.into();
    
    Ok(serde_json::json!({
        "family": format!("{:?}", info.family),
        "display_name": info.display_name,
        "vendor": info.vendor,
        "architecture": info.architecture,
        "max_freq_mhz": info.max_freq_mhz,
        "flash_kb": info.flash_kb,
        "ram_kb": info.ram_kb,
        "has_fpu": info.has_fpu,
        "has_dsp": info.has_dsp,
        "has_ble": info.has_ble,
        "has_wifi": info.has_wifi,
    }))
}

/// Generate GPIO code for specific MCU
#[tauri::command]
fn generate_mcu_gpio(
    family: String,
    pin: String,
    mode: String,
    pull: String,
    initial_state: Option<bool>,
) -> Result<serde_json::Value, String> {
    use drivers::mcu::{McuFamily, McuHal, GpioConfig, GpioMode, GpioPull, GpioSpeed};
    use drivers::mcu::stm32::Stm32Hal;
    use drivers::mcu::esp32::Esp32Hal;
    use drivers::mcu::rp2040::Rp2040Hal;
    use drivers::mcu::nordic::NordicHal;
    use drivers::mcu::nxp::NxpHal;
    
    let gpio_mode = match mode.to_lowercase().as_str() {
        "input" => GpioMode::Input,
        "output" => GpioMode::Output,
        "analog" => GpioMode::Analog,
        _ => GpioMode::Output,
    };
    
    let gpio_pull = match pull.to_lowercase().as_str() {
        "up" => GpioPull::Up,
        "down" => GpioPull::Down,
        _ => GpioPull::None,
    };
    
    let config = GpioConfig {
        pin: pin.clone(),
        mode: gpio_mode,
        pull: gpio_pull,
        speed: GpioSpeed::High,
        initial_state,
    };
    
    let code = match family.as_str() {
        "STM32F1" => Stm32Hal::new(McuFamily::STM32F1).generate_gpio(&config),
        "STM32F4" => Stm32Hal::new(McuFamily::STM32F4).generate_gpio(&config),
        "STM32H7" => Stm32Hal::new(McuFamily::STM32H7).generate_gpio(&config),
        "STM32L4" => Stm32Hal::new(McuFamily::STM32L4).generate_gpio(&config),
        "STM32G4" => Stm32Hal::new(McuFamily::STM32G4).generate_gpio(&config),
        "ESP32" | "ESP32S3" | "ESP32C3" => Esp32Hal::new(McuFamily::ESP32).generate_gpio(&config),
        "RP2040" => Rp2040Hal::new().generate_gpio(&config),
        "NRF52832" => NordicHal::new(McuFamily::NRF52832).generate_gpio(&config),
        "NRF52840" => NordicHal::new(McuFamily::NRF52840).generate_gpio(&config),
        "LPC1768" => NxpHal::new(McuFamily::LPC1768).generate_gpio(&config),
        "LPC5500" => NxpHal::new(McuFamily::LPC5500).generate_gpio(&config),
        _ => return Err(format!("Unknown MCU family: {}", family)),
    };
    
    Ok(serde_json::json!({
        "code": code,
        "family": family,
        "pin": pin,
    }))
}

/// Generate peripheral code for specific MCU
#[tauri::command]
fn generate_mcu_peripheral(
    family: String,
    peripheral: String,
    config: serde_json::Value,
) -> Result<serde_json::Value, String> {
    use drivers::mcu::{McuFamily, McuHal, SpiConfigAbstract, I2cConfigAbstract, 
                       UartConfigAbstract, I2cSpeedAbstract, UartParity};
    use drivers::mcu::stm32::Stm32Hal;
    use drivers::mcu::esp32::Esp32Hal;
    use drivers::mcu::rp2040::Rp2040Hal;
    use drivers::mcu::nordic::NordicHal;
    use drivers::mcu::nxp::NxpHal;
    
    let mcu_family = match family.as_str() {
        "STM32F1" => McuFamily::STM32F1,
        "STM32F4" => McuFamily::STM32F4,
        "STM32H7" => McuFamily::STM32H7,
        "STM32L4" => McuFamily::STM32L4,
        "STM32G4" => McuFamily::STM32G4,
        "ESP32" => McuFamily::ESP32,
        "ESP32S3" => McuFamily::ESP32S3,
        "ESP32C3" => McuFamily::ESP32C3,
        "RP2040" => McuFamily::RP2040,
        "NRF52832" => McuFamily::NRF52832,
        "NRF52840" => McuFamily::NRF52840,
        "LPC1768" => McuFamily::LPC1768,
        "LPC5500" => McuFamily::LPC5500,
        _ => return Err(format!("Unknown MCU family: {}", family)),
    };
    
    let hal: Box<dyn McuHal> = match mcu_family {
        McuFamily::STM32F1 | McuFamily::STM32F4 | McuFamily::STM32H7 |
        McuFamily::STM32L4 | McuFamily::STM32G4 => Box::new(Stm32Hal::new(mcu_family)),
        McuFamily::ESP32 | McuFamily::ESP32S3 | McuFamily::ESP32C3 => Box::new(Esp32Hal::new(mcu_family)),
        McuFamily::RP2040 => Box::new(Rp2040Hal::new()),
        McuFamily::NRF52832 | McuFamily::NRF52840 => Box::new(NordicHal::new(mcu_family)),
        McuFamily::LPC1768 | McuFamily::LPC5500 => Box::new(NxpHal::new(mcu_family)),
    };
    
    let code = match peripheral.to_lowercase().as_str() {
        "spi" => {
            let spi_config = SpiConfigAbstract {
                instance: config.get("instance").and_then(|v| v.as_u64()).unwrap_or(1) as u8,
                mode: config.get("mode").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
                clock_hz: config.get("clock_hz").and_then(|v| v.as_u64()).unwrap_or(1_000_000) as u32,
                data_bits: config.get("data_bits").and_then(|v| v.as_u64()).unwrap_or(8) as u8,
                msb_first: config.get("msb_first").and_then(|v| v.as_bool()).unwrap_or(true),
                dma: config.get("dma").and_then(|v| v.as_bool()).unwrap_or(false),
            };
            hal.generate_spi(&spi_config)
        },
        "i2c" => {
            let speed = match config.get("speed").and_then(|v| v.as_str()).unwrap_or("100k") {
                "400k" => I2cSpeedAbstract::Fast400k,
                "1m" => I2cSpeedAbstract::FastPlus1m,
                _ => I2cSpeedAbstract::Standard100k,
            };
            let i2c_config = I2cConfigAbstract {
                instance: config.get("instance").and_then(|v| v.as_u64()).unwrap_or(1) as u8,
                speed,
                address_bits: config.get("address_bits").and_then(|v| v.as_u64()).unwrap_or(7) as u8,
            };
            hal.generate_i2c(&i2c_config)
        },
        "uart" => {
            let parity = match config.get("parity").and_then(|v| v.as_str()).unwrap_or("none") {
                "even" => UartParity::Even,
                "odd" => UartParity::Odd,
                _ => UartParity::None,
            };
            let uart_config = UartConfigAbstract {
                instance: config.get("instance").and_then(|v| v.as_u64()).unwrap_or(1) as u8,
                baud_rate: config.get("baud_rate").and_then(|v| v.as_u64()).unwrap_or(115200) as u32,
                data_bits: config.get("data_bits").and_then(|v| v.as_u64()).unwrap_or(8) as u8,
                parity,
                stop_bits: config.get("stop_bits").and_then(|v| v.as_u64()).unwrap_or(1) as u8,
                flow_control: config.get("flow_control").and_then(|v| v.as_bool()).unwrap_or(false),
                dma: config.get("dma").and_then(|v| v.as_bool()).unwrap_or(false),
            };
            hal.generate_uart(&uart_config)
        },
        "clock" => {
            let freq = config.get("freq_mhz").and_then(|v| v.as_u64()).unwrap_or(168) as u32;
            hal.generate_clock_init(freq)
        },
        "system" => hal.generate_system_init(),
        _ => return Err(format!("Unknown peripheral: {}", peripheral)),
    };
    
    Ok(serde_json::json!({
        "code": code,
        "family": family,
        "peripheral": peripheral,
    }))
}

// ============================================================================
// RTOS Generation Commands
// ============================================================================

/// Generate RTOS task code
#[tauri::command]
fn generate_rtos_task(
    rtos: String,
    name: String,
    stack_size: u32,
    priority: String,
    entry_function: String,
    auto_start: bool,
) -> Result<serde_json::Value, String> {
    use drivers::rtos_gen::{RtosType, TaskConfig, TaskPriority, get_rtos_hal};
    
    let rtos_type = match rtos.to_lowercase().as_str() {
        "freertos" => RtosType::FreeRtos,
        "zephyr" => RtosType::Zephyr,
        _ => RtosType::FreeRtos,
    };
    
    let task_priority = match priority.to_lowercase().as_str() {
        "idle" => TaskPriority::Idle,
        "low" => TaskPriority::Low,
        "normal" => TaskPriority::Normal,
        "high" => TaskPriority::High,
        "realtime" => TaskPriority::Realtime,
        _ => TaskPriority::Normal,
    };
    
    let config = TaskConfig {
        name: name.clone(),
        stack_size,
        priority: task_priority,
        entry_function,
        parameter: None,
        auto_start,
    };
    
    let hal = get_rtos_hal(rtos_type);
    let code = hal.generate_task(&config);
    
    Ok(serde_json::json!({
        "code": code,
        "rtos": rtos,
        "name": name,
    }))
}

/// Generate RTOS semaphore code
#[tauri::command]
fn generate_rtos_semaphore(
    rtos: String,
    name: String,
    sem_type: String,
    max_count: u32,
    initial_count: u32,
) -> Result<serde_json::Value, String> {
    use drivers::rtos_gen::{RtosType, SemaphoreConfig, SemaphoreType, get_rtos_hal};
    
    let rtos_type = match rtos.to_lowercase().as_str() {
        "freertos" => RtosType::FreeRtos,
        "zephyr" => RtosType::Zephyr,
        _ => RtosType::FreeRtos,
    };
    
    let stype = if sem_type.to_lowercase() == "binary" || max_count <= 1 {
        SemaphoreType::Binary
    } else {
        SemaphoreType::Counting(max_count)
    };
    
    let config = SemaphoreConfig {
        name: name.clone(),
        sem_type: stype,
        initial_count,
    };
    
    let hal = get_rtos_hal(rtos_type);
    let code = hal.generate_semaphore(&config);
    
    Ok(serde_json::json!({
        "code": code,
        "rtos": rtos,
        "name": name,
    }))
}

/// Generate RTOS mutex code
#[tauri::command]
fn generate_rtos_mutex(
    rtos: String,
    name: String,
    recursive: bool,
) -> Result<serde_json::Value, String> {
    use drivers::rtos_gen::{RtosType, MutexConfig, get_rtos_hal};
    
    let rtos_type = match rtos.to_lowercase().as_str() {
        "freertos" => RtosType::FreeRtos,
        "zephyr" => RtosType::Zephyr,
        _ => RtosType::FreeRtos,
    };
    
    let config = MutexConfig {
        name: name.clone(),
        recursive,
    };
    
    let hal = get_rtos_hal(rtos_type);
    let code = hal.generate_mutex(&config);
    
    Ok(serde_json::json!({
        "code": code,
        "rtos": rtos,
        "name": name,
    }))
}

/// Generate RTOS queue code
#[tauri::command]
fn generate_rtos_queue(
    rtos: String,
    name: String,
    length: u32,
    item_size: u32,
) -> Result<serde_json::Value, String> {
    use drivers::rtos_gen::{RtosType, QueueConfig, get_rtos_hal};
    
    let rtos_type = match rtos.to_lowercase().as_str() {
        "freertos" => RtosType::FreeRtos,
        "zephyr" => RtosType::Zephyr,
        _ => RtosType::FreeRtos,
    };
    
    let config = QueueConfig {
        name: name.clone(),
        length,
        item_size,
    };
    
    let hal = get_rtos_hal(rtos_type);
    let code = hal.generate_queue(&config);
    
    Ok(serde_json::json!({
        "code": code,
        "rtos": rtos,
        "name": name,
    }))
}

/// Generate RTOS software timer code
#[tauri::command]
fn generate_rtos_timer(
    rtos: String,
    name: String,
    period_ms: u32,
    auto_reload: bool,
    callback: String,
) -> Result<serde_json::Value, String> {
    use drivers::rtos_gen::{RtosType, TimerConfig, get_rtos_hal};
    
    let rtos_type = match rtos.to_lowercase().as_str() {
        "freertos" => RtosType::FreeRtos,
        "zephyr" => RtosType::Zephyr,
        _ => RtosType::FreeRtos,
    };
    
    let config = TimerConfig {
        name: name.clone(),
        period_ms,
        auto_reload,
        callback,
    };
    
    let hal = get_rtos_hal(rtos_type);
    let code = hal.generate_timer(&config);
    
    Ok(serde_json::json!({
        "code": code,
        "rtos": rtos,
        "name": name,
    }))
}

/// Generate RTOS configuration file
#[tauri::command]
fn generate_rtos_config(rtos: String) -> Result<serde_json::Value, String> {
    use drivers::rtos_gen::{RtosType, get_rtos_hal};
    
    let rtos_type = match rtos.to_lowercase().as_str() {
        "freertos" => RtosType::FreeRtos,
        "zephyr" => RtosType::Zephyr,
        _ => RtosType::FreeRtos,
    };
    
    let hal = get_rtos_hal(rtos_type);
    let code = hal.generate_config_header();
    
    let filename = match rtos_type {
        RtosType::FreeRtos => "FreeRTOSConfig.h",
        RtosType::Zephyr => "prj.conf",
        RtosType::BareMetal => "config.h",
    };
    
    Ok(serde_json::json!({
        "code": code,
        "rtos": rtos,
        "filename": filename,
    }))
}

// ============================================================================
// Wireless Generation Commands
// ============================================================================

/// Generate BLE GATT service code
#[tauri::command]
fn generate_ble_service(
    platform: String,
    device_name: String,
    service_uuid: String,
    service_name: String,
    characteristics: Vec<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    use drivers::wireless::{BleConfig, BleService, BleCharacteristic, CharacteristicProperties, BleRole};
    use drivers::wireless::ble::{generate_nrf52_ble, generate_esp32_ble};
    
    let chars: Vec<BleCharacteristic> = characteristics.iter().map(|c| {
        BleCharacteristic {
            uuid: c.get("uuid").and_then(|v| v.as_str()).unwrap_or("0001").to_string(),
            name: c.get("name").and_then(|v| v.as_str()).unwrap_or("Char").to_string(),
            properties: CharacteristicProperties {
                read: c.get("read").and_then(|v| v.as_bool()).unwrap_or(true),
                write: c.get("write").and_then(|v| v.as_bool()).unwrap_or(false),
                write_no_response: c.get("write_no_resp").and_then(|v| v.as_bool()).unwrap_or(false),
                notify: c.get("notify").and_then(|v| v.as_bool()).unwrap_or(false),
                indicate: c.get("indicate").and_then(|v| v.as_bool()).unwrap_or(false),
            },
            max_length: c.get("max_length").and_then(|v| v.as_u64()).unwrap_or(20) as u16,
            description: None,
        }
    }).collect();
    
    let service = BleService {
        uuid: service_uuid.clone(),
        name: service_name.clone(),
        is_primary: true,
        characteristics: chars,
    };
    
    let config = BleConfig {
        device_name: device_name.clone(),
        role: BleRole::Peripheral,
        services: vec![service],
        advertising_interval_ms: 100,
        connection_interval_ms: 20,
        mtu: 23,
    };
    
    let code = match platform.to_lowercase().as_str() {
        "nrf52" | "nordic" => generate_nrf52_ble(&config),
        "esp32" => generate_esp32_ble(&config),
        _ => generate_nrf52_ble(&config),
    };
    
    Ok(serde_json::json!({
        "code": code,
        "platform": platform,
        "device_name": device_name,
        "service_name": service_name,
    }))
}

/// Generate WiFi configuration code
#[tauri::command]
fn generate_wifi_config(
    mode: String,
    ssid: String,
    password: String,
    security: String,
    channel: u8,
) -> Result<serde_json::Value, String> {
    use drivers::wireless::{WifiConfig, WifiMode, WifiSecurity};
    use drivers::wireless::wifi::generate_wifi_code;
    
    let wifi_mode = match mode.to_lowercase().as_str() {
        "ap" | "accesspoint" => WifiMode::AccessPoint,
        "sta_ap" | "both" => WifiMode::StationAndAP,
        _ => WifiMode::Station,
    };
    
    let wifi_security = match security.to_lowercase().as_str() {
        "open" => WifiSecurity::Open,
        "wpa3" => WifiSecurity::WPA3Personal,
        "enterprise" => WifiSecurity::WPA2Enterprise,
        _ => WifiSecurity::WPA2Personal,
    };
    
    let config = WifiConfig {
        mode: wifi_mode,
        ssid: ssid.clone(),
        password,
        security: wifi_security,
        channel,
        hostname: "neurobench".to_string(),
        static_ip: None,
    };
    
    let code = generate_wifi_code(&config);
    
    Ok(serde_json::json!({
        "code": code,
        "mode": mode,
        "ssid": ssid,
    }))
}

/// Generate LoRa configuration code
#[tauri::command]
fn generate_lora_config(
    frequency_mhz: u32,
    spreading_factor: u8,
    bandwidth: String,
    coding_rate: u8,
    tx_power: i8,
) -> Result<serde_json::Value, String> {
    use drivers::wireless::{LoraConfig, LoraSpreadingFactor, LoraBandwidth};
    use drivers::wireless::lora::generate_sx127x_lora;
    
    let sf = match spreading_factor {
        7 => LoraSpreadingFactor::SF7,
        8 => LoraSpreadingFactor::SF8,
        9 => LoraSpreadingFactor::SF9,
        10 => LoraSpreadingFactor::SF10,
        11 => LoraSpreadingFactor::SF11,
        12 => LoraSpreadingFactor::SF12,
        _ => LoraSpreadingFactor::SF7,
    };
    
    let bw = match bandwidth.as_str() {
        "250" | "BW250" => LoraBandwidth::BW250,
        "500" | "BW500" => LoraBandwidth::BW500,
        _ => LoraBandwidth::BW125,
    };
    
    let config = LoraConfig {
        frequency_mhz,
        spreading_factor: sf,
        bandwidth: bw,
        coding_rate,
        tx_power_dbm: tx_power,
        sync_word: 0x12,
        preamble_length: 8,
    };
    
    let code = generate_sx127x_lora(&config);
    
    Ok(serde_json::json!({
        "code": code,
        "frequency_mhz": frequency_mhz,
        "spreading_factor": spreading_factor,
    }))
}

// ============================================================================
// DSP Generation Commands
// ============================================================================

/// Generate FIR filter code
#[tauri::command]
fn generate_fir_filter(
    name: String,
    filter_type: String,
    order: u16,
    sample_rate: f32,
    cutoff_freq: f32,
    window: String,
) -> Result<serde_json::Value, String> {
    use drivers::dsp::{FirConfig, FilterType, WindowType};
    use drivers::dsp::filters::generate_fir_code;
    
    let ftype = match filter_type.to_lowercase().as_str() {
        "highpass" => FilterType::Highpass,
        "bandpass" => FilterType::Bandpass,
        _ => FilterType::Lowpass,
    };
    
    let win = match window.to_lowercase().as_str() {
        "hanning" => WindowType::Hanning,
        "blackman" => WindowType::Blackman,
        "rectangular" => WindowType::Rectangular,
        _ => WindowType::Hamming,
    };
    
    let config = FirConfig {
        name: name.clone(),
        filter_type: ftype,
        order,
        sample_rate,
        cutoff_freq,
        cutoff_freq_high: None,
        window: win,
        coefficients: None,
    };
    
    let code = generate_fir_code(&config);
    
    Ok(serde_json::json!({
        "code": code,
        "name": name,
        "order": order,
    }))
}

/// Generate IIR (biquad) filter code
#[tauri::command]
fn generate_iir_filter(
    name: String,
    filter_type: String,
    sample_rate: f32,
    cutoff_freq: f32,
    q_factor: f32,
) -> Result<serde_json::Value, String> {
    use drivers::dsp::{IirConfig, FilterType, IirTopology};
    use drivers::dsp::filters::generate_iir_code;
    
    let ftype = match filter_type.to_lowercase().as_str() {
        "highpass" => FilterType::Highpass,
        "bandpass" => FilterType::Bandpass,
        _ => FilterType::Lowpass,
    };
    
    let config = IirConfig {
        name: name.clone(),
        filter_type: ftype,
        order: 2,
        sample_rate,
        cutoff_freq,
        cutoff_freq_high: None,
        topology: IirTopology::DirectForm2,
        q_factor,
        gain_db: 0.0,
    };
    
    let code = generate_iir_code(&config);
    
    Ok(serde_json::json!({
        "code": code,
        "name": name,
        "cutoff_freq": cutoff_freq,
    }))
}

/// Generate FFT code
#[tauri::command]
fn generate_fft_block(
    name: String,
    size: u16,
    use_window: bool,
    window: String,
) -> Result<serde_json::Value, String> {
    use drivers::dsp::{FftConfig, WindowType};
    use drivers::dsp::fft::generate_fft_code;
    
    let win = if use_window {
        Some(match window.to_lowercase().as_str() {
            "hamming" => WindowType::Hamming,
            "blackman" => WindowType::Blackman,
            _ => WindowType::Hanning,
        })
    } else {
        None
    };
    
    let config = FftConfig {
        name: name.clone(),
        size,
        inverse: false,
        use_cmsis: true,
        window: win,
    };
    
    let code = generate_fft_code(&config);
    
    Ok(serde_json::json!({
        "code": code,
        "name": name,
        "size": size,
    }))
}

/// Generate PID controller code
#[tauri::command]
fn generate_pid_controller(
    name: String,
    kp: f32,
    ki: f32,
    kd: f32,
    output_min: f32,
    output_max: f32,
    sample_time_ms: u32,
    anti_windup: bool,
) -> Result<serde_json::Value, String> {
    use drivers::dsp::PidConfig;
    use drivers::dsp::pid::generate_pid_code;
    
    let config = PidConfig {
        name: name.clone(),
        kp,
        ki,
        kd,
        output_min,
        output_max,
        sample_time_ms,
        anti_windup,
        derivative_filter: true,
    };
    
    let code = generate_pid_code(&config);
    
    Ok(serde_json::json!({
        "code": code,
        "name": name,
        "kp": kp,
        "ki": ki,
        "kd": kd,
    }))
}

/// Generate circular buffer code
#[tauri::command]
fn generate_circular_buffer(
    name: String,
    size: u32,
    element_type: String,
    thread_safe: bool,
) -> Result<serde_json::Value, String> {
    use drivers::dsp::CircularBufferConfig;
    use drivers::dsp::buffer::generate_buffer_code;
    
    let config = CircularBufferConfig {
        name: name.clone(),
        size,
        element_type,
        thread_safe,
    };
    
    let code = generate_buffer_code(&config);
    
    Ok(serde_json::json!({
        "code": code,
        "name": name,
        "size": size,
    }))
}

// ============================================================================
// Security Generation Commands
// ============================================================================

/// Generate bootloader code
#[tauri::command]
fn generate_bootloader(
    name: String,
    bootloader_type: String,
    flash_base: u32,
    flash_size: u32,
    bootloader_size: u32,
    app_size: u32,
    enable_watchdog: bool,
    enable_crc: bool,
) -> Result<serde_json::Value, String> {
    use drivers::security::{BootloaderConfig, BootloaderType};
    use drivers::security::bootloader::generate_bootloader_code;
    
    let boot_type = match bootloader_type.to_lowercase().as_str() {
        "single" => BootloaderType::SingleBank,
        "dual_rollback" => BootloaderType::DualBankWithRollback,
        _ => BootloaderType::DualBank,
    };
    
    let config = BootloaderConfig {
        name: name.clone(),
        bootloader_type: boot_type,
        flash_base,
        flash_size,
        bootloader_size,
        app_size,
        vector_table_offset: bootloader_size,
        enable_watchdog,
        boot_timeout_ms: 3000,
        enable_crc_check: enable_crc,
        enable_signature_check: false,
    };
    
    let code = generate_bootloader_code(&config);
    
    Ok(serde_json::json!({
        "code": code,
        "name": name,
        "type": bootloader_type,
    }))
}

/// Generate OTA update client code
#[tauri::command]
fn generate_ota_client(
    name: String,
    transport: String,
    server_url: String,
    firmware_path: String,
    chunk_size: u32,
    verify_signature: bool,
) -> Result<serde_json::Value, String> {
    use drivers::security::{OtaConfig, OtaTransport};
    use drivers::security::ota::generate_ota_code;
    
    let ota_transport = match transport.to_lowercase().as_str() {
        "mqtt" => OtaTransport::MQTT,
        "ble" => OtaTransport::BLE,
        "uart" => OtaTransport::UART,
        "http" => OtaTransport::HTTP,
        _ => OtaTransport::HTTPS,
    };
    
    let config = OtaConfig {
        name: name.clone(),
        transport: ota_transport,
        server_url,
        firmware_path,
        version_check: true,
        chunk_size,
        retry_count: 3,
        timeout_ms: 30000,
        verify_signature,
        verify_checksum: true,
    };
    
    let code = generate_ota_code(&config);
    
    Ok(serde_json::json!({
        "code": code,
        "name": name,
        "transport": transport,
    }))
}

/// Generate secure boot verification code
#[tauri::command]
fn generate_secure_boot(
    name: String,
    algorithm: String,
    enable_rollback: bool,
    enable_debug_lock: bool,
) -> Result<serde_json::Value, String> {
    use drivers::security::{SecureBootConfig, SecureBootAlgorithm};
    use drivers::security::secure_boot::generate_secure_boot_code;
    
    let algo = match algorithm.to_lowercase().as_str() {
        "rsa2048" => SecureBootAlgorithm::RSA2048,
        "rsa4096" => SecureBootAlgorithm::RSA4096,
        "ecdsa384" => SecureBootAlgorithm::ECDSA384,
        "ed25519" => SecureBootAlgorithm::ED25519,
        _ => SecureBootAlgorithm::ECDSA256,
    };
    
    let config = SecureBootConfig {
        name: name.clone(),
        algorithm: algo,
        public_key_hash: None,
        enable_rollback_protection: enable_rollback,
        secure_counter_address: Some(0x1FFF7800),
        enable_debug_lock,
        enable_jtag_disable: false,
    };
    
    let code = generate_secure_boot_code(&config);
    
    Ok(serde_json::json!({
        "code": code,
        "name": name,
        "algorithm": algorithm,
    }))
}

/// Generate crypto utilities code
#[tauri::command]
fn generate_crypto_utils(
    name: String,
    include_aes: bool,
    include_hash: bool,
    include_rng: bool,
    include_ecdsa: bool,
    hash_algorithm: String,
) -> Result<serde_json::Value, String> {
    use drivers::security::{CryptoConfig, HashAlgorithm};
    use drivers::security::crypto::generate_crypto_code;
    
    let hash_algo = match hash_algorithm.to_lowercase().as_str() {
        "sha384" => HashAlgorithm::SHA384,
        "sha512" => HashAlgorithm::SHA512,
        "sha3" => HashAlgorithm::SHA3_256,
        _ => HashAlgorithm::SHA256,
    };
    
    let config = CryptoConfig {
        name: name.clone(),
        include_aes,
        include_hash,
        include_rsa: false,
        include_ecdsa,
        include_rng,
        use_hardware_crypto: true,
        hash_algorithm: hash_algo,
    };
    
    let code = generate_crypto_code(&config);
    
    Ok(serde_json::json!({
        "code": code,
        "name": name,
    }))
}

// ============================================================================
// Export Commands
// ============================================================================

/// Export code to a file
#[tauri::command]
async fn export_code_to_file(
    code: String,
    file_path: String,
) -> Result<serde_json::Value, String> {
    use std::fs;
    use std::path::Path;
    
    let path = Path::new(&file_path);
    
    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    
    fs::write(path, &code).map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "success": true,
        "path": file_path,
        "bytes": code.len(),
    }))
}

/// Generate CMakeLists.txt for project
#[tauri::command]
fn generate_project_cmake(
    project_name: String,
    sources: Vec<String>,
    mcu: String,
) -> Result<serde_json::Value, String> {
    use drivers::export::generate_cmake;
    
    let sources_refs: Vec<&str> = sources.iter().map(|s| s.as_str()).collect();
    let cmake = generate_cmake(&project_name, &sources_refs, &mcu);
    
    Ok(serde_json::json!({
        "cmake": cmake,
        "project": project_name,
    }))
}

/// Generate Makefile for project
#[tauri::command]
fn generate_project_makefile(
    project_name: String,
    sources: Vec<String>,
    mcu: String,
) -> Result<serde_json::Value, String> {
    use drivers::export::generate_makefile;
    
    let sources_refs: Vec<&str> = sources.iter().map(|s| s.as_str()).collect();
    let makefile = generate_makefile(&project_name, &sources_refs, &mcu);
    
    Ok(serde_json::json!({
        "makefile": makefile,
        "project": project_name,
    }))
}

/// Validate generated code using external compilers
#[tauri::command]
fn validate_code(
    code: String,
    language: String,
    embedded: bool,
) -> Result<serde_json::Value, String> {
    use validation::{validate_c_code, validate_rust_code, validate_embedded_c, ValidationResult};
    
    let result: ValidationResult = match language.to_lowercase().as_str() {
        "c" => {
            if embedded {
                validate_embedded_c(&code, false)?
            } else {
                validate_c_code(&code, false)?
            }
        },
        "cpp" | "c++" => {
            if embedded {
                validate_embedded_c(&code, true)?
            } else {
                validate_c_code(&code, true)?
            }
        },
        "rust" | "rs" => validate_rust_code(&code)?,
        _ => return Err(format!("Unsupported language: {}", language)),
    };
    
    Ok(serde_json::json!({
        "success": result.success,
        "errors": result.errors,
        "warnings": result.warnings,
        "compiler": result.compiler,
        "exitCode": result.exit_code,
    }))
}

// === Git Integration Commands ===

/// Initialize a Git repository
#[tauri::command]
fn git_init(path: String) -> Result<serde_json::Value, String> {
    let result = git::init_repo(&path)?;
    Ok(serde_json::json!({
        "success": true,
        "path": result,
    }))
}

/// Get Git repository status
#[tauri::command]
fn git_status(path: String) -> Result<serde_json::Value, String> {
    let status = git::get_status(&path)?;
    Ok(serde_json::to_value(status).map_err(|e| e.to_string())?)
}

/// Stage files for commit
#[tauri::command]
fn git_stage_files(path: String, files: Vec<String>) -> Result<serde_json::Value, String> {
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    let count = git::stage_files(&path, &file_refs)?;
    Ok(serde_json::json!({
        "success": true,
        "stagedCount": count,
    }))
}

/// Stage all changes
#[tauri::command]
fn git_stage_all(path: String) -> Result<serde_json::Value, String> {
    let count = git::stage_all(&path)?;
    Ok(serde_json::json!({
        "success": true,
        "stagedCount": count,
    }))
}

/// Create a commit
#[tauri::command]
fn git_commit(
    path: String, 
    message: String, 
    author_name: String, 
    author_email: String
) -> Result<serde_json::Value, String> {
    let commit = git::commit(&path, &message, &author_name, &author_email)?;
    Ok(serde_json::to_value(commit).map_err(|e| e.to_string())?)
}

/// Get commit history
#[tauri::command]
fn git_history(path: String, limit: usize) -> Result<serde_json::Value, String> {
    let history = git::get_history(&path, limit)?;
    Ok(serde_json::to_value(history).map_err(|e| e.to_string())?)
}

/// Get diff between working tree and HEAD
#[tauri::command]
fn git_diff(path: String) -> Result<serde_json::Value, String> {
    let diff = git::get_diff(&path)?;
    Ok(serde_json::to_value(diff).map_err(|e| e.to_string())?)
}

// === QEMU Simulation Commands ===

/// Check if QEMU is available
#[tauri::command]
fn qemu_check() -> Result<serde_json::Value, String> {
    let available = qemu::is_qemu_available();
    Ok(serde_json::json!({
        "available": available,
    }))
}

/// Get QEMU version
#[tauri::command]
fn qemu_version() -> Result<serde_json::Value, String> {
    let version = qemu::get_qemu_version()?;
    Ok(serde_json::json!({
        "version": version,
    }))
}

/// List available QEMU machines
#[tauri::command]
fn qemu_list_machines() -> Result<serde_json::Value, String> {
    let machines = qemu::list_machines()?;
    Ok(serde_json::json!({
        "machines": machines,
    }))
}

/// Get preset machine configurations
#[tauri::command]
fn qemu_get_presets() -> Result<serde_json::Value, String> {
    let presets = qemu::get_machine_presets();
    Ok(serde_json::to_value(presets).map_err(|e| e.to_string())?)
}

// === Cloud Sync Commands ===

/// Export a project to JSON
#[tauri::command]
fn cloud_export_project(
    name: String,
    description: String,
    mcu_target: String,
    files: Vec<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let exported_files: Vec<cloud::ExportedFile> = files
        .into_iter()
        .filter_map(|f| serde_json::from_value(f).ok())
        .collect();
    
    let export = cloud::export_project(
        &name,
        &description,
        &mcu_target,
        exported_files,
        cloud::ProjectConfig::default(),
    )?;
    
    Ok(serde_json::json!({
        "success": true,
        "json": export,
    }))
}

/// Import a project from JSON
#[tauri::command]
fn cloud_import_project(json: String) -> Result<serde_json::Value, String> {
    let project = cloud::import_project(&json)?;
    Ok(serde_json::to_value(project).map_err(|e| e.to_string())?)
}

/// Generate a unique share ID
#[tauri::command]
fn cloud_generate_share_id() -> Result<serde_json::Value, String> {
    let id = cloud::generate_share_id();
    Ok(serde_json::json!({
        "shareId": id,
    }))
}

/// Collect project files from a directory
#[tauri::command]
fn cloud_collect_files(dir: String, extensions: Vec<String>) -> Result<serde_json::Value, String> {
    let ext_refs: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
    let files = cloud::collect_project_files(&dir, &ext_refs)?;
    Ok(serde_json::to_value(files).map_err(|e| e.to_string())?)
}

// === Templates Commands ===

/// Get all templates
#[tauri::command]
fn templates_get_all() -> Result<serde_json::Value, String> {
    let templates = templates::get_templates();
    Ok(serde_json::to_value(templates).map_err(|e| e.to_string())?)
}

/// Get template by ID
#[tauri::command]
fn templates_get_by_id(id: String) -> Result<serde_json::Value, String> {
    let template = templates::get_template_by_id(&id)
        .ok_or_else(|| format!("Template '{}' not found", id))?;
    Ok(serde_json::to_value(template).map_err(|e| e.to_string())?)
}

/// Get template categories
#[tauri::command]
fn templates_get_categories() -> Result<serde_json::Value, String> {
    let categories = templates::get_categories();
    Ok(serde_json::json!({ "categories": categories }))
}

// === Snippets Commands ===

/// Get all snippets
#[tauri::command]
fn snippets_get_all() -> Result<serde_json::Value, String> {
    let snippets = snippets::get_snippets();
    Ok(serde_json::to_value(snippets).map_err(|e| e.to_string())?)
}

/// Search snippets
#[tauri::command]
fn snippets_search(query: String) -> Result<serde_json::Value, String> {
    let results = snippets::search_snippets(&query);
    Ok(serde_json::to_value(results).map_err(|e| e.to_string())?)
}

/// Get snippet by ID
#[tauri::command]
fn snippets_get_by_id(id: String) -> Result<serde_json::Value, String> {
    let snippet = snippets::get_snippet_by_id(&id)
        .ok_or_else(|| format!("Snippet '{}' not found", id))?;
    Ok(serde_json::to_value(snippet).map_err(|e| e.to_string())?)
}

// === Memory Analyzer Commands ===

/// Estimate memory usage
#[tauri::command]
fn memory_estimate(code: String, mcu: String) -> Result<serde_json::Value, String> {
    let analysis = memory::estimate_memory(&code, &mcu)?;
    Ok(serde_json::to_value(analysis).map_err(|e| e.to_string())?)
}

/// Get MCU memory configs
#[tauri::command]
fn memory_get_mcu_configs() -> Result<serde_json::Value, String> {
    let configs = memory::get_mcu_configs();
    Ok(serde_json::to_value(configs).map_err(|e| e.to_string())?)
}

// === Power Estimator Commands ===

/// Estimate power consumption
#[tauri::command]
fn power_estimate(
    mcu: String,
    peripherals: Vec<String>,
    duty_cycle: f32,
    battery_mah: Option<f32>,
) -> Result<serde_json::Value, String> {
    let estimation = power::estimate_power(&mcu, &peripherals, duty_cycle, battery_mah)?;
    Ok(serde_json::to_value(estimation).map_err(|e| e.to_string())?)
}

/// Get MCU power specs
#[tauri::command]
fn power_get_mcu_specs() -> Result<serde_json::Value, String> {
    let specs = power::get_mcu_power_specs();
    Ok(serde_json::to_value(specs).map_err(|e| e.to_string())?)
}

// === Pin Configuration Commands ===

/// Get MCU packages
#[tauri::command]
fn pins_get_packages() -> Result<serde_json::Value, String> {
    let packages = pins::get_mcu_packages();
    Ok(serde_json::to_value(packages).map_err(|e| e.to_string())?)
}

/// Generate pin init code
#[tauri::command]
fn pins_generate_code(configs: Vec<serde_json::Value>) -> Result<serde_json::Value, String> {
    let pin_configs: Vec<pins::PinConfig> = configs
        .into_iter()
        .filter_map(|c| serde_json::from_value(c).ok())
        .collect();
    
    let code = pins::generate_pin_init_code(&pin_configs);
    Ok(serde_json::json!({ "code": code }))
}

// === Build System Commands ===

/// Generate Makefile
#[tauri::command]
fn build_generate_makefile(config: serde_json::Value) -> Result<serde_json::Value, String> {
    let build_config: build::BuildConfig = serde_json::from_value(config)
        .map_err(|e| e.to_string())?;
    let makefile = build::generate_makefile(&build_config);
    Ok(serde_json::json!({ "makefile": makefile }))
}

/// Generate CMakeLists.txt
#[tauri::command]
fn build_generate_cmake(config: serde_json::Value) -> Result<serde_json::Value, String> {
    let build_config: build::BuildConfig = serde_json::from_value(config)
        .map_err(|e| e.to_string())?;
    let cmake = build::generate_cmake(&build_config);
    Ok(serde_json::json!({ "cmake": cmake }))
}

/// Check toolchain availability
#[tauri::command]
fn build_check_toolchain() -> Result<serde_json::Value, String> {
    let tools = build::check_toolchain();
    Ok(serde_json::to_value(tools).map_err(|e| e.to_string())?)
}

// === Serial Monitor Commands ===

/// List available serial ports
#[tauri::command]
fn serial_list_ports() -> Result<serde_json::Value, String> {
    let ports = serial::list_ports()?;
    Ok(serde_json::to_value(ports).map_err(|e| e.to_string())?)
}

/// Get common baud rates
#[tauri::command]
fn serial_get_baud_rates() -> Result<serde_json::Value, String> {
    let rates = serial::get_baud_rates();
    Ok(serde_json::json!({ "baudRates": rates }))
}

/// Format data for display
#[tauri::command]
fn serial_format_data(data: Vec<u8>, format: String) -> Result<serde_json::Value, String> {
    let formatted = serial::format_data(&data, &format);
    Ok(serde_json::json!({ "formatted": formatted }))
}

/// Parse escape sequences
#[tauri::command]
fn serial_parse_escape(input: String) -> Result<serde_json::Value, String> {
    let bytes = serial::parse_escape_sequences(&input);
    Ok(serde_json::json!({ "bytes": bytes }))
}

/// Calculate checksum
#[tauri::command]
fn serial_calculate_checksum(data: Vec<u8>, algorithm: String) -> Result<serde_json::Value, String> {
    let checksum = serial::calculate_checksum(&data, &algorithm);
    Ok(serde_json::json!({ "checksum": checksum }))
}

// === Documentation Generator Commands ===

/// Generate documentation for code
#[tauri::command]
fn docs_generate(code: String, filename: String, author: String, brief: String) -> Result<serde_json::Value, String> {
    let documentation = docs::generate_documentation(&code, &filename, &author, &brief);
    Ok(serde_json::json!({ "documentation": documentation }))
}

/// Generate Doxyfile configuration
#[tauri::command]
fn docs_generate_doxyfile(project_name: String, output_dir: String) -> Result<serde_json::Value, String> {
    let doxyfile = docs::generate_doxyfile(&project_name, &output_dir);
    Ok(serde_json::json!({ "doxyfile": doxyfile }))
}

/// Extract functions from code
#[tauri::command]
fn docs_extract_functions(code: String) -> Result<serde_json::Value, String> {
    let functions = docs::extract_functions(&code);
    Ok(serde_json::to_value(functions).map_err(|e| e.to_string())?)
}

// === Profiler Commands ===

/// Analyze code performance
#[tauri::command]
fn profiler_analyze(code: String, mcu_freq_mhz: u32) -> Result<serde_json::Value, String> {
    let result = profiler::analyze_performance(&code, mcu_freq_mhz);
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

/// Estimate function timing
#[tauri::command]
fn profiler_estimate_timing(code: String, mcu_freq_mhz: u32) -> Result<serde_json::Value, String> {
    let timing = profiler::estimate_timing(&code, mcu_freq_mhz);
    Ok(serde_json::to_value(timing).map_err(|e| e.to_string())?)
}

// === Register Commands ===

/// Get all peripherals
#[tauri::command]
fn registers_get_peripherals() -> Result<serde_json::Value, String> {
    let peripherals = registers::get_peripherals();
    Ok(serde_json::to_value(peripherals).map_err(|e| e.to_string())?)
}

/// Get GPIO registers for a port
#[tauri::command]
fn registers_get_gpio(port: String) -> Result<serde_json::Value, String> {
    let port_char = port.chars().next().unwrap_or('A');
    let gpio = registers::get_gpio_registers(port_char);
    Ok(serde_json::to_value(gpio).map_err(|e| e.to_string())?)
}

/// Generate register access code
#[tauri::command]
fn registers_generate_code(peripheral: String, reg: String, operation: String, value: Option<u32>) -> Result<serde_json::Value, String> {
    let code = registers::generate_register_code(&peripheral, &reg, &operation, value);
    Ok(serde_json::json!({ "code": code }))
}

// ==================== Advanced Terminal Commands ====================

/// Execute an advanced terminal command with parsing and autocomplete
#[tauri::command]
fn terminal_execute_advanced(command: String, variables: Option<std::collections::HashMap<String, String>>) -> Result<serde_json::Value, String> {
    let vars = variables.unwrap_or_default();
    let parsed_commands = terminal::parser::parse_command_line(&command, &vars);
    
    let mut all_output = Vec::new();
    let mut overall_success = true;
    
    for parsed_cmd in &parsed_commands {
        let result = terminal::commands::process_embedded_command(parsed_cmd);
        overall_success = overall_success && result.success;
        all_output.extend(result.output);
    }
    
    Ok(serde_json::json!({
        "success": overall_success,
        "output": all_output,
        "command_count": parsed_commands.len()
    }))
}

/// Get tab completions for current input
#[tauri::command]
fn terminal_get_completions(input: String, cursor_pos: usize) -> Result<serde_json::Value, String> {
    let completions = terminal::autocomplete::get_completions(&input, cursor_pos);
    Ok(serde_json::to_value(completions).map_err(|e| e.to_string())?)
}

/// Get available terminal themes
#[tauri::command]
fn terminal_get_themes() -> Result<serde_json::Value, String> {
    let theme_names = terminal::themes::get_available_themes();
    let themes: Vec<_> = theme_names.iter()
        .map(|name| terminal::themes::get_theme(name))
        .collect();
    Ok(serde_json::to_value(themes).map_err(|e| e.to_string())?)
}

/// Get terminal welcome message
#[tauri::command]
fn terminal_get_welcome() -> Result<serde_json::Value, String> {
    let welcome = terminal::get_welcome_message();
    Ok(serde_json::to_value(welcome).map_err(|e| e.to_string())?)
}

/// Parse a command without executing (for syntax highlighting)
#[tauri::command]
fn terminal_parse_command(command: String) -> Result<serde_json::Value, String> {
    let vars = std::collections::HashMap::new();
    let parsed = terminal::parser::parse_command_line(&command, &vars);
    Ok(serde_json::to_value(parsed).map_err(|e| e.to_string())?)
}

// ==================== Performance Monitor Commands ====================

/// Get current system performance metrics
#[tauri::command]
fn performance_get_system_metrics() -> Result<serde_json::Value, String> {
    let metrics = performance::get_system_metrics();
    Ok(serde_json::to_value(metrics).map_err(|e| e.to_string())?)
}

/// Get list of running processes
#[tauri::command]
fn performance_get_process_list(limit: Option<usize>) -> Result<serde_json::Value, String> {
    let processes = performance::get_process_list(limit.unwrap_or(20));
    Ok(serde_json::to_value(processes).map_err(|e| e.to_string())?)
}

/// Get embedded device metrics
#[tauri::command]
fn performance_get_embedded_metrics(port: Option<String>) -> Result<serde_json::Value, String> {
    let metrics = performance::get_embedded_metrics(port.as_deref());
    Ok(serde_json::to_value(metrics).map_err(|e| e.to_string())?)
}

// ==================== Toolchain & IDE Loop Commands ====================

use toolchain::{
    BuildConfig, BuildResult, SizeReport, MapFileInfo,
    probe::{ProbeConfig, ProbeInfo, FlashResult, CpuState, RegisterSet, RttChannel, RttMessage, ResetMode},
};
use std::sync::OnceLock;

// Global probe manager
static PROBE_MANAGER: OnceLock<tokio::sync::Mutex<toolchain::probe::ProbeManager>> = OnceLock::new();

fn get_probe_manager() -> &'static tokio::sync::Mutex<toolchain::probe::ProbeManager> {
    PROBE_MANAGER.get_or_init(|| {
        tokio::sync::Mutex::new(toolchain::probe::ProbeManager::new())
    })
}

/// Discover available toolchains
#[tauri::command]
fn toolchain_discover() -> Result<serde_json::Value, String> {
    let toolchains = toolchain::discovery::discover_all();
    Ok(serde_json::to_value(toolchains).map_err(|e| e.to_string())?)
}

/// Build project using discovered toolchain
#[tauri::command]
async fn toolchain_build(config: BuildConfig) -> Result<BuildResult, String> {
    use toolchain::Toolchain;
    
    // Discover toolchain
    let toolchains = toolchain::discovery::discover_all();
    let tc = toolchains.iter()
        .find(|t| config.toolchain_id.as_ref().map_or(true, |id| &t.id == id))
        .ok_or_else(|| "No suitable toolchain found. Install ARM GCC or run 'rustup target add thumbv7em-none-eabihf'".to_string())?;
    
    // Use ARM GCC if available
    let gcc = toolchain::arm_gcc::ArmGcc::new(tc.clone());
    gcc.build(&config).map_err(|e| e.to_string())
}

/// Clean project build artifacts
#[tauri::command]
fn toolchain_clean(project_path: String) -> Result<(), String> {
    use toolchain::Toolchain;
    
    let toolchains = toolchain::discovery::discover_all();
    if let Some(tc) = toolchains.first() {
        let gcc = toolchain::arm_gcc::ArmGcc::new(tc.clone());
        gcc.clean(std::path::Path::new(&project_path)).map_err(|e| e.to_string())
    } else {
        // Just remove build directory manually
        let build_dir = std::path::Path::new(&project_path).join("build");
        if build_dir.exists() {
            std::fs::remove_dir_all(&build_dir).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

/// Get size report for compiled binary
#[tauri::command]
fn toolchain_size_report(elf_path: String) -> Result<SizeReport, String> {
    use toolchain::Toolchain;
    
    let toolchains = toolchain::discovery::discover_all();
    let tc = toolchains.first()
        .ok_or("No toolchain found")?;
    
    let gcc = toolchain::arm_gcc::ArmGcc::new(tc.clone());
    gcc.size(std::path::Path::new(&elf_path)).map_err(|e| e.to_string())
}

/// Parse linker map file
#[tauri::command]
fn toolchain_parse_map(map_path: String) -> Result<MapFileInfo, String> {
    use toolchain::Toolchain;
    
    let toolchains = toolchain::discovery::discover_all();
    let tc = toolchains.first()
        .ok_or("No toolchain found")?;
    
    let gcc = toolchain::arm_gcc::ArmGcc::new(tc.clone());
    gcc.parse_map(std::path::Path::new(&map_path)).map_err(|e| e.to_string())
}

// ==================== Probe Commands ====================

/// List connected debug probes
#[tauri::command]
fn probe_list() -> Result<Vec<ProbeInfo>, String> {
    Ok(toolchain::probe::ProbeManager::list_probes())
}

/// Connect to a debug probe
#[tauri::command]
async fn probe_connect(config: ProbeConfig) -> Result<ProbeInfo, String> {
    let pm = get_probe_manager();
    let mut manager = pm.lock().await;
    manager.connect(config).await.map_err(|e| e.to_string())
}

/// Disconnect from probe
#[tauri::command]
async fn probe_disconnect() -> Result<(), String> {
    let pm = get_probe_manager();
    let mut manager = pm.lock().await;
    manager.disconnect();
    Ok(())
}

/// Flash firmware to target
#[tauri::command]
async fn probe_flash(elf_path: String, verify: bool) -> Result<FlashResult, String> {
    let pm = get_probe_manager();
    let manager = pm.lock().await;
    manager.flash(std::path::Path::new(&elf_path), verify).await.map_err(|e| e.to_string())
}

/// Reset target
#[tauri::command]
async fn probe_reset(mode: String) -> Result<(), String> {
    let reset_mode = match mode.to_lowercase().as_str() {
        "hardware" => ResetMode::Hardware,
        "software" => ResetMode::Software,
        "halt" => ResetMode::HaltAfterReset,
        _ => ResetMode::Software,
    };
    
    let pm = get_probe_manager();
    let manager = pm.lock().await;
    manager.reset(reset_mode).await.map_err(|e| e.to_string())
}

/// Halt CPU execution
#[tauri::command]
async fn probe_halt() -> Result<CpuState, String> {
    let pm = get_probe_manager();
    let manager = pm.lock().await;
    manager.halt().await.map_err(|e| e.to_string())
}

/// Resume CPU execution
#[tauri::command]
async fn probe_resume() -> Result<(), String> {
    let pm = get_probe_manager();
    let manager = pm.lock().await;
    manager.resume().await.map_err(|e| e.to_string())
}

/// Read memory from target
#[tauri::command]
async fn probe_read_memory(address: u32, length: usize) -> Result<Vec<u8>, String> {
    let pm = get_probe_manager();
    let manager = pm.lock().await;
    manager.read_memory(address, length).await.map_err(|e| e.to_string())
}

/// Read CPU registers
#[tauri::command]
async fn probe_read_registers() -> Result<RegisterSet, String> {
    let pm = get_probe_manager();
    let manager = pm.lock().await;
    manager.read_registers().await.map_err(|e| e.to_string())
}

// ==================== RTT Commands ====================

/// Start RTT streaming
#[tauri::command]
async fn rtt_start(channel: u32) -> Result<RttChannel, String> {
    let pm = get_probe_manager();
    let mut manager = pm.lock().await;
    manager.start_rtt(channel).await.map_err(|e| e.to_string())
}

/// Read RTT data
#[tauri::command]
async fn rtt_read() -> Result<Vec<RttMessage>, String> {
    let pm = get_probe_manager();
    let manager = pm.lock().await;
    manager.read_rtt().await.map_err(|e| e.to_string())
}

/// Stop RTT streaming
#[tauri::command]
async fn rtt_stop() -> Result<(), String> {
    let pm = get_probe_manager();
    let mut manager = pm.lock().await;
    manager.stop_rtt();
    Ok(())
}

/// Decode HardFault from stack dump
#[tauri::command]
fn decode_hardfault(stack_hex: String, elf_path: Option<String>) -> Result<serde_json::Value, String> {
    // Parse hex string to bytes
    let stack: Vec<u8> = stack_hex
        .replace(' ', "")
        .as_bytes()
        .chunks(2)
        .filter_map(|chunk| {
            let s = std::str::from_utf8(chunk).ok()?;
            u8::from_str_radix(s, 16).ok()
        })
        .collect();
    
    let elf = elf_path.map(|p| std::path::PathBuf::from(p));
    let bt = toolchain::probe::decode_hardfault(&stack, elf.as_deref())
        .map_err(|e| e.to_string())?;
    
    Ok(serde_json::to_value(bt).map_err(|e| e.to_string())?)
}

// ==================== AI Model Management ====================

/// Get available AI providers and current settings
#[tauri::command]
fn ai_get_providers() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "providers": [
            {
                "id": "gemini",
                "name": "Google Gemini",
                "models": ["gemini-1.5-flash", "gemini-1.5-pro", "gemini-2.0-flash-exp"],
                "configured": std::env::var("GEMINI_API_KEY").is_ok(),
            },
            {
                "id": "openai",
                "name": "OpenAI",
                "models": ["gpt-4o-mini", "gpt-4o", "gpt-4-turbo"],
                "configured": std::env::var("OPENAI_API_KEY").is_ok(),
            },
            {
                "id": "ollama",
                "name": "Ollama (Local)",
                "models": ["llama3.2", "codellama", "mistral", "phi3"],
                "configured": true, // Ollama doesn't need API key
            },
        ],
        "current": "gemini" // Default
    }))
}

/// Set the AI provider to use
#[tauri::command]
fn ai_set_provider(provider: String, _model: Option<String>, _api_key: Option<String>) -> Result<(), String> {
    // In a full implementation, this would update global state
    // For now, just validate the provider
    match provider.as_str() {
        "gemini" | "openai" | "ollama" => Ok(()),
        _ => Err(format!("Unknown provider: {}", provider)),
    }
}

// ==================== Streaming Build Commands ====================

use toolchain::streaming_build::{StreamingBuildConfig, BuildEvent, BuildId};

/// Start a streaming build - returns build_id immediately, emits events via Tauri
#[tauri::command]
async fn streaming_build_start(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    config: serde_json::Value,
) -> Result<String, String> {
    // Parse config
    let build_config: StreamingBuildConfig = serde_json::from_value(config)
        .map_err(|e| format!("Invalid build config: {}", e))?;
    
    // Start build
    let build_id = state.build_manager.start_build(build_config).await;
    
    // Spawn event forwarder to Tauri
    let mut rx = state.build_manager.subscribe();
    let bid = build_id.clone();
    
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            // Extract build_id from header
            let event_build_id = match &event {
                BuildEvent::Started { header, .. } => &header.build_id,
                BuildEvent::Output { header, .. } => &header.build_id,
                BuildEvent::Diagnostic { header, .. } => &header.build_id,
                BuildEvent::Progress { header, .. } => &header.build_id,
                BuildEvent::Completed { header, .. } => &header.build_id,
                BuildEvent::Cancelled { header, .. } => &header.build_id,
                BuildEvent::InternalError { header, .. } => &header.build_id,
            };
            
            if event_build_id == &bid {
                // Emit to frontend
                let event_name = match &event {
                    BuildEvent::Started { .. } => "build:started",
                    BuildEvent::Output { .. } => "build:output",
                    BuildEvent::Diagnostic { .. } => "build:diagnostic",
                    BuildEvent::Progress { .. } => "build:progress",
                    BuildEvent::Completed { .. } => "build:completed",
                    BuildEvent::Cancelled { .. } => "build:cancelled",
                    BuildEvent::InternalError { .. } => "build:internal_error",
                };
                
                let _ = app.emit(event_name, &event);
                
                // Stop forwarding after terminal events
                if matches!(event, BuildEvent::Completed { .. } | BuildEvent::Cancelled { .. } | BuildEvent::InternalError { .. }) {
                    break;
                }
            }
        }
    });
    
    Ok(build_id)
}

/// Cancel a running build
#[tauri::command]
async fn streaming_build_cancel(
    state: State<'_, AppState>,
    build_id: String,
) -> Result<bool, String> {
    Ok(state.build_manager.cancel_build(&build_id).await)
}

/// List active builds
#[tauri::command]
async fn streaming_build_list(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    Ok(state.build_manager.active_builds().await)
}

/// Get full log for a build
#[tauri::command]
async fn streaming_build_get_log(
    state: State<'_, AppState>,
    build_id: String,
    last_n: Option<usize>,
) -> Result<Vec<String>, String> {
    state.build_manager.get_log(&build_id, last_n).await
        .ok_or_else(|| format!("Build {} not found", build_id))
}

/// Get diagnostics for a build
#[tauri::command]
async fn streaming_build_get_diagnostics(
    state: State<'_, AppState>,
    build_id: String,
) -> Result<serde_json::Value, String> {
    let diags = state.build_manager.get_diagnostics(&build_id).await
        .ok_or_else(|| format!("Build {} not found", build_id))?;
    
    Ok(serde_json::json!({ "diagnostics": diags }))
}

/// Get latest successful build artifacts
#[tauri::command]
async fn streaming_build_get_latest_artifacts(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let artifacts = state.build_manager.get_latest_artifacts().await;
    Ok(serde_json::json!({ "artifacts": artifacts }))
}

/// Get artifacts for a specific build
#[tauri::command]
async fn streaming_build_get_artifacts(
    state: State<'_, AppState>,
    build_id: String,
) -> Result<serde_json::Value, String> {
    let artifacts = state.build_manager.get_artifacts(&build_id).await;
    Ok(serde_json::json!({ "artifacts": artifacts }))
}

// ==================== Flash Commands ====================

use jobs::{JobKind, JobInfo, JobStatus};
use jobs::flash::{FlashConfig, MockProbeBackend, ProbeBackend};
use jobs::flash::run_flash_job;

/// Start flash operation
#[tauri::command]
async fn flash_start(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    elf_path: Option<String>,
    use_latest: Option<bool>,
    verify: Option<bool>,
    chip: Option<String>,
) -> Result<String, String> {
    // Determine ELF path
    let elf = if use_latest.unwrap_or(false) {
        // Get latest successful build artifact
        if let Some(artifacts) = state.build_manager.get_latest_artifacts().await {
            std::path::PathBuf::from(&artifacts.elf_path)
        } else {
            return Err("No successful build artifacts found".to_string());
        }
    } else if let Some(path) = elf_path {
        std::path::PathBuf::from(path)
    } else {
        return Err("Either elf_path or use_latest=true required".to_string());
    };
    
    let config = FlashConfig {
        elf_path: elf,
        verify: verify.unwrap_or(true),
        chip,
        speed_khz: Some(4000),
    };
    
    // Use mock backend for now
    let backend = Arc::new(MockProbeBackend::new());
    
    // Create event emitter closure that uses Tauri
    let app_clone = app.clone();
    let emit_event = move |event_name: String, payload: serde_json::Value| {
        let _ = app_clone.emit(&event_name, &payload);
    };
    
    // Run flash job with full event pipeline
    run_flash_job(
        state.job_manager.clone(),
        backend,
        config,
        emit_event,
    ).await
}

/// Cancel flash operation
#[tauri::command]
async fn flash_cancel(
    state: State<'_, AppState>,
    flash_id: String,
) -> Result<bool, String> {
    Ok(state.job_manager.cancel_job(&flash_id))
}

// ==================== RTT Commands ====================

use jobs::rtt::{RttConfig, MockRttBackend as MockRtt, run_rtt_job};

/// Start RTT streaming job (uses job manager, emits batched events)
#[tauri::command]
async fn rtt_stream_start(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    chip: String,
    channels: Option<Vec<u32>>,
    poll_interval_ms: Option<u64>,
) -> Result<String, String> {
    let config = RttConfig {
        chip,
        channels: channels.unwrap_or(vec![0]),
        poll_interval_ms: poll_interval_ms.unwrap_or(10),
        ..Default::default()
    };
    
    // Use mock backend for now
    let backend = Arc::new(MockRtt::new());
    
    // Create event emitter closure
    let app_clone = app.clone();
    let emit_event = move |event_name: String, payload: serde_json::Value| {
        let _ = app_clone.emit(&event_name, &payload);
    };
    
    run_rtt_job(
        state.job_manager.clone(),
        backend,
        config,
        emit_event,
    ).await
}

/// Stop RTT streaming job
#[tauri::command]
async fn rtt_stream_stop(
    state: State<'_, AppState>,
    rtt_id: String,
) -> Result<bool, String> {
    Ok(state.job_manager.cancel_job(&rtt_id))
}

// ==================== Run Chain Command ====================

/// Run chain guidance: returns the workflow steps for build → flash → rtt
/// Frontend orchestrates by calling each step and listening to events
#[tauri::command]
async fn run_chain(
    project_path: String,
    chip: String,
    start_rtt: Option<bool>,
) -> Result<serde_json::Value, String> {
    // Validate project path exists
    if !std::path::Path::new(&project_path).exists() {
        return Err(format!("Project path not found: {}", project_path));
    }
    
    // Return workflow guidance for frontend to execute
    Ok(serde_json::json!({
        "workflow": [
            {
                "step": 1,
                "action": "streaming_build_start",
                "params": { "project_path": project_path }
            },
            {
                "step": 2,
                "action": "flash_start",
                "params": { "use_latest": true, "chip": chip, "verify": true },
                "trigger": "on build:completed with success=true"
            },
            {
                "step": 3,
                "action": "rtt_stream_start",
                "params": { "chip": chip, "channels": [0] },
                "trigger": "on flash:completed with success=true",
                "enabled": start_rtt.unwrap_or(true)
            }
        ],
        "chip": chip,
        "project_path": project_path,
        "start_rtt_after_flash": start_rtt.unwrap_or(true)
    }))
}

// ==================== Device Status Commands ====================

use jobs::DeviceStatus;

/// Get current device/job status for UI status strip
#[tauri::command]
async fn device_status_get(
    state: State<'_, AppState>,
) -> Result<DeviceStatus, String> {
    Ok(state.job_manager.get_device_status().await)
}

/// Cancel workflow: stops RTT → flash → build in order (single "Stop" button)
#[tauri::command]
async fn workflow_cancel(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let status = state.job_manager.get_device_status().await;
    let mut cancelled = Vec::new();
    
    // Cancel in reverse order: RTT first, then flash, then build
    if let Some(rtt_id) = status.active_rtt_id {
        if state.job_manager.cancel_job(&rtt_id) {
            cancelled.push(("rtt", rtt_id));
        }
    }
    
    if let Some(flash_id) = status.active_flash_id {
        if state.job_manager.cancel_job(&flash_id) {
            cancelled.push(("flash", flash_id));
        }
    }
    
    // Cancel any builds too
    let jobs = state.job_manager.list_jobs(Some(JobKind::Build)).await;
    for job in jobs {
        if job.status.terminal.is_none() {
            if state.job_manager.cancel_job(&job.id) {
                cancelled.push(("build", job.id));
            }
        }
    }
    
    Ok(serde_json::json!({
        "cancelled": cancelled,
        "count": cancelled.len()
    }))
}

// ==================== Generic Job Commands ====================

/// List jobs
#[tauri::command]
async fn job_list(
    state: State<'_, AppState>,
    kind: Option<String>,
) -> Result<Vec<JobInfo>, String> {
    let kind = kind.map(|k| match k.as_str() {
        "build" => JobKind::Build,
        "flash" => JobKind::Flash,
        "rtt" => JobKind::Rtt,
        "agent" => JobKind::Agent,
        _ => JobKind::Build,
    });
    Ok(state.job_manager.list_jobs(kind).await)
}

/// Get job status
#[tauri::command]
async fn job_get_status(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<JobStatus, String> {
    state.job_manager.get_status(&job_id).await
        .ok_or_else(|| format!("Job {} not found", job_id))
}

/// Get job log
#[tauri::command]
async fn job_get_log(
    state: State<'_, AppState>,
    job_id: String,
    last_n: Option<usize>,
) -> Result<Vec<String>, String> {
    state.job_manager.get_log(&job_id, last_n).await
        .ok_or_else(|| format!("Job {} not found", job_id))
}

/// Cancel job
#[tauri::command]
async fn job_cancel(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<bool, String> {
    Ok(state.job_manager.cancel_job(&job_id))
}

// ==================== Tool Registry Commands ====================

use agents::typed_tools::{ToolContext, ToolPermission};

/// List available tools
#[tauri::command]
async fn tool_list(
    state: State<'_, AppState>,
    category: Option<String>,
) -> Result<serde_json::Value, String> {
    let registry = state.tool_registry.lock().await;
    
    let tools: Vec<serde_json::Value> = if let Some(cat) = category {
        let category = match cat.as_str() {
            "fsm" => agents::ToolCategory::FSM,
            "peripheral" => agents::ToolCategory::Peripheral,
            "build" => agents::ToolCategory::Build,
            "debug" => agents::ToolCategory::Debug,
            "docs" => agents::ToolCategory::Documentation,
            _ => agents::ToolCategory::General,
        };
        registry.list_by_category(category)
            .iter()
            .map(|t| serde_json::json!({
                "name": t.name,
                "description": t.description,
                "category": format!("{:?}", t.category),
                "permissions": t.required_permissions,
            }))
            .collect()
    } else {
        registry.list()
            .iter()
            .map(|t| serde_json::json!({
                "name": t.name,
                "description": t.description,
                "category": format!("{:?}", t.category),
                "permissions": t.required_permissions,
            }))
            .collect()
    };
    
    Ok(serde_json::json!({ "tools": tools }))
}

/// Execute a tool
#[tauri::command]
async fn tool_execute(
    state: State<'_, AppState>,
    tool_name: String,
    input: serde_json::Value,
    agent_id: Option<String>,
) -> Result<serde_json::Value, String> {
    let registry = state.tool_registry.lock().await;
    
    // Create context with permissions
    let ctx = ToolContext::new(agent_id.unwrap_or_else(|| "user".to_string()))
        .with_permissions(vec![
            ToolPermission::ReadFSM,
            ToolPermission::WriteFSM,
            ToolPermission::ReadConfig,
            ToolPermission::WriteConfig,
            ToolPermission::RunBuild,
        ]);
    
    registry.execute(&tool_name, input, &ctx)
        .map_err(|e| format!("{}", e))
}

/// Get JSON schemas for all tools (for AI function calling)
#[tauri::command]
async fn tool_get_schemas(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let registry = state.tool_registry.lock().await;
    let schemas = registry.get_schemas();
    Ok(serde_json::json!({ "tools": schemas }))
}

// ==================== Patch/Audit Commands ====================

use agents::diff_engine::{Patch, PatchTarget, PatchOperations, JsonPatchOp};

/// Propose a patch (from agent or user)
#[tauri::command]
async fn patch_propose(
    state: State<'_, AppState>,
    agent_id: String,
    description: String,
    target_type: String,
    operations: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let mut audit_log = state.audit_log.lock().await;
    
    // Parse target
    let target = match target_type.as_str() {
        "fsm_graph" => PatchTarget::FsmGraph,
        "project_settings" => PatchTarget::ProjectSettings,
        _ => return Err(format!("Unknown target type: {}", target_type)),
    };
    
    // Parse operations as JSON patch
    let ops: Vec<JsonPatchOp> = serde_json::from_value(operations)
        .map_err(|e| format!("Invalid patch operations: {}", e))?;
    
    let patch = Patch::json_patch(description, target, ops);
    let entry_id = audit_log.record_proposal(&agent_id, patch.clone());
    
    Ok(serde_json::json!({
        "entry_id": entry_id,
        "patch_id": patch.id,
        "status": "pending"
    }))
}

/// Apply a pending patch
#[tauri::command]
async fn patch_apply(
    state: State<'_, AppState>,
    entry_id: String,
) -> Result<serde_json::Value, String> {
    let mut audit_log = state.audit_log.lock().await;
    audit_log.record_applied(&entry_id);
    
    // In a real implementation, this would:
    // 1. Get the patch from the audit log
    // 2. Apply it to the actual FSM/config state
    // 3. Emit an event to update the UI
    
    Ok(serde_json::json!({
        "entry_id": entry_id,
        "status": "applied"
    }))
}

/// Reject a pending patch
#[tauri::command]
async fn patch_reject(
    state: State<'_, AppState>,
    entry_id: String,
    reason: Option<String>,
) -> Result<serde_json::Value, String> {
    let mut audit_log = state.audit_log.lock().await;
    audit_log.record_rejected(&entry_id);
    
    Ok(serde_json::json!({
        "entry_id": entry_id,
        "status": "rejected",
        "reason": reason
    }))
}

/// Get pending patches for review
#[tauri::command]
async fn patch_get_pending(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let audit_log = state.audit_log.lock().await;
    let pending = audit_log.get_pending();
    
    let entries: Vec<serde_json::Value> = pending.iter()
        .map(|e| serde_json::json!({
            "id": e.id,
            "timestamp": e.timestamp.to_rfc3339(),
            "agent_id": e.agent_id,
            "description": e.patch.description,
            "target": format!("{:?}", e.patch.target),
        }))
        .collect();
    
    Ok(serde_json::json!({ "pending": entries }))
}
