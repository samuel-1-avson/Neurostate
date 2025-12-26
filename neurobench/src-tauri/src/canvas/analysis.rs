//! Analysis - Coverage, deadlock detection, and design analysis
//!
//! Advanced analysis for FSM and RTOS designs.

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Path coverage analysis for FSM testing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoverageAnalysis {
    /// All possible paths (limited to avoid explosion)
    pub paths: Vec<Vec<String>>,
    /// All states
    pub all_states: HashSet<String>,
    /// Covered states
    pub covered_states: HashSet<String>,
    /// All transitions (edge IDs)
    pub all_transitions: HashSet<String>,
    /// Covered transitions
    pub covered_transitions: HashSet<String>,
    /// State coverage percentage
    pub state_coverage: f64,
    /// Transition coverage percentage
    pub transition_coverage: f64,
}

impl CoverageAnalysis {
    /// Generate all paths from initial state (bounded)
    pub fn analyze(
        nodes: &HashMap<String, super::types::CanvasNode>,
        edges: &HashMap<String, super::types::CanvasEdge>,
        max_paths: usize,
        max_depth: usize,
    ) -> Self {
        let mut analysis = Self::default();
        
        // Find initial state
        let initial = nodes.iter()
            .find(|(_, n)| n.node_type == super::types::NodeType::Initial)
            .map(|(id, _)| id.clone());
        
        // Collect all states and transitions
        for id in nodes.keys() {
            analysis.all_states.insert(id.clone());
        }
        for id in edges.keys() {
            analysis.all_transitions.insert(id.clone());
        }

        // Build adjacency map
        let mut adj: HashMap<String, Vec<(String, String)>> = HashMap::new();
        for (edge_id, edge) in edges {
            adj.entry(edge.source.clone())
                .or_default()
                .push((edge.target.clone(), edge_id.clone()));
        }

        // DFS to find paths
        if let Some(start) = initial {
            let mut stack: Vec<(String, Vec<String>, HashSet<String>)> = vec![];
            stack.push((start.clone(), vec![start.clone()], HashSet::new()));

            while let Some((current, path, visited_edges)) = stack.pop() {
                if analysis.paths.len() >= max_paths {
                    break;
                }

                if path.len() > max_depth {
                    analysis.paths.push(path);
                    continue;
                }

                let neighbors = adj.get(&current).cloned().unwrap_or_default();
                
                if neighbors.is_empty() {
                    // Terminal state - save path
                    analysis.paths.push(path);
                } else {
                    let mut added = false;
                    for (next, edge_id) in neighbors {
                        if !visited_edges.contains(&edge_id) {
                            let mut new_path = path.clone();
                            new_path.push(next.clone());
                            let mut new_visited = visited_edges.clone();
                            new_visited.insert(edge_id);
                            stack.push((next, new_path, new_visited));
                            added = true;
                        }
                    }
                    if !added {
                        analysis.paths.push(path);
                    }
                }
            }
        }

        // Calculate coverage
        analysis.state_coverage = if analysis.all_states.is_empty() {
            100.0
        } else {
            (analysis.covered_states.len() as f64 / analysis.all_states.len() as f64) * 100.0
        };

        analysis.transition_coverage = if analysis.all_transitions.is_empty() {
            100.0
        } else {
            (analysis.covered_transitions.len() as f64 / analysis.all_transitions.len() as f64) * 100.0
        };

        analysis
    }

    /// Mark states/transitions as covered (for testing)
    pub fn mark_covered(&mut self, state_ids: &[String], transition_ids: &[String]) {
        for id in state_ids {
            self.covered_states.insert(id.clone());
        }
        for id in transition_ids {
            self.covered_transitions.insert(id.clone());
        }
        
        // Recalculate coverage
        self.state_coverage = if self.all_states.is_empty() {
            100.0
        } else {
            (self.covered_states.len() as f64 / self.all_states.len() as f64) * 100.0
        };
        self.transition_coverage = if self.all_transitions.is_empty() {
            100.0
        } else {
            (self.covered_transitions.len() as f64 / self.all_transitions.len() as f64) * 100.0
        };
    }
}

/// Deadlock detection for RTOS designs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeadlockAnalysis {
    /// Potential deadlock cycles
    pub cycles: Vec<DeadlockCycle>,
    /// Resource wait graph edges
    pub wait_graph: Vec<(String, String)>,
    /// Is design deadlock-free
    pub deadlock_free: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadlockCycle {
    /// Task IDs in the cycle
    pub tasks: Vec<String>,
    /// Resources involved
    pub resources: Vec<String>,
    /// Description
    pub description: String,
}

impl DeadlockAnalysis {
    /// Analyze for potential deadlocks using resource allocation graph
    pub fn analyze(
        nodes: &HashMap<String, super::types::CanvasNode>,
        edges: &HashMap<String, super::types::CanvasEdge>,
    ) -> Self {
        let mut analysis = Self::default();
        analysis.deadlock_free = true;

        // Find RTOS tasks and synchronization primitives
        let tasks: Vec<_> = nodes.iter()
            .filter(|(_, n)| n.node_type == super::types::NodeType::Task)
            .map(|(id, _)| id.clone())
            .collect();

        let mutexes: HashSet<_> = nodes.iter()
            .filter(|(_, n)| n.node_type == super::types::NodeType::Mutex)
            .map(|(id, _)| id.clone())
            .collect();

        let semaphores: HashSet<_> = nodes.iter()
            .filter(|(_, n)| n.node_type == super::types::NodeType::Semaphore)
            .map(|(id, _)| id.clone())
            .collect();

        // Build resource wait graph
        // Edge from resource -> task means task holds resource
        // Edge from task -> resource means task waits for resource
        for edge in edges.values() {
            let is_sync = mutexes.contains(&edge.source) 
                || mutexes.contains(&edge.target)
                || semaphores.contains(&edge.source)
                || semaphores.contains(&edge.target);
            
            if is_sync {
                analysis.wait_graph.push((edge.source.clone(), edge.target.clone()));
            }
        }

        // Detect cycles in wait graph using DFS
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for task in &tasks {
            if !visited.contains(task) {
                if Self::dfs_cycle(
                    task,
                    &analysis.wait_graph,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut analysis.cycles,
                ) {
                    analysis.deadlock_free = false;
                }
            }
        }

        analysis
    }

    fn dfs_cycle(
        node: &str,
        graph: &[(String, String)],
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<DeadlockCycle>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        let neighbors: Vec<String> = graph.iter()
            .filter(|(src, _)| src == node)
            .map(|(_, dst)| dst.clone())
            .collect();

        for neighbor in neighbors {
            if !visited.contains(&neighbor) {
                if Self::dfs_cycle(&neighbor, graph, visited, rec_stack, path, cycles) {
                    return true;
                }
            } else if rec_stack.contains(&neighbor) {
                // Found cycle
                let start_idx = path.iter().position(|n| n == &neighbor).unwrap();
                let cycle_path = path[start_idx..].to_vec();
                
                cycles.push(DeadlockCycle {
                    tasks: cycle_path.clone(),
                    resources: vec![],
                    description: format!(
                        "Potential deadlock: {} -> {}",
                        cycle_path.join(" -> "),
                        neighbor
                    ),
                });
                return true;
            }
        }

        path.pop();
        rec_stack.remove(node);
        false
    }
}

/// Design statistics summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DesignStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub states: usize,
    pub transitions: usize,
    pub peripherals: usize,
    pub rtos_components: usize,
    pub groups: usize,
    pub layers: usize,
}

impl DesignStats {
    pub fn calculate(nodes: &HashMap<String, super::types::CanvasNode>) -> Self {
        let mut stats = Self::default();
        stats.total_nodes = nodes.len();
        
        for node in nodes.values() {
            if node.node_type.is_fsm() {
                stats.states += 1;
            }
            if node.node_type.is_peripheral() {
                stats.peripherals += 1;
            }
            if node.node_type.is_rtos() {
                stats.rtos_components += 1;
            }
        }
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_analysis() {
        let mut analysis = CoverageAnalysis::default();
        analysis.all_states.insert("s1".into());
        analysis.all_states.insert("s2".into());
        
        analysis.mark_covered(&["s1".into()], &[]);
        
        assert_eq!(analysis.state_coverage, 50.0);
    }
}
