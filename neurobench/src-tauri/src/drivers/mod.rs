// Driver Generator Module
// Generates embedded peripheral drivers for various MCU architectures

pub mod templates;
pub mod gpio;
pub mod uart;
pub mod spi;
pub mod i2c;
pub mod can;
pub mod modbus;
pub mod pins;
pub mod rtos;
pub mod generator;
pub mod interrupts;
pub mod clock;
pub mod analog;
pub mod mcu;
pub mod rtos_gen;
pub mod wireless;
pub mod dsp;
pub mod security;
pub mod export;

// New core drivers
pub mod timer;
pub mod pwm;
pub mod watchdog;
pub mod usb;
pub mod rtc;
pub mod flash;

pub use generator::*;
pub use mcu::{McuFamily, McuInfo, McuHal, get_all_mcus};
pub use rtos_gen::{RtosType, RtosHal, get_rtos_hal};
pub use templates::{DriverError, ConfigValidator, DriverLanguage};
