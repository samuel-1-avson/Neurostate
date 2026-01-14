// Integration Tests
// Tests that verify interaction between multiple components and hooks

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

describe('Integration Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('IPC Communication', () => {
    it('invoke function is mocked and callable', async () => {
      // Test that IPC mock works correctly
      const result = await invoke('get_system_info') as any;
      expect(result).toHaveProperty('name');
      expect(result).toHaveProperty('version');
    });

    it('canvas operations work through IPC', async () => {
      const state = await invoke('canvas_get_state') as any;
      expect(state).toHaveProperty('nodes');
      expect(state).toHaveProperty('edges');
    });

    it('driver generation produces valid output', async () => {
      const result = await invoke('generate_gpio_driver', {
        config: { port: 'A', pin: 0, mode: 'output' }
      }) as any;
      // Driver responses have 'source' property
      expect(result).toHaveProperty('source');
    });

    it('node palette is available', async () => {
      const palette = await invoke('nodes_get_palette');
      expect(palette).toBeInstanceOf(Array);
    });

    it('toolchain discovery works', async () => {
      const toolchains = await invoke('toolchain_discover') as any;
      expect(toolchains).toHaveProperty('arm_gcc');
    });
  });

  describe('Code Validation Integration', () => {
    it('code validation returns structured result', async () => {
      const result = await invoke('validate_code', {
        code: 'int main() { return 0; }',
        language: 'c'
      }) as any;
      expect(result).toHaveProperty('success');
      expect(result).toHaveProperty('errors');
    });
  });

  describe('Build Pipeline Integration', () => {
    it('build process produces result', async () => {
      const result = await invoke('toolchain_build', {
        config: { target: 'stm32f407', sources: ['main.c'] }
      }) as any;
      expect(result).toHaveProperty('success');
    });
  });

  describe('DSP Code Generation', () => {
    it('FIR filter generation works', async () => {
      const result = await invoke('generate_fir_filter', {
        order: 16,
        cutoff: 1000,
        sample_rate: 44100
      }) as any;
      expect(result).toHaveProperty('code');
    });

    it('IIR filter generation works', async () => {
      const result = await invoke('generate_iir_filter', {
        type: 'lowpass',
        order: 2,
        cutoff: 500
      }) as any;
      expect(result).toHaveProperty('code');
    });

    it('FFT block generation works', async () => {
      const result = await invoke('generate_fft_block', {
        size: 1024
      }) as any;
      expect(result).toHaveProperty('code');
    });

    it('PID controller generation works', async () => {
      const result = await invoke('generate_pid_controller', {
        kp: 1.0,
        ki: 0.1,
        kd: 0.01
      }) as any;
      expect(result).toHaveProperty('code');
    });
  });

  describe('Wireless Configuration', () => {
    it('BLE service generation works', async () => {
      const result = await invoke('generate_ble_service', {
        name: 'TestService',
        uuid: '0000-1111-2222-3333'
      }) as any;
      expect(result).toHaveProperty('code');
    });

    it('WiFi config generation works', async () => {
      const result = await invoke('generate_wifi_config', {
        ssid: 'TestNetwork',
        security: 'wpa2'
      }) as any;
      expect(result).toHaveProperty('code');
    });
  });

  describe('Security Code Generation', () => {
    it('bootloader generation works', async () => {
      const result = await invoke('generate_bootloader', {
        type: 'secure',
        target: 'stm32'
      }) as any;
      expect(result).toHaveProperty('code');
    });
  });

  describe('Driver Generation Suite', () => {
    it('UART driver generation works', async () => {
      const result = await invoke('generate_uart_driver', {
        instance: 'USART1',
        baudrate: 115200
      }) as any;
      expect(result).toHaveProperty('source');
    });

    it('SPI driver generation works', async () => {
      const result = await invoke('generate_spi_driver', {
        instance: 'SPI1',
        mode: 'master'
      }) as any;
      expect(result).toHaveProperty('source');
    });
  });
});

