//! External Device Simulation
//!
//! Simulates external devices: LCDs, motors, sensors, actuators.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// LCD display type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LcdType {
    /// Character LCD (e.g., HD44780 16x2)
    Character { rows: u8, cols: u8 },
    /// Graphic LCD (e.g., ST7735, ILI9341)
    Graphic { width: u16, height: u16, bits_per_pixel: u8 },
    /// OLED (e.g., SSD1306)
    Oled { width: u16, height: u16 },
}

/// Character LCD simulator (HD44780 compatible)
pub struct CharacterLcd {
    pub lcd_type: LcdType,
    /// Display buffer (rows x cols)
    pub buffer: Vec<Vec<char>>,
    /// Cursor position
    pub cursor_row: u8,
    pub cursor_col: u8,
    /// Display on/off
    pub display_on: bool,
    /// Cursor visible
    pub cursor_visible: bool,
    /// Cursor blinking
    pub cursor_blink: bool,
    /// Entry mode (increment/decrement)
    pub increment: bool,
    /// Display shift on write
    pub shift_display: bool,
    /// DDRAM address
    pub address: u8,
    /// Data/command mode
    pub rs: bool,
    /// Command queue
    commands: VecDeque<u8>,
}

impl CharacterLcd {
    pub fn new(rows: u8, cols: u8) -> Self {
        let buffer = vec![vec![' '; cols as usize]; rows as usize];
        
        Self {
            lcd_type: LcdType::Character { rows, cols },
            buffer,
            cursor_row: 0,
            cursor_col: 0,
            display_on: true,
            cursor_visible: false,
            cursor_blink: false,
            increment: true,
            shift_display: false,
            address: 0,
            rs: false,
            commands: VecDeque::new(),
        }
    }

    /// Write command or data
    pub fn write(&mut self, rs: bool, data: u8) {
        self.rs = rs;
        
        if rs {
            // Data write
            self.write_char(data as char);
        } else {
            // Command
            self.execute_command(data);
        }
    }

    fn write_char(&mut self, c: char) {
        if let LcdType::Character { rows, cols } = self.lcd_type {
            if self.cursor_row < rows && self.cursor_col < cols {
                self.buffer[self.cursor_row as usize][self.cursor_col as usize] = c;
            }
            
            if self.increment {
                self.cursor_col += 1;
                if self.cursor_col >= cols {
                    self.cursor_col = 0;
                    self.cursor_row = (self.cursor_row + 1) % rows;
                }
            }
        }
    }

    fn execute_command(&mut self, cmd: u8) {
        if let LcdType::Character { rows, cols } = self.lcd_type {
            match cmd {
                0x01 => {
                    // Clear display
                    for row in &mut self.buffer {
                        for c in row.iter_mut() {
                            *c = ' ';
                        }
                    }
                    self.cursor_row = 0;
                    self.cursor_col = 0;
                }
                0x02 => {
                    // Return home
                    self.cursor_row = 0;
                    self.cursor_col = 0;
                }
                0x04..=0x07 => {
                    // Entry mode set
                    self.increment = cmd & 0x02 != 0;
                    self.shift_display = cmd & 0x01 != 0;
                }
                0x08..=0x0F => {
                    // Display on/off control
                    self.display_on = cmd & 0x04 != 0;
                    self.cursor_visible = cmd & 0x02 != 0;
                    self.cursor_blink = cmd & 0x01 != 0;
                }
                0x80..=0xFF => {
                    // Set DDRAM address
                    self.address = cmd & 0x7F;
                    // Convert address to row/col
                    if self.address < 0x40 {
                        self.cursor_row = 0;
                        self.cursor_col = (self.address % cols).min(cols - 1);
                    } else {
                        self.cursor_row = (1).min(rows - 1);
                        self.cursor_col = ((self.address - 0x40) % cols).min(cols - 1);
                    }
                }
                _ => {}
            }
        }
    }

    /// Get display content as string
    pub fn get_display(&self) -> Vec<String> {
        self.buffer.iter().map(|row| row.iter().collect()).collect()
    }

    pub fn get_state(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "CharacterLCD",
            "display_on": self.display_on,
            "cursor": [self.cursor_row, self.cursor_col],
            "content": self.get_display(),
        })
    }
}

/// Servo motor simulator
pub struct ServoMotor {
    pub name: String,
    /// Current angle (degrees)
    pub angle: f32,
    /// Target angle
    pub target_angle: f32,
    /// PWM pulse width (microseconds)
    pub pulse_width_us: u16,
    /// Rotation speed (degrees per second)
    pub speed_dps: f32,
    /// Min pulse width (0°)
    pub min_pulse_us: u16,
    /// Max pulse width (180°)
    pub max_pulse_us: u16,
    /// Min angle
    pub min_angle: f32,
    /// Max angle
    pub max_angle: f32,
}

impl ServoMotor {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            angle: 90.0,
            target_angle: 90.0,
            pulse_width_us: 1500,
            speed_dps: 300.0,
            min_pulse_us: 500,
            max_pulse_us: 2500,
            min_angle: 0.0,
            max_angle: 180.0,
        }
    }

    /// Set position from PWM pulse width
    pub fn set_pulse(&mut self, pulse_us: u16) {
        self.pulse_width_us = pulse_us.clamp(self.min_pulse_us, self.max_pulse_us);
        
        // Convert pulse to angle
        let range = self.max_pulse_us - self.min_pulse_us;
        let normalized = (self.pulse_width_us - self.min_pulse_us) as f32 / range as f32;
        self.target_angle = self.min_angle + normalized * (self.max_angle - self.min_angle);
    }

    /// Update servo position
    pub fn update(&mut self, dt_seconds: f32) {
        let diff = self.target_angle - self.angle;
        let max_move = self.speed_dps * dt_seconds;
        
        if diff.abs() <= max_move {
            self.angle = self.target_angle;
        } else {
            self.angle += diff.signum() * max_move;
        }
    }

    pub fn get_state(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "Servo",
            "name": self.name,
            "angle": self.angle,
            "target": self.target_angle,
            "pulse_us": self.pulse_width_us,
        })
    }
}

/// DC motor simulator
pub struct DcMotor {
    pub name: String,
    /// PWM duty cycle (0-100)
    pub duty_cycle: f32,
    /// Direction (true = forward)
    pub forward: bool,
    /// Current RPM
    pub rpm: f32,
    /// Target RPM based on duty
    pub target_rpm: f32,
    /// Max RPM at 100% duty
    pub max_rpm: f32,
    /// Acceleration time constant
    pub time_constant: f32,
}

impl DcMotor {
    pub fn new(name: &str, max_rpm: f32) -> Self {
        Self {
            name: name.to_string(),
            duty_cycle: 0.0,
            forward: true,
            rpm: 0.0,
            target_rpm: 0.0,
            max_rpm,
            time_constant: 0.1,
        }
    }

    /// Set motor from H-bridge inputs
    pub fn set_hbridge(&mut self, in1: bool, in2: bool, pwm_duty: f32) {
        self.duty_cycle = pwm_duty.clamp(0.0, 100.0);
        
        match (in1, in2) {
            (true, false) => {
                self.forward = true;
                self.target_rpm = self.duty_cycle / 100.0 * self.max_rpm;
            }
            (false, true) => {
                self.forward = false;
                self.target_rpm = -(self.duty_cycle / 100.0 * self.max_rpm);
            }
            _ => {
                // Brake or coast
                self.target_rpm = 0.0;
            }
        }
    }

    /// Update motor speed (first-order response)
    pub fn update(&mut self, dt_seconds: f32) {
        let alpha = dt_seconds / (self.time_constant + dt_seconds);
        self.rpm = self.rpm + alpha * (self.target_rpm - self.rpm);
    }

    pub fn get_state(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "DCMotor",
            "name": self.name,
            "rpm": self.rpm,
            "target_rpm": self.target_rpm,
            "duty": self.duty_cycle,
            "forward": self.forward,
        })
    }
}

/// Stepper motor simulator
pub struct StepperMotor {
    pub name: String,
    /// Current step position
    pub steps: i32,
    /// Steps per revolution
    pub steps_per_rev: u16,
    /// Current coil states (4 half-bridge)
    pub coils: [bool; 4],
    /// Microstep position (0-255)
    pub microstep: u8,
    /// Speed (steps per second)
    pub speed_sps: f32,
    /// Target position
    pub target_steps: i32,
}

impl StepperMotor {
    pub fn new(name: &str, steps_per_rev: u16) -> Self {
        Self {
            name: name.to_string(),
            steps: 0,
            steps_per_rev,
            coils: [false; 4],
            microstep: 0,
            speed_sps: 200.0,
            target_steps: 0,
        }
    }

    /// Set coil pattern
    pub fn set_coils(&mut self, coils: [bool; 4]) {
        let old_pattern = self.coils_to_step();
        self.coils = coils;
        let new_pattern = self.coils_to_step();
        
        // Detect step direction
        let diff = (new_pattern as i8 - old_pattern as i8 + 4) % 4;
        if diff == 1 {
            self.steps += 1;
        } else if diff == 3 {
            self.steps -= 1;
        }
    }

    fn coils_to_step(&self) -> u8 {
        match self.coils {
            [true, false, false, false] => 0,
            [true, true, false, false] => 1,
            [false, true, false, false] => 2,
            [false, true, true, false] => 3,
            [false, false, true, false] => 4,
            [false, false, true, true] => 5,
            [false, false, false, true] => 6,
            [true, false, false, true] => 7,
            _ => 0,
        }
    }

    /// Get angle in degrees
    pub fn get_angle(&self) -> f32 {
        (self.steps as f32 / self.steps_per_rev as f32) * 360.0
    }

    pub fn get_state(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "Stepper",
            "name": self.name,
            "steps": self.steps,
            "angle": self.get_angle(),
            "coils": self.coils,
        })
    }
}

/// Relay actuator
pub struct Relay {
    pub name: String,
    pub state: bool,
    pub normally_open: bool,
    /// Contact state (considering NO/NC)
    pub contact_closed: bool,
    /// Energize time (cycles)
    pub energize_delay: u64,
    /// Accumulated cycles since state change
    accumulated_cycles: u64,
}

impl Relay {
    pub fn new(name: &str, normally_open: bool) -> Self {
        Self {
            name: name.to_string(),
            state: false,
            normally_open,
            contact_closed: !normally_open,
            energize_delay: 10000, // ~1ms at 10MHz
            accumulated_cycles: 0,
        }
    }

    pub fn set(&mut self, energized: bool) {
        if self.state != energized {
            self.state = energized;
            self.accumulated_cycles = 0;
        }
    }

    pub fn update(&mut self, cycles: u64) {
        self.accumulated_cycles += cycles;
        
        if self.accumulated_cycles >= self.energize_delay {
            self.contact_closed = if self.normally_open {
                self.state
            } else {
                !self.state
            };
        }
    }

    pub fn get_state(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "Relay",
            "name": self.name,
            "energized": self.state,
            "contact_closed": self.contact_closed,
        })
    }
}

/// RGB LED
pub struct RgbLed {
    pub name: String,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub common_anode: bool,
}

impl RgbLed {
    pub fn new(name: &str, common_anode: bool) -> Self {
        Self {
            name: name.to_string(),
            red: 0,
            green: 0,
            blue: 0,
            common_anode,
        }
    }

    /// Set from PWM duty cycles (0-255)
    pub fn set_pwm(&mut self, r: u8, g: u8, b: u8) {
        if self.common_anode {
            self.red = 255 - r;
            self.green = 255 - g;
            self.blue = 255 - b;
        } else {
            self.red = r;
            self.green = g;
            self.blue = b;
        }
    }

    pub fn get_color(&self) -> u32 {
        ((self.red as u32) << 16) | ((self.green as u32) << 8) | (self.blue as u32)
    }

    pub fn get_state(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "RGB_LED",
            "name": self.name,
            "r": self.red,
            "g": self.green,
            "b": self.blue,
            "hex": format!("#{:02X}{:02X}{:02X}", self.red, self.green, self.blue),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lcd_write() {
        let mut lcd = CharacterLcd::new(2, 16);
        
        lcd.write(true, b'H');
        lcd.write(true, b'i');
        
        assert_eq!(lcd.buffer[0][0], 'H');
        assert_eq!(lcd.buffer[0][1], 'i');
    }

    #[test]
    fn test_servo() {
        let mut servo = ServoMotor::new("Servo1");
        servo.set_pulse(1500); // Center position
        
        assert!((servo.target_angle - 90.0).abs() < 1.0);
    }

    #[test]
    fn test_dc_motor() {
        let mut motor = DcMotor::new("Motor1", 3000.0);
        motor.set_hbridge(true, false, 50.0);
        
        for _ in 0..100 {
            motor.update(0.01);
        }
        
        assert!(motor.rpm > 1000.0);
    }
}
