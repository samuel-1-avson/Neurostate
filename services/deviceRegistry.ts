
import { McuDefinition } from "../types";

export const MCU_REGISTRY: McuDefinition[] = [
  {
    id: 'stm32f401',
    name: 'STM32F401 BlackPill',
    family: 'STM32',
    arch: 'Cortex-M4',
    flashMethod: 'WEB_SERIAL',
    description: 'High-performance DSP with FPU, 84MHz.',
    specs: { flashKB: 512, ramKB: 96, freqMHz: 84, voltage: 3.3 }
  },
  {
    id: 'stm32f103',
    name: 'STM32F103 BluePill',
    family: 'STM32',
    arch: 'Cortex-M3',
    flashMethod: 'WEB_SERIAL',
    description: 'Standard medium-density performance line, 72MHz.',
    specs: { flashKB: 64, ramKB: 20, freqMHz: 72, voltage: 3.3 }
  },
  {
    id: 'esp32_wroom',
    name: 'ESP32 WROOM-32',
    family: 'ESP32',
    arch: 'Xtensa LX6',
    flashMethod: 'WEB_SERIAL',
    description: 'Wi-Fi+BT/BLE MCU module, 240MHz dual core.',
    specs: { flashKB: 4096, ramKB: 520, freqMHz: 240, voltage: 3.3 }
  },
  {
    id: 'rp2040_pico',
    name: 'Raspberry Pi Pico (RP2040)',
    family: 'RP2040',
    arch: 'Cortex-M0+',
    flashMethod: 'USB_MSD',
    description: 'Dual-core M0+ with PIO state machines.',
    specs: { flashKB: 2048, ramKB: 264, freqMHz: 133, voltage: 3.3 }
  },
  {
    id: 'nrf52840',
    name: 'nRF52840 Dongle',
    family: 'NRF52',
    arch: 'Cortex-M4F',
    flashMethod: 'USB_MSD',
    description: 'Bluetooth 5, Thread, Zigbee multi-protocol.',
    specs: { flashKB: 1024, ramKB: 256, freqMHz: 64, voltage: 3.3 }
  },
  {
    id: 'arduino_uno',
    name: 'Arduino Uno (ATmega328P)',
    family: 'AVR',
    arch: 'AVR 8-bit',
    flashMethod: 'WEB_SERIAL',
    description: 'Classic 8-bit AVR microcontroller.',
    specs: { flashKB: 32, ramKB: 2, freqMHz: 16, voltage: 5.0 }
  },
  {
    id: 'teensy_41',
    name: 'Teensy 4.1',
    family: 'OTHER',
    arch: 'Cortex-M7',
    flashMethod: 'DOWNLOAD_BIN',
    description: 'Extreme performance 600MHz MCU.',
    specs: { flashKB: 8192, ramKB: 1024, freqMHz: 600, voltage: 3.3 }
  }
];
