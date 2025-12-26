//! Signal Flow - Data flow visualization and analysis
//!
//! Tracks signal paths through the design for debugging and verification.

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Signal type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SignalType {
    /// Digital high/low
    Digital,
    /// Analog voltage
    Analog,
    /// Serial data stream
    Serial,
    /// Clock signal
    Clock,
    /// Interrupt/trigger
    Event,
    /// Data bus
    Bus,
    /// Power rail
    Power,
}

/// A signal in the design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    /// Unique signal ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Signal type
    pub signal_type: SignalType,
    /// Source node ID
    pub source_node: String,
    /// Source port ID
    pub source_port: Option<String>,
    /// Target node IDs (can fan out)
    pub targets: Vec<SignalTarget>,
    /// Current value (for simulation/debug)
    pub current_value: Option<String>,
    /// Color for visualization
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalTarget {
    pub node_id: String,
    pub port_id: Option<String>,
    pub edge_id: String,
}

/// Signal flow analyzer
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignalFlowAnalyzer {
    /// All tracked signals
    signals: HashMap<String, Signal>,
    /// Node to input signals map
    node_inputs: HashMap<String, HashSet<String>>,
    /// Node to output signals map  
    node_outputs: HashMap<String, HashSet<String>>,
}

impl SignalFlowAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a signal
    pub fn add_signal(&mut self, signal: Signal) {
        let id = signal.id.clone();
        
        // Track outputs
        self.node_outputs
            .entry(signal.source_node.clone())
            .or_default()
            .insert(id.clone());
        
        // Track inputs
        for target in &signal.targets {
            self.node_inputs
                .entry(target.node_id.clone())
                .or_default()
                .insert(id.clone());
        }
        
        self.signals.insert(id, signal);
    }

    /// Remove a signal
    pub fn remove_signal(&mut self, signal_id: &str) {
        if let Some(signal) = self.signals.remove(signal_id) {
            if let Some(outputs) = self.node_outputs.get_mut(&signal.source_node) {
                outputs.remove(signal_id);
            }
            for target in &signal.targets {
                if let Some(inputs) = self.node_inputs.get_mut(&target.node_id) {
                    inputs.remove(signal_id);
                }
            }
        }
    }

    /// Get signals entering a node
    pub fn get_inputs(&self, node_id: &str) -> Vec<&Signal> {
        self.node_inputs
            .get(node_id)
            .map(|ids| ids.iter().filter_map(|id| self.signals.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get signals leaving a node
    pub fn get_outputs(&self, node_id: &str) -> Vec<&Signal> {
        self.node_outputs
            .get(node_id)
            .map(|ids| ids.iter().filter_map(|id| self.signals.get(id)).collect())
            .unwrap_or_default()
    }

    /// Trace signal path from source to all destinations
    pub fn trace_forward(&self, start_node: &str) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut path = Vec::new();
        
        queue.push_back(start_node.to_string());
        
        while let Some(node_id) = queue.pop_front() {
            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id.clone());
            path.push(node_id.clone());
            
            // Find all downstream nodes
            if let Some(output_ids) = self.node_outputs.get(&node_id) {
                for signal_id in output_ids {
                    if let Some(signal) = self.signals.get(signal_id) {
                        for target in &signal.targets {
                            if !visited.contains(&target.node_id) {
                                queue.push_back(target.node_id.clone());
                            }
                        }
                    }
                }
            }
        }
        
        path
    }

    /// Trace signal path backwards from destination
    pub fn trace_backward(&self, end_node: &str) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut path = Vec::new();
        
        queue.push_back(end_node.to_string());
        
        while let Some(node_id) = queue.pop_front() {
            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id.clone());
            path.push(node_id.clone());
            
            // Find all upstream nodes
            if let Some(input_ids) = self.node_inputs.get(&node_id) {
                for signal_id in input_ids {
                    if let Some(signal) = self.signals.get(signal_id) {
                        if !visited.contains(&signal.source_node) {
                            queue.push_back(signal.source_node.clone());
                        }
                    }
                }
            }
        }
        
        path
    }

    /// Find all signals between two nodes
    pub fn find_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parents: HashMap<String, String> = HashMap::new();
        
        queue.push_back(from.to_string());
        visited.insert(from.to_string());
        
        while let Some(node_id) = queue.pop_front() {
            if node_id == to {
                // Reconstruct path
                let mut path = vec![to.to_string()];
                let mut current = to.to_string();
                while let Some(parent) = parents.get(&current) {
                    path.push(parent.clone());
                    current = parent.clone();
                }
                path.reverse();
                return Some(path);
            }
            
            if let Some(output_ids) = self.node_outputs.get(&node_id) {
                for signal_id in output_ids {
                    if let Some(signal) = self.signals.get(signal_id) {
                        for target in &signal.targets {
                            if !visited.contains(&target.node_id) {
                                visited.insert(target.node_id.clone());
                                parents.insert(target.node_id.clone(), node_id.clone());
                                queue.push_back(target.node_id.clone());
                            }
                        }
                    }
                }
            }
        }
        
        None
    }

    /// Detect signal loops (cyclic paths)
    pub fn find_loops(&self) -> Vec<Vec<String>> {
        let mut loops = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        for node_id in self.node_outputs.keys() {
            if !visited.contains(node_id) {
                let mut path = Vec::new();
                self.dfs_find_loops(node_id, &mut visited, &mut rec_stack, &mut path, &mut loops);
            }
        }
        
        loops
    }

    fn dfs_find_loops(
        &self,
        node_id: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        loops: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node_id.to_string());
        rec_stack.insert(node_id.to_string());
        path.push(node_id.to_string());
        
        if let Some(output_ids) = self.node_outputs.get(node_id) {
            for signal_id in output_ids {
                if let Some(signal) = self.signals.get(signal_id) {
                    for target in &signal.targets {
                        if !visited.contains(&target.node_id) {
                            self.dfs_find_loops(&target.node_id, visited, rec_stack, path, loops);
                        } else if rec_stack.contains(&target.node_id) {
                            // Found a cycle
                            let start_idx = path.iter().position(|n| n == &target.node_id).unwrap();
                            loops.push(path[start_idx..].to_vec());
                        }
                    }
                }
            }
        }
        
        path.pop();
        rec_stack.remove(node_id);
    }

    /// Get all signals
    pub fn all_signals(&self) -> Vec<&Signal> {
        self.signals.values().collect()
    }

    /// Clear all signals
    pub fn clear(&mut self) {
        self.signals.clear();
        self.node_inputs.clear();
        self.node_outputs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_tracing() {
        let mut analyzer = SignalFlowAnalyzer::new();
        
        analyzer.add_signal(Signal {
            id: "s1".into(),
            name: "Data".into(),
            signal_type: SignalType::Digital,
            source_node: "n1".into(),
            source_port: None,
            targets: vec![SignalTarget {
                node_id: "n2".into(),
                port_id: None,
                edge_id: "e1".into(),
            }],
            current_value: None,
            color: None,
        });
        
        let path = analyzer.trace_forward("n1");
        assert_eq!(path, vec!["n1", "n2"]);
    }
}
