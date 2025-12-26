//! FSM - Extended finite state machine support
//!
//! Hierarchical states, parallel regions, history states, and simulation.

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Hierarchical state (compound state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalState {
    /// Parent state node ID
    pub id: String,
    /// Child state node IDs
    pub children: HashSet<String>,
    /// Initial child state ID
    pub initial_child: Option<String>,
    /// History type
    pub history: HistoryType,
    /// Last active child (for history)
    pub last_active: Option<String>,
}

/// History state type
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HistoryType {
    #[default]
    None,
    /// Shallow history (remembers direct child)
    Shallow,
    /// Deep history (remembers nested states)
    Deep,
}

/// Parallel region (orthogonal state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelRegion {
    /// Region ID
    pub id: String,
    /// Fork node ID (entry point)
    pub fork_id: String,
    /// Join node ID (exit point)
    pub join_id: Option<String>,
    /// States in each parallel branch
    pub branches: Vec<Vec<String>>,
}

/// FSM execution state for simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsmExecutionState {
    /// Current active state(s)
    pub active_states: HashSet<String>,
    /// State history stack
    pub state_history: VecDeque<String>,
    /// Variable context
    pub variables: HashMap<String, String>,
    /// Pending events
    pub event_queue: VecDeque<String>,
    /// Execution step counter
    pub step_count: usize,
}

impl Default for FsmExecutionState {
    fn default() -> Self {
        Self {
            active_states: HashSet::new(),
            state_history: VecDeque::new(),
            variables: HashMap::new(),
            event_queue: VecDeque::new(),
            step_count: 0,
        }
    }
}

impl FsmExecutionState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enter a state
    pub fn enter_state(&mut self, state_id: impl Into<String>) {
        let id = state_id.into();
        self.state_history.push_front(id.clone());
        self.active_states.insert(id);
        self.step_count += 1;
    }

    /// Exit a state  
    pub fn exit_state(&mut self, state_id: &str) {
        self.active_states.remove(state_id);
    }

    /// Queue an event
    pub fn queue_event(&mut self, event: impl Into<String>) {
        self.event_queue.push_back(event.into());
    }

    /// Pop next event
    pub fn pop_event(&mut self) -> Option<String> {
        self.event_queue.pop_front()
    }

    /// Check if state is active
    pub fn is_active(&self, state_id: &str) -> bool {
        self.active_states.contains(state_id)
    }

    /// Get current state(s)
    pub fn current_states(&self) -> Vec<&str> {
        self.active_states.iter().map(|s| s.as_str()).collect()
    }

    /// Reset to initial
    pub fn reset(&mut self) {
        self.active_states.clear();
        self.state_history.clear();
        self.event_queue.clear();
        self.step_count = 0;
    }
}

/// FSM simulator
#[derive(Debug, Clone, Default)]
pub struct FsmSimulator {
    /// Hierarchical states
    hierarchical: HashMap<String, HierarchicalState>,
    /// Parallel regions
    parallel: HashMap<String, ParallelRegion>,
    /// Execution state
    pub state: FsmExecutionState,
    /// Transition callbacks (for UI)
    trace: Vec<TransitionTrace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionTrace {
    pub step: usize,
    pub from_state: String,
    pub to_state: String,
    pub event: Option<String>,
    pub guard: Option<String>,
}

impl FsmSimulator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add hierarchical state
    pub fn add_hierarchical(&mut self, state: HierarchicalState) {
        self.hierarchical.insert(state.id.clone(), state);
    }

    /// Add parallel region
    pub fn add_parallel(&mut self, region: ParallelRegion) {
        self.parallel.insert(region.id.clone(), region);
    }

    /// Initialize simulation
    pub fn initialize(&mut self, initial_state: impl Into<String>) {
        self.state.reset();
        self.trace.clear();
        self.state.enter_state(initial_state);
    }

    /// Step simulation (process one event/transition)
    pub fn step(&mut self) -> Option<&TransitionTrace> {
        // Pop event and process transition
        if let Some(_event) = self.state.pop_event() {
            // In real impl, would evaluate guards and execute transitions
            // This is a placeholder for the simulation engine
        }
        self.trace.last()
    }

    /// Record a transition
    pub fn record_transition(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        event: Option<String>,
    ) {
        let from = from.into();
        let to = to.into();
        
        self.state.exit_state(&from);
        self.state.enter_state(to.clone());
        
        self.trace.push(TransitionTrace {
            step: self.state.step_count,
            from_state: from,
            to_state: to,
            event,
            guard: None,
        });
    }

    /// Get execution trace
    pub fn get_trace(&self) -> &[TransitionTrace] {
        &self.trace
    }

    /// Check if in compound state
    pub fn is_in_hierarchical(&self, parent_id: &str) -> bool {
        if let Some(hs) = self.hierarchical.get(parent_id) {
            self.state.active_states.iter()
                .any(|s| hs.children.contains(s))
        } else {
            false
        }
    }
}

/// Smart placement for new nodes
#[derive(Debug, Clone, Default)]
pub struct SmartPlacement {
    /// Grid size for snapping
    pub grid_size: f64,
    /// Minimum spacing between nodes
    pub min_spacing: f64,
    /// Preferred placement direction
    pub direction: PlacementDirection,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum PlacementDirection {
    #[default]
    Right,
    Down,
    Left,
    Up,
}

impl SmartPlacement {
    pub fn new(grid_size: f64, min_spacing: f64) -> Self {
        Self {
            grid_size,
            min_spacing,
            direction: PlacementDirection::Right,
        }
    }

    /// Snap position to grid
    pub fn snap_to_grid(&self, x: f64, y: f64) -> (f64, f64) {
        if self.grid_size > 0.0 {
            (
                (x / self.grid_size).round() * self.grid_size,
                (y / self.grid_size).round() * self.grid_size,
            )
        } else {
            (x, y)
        }
    }

    /// Find best position for new node relative to reference
    pub fn suggest_position(
        &self,
        reference: Option<(&f64, &f64, &f64, &f64)>, // x, y, w, h
        occupied: &[(f64, f64, f64, f64)],
    ) -> (f64, f64) {
        let (base_x, base_y) = if let Some((rx, ry, rw, rh)) = reference {
            match self.direction {
                PlacementDirection::Right => (*rx + *rw + self.min_spacing, *ry),
                PlacementDirection::Down => (*rx, *ry + *rh + self.min_spacing),
                PlacementDirection::Left => (*rx - 200.0 - self.min_spacing, *ry),
                PlacementDirection::Up => (*rx, *ry - 100.0 - self.min_spacing),
            }
        } else {
            (100.0, 100.0)
        };

        let (snapped_x, snapped_y) = self.snap_to_grid(base_x, base_y);

        // Check for collisions and adjust
        self.avoid_collisions(snapped_x, snapped_y, 200.0, 100.0, occupied)
    }

    fn avoid_collisions(
        &self,
        mut x: f64,
        mut y: f64,
        w: f64,
        h: f64,
        occupied: &[(f64, f64, f64, f64)],
    ) -> (f64, f64) {
        let mut attempts = 0;
        while attempts < 20 {
            let mut collision = false;
            for (ox, oy, ow, oh) in occupied {
                if x < ox + ow + self.min_spacing
                    && x + w + self.min_spacing > *ox
                    && y < oy + oh + self.min_spacing
                    && y + h + self.min_spacing > *oy
                {
                    collision = true;
                    x += w + self.min_spacing;
                    break;
                }
            }
            if !collision {
                break;
            }
            attempts += 1;
        }
        self.snap_to_grid(x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_state() {
        let mut state = FsmExecutionState::new();
        state.enter_state("s1");
        assert!(state.is_active("s1"));
        
        state.exit_state("s1");
        assert!(!state.is_active("s1"));
    }

    #[test]
    fn test_smart_placement() {
        let placement = SmartPlacement::new(20.0, 10.0);
        let (x, y) = placement.snap_to_grid(105.0, 48.0);
        assert_eq!(x, 100.0);
        assert_eq!(y, 40.0);
    }
}
