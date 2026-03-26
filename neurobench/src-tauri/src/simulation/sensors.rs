//! Sensor Simulation with Noise Models
//!
//! Provides configurable sensor models for testing embedded firmware.

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Noise model types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NoiseModel {
    /// No noise
    None,
    /// Gaussian (normal) distribution
    Gaussian { mean: f64, std_dev: f64 },
    /// Uniform random noise
    Uniform { min: f64, max: f64 },
    /// Pink noise (1/f)
    Pink { amplitude: f64 },
    /// Quantization noise (ADC)
    Quantization { bits: u8 },
    /// Combined noise sources
    Combined(Vec<NoiseModel>),
}

/// Simple pseudo-random number generator
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed.wrapping_add(1) }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        (self.next() as f64) / (u64::MAX as f64)
    }

    /// Box-Muller transform for Gaussian
    fn next_gaussian(&mut self, mean: f64, std_dev: f64) -> f64 {
        let u1 = self.next_f64();
        let u2 = self.next_f64();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();
        mean + std_dev * z
    }
}

/// Sensor simulator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorSimulator {
    /// Sensor name
    pub name: String,
    /// Base value
    pub base_value: f64,
    /// Current value (with noise applied)
    pub current_value: f64,
    /// Noise model
    pub noise_model: NoiseModel,
    /// Minimum output value (clamp)
    pub min_value: f64,
    /// Maximum output value (clamp)
    pub max_value: f64,
    /// Update rate in Hz
    pub update_rate_hz: f32,
    /// Time since last update
    #[serde(skip)]
    last_update_cycles: u64,
    /// Internal RNG seed
    rng_seed: u64,
}

impl SensorSimulator {
    pub fn new(name: &str, base_value: f64, noise_model: NoiseModel) -> Self {
        Self {
            name: name.to_string(),
            base_value,
            current_value: base_value,
            noise_model,
            min_value: f64::MIN,
            max_value: f64::MAX,
            update_rate_hz: 100.0,
            last_update_cycles: 0,
            rng_seed: 42,
        }
    }

    /// Set value range
    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.min_value = min;
        self.max_value = max;
        self
    }

    /// Set update rate
    pub fn with_update_rate(mut self, hz: f32) -> Self {
        self.update_rate_hz = hz;
        self
    }

    /// Apply noise to get noisy reading
    fn apply_noise(&mut self, value: f64) -> f64 {
        let mut rng = SimpleRng::new(self.rng_seed);
        self.rng_seed = rng.next();
        
        let noise = self.calculate_noise(&mut rng, &self.noise_model);
        (value + noise).clamp(self.min_value, self.max_value)
    }

    fn calculate_noise(&self, rng: &mut SimpleRng, model: &NoiseModel) -> f64 {
        match model {
            NoiseModel::None => 0.0,
            NoiseModel::Gaussian { mean, std_dev } => {
                rng.next_gaussian(*mean, *std_dev)
            }
            NoiseModel::Uniform { min, max } => {
                min + rng.next_f64() * (max - min)
            }
            NoiseModel::Pink { amplitude } => {
                // Simplified pink noise approximation
                let white = rng.next_f64() * 2.0 - 1.0;
                white * amplitude * 0.5
            }
            NoiseModel::Quantization { bits } => {
                let step = (self.max_value - self.min_value) / (1 << bits) as f64;
                (rng.next_f64() - 0.5) * step
            }
            NoiseModel::Combined(models) => {
                models.iter().map(|m| self.calculate_noise(rng, m)).sum()
            }
        }
    }

    /// Update sensor (call periodically)
    pub fn update(&mut self, cycles: u64, clock_hz: u32) {
        self.last_update_cycles += cycles;
        
        let cycles_per_update = (clock_hz as f64 / self.update_rate_hz as f64) as u64;
        
        if self.last_update_cycles >= cycles_per_update {
            self.last_update_cycles = 0;
            self.current_value = self.apply_noise(self.base_value);
        }
    }

    /// Set the base value (simulated physical value)
    pub fn set_value(&mut self, value: f64) {
        self.base_value = value;
    }

    /// Get current reading as ADC value
    pub fn read_adc(&self, bits: u8) -> u16 {
        let range = self.max_value - self.min_value;
        let normalized = (self.current_value - self.min_value) / range;
        let max_val = (1 << bits) - 1;
        (normalized * max_val as f64).round().clamp(0.0, max_val as f64) as u16
    }

    /// Get current reading as raw value
    pub fn read(&self) -> f64 {
        self.current_value
    }
}

/// Pre-configured sensor types
pub mod sensors {
    use super::*;

    /// Temperature sensor (e.g., LM35, DS18B20)
    pub fn temperature() -> SensorSimulator {
        SensorSimulator::new(
            "Temperature",
            25.0, // 25°C
            NoiseModel::Gaussian { mean: 0.0, std_dev: 0.2 },
        )
        .with_range(-40.0, 125.0)
        .with_update_rate(10.0)
    }

    /// Humidity sensor (e.g., DHT22)
    pub fn humidity() -> SensorSimulator {
        SensorSimulator::new(
            "Humidity",
            50.0, // 50% RH
            NoiseModel::Gaussian { mean: 0.0, std_dev: 2.0 },
        )
        .with_range(0.0, 100.0)
        .with_update_rate(2.0)
    }

    /// Pressure sensor (e.g., BMP280)
    pub fn pressure() -> SensorSimulator {
        SensorSimulator::new(
            "Pressure",
            101325.0, // 1 atm in Pa
            NoiseModel::Gaussian { mean: 0.0, std_dev: 10.0 },
        )
        .with_range(30000.0, 110000.0)
        .with_update_rate(50.0)
    }

    /// Accelerometer (single axis)
    pub fn accelerometer() -> SensorSimulator {
        SensorSimulator::new(
            "Accelerometer",
            0.0, // g
            NoiseModel::Combined(vec![
                NoiseModel::Gaussian { mean: 0.0, std_dev: 0.01 },
                NoiseModel::Pink { amplitude: 0.005 },
            ]),
        )
        .with_range(-16.0, 16.0) // ±16g
        .with_update_rate(1000.0)
    }

    /// Gyroscope (single axis)
    pub fn gyroscope() -> SensorSimulator {
        SensorSimulator::new(
            "Gyroscope",
            0.0, // dps
            NoiseModel::Combined(vec![
                NoiseModel::Gaussian { mean: 0.0, std_dev: 0.1 },
                NoiseModel::Uniform { min: -0.05, max: 0.05 }, // Bias drift
            ]),
        )
        .with_range(-2000.0, 2000.0) // ±2000 dps
        .with_update_rate(1000.0)
    }

    /// Light sensor (e.g., BH1750)
    pub fn light() -> SensorSimulator {
        SensorSimulator::new(
            "Light",
            500.0, // lux
            NoiseModel::Gaussian { mean: 0.0, std_dev: 5.0 },
        )
        .with_range(0.0, 65535.0)
        .with_update_rate(10.0)
    }

    /// Voltage sensor (ADC input)
    pub fn voltage(max_voltage: f64, bits: u8) -> SensorSimulator {
        SensorSimulator::new(
            "Voltage",
            max_voltage / 2.0,
            NoiseModel::Combined(vec![
                NoiseModel::Gaussian { mean: 0.0, std_dev: max_voltage * 0.001 },
                NoiseModel::Quantization { bits },
            ]),
        )
        .with_range(0.0, max_voltage)
        .with_update_rate(10000.0)
    }

    /// Current sensor (e.g., INA219)
    pub fn current() -> SensorSimulator {
        SensorSimulator::new(
            "Current",
            0.0, // A
            NoiseModel::Gaussian { mean: 0.0, std_dev: 0.001 },
        )
        .with_range(-3.0, 3.0)
        .with_update_rate(100.0)
    }

    /// Distance sensor (e.g., ultrasonic HC-SR04)
    pub fn distance_ultrasonic() -> SensorSimulator {
        SensorSimulator::new(
            "Distance",
            100.0, // cm
            NoiseModel::Combined(vec![
                NoiseModel::Gaussian { mean: 0.0, std_dev: 1.0 },
                NoiseModel::Uniform { min: -0.5, max: 0.5 },
            ]),
        )
        .with_range(2.0, 400.0)
        .with_update_rate(20.0)
    }
}

/// External device base trait
pub trait ExternalDevice: Send + Sync {
    fn name(&self) -> &str;
    fn update(&mut self, cycles: u64, clock_hz: u32);
    fn read(&self, register: u8) -> u8;
    fn write(&mut self, register: u8, value: u8);
}

/// I2C sensor device
pub struct I2cSensorDevice {
    address: u8,
    name: String,
    sensor: SensorSimulator,
    registers: [u8; 256],
}

impl I2cSensorDevice {
    pub fn new(address: u8, name: &str, sensor: SensorSimulator) -> Self {
        Self {
            address,
            name: name.to_string(),
            sensor,
            registers: [0; 256],
        }
    }
}

impl ExternalDevice for I2cSensorDevice {
    fn name(&self) -> &str { &self.name }

    fn update(&mut self, cycles: u64, clock_hz: u32) {
        self.sensor.update(cycles, clock_hz);
        
        // Update data registers
        let value = self.sensor.read_adc(16);
        self.registers[0] = (value >> 8) as u8;
        self.registers[1] = (value & 0xFF) as u8;
    }

    fn read(&self, register: u8) -> u8 {
        self.registers[register as usize]
    }

    fn write(&mut self, register: u8, value: u8) {
        self.registers[register as usize] = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussian_noise() {
        let mut sensor = SensorSimulator::new(
            "Test",
            100.0,
            NoiseModel::Gaussian { mean: 0.0, std_dev: 5.0 },
        );
        
        sensor.update(1000, 1000);
        let value = sensor.read();
        
        // Should be within ~3 standard deviations
        assert!(value > 85.0 && value < 115.0);
    }

    #[test]
    fn test_adc_reading() {
        let sensor = SensorSimulator::new("Test", 50.0, NoiseModel::None)
            .with_range(0.0, 100.0);
        
        let adc = sensor.read_adc(10);
        assert_eq!(adc, 512); // 50% of 1024
    }

    #[test]
    fn test_temperature_sensor() {
        let mut sensor = sensors::temperature();
        sensor.set_value(30.0);
        sensor.update(100000, 1000000);
        
        let temp = sensor.read();
        assert!(temp > 29.0 && temp < 31.0);
    }
}
