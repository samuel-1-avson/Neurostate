//! Hierarchical Planner - Recursive Task Decomposition
//!
//! Provides advanced task planning capabilities for the multi-agent system:
//! - Recursive task decomposition until atomic tasks
//! - Dependency graph management (DAG)
//! - Automatic parallelization detection
//! - Execution with dependency ordering

use crate::ai::providers::{AIModel, ChatMessage, Role};
use crate::agents::director::{Task, TaskStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

// =============================================================================
// TYPES
// =============================================================================

/// Unique identifier for a task node
pub type NodeId = String;

/// A node in the task tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNode {
    /// Unique identifier
    pub id: NodeId,
    /// Task description
    pub description: String,
    /// Agent responsible for this task
    pub assigned_agent: Option<String>,
    /// Whether this task can be further decomposed
    pub is_atomic: bool,
    /// Child tasks (subtasks)
    pub children: Vec<NodeId>,
    /// Dependencies (must complete before this task)
    pub dependencies: Vec<NodeId>,
    /// Execution status
    pub status: TaskNodeStatus,
    /// Estimated complexity (1-10)
    pub complexity: u8,
    /// Result after execution
    pub result: Option<TaskResult>,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

impl TaskNode {
    pub fn new(id: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            assigned_agent: None,
            is_atomic: false,
            children: Vec::new(),
            dependencies: Vec::new(),
            status: TaskNodeStatus::Pending,
            complexity: 5,
            result: None,
            metadata: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    pub fn atomic(id: &str, description: &str, agent: &str) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            assigned_agent: Some(agent.to_string()),
            is_atomic: true,
            children: Vec::new(),
            dependencies: Vec::new(),
            status: TaskNodeStatus::Pending,
            complexity: 3,
            result: None,
            metadata: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    pub fn with_complexity(mut self, complexity: u8) -> Self {
        self.complexity = complexity.min(10);
        self
    }

    pub fn with_agent(mut self, agent: &str) -> Self {
        self.assigned_agent = Some(agent.to_string());
        self
    }

    pub fn with_dependency(mut self, dep_id: &str) -> Self {
        self.dependencies.push(dep_id.to_string());
        self
    }
}

/// Status of a task node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskNodeStatus {
    /// Not yet started
    Pending,
    /// Waiting for dependencies
    Blocked,
    /// Currently executing
    InProgress,
    /// Successfully completed
    Completed,
    /// Failed with error
    Failed,
    /// Skipped (e.g., optional and not needed)
    Skipped,
}

/// Result of task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Success or failure
    pub success: bool,
    /// Output/response
    pub output: String,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Tokens used (if AI task)
    pub tokens_used: Option<u32>,
}

/// A complete task tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTree {
    /// Root node ID
    pub root_id: NodeId,
    /// All nodes indexed by ID
    pub nodes: HashMap<NodeId, TaskNode>,
    /// Overall goal description
    pub goal: String,
    /// Tree creation time
    pub created_at: DateTime<Utc>,
    /// Execution start time
    pub started_at: Option<DateTime<Utc>>,
    /// Execution end time
    pub completed_at: Option<DateTime<Utc>>,
}

impl TaskTree {
    pub fn new(goal: &str, root: TaskNode) -> Self {
        let root_id = root.id.clone();
        let mut nodes = HashMap::new();
        nodes.insert(root_id.clone(), root);

        Self {
            root_id,
            nodes,
            goal: goal.to_string(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
        }
    }

    /// Add a node to the tree
    pub fn add_node(&mut self, node: TaskNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Add a child to a parent node
    pub fn add_child(&mut self, parent_id: &str, child: TaskNode) {
        let child_id = child.id.clone();
        self.nodes.insert(child_id.clone(), child);
        
        if let Some(parent) = self.nodes.get_mut(parent_id) {
            parent.children.push(child_id);
        }
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &str) -> Option<&TaskNode> {
        self.nodes.get(id)
    }

    /// Get a mutable node by ID
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut TaskNode> {
        self.nodes.get_mut(id)
    }

    /// Get all leaf nodes (atomic tasks)
    pub fn get_leaves(&self) -> Vec<&TaskNode> {
        self.nodes.values()
            .filter(|n| n.children.is_empty())
            .collect()
    }

    /// Get nodes ready for execution (dependencies satisfied)
    pub fn get_ready_nodes(&self) -> Vec<&TaskNode> {
        self.nodes.values()
            .filter(|node| {
                node.status == TaskNodeStatus::Pending &&
                node.is_atomic &&
                node.dependencies.iter().all(|dep_id| {
                    self.nodes.get(dep_id)
                        .map(|dep| dep.status == TaskNodeStatus::Completed)
                        .unwrap_or(true)
                })
            })
            .collect()
    }

    /// Get nodes that can run in parallel
    pub fn get_parallel_batch(&self) -> Vec<&TaskNode> {
        let ready = self.get_ready_nodes();
        
        // Filter to nodes that don't depend on each other
        let ready_ids: HashSet<&str> = ready.iter().map(|n| n.id.as_str()).collect();
        
        ready.into_iter()
            .filter(|node| {
                !node.dependencies.iter().any(|dep| ready_ids.contains(dep.as_str()))
            })
            .collect()
    }

    /// Update node status
    pub fn update_status(&mut self, id: &str, status: TaskNodeStatus) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.status = status;
        }
        
        // Update blocked nodes
        self.update_blocked_status();
    }

    /// Set node result
    pub fn set_result(&mut self, id: &str, result: TaskResult) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.status = if result.success {
                TaskNodeStatus::Completed
            } else {
                TaskNodeStatus::Failed
            };
            node.result = Some(result);
        }
        
        self.update_blocked_status();
    }

    /// Update blocked status for nodes waiting on dependencies
    fn update_blocked_status(&mut self) {
        let node_ids: Vec<NodeId> = self.nodes.keys().cloned().collect();
        
        for id in node_ids {
            let should_block = {
                let node = self.nodes.get(&id).unwrap();
                node.status == TaskNodeStatus::Pending &&
                !node.dependencies.is_empty() &&
                node.dependencies.iter().any(|dep_id| {
                    self.nodes.get(dep_id)
                        .map(|dep| dep.status != TaskNodeStatus::Completed)
                        .unwrap_or(false)
                })
            };
            
            if should_block {
                if let Some(node) = self.nodes.get_mut(&id) {
                    node.status = TaskNodeStatus::Blocked;
                }
            }
        }
    }

    /// Check if the tree is fully executed
    pub fn is_complete(&self) -> bool {
        self.nodes.values()
            .filter(|n| n.is_atomic)
            .all(|n| matches!(n.status, TaskNodeStatus::Completed | TaskNodeStatus::Failed | TaskNodeStatus::Skipped))
    }

    /// Get execution progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        let atomic_nodes: Vec<_> = self.nodes.values().filter(|n| n.is_atomic).collect();
        if atomic_nodes.is_empty() {
            return 1.0;
        }

        let completed = atomic_nodes.iter()
            .filter(|n| matches!(n.status, TaskNodeStatus::Completed | TaskNodeStatus::Skipped))
            .count();

        completed as f32 / atomic_nodes.len() as f32
    }

    /// Get summary of execution
    pub fn summary(&self) -> TreeSummary {
        let total = self.nodes.len();
        let atomic = self.nodes.values().filter(|n| n.is_atomic).count();
        let completed = self.nodes.values().filter(|n| n.status == TaskNodeStatus::Completed).count();
        let failed = self.nodes.values().filter(|n| n.status == TaskNodeStatus::Failed).count();
        let pending = self.nodes.values().filter(|n| matches!(n.status, TaskNodeStatus::Pending | TaskNodeStatus::Blocked)).count();

        TreeSummary {
            total_nodes: total,
            atomic_tasks: atomic,
            completed,
            failed,
            pending,
            progress: self.progress(),
        }
    }
}

/// Summary statistics for a task tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeSummary {
    pub total_nodes: usize,
    pub atomic_tasks: usize,
    pub completed: usize,
    pub failed: usize,
    pub pending: usize,
    pub progress: f32,
}

// =============================================================================
// HIERARCHICAL PLANNER
// =============================================================================

/// Hierarchical planner for recursive task decomposition
pub struct HierarchicalPlanner {
    /// Maximum decomposition depth
    max_depth: usize,
    /// Complexity threshold for atomic tasks
    atomic_threshold: u8,
    /// Agent assignment mapping
    agent_capabilities: HashMap<String, Vec<String>>,
}

impl HierarchicalPlanner {
    pub fn new() -> Self {
        let mut capabilities = HashMap::new();
        
        // Default agent capabilities
        capabilities.insert("director".to_string(), vec!["planning".to_string(), "coordination".to_string()]);
        capabilities.insert("code".to_string(), vec!["code_generation".to_string(), "refactoring".to_string()]);
        capabilities.insert("debug".to_string(), vec!["debugging".to_string(), "error_analysis".to_string()]);
        capabilities.insert("fsm".to_string(), vec!["fsm_design".to_string(), "state_machine".to_string()]);
        capabilities.insert("hardware".to_string(), vec!["hardware".to_string(), "mcu".to_string(), "peripheral".to_string()]);
        capabilities.insert("docs".to_string(), vec!["documentation".to_string(), "commenting".to_string()]);
        capabilities.insert("build".to_string(), vec!["compilation".to_string(), "build".to_string()]);
        capabilities.insert("deploy".to_string(), vec!["flashing".to_string(), "deployment".to_string()]);

        Self {
            max_depth: 5,
            atomic_threshold: 3,
            agent_capabilities: capabilities,
        }
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn with_atomic_threshold(mut self, threshold: u8) -> Self {
        self.atomic_threshold = threshold;
        self
    }

    /// Decompose a goal into a task tree using AI
    pub async fn decompose<M: AIModel + ?Sized>(
        &self,
        goal: &str,
        context: &str,
        model: &M,
    ) -> Result<TaskTree, PlannerError> {
        // Create root node
        let root = TaskNode::new("root", goal).with_complexity(8);
        let mut tree = TaskTree::new(goal, root);

        // Recursively decompose
        self.decompose_node(&mut tree, "root", 0, context, model).await?;

        // Detect parallelization opportunities
        self.detect_parallelization(&mut tree);

        Ok(tree)
    }

    /// Recursively decompose a node
    async fn decompose_node<M: AIModel + ?Sized>(
        &self,
        tree: &mut TaskTree,
        node_id: &str,
        depth: usize,
        context: &str,
        model: &M,
    ) -> Result<(), PlannerError> {
        if depth >= self.max_depth {
            // Max depth reached, mark as atomic
            if let Some(node) = tree.get_node_mut(node_id) {
                node.is_atomic = true;
                self.assign_agent(node);
            }
            return Ok(());
        }

        let node_description = {
            let node = tree.get_node(node_id).ok_or_else(|| PlannerError::NodeNotFound(node_id.to_string()))?;
            
            // If already simple enough, mark as atomic
            if node.complexity <= self.atomic_threshold {
                return Ok(());
            }
            
            node.description.clone()
        };

        // Ask AI to decompose
        let subtasks = self.ai_decompose(&node_description, context, model).await?;

        if subtasks.is_empty() || subtasks.len() == 1 {
            // Can't decompose further, mark as atomic
            if let Some(node) = tree.get_node_mut(node_id) {
                node.is_atomic = true;
                self.assign_agent(node);
            }
            return Ok(());
        }

        // Create child nodes
        for (i, subtask) in subtasks.iter().enumerate() {
            let child_id = format!("{}_{}", node_id, i);
            let mut child = TaskNode::new(&child_id, &subtask.description)
                .with_complexity(subtask.complexity);
            
            // Add dependency on previous sibling if sequential
            if i > 0 && subtask.depends_on_previous {
                let prev_id = format!("{}_{}", node_id, i - 1);
                child = child.with_dependency(&prev_id);
            }

            tree.add_child(node_id, child);

            // Recursively decompose child
            Box::pin(self.decompose_node(tree, &child_id, depth + 1, context, model)).await?;
        }

        Ok(())
    }

    /// Use AI to decompose a task into subtasks
    async fn ai_decompose<M: AIModel + ?Sized>(
        &self,
        task: &str,
        context: &str,
        model: &M,
    ) -> Result<Vec<SubtaskInfo>, PlannerError> {
        let prompt = format!(
            r#"Decompose this task into smaller subtasks (2-5 subtasks).

Task: {}

Context: {}

Return a JSON array of subtasks with this format:
```json
[
  {{"description": "subtask description", "complexity": 1-10, "depends_on_previous": true/false, "category": "code|debug|fsm|hardware|docs|build|deploy|general"}}
]
```

Rules:
- Each subtask should be simpler than the parent
- Mark depends_on_previous as true if this subtask needs the previous one to complete first
- Complexity 1-3 means atomic (can be done directly), 4-10 needs further decomposition
- If the task is already simple enough, return a single subtask with the same description

Return ONLY the JSON array, no other text."#,
            task, context
        );

        let messages = vec![
            ChatMessage {
                role: Role::System,
                content: "You are a task planning assistant. Decompose complex tasks into simpler subtasks.".to_string(),
            },
            ChatMessage {
                role: Role::User,
                content: prompt,
            },
        ];

        let response = model.chat(&messages).await
            .map_err(|e| PlannerError::AIError(e.to_string()))?;

        // Parse JSON response
        let json_str = extract_json(&response.content);
        let subtasks: Vec<SubtaskInfo> = serde_json::from_str(json_str)
            .map_err(|e| PlannerError::ParseError(format!("Failed to parse subtasks: {}. Response: {}", e, json_str)))?;

        Ok(subtasks)
    }

    /// Assign an agent to a task based on its description
    fn assign_agent(&self, node: &mut TaskNode) {
        let description_lower = node.description.to_lowercase();
        
        // Simple keyword matching for agent assignment
        let agent = if description_lower.contains("fsm") || description_lower.contains("state machine") {
            "fsm"
        } else if description_lower.contains("debug") || description_lower.contains("error") || description_lower.contains("fix") {
            "debug"
        } else if description_lower.contains("code") || description_lower.contains("implement") || description_lower.contains("function") {
            "code"
        } else if description_lower.contains("hardware") || description_lower.contains("gpio") || description_lower.contains("peripheral") {
            "hardware"
        } else if description_lower.contains("document") || description_lower.contains("comment") {
            "docs"
        } else if description_lower.contains("build") || description_lower.contains("compile") {
            "build"
        } else if description_lower.contains("flash") || description_lower.contains("deploy") || description_lower.contains("upload") {
            "deploy"
        } else {
            "code" // Default to code agent
        };

        node.assigned_agent = Some(agent.to_string());
    }

    /// Detect and mark tasks that can run in parallel
    fn detect_parallelization(&self, tree: &mut TaskTree) {
        // Find sibling groups that have no dependencies on each other
        let node_ids: Vec<NodeId> = tree.nodes.keys().cloned().collect();
        
        for id in &node_ids {
            if let Some(node) = tree.nodes.get(id) {
                let children = node.children.clone();
                
                // Check if children can run in parallel
                if children.len() > 1 {
                    let mut independent_groups: Vec<Vec<&str>> = Vec::new();
                    let mut current_group: Vec<&str> = Vec::new();
                    
                    for child_id in &children {
                        if let Some(child) = tree.nodes.get(child_id) {
                            // Check if this child depends on any sibling
                            let depends_on_sibling = child.dependencies.iter()
                                .any(|dep| children.contains(dep));
                            
                            if depends_on_sibling {
                                // Start a new group
                                if !current_group.is_empty() {
                                    independent_groups.push(current_group.iter().map(|s| *s).collect());
                                    current_group.clear();
                                }
                            }
                            
                            current_group.push(child_id);
                        }
                    }
                    
                    if !current_group.is_empty() {
                        independent_groups.push(current_group);
                    }
                    
                    // Mark parallel groups in metadata
                    for (group_idx, group) in independent_groups.iter().enumerate() {
                        if group.len() > 1 {
                            for child_id in group {
                                if let Some(child) = tree.nodes.get_mut(*child_id) {
                                    child.metadata.insert(
                                        "parallel_group".to_string(),
                                        serde_json::json!(group_idx)
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Execute a task tree
    pub async fn execute<E>(
        &self,
        tree: &mut TaskTree,
        executor: &E,
    ) -> Result<ExecutionResult, PlannerError>
    where
        E: TaskExecutor,
    {
        tree.started_at = Some(Utc::now());
        let mut total_tokens = 0u32;
        let mut errors = Vec::new();

        while !tree.is_complete() {
            // Get tasks ready for execution
            let ready_nodes: Vec<NodeId> = tree.get_parallel_batch()
                .iter()
                .map(|n| n.id.clone())
                .collect();

            if ready_nodes.is_empty() {
                // No progress possible - might be a cycle or all blocked
                break;
            }

            // Execute ready tasks (could be parallelized with tokio::spawn)
            for node_id in ready_nodes {
                if let Some(node) = tree.get_node_mut(&node_id) {
                    node.status = TaskNodeStatus::InProgress;
                }

                let result = {
                    let node = tree.get_node(&node_id).unwrap();
                    executor.execute_task(node).await
                };

                match result {
                    Ok(task_result) => {
                        if let Some(tokens) = task_result.tokens_used {
                            total_tokens += tokens;
                        }
                        tree.set_result(&node_id, task_result);
                    }
                    Err(e) => {
                        errors.push(format!("{}: {}", node_id, e));
                        tree.set_result(&node_id, TaskResult {
                            success: false,
                            output: String::new(),
                            error: Some(e.to_string()),
                            duration_ms: 0,
                            tokens_used: None,
                        });
                    }
                }
            }
        }

        tree.completed_at = Some(Utc::now());

        Ok(ExecutionResult {
            success: errors.is_empty(),
            summary: tree.summary(),
            errors,
            total_tokens_used: total_tokens,
            duration_ms: tree.completed_at.unwrap()
                .signed_duration_since(tree.started_at.unwrap())
                .num_milliseconds() as u64,
        })
    }
}

impl Default for HierarchicalPlanner {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// EXECUTOR TRAIT
// =============================================================================

/// Trait for executing individual tasks
#[async_trait::async_trait]
pub trait TaskExecutor: Send + Sync {
    async fn execute_task(&self, task: &TaskNode) -> Result<TaskResult, PlannerError>;
}

// =============================================================================
// HELPER TYPES
// =============================================================================

/// Subtask info from AI decomposition
#[derive(Debug, Clone, Deserialize)]
struct SubtaskInfo {
    description: String,
    complexity: u8,
    #[serde(default)]
    depends_on_previous: bool,
    #[serde(default)]
    category: String,
}

/// Result of tree execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub summary: TreeSummary,
    pub errors: Vec<String>,
    pub total_tokens_used: u32,
    pub duration_ms: u64,
}

/// Planner errors
#[derive(Debug, thiserror::Error)]
pub enum PlannerError {
    #[error("AI error: {0}")]
    AIError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Cycle detected in task dependencies")]
    CycleDetected,
}

/// Extract JSON from possibly markdown-wrapped content
fn extract_json(content: &str) -> &str {
    if let Some(start) = content.find("```json") {
        let start = start + 7;
        if let Some(end) = content[start..].find("```") {
            return content[start..start + end].trim();
        }
    }
    
    if let Some(start) = content.find("```") {
        let start = start + 3;
        let start = content[start..].find('\n').map(|n| start + n + 1).unwrap_or(start);
        if let Some(end) = content[start..].find("```") {
            return content[start..start + end].trim();
        }
    }

    if let Some(start) = content.find('[') {
        if let Some(end) = content.rfind(']') {
            return &content[start..=end];
        }
    }

    content.trim()
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_node_creation() {
        let node = TaskNode::new("test", "Test task")
            .with_complexity(7)
            .with_agent("code");

        assert_eq!(node.id, "test");
        assert_eq!(node.complexity, 7);
        assert_eq!(node.assigned_agent, Some("code".to_string()));
        assert!(!node.is_atomic);
    }

    #[test]
    fn test_task_tree_operations() {
        let root = TaskNode::new("root", "Main task");
        let mut tree = TaskTree::new("Test goal", root);

        let child1 = TaskNode::atomic("child1", "Subtask 1", "code");
        let child2 = TaskNode::atomic("child2", "Subtask 2", "debug")
            .with_dependency("child1");

        tree.add_child("root", child1);
        tree.add_child("root", child2);

        assert_eq!(tree.nodes.len(), 3);
        
        // Only child1 should be ready (child2 depends on child1)
        let ready = tree.get_ready_nodes();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "child1");
    }

    #[test]
    fn test_progress_calculation() {
        let root = TaskNode::new("root", "Main task");
        let mut tree = TaskTree::new("Test goal", root);

        let child1 = TaskNode::atomic("child1", "Subtask 1", "code");
        let child2 = TaskNode::atomic("child2", "Subtask 2", "code");

        tree.add_child("root", child1);
        tree.add_child("root", child2);

        assert_eq!(tree.progress(), 0.0);

        tree.update_status("child1", TaskNodeStatus::Completed);
        assert_eq!(tree.progress(), 0.5);

        tree.update_status("child2", TaskNodeStatus::Completed);
        assert_eq!(tree.progress(), 1.0);
    }

    #[test]
    fn test_parallel_batch() {
        let root = TaskNode::new("root", "Main task");
        let mut tree = TaskTree::new("Test goal", root);

        // Two independent children
        let child1 = TaskNode::atomic("child1", "Subtask 1", "code");
        let child2 = TaskNode::atomic("child2", "Subtask 2", "debug");

        tree.add_child("root", child1);
        tree.add_child("root", child2);

        // Both should be ready for parallel execution
        let parallel = tree.get_parallel_batch();
        assert_eq!(parallel.len(), 2);
    }

    #[test]
    fn test_json_extraction() {
        let markdown = "Here's the result:\n```json\n[{\"description\": \"test\"}]\n```\nDone!";
        let extracted = extract_json(markdown);
        assert!(extracted.starts_with('['));
        assert!(extracted.contains("description"));
    }
}
