//! Plan Executor
//!
//! Executes multi-step task plans created by the Director Agent.
//! Handles agent delegation, progress tracking, and result synthesis.

use super::{AgentRegistry, AgentContext, AgentResponse, Task, TaskPlan, TaskStatus as PlanTaskStatus};
use super::memory::{AgentMemory, MemoryCategory};
use super::task_queue::{TaskQueue, BackgroundTask, TaskType, TaskStatus};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Result of executing a single step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub task_id: String,
    pub agent_id: String,
    pub success: bool,
    pub response: Option<AgentResponse>,
    pub error: Option<String>,
}

/// Result of executing an entire plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanResult {
    pub plan_id: String,
    pub success: bool,
    pub steps: Vec<StepResult>,
    pub final_response: String,
}

/// Executes task plans step by step
pub struct PlanExecutor {
    registry: Arc<RwLock<AgentRegistry>>,
    context: Arc<RwLock<AgentContext>>,
    memory: Arc<RwLock<AgentMemory>>,
    task_queue: Arc<TaskQueue>,
}

impl PlanExecutor {
    pub fn new(
        registry: Arc<RwLock<AgentRegistry>>,
        context: Arc<RwLock<AgentContext>>,
        memory: Arc<RwLock<AgentMemory>>,
        task_queue: Arc<TaskQueue>,
    ) -> Self {
        Self {
            registry,
            context,
            memory,
            task_queue,
        }
    }
    
    /// Execute a complete plan
    pub async fn execute_plan(&self, mut plan: TaskPlan) -> PlanResult {
        let mut step_results = Vec::new();
        let mut all_success = true;
        
        // Create background task for the plan
        let bg_task = BackgroundTask::new(&plan.original_request, TaskType::AgentChat);
        let bg_task_id = self.task_queue.submit(bg_task).await.unwrap_or_default();
        self.task_queue.start_task(&bg_task_id).await;
        
        // Execute each task in sequence
        let task_count = plan.tasks.len();
        for (index, task) in plan.tasks.iter_mut().enumerate() {
            // Update progress
            let percent = (index as f32 / task_count as f32) * 100.0;
            self.task_queue.update_progress(
                &bg_task_id,
                percent,
                &format!("Step {}/{}: {}", index + 1, task_count, task.description)
            ).await;
            
            // Execute the step
            let result = self.execute_step(task).await;
            
            if result.success {
                task.status = PlanTaskStatus::Completed;
                
                // Store successful result in memory
                if let Some(ref response) = result.response {
                    let mut memory = self.memory.write().await;
                    memory.learn(
                        &format!("Completed: {} via {} agent", task.description, task.agent_id),
                        MemoryCategory::TaskHistory,
                        0.6
                    );
                    
                    // Record the response message
                    memory.record_assistant_message(&response.message);
                }
            } else {
                task.status = PlanTaskStatus::Failed;
                all_success = false;
                
                // Store failure in memory
                if let Some(ref error) = result.error {
                    let mut memory = self.memory.write().await;
                    memory.learn(
                        &format!("Failed: {} - {}", task.description, error),
                        MemoryCategory::ErrorPattern,
                        0.8
                    );
                }
            }
            
            step_results.push(result);
            plan.current_step = index + 1;
            
            // Stop on failure (unless task is optional)
            if !all_success {
                break;
            }
        }
        
        // Mark plan complete
        plan.status = if all_success {
            PlanTaskStatus::Completed
        } else {
            PlanTaskStatus::Failed
        };
        
        // Synthesize final response
        let final_response = self.synthesize_results(&plan, &step_results);
        
        // Complete background task
        if all_success {
            self.task_queue.complete_task(&bg_task_id, serde_json::json!({
                "plan_id": plan.id,
                "steps_completed": step_results.len(),
            })).await;
        } else {
            self.task_queue.fail_task(&bg_task_id, "Plan execution failed").await;
        }
        
        PlanResult {
            plan_id: plan.id,
            success: all_success,
            steps: step_results,
            final_response,
        }
    }
    
    /// Execute a single step by delegating to the appropriate agent
    async fn execute_step(&self, task: &mut Task) -> StepResult {
        task.status = PlanTaskStatus::InProgress;
        
        // Get the agent
        let registry = self.registry.read().await;
        let agent = match registry.get(&task.agent_id) {
            Some(agent) => agent,
            None => {
                return StepResult {
                    task_id: task.id.clone(),
                    agent_id: task.agent_id.clone(),
                    success: false,
                    response: None,
                    error: Some(format!("Agent '{}' not found", task.agent_id)),
                };
            }
        };
        
        // Get context
        let context = self.context.read().await;
        
        // Execute the task
        match agent.process(&task.description, &context).await {
            Ok(response) => StepResult {
                task_id: task.id.clone(),
                agent_id: task.agent_id.clone(),
                success: true,
                response: Some(response),
                error: None,
            },
            Err(error) => StepResult {
                task_id: task.id.clone(),
                agent_id: task.agent_id.clone(),
                success: false,
                response: None,
                error: Some(error),
            },
        }
    }
    
    /// Synthesize results from multiple steps into a coherent response
    fn synthesize_results(&self, plan: &TaskPlan, results: &[StepResult]) -> String {
        let mut parts = Vec::new();
        
        // Header
        parts.push(format!("## Plan Execution: {}", plan.original_request));
        parts.push(String::new());
        
        // Results summary
        let successful = results.iter().filter(|r| r.success).count();
        let total = results.len();
        
        if successful == total {
            parts.push(format!("✅ All {} steps completed successfully!", total));
        } else {
            parts.push(format!("⚠️ {} of {} steps completed", successful, total));
        }
        parts.push(String::new());
        
        // Individual step results
        for (i, result) in results.iter().enumerate() {
            let status = if result.success { "✓" } else { "✗" };
            let agent = &result.agent_id;
            
            if let Some(ref response) = result.response {
                // Truncate long responses
                let msg = if response.message.len() > 200 {
                    format!("{}...", &response.message[..200])
                } else {
                    response.message.clone()
                };
                parts.push(format!("{} Step {}: [{}] {}", status, i + 1, agent, msg));
            } else if let Some(ref error) = result.error {
                parts.push(format!("{} Step {}: [{}] Error: {}", status, i + 1, agent, error));
            }
        }
        
        parts.join("\n")
    }
    
    /// Execute a single task (not part of a plan)
    pub async fn execute_single(&self, agent_id: &str, message: &str) -> Result<AgentResponse, String> {
        let registry = self.registry.read().await;
        let agent = registry.get(agent_id)
            .ok_or_else(|| format!("Agent '{}' not found", agent_id))?;
        
        let context = self.context.read().await;
        
        // Record in memory
        {
            let mut memory = self.memory.write().await;
            memory.record_user_message(message);
        }
        
        let response = agent.process(message, &context).await?;
        
        // Record response in memory
        {
            let mut memory = self.memory.write().await;
            memory.record_assistant_message(&response.message);
        }
        
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_step_result() {
        let result = StepResult {
            task_id: "task_1".to_string(),
            agent_id: "code".to_string(),
            success: true,
            response: Some(AgentResponse {
                message: "Code generated".to_string(),
                tool_calls: vec![],
                suggestions: vec![],
            }),
            error: None,
        };
        
        assert!(result.success);
        assert!(result.error.is_none());
    }
}
