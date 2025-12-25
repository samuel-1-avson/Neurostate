// Digital Filter Generators
// FIR and IIR filter code generation with CMSIS-DSP support

use super::*;
use std::f32::consts::PI;

/// Generate FIR filter coefficients (for design)
pub fn design_fir_coefficients(config: &FirConfig) -> Vec<f32> {
    let n = config.order as usize + 1;
    let mut coeffs = vec![0.0f32; n];
    let fc = config.cutoff_freq / config.sample_rate;
    
    // Sinc function for lowpass
    for i in 0..n {
        let m = i as f32 - (n as f32 - 1.0) / 2.0;
        if m == 0.0 {
            coeffs[i] = 2.0 * fc;
        } else {
            coeffs[i] = (2.0 * PI * fc * m).sin() / (PI * m);
        }
    }
    
    // Apply window
    let window = generate_window(config.window, n);
    for i in 0..n {
        coeffs[i] *= window[i];
    }
    
    // Normalize (for unity gain at DC for lowpass)
    let sum: f32 = coeffs.iter().sum();
    if sum.abs() > 1e-10 {
        for c in coeffs.iter_mut() {
            *c /= sum;
        }
    }
    
    coeffs
}

/// Generate window function
fn generate_window(window_type: WindowType, size: usize) -> Vec<f32> {
    let mut w = vec![0.0f32; size];
    let n = size as f32;
    
    for i in 0..size {
        let x = i as f32;
        w[i] = match window_type {
            WindowType::Rectangular => 1.0,
            WindowType::Hamming => 0.54 - 0.46 * (2.0 * PI * x / (n - 1.0)).cos(),
            WindowType::Hanning => 0.5 * (1.0 - (2.0 * PI * x / (n - 1.0)).cos()),
            WindowType::Blackman => {
                0.42 - 0.5 * (2.0 * PI * x / (n - 1.0)).cos() 
                    + 0.08 * (4.0 * PI * x / (n - 1.0)).cos()
            },
            WindowType::Kaiser => 1.0, // Simplified, needs beta parameter
        };
    }
    w
}

/// Generate FIR filter C code (CMSIS-DSP)
pub fn generate_fir_code(config: &FirConfig) -> String {
    let coeffs = config.coefficients.clone()
        .unwrap_or_else(|| design_fir_coefficients(config));
    let num_taps = coeffs.len();
    
    let coeffs_str: String = coeffs.iter()
        .enumerate()
        .map(|(i, c)| {
            if i % 8 == 0 { format!("\n    {:>12.9}f,", c) }
            else { format!(" {:>12.9}f,", c) }
        })
        .collect();

    format!(r#"/**
 * FIR Filter: {name}
 * Type: {filter_type:?}
 * Order: {order}, Taps: {num_taps}
 * Fs: {sample_rate} Hz, Fc: {cutoff} Hz
 * Window: {window:?}
 */

#include "arm_math.h"

#define FIR_{name_upper}_NUM_TAPS {num_taps}
#define FIR_{name_upper}_BLOCK_SIZE 32

static float32_t fir_{name}_coeffs[FIR_{name_upper}_NUM_TAPS] = {{{coeffs_str}
}};

static float32_t fir_{name}_state[FIR_{name_upper}_NUM_TAPS + FIR_{name_upper}_BLOCK_SIZE - 1];
static arm_fir_instance_f32 fir_{name}_instance;

void fir_{name}_init(void) {{
    arm_fir_init_f32(
        &fir_{name}_instance,
        FIR_{name_upper}_NUM_TAPS,
        fir_{name}_coeffs,
        fir_{name}_state,
        FIR_{name_upper}_BLOCK_SIZE
    );
}}

void fir_{name}_process(float32_t *input, float32_t *output, uint32_t block_size) {{
    arm_fir_f32(&fir_{name}_instance, input, output, block_size);
}}

// Single sample processing
float32_t fir_{name}_process_sample(float32_t sample) {{
    float32_t output;
    arm_fir_f32(&fir_{name}_instance, &sample, &output, 1);
    return output;
}}
"#,
        name = config.name,
        name_upper = config.name.to_uppercase(),
        filter_type = config.filter_type,
        order = config.order,
        num_taps = num_taps,
        sample_rate = config.sample_rate,
        cutoff = config.cutoff_freq,
        window = config.window,
        coeffs_str = coeffs_str,
    )
}

/// Generate IIR (Biquad) filter C code
pub fn generate_iir_code(config: &IirConfig) -> String {
    // Calculate biquad coefficients for Butterworth lowpass
    let w0 = 2.0 * PI * config.cutoff_freq / config.sample_rate;
    let alpha = w0.sin() / (2.0 * config.q_factor);
    let cos_w0 = w0.cos();
    
    // Lowpass coefficients
    let (b0, b1, b2, a0, a1, a2) = match config.filter_type {
        FilterType::Lowpass => {
            let b0 = (1.0 - cos_w0) / 2.0;
            let b1 = 1.0 - cos_w0;
            let b2 = (1.0 - cos_w0) / 2.0;
            let a0 = 1.0 + alpha;
            let a1 = -2.0 * cos_w0;
            let a2 = 1.0 - alpha;
            (b0, b1, b2, a0, a1, a2)
        },
        FilterType::Highpass => {
            let b0 = (1.0 + cos_w0) / 2.0;
            let b1 = -(1.0 + cos_w0);
            let b2 = (1.0 + cos_w0) / 2.0;
            let a0 = 1.0 + alpha;
            let a1 = -2.0 * cos_w0;
            let a2 = 1.0 - alpha;
            (b0, b1, b2, a0, a1, a2)
        },
        FilterType::Bandpass => {
            let b0 = alpha;
            let b1 = 0.0;
            let b2 = -alpha;
            let a0 = 1.0 + alpha;
            let a1 = -2.0 * cos_w0;
            let a2 = 1.0 - alpha;
            (b0, b1, b2, a0, a1, a2)
        },
        _ => (1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
    };
    
    // Normalize coefficients
    let b0n = b0 / a0;
    let b1n = b1 / a0;
    let b2n = b2 / a0;
    let a1n = a1 / a0;
    let a2n = a2 / a0;

    format!(r#"/**
 * IIR Biquad Filter: {name}
 * Type: {filter_type:?}
 * Order: {order}
 * Fs: {sample_rate} Hz, Fc: {cutoff} Hz
 * Q: {q}
 */

#include "arm_math.h"

// Biquad coefficients: [b0, b1, b2, a1, a2] (CMSIS format)
// Note: CMSIS uses negated a1, a2
static float32_t biquad_{name}_coeffs[5] = {{
    {b0:.9}f,  // b0
    {b1:.9}f,  // b1
    {b2:.9}f,  // b2
    {a1:.9}f,  // -a1 (negated for CMSIS)
    {a2:.9}f   // -a2 (negated for CMSIS)
}};

static float32_t biquad_{name}_state[4] = {{0}};
static arm_biquad_casd_df1_inst_f32 biquad_{name}_instance;

void biquad_{name}_init(void) {{
    arm_biquad_cascade_df1_init_f32(
        &biquad_{name}_instance,
        1,  // Number of stages
        biquad_{name}_coeffs,
        biquad_{name}_state
    );
}}

void biquad_{name}_process(float32_t *input, float32_t *output, uint32_t block_size) {{
    arm_biquad_cascade_df1_f32(&biquad_{name}_instance, input, output, block_size);
}}

// Direct Form 2 implementation (single sample, no CMSIS)
typedef struct {{
    float32_t b0, b1, b2, a1, a2;
    float32_t z1, z2;
}} biquad_df2_t;

static biquad_df2_t biquad_{name}_df2 = {{
    .b0 = {b0:.9}f, .b1 = {b1:.9}f, .b2 = {b2:.9}f,
    .a1 = {a1_raw:.9}f, .a2 = {a2_raw:.9}f,
    .z1 = 0, .z2 = 0
}};

float32_t biquad_{name}_process_sample(float32_t x) {{
    float32_t w = x - biquad_{name}_df2.a1 * biquad_{name}_df2.z1 
                    - biquad_{name}_df2.a2 * biquad_{name}_df2.z2;
    float32_t y = biquad_{name}_df2.b0 * w 
                + biquad_{name}_df2.b1 * biquad_{name}_df2.z1 
                + biquad_{name}_df2.b2 * biquad_{name}_df2.z2;
    
    biquad_{name}_df2.z2 = biquad_{name}_df2.z1;
    biquad_{name}_df2.z1 = w;
    
    return y;
}}

void biquad_{name}_reset(void) {{
    biquad_{name}_df2.z1 = 0;
    biquad_{name}_df2.z2 = 0;
    arm_biquad_cascade_df1_init_f32(&biquad_{name}_instance, 1, 
                                     biquad_{name}_coeffs, biquad_{name}_state);
}}
"#,
        name = config.name,
        filter_type = config.filter_type,
        order = config.order,
        sample_rate = config.sample_rate,
        cutoff = config.cutoff_freq,
        q = config.q_factor,
        b0 = b0n,
        b1 = b1n,
        b2 = b2n,
        a1 = -a1n,  // Negated for CMSIS
        a2 = -a2n,  // Negated for CMSIS
        a1_raw = a1n,
        a2_raw = a2n,
    )
}
