//! FSM Behavior Simulator
//!
//! Bridges the canvas FSM design with actual peripheral simulation.
//! Parses node actions and maps them to simulated hardware behavior.

use crate::core::types::{FSMNode, FSMEdge, NodeType, NodeId, SimulationStatus};
use crate::core::graph::FSMGraph;
use crate::simulation::peripherals::PeripheralBus;
use crate::simulation::engine::SimulationConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Peripheral action parsed from node entry/exit actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeripheralAction {
    /// Set GPIO pin state: port, pin, high/low
    GpioSet { port: char, pin: u8, state: bool },
    /// Toggle GPIO pin
    GpioToggle { port: char, pin: u8 },
    /// Read GPIO pin
    GpioRead { port: char, pin: u8 },
    /// Send UART data
    UartSend { instance: u8, data: Vec<u8> },
    /// Start a timer
    TimerStart { timer: u8, period_ms: u32 },
    /// Stop a timer
    TimerStop { timer: u8 },
    /// Read ADC channel
    AdcRead { channel: u8 },
    /// Set PWM duty cycle
    PwmSet { timer: u8, channel: u8, duty: u16 },
    /// Send SPI data
    SpiSend { instance: u8, data: Vec<u8> },
    /// Send I2C data
    I2cSend { instance: u8, address: u8, data: Vec<u8> },
    /// Delay in milliseconds (simulated)
    Delay { ms: u32 },
    /// Log a message
    Log { message: String },
    /// Custom action (unparsed)
    Custom { action: String },
}

/// Result of executing a peripheral action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub action: String,
    pub success: bool,
    pub value: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// State of the behavioral simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorState {
    pub current_node: Option<NodeId>,
    pub status: SimulationStatus,
    pub step_count: u64,
    pub simulated_time_ms: u64,
    pub gpio_state: HashMap<String, bool>, // "A5" -> true
    pub uart_output: HashMap<u8, Vec<u8>>,
    pub adc_values: HashMap<u8, u16>,
    pub timer_active: HashMap<u8, bool>,
    pub logs: Vec<String>,
}

impl Default for BehaviorState {
    fn default() -> Self {
        Self {
            current_node: None,
            status: SimulationStatus::Idle,
            step_count: 0,
            simulated_time_ms: 0,
            gpio_state: HashMap::new(),
            uart_output: HashMap::new(),
            adc_values: HashMap::new(),
            timer_active: HashMap::new(),
            logs: vec![],
        }
    }
}

/// FSM Behavior Simulator
/// Executes FSM designs with simulated peripheral behavior
pub struct FsmBehaviorSimulator {
    graph: Arc<Mutex<FSMGraph>>,
    peripheral_bus: Option<PeripheralBus>,
    state: BehaviorState,
    action_history: Vec<ActionResult>,
}

impl FsmBehaviorSimulator {
    /// Create a new behavior simulator from an FSM graph
    pub fn new(graph: FSMGraph) -> Self {
        Self {
            graph: Arc::new(Mutex::new(graph)),
            peripheral_bus: None,
            state: BehaviorState::default(),
            action_history: vec![],
        }
    }

    /// Create with a peripheral bus for hardware-level simulation
    pub fn with_peripherals(graph: FSMGraph, config: &SimulationConfig) -> Self {
        Self {
            graph: Arc::new(Mutex::new(graph)),
            peripheral_bus: Some(PeripheralBus::new(config)),
            state: BehaviorState::default(),
            action_history: vec![],
        }
    }

    /// Start the simulation from the initial node
    pub fn start(&mut self) -> Result<(), String> {
        let start_id = {
            let graph = self.graph.lock().map_err(|e| e.to_string())?;
            let start_node = graph.find_start_node()
                .ok_or("No start node found in the FSM")?;
            start_node.id
        };

        self.state.current_node = Some(start_id);
        self.state.status = SimulationStatus::Running;
        self.state.step_count = 0;
        self.state.simulated_time_ms = 0;
        self.state.logs.clear();
        self.action_history.clear();

        // Execute entry action of start node
        self.execute_current_entry_action()?;

        self.log(format!("Simulation started at node {:?}", start_id));
        Ok(())
    }

    /// Stop the simulation
    pub fn stop(&mut self) {
        self.state.status = SimulationStatus::Idle;
        self.log("Simulation stopped".to_string());
    }

    /// Pause the simulation
    pub fn pause(&mut self) {
        self.state.status = SimulationStatus::Paused;
        self.log("Simulation paused".to_string());
    }

    /// Resume the simulation
    pub fn resume(&mut self) {
        if self.state.status == SimulationStatus::Paused {
            self.state.status = SimulationStatus::Running;
            self.log("Simulation resumed".to_string());
        }
    }

    /// Execute a single step
    pub fn step(&mut self) -> Result<StepResult, String> {
        if self.state.status != SimulationStatus::Running && 
           self.state.status != SimulationStatus::Stepping {
            return Err("Simulation not running".to_string());
        }

        let current_id = self.state.current_node
            .ok_or("No current state")?;

        // Get transition data
        let transition = {
            let graph = self.graph.lock().map_err(|e| e.to_string())?;
            
            // Check node type for special handling
            let current_node = graph.get_node(current_id).ok_or("Node not found")?;
            
            // Get outgoing edges
            let edges = graph.get_outgoing(current_id);
            
            if edges.is_empty() {
                if current_node.node_type == NodeType::Output {
                    return Ok(StepResult::Completed);
                } else {
                    return Ok(StepResult::Deadlock);
                }
            }

            // Find the first edge with satisfied guard (or no guard)
            let mut selected_edge = None;
            for edge in edges {
                if self.evaluate_guard(&edge) {
                    selected_edge = Some((
                        edge.target,
                        edge.label.clone(),
                        current_node.exit_action.clone(),
                        graph.get_node(edge.target).and_then(|n| n.entry_action.clone()),
                        graph.get_node(edge.target).map(|n| n.node_type),
                        graph.get_node(edge.target).map(|n| n.label.clone()),
                    ));
                    break;
                }
            }

            selected_edge.ok_or("No valid transition found")?
        };

        let (next_id, edge_label, exit_action, entry_action, next_type, next_label) = transition;

        // Execute exit action
        if let Some(ref action) = exit_action {
            self.execute_action(action)?;
        }

        // Handle node-type specific behavior during transition
        if let Some(label) = &edge_label {
            self.log(format!("Transition: {}", label));
        }

        // Move to next node
        self.state.current_node = Some(next_id);
        self.state.step_count += 1;

        // Execute entry action
        if let Some(ref action) = entry_action {
            self.execute_action(action)?;
        }

        // Handle special node types
        if let Some(node_type) = next_type {
            self.handle_node_type(node_type, next_label.as_deref())?;
        }

        Ok(StepResult::Transitioned { 
            from: current_id, 
            to: next_id,
            actions_executed: self.action_history.len(),
        })
    }

    /// Execute an action string (entry/exit action from a node)
    fn execute_action(&mut self, action: &str) -> Result<(), String> {
        let parsed_actions = self.parse_action(action);
        
        for pa in parsed_actions {
            let result = self.execute_peripheral_action(&pa);
            self.action_history.push(result);
        }
        
        Ok(())
    }

    /// Parse an action string into peripheral actions
    fn parse_action(&self, action: &str) -> Vec<PeripheralAction> {
        let mut actions = vec![];
        
        for line in action.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            // Parse GPIO actions
            // Format: GPIO.A5 = HIGH, GPIO_SET(A, 5, true), gpio_write(PA5, 1)
            if let Some(gpio_action) = self.parse_gpio_action(line) {
                actions.push(gpio_action);
                continue;
            }

            // Parse UART actions
            // Format: UART1.send("Hello"), uart_write(1, "data")
            if let Some(uart_action) = self.parse_uart_action(line) {
                actions.push(uart_action);
                continue;
            }

            // Parse Timer actions
            // Format: TIM2.start(100), timer_start(2, 100)
            if let Some(timer_action) = self.parse_timer_action(line) {
                actions.push(timer_action);
                continue;
            }

            // Parse delay actions
            // Format: delay(100), delay_ms(100)
            if let Some(delay_action) = self.parse_delay_action(line) {
                actions.push(delay_action);
                continue;
            }

            // Parse log actions
            // Format: log("message"), print("message")
            if let Some(log_action) = self.parse_log_action(line) {
                actions.push(log_action);
                continue;
            }

            // Default: treat as custom action
            actions.push(PeripheralAction::Custom { action: line.to_string() });
        }

        actions
    }

    /// Parse GPIO action from a line
    fn parse_gpio_action(&self, line: &str) -> Option<PeripheralAction> {
        // Pattern: GPIO.A5 = HIGH
        if line.starts_with("GPIO.") || line.starts_with("gpio.") {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() == 2 {
                let pin_spec = parts[0].trim().replace("GPIO.", "").replace("gpio.", "");
                let state_str = parts[1].trim().to_uppercase();
                
                if let Some(port) = pin_spec.chars().next() {
                    if let Ok(pin) = pin_spec[1..].parse::<u8>() {
                        let state = state_str == "HIGH" || state_str == "1" || state_str == "TRUE";
                        return Some(PeripheralAction::GpioSet { port, pin, state });
                    }
                }
            }
        }

        // Pattern: GPIO_SET(A, 5, true)
        if line.to_uppercase().starts_with("GPIO_SET") || line.to_uppercase().starts_with("GPIO_WRITE") {
            if let Some(args) = extract_parens(line) {
                let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
                if parts.len() == 3 {
                    if let Some(port) = parts[0].chars().next() {
                        if let Ok(pin) = parts[1].parse::<u8>() {
                            let state = parts[2] == "true" || parts[2] == "1" || parts[2].to_uppercase() == "HIGH";
                            return Some(PeripheralAction::GpioSet { port, pin, state });
                        }
                    }
                }
            }
        }

        // Pattern: PA5 = 1
        if line.len() >= 3 && line.starts_with('P') {
            let port = line.chars().nth(1)?;
            if port.is_ascii_uppercase() {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 2 {
                    if let Ok(pin) = parts[0][2..].trim().parse::<u8>() {
                        let state = parts[1].trim() == "1" || parts[1].trim().to_uppercase() == "HIGH";
                        return Some(PeripheralAction::GpioSet { port, pin, state });
                    }
                }
            }
        }

        None
    }

    /// Parse UART action from a line
    fn parse_uart_action(&self, line: &str) -> Option<PeripheralAction> {
        // Pattern: UART1.send("Hello")
        if line.to_uppercase().starts_with("UART") || line.to_uppercase().starts_with("USART") {
            let instance = line.chars().filter(|c| c.is_ascii_digit()).next()
                .and_then(|c| c.to_digit(10).map(|d| d as u8))
                .unwrap_or(1);
            
            if let Some(content) = extract_string_arg(line) {
                return Some(PeripheralAction::UartSend { 
                    instance, 
                    data: content.as_bytes().to_vec() 
                });
            }
        }

        // Pattern: serial_print("text")
        if line.to_lowercase().starts_with("serial_print") || line.to_lowercase().starts_with("print") {
            if let Some(content) = extract_string_arg(line) {
                return Some(PeripheralAction::UartSend { 
                    instance: 1, 
                    data: content.as_bytes().to_vec() 
                });
            }
        }

        None
    }

    /// Parse Timer action from a line
    fn parse_timer_action(&self, line: &str) -> Option<PeripheralAction> {
        let upper = line.to_uppercase();
        
        // Pattern: TIM2.start(100)
        if upper.starts_with("TIM") && upper.contains("START") {
            let timer = line.chars().filter(|c| c.is_ascii_digit()).next()
                .and_then(|c| c.to_digit(10).map(|d| d as u8))
                .unwrap_or(1);
            
            if let Some(args) = extract_parens(line) {
                if let Ok(period) = args.trim().parse::<u32>() {
                    return Some(PeripheralAction::TimerStart { timer, period_ms: period });
                }
            }
        }

        // Pattern: TIM2.stop()
        if upper.starts_with("TIM") && upper.contains("STOP") {
            let timer = line.chars().filter(|c| c.is_ascii_digit()).next()
                .and_then(|c| c.to_digit(10).map(|d| d as u8))
                .unwrap_or(1);
            return Some(PeripheralAction::TimerStop { timer });
        }

        None
    }

    /// Parse delay action from a line
    fn parse_delay_action(&self, line: &str) -> Option<PeripheralAction> {
        let lower = line.to_lowercase();
        
        if lower.starts_with("delay") || lower.starts_with("wait") || lower.starts_with("sleep") {
            if let Some(args) = extract_parens(line) {
                if let Ok(ms) = args.trim().parse::<u32>() {
                    return Some(PeripheralAction::Delay { ms });
                }
            }
        }

        None
    }

    /// Parse log action from a line
    fn parse_log_action(&self, line: &str) -> Option<PeripheralAction> {
        let lower = line.to_lowercase();
        
        if lower.starts_with("log") || lower.starts_with("debug") {
            if let Some(content) = extract_string_arg(line) {
                return Some(PeripheralAction::Log { message: content });
            }
        }

        None
    }

    /// Execute a parsed peripheral action
    fn execute_peripheral_action(&mut self, action: &PeripheralAction) -> ActionResult {
        match action {
            PeripheralAction::GpioSet { port, pin, state } => {
                let key = format!("{}{}", port, pin);
                self.state.gpio_state.insert(key.clone(), *state);
                
                // If we have a peripheral bus, also set it there
                if let Some(ref mut bus) = self.peripheral_bus {
                    bus.gpio_inject(*port, *pin, *state);
                }
                
                self.log(format!("GPIO {}{} = {}", port, pin, if *state { "HIGH" } else { "LOW" }));
                
                ActionResult {
                    action: format!("GPIO_SET({}, {}, {})", port, pin, state),
                    success: true,
                    value: Some(serde_json::json!({ "pin": key, "state": state })),
                    error: None,
                }
            }
            PeripheralAction::GpioToggle { port, pin } => {
                let key = format!("{}{}", port, pin);
                let current = *self.state.gpio_state.get(&key).unwrap_or(&false);
                let new_state = !current;
                self.state.gpio_state.insert(key.clone(), new_state);
                
                if let Some(ref mut bus) = self.peripheral_bus {
                    bus.gpio_inject(*port, *pin, new_state);
                }
                
                self.log(format!("GPIO {}{} toggled to {}", port, pin, if new_state { "HIGH" } else { "LOW" }));
                
                ActionResult {
                    action: format!("GPIO_TOGGLE({}, {})", port, pin),
                    success: true,
                    value: Some(serde_json::json!({ "pin": key, "state": new_state })),
                    error: None,
                }
            }
            PeripheralAction::GpioRead { port, pin } => {
                let key = format!("{}{}", port, pin);
                let state = *self.state.gpio_state.get(&key).unwrap_or(&false);
                
                ActionResult {
                    action: format!("GPIO_READ({}, {})", port, pin),
                    success: true,
                    value: Some(serde_json::json!({ "pin": key, "state": state })),
                    error: None,
                }
            }
            PeripheralAction::UartSend { instance, data } => {
                let entry = self.state.uart_output.entry(*instance).or_insert_with(Vec::new);
                entry.extend_from_slice(data);
                
                if let Some(ref mut bus) = self.peripheral_bus {
                    bus.uart_inject(*instance, data);
                }
                
                let text = String::from_utf8_lossy(data);
                self.log(format!("UART{}: {}", instance, text));
                
                ActionResult {
                    action: format!("UART_SEND({}, {} bytes)", instance, data.len()),
                    success: true,
                    value: Some(serde_json::json!({ 
                        "instance": instance, 
                        "data": text.to_string(),
                        "bytes": data.len()
                    })),
                    error: None,
                }
            }
            PeripheralAction::TimerStart { timer, period_ms } => {
                self.state.timer_active.insert(*timer, true);
                self.log(format!("TIM{} started with period {}ms", timer, period_ms));
                
                ActionResult {
                    action: format!("TIMER_START({}, {}ms)", timer, period_ms),
                    success: true,
                    value: Some(serde_json::json!({ "timer": timer, "period_ms": period_ms })),
                    error: None,
                }
            }
            PeripheralAction::TimerStop { timer } => {
                self.state.timer_active.insert(*timer, false);
                self.log(format!("TIM{} stopped", timer));
                
                ActionResult {
                    action: format!("TIMER_STOP({})", timer),
                    success: true,
                    value: None,
                    error: None,
                }
            }
            PeripheralAction::AdcRead { channel } => {
                let value = *self.state.adc_values.get(channel).unwrap_or(&2048);
                
                ActionResult {
                    action: format!("ADC_READ({})", channel),
                    success: true,
                    value: Some(serde_json::json!({ "channel": channel, "value": value })),
                    error: None,
                }
            }
            PeripheralAction::PwmSet { timer, channel, duty } => {
                self.log(format!("PWM TIM{} CH{} duty = {}", timer, channel, duty));
                
                ActionResult {
                    action: format!("PWM_SET({}, {}, {})", timer, channel, duty),
                    success: true,
                    value: Some(serde_json::json!({ "timer": timer, "channel": channel, "duty": duty })),
                    error: None,
                }
            }
            PeripheralAction::SpiSend { instance, data } => {
                self.log(format!("SPI{}: {} bytes", instance, data.len()));
                
                ActionResult {
                    action: format!("SPI_SEND({}, {} bytes)", instance, data.len()),
                    success: true,
                    value: Some(serde_json::json!({ "instance": instance, "bytes": data.len() })),
                    error: None,
                }
            }
            PeripheralAction::I2cSend { instance, address, data } => {
                self.log(format!("I2C{} @ 0x{:02X}: {} bytes", instance, address, data.len()));
                
                ActionResult {
                    action: format!("I2C_SEND({}, 0x{:02X}, {} bytes)", instance, address, data.len()),
                    success: true,
                    value: Some(serde_json::json!({ 
                        "instance": instance, 
                        "address": address,
                        "bytes": data.len() 
                    })),
                    error: None,
                }
            }
            PeripheralAction::Delay { ms } => {
                self.state.simulated_time_ms += *ms as u64;
                self.log(format!("Delay {}ms (simulated)", ms));
                
                ActionResult {
                    action: format!("DELAY({}ms)", ms),
                    success: true,
                    value: Some(serde_json::json!({ "ms": ms, "total_time_ms": self.state.simulated_time_ms })),
                    error: None,
                }
            }
            PeripheralAction::Log { message } => {
                self.log(format!("LOG: {}", message));
                
                ActionResult {
                    action: "LOG".to_string(),
                    success: true,
                    value: Some(serde_json::json!({ "message": message })),
                    error: None,
                }
            }
            PeripheralAction::Custom { action } => {
                self.log(format!("Custom: {}", action));
                
                ActionResult {
                    action: "CUSTOM".to_string(),
                    success: true,
                    value: Some(serde_json::json!({ "action": action })),
                    error: None,
                }
            }
        }
    }

    /// Handle special behavior based on node type
    fn handle_node_type(&mut self, node_type: NodeType, label: Option<&str>) -> Result<(), String> {
        match node_type {
            NodeType::Hardware => {
                self.log(format!("Hardware node: {}", label.unwrap_or("unknown")));
            }
            NodeType::Uart => {
                // If the label looks like data, send it
                if let Some(label) = label {
                    if !label.is_empty() {
                        self.execute_peripheral_action(&PeripheralAction::UartSend {
                            instance: 1,
                            data: label.as_bytes().to_vec(),
                        });
                    }
                }
            }
            NodeType::Timer => {
                self.log(format!("Timer node: {}", label.unwrap_or("unknown")));
            }
            NodeType::Sensor => {
                self.log(format!("Sensor node: {}", label.unwrap_or("unknown")));
            }
            NodeType::Interrupt => {
                self.log(format!("Interrupt: {}", label.unwrap_or("unknown")));
            }
            _ => {}
        }
        Ok(())
    }

    /// Execute entry action of current node
    fn execute_current_entry_action(&mut self) -> Result<(), String> {
        let entry_action = {
            let graph = self.graph.lock().map_err(|e| e.to_string())?;
            if let Some(id) = self.state.current_node {
                graph.get_node(id).and_then(|n| n.entry_action.clone())
            } else {
                None
            }
        };

        if let Some(action) = entry_action {
            self.execute_action(&action)?;
        }

        Ok(())
    }

    /// Evaluate edge guard condition
    fn evaluate_guard(&self, edge: &FSMEdge) -> bool {
        match &edge.guard {
            None => true, // No guard = always pass
            Some(guard) => {
                // Simple expression evaluation
                // TODO: Implement proper expression parser
                let guard = guard.trim().to_lowercase();
                
                // Check for GPIO conditions
                for (key, value) in &self.state.gpio_state {
                    if guard.contains(&key.to_lowercase()) {
                        if guard.contains("== true") || guard.contains("== high") || guard.contains("== 1") {
                            return *value;
                        }
                        if guard.contains("== false") || guard.contains("== low") || guard.contains("== 0") {
                            return !*value;
                        }
                    }
                }
                
                // Default: pass if guard is just "true" or empty-ish
                guard == "true" || guard.is_empty()
            }
        }
    }

    /// Add a log entry
    fn log(&mut self, message: String) {
        self.state.logs.push(message);
    }

    /// Get current state
    pub fn state(&self) -> &BehaviorState {
        &self.state
    }

    /// Get action history
    pub fn action_history(&self) -> &[ActionResult] {
        &self.action_history
    }

    /// Set ADC channel value (for input injection)
    pub fn set_adc_value(&mut self, channel: u8, value: u16) {
        self.state.adc_values.insert(channel, value);
    }

    /// Inject GPIO input
    pub fn inject_gpio(&mut self, port: char, pin: u8, state: bool) {
        let key = format!("{}{}", port, pin);
        self.state.gpio_state.insert(key, state);
        
        if let Some(ref mut bus) = self.peripheral_bus {
            bus.gpio_inject(port, pin, state);
        }
    }
}

/// Result of a simulation step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepResult {
    Transitioned { from: NodeId, to: NodeId, actions_executed: usize },
    Completed,
    Deadlock,
    Breakpoint(NodeId),
}

// Helper functions

fn extract_parens(s: &str) -> Option<String> {
    let start = s.find('(')?;
    let end = s.rfind(')')?;
    if end > start + 1 {
        Some(s[start + 1..end].to_string())
    } else {
        None
    }
}

fn extract_string_arg(s: &str) -> Option<String> {
    // Find content between quotes
    let start = s.find('"').or_else(|| s.find('\''))?;
    let end = s.rfind('"').or_else(|| s.rfind('\''))?;
    if end > start + 1 {
        Some(s[start + 1..end].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::graph::FSMGraph;
    use crate::core::types::FSMNode;

    #[test]
    fn test_parse_gpio_action() {
        let sim = FsmBehaviorSimulator::new(FSMGraph::new());
        
        let result = sim.parse_gpio_action("GPIO.A5 = HIGH");
        assert!(matches!(result, Some(PeripheralAction::GpioSet { port: 'A', pin: 5, state: true })));
        
        let result = sim.parse_gpio_action("PA0 = 1");
        assert!(matches!(result, Some(PeripheralAction::GpioSet { port: 'A', pin: 0, state: true })));
    }

    #[test]
    fn test_basic_simulation() {
        let mut graph = FSMGraph::new();
        
        let start = FSMNode::new("START", NodeType::Input)
            .with_entry_action("GPIO.A5 = HIGH");
        let end = FSMNode::new("END", NodeType::Output)
            .with_entry_action("GPIO.A5 = LOW");
        
        let start_id = graph.add_node(start);
        let end_id = graph.add_node(end);
        
        graph.add_edge(FSMEdge::new(start_id, end_id).with_label("GO"));
        
        let mut sim = FsmBehaviorSimulator::new(graph);
        
        assert!(sim.start().is_ok());
        assert_eq!(sim.state().gpio_state.get("A5"), Some(&true));
        
        let result = sim.step();
        assert!(result.is_ok());
        assert_eq!(sim.state().gpio_state.get("A5"), Some(&false));
    }
}
