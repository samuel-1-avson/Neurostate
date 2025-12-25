import { createSignal, For, Show, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

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
      case "new": return "➕";
      case "modified": return "✏️";
      case "deleted": return "🗑️";
      case "untracked": return "❓";
      default: return "📄";
    }
  };

  return (
    <div class="git-panel">
      <div class="git-header">
        <h3>🔀 Version Control</h3>
        <button class="refresh-btn" onClick={refreshStatus} disabled={isLoading()}>
          {isLoading() ? "⏳" : "🔄"}
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
          <span class="branch-icon">🌿</span>
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
                ✅ Working tree clean
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

      <style>{`
        .git-panel {
          background: var(--bg-secondary, #1a1a2e);
          border: 1px solid var(--border, #333);
          border-radius: 8px;
          padding: 12px;
        }

        .git-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 12px;
        }

        .git-header h3 {
          margin: 0;
          font-size: 14px;
        }

        .refresh-btn {
          background: transparent;
          border: none;
          cursor: pointer;
          font-size: 16px;
          padding: 4px 8px;
        }

        .refresh-btn:hover {
          background: rgba(255,255,255,0.1);
          border-radius: 4px;
        }

        .no-repo {
          text-align: center;
          padding: 20px;
        }

        .init-btn {
          background: linear-gradient(135deg, #3b82f6, #2563eb);
          color: white;
          border: none;
          padding: 10px 20px;
          border-radius: 6px;
          cursor: pointer;
          font-weight: 600;
          margin-top: 10px;
        }

        .branch-info {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 8px 12px;
          background: rgba(74, 222, 128, 0.1);
          border-radius: 6px;
          margin-bottom: 12px;
          font-size: 12px;
        }

        .branch-name {
          font-weight: 600;
          color: #4ade80;
        }

        .status-summary {
          color: #888;
          margin-left: auto;
        }

        .git-tabs {
          display: flex;
          gap: 4px;
          margin-bottom: 12px;
        }

        .tab {
          flex: 1;
          padding: 8px;
          background: transparent;
          border: 1px solid #333;
          color: #888;
          cursor: pointer;
          border-radius: 6px;
          font-size: 12px;
        }

        .tab.active {
          background: linear-gradient(135deg, #3b82f6, #2563eb);
          color: white;
          border-color: #3b82f6;
        }

        .file-group {
          margin-bottom: 12px;
        }

        .group-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          font-size: 11px;
          color: #888;
          margin-bottom: 6px;
          text-transform: uppercase;
          letter-spacing: 0.5px;
        }

        .stage-all-btn {
          background: rgba(74, 222, 128, 0.2);
          border: 1px solid rgba(74, 222, 128, 0.3);
          color: #4ade80;
          padding: 4px 10px;
          border-radius: 4px;
          cursor: pointer;
          font-size: 11px;
        }

        .file-item {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 6px 10px;
          background: rgba(255,255,255,0.03);
          border-radius: 4px;
          margin-bottom: 4px;
          font-size: 12px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .file-item.unstaged:hover {
          background: rgba(74, 222, 128, 0.1);
        }

        .file-item.staged {
          border-left: 2px solid #4ade80;
        }

        .status-icon {
          font-size: 14px;
        }

        .file-path {
          flex: 1;
          color: #ccc;
          font-family: 'Fira Code', monospace;
        }

        .stage-hint {
          color: #666;
          font-size: 10px;
        }

        .no-changes {
          text-align: center;
          padding: 20px;
          color: #4ade80;
        }

        .commit-form {
          display: flex;
          gap: 8px;
          margin-top: 12px;
        }

        .commit-form input {
          flex: 1;
          background: rgba(255,255,255,0.05);
          border: 1px solid #333;
          color: #fff;
          padding: 10px 12px;
          border-radius: 6px;
          font-size: 13px;
        }

        .commit-btn {
          background: linear-gradient(135deg, #4ade80, #22c55e);
          color: #000;
          border: none;
          padding: 10px 20px;
          border-radius: 6px;
          cursor: pointer;
          font-weight: 600;
        }

        .commit-btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .history-section {
          max-height: 300px;
          overflow-y: auto;
        }

        .commit-item {
          padding: 10px;
          background: rgba(255,255,255,0.03);
          border-radius: 6px;
          margin-bottom: 8px;
          border-left: 2px solid #3b82f6;
        }

        .commit-header {
          display: flex;
          justify-content: space-between;
          margin-bottom: 4px;
        }

        .commit-id {
          font-family: 'Fira Code', monospace;
          color: #60a5fa;
          font-size: 11px;
        }

        .commit-time {
          color: #666;
          font-size: 11px;
        }

        .commit-message {
          color: #fff;
          font-size: 13px;
          margin-bottom: 4px;
        }

        .commit-author {
          color: #888;
          font-size: 11px;
        }

        .no-history {
          text-align: center;
          padding: 20px;
          color: #666;
        }
      `}</style>
    </div>
  );
}
