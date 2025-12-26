//! Background Task Queue
//!
//! Enables async operations for builds, flashing, and agent work.
//! Provides progress tracking and task management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, oneshot};
use chrono::{DateTime, Utc};

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Task status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Task type for categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    Build,
    Flash,
    CodeGen,
    Validate,
    AgentChat,
    Analysis,
    Custom(String),
}

/// Progress update for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    pub percent: f32,
    pub message: String,
    pub stage: Option<String>,
}

impl TaskProgress {
    pub fn new(percent: f32, message: &str) -> Self {
        Self {
            percent: percent.clamp(0.0, 100.0),
            message: message.to_string(),
            stage: None,
        }
    }
    
    pub fn with_stage(mut self, stage: &str) -> Self {
        self.stage = Some(stage.to_string());
        self
    }
}

/// Background task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundTask {
    pub id: String,
    pub name: String,
    pub task_type: TaskType,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub progress: TaskProgress,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl BackgroundTask {
    pub fn new(name: &str, task_type: TaskType) -> Self {
        Self {
            id: format!("task_{}", Utc::now().timestamp_millis()),
            name: name.to_string(),
            task_type,
            priority: TaskPriority::Normal,
            status: TaskStatus::Queued,
            progress: TaskProgress::new(0.0, "Queued"),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
        }
    }
    
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }
    
    pub fn start(&mut self) {
        self.status = TaskStatus::Running;
        self.started_at = Some(Utc::now());
        self.progress = TaskProgress::new(0.0, "Starting...");
    }
    
    pub fn update_progress(&mut self, percent: f32, message: &str) {
        self.progress = TaskProgress::new(percent, message);
    }
    
    pub fn complete(&mut self, result: serde_json::Value) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.progress = TaskProgress::new(100.0, "Completed");
        self.result = Some(result);
    }
    
    pub fn fail(&mut self, error: &str) {
        self.status = TaskStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.progress.message = format!("Failed: {}", error);
        self.error = Some(error.to_string());
    }
    
    pub fn cancel(&mut self) {
        self.status = TaskStatus::Cancelled;
        self.completed_at = Some(Utc::now());
        self.progress.message = "Cancelled".to_string();
    }
    
    pub fn elapsed_seconds(&self) -> Option<i64> {
        self.started_at.map(|start| {
            let end = self.completed_at.unwrap_or_else(Utc::now);
            (end - start).num_seconds()
        })
    }
}

/// Message types for task queue communication
#[derive(Debug)]
pub enum TaskMessage {
    /// Submit a new task
    Submit(BackgroundTask, oneshot::Sender<String>),
    /// Cancel a task
    Cancel(String),
    /// Get task status
    Status(String, oneshot::Sender<Option<BackgroundTask>>),
    /// List all tasks
    List(oneshot::Sender<Vec<BackgroundTask>>),
    /// Clear completed tasks
    ClearCompleted,
}

/// Shared task state
pub type SharedTaskState = Arc<RwLock<HashMap<String, BackgroundTask>>>;

/// Background task queue manager
pub struct TaskQueue {
    tasks: SharedTaskState,
    sender: mpsc::Sender<TaskMessage>,
}

impl TaskQueue {
    /// Create a new task queue with a worker
    pub fn new() -> Self {
        let tasks: SharedTaskState = Arc::new(RwLock::new(HashMap::new()));
        let (sender, mut receiver) = mpsc::channel::<TaskMessage>(100);
        
        let tasks_clone = tasks.clone();
        
        // Spawn worker task
        tokio::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                match msg {
                    TaskMessage::Submit(task, response) => {
                        let id = task.id.clone();
                        tasks_clone.write().await.insert(id.clone(), task);
                        let _ = response.send(id);
                    }
                    TaskMessage::Cancel(id) => {
                        if let Some(task) = tasks_clone.write().await.get_mut(&id) {
                            task.cancel();
                        }
                    }
                    TaskMessage::Status(id, response) => {
                        let task = tasks_clone.read().await.get(&id).cloned();
                        let _ = response.send(task);
                    }
                    TaskMessage::List(response) => {
                        let list: Vec<BackgroundTask> = tasks_clone
                            .read()
                            .await
                            .values()
                            .cloned()
                            .collect();
                        let _ = response.send(list);
                    }
                    TaskMessage::ClearCompleted => {
                        let mut tasks = tasks_clone.write().await;
                        tasks.retain(|_, t| {
                            !matches!(t.status, TaskStatus::Completed | TaskStatus::Cancelled | TaskStatus::Failed)
                        });
                    }
                }
            }
        });
        
        Self { tasks, sender }
    }
    
    /// Submit a task and get its ID
    pub async fn submit(&self, task: BackgroundTask) -> Result<String, String> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(TaskMessage::Submit(task, tx))
            .await
            .map_err(|e| e.to_string())?;
        rx.await.map_err(|e| e.to_string())
    }
    
    /// Cancel a task
    pub async fn cancel(&self, task_id: &str) -> Result<(), String> {
        self.sender
            .send(TaskMessage::Cancel(task_id.to_string()))
            .await
            .map_err(|e| e.to_string())
    }
    
    /// Get task status
    pub async fn get_status(&self, task_id: &str) -> Option<BackgroundTask> {
        let (tx, rx) = oneshot::channel();
        if self.sender
            .send(TaskMessage::Status(task_id.to_string(), tx))
            .await
            .is_ok()
        {
            rx.await.ok().flatten()
        } else {
            None
        }
    }
    
    /// List all tasks
    pub async fn list(&self) -> Vec<BackgroundTask> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(TaskMessage::List(tx)).await.is_ok() {
            rx.await.unwrap_or_default()
        } else {
            Vec::new()
        }
    }
    
    /// Clear completed tasks
    pub async fn clear_completed(&self) {
        let _ = self.sender.send(TaskMessage::ClearCompleted).await;
    }
    
    /// Update task progress directly
    pub async fn update_progress(&self, task_id: &str, percent: f32, message: &str) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.update_progress(percent, message);
        }
    }
    
    /// Mark task as completed
    pub async fn complete_task(&self, task_id: &str, result: serde_json::Value) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.complete(result);
        }
    }
    
    /// Mark task as failed
    pub async fn fail_task(&self, task_id: &str, error: &str) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.fail(error);
        }
    }
    
    /// Start a task
    pub async fn start_task(&self, task_id: &str) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.start();
        }
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Task summary for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub id: String,
    pub name: String,
    pub status: TaskStatus,
    pub progress_percent: f32,
    pub progress_message: String,
    pub elapsed_seconds: Option<i64>,
}

impl From<&BackgroundTask> for TaskSummary {
    fn from(task: &BackgroundTask) -> Self {
        Self {
            id: task.id.clone(),
            name: task.name.clone(),
            status: task.status.clone(),
            progress_percent: task.progress.percent,
            progress_message: task.progress.message.clone(),
            elapsed_seconds: task.elapsed_seconds(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_task_lifecycle() {
        let queue = TaskQueue::new();
        
        // Submit task
        let task = BackgroundTask::new("Test Build", TaskType::Build);
        let id = queue.submit(task).await.unwrap();
        
        // Check status
        let status = queue.get_status(&id).await.unwrap();
        assert_eq!(status.status, TaskStatus::Queued);
        
        // Start task
        queue.start_task(&id).await;
        let status = queue.get_status(&id).await.unwrap();
        assert_eq!(status.status, TaskStatus::Running);
        
        // Update progress
        queue.update_progress(&id, 50.0, "Compiling...").await;
        
        // Complete task
        queue.complete_task(&id, serde_json::json!({"success": true})).await;
        let status = queue.get_status(&id).await.unwrap();
        assert_eq!(status.status, TaskStatus::Completed);
    }
}
