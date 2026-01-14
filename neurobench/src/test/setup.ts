// Test setup for Vitest
import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

// Mock Tauri IPC
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string, args?: any) => {
    // Mock responses for different commands
    switch (cmd) {
      // === System ===
      case 'get_system_info':
        return {
          name: 'NeuroBench',
          version: '0.1.0',
          ai_available: true,
        };
      
      // === Code Validation ===
      case 'validate_code':
        return {
          success: true,
          errors: [],
          warnings: [],
          compiler: 'mock-gcc',
          exitCode: 0,
        };
      
      // === DSP ===
      case 'generate_fir_filter':
        return {
          code: '// FIR Filter Mock\nstatic float coeffs[32];',
        };
      
      case 'generate_iir_filter':
        return {
          code: '// IIR Filter Mock\nstatic float biquad_coeffs[5];',
        };
      
      case 'generate_fft_block':
        return {
          code: '// FFT Mock\n#define FFT_SIZE 256',
        };
      
      case 'generate_pid_controller':
        return {
          code: '// PID Mock\nfloat pid_update(float setpoint, float measurement);',
        };
      
      // === Wireless ===
      case 'generate_ble_service':
        return {
          code: '// BLE Mock\nvoid ble_init(void);',
        };
      
      case 'generate_wifi_config':
        return {
          code: '// WiFi Mock\nvoid wifi_init(void);',
        };
      
      // === Security ===
      case 'generate_bootloader':
        return {
          code: '// Bootloader Mock\nvoid bootloader_main(void);',
        };
      
      // === Canvas ===
      case 'canvas_init':
        return null;
      
      case 'canvas_get_state':
        return {
          nodes: [],
          edges: [],
          selection: [],
        };
      
      case 'canvas_add_node':
        return null;
      
      case 'canvas_connect':
        return { id: 'edge_mock', source: '', target: '', label: '' };
      
      case 'canvas_delete_nodes':
        return { deleted_nodes: [], deleted_edges: [] };
      
      case 'canvas_undo':
      case 'canvas_redo':
        return { nodes: [], edges: [], selection: [] };
      
      case 'canvas_get_edge_paths':
        return {};
      
      // === Driver Generation ===
      case 'generate_gpio_driver':
        return {
          source: '// GPIO Driver Mock\nvoid GPIO_Init(void) {}',
          header: '// GPIO Header\nvoid GPIO_Init(void);',
        };
      
      case 'generate_uart_driver':
        return {
          source: '// UART Driver Mock',
          header: '// UART Header',
        };
      
      case 'generate_spi_driver':
        return {
          source: '// SPI Driver Mock',
          header: '// SPI Header',
        };
      
      // === Build ===
      case 'toolchain_discover':
        return {
          arm_gcc: { available: true, version: '12.2.0', path: '/usr/bin/arm-none-eabi-gcc' },
        };
      
      case 'toolchain_build':
        return {
          success: true,
          elf_path: 'build/firmware.elf',
          errors: [],
          warnings: [],
        };
      
      // === Node Engine ===
      case 'nodes_get_palette':
        return [
          { node_type: 'state', name: 'State', category: 'fsm', icon: '●' },
          { node_type: 'gpio', name: 'GPIO', category: 'hardware', icon: '📍' },
        ];
      
      case 'nodes_get_all_types':
        return [
          { node_type: 'state', name: 'State', category: 'fsm', icon: '●' },
          { node_type: 'gpio', name: 'GPIO', category: 'hardware', icon: '📍' },
          { node_type: 'timer', name: 'Timer', category: 'hardware', icon: '⏱️' },
        ];
      
      case 'nodes_generate_code':
        return {
          includes: ['<stdint.h>'],
          defines: [],
          types: [],
          globals: [],
          init_code: ['GPIO_Init();'],
          handler_code: [],
          main_loop: [],
        };
      
      case 'nodes_generate_files':
        return [
          { name: 'main.c', content: '// Main file', language: 'c' },
        ];
      
      default:
        return { code: `// Mock for ${cmd}` };
    }
  }),
}));


// Mock clipboard API
Object.assign(navigator, {
  clipboard: {
    writeText: vi.fn(() => Promise.resolve()),
    readText: vi.fn(() => Promise.resolve('')),
  },
});
