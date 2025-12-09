
import { McuDefinition } from "../types";

export const MCU_REGISTRY: McuDefinition[] = [
  // --- STM32 FAMILY ---
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
    description: 'Standard medium-density performance line.',
    specs: { flashKB: 64, ramKB: 20, freqMHz: 72, voltage: 3.3 }
  },
  {
    id: 'stm32h743',
    name: 'STM32H743ZI Nucleo',
    family: 'STM32',
    arch: 'Cortex-M7',
    flashMethod: 'USB_MSD',
    description: 'Extreme performance, dual-precision FPU, Chrom-ART.',
    specs: { flashKB: 2048, ramKB: 1024, freqMHz: 480, voltage: 3.3 }
  },
  {
    id: 'stm32l476',
    name: 'STM32L476RG Nucleo',
    family: 'STM32',
    arch: 'Cortex-M4F',
    flashMethod: 'USB_MSD',
    description: 'Ultra-low-power, suitable for battery devices.',
    specs: { flashKB: 1024, ramKB: 128, freqMHz: 80, voltage: 3.3 }
  },
  {
    id: 'stm32f746',
    name: 'STM32F746 Discovery',
    family: 'STM32',
    arch: 'Cortex-M7',
    flashMethod: 'USB_MSD',
    description: 'Multimedia focused with LCD-TFT controller.',
    specs: { flashKB: 1024, ramKB: 320, freqMHz: 216, voltage: 3.3 }
  },
  {
    id: 'stm32g031',
    name: 'STM32G031J6',
    family: 'STM32',
    arch: 'Cortex-M0+',
    flashMethod: 'WEB_SERIAL',
    description: 'Mainstream efficient line, SO8 package.',
    specs: { flashKB: 32, ramKB: 8, freqMHz: 64, voltage: 3.3 }
  },

  // --- ESPRESSIF FAMILY ---
  {
    id: 'esp32_wroom',
    name: 'ESP32 WROOM-32',
    family: 'ESP32',
    arch: 'Xtensa LX6',
    flashMethod: 'WEB_SERIAL',
    description: 'Wi-Fi+BT/BLE MCU module, Dual Core.',
    specs: { flashKB: 4096, ramKB: 520, freqMHz: 240, voltage: 3.3 }
  },
  {
    id: 'esp32s3',
    name: 'ESP32-S3 DevKitC',
    family: 'ESP32',
    arch: 'Xtensa LX7',
    flashMethod: 'WEB_SERIAL',
    description: 'AI/ML focused with vector instructions, Native USB.',
    specs: { flashKB: 8192, ramKB: 512, freqMHz: 240, voltage: 3.3 }
  },
  {
    id: 'esp32c3',
    name: 'ESP32-C3 Mini',
    family: 'ESP32',
    arch: 'RISC-V',
    flashMethod: 'WEB_SERIAL',
    description: 'Ultra-low-power, cost-effective RISC-V WiFi/BLE.',
    specs: { flashKB: 4096, ramKB: 400, freqMHz: 160, voltage: 3.3 }
  },
  {
    id: 'esp8266',
    name: 'NodeMCU v2 (ESP8266)',
    family: 'ESP32',
    arch: 'Xtensa L106',
    flashMethod: 'WEB_SERIAL',
    description: 'Legacy low-cost WiFi module.',
    specs: { flashKB: 4096, ramKB: 50, freqMHz: 80, voltage: 3.3 }
  },

  // --- RASPBERRY PI ---
  {
    id: 'rp2040_pico',
    name: 'Raspberry Pi Pico',
    family: 'RP2040',
    arch: 'Cortex-M0+',
    flashMethod: 'USB_MSD',
    description: 'Dual-core M0+ with PIO state machines.',
    specs: { flashKB: 2048, ramKB: 264, freqMHz: 133, voltage: 3.3 }
  },
  {
    id: 'rp2040_w',
    name: 'Raspberry Pi Pico W',
    family: 'RP2040',
    arch: 'Cortex-M0+',
    flashMethod: 'USB_MSD',
    description: 'Pico with Infineon CYW43439 WiFi/BLE.',
    specs: { flashKB: 2048, ramKB: 264, freqMHz: 133, voltage: 3.3 }
  },

  // --- NORDIC SEMICONDUCTOR ---
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
    id: 'microbit_v2',
    name: 'BBC micro:bit v2',
    family: 'NRF52',
    arch: 'Cortex-M4F',
    flashMethod: 'USB_MSD',
    description: 'Education board with nRF52833 and Sensors.',
    specs: { flashKB: 512, ramKB: 128, freqMHz: 64, voltage: 3.3 }
  },

  // --- MICROCHIP / ATMEL (AVR & SAM) ---
  {
    id: 'arduino_uno',
    name: 'Arduino Uno R3',
    family: 'AVR',
    arch: 'AVR 8-bit',
    flashMethod: 'WEB_SERIAL',
    description: 'The classic ATmega328P development board.',
    specs: { flashKB: 32, ramKB: 2, freqMHz: 16, voltage: 5.0 }
  },
  {
    id: 'arduino_mega',
    name: 'Arduino Mega 2560',
    family: 'AVR',
    arch: 'AVR 8-bit',
    flashMethod: 'WEB_SERIAL',
    description: 'More I/O, more memory. ATmega2560.',
    specs: { flashKB: 256, ramKB: 8, freqMHz: 16, voltage: 5.0 }
  },
  {
    id: 'arduino_leo',
    name: 'Arduino Leonardo',
    family: 'AVR',
    arch: 'AVR 8-bit',
    flashMethod: 'WEB_SERIAL',
    description: 'ATmega32U4 with native USB HID support.',
    specs: { flashKB: 32, ramKB: 2.5, freqMHz: 16, voltage: 5.0 }
  },
  {
    id: 'feather_m0',
    name: 'Adafruit Feather M0',
    family: 'OTHER',
    arch: 'Cortex-M0+',
    flashMethod: 'USB_MSD',
    description: 'ATSAMD21G18 based compact board.',
    specs: { flashKB: 256, ramKB: 32, freqMHz: 48, voltage: 3.3 }
  },
  {
    id: 'metro_m4',
    name: 'Adafruit Metro M4',
    family: 'OTHER',
    arch: 'Cortex-M4F',
    flashMethod: 'USB_MSD',
    description: 'ATSAMD51 based, fast with crypto engines.',
    specs: { flashKB: 512, ramKB: 192, freqMHz: 120, voltage: 3.3 }
  },
  {
    id: 'pic32mz',
    name: 'Curiosity PIC32MZ',
    family: 'PIC',
    arch: 'MIPS32',
    flashMethod: 'DOWNLOAD_BIN',
    description: 'High-performance MIPS microAptiv core.',
    specs: { flashKB: 2048, ramKB: 512, freqMHz: 200, voltage: 3.3 }
  },

  // --- TEXAS INSTRUMENTS ---
  {
    id: 'msp430g2',
    name: 'MSP430 LaunchPad',
    family: 'OTHER',
    arch: 'MSP430',
    flashMethod: 'WEB_SERIAL',
    description: 'Ultra-low-power 16-bit RISC architecture.',
    specs: { flashKB: 16, ramKB: 0.5, freqMHz: 16, voltage: 3.3 }
  },
  {
    id: 'tm4c123',
    name: 'Tiva C TM4C123G',
    family: 'OTHER',
    arch: 'Cortex-M4F',
    flashMethod: 'DOWNLOAD_BIN',
    description: 'Robust ARM MCU for industrial applications.',
    specs: { flashKB: 256, ramKB: 32, freqMHz: 80, voltage: 3.3 }
  },

  // --- RISC-V & OTHERS ---
  {
    id: 'hifive1',
    name: 'SiFive HiFive1 Rev B',
    family: 'RISC-V',
    arch: 'RISC-V (FE310)',
    flashMethod: 'USB_MSD',
    description: 'The first open-source RISC-V dev kit.',
    specs: { flashKB: 16384, ramKB: 16, freqMHz: 320, voltage: 3.3 }
  },
  {
    id: 'longan_nano',
    name: 'Sipeed Longan Nano',
    family: 'RISC-V',
    arch: 'GD32V',
    flashMethod: 'USB_MSD',
    description: 'GD32VF103C8T6 RISC-V with small screen.',
    specs: { flashKB: 64, ramKB: 20, freqMHz: 108, voltage: 3.3 }
  },
  {
    id: 'teensy_41',
    name: 'Teensy 4.1',
    family: 'OTHER',
    arch: 'Cortex-M7',
    flashMethod: 'DOWNLOAD_BIN',
    description: 'Extreme performance 600MHz MCU.',
    specs: { flashKB: 8192, ramKB: 1024, freqMHz: 600, voltage: 3.3 }
  },
  {
    id: 'sony_spresense',
    name: 'Sony Spresense',
    family: 'OTHER',
    arch: 'Cortex-M4F (x6)',
    flashMethod: 'WEB_SERIAL',
    description: 'Hexa-core microcontroller for Audio/AI.',
    specs: { flashKB: 8192, ramKB: 1536, freqMHz: 156, voltage: 1.8 }
  }
];
