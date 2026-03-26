import { createSignal, For, Show, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { Icons } from "./AppIcons";
import "./GitPanel.css";

interface FileStatus {
  path: string;
  status: string;
  staged: boolean;
}

interface CommitInfo {
  id: string;
  message: string;
  author: string;
  email: string;
  time: number;
  short_id: string;
}

interface RepoStatus {
  is_repo: boolean;
  branch: string | null;
  files: FileStatus[];
  staged_count: number;
  modified_count: number;
  untracked_count: number;
}

interface GitPanelProps {
  projectPath: string;
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function GitPanel(props: GitPanelProps) {
  const [status, setStatus] = createSignal<RepoStatus | null>(null);
  const [history, setHistory] = createSignal<CommitInfo[]>([]);
  const [commitMessage, setCommitMessage] = createSignal("");
  const [authorName, setAuthorName] = createSignal("NeuroBench User");
  const [authorEmail, setAuthorEmail] = createSignal("user@neurobench.dev");
  const [activeTab, setActiveTab] = createSignal<"changes" | "history">("changes");
  const [isLoading, setIsLoading] = createSignal(false);

  const addLog = (source: string, message: string, type: "info" | "success" | "warning" | "error" = "info") => {
    props.onLog?.(source, message, type);
  };

  const refreshStatus = async () => {
    if (!props.projectPath) return;
    
    setIsLoading(true);
    try {
      const result = await invoke("git_status", { path: props.projectPath }) as RepoStatus;
      setStatus(result);
    } catch (e) {
      addLog("Git", `Status error: ${e}`, "error");
    }
    setIsLoading(false);
  };

  const loadHistory = async () => {
    if (!props.projectPath) return;
    
    try {
      const result = await invoke("git_history", { 
        path: props.projectPath, 
        limit: 20 
      }) as CommitInfo[];
      setHistory(result);
    } catch (e) {
      // No commits yet
      setHistory([]);
    }
  };

  const initRepo = async () => {
    try {
      await invoke("git_init", { path: props.projectPath });
      addLog("Git", "Initialized new repository", "success");
      await refreshStatus();
    } catch (e) {
      addLog("Git", `Init failed: ${e}`, "error");
    }
  };

  const stageAll = async () => {
    try {
      await invoke("git_stage_all", { path: props.projectPath });
      addLog("Git", "Staged all changes", "info");
      await refreshStatus();
    } catch (e) {
      addLog("Git", `Stage failed: ${e}`, "error");
    }
  };

  const stageFile = async (file: string) => {
    try {
      await invoke("git_stage_files", { 
        path: props.projectPath, 
        files: [file] 
      });
      addLog("Git", `Staged: ${file}`, "info");
      await refreshStatus();
    } catch (e) {
      addLog("Git", `Stage failed: ${e}`, "error");
    }
  };

  const doCommit = async () => {
    if (!commitMessage().trim()) {
      addLog("Git", "Commit message required", "warning");
      return;
    }

    try {
      const result = await invoke("git_commit", {
        path: props.projectPath,
        message: commitMessage(),
        authorName: authorName(),
        authorEmail: authorEmail(),
      }) as CommitInfo;
      
      addLog("Git", `Committed: ${result.short_id} - ${result.message}`, "success");
      setCommitMessage("");
      await refreshStatus();
      await loadHistory();
    } catch (e) {
      addLog("Git", `Commit failed: ${e}`, "error");
    }
  };

  onMount(() => {
    if (props.projectPath) {
      refreshStatus();
      loadHistory();
    }
  });

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString() + " " + date.toLocaleTimeString();
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case "new": return <span class="status-svg">{Icons.plus()}</span>;
      case "modified": return <span class="status-svg">{Icons.edit()}</span>;
      case "deleted": return <span class="status-svg">{Icons.trash()}</span>;
      case "untracked": return <span class="status-svg">{Icons.question()}</span>;
      default: return <span class="status-svg">{Icons.file()}</span>;
    }
  };

  return (
    <div class="git-panel">
      <div class="git-header">
        <h3><span class="header-icon">{Icons.gitBranch()}</span> Version Control</h3>
        <button class="refresh-btn" onClick={refreshStatus} disabled={isLoading()}>
          {isLoading() ? <span class="spinning">{Icons.refresh()}</span> : Icons.refresh()}
        </button>
      </div>

      {/* Not a repo yet */}
      <Show when={status() && !status()!.is_repo}>
        <div class="no-repo">
          <p>No Git repository found.</p>
          <button class="init-btn" onClick={initRepo}>
            Initialize Repository
          </button>
        </div>
      </Show>

      {/* Repo exists */}
      <Show when={status()?.is_repo}>
        <div class="branch-info">
          <span class="branch-icon">{Icons.gitBranch()}</span>
          <span class="branch-name">{status()?.branch || "main"}</span>
          <span class="status-summary">
            {status()?.staged_count} staged, {status()?.modified_count} modified, {status()?.untracked_count} untracked
          </span>
        </div>

        {/* Tabs */}
        <div class="git-tabs">
          <button 
            class={`tab ${activeTab() === "changes" ? "active" : ""}`}
            onClick={() => setActiveTab("changes")}
          >
            Changes
          </button>
          <button 
            class={`tab ${activeTab() === "history" ? "active" : ""}`}
            onClick={() => { setActiveTab("history"); loadHistory(); }}
          >
            History
          </button>
        </div>

        {/* Changes Tab */}
        <Show when={activeTab() === "changes"}>
          <div class="changes-section">
            {/* Staged files */}
            <Show when={status()?.files.some(f => f.staged)}>
              <div class="file-group staged">
                <div class="group-header">
                  <span>Staged Changes</span>
                </div>
                <For each={status()?.files.filter(f => f.staged)}>
                  {(file) => (
                    <div class="file-item staged">
                      <span class="status-icon">{getStatusIcon(file.status)}</span>
                      <span class="file-path">{file.path}</span>
                    </div>
                  )}
                </For>
              </div>
            </Show>

            {/* Unstaged files */}
            <Show when={status()?.files.some(f => !f.staged)}>
              <div class="file-group unstaged">
                <div class="group-header">
                  <span>Unstaged Changes</span>
                  <button class="stage-all-btn" onClick={stageAll}>Stage All</button>
                </div>
                <For each={status()?.files.filter(f => !f.staged)}>
                  {(file) => (
                    <div class="file-item unstaged" onClick={() => stageFile(file.path)}>
                      <span class="status-icon">{getStatusIcon(file.status)}</span>
                      <span class="file-path">{file.path}</span>
                      <span class="stage-hint">Click to stage</span>
                    </div>
                  )}
                </For>
              </div>
            </Show>

            {/* No changes */}
            <Show when={status()?.files.length === 0}>
              <div class="no-changes">
                <span class="clean-icon">{Icons.checkCircle()}</span> Working tree clean
              </div>
            </Show>

            {/* Commit form */}
            <Show when={(status()?.staged_count ?? 0) > 0}>
              <div class="commit-form">
                <input 
                  type="text" 
                  placeholder="Commit message..."
                  value={commitMessage()}
                  onInput={(e) => setCommitMessage(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && doCommit()}
                />
                <button 
                  class="commit-btn"
                  onClick={doCommit}
                  disabled={!commitMessage().trim()}
                >
                  Commit
                </button>
              </div>
            </Show>
          </div>
        </Show>

        {/* History Tab */}
        <Show when={activeTab() === "history"}>
          <div class="history-section">
            <Show when={history().length === 0}>
              <div class="no-history">No commits yet</div>
            </Show>
            <For each={history()}>
              {(commit) => (
                <div class="commit-item">
                  <div class="commit-header">
                    <span class="commit-id">{commit.short_id}</span>
                    <span class="commit-time">{formatTime(commit.time)}</span>
                  </div>
                  <div class="commit-message">{commit.message}</div>
                  <div class="commit-author">{commit.author}</div>
                </div>
              )}
            </For>
          </div>
        </Show>
      </Show>


    </div>
  );
}
