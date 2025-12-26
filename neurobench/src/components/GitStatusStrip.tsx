import { createSignal, onMount, onCleanup, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

// Types matching Rust backend
interface RepoStatus {
  is_repo: boolean;
  branch: string | null;
  clean: boolean;
  staged: number;
  modified: number;
  untracked: number;
  conflicts: number;
  ahead: number;
  behind: number;
}

interface Props {
  projectPath: string;
  onCommit?: () => void;
}

export default function GitStatusStrip(props: Props) {
  const [status, setStatus] = createSignal<RepoStatus | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [showCommit, setShowCommit] = createSignal(false);
  const [commitMsg, setCommitMsg] = createSignal("");
  const [committing, setCommitting] = createSignal(false);
  
  let pollInterval: number | undefined;

  const refreshStatus = async () => {
    if (!props.projectPath) return;
    
    try {
      const result = await invoke<RepoStatus>("git_status_get", {
        path: props.projectPath
      });
      setStatus(result);
    } catch (err) {
      // Not a git repo or error - clear status
      setStatus(null);
    }
  };

  const doCommit = async () => {
    if (!commitMsg().trim()) return;
    
    setCommitting(true);
    try {
      // Stage all first
      await invoke("git_stage_all", { path: props.projectPath });
      
      // Commit
      await invoke("git_commit", {
        path: props.projectPath,
        message: commitMsg()
      });
      
      setCommitMsg("");
      setShowCommit(false);
      props.onCommit?.();
      await refreshStatus();
    } catch (err) {
      alert(`Commit failed: ${err}`);
    } finally {
      setCommitting(false);
    }
  };

  onMount(() => {
    refreshStatus();
    pollInterval = setInterval(refreshStatus, 5000) as unknown as number;
  });

  onCleanup(() => {
    if (pollInterval) clearInterval(pollInterval);
  });

  const hasChanges = () => {
    const s = status();
    if (!s) return false;
    return s.staged > 0 || s.modified > 0 || s.untracked > 0;
  };

  const changeCount = () => {
    const s = status();
    if (!s) return 0;
    return s.staged + s.modified + s.untracked;
  };

  return (
    <div class="git-status-strip">
      <Show when={status()?.is_repo} fallback={
        <span class="no-git">No Git</span>
      }>
        <div class="git-info">
          <span class="branch">⎇ {status()?.branch || 'main'}</span>
          
          <Show when={hasChanges()}>
            <span class="changes" onClick={() => setShowCommit(!showCommit())}>
              {changeCount()} changes
            </span>
          </Show>
          
          <Show when={status()?.clean}>
            <span class="clean">✓</span>
          </Show>
          
          <Show when={(status()?.ahead || 0) > 0}>
            <span class="ahead">↑{status()?.ahead}</span>
          </Show>
          
          <Show when={(status()?.behind || 0) > 0}>
            <span class="behind">↓{status()?.behind}</span>
          </Show>
        </div>
        
        <Show when={showCommit()}>
          <div class="commit-dialog">
            <input
              type="text"
              placeholder="Commit message"
              value={commitMsg()}
              onInput={(e) => setCommitMsg(e.currentTarget.value)}
              onKeyDown={(e) => e.key === 'Enter' && doCommit()}
            />
            <button 
              onClick={doCommit} 
              disabled={committing() || !commitMsg().trim()}
            >
              {committing() ? '...' : 'Commit'}
            </button>
          </div>
        </Show>
      </Show>

      <style>{`
        .git-status-strip {
          display: flex;
          flex-direction: column;
          gap: 8px;
          padding: 8px 12px;
          background: #252526;
          border-radius: 4px;
          font-size: 13px;
        }
        
        .git-info {
          display: flex;
          align-items: center;
          gap: 12px;
        }
        
        .branch {
          color: #569cd6;
          font-weight: 500;
        }
        
        .changes {
          color: #dcdcaa;
          cursor: pointer;
        }
        
        .changes:hover {
          text-decoration: underline;
        }
        
        .clean {
          color: #4ec9b0;
        }
        
        .ahead {
          color: #4CAF50;
        }
        
        .behind {
          color: #f44336;
        }
        
        .no-git {
          color: #666;
          font-style: italic;
        }
        
        .commit-dialog {
          display: flex;
          gap: 8px;
          margin-top: 8px;
        }
        
        .commit-dialog input {
          flex: 1;
          padding: 6px 10px;
          background: #3c3c3c;
          border: 1px solid #444;
          border-radius: 4px;
          color: #e0e0e0;
          font-size: 13px;
        }
        
        .commit-dialog button {
          padding: 6px 16px;
          background: #0e639c;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        }
        
        .commit-dialog button:disabled {
          background: #333;
          cursor: not-allowed;
        }
      `}</style>
    </div>
  );
}
