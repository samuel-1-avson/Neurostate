//! Job Scheduler - Priority queue, dependencies, and cron-like scheduling
//!
//! Extends the job system with advanced scheduling capabilities.

use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

use super::{JobId, JobKind};

// =============================================================================
// PRIORITY
// =============================================================================

/// Job priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum JobPriority {
    /// Background tasks (indexing, cleanup)
    Low = 0,
    /// Normal user-initiated tasks
    Normal = 1,
    /// User is waiting (build, flash)
    High = 2,
    /// Critical system tasks
    Critical = 3,
}

impl Default for JobPriority {
    fn default() -> Self {
        Self::Normal
    }
}

// =============================================================================
// SCHEDULED JOB
// =============================================================================

/// A job with scheduling metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    /// Job ID
    pub id: JobId,
    /// Job kind
    pub kind: JobKind,
    /// Priority
    pub priority: JobPriority,
    /// Dependencies (must complete before this runs)
    pub depends_on: Vec<JobId>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Scheduled run time (None = immediate)
    pub scheduled_at: Option<DateTime<Utc>>,
    /// Retry count
    pub retry_count: u32,
    /// Max retries
    pub max_retries: u32,
    /// Payload (serialized job data)
    pub payload: serde_json::Value,
}

impl ScheduledJob {
    pub fn new(id: JobId, kind: JobKind) -> Self {
        Self {
            id,
            kind,
            priority: JobPriority::Normal,
            depends_on: Vec::new(),
            created_at: Utc::now(),
            scheduled_at: None,
            retry_count: 0,
            max_retries: 3,
            payload: serde_json::Value::Null,
        }
    }

    pub fn with_priority(mut self, priority: JobPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn depends_on(mut self, job_id: JobId) -> Self {
        self.depends_on.push(job_id);
        self
    }

    pub fn scheduled_at(mut self, time: DateTime<Utc>) -> Self {
        self.scheduled_at = Some(time);
        self
    }

    pub fn delayed(mut self, delay: Duration) -> Self {
        self.scheduled_at = Some(Utc::now() + delay);
        self
    }

    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = payload;
        self
    }

    /// Is job ready to run?
    pub fn is_ready(&self, completed_jobs: &HashSet<JobId>) -> bool {
        // Check schedule time
        if let Some(scheduled) = self.scheduled_at {
            if Utc::now() < scheduled {
                return false;
            }
        }

        // Check dependencies
        self.depends_on.iter().all(|dep| completed_jobs.contains(dep))
    }
}

/// Wrapper for priority queue ordering
#[derive(Debug)]
struct PrioritizedJob(ScheduledJob);

impl PartialEq for PrioritizedJob {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}

impl Eq for PrioritizedJob {}

impl PartialOrd for PrioritizedJob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedJob {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first (BinaryHeap is max-heap, High=2 > Low=0)
        match self.0.priority.cmp(&other.0.priority) {
            Ordering::Equal => {
                // Earlier created first (FIFO for same priority)
                other.0.created_at.cmp(&self.0.created_at)
            }
            ord => ord, // No reverse needed - max-heap extracts highest first
        }
    }
}

// =============================================================================
// RESOURCE LOCK
// =============================================================================

/// Resource that requires exclusive access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resource {
    /// Debug probe device
    Probe,
    /// USB device
    Usb,
    /// Serial port
    Serial,
    /// Filesystem (for builds)
    BuildDir,
}

/// Resource lock manager
#[derive(Debug, Default)]
pub struct ResourceLocks {
    locks: HashMap<Resource, JobId>,
}

impl ResourceLocks {
    pub fn new() -> Self {
        Self::default()
    }

    /// Try to acquire a lock
    pub fn try_acquire(&mut self, resource: Resource, job_id: &JobId) -> bool {
        if self.locks.contains_key(&resource) {
            false
        } else {
            self.locks.insert(resource, job_id.clone());
            true
        }
    }

    /// Release a lock
    pub fn release(&mut self, resource: Resource, job_id: &JobId) {
        if self.locks.get(&resource) == Some(job_id) {
            self.locks.remove(&resource);
        }
    }

    /// Release all locks for a job
    pub fn release_all(&mut self, job_id: &JobId) {
        self.locks.retain(|_, id| id != job_id);
    }

    /// Check if resource is locked
    pub fn is_locked(&self, resource: Resource) -> bool {
        self.locks.contains_key(&resource)
    }

    /// Get lock holder
    pub fn locked_by(&self, resource: Resource) -> Option<&JobId> {
        self.locks.get(&resource)
    }
}

// =============================================================================
// SCHEDULER
// =============================================================================

/// Job scheduler with priority queue and dependencies
pub struct JobScheduler {
    /// Pending jobs (priority queue)
    pending: BinaryHeap<PrioritizedJob>,
    /// Running jobs
    running: HashMap<JobId, ScheduledJob>,
    /// Completed job IDs
    completed: HashSet<JobId>,
    /// Failed job IDs
    failed: HashSet<JobId>,
    /// Resource locks
    resources: ResourceLocks,
    /// Max concurrent jobs per kind
    concurrency_limits: HashMap<JobKind, usize>,
}

impl JobScheduler {
    pub fn new() -> Self {
        let mut limits = HashMap::new();
        // Default limits
        limits.insert(JobKind::Build, 1);  // One build at a time
        limits.insert(JobKind::Flash, 1);  // One flash at a time
        limits.insert(JobKind::Debug, 1);  // One debug session
        limits.insert(JobKind::Rtt, 1);    // One RTT session
        limits.insert(JobKind::Agent, 3);  // Multiple AI agents
        limits.insert(JobKind::Index, 2);  // Background indexing

        Self {
            pending: BinaryHeap::new(),
            running: HashMap::new(),
            completed: HashSet::new(),
            failed: HashSet::new(),
            resources: ResourceLocks::new(),
            concurrency_limits: limits,
        }
    }

    /// Schedule a new job
    pub fn schedule(&mut self, job: ScheduledJob) -> JobId {
        let id = job.id.clone();
        self.pending.push(PrioritizedJob(job));
        id
    }

    /// Get next ready job
    pub fn next_ready(&mut self) -> Option<ScheduledJob> {
        // Check concurrency limits
        let mut running_counts: HashMap<JobKind, usize> = HashMap::new();
        for job in self.running.values() {
            *running_counts.entry(job.kind).or_default() += 1;
        }

        // Find first ready job that can run
        let mut temp = Vec::new();
        let mut result = None;

        while let Some(pj) = self.pending.pop() {
            let job = pj.0;
            
            // Check if ready
            if !job.is_ready(&self.completed) {
                temp.push(PrioritizedJob(job));
                continue;
            }

            // Check concurrency limit
            let count = running_counts.get(&job.kind).copied().unwrap_or(0);
            let limit = self.concurrency_limits.get(&job.kind).copied().unwrap_or(10);
            
            if count >= limit {
                temp.push(PrioritizedJob(job));
                continue;
            }

            // Check resource requirements
            let resources_available = match job.kind {
                JobKind::Flash | JobKind::Debug | JobKind::Rtt => {
                    !self.resources.is_locked(Resource::Probe)
                }
                _ => true,
            };

            if !resources_available {
                temp.push(PrioritizedJob(job));
                continue;
            }

            // This job can run
            result = Some(job);
            break;
        }

        // Put back jobs that couldn't run
        for pj in temp {
            self.pending.push(pj);
        }

        // Mark job as running
        if let Some(ref job) = result {
            self.running.insert(job.id.clone(), job.clone());
            
            // Acquire resources
            if matches!(job.kind, JobKind::Flash | JobKind::Debug | JobKind::Rtt) {
                self.resources.try_acquire(Resource::Probe, &job.id);
            }
        }

        result
    }

    /// Mark job as completed
    pub fn complete(&mut self, job_id: JobId, success: bool) {
        if let Some(job) = self.running.remove(&job_id) {
            // Release resources
            self.resources.release_all(&job_id);

            if success {
                self.completed.insert(job_id);
            } else {
                // Check for retry
                if job.retry_count < job.max_retries {
                    let retry = ScheduledJob {
                        retry_count: job.retry_count + 1,
                        scheduled_at: Some(Utc::now() + Duration::seconds(5)),
                        ..job
                    };
                    self.pending.push(PrioritizedJob(retry));
                } else {
                    self.failed.insert(job_id);
                }
            }
        }
    }

    /// Cancel a job
    pub fn cancel(&mut self, job_id: &JobId) {
        // Remove from running
        if self.running.remove(job_id).is_some() {
            self.resources.release_all(job_id);
        }

        // Remove from pending (expensive but rare)
        let pending: Vec<_> = std::mem::take(&mut self.pending).into_vec();
        for pj in pending {
            if &pj.0.id != job_id {
                self.pending.push(pj);
            }
        }
    }

    /// Get queue status
    pub fn status(&self) -> SchedulerStatus {
        SchedulerStatus {
            pending_count: self.pending.len(),
            running_count: self.running.len(),
            completed_count: self.completed.len(),
            failed_count: self.failed.len(),
            running_jobs: self.running.keys().cloned().collect(),
        }
    }

    /// Set concurrency limit for a job kind
    pub fn set_limit(&mut self, kind: JobKind, limit: usize) {
        self.concurrency_limits.insert(kind, limit);
    }
}

impl Default for JobScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStatus {
    pub pending_count: usize,
    pub running_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
    pub running_jobs: Vec<JobId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        let mut scheduler = JobScheduler::new();
        
        let low = ScheduledJob::new("job_1".to_string(), JobKind::Index).with_priority(JobPriority::Low);
        let high = ScheduledJob::new("job_2".to_string(), JobKind::Build).with_priority(JobPriority::High);
        let normal = ScheduledJob::new("job_3".to_string(), JobKind::Agent).with_priority(JobPriority::Normal);

        scheduler.schedule(low);
        scheduler.schedule(normal);
        scheduler.schedule(high);

        // High priority should come first
        let next = scheduler.next_ready().unwrap();
        assert_eq!(next.id, "job_2");
    }

    #[test]
    fn test_dependencies() {
        let mut scheduler = JobScheduler::new();
        
        let first = ScheduledJob::new("job_1".to_string(), JobKind::Build);
        let second = ScheduledJob::new("job_2".to_string(), JobKind::Flash).depends_on("job_1".to_string());

        scheduler.schedule(first);
        scheduler.schedule(second);

        // First job should be ready
        let next = scheduler.next_ready().unwrap();
        assert_eq!(next.id, "job_1");

        // Second not ready yet
        assert!(scheduler.next_ready().is_none());

        // Complete first
        scheduler.complete("job_1".to_string(), true);

        // Now second is ready
        let next = scheduler.next_ready().unwrap();
        assert_eq!(next.id, "job_2");
    }

    #[test]
    fn test_resource_locks() {
        let mut locks = ResourceLocks::new();
        
        assert!(locks.try_acquire(Resource::Probe, &"1".to_string()));
        assert!(!locks.try_acquire(Resource::Probe, &"2".to_string()));
        
        locks.release(Resource::Probe, &"1".to_string());
        assert!(locks.try_acquire(Resource::Probe, &"2".to_string()));
    }
}
