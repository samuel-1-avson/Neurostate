// FFT Code Generator
// Fast Fourier Transform using CMSIS-DSP

use super::*;

/// Generate FFT C code (CMSIS-DSP)
pub fn generate_fft_code(config: &FftConfig) -> String {
    let size = config.size;
    
    // Generate window coefficients
    let window_code = if let Some(window) = config.window {
        let window_name = format!("{:?}", window).to_lowercase();
        format!(r#"
// Window coefficients ({window_name})
static float32_t fft_{name}_window[{size}];

void fft_{name}_init_window(void) {{
    for (int i = 0; i < {size}; i++) {{
        float x = (float)i / ({size}.0f - 1.0f);
{window_calc}    }}
}}

void fft_{name}_apply_window(float32_t *data) {{
    for (int i = 0; i < {size}; i++) {{
        data[i] *= fft_{name}_window[i];
    }}
}}
"#,
            name = config.name,
            size = size,
            window_name = window_name,
            window_calc = match window {
                WindowType::Hamming => "        fft_window[i] = 0.54f - 0.46f * arm_cos_f32(2.0f * PI * x);",
                WindowType::Hanning => "        fft_window[i] = 0.5f * (1.0f - arm_cos_f32(2.0f * PI * x));",
                WindowType::Blackman => "        fft_window[i] = 0.42f - 0.5f * arm_cos_f32(2.0f * PI * x) + 0.08f * arm_cos_f32(4.0f * PI * x);",
                _ => "        fft_window[i] = 1.0f;",
            },
        )
    } else {
        String::new()
    };

    format!(r#"/**
 * FFT Configuration: {name}
 * Size: {size} points
 * Inverse: {inverse}
 * CMSIS-DSP: {cmsis}
 */

#include "arm_math.h"
#include "arm_const_structs.h"

#define FFT_{name_upper}_SIZE {size}

// Complex FFT buffers (interleaved real/imag)
static float32_t fft_{name}_input[FFT_{name_upper}_SIZE * 2];
static float32_t fft_{name}_output[FFT_{name_upper}_SIZE * 2];
static float32_t fft_{name}_magnitude[FFT_{name_upper}_SIZE];

// CFFT instance pointer
static const arm_cfft_instance_f32 *fft_{name}_instance = &arm_cfft_sR_f32_len{size};
{window_code}
void fft_{name}_init(void) {{
    // Clear buffers
    arm_fill_f32(0, fft_{name}_input, FFT_{name_upper}_SIZE * 2);
    arm_fill_f32(0, fft_{name}_output, FFT_{name_upper}_SIZE * 2);
{window_init}}}

void fft_{name}_compute(float32_t *real_input) {{
    // Copy real input to complex buffer (imaginary = 0)
    for (int i = 0; i < FFT_{name_upper}_SIZE; i++) {{
        fft_{name}_input[i * 2] = real_input[i];      // Real
        fft_{name}_input[i * 2 + 1] = 0.0f;           // Imag
    }}
    
    // Perform FFT in-place
    arm_cfft_f32(fft_{name}_instance, fft_{name}_input, {ifft_flag}, 1);
    
    // Copy to output buffer
    arm_copy_f32(fft_{name}_input, fft_{name}_output, FFT_{name_upper}_SIZE * 2);
}}

void fft_{name}_compute_magnitude(void) {{
    // Calculate magnitude: sqrt(re^2 + im^2)
    arm_cmplx_mag_f32(fft_{name}_output, fft_{name}_magnitude, FFT_{name_upper}_SIZE);
}}

float32_t *fft_{name}_get_output(void) {{
    return fft_{name}_output;
}}

float32_t *fft_{name}_get_magnitude(void) {{
    return fft_{name}_magnitude;
}}

// Get magnitude at specific bin
float32_t fft_{name}_get_bin_magnitude(uint32_t bin) {{
    if (bin >= FFT_{name_upper}_SIZE) return 0.0f;
    return fft_{name}_magnitude[bin];
}}

// Find peak frequency bin
uint32_t fft_{name}_find_peak(uint32_t start_bin, uint32_t end_bin) {{
    float32_t max_val;
    uint32_t max_idx;
    
    if (end_bin > FFT_{name_upper}_SIZE / 2) {{
        end_bin = FFT_{name_upper}_SIZE / 2;
    }}
    
    arm_max_f32(&fft_{name}_magnitude[start_bin], end_bin - start_bin, &max_val, &max_idx);
    return start_bin + max_idx;
}}

// Convert bin to frequency
float32_t fft_{name}_bin_to_freq(uint32_t bin, float32_t sample_rate) {{
    return (float32_t)bin * sample_rate / (float32_t)FFT_{name_upper}_SIZE;
}}

// Real FFT (optimized for real-only input)
static arm_rfft_fast_instance_f32 rfft_{name}_instance;
static float32_t rfft_{name}_output[FFT_{name_upper}_SIZE];

void rfft_{name}_init(void) {{
    arm_rfft_fast_init_f32(&rfft_{name}_instance, FFT_{name_upper}_SIZE);
}}

void rfft_{name}_compute(float32_t *input, float32_t *output) {{
    arm_rfft_fast_f32(&rfft_{name}_instance, input, output, {ifft_flag});
}}
"#,
        name = config.name,
        name_upper = config.name.to_uppercase(),
        size = size,
        inverse = config.inverse,
        cmsis = config.use_cmsis,
        ifft_flag = if config.inverse { 1 } else { 0 },
        window_code = window_code,
        window_init = if config.window.is_some() {
            format!("    fft_{}_init_window();", config.name)
        } else {
            String::new()
        },
    )
}
