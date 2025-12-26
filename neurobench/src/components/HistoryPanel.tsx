// History Panel - Event store visualization and time travel (SolidJS)
import { createSignal, onMount, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./HistoryPanel.css";

interface EventHistoryItem {
  sequence: number;
  timestamp: string;
  domain: unknown;
  author: string;
}

interface DiffResult {
  from_sequence: number;
  to_sequence: number;
  changes: unknown[];
  summary: {
    total_changes: number;
    nodes_added: number;
    nodes_removed: number;
    breaking_changes: number;
  };
}

export function HistoryPanel() {
  const [events, setEvents] = createSignal<EventHistoryItem[]>([]);
  const [currentSequence, setCurrentSequence] = createSignal(0);
  const [isLoading, setIsLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [selectedEvent, setSelectedEvent] = createSignal<number | null>(null);
  const [diffFrom, setDiffFrom] = createSignal(0);
  const [diffTo, setDiffTo] = createSignal(0);
  const [diffResult, setDiffResult] = createSignal<DiffResult | null>(null);
  const [snapshotName, setSnapshotName] = createSignal("");

  const fetchHistory = async (sinceSequence?: number, domain?: string) => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<{
        events: EventHistoryItem[];
        current_sequence: number;
      }>("canvas_get_event_history", {
        since_sequence: sinceSequence,
        domain,
      });
      setEvents(result.events);
      setCurrentSequence(result.current_sequence);
    } catch (err) {
      setError(err as string);
    } finally {
      setIsLoading(false);
    }
  };

  const handleCreateSnapshot = async () => {
    try {
      await invoke("canvas_create_snapshot", { name: snapshotName() || null });
      setSnapshotName("");
      alert("Snapshot created!");
    } catch (err) {
      setError(err as string);
    }
  };

  const handleDiff = async () => {
    try {
      const result = await invoke<DiffResult>("canvas_diff_versions", {
        from_sequence: diffFrom(),
        to_sequence: diffTo(),
      });
      setDiffResult(result);
    } catch (err) {
      setError(err as string);
    }
  };

  const handleRestore = async (sequence: number) => {
    if (confirm(`Restore to version ${sequence}?`)) {
      try {
        await invoke("canvas_restore_version", { sequence });
        await fetchHistory();
      } catch (err) {
        setError(err as string);
      }
    }
  };

  const getEventIcon = (eventType: string) => {
    const icons: Record<string, string> = {
      node_added: "➕",
      node_removed: "➖",
      node_moved: "↔️",
      edge_added: "🔗",
      edge_removed: "✂️",
      property_changed: "✏️",
      state_changed: "🔄",
    };
    return icons[eventType] || "📝";
  };

  onMount(() => {
    fetchHistory();
  });

  return (
    <div class="history-panel">
      <div class="history-header">
        <h3>📜 History</h3>
        <button onClick={() => fetchHistory()} disabled={isLoading()}>
          🔄 Refresh
        </button>
      </div>

      <Show when={error()}>
        <div class="history-error">{error()}</div>
      </Show>

      {/* Snapshot Section */}
      <div class="snapshot-section">
        <h4>📸 Create Snapshot</h4>
        <div class="snapshot-form">
          <input
            type="text"
            placeholder="Snapshot name (optional)"
            value={snapshotName()}
            onInput={(e) => setSnapshotName(e.currentTarget.value)}
          />
          <button onClick={handleCreateSnapshot}>Create</button>
        </div>
      </div>

      {/* Diff Section */}
      <div class="diff-section">
        <h4>🔍 Compare Versions</h4>
        <div class="diff-form">
          <input
            type="number"
            placeholder="From"
            value={diffFrom()}
            onInput={(e) => setDiffFrom(Number(e.currentTarget.value))}
          />
          <span>→</span>
          <input
            type="number"
            placeholder="To"
            value={diffTo()}
            onInput={(e) => setDiffTo(Number(e.currentTarget.value))}
          />
          <button onClick={handleDiff}>Diff</button>
        </div>
        <Show when={diffResult()}>
          <div class="diff-result">
            <pre>{JSON.stringify(diffResult(), null, 2)}</pre>
          </div>
        </Show>
      </div>

      {/* Event List */}
      <div class="event-list">
        <h4>📋 Events (Current: {currentSequence()})</h4>
        <Show when={isLoading()}>
          <div class="loading">Loading...</div>
        </Show>
        <Show when={!isLoading() && events().length === 0}>
          <div class="empty">No events yet</div>
        </Show>
        <div class="events">
          <For each={events()}>
            {(event) => (
              <div
                class={`event-item ${selectedEvent() === event.sequence ? "selected" : ""}`}
                onClick={() => setSelectedEvent(event.sequence)}
              >
                <span class="event-icon">{getEventIcon("state_changed")}</span>
                <div class="event-details">
                  <div class="event-type">Event #{event.sequence}</div>
                  <div class="event-meta">
                    <span class="event-seq">{event.timestamp}</span>
                    <span class="event-domain">{event.author}</span>
                  </div>
                </div>
                <button
                  class="restore-btn"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleRestore(event.sequence);
                  }}
                  title="Restore to this version"
                >
                  ⏪
                </button>
              </div>
            )}
          </For>
        </div>
      </div>
    </div>
  );
}
