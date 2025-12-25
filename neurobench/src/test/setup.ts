// Test setup for Vitest
import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

// Mock Tauri IPC
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string, args?: any) => {
    // Mock responses for different commands
    switch (cmd) {
      case 'validate_code':
        return {
          success: true,
          errors: [],
          warnings: [],
          compiler: 'mock-gcc',
          exitCode: 0,
        };
      
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
      
      case 'generate_ble_service':
        return {
          code: '// BLE Mock\nvoid ble_init(void);',
        };
      
      case 'generate_wifi_config':
        return {
          code: '// WiFi Mock\nvoid wifi_init(void);',
        };
      
      case 'generate_bootloader':
        return {
          code: '// Bootloader Mock\nvoid bootloader_main(void);',
        };
      
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
