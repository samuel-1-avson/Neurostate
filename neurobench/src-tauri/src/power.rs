// Power Estimator Module
// Power consumption estimation for embedded systems

use serde::{Deserialize, Serialize};

/// Power profile for MCU state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerProfile {
    pub mode: String,
    pub current_ma: f32,
    pub voltage_v: f32,
    pub description: String,
}

/// Peripheral power consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeripheralPower {
    pub name: String,
    pub active_current_ma: f32,
    pub sleep_current_ma: f32,
}

/// Power estimation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerEstimation {
    pub mcu: String,
    pub total_current_ma: f32,
    pub power_mw: f32,
    pub battery_life_hours: Option<f32>,
    pub breakdown: Vec<PowerBreakdown>,
    pub recommendations: Vec<String>,
}

/// Power breakdown by component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerBreakdown {
    pub component: String,
    pub current_ma: f32,
    pub percent: f32,
}

/// MCU power specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McuPowerSpec {
    pub name: String,
    pub voltage_min: f32,
    pub voltage_max: f32,
    pub voltage_typical: f32,
    pub run_current_ma: f32,
    pub sleep_current_ma: f32,
    pub stop_current_ua: f32,
    pub standby_current_ua: f32,
}

/// Get MCU power specifications
pub fn get_mcu_power_specs() -> Vec<McuPowerSpec> {
    vec![
        McuPowerSpec {
            name: "STM32F407".to_string(),
            voltage_min: 1.8,
            voltage_max: 3.6,
            voltage_typical: 3.3,
            run_current_ma: 50.0,       // @ 168MHz
            sleep_current_ma: 1.0,
            stop_current_ua: 40.0,
            standby_current_ua: 2.0,
        },
        McuPowerSpec {
            name: "STM32F103".to_string(),
            voltage_min: 2.0,
            voltage_max: 3.6,
            voltage_typical: 3.3,
            run_current_ma: 36.0,       // @ 72MHz
            sleep_current_ma: 0.5,
            stop_current_ua: 20.0,
            standby_current_ua: 2.0,
        },
        McuPowerSpec {
            name: "STM32L476".to_string(),
            voltage_min: 1.7,
            voltage_max: 3.6,
            voltage_typical: 3.3,
            run_current_ma: 12.0,       // @ 80MHz
            sleep_current_ma: 0.3,
            stop_current_ua: 0.8,
            standby_current_ua: 0.03,
        },
        McuPowerSpec {
            name: "ESP32".to_string(),
            voltage_min: 2.3,
            voltage_max: 3.6,
            voltage_typical: 3.3,
            run_current_ma: 80.0,       // WiFi active
            sleep_current_ma: 10.0,     // Modem sleep
            stop_current_ua: 150.0,     // Light sleep
            standby_current_ua: 10.0,   // Deep sleep
        },
        McuPowerSpec {
            name: "nRF52832".to_string(),
            voltage_min: 1.7,
            voltage_max: 3.6,
            voltage_typical: 3.0,
            run_current_ma: 5.0,        // @ 64MHz
            sleep_current_ma: 0.003,    // System ON, RAM retention
            stop_current_ua: 1.5,
            standby_current_ua: 0.4,
        },
    ]
}

/// Get peripheral power consumption
pub fn get_peripheral_power() -> Vec<PeripheralPower> {
    vec![
        PeripheralPower {
            name: "UART".to_string(),
            active_current_ma: 0.5,
            sleep_current_ma: 0.0,
        },
        PeripheralPower {
            name: "SPI".to_string(),
            active_current_ma: 1.0,
            sleep_current_ma: 0.0,
        },
        PeripheralPower {
            name: "I2C".to_string(),
            active_current_ma: 0.3,
            sleep_current_ma: 0.0,
        },
        PeripheralPower {
            name: "ADC".to_string(),
            active_current_ma: 2.0,
            sleep_current_ma: 0.0,
        },
        PeripheralPower {
            name: "DAC".to_string(),
            active_current_ma: 1.5,
            sleep_current_ma: 0.0,
        },
        PeripheralPower {
            name: "Timer".to_string(),
            active_current_ma: 0.2,
            sleep_current_ma: 0.0,
        },
        PeripheralPower {
            name: "DMA".to_string(),
            active_current_ma: 0.5,
            sleep_current_ma: 0.0,
        },
        PeripheralPower {
            name: "USB".to_string(),
            active_current_ma: 10.0,
            sleep_current_ma: 0.5,
        },
        PeripheralPower {
            name: "WiFi".to_string(),
            active_current_ma: 80.0,
            sleep_current_ma: 5.0,
        },
        PeripheralPower {
            name: "BLE".to_string(),
            active_current_ma: 15.0,
            sleep_current_ma: 0.01,
        },
    ]
}

/// Estimate power consumption
pub fn estimate_power(
    mcu: &str,
    peripherals: &[String],
    duty_cycle_percent: f32,
    battery_mah: Option<f32>,
) -> Result<PowerEstimation, String> {
    let mcu_spec = get_mcu_power_specs()
        .into_iter()
        .find(|s| s.name.to_lowercase().contains(&mcu.to_lowercase()))
        .ok_or_else(|| format!("MCU {} not found", mcu))?;

    let peripheral_power = get_peripheral_power();
    
    // Calculate MCU power
    let mcu_active = mcu_spec.run_current_ma * (duty_cycle_percent / 100.0);
    let mcu_sleep = mcu_spec.sleep_current_ma * ((100.0 - duty_cycle_percent) / 100.0);
    let mcu_total = mcu_active + mcu_sleep;

    // Calculate peripheral power
    let mut peripheral_total = 0.0f32;
    let mut breakdown = vec![
        PowerBreakdown {
            component: "MCU Core".to_string(),
            current_ma: mcu_total,
            percent: 0.0,  // Will be calculated
        },
    ];

    for periph_name in peripherals {
        if let Some(periph) = peripheral_power.iter().find(|p| 
            p.name.to_lowercase() == periph_name.to_lowercase()
        ) {
            let current = periph.active_current_ma * (duty_cycle_percent / 100.0);
            peripheral_total += current;
            breakdown.push(PowerBreakdown {
                component: periph.name.clone(),
                current_ma: current,
                percent: 0.0,
            });
        }
    }

    let total_current = mcu_total + peripheral_total;
    let power_mw = total_current * mcu_spec.voltage_typical;

    // Calculate percentages
    for b in &mut breakdown {
        b.percent = (b.current_ma / total_current) * 100.0;
    }

    // Battery life calculation
    let battery_life = battery_mah.map(|mah| mah / total_current);

    // Generate recommendations
    let mut recommendations = vec![];
    
    if duty_cycle_percent > 50.0 {
        recommendations.push("Consider reducing active duty cycle for better battery life".to_string());
    }
    
    if peripherals.iter().any(|p| p.to_lowercase() == "wifi") {
        recommendations.push("WiFi is power hungry - use modem sleep when possible".to_string());
    }
    
    if total_current > 100.0 {
        recommendations.push("High current draw - consider low-power MCU variants".to_string());
    }

    if battery_life.map(|h| h < 24.0).unwrap_or(false) {
        recommendations.push("Battery life under 24h - optimize power management".to_string());
    }

    Ok(PowerEstimation {
        mcu: mcu_spec.name,
        total_current_ma: total_current,
        power_mw,
        battery_life_hours: battery_life,
        breakdown,
        recommendations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_mcu_specs() {
        let specs = get_mcu_power_specs();
        assert!(!specs.is_empty());
    }

    #[test]
    fn test_estimate_power() {
        let result = estimate_power(
            "STM32F407",
            &["UART".to_string(), "SPI".to_string()],
            50.0,
            Some(1000.0),
        );
        assert!(result.is_ok());
    }
}
