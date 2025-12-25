// Digital Signal Processing Module
// FIR/IIR Filters, FFT, PID Controllers, Circular Buffers

use serde::{Deserialize, Serialize};

// ============================================================================
// Filter Types
// ============================================================================

/// Filter response type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FilterType {
    Lowpass,
    Highpass,
    Bandpass,
    Bandstop,
    Allpass,
}

/// Window function for FIR design
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WindowType {
    Rectangular,
    Hamming,
    Hanning,
    Blackman,
    Kaiser,
}

/// IIR filter topology
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IirTopology {
    DirectForm1,
    DirectForm2,
    Transposed,
    Cascaded,  // Second-order sections
}

// ============================================================================
// FIR Filter Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirConfig {
    pub name: String,
    pub filter_type: FilterType,
    pub order: u16,
    pub sample_rate: f32,
    pub cutoff_freq: f32,
    pub cutoff_freq_high: Option<f32>,  // For bandpass/bandstop
    pub window: WindowType,
    pub coefficients: Option<Vec<f32>>,  // Pre-computed coefficients
}

impl Default for FirConfig {
    fn default() -> Self {
        Self {
            name: "lowpass_filter".to_string(),
            filter_type: FilterType::Lowpass,
            order: 31,
            sample_rate: 48000.0,
            cutoff_freq: 1000.0,
            cutoff_freq_high: None,
            window: WindowType::Hamming,
            coefficients: None,
        }
    }
}

// ============================================================================
// IIR Filter Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IirConfig {
    pub name: String,
    pub filter_type: FilterType,
    pub order: u8,
    pub sample_rate: f32,
    pub cutoff_freq: f32,
    pub cutoff_freq_high: Option<f32>,
    pub topology: IirTopology,
    pub q_factor: f32,
    pub gain_db: f32,
}

impl Default for IirConfig {
    fn default() -> Self {
        Self {
            name: "biquad_filter".to_string(),
            filter_type: FilterType::Lowpass,
            order: 2,
            sample_rate: 48000.0,
            cutoff_freq: 1000.0,
            cutoff_freq_high: None,
            topology: IirTopology::DirectForm2,
            q_factor: 0.707,  // Butterworth
            gain_db: 0.0,
        }
    }
}

// ============================================================================
// FFT Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FftConfig {
    pub name: String,
    pub size: u16,  // Must be power of 2
    pub inverse: bool,
    pub use_cmsis: bool,
    pub window: Option<WindowType>,
}

impl Default for FftConfig {
    fn default() -> Self {
        Self {
            name: "fft_256".to_string(),
            size: 256,
            inverse: false,
            use_cmsis: true,
            window: Some(WindowType::Hanning),
        }
    }
}

// ============================================================================
// PID Controller Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidConfig {
    pub name: String,
    pub kp: f32,
    pub ki: f32,
    pub kd: f32,
    pub output_min: f32,
    pub output_max: f32,
    pub sample_time_ms: u32,
    pub anti_windup: bool,
    pub derivative_filter: bool,  // Low-pass filter on derivative
}

impl Default for PidConfig {
    fn default() -> Self {
        Self {
            name: "pid_controller".to_string(),
            kp: 1.0,
            ki: 0.1,
            kd: 0.01,
            output_min: -100.0,
            output_max: 100.0,
            sample_time_ms: 10,
            anti_windup: true,
            derivative_filter: true,
        }
    }
}

// ============================================================================
// Circular Buffer Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularBufferConfig {
    pub name: String,
    pub size: u32,
    pub element_type: String,  // "int16_t", "float", etc.
    pub thread_safe: bool,
}

impl Default for CircularBufferConfig {
    fn default() -> Self {
        Self {
            name: "audio_buffer".to_string(),
            size: 1024,
            element_type: "int16_t".to_string(),
            thread_safe: false,
        }
    }
}

pub mod filters;
pub mod fft;
pub mod pid;
pub mod buffer;
