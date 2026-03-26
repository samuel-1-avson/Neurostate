//! RTOS Simulation
//!
//! Simulates FreeRTOS and Zephyr task scheduling, semaphores, mutexes, and queues.

use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::cmp::Ordering;

/// Task state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Suspended,
    Deleted,
}

/// Task control block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub name: String,
    pub priority: u8,
    pub state: TaskState,
    pub stack_size: usize,
    pub stack_used: usize,
    pub entry_point: u32,
    pub blocked_on: Option<BlockReason>,
    pub blocked_until: Option<u64>,
    pub cpu_time: u64,
    pub context: TaskContext,
}

/// Task context (saved registers)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskContext {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r12: u32,
    pub lr: u32,
    pub pc: u32,
    pub xpsr: u32,
    // Remaining registers saved by software
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    pub r8: u32,
    pub r9: u32,
    pub r10: u32,
    pub r11: u32,
    pub sp: u32,
}

/// Reason a task is blocked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockReason {
    Delay,
    Semaphore(u32),
    Mutex(u32),
    Queue(u32),
    Event(u32),
}

/// Priority-ordered task for scheduling
#[derive(Eq, PartialEq)]
struct PriorityTask {
    priority: u8,
    id: u32,
}

impl Ord for PriorityTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority = higher value = should come first
        other.priority.cmp(&self.priority)
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for PriorityTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Semaphore
#[derive(Debug, Clone)]
pub struct Semaphore {
    pub id: u32,
    pub count: i32,
    pub max_count: i32,
    pub waiting_tasks: VecDeque<u32>,
}

/// Mutex
#[derive(Debug, Clone)]
pub struct Mutex {
    pub id: u32,
    pub owner: Option<u32>,
    pub recursive_count: u32,
    pub waiting_tasks: VecDeque<u32>,
    pub original_priority: Option<u8>, // For priority inheritance
}

/// Message queue
#[derive(Debug, Clone)]
pub struct MessageQueue {
    pub id: u32,
    pub capacity: usize,
    pub item_size: usize,
    pub messages: VecDeque<Vec<u8>>,
    pub waiting_readers: VecDeque<u32>,
    pub waiting_writers: VecDeque<u32>,
}

/// Timer callback
#[derive(Debug, Clone)]
pub struct SoftwareTimer {
    pub id: u32,
    pub name: String,
    pub period_ticks: u64,
    pub one_shot: bool,
    pub active: bool,
    pub next_expiry: u64,
    pub callback_addr: u32,
}

/// RTOS kernel state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KernelState {
    NotStarted,
    Running,
    Suspended,
}

/// RTOS Scheduler
pub struct RtosScheduler {
    /// Kernel state
    state: KernelState,
    /// All tasks
    tasks: HashMap<u32, Task>,
    /// Ready queue (priority heap)
    ready_queue: BinaryHeap<PriorityTask>,
    /// Currently running task
    current_task: Option<u32>,
    /// Idle task ID
    idle_task_id: u32,
    /// Semaphores
    semaphores: HashMap<u32, Semaphore>,
    /// Mutexes
    mutexes: HashMap<u32, Mutex>,
    /// Message queues
    queues: HashMap<u32, MessageQueue>,
    /// Software timers
    timers: Vec<SoftwareTimer>,
    /// Next object ID
    next_id: u32,
    /// System tick counter
    tick_count: u64,
    /// Ticks per time slice (for round-robin)
    time_slice_ticks: u32,
    /// Current time slice remaining
    slice_remaining: u32,
    /// Context switch required
    context_switch_pending: bool,
    /// Statistics
    context_switches: u64,
    total_idle_time: u64,
}

impl RtosScheduler {
    pub fn new() -> Self {
        Self {
            state: KernelState::NotStarted,
            tasks: HashMap::new(),
            ready_queue: BinaryHeap::new(),
            current_task: None,
            idle_task_id: 0,
            semaphores: HashMap::new(),
            mutexes: HashMap::new(),
            queues: HashMap::new(),
            timers: Vec::new(),
            next_id: 1,
            tick_count: 0,
            time_slice_ticks: 10,
            slice_remaining: 10,
            context_switch_pending: false,
            context_switches: 0,
            total_idle_time: 0,
        }
    }

    /// Create a new task
    pub fn task_create(
        &mut self,
        name: &str,
        priority: u8,
        stack_size: usize,
        entry_point: u32,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        let task = Task {
            id,
            name: name.to_string(),
            priority,
            state: TaskState::Ready,
            stack_size,
            stack_used: 0,
            entry_point,
            blocked_on: None,
            blocked_until: None,
            cpu_time: 0,
            context: TaskContext {
                pc: entry_point,
                xpsr: 0x0100_0000, // Thumb bit set
                ..Default::default()
            },
        };

        self.tasks.insert(id, task);
        self.ready_queue.push(PriorityTask { priority, id });

        id
    }

    /// Delete a task
    pub fn task_delete(&mut self, task_id: u32) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.state = TaskState::Deleted;
        }
    }

    /// Suspend a task
    pub fn task_suspend(&mut self, task_id: u32) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.state = TaskState::Suspended;
        }
    }

    /// Resume a task
    pub fn task_resume(&mut self, task_id: u32) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            if task.state == TaskState::Suspended {
                task.state = TaskState::Ready;
                self.ready_queue.push(PriorityTask { priority: task.priority, id: task_id });
            }
        }
    }

    /// Delay task for ticks
    pub fn task_delay(&mut self, ticks: u64) {
        if let Some(task_id) = self.current_task {
            if let Some(task) = self.tasks.get_mut(&task_id) {
                task.state = TaskState::Blocked;
                task.blocked_on = Some(BlockReason::Delay);
                task.blocked_until = Some(self.tick_count + ticks);
                self.context_switch_pending = true;
            }
        }
    }

    /// Create semaphore
    pub fn semaphore_create(&mut self, initial_count: i32, max_count: i32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        self.semaphores.insert(id, Semaphore {
            id,
            count: initial_count,
            max_count,
            waiting_tasks: VecDeque::new(),
        });

        id
    }

    /// Take semaphore
    pub fn semaphore_take(&mut self, sem_id: u32, timeout_ticks: Option<u64>) -> bool {
        if let Some(sem) = self.semaphores.get_mut(&sem_id) {
            if sem.count > 0 {
                sem.count -= 1;
                return true;
            }

            // Need to block
            if let Some(task_id) = self.current_task {
                if let Some(task) = self.tasks.get_mut(&task_id) {
                    task.state = TaskState::Blocked;
                    task.blocked_on = Some(BlockReason::Semaphore(sem_id));
                    task.blocked_until = timeout_ticks.map(|t| self.tick_count + t);
                    sem.waiting_tasks.push_back(task_id);
                    self.context_switch_pending = true;
                }
            }
        }
        false
    }

    /// Give semaphore
    pub fn semaphore_give(&mut self, sem_id: u32) -> bool {
        if let Some(sem) = self.semaphores.get_mut(&sem_id) {
            // Wake up waiting task
            if let Some(task_id) = sem.waiting_tasks.pop_front() {
                if let Some(task) = self.tasks.get_mut(&task_id) {
                    task.state = TaskState::Ready;
                    task.blocked_on = None;
                    task.blocked_until = None;
                    self.ready_queue.push(PriorityTask { priority: task.priority, id: task_id });
                }
                return true;
            }

            // No waiters, increment count
            if sem.count < sem.max_count {
                sem.count += 1;
                return true;
            }
        }
        false
    }

    /// Create mutex
    pub fn mutex_create(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        self.mutexes.insert(id, Mutex {
            id,
            owner: None,
            recursive_count: 0,
            waiting_tasks: VecDeque::new(),
            original_priority: None,
        });

        id
    }

    /// Lock mutex
    pub fn mutex_lock(&mut self, mutex_id: u32, timeout_ticks: Option<u64>) -> bool {
        let current = self.current_task;
        
        if let Some(mutex) = self.mutexes.get_mut(&mutex_id) {
            if mutex.owner.is_none() {
                mutex.owner = current;
                mutex.recursive_count = 1;
                return true;
            }
            
            if mutex.owner == current {
                mutex.recursive_count += 1;
                return true;
            }

            // Need to block (with priority inheritance)
            if let Some(task_id) = current {
                if let Some(task) = self.tasks.get_mut(&task_id) {
                    task.state = TaskState::Blocked;
                    task.blocked_on = Some(BlockReason::Mutex(mutex_id));
                    task.blocked_until = timeout_ticks.map(|t| self.tick_count + t);
                    mutex.waiting_tasks.push_back(task_id);
                    self.context_switch_pending = true;
                }
            }
        }
        false
    }

    /// Unlock mutex
    pub fn mutex_unlock(&mut self, mutex_id: u32) -> bool {
        if let Some(mutex) = self.mutexes.get_mut(&mutex_id) {
            if mutex.owner != self.current_task {
                return false; // Not owner
            }

            mutex.recursive_count -= 1;
            if mutex.recursive_count == 0 {
                mutex.owner = None;

                // Wake up next waiter
                if let Some(task_id) = mutex.waiting_tasks.pop_front() {
                    if let Some(task) = self.tasks.get_mut(&task_id) {
                        task.state = TaskState::Ready;
                        task.blocked_on = None;
                        self.ready_queue.push(PriorityTask { priority: task.priority, id: task_id });
                    }
                    
                    // Transfer ownership
                    mutex.owner = Some(task_id);
                    mutex.recursive_count = 1;
                }
            }
            return true;
        }
        false
    }

    /// Create message queue
    pub fn queue_create(&mut self, capacity: usize, item_size: usize) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        self.queues.insert(id, MessageQueue {
            id,
            capacity,
            item_size,
            messages: VecDeque::new(),
            waiting_readers: VecDeque::new(),
            waiting_writers: VecDeque::new(),
        });

        id
    }

    /// Send to queue
    pub fn queue_send(&mut self, queue_id: u32, data: Vec<u8>, timeout_ticks: Option<u64>) -> bool {
        if let Some(queue) = self.queues.get_mut(&queue_id) {
            // Wake up waiting reader
            if let Some(task_id) = queue.waiting_readers.pop_front() {
                if let Some(task) = self.tasks.get_mut(&task_id) {
                    task.state = TaskState::Ready;
                    task.blocked_on = None;
                    self.ready_queue.push(PriorityTask { priority: task.priority, id: task_id });
                }
            }

            if queue.messages.len() < queue.capacity {
                queue.messages.push_back(data);
                return true;
            }

            // Queue full, block
            if let Some(task_id) = self.current_task {
                if let Some(task) = self.tasks.get_mut(&task_id) {
                    task.state = TaskState::Blocked;
                    task.blocked_on = Some(BlockReason::Queue(queue_id));
                    task.blocked_until = timeout_ticks.map(|t| self.tick_count + t);
                    queue.waiting_writers.push_back(task_id);
                    self.context_switch_pending = true;
                }
            }
        }
        false
    }

    /// Receive from queue
    pub fn queue_receive(&mut self, queue_id: u32, timeout_ticks: Option<u64>) -> Option<Vec<u8>> {
        if let Some(queue) = self.queues.get_mut(&queue_id) {
            if let Some(msg) = queue.messages.pop_front() {
                // Wake up waiting writer
                if let Some(task_id) = queue.waiting_writers.pop_front() {
                    if let Some(task) = self.tasks.get_mut(&task_id) {
                        task.state = TaskState::Ready;
                        task.blocked_on = None;
                        self.ready_queue.push(PriorityTask { priority: task.priority, id: task_id });
                    }
                }
                return Some(msg);
            }

            // Queue empty, block
            if let Some(task_id) = self.current_task {
                if let Some(task) = self.tasks.get_mut(&task_id) {
                    task.state = TaskState::Blocked;
                    task.blocked_on = Some(BlockReason::Queue(queue_id));
                    task.blocked_until = timeout_ticks.map(|t| self.tick_count + t);
                    queue.waiting_readers.push_back(task_id);
                    self.context_switch_pending = true;
                }
            }
        }
        None
    }

    /// System tick - call from SysTick handler
    pub fn tick(&mut self) {
        self.tick_count += 1;

        // Wake up delayed tasks
        let tick = self.tick_count;
        for task in self.tasks.values_mut() {
            if task.state == TaskState::Blocked {
                if let Some(until) = task.blocked_until {
                    if tick >= until {
                        task.state = TaskState::Ready;
                        task.blocked_on = None;
                        task.blocked_until = None;
                        self.ready_queue.push(PriorityTask { priority: task.priority, id: task.id });
                    }
                }
            }
        }

        // Check timers
        let expired: Vec<_> = self.timers.iter()
            .filter(|t| t.active && tick >= t.next_expiry)
            .map(|t| t.id)
            .collect();

        for timer_id in expired {
            if let Some(timer) = self.timers.iter_mut().find(|t| t.id == timer_id) {
                if timer.one_shot {
                    timer.active = false;
                } else {
                    timer.next_expiry = tick + timer.period_ticks;
                }
                // Would trigger callback here
            }
        }

        // Time slice expiry
        self.slice_remaining = self.slice_remaining.saturating_sub(1);
        if self.slice_remaining == 0 {
            self.slice_remaining = self.time_slice_ticks;
            self.context_switch_pending = true;
        }
    }

    /// Schedule next task
    pub fn schedule(&mut self) -> Option<&Task> {
        if !self.context_switch_pending {
            return self.current_task.and_then(|id| self.tasks.get(&id));
        }

        self.context_switch_pending = false;
        self.context_switches += 1;

        // Find highest priority ready task
        while let Some(pt) = self.ready_queue.pop() {
            if let Some(task) = self.tasks.get(&pt.id) {
                if task.state == TaskState::Ready {
                    if let Some(old_id) = self.current_task {
                        if let Some(old) = self.tasks.get_mut(&old_id) {
                            if old.state == TaskState::Running {
                                old.state = TaskState::Ready;
                            }
                        }
                    }

                    self.current_task = Some(pt.id);
                    if let Some(task) = self.tasks.get_mut(&pt.id) {
                        task.state = TaskState::Running;
                    }
                    return self.tasks.get(&pt.id);
                }
            }
        }

        // No ready tasks, run idle
        self.current_task = Some(self.idle_task_id);
        self.tasks.get(&self.idle_task_id)
    }

    /// Start the scheduler
    pub fn start(&mut self) {
        // Create idle task
        self.idle_task_id = self.task_create("IDLE", 0, 128, 0);
        self.state = KernelState::Running;
        self.schedule();
    }

    /// Get all tasks
    pub fn get_tasks(&self) -> Vec<&Task> {
        self.tasks.values().collect()
    }

    /// Get statistics
    pub fn get_stats(&self) -> RtosStats {
        RtosStats {
            tick_count: self.tick_count,
            context_switches: self.context_switches,
            task_count: self.tasks.len(),
            semaphore_count: self.semaphores.len(),
            mutex_count: self.mutexes.len(),
            queue_count: self.queues.len(),
        }
    }
}

impl Default for RtosScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// RTOS statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtosStats {
    pub tick_count: u64,
    pub context_switches: u64,
    pub task_count: usize,
    pub semaphore_count: usize,
    pub mutex_count: usize,
    pub queue_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_create() {
        let mut rtos = RtosScheduler::new();
        let id = rtos.task_create("Task1", 5, 512, 0x0800_0100);
        
        assert_eq!(id, 1);
        assert!(rtos.tasks.contains_key(&id));
    }

    #[test]
    fn test_semaphore() {
        let mut rtos = RtosScheduler::new();
        let sem = rtos.semaphore_create(1, 1);
        
        assert!(rtos.semaphore_take(sem, None));
        assert!(!rtos.semaphore_take(sem, None)); // Would block
    }

    #[test]
    fn test_queue() {
        let mut rtos = RtosScheduler::new();
        let q = rtos.queue_create(5, 8);
        
        rtos.queue_send(q, vec![1, 2, 3], None);
        let msg = rtos.queue_receive(q, None);
        
        assert_eq!(msg, Some(vec![1, 2, 3]));
    }
}
