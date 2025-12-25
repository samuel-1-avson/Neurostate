// RTOS Code Generation Module
// Supports FreeRTOS and Zephyr

use serde::{Deserialize, Serialize};

/// Supported RTOS types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RtosType {
    FreeRtos,
    Zephyr,
    BareMetal,
}

impl Default for RtosType {
    fn default() -> Self {
        RtosType::FreeRtos
    }
}

/// Task priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TaskPriority {
    Idle,
    Low,
    BelowNormal,
    Normal,
    AboveNormal,
    High,
    Realtime,
    Custom(u8),
}

impl TaskPriority {
    pub fn to_freertos(&self) -> u8 {
        match self {
            TaskPriority::Idle => 0,
            TaskPriority::Low => 1,
            TaskPriority::BelowNormal => 2,
            TaskPriority::Normal => 3,
            TaskPriority::AboveNormal => 4,
            TaskPriority::High => 5,
            TaskPriority::Realtime => 6,
            TaskPriority::Custom(p) => *p,
        }
    }
    
    pub fn to_zephyr(&self) -> i8 {
        // Zephyr uses lower numbers for higher priority
        match self {
            TaskPriority::Idle => 14,
            TaskPriority::Low => 10,
            TaskPriority::BelowNormal => 7,
            TaskPriority::Normal => 5,
            TaskPriority::AboveNormal => 3,
            TaskPriority::High => 1,
            TaskPriority::Realtime => 0,
            TaskPriority::Custom(p) => 14 - (*p as i8).min(14),
        }
    }
}

/// Task configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    pub name: String,
    pub stack_size: u32,
    pub priority: TaskPriority,
    pub entry_function: String,
    pub parameter: Option<String>,
    pub auto_start: bool,
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            name: "Task1".to_string(),
            stack_size: 1024,
            priority: TaskPriority::Normal,
            entry_function: "vTask1".to_string(),
            parameter: None,
            auto_start: true,
        }
    }
}

/// Semaphore types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SemaphoreType {
    Binary,
    Counting(u32),  // max count
}

/// Semaphore configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemaphoreConfig {
    pub name: String,
    pub sem_type: SemaphoreType,
    pub initial_count: u32,
}

/// Mutex configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutexConfig {
    pub name: String,
    pub recursive: bool,
}

/// Queue configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    pub name: String,
    pub length: u32,
    pub item_size: u32,
}

/// Software timer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerConfig {
    pub name: String,
    pub period_ms: u32,
    pub auto_reload: bool,
    pub callback: String,
}

/// Event group configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventGroupConfig {
    pub name: String,
    pub num_bits: u8,
}

pub mod freertos;
pub mod zephyr;

/// RTOS HAL trait
pub trait RtosHal {
    fn rtos_type(&self) -> RtosType;
    fn generate_task(&self, config: &TaskConfig) -> String;
    fn generate_semaphore(&self, config: &SemaphoreConfig) -> String;
    fn generate_mutex(&self, config: &MutexConfig) -> String;
    fn generate_queue(&self, config: &QueueConfig) -> String;
    fn generate_timer(&self, config: &TimerConfig) -> String;
    fn generate_event_group(&self, config: &EventGroupConfig) -> String;
    fn generate_config_header(&self) -> String;
    fn generate_main(&self, tasks: &[TaskConfig]) -> String;
}

/// Get RTOS HAL by type
pub fn get_rtos_hal(rtos: RtosType) -> Box<dyn RtosHal> {
    match rtos {
        RtosType::FreeRtos => Box::new(freertos::FreeRtosHal::new()),
        RtosType::Zephyr => Box::new(zephyr::ZephyrHal::new()),
        RtosType::BareMetal => Box::new(freertos::FreeRtosHal::new()), // Fallback
    }
}
