// PID Controller Generator
// Generates PID controller code with anti-windup

use super::*;

/// Generate PID controller C code
pub fn generate_pid_code(config: &PidConfig) -> String {
    format!(r#"/**
 * PID Controller: {name}
 * Kp: {kp}, Ki: {ki}, Kd: {kd}
 * Output range: [{min}, {max}]
 * Sample time: {sample_time}ms
 * Anti-windup: {anti_windup}
 */

#include <stdint.h>
#include <stdbool.h>
#include <math.h>

typedef struct {{
    // Gains
    float kp;
    float ki;
    float kd;
    
    // Output limits
    float out_min;
    float out_max;
    
    // Internal state
    float integral;
    float prev_error;
    float prev_measurement;  // For derivative on measurement
    
    // Derivative filter coefficient
    float d_filter_coeff;
    float d_filtered;
    
    // Configuration
    float sample_time;  // Seconds
    bool anti_windup;
    bool use_deriv_filter;
}} pid_controller_t;

static pid_controller_t {name} = {{
    .kp = {kp}f,
    .ki = {ki}f,
    .kd = {kd}f,
    .out_min = {min}f,
    .out_max = {max}f,
    .integral = 0.0f,
    .prev_error = 0.0f,
    .prev_measurement = 0.0f,
    .d_filter_coeff = 0.1f,
    .d_filtered = 0.0f,
    .sample_time = {sample_time_sec}f,
    .anti_windup = {anti_windup},
    .use_deriv_filter = {deriv_filter}
}};

void {name}_init(void) {{
    {name}.integral = 0.0f;
    {name}.prev_error = 0.0f;
    {name}.prev_measurement = 0.0f;
    {name}.d_filtered = 0.0f;
}}

void {name}_set_gains(float kp, float ki, float kd) {{
    {name}.kp = kp;
    {name}.ki = ki;
    {name}.kd = kd;
}}

void {name}_set_limits(float min, float max) {{
    {name}.out_min = min;
    {name}.out_max = max;
}}

static float clamp(float value, float min, float max) {{
    if (value > max) return max;
    if (value < min) return min;
    return value;
}}

float {name}_update(float setpoint, float measurement) {{
    float error = setpoint - measurement;
    
    // Proportional term
    float p_term = {name}.kp * error;
    
    // Integral term with anti-windup
    {name}.integral += error * {name}.sample_time;
    
    if ({name}.anti_windup) {{
        // Clamp integral to prevent windup
        float i_max = ({name}.out_max - p_term) / {name}.ki;
        float i_min = ({name}.out_min - p_term) / {name}.ki;
        {name}.integral = clamp({name}.integral, i_min, i_max);
    }}
    
    float i_term = {name}.ki * {name}.integral;
    
    // Derivative term (on measurement to avoid derivative kick)
    float derivative;
    if ({name}.use_deriv_filter) {{
        // Low-pass filtered derivative
        float raw_deriv = (measurement - {name}.prev_measurement) / {name}.sample_time;
        {name}.d_filtered = {name}.d_filtered + 
                           {name}.d_filter_coeff * (raw_deriv - {name}.d_filtered);
        derivative = {name}.d_filtered;
    }} else {{
        derivative = (measurement - {name}.prev_measurement) / {name}.sample_time;
    }}
    
    float d_term = -{name}.kd * derivative;  // Negative because derivative on measurement
    
    // Store previous values
    {name}.prev_error = error;
    {name}.prev_measurement = measurement;
    
    // Calculate output
    float output = p_term + i_term + d_term;
    
    // Clamp output
    return clamp(output, {name}.out_min, {name}.out_max);
}}

// Alternative: Derivative on error (may cause "derivative kick" on setpoint changes)
float {name}_update_deriv_on_error(float setpoint, float measurement) {{
    float error = setpoint - measurement;
    
    float p_term = {name}.kp * error;
    
    {name}.integral += error * {name}.sample_time;
    float i_term = {name}.ki * {name}.integral;
    
    float d_term = {name}.kd * (error - {name}.prev_error) / {name}.sample_time;
    
    {name}.prev_error = error;
    
    float output = p_term + i_term + d_term;
    return clamp(output, {name}.out_min, {name}.out_max);
}}

void {name}_reset(void) {{
    {name}.integral = 0.0f;
    {name}.prev_error = 0.0f;
    {name}.prev_measurement = 0.0f;
    {name}.d_filtered = 0.0f;
}}

// Get current state for debugging
void {name}_get_state(float *integral, float *prev_error) {{
    *integral = {name}.integral;
    *prev_error = {name}.prev_error;
}}
"#,
        name = config.name,
        kp = config.kp,
        ki = config.ki,
        kd = config.kd,
        min = config.output_min,
        max = config.output_max,
        sample_time = config.sample_time_ms,
        sample_time_sec = config.sample_time_ms as f32 / 1000.0,
        anti_windup = if config.anti_windup { "true" } else { "false" },
        deriv_filter = if config.derivative_filter { "true" } else { "false" },
    )
}
