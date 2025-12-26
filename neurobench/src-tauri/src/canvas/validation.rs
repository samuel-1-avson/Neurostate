//! Validation - Real-time design validation with error overlays
//!
//! Provides comprehensive validation for FSM and embedded designs.

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

/// Validation issue severity
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Validation issue category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueCategory {
    /// FSM structure issues
    FsmStructure,
    /// Graph connectivity issues
    Connectivity,
    /// Resource/hardware conflicts
    Resource,
    /// Naming/labeling issues
    Naming,
    /// Performance/optimization hints
    Performance,
    /// RTOS-specific issues
    Rtos,
    /// Dead code/unreachable states
    DeadCode,
    /// Timing/synchronization issues
    Timing,
}

/// A validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Unique issue ID
    pub id: String,
    /// Issue severity
    pub severity: IssueSeverity,
    /// Issue category
    pub category: IssueCategory,
    /// Related node IDs
    pub node_ids: Vec<String>,
    /// Related edge IDs
    pub edge_ids: Vec<String>,
    /// Short message
    pub message: String,
    /// Detailed description
    pub description: Option<String>,
    /// Suggested fix
    pub suggestion: Option<String>,
    /// Auto-fixable
    pub fixable: bool,
}

/// Validation result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationResult {
    /// All issues found
    pub issues: Vec<ValidationIssue>,
    /// Is design valid (no errors)
    pub valid: bool,
    /// Error count
    pub error_count: usize,
    /// Warning count
    pub warning_count: usize,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_issue(&mut self, issue: ValidationIssue) {
        match issue.severity {
            IssueSeverity::Error => {
                self.error_count += 1;
                self.valid = false;
            }
            IssueSeverity::Warning => self.warning_count += 1,
            _ => {}
        }
        self.issues.push(issue);
    }

    pub fn merge(&mut self, other: ValidationResult) {
        for issue in other.issues {
            self.add_issue(issue);
        }
    }

    pub fn errors(&self) -> Vec<&ValidationIssue> {
        self.issues.iter()
            .filter(|i| i.severity == IssueSeverity::Error)
            .collect()
    }

    pub fn warnings(&self) -> Vec<&ValidationIssue> {
        self.issues.iter()
            .filter(|i| i.severity == IssueSeverity::Warning)
            .collect()
    }

    pub fn for_node(&self, node_id: &str) -> Vec<&ValidationIssue> {
        self.issues.iter()
            .filter(|i| i.node_ids.contains(&node_id.to_string()))
            .collect()
    }

    pub fn for_edge(&self, edge_id: &str) -> Vec<&ValidationIssue> {
        self.issues.iter()
            .filter(|i| i.edge_ids.contains(&edge_id.to_string()))
            .collect()
    }
}

/// FSM-specific validator
pub struct FsmValidator;

impl FsmValidator {
    /// Validate FSM structure
    pub fn validate(
        nodes: &HashMap<String, super::types::CanvasNode>,
        edges: &HashMap<String, super::types::CanvasEdge>,
    ) -> ValidationResult {
        let mut result = ValidationResult::new();
        result.valid = true;

        // Check for initial state
        let initial_count = nodes.values()
            .filter(|n| n.node_type == super::types::NodeType::Initial)
            .count();

        if initial_count == 0 {
            result.add_issue(ValidationIssue {
                id: "fsm-no-initial".into(),
                severity: IssueSeverity::Error,
                category: IssueCategory::FsmStructure,
                node_ids: vec![],
                edge_ids: vec![],
                message: "No initial state defined".into(),
                description: Some("FSM must have exactly one initial state".into()),
                suggestion: Some("Add an Initial node to the design".into()),
                fixable: false,
            });
        } else if initial_count > 1 {
            let initial_ids: Vec<_> = nodes.iter()
                .filter(|(_, n)| n.node_type == super::types::NodeType::Initial)
                .map(|(id, _)| id.clone())
                .collect();
            
            result.add_issue(ValidationIssue {
                id: "fsm-multiple-initial".into(),
                severity: IssueSeverity::Error,
                category: IssueCategory::FsmStructure,
                node_ids: initial_ids,
                edge_ids: vec![],
                message: format!("Multiple initial states ({})", initial_count),
                description: Some("FSM must have exactly one initial state".into()),
                suggestion: Some("Remove extra Initial nodes".into()),
                fixable: false,
            });
        }

        // Check for orphan nodes (no connections)
        let connected_nodes: HashSet<_> = edges.values()
            .flat_map(|e| vec![e.source.clone(), e.target.clone()])
            .collect();

        for (id, node) in nodes {
            if !connected_nodes.contains(id) && node.node_type != super::types::NodeType::Initial {
                result.add_issue(ValidationIssue {
                    id: format!("orphan-{}", id),
                    severity: IssueSeverity::Warning,
                    category: IssueCategory::Connectivity,
                    node_ids: vec![id.clone()],
                    edge_ids: vec![],
                    message: format!("Node '{}' has no connections", node.label),
                    description: None,
                    suggestion: Some("Connect this node or remove it".into()),
                    fixable: false,
                });
            }
        }

        // Check for unreachable states (no incoming edges except initial)
        let targets: HashSet<_> = edges.values().map(|e| e.target.clone()).collect();
        
        for (id, node) in nodes {
            if !targets.contains(id) 
               && node.node_type != super::types::NodeType::Initial
               && connected_nodes.contains(id) {
                result.add_issue(ValidationIssue {
                    id: format!("unreachable-{}", id),
                    severity: IssueSeverity::Warning,
                    category: IssueCategory::DeadCode,
                    node_ids: vec![id.clone()],
                    edge_ids: vec![],
                    message: format!("State '{}' is unreachable", node.label),
                    description: Some("No transitions lead to this state".into()),
                    suggestion: None,
                    fixable: false,
                });
            }
        }

        // Check for terminal states (no outgoing except Final)
        let sources: HashSet<_> = edges.values().map(|e| e.source.clone()).collect();
        
        for (id, node) in nodes {
            if !sources.contains(id) 
               && node.node_type != super::types::NodeType::Final
               && connected_nodes.contains(id) {
                result.add_issue(ValidationIssue {
                    id: format!("terminal-{}", id),
                    severity: IssueSeverity::Info,
                    category: IssueCategory::FsmStructure,
                    node_ids: vec![id.clone()],
                    edge_ids: vec![],
                    message: format!("State '{}' is terminal (no outgoing)", node.label),
                    description: None,
                    suggestion: None,
                    fixable: false,
                });
            }
        }

        // Check for empty labels
        for (id, node) in nodes {
            if node.label.trim().is_empty() {
                result.add_issue(ValidationIssue {
                    id: format!("empty-label-{}", id),
                    severity: IssueSeverity::Warning,
                    category: IssueCategory::Naming,
                    node_ids: vec![id.clone()],
                    edge_ids: vec![],
                    message: "Node has empty label".into(),
                    description: None,
                    suggestion: Some("Add a descriptive label".into()),
                    fixable: false,
                });
            }
        }

        result
    }
}

/// Design complexity metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    /// Number of nodes
    pub node_count: usize,
    /// Number of edges
    pub edge_count: usize,
    /// Cyclomatic complexity (edges - nodes + 2*connected_components)
    pub cyclomatic_complexity: usize,
    /// Maximum depth from initial state
    pub max_depth: usize,
    /// Average transitions per state
    pub avg_transitions: f64,
    /// Number of decision points
    pub decision_points: usize,
    /// Number of parallel branches (forks)
    pub parallel_branches: usize,
}

impl ComplexityMetrics {
    /// Calculate complexity from nodes and edges
    pub fn calculate(
        nodes: &HashMap<String, super::types::CanvasNode>,
        edges: &HashMap<String, super::types::CanvasEdge>,
    ) -> Self {
        let node_count = nodes.len();
        let edge_count = edges.len();
        
        // Count decision points and forks
        let mut decision_points = 0;
        let mut parallel_branches = 0;
        
        for node in nodes.values() {
            match node.node_type {
                super::types::NodeType::Choice | super::types::NodeType::Decision => {
                    decision_points += 1;
                }
                super::types::NodeType::Fork => {
                    parallel_branches += 1;
                }
                _ => {}
            }
        }

        // Calculate outgoing edge counts
        let mut outgoing: HashMap<String, usize> = HashMap::new();
        for edge in edges.values() {
            *outgoing.entry(edge.source.clone()).or_default() += 1;
        }

        let avg_transitions = if node_count > 0 {
            edge_count as f64 / node_count as f64
        } else {
            0.0
        };

        // Cyclomatic complexity: E - N + 2P (simplified: assume 1 component)
        let cyclomatic = edge_count.saturating_sub(node_count) + 2;

        Self {
            node_count,
            edge_count,
            cyclomatic_complexity: cyclomatic,
            max_depth: 0, // Would require BFS from initial
            avg_transitions,
            decision_points,
            parallel_branches,
        }
    }

    /// Get complexity rating
    pub fn rating(&self) -> &'static str {
        match self.cyclomatic_complexity {
            0..=5 => "Simple",
            6..=10 => "Moderate",
            11..=20 => "Complex",
            21..=50 => "High",
            _ => "Very High",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::new();
        assert!(result.valid);
        
        result.add_issue(ValidationIssue {
            id: "test".into(),
            severity: IssueSeverity::Error,
            category: IssueCategory::FsmStructure,
            node_ids: vec![],
            edge_ids: vec![],
            message: "Test error".into(),
            description: None,
            suggestion: None,
            fixable: false,
        });
        
        assert!(!result.valid);
        assert_eq!(result.error_count, 1);
    }
}
