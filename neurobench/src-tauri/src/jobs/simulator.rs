//! FSM Simulator - Test state machines without hardware
//!
//! Provides virtual peripherals and execution tracing.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc};

// =============================================================================
// VIRTUAL PERIPHERALS
// =============================================================================

/// Virtual GPIO state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VirtualGpio {
    pub ports: HashMap<String, u16>,
    pub pin_modes: HashMap<String, PinMode>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PinMode {
    Input,
    Output,
    Analog,
}

impl VirtualGpio {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_pin(&mut self, port: &str, pin: u8, value: bool) {
        let current = self.ports.entry(port.to_string()).or_insert(0);
        if value {
            *current |= 1 << pin;
        } else {
            *current &= !(1 << pin);
        }
    }

    pub fn get_pin(&self, port: &str, pin: u8) -> bool {
        self.ports.get(port).map(|v| (v >> pin) & 1 == 1).unwrap_or(false)
    }
}

/// Virtual Timer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualTimer {
    pub id: String,
    pub period_ms: u32,
    pub counter: u32,
    pub running: bool,
    pub auto_reload: bool,
}

impl VirtualTimer {
    pub fn new(id: &str, period_ms: u32) -> Self {
        Self {
            id: id.to_string(),
            period_ms,
            counter: 0,
            running: false,
            auto_reload: true,
        }
    }

    pub fn tick(&mut self, elapsed_ms: u32) -> bool {
        if !self.running {
            return false;
        }
        
        self.counter += elapsed_ms;
        if self.counter >= self.period_ms {
            if self.auto_reload {
                self.counter %= self.period_ms;
            } else {
                self.running = false;
            }
            return true; // Timer fired
        }
        false
    }
}

// =============================================================================
// SIMULATION STATE
// =============================================================================

/// FSM simulation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationState {
    pub current_state: String,
    pub previous_state: Option<String>,
    pub state_entry_time: DateTime<Utc>,
    pub variables: HashMap<String, SimValue>,
    pub gpio: VirtualGpio,
    pub timers: HashMap<String, VirtualTimer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SimValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            current_state: "IDLE".to_string(),
            previous_state: None,
            state_entry_time: Utc::now(),
            variables: HashMap::new(),
            gpio: VirtualGpio::new(),
            timers: HashMap::new(),
        }
    }
}

// =============================================================================
// TRACE
// =============================================================================

/// Trace event for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: TraceEventType,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceEventType {
    StateEnter,
    StateExit,
    Transition,
    GuardEval,
    ActionExec,
    TimerFire,
    GpioChange,
    VariableSet,
}

// =============================================================================
// SIMULATOR
// =============================================================================

/// FSM Simulator
#[derive(Debug)]
pub struct FsmSimulator {
    pub state: SimulationState,
    pub trace: VecDeque<TraceEvent>,
    pub max_trace_size: usize,
    pub running: bool,
    pub step_count: u64,
}

impl FsmSimulator {
    pub fn new(initial_state: &str) -> Self {
        let mut sim = Self {
            state: SimulationState::default(),
            trace: VecDeque::with_capacity(1000),
            max_trace_size: 1000,
            running: false,
            step_count: 0,
        };
        sim.state.current_state = initial_state.to_string();
        sim.log_trace(TraceEventType::StateEnter, &format!("Initial: {}", initial_state));
        sim
    }

    /// Log a trace event
    fn log_trace(&mut self, event_type: TraceEventType, details: &str) {
        if self.trace.len() >= self.max_trace_size {
            self.trace.pop_front();
        }
        self.trace.push_back(TraceEvent {
            timestamp: Utc::now(),
            event_type,
            details: details.to_string(),
        });
    }

    /// Transition to a new state
    pub fn transition(&mut self, new_state: &str) {
        let old_state = self.state.current_state.clone();
        
        self.log_trace(TraceEventType::StateExit, &old_state);
        self.log_trace(TraceEventType::Transition, &format!("{} -> {}", old_state, new_state));
        
        self.state.previous_state = Some(old_state);
        self.state.current_state = new_state.to_string();
        self.state.state_entry_time = Utc::now();
        
        self.log_trace(TraceEventType::StateEnter, new_state);
    }

    /// Evaluate a guard condition
    pub fn eval_guard(&mut self, guard_name: &str, result: bool) {
        self.log_trace(
            TraceEventType::GuardEval,
            &format!("{}() = {}", guard_name, result),
        );
    }

    /// Execute an action
    pub fn exec_action(&mut self, action_name: &str) {
        self.log_trace(TraceEventType::ActionExec, action_name);
    }

    /// Set a variable
    pub fn set_var(&mut self, name: &str, value: SimValue) {
        self.log_trace(
            TraceEventType::VariableSet,
            &format!("{} = {:?}", name, value),
        );
        self.state.variables.insert(name.to_string(), value);
    }

    /// Set GPIO pin
    pub fn set_gpio(&mut self, port: &str, pin: u8, value: bool) {
        self.state.gpio.set_pin(port, pin, value);
        self.log_trace(
            TraceEventType::GpioChange,
            &format!("{}{} = {}", port, pin, value),
        );
    }

    /// Add a timer
    pub fn add_timer(&mut self, id: &str, period_ms: u32) {
        let timer = VirtualTimer::new(id, period_ms);
        self.state.timers.insert(id.to_string(), timer);
    }

    /// Start a timer
    pub fn start_timer(&mut self, id: &str) {
        if let Some(timer) = self.state.timers.get_mut(id) {
            timer.running = true;
            timer.counter = 0;
        }
    }

    /// Tick all timers, returns fired timer IDs
    pub fn tick_timers(&mut self, elapsed_ms: u32) -> Vec<String> {
        let mut fired = Vec::new();
        for (id, timer) in &mut self.state.timers {
            if timer.tick(elapsed_ms) {
                fired.push(id.clone());
            }
        }
        for id in &fired {
            self.log_trace(TraceEventType::TimerFire, id);
        }
        fired
    }

    /// Step simulation
    pub fn step(&mut self) {
        self.step_count += 1;
    }

    /// Get trace as JSON
    pub fn get_trace(&self, last_n: Option<usize>) -> Vec<TraceEvent> {
        match last_n {
            Some(n) => self.trace.iter().rev().take(n).rev().cloned().collect(),
            None => self.trace.iter().cloned().collect(),
        }
    }

    /// Get current state summary
    pub fn get_summary(&self) -> SimulationSummary {
        SimulationSummary {
            current_state: self.state.current_state.clone(),
            previous_state: self.state.previous_state.clone(),
            step_count: self.step_count,
            trace_count: self.trace.len(),
            variable_count: self.state.variables.len(),
            timer_count: self.state.timers.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationSummary {
    pub current_state: String,
    pub previous_state: Option<String>,
    pub step_count: u64,
    pub trace_count: usize,
    pub variable_count: usize,
    pub timer_count: usize,
}

impl Default for FsmSimulator {
    fn default() -> Self {
        Self::new("IDLE")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulator_basic() {
        let mut sim = FsmSimulator::new("STATE_A");
        
        assert_eq!(sim.state.current_state, "STATE_A");
        
        sim.transition("STATE_B");
        assert_eq!(sim.state.current_state, "STATE_B");
        assert_eq!(sim.state.previous_state, Some("STATE_A".to_string()));
    }

    #[test]
    fn test_gpio() {
        let mut gpio = VirtualGpio::new();
        
        gpio.set_pin("A", 5, true);
        assert!(gpio.get_pin("A", 5));
        
        gpio.set_pin("A", 5, false);
        assert!(!gpio.get_pin("A", 5));
    }

    #[test]
    fn test_timer() {
        let mut timer = VirtualTimer::new("TIM1", 100);
        timer.running = true;
        
        assert!(!timer.tick(50));
        assert!(!timer.tick(40));
        assert!(timer.tick(15)); // Fires at 105ms
    }
}
