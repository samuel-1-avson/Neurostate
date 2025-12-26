// Scheduler Panel - Job queue management (SolidJS)
import { createSignal, onMount, onCleanup, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./SchedulerPanel.css";

type JobPriority = "low" | "normal" | "high" | "critical";
type JobKind = "build" | "flash" | "rtt" | "debug" | "agent" | "index";

interface SchedulerStatus {
  pending_count: number;
  running_count: number;
  completed_count: number;
  failed_count: number;
  running_jobs: string[];
}

export function SchedulerPanel() {
  const [status, setStatus] = createSignal<SchedulerStatus>({
    pending_count: 0,
    running_count: 0,
    completed_count: 0,
    failed_count: 0,
    running_jobs: [],
  });
  const [isLoading, setIsLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [newJobKind, setNewJobKind] = createSignal<JobKind>("build");
  const [newJobPriority, setNewJobPriority] = createSignal<JobPriority>("normal");
  const [dependsOn, setDependsOn] = createSignal("");

  let pollInterval: ReturnType<typeof setInterval>;

  const fetchStatus = async () => {
    setIsLoading(true);
    try {
      const result = await invoke<SchedulerStatus>("scheduler_get_status");
      setStatus(result);
    } catch (err) {
      setError(err as string);
    } finally {
      setIsLoading(false);
    }
  };

  const scheduleJob = async (
    kind: JobKind,
    priority: JobPriority,
    deps?: string[]
  ) => {
    try {
      await invoke("schedule_priority_job", {
        kind,
        priority,
        depends_on: deps,
        payload: null,
      });
      await fetchStatus();
    } catch (err) {
      setError(err as string);
    }
  };

  const handleSchedule = async () => {
    const deps = dependsOn() ? dependsOn().split(",").map((s) => s.trim()) : undefined;
    await scheduleJob(newJobKind(), newJobPriority(), deps);
    setDependsOn("");
  };

  const handleBuildAndFlash = async () => {
    const buildResult = await invoke<{ job_id: string }>("schedule_priority_job", {
      kind: "build",
      priority: "high",
      depends_on: null,
      payload: null,
    });
    await invoke("schedule_priority_job", {
      kind: "flash",
      priority: "high",
      depends_on: [buildResult.job_id],
      payload: null,
    });
    await fetchStatus();
  };

  const setLimit = async (kind: JobKind, limit: number) => {
    try {
      await invoke("scheduler_set_limit", { kind, limit });
    } catch (err) {
      console.error(err);
    }
  };

  onMount(() => {
    fetchStatus();
    pollInterval = setInterval(fetchStatus, 5000);
  });

  onCleanup(() => {
    clearInterval(pollInterval);
  });

  return (
    <div class="scheduler-panel">
      <div class="scheduler-header">
        <h3>⚡ Job Scheduler</h3>
        <button onClick={fetchStatus} disabled={isLoading()}>
          🔄 Refresh
        </button>
      </div>

      <Show when={error()}>
        <div class="scheduler-error">{error()}</div>
      </Show>

      {/* Status Overview */}
      <div class="status-overview">
        <div class="status-card pending">
          <div class="status-value">{status().pending_count}</div>
          <div class="status-label">Pending</div>
        </div>
        <div class="status-card running">
          <div class="status-value">{status().running_count}</div>
          <div class="status-label">Running</div>
        </div>
        <div class="status-card completed">
          <div class="status-value">{status().completed_count}</div>
          <div class="status-label">Completed</div>
        </div>
        <div class="status-card failed">
          <div class="status-value">{status().failed_count}</div>
          <div class="status-label">Failed</div>
        </div>
      </div>

      {/* Quick Actions */}
      <div class="quick-actions">
        <h4>🚀 Quick Actions</h4>
        <div class="action-buttons">
          <button class="action-btn build" onClick={() => scheduleJob("build", "high")}>
            🔨 Build
          </button>
          <button class="action-btn flash" onClick={() => scheduleJob("flash", "high")}>
            ⚡ Flash
          </button>
          <button class="action-btn chain" onClick={handleBuildAndFlash}>
            🔗 Build + Flash
          </button>
        </div>
      </div>

      {/* Schedule New Job */}
      <div class="new-job-section">
        <h4>➕ Schedule Job</h4>
        <div class="job-form">
          <div class="form-row">
            <label>Kind:</label>
            <select
              value={newJobKind()}
              onChange={(e) => setNewJobKind(e.currentTarget.value as JobKind)}
            >
              <option value="build">Build</option>
              <option value="flash">Flash</option>
              <option value="rtt">RTT</option>
              <option value="debug">Debug</option>
              <option value="agent">Agent</option>
              <option value="index">Index</option>
            </select>
          </div>
          <div class="form-row">
            <label>Priority:</label>
            <select
              value={newJobPriority()}
              onChange={(e) => setNewJobPriority(e.currentTarget.value as JobPriority)}
            >
              <option value="low">🔵 Low</option>
              <option value="normal">🟢 Normal</option>
              <option value="high">🟡 High</option>
              <option value="critical">🔴 Critical</option>
            </select>
          </div>
          <div class="form-row">
            <label>Depends On:</label>
            <input
              type="text"
              placeholder="job_id1, job_id2..."
              value={dependsOn()}
              onInput={(e) => setDependsOn(e.currentTarget.value)}
            />
          </div>
          <button class="schedule-btn" onClick={handleSchedule}>
            Schedule Job
          </button>
        </div>
      </div>

      {/* Running Jobs */}
      <div class="running-jobs">
        <h4>🏃 Running Jobs</h4>
        <Show when={status().running_jobs.length === 0}>
          <div class="empty">No jobs running</div>
        </Show>
        <div class="job-list">
          <For each={status().running_jobs}>
            {(jobId) => (
              <div class="job-item">
                <span class="job-id">{jobId}</span>
                <div class="job-progress">
                  <div class="progress-bar" />
                </div>
              </div>
            )}
          </For>
        </div>
      </div>

      {/* Concurrency Settings */}
      <div class="concurrency-section">
        <h4>⚙️ Concurrency Limits</h4>
        <div class="limit-grid">
          <For each={["build", "flash", "agent"] as JobKind[]}>
            {(kind) => (
              <div class="limit-item">
                <span class="limit-label">{kind}</span>
                <input
                  type="number"
                  value={kind === "agent" ? 3 : 1}
                  min={1}
                  max={10}
                  onChange={(e) => setLimit(kind, Number(e.currentTarget.value))}
                />
              </div>
            )}
          </For>
        </div>
      </div>
    </div>
  );
}
