//! Power Management Controller
//!
//! Simulates low-power modes: Sleep, Stop, Standby

use serde::{Deserialize, Serialize};

/// Power mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerMode {
    /// Normal run mode - all clocks active
    Run,
    /// Low-power run mode - reduced frequency
    LowPowerRun,
    /// Sleep mode - CPU halted, peripherals running
    Sleep,
    /// Low-power sleep - CPU halted, reduced clocks
    LowPowerSleep,
    /// Stop mode - most clocks stopped, wakeup by interrupt
    Stop,
    /// Deep stop - even more peripherals disabled
    DeepStop,
    /// Standby mode - minimal power, RAM lost
    Standby,
    /// Shutdown - lowest power, cold restart required
    Shutdown,
}

impl Default for PowerMode {
    fn default() -> Self {
        Self::Run
    }
}

/// Power configuration for a specific mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerModeConfig {
    pub mode: PowerMode,
    /// CPU active
    pub cpu_active: bool,
    /// Peripherals active
    pub peripherals_active: bool,
    /// Flash accessible
    pub flash_active: bool,
    /// RAM retained
    pub ram_retained: bool,
    /// Wakeup sources enabled
    pub wakeup_sources: WakeupSources,
    /// Clock reduction factor (1 = full speed)
    pub clock_divisor: u8,
    /// Power consumption in microamps (simulated)
    pub power_ua: u32,
}

impl PowerModeConfig {
    pub fn for_mode(mode: PowerMode) -> Self {
        match mode {
            PowerMode::Run => Self {
                mode,
                cpu_active: true,
                peripherals_active: true,
                flash_active: true,
                ram_retained: true,
                wakeup_sources: WakeupSources::all(),
                clock_divisor: 1,
                power_ua: 40_000, // 40mA typical at full speed
            },
            PowerMode::LowPowerRun => Self {
                mode,
                cpu_active: true,
                peripherals_active: true,
                flash_active: true,
                ram_retained: true,
                wakeup_sources: WakeupSources::all(),
                clock_divisor: 4,
                power_ua: 12_000, // 12mA
            },
            PowerMode::Sleep => Self {
                mode,
                cpu_active: false,
                peripherals_active: true,
                flash_active: true,
                ram_retained: true,
                wakeup_sources: WakeupSources::all(),
                clock_divisor: 1,
                power_ua: 5_000, // 5mA
            },
            PowerMode::LowPowerSleep => Self {
                mode,
                cpu_active: false,
                peripherals_active: true,
                flash_active: true,
                ram_retained: true,
                wakeup_sources: WakeupSources::all(),
                clock_divisor: 8,
                power_ua: 1_500, // 1.5mA
            },
            PowerMode::Stop => Self {
                mode,
                cpu_active: false,
                peripherals_active: false,
                flash_active: false,
                ram_retained: true,
                wakeup_sources: WakeupSources::stop_wakeup(),
                clock_divisor: 0,
                power_ua: 10, // 10uA
            },
            PowerMode::DeepStop => Self {
                mode,
                cpu_active: false,
                peripherals_active: false,
                flash_active: false,
                ram_retained: true,
                wakeup_sources: WakeupSources::rtc_only(),
                clock_divisor: 0,
                power_ua: 3, // 3uA
            },
            PowerMode::Standby => Self {
                mode,
                cpu_active: false,
                peripherals_active: false,
                flash_active: false,
                ram_retained: false,
                wakeup_sources: WakeupSources::standby_wakeup(),
                clock_divisor: 0,
                power_ua: 1, // 1uA
            },
            PowerMode::Shutdown => Self {
                mode,
                cpu_active: false,
                peripherals_active: false,
                flash_active: false,
                ram_retained: false,
                wakeup_sources: WakeupSources::reset_only(),
                clock_divisor: 0,
                power_ua: 0, // <1uA
            },
        }
    }
}

/// Wakeup source configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WakeupSources {
    pub any_interrupt: bool,
    pub exti_pins: bool,
    pub rtc_alarm: bool,
    pub rtc_wakeup: bool,
    pub iwdg: bool,
    pub nrst_pin: bool,
    pub usb: bool,
    pub lpuart: bool,
    pub i2c_address: bool,
    pub comparator: bool,
}

impl WakeupSources {
    pub fn all() -> Self {
        Self {
            any_interrupt: true,
            exti_pins: true,
            rtc_alarm: true,
            rtc_wakeup: true,
            iwdg: true,
            nrst_pin: true,
            usb: true,
            lpuart: true,
            i2c_address: true,
            comparator: true,
        }
    }

    pub fn stop_wakeup() -> Self {
        Self {
            any_interrupt: true,
            exti_pins: true,
            rtc_alarm: true,
            rtc_wakeup: true,
            iwdg: true,
            nrst_pin: true,
            usb: false,
            lpuart: true,
            i2c_address: true,
            comparator: true,
        }
    }

    pub fn rtc_only() -> Self {
        Self {
            any_interrupt: false,
            exti_pins: false,
            rtc_alarm: true,
            rtc_wakeup: true,
            iwdg: true,
            nrst_pin: true,
            usb: false,
            lpuart: false,
            i2c_address: false,
            comparator: false,
        }
    }

    pub fn standby_wakeup() -> Self {
        Self {
            any_interrupt: false,
            exti_pins: true, // WKUP pins only
            rtc_alarm: true,
            rtc_wakeup: true,
            iwdg: true,
            nrst_pin: true,
            usb: false,
            lpuart: false,
            i2c_address: false,
            comparator: false,
        }
    }

    pub fn reset_only() -> Self {
        Self {
            any_interrupt: false,
            exti_pins: false,
            rtc_alarm: false,
            rtc_wakeup: false,
            iwdg: false,
            nrst_pin: true,
            usb: false,
            lpuart: false,
            i2c_address: false,
            comparator: false,
        }
    }
}

/// Power Management Controller
pub struct PowerController {
    /// Current power mode
    mode: PowerMode,
    /// Current configuration
    config: PowerModeConfig,
    /// PWR Control register 1
    cr1: u32,
    /// PWR Control register 2
    cr2: u32,
    /// PWR Control register 3
    cr3: u32,
    /// PWR Status register 1
    sr1: u32,
    /// PWR Status register 2
    sr2: u32,
    /// Total energy consumed (in nanojoules, for tracking)
    energy_consumed_nj: u64,
    /// Time in current mode (cycles)
    mode_cycles: u64,
    /// Time spent in each mode (for statistics)
    mode_statistics: [u64; 8],
    /// Wakeup pending
    wakeup_pending: Option<WakeupSource>,
}

/// Wakeup source that triggered exit from low-power mode
#[derive(Debug, Clone, Copy)]
pub enum WakeupSource {
    Interrupt(u8),
    ExtiPin(u8),
    RtcAlarm,
    RtcWakeup,
    Iwdg,
    Reset,
}

impl PowerController {
    pub fn new() -> Self {
        Self {
            mode: PowerMode::Run,
            config: PowerModeConfig::for_mode(PowerMode::Run),
            cr1: 0,
            cr2: 0,
            cr3: 0,
            sr1: 0,
            sr2: 0,
            energy_consumed_nj: 0,
            mode_cycles: 0,
            mode_statistics: [0; 8],
            wakeup_pending: None,
        }
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Get current power mode
    pub fn mode(&self) -> PowerMode {
        self.mode
    }

    /// Get current mode configuration
    pub fn config(&self) -> &PowerModeConfig {
        &self.config
    }

    /// Check if CPU should be running
    pub fn is_cpu_active(&self) -> bool {
        self.config.cpu_active && self.wakeup_pending.is_none()
    }

    /// Check if peripherals should be running
    pub fn are_peripherals_active(&self) -> bool {
        self.config.peripherals_active
    }

    /// Get clock divisor for current mode
    pub fn clock_divisor(&self) -> u8 {
        self.config.clock_divisor
    }

    /// Enter a low-power mode
    pub fn enter_mode(&mut self, mode: PowerMode) {
        // Record time in previous mode
        self.mode_statistics[self.mode as usize] += self.mode_cycles;
        
        self.mode = mode;
        self.config = PowerModeConfig::for_mode(mode);
        self.mode_cycles = 0;
        
        // Update status register
        self.sr1 = (mode as u32) & 0x7;
    }

    /// Tick power controller
    pub fn tick(&mut self, cycles: u64, clock_hz: u32) {
        self.mode_cycles += cycles;
        
        // Calculate energy consumed
        // E = P * t = (V * I) * (cycles / freq)
        // Simplified: power_ua * (cycles * 1000 / clock_hz) = nanojoules
        let time_ns = (cycles as u128 * 1_000_000_000) / clock_hz as u128;
        let energy = (self.config.power_ua as u128 * time_ns) / 1_000_000;
        self.energy_consumed_nj += energy as u64;
    }

    /// Trigger wakeup from low-power mode
    pub fn trigger_wakeup(&mut self, source: WakeupSource) -> bool {
        let can_wake = match source {
            WakeupSource::Interrupt(_) => self.config.wakeup_sources.any_interrupt,
            WakeupSource::ExtiPin(_) => self.config.wakeup_sources.exti_pins,
            WakeupSource::RtcAlarm => self.config.wakeup_sources.rtc_alarm,
            WakeupSource::RtcWakeup => self.config.wakeup_sources.rtc_wakeup,
            WakeupSource::Iwdg => self.config.wakeup_sources.iwdg,
            WakeupSource::Reset => self.config.wakeup_sources.nrst_pin,
        };
        
        if can_wake {
            self.wakeup_pending = Some(source);
            true
        } else {
            false
        }
    }

    /// Process wakeup - returns to Run mode
    pub fn process_wakeup(&mut self) -> Option<WakeupSource> {
        if let Some(source) = self.wakeup_pending.take() {
            self.enter_mode(PowerMode::Run);
            Some(source)
        } else {
            None
        }
    }

    /// Read power register
    pub fn read(&self, offset: u32) -> u32 {
        match offset {
            0x00 => self.cr1,
            0x04 => self.cr2,
            0x08 => self.cr3,
            0x10 => self.sr1,
            0x14 => self.sr2,
            _ => 0,
        }
    }

    /// Write power register
    pub fn write(&mut self, offset: u32, value: u32) {
        match offset {
            0x00 => {
                self.cr1 = value;
                
                // LPMS bits select low-power mode
                let lpms = value & 0x7;
                match lpms {
                    0 => {} // Stay in Run
                    1 => self.enter_mode(PowerMode::Sleep),
                    2 => self.enter_mode(PowerMode::LowPowerSleep),
                    3 => self.enter_mode(PowerMode::Stop),
                    4 => self.enter_mode(PowerMode::DeepStop),
                    5 => self.enter_mode(PowerMode::Standby),
                    6 => self.enter_mode(PowerMode::Shutdown),
                    _ => {}
                }
            }
            0x04 => self.cr2 = value,
            0x08 => self.cr3 = value,
            0x10 => self.sr1 = self.sr1 & !value, // Write 1 to clear
            0x14 => self.sr2 = self.sr2 & !value,
            _ => {}
        }
    }

    /// Get power statistics
    pub fn get_statistics(&self) -> PowerStatistics {
        PowerStatistics {
            current_mode: self.mode,
            energy_consumed_nj: self.energy_consumed_nj,
            mode_cycles: self.mode_statistics,
        }
    }
}

impl Default for PowerController {
    fn default() -> Self {
        Self::new()
    }
}

/// Power statistics for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerStatistics {
    pub current_mode: PowerMode,
    pub energy_consumed_nj: u64,
    pub mode_cycles: [u64; 8],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_modes() {
        let mut pwr = PowerController::new();
        assert_eq!(pwr.mode(), PowerMode::Run);
        assert!(pwr.is_cpu_active());
        
        pwr.enter_mode(PowerMode::Sleep);
        assert_eq!(pwr.mode(), PowerMode::Sleep);
        assert!(!pwr.is_cpu_active());
    }

    #[test]
    fn test_wakeup() {
        let mut pwr = PowerController::new();
        pwr.enter_mode(PowerMode::Stop);
        
        assert!(pwr.trigger_wakeup(WakeupSource::ExtiPin(0)));
        assert!(pwr.wakeup_pending.is_some());
        
        let source = pwr.process_wakeup();
        assert!(source.is_some());
        assert_eq!(pwr.mode(), PowerMode::Run);
    }

    #[test]
    fn test_energy_tracking() {
        let mut pwr = PowerController::new();
        pwr.tick(1_000_000, 100_000_000); // 10ms at 100MHz
        
        assert!(pwr.energy_consumed_nj > 0);
    }
}
