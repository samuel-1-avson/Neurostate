import { createSignal, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface PerformanceIssue {
  severity: string;
  category: string;
  message: string;
  line: number | null;
  suggestion: string;
}

interface CodeMetrics {
  lines_of_code: number;
  functions: number;
  loops: number;
  conditionals: number;
  cyclomatic_complexity: number;
  max_nesting_depth: number;
  estimated_stack_usage: number;
}

interface ProfilingResult {
  metrics: CodeMetrics;
  issues: PerformanceIssue[];
  optimization_score: number;
  suggestions: string[];
}

interface ProfilerPanelProps {
  code?: string;
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function ProfilerPanel(props: ProfilerPanelProps) {
  const [result, setResult] = createSignal<ProfilingResult | null>(null);
  const [mcuFreq, setMcuFreq] = createSignal(168);
  const [isLoading, setIsLoading] = createSignal(false);

  const analyzeCode = async () => {
    const code = props.code || "";
    if (!code.trim()) {
      props.onLog?.("Profiler", "No code to analyze", "warning");
      return;
    }

    setIsLoading(true);
    try {
      const analysis = await invoke("profiler_analyze", {
        code,
        mcuFreqMhz: mcuFreq(),
      }) as ProfilingResult;
      
      setResult(analysis);
      props.onLog?.("Profiler", `Score: ${analysis.optimization_score}/100`, "info");
    } catch (e) {
      props.onLog?.("Profiler", `Analysis failed: ${e}`, "error");
    }
    setIsLoading(false);
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case "critical": return "#ef4444";
      case "warning": return "#fbbf24";
      case "info": return "#60a5fa";
      default: return "#888";
    }
  };

  const getScoreColor = (score: number) => {
    if (score >= 80) return "#4ade80";
    if (score >= 60) return "#fbbf24";
    return "#ef4444";
  };

  return (
    <div class="profiler-panel">
      <div class="panel-header">
        <h3>📊 Performance Profiler</h3>
      </div>

      <div class="config-row">
        <label>MCU Frequency (MHz)</label>
        <input 
          type="number" 
          value={mcuFreq()}
          onInput={(e) => setMcuFreq(parseInt(e.target.value) || 168)}
        />
      </div>

      <button class="analyze-btn" onClick={analyzeCode} disabled={isLoading()}>
        {isLoading() ? "Analyzing..." : "Analyze Performance"}
      </button>

      <Show when={result()}>
        <div class="results">
          {/* Score */}
          <div class="score-display" style={{ "border-color": getScoreColor(result()!.optimization_score) }}>
            <span class="score-value" style={{ color: getScoreColor(result()!.optimization_score) }}>
              {result()!.optimization_score}
            </span>
            <span class="score-label">/ 100</span>
          </div>

          {/* Metrics */}
          <div class="metrics">
            <div class="metric">
              <span class="metric-value">{result()!.metrics.lines_of_code}</span>
              <span class="metric-label">Lines</span>
            </div>
            <div class="metric">
              <span class="metric-value">{result()!.metrics.functions}</span>
              <span class="metric-label">Functions</span>
            </div>
            <div class="metric">
              <span class="metric-value">{result()!.metrics.cyclomatic_complexity}</span>
              <span class="metric-label">Complexity</span>
            </div>
            <div class="metric">
              <span class="metric-value">{result()!.metrics.estimated_stack_usage}B</span>
              <span class="metric-label">Stack</span>
            </div>
          </div>

          {/* Issues */}
          <Show when={result()!.issues.length > 0}>
            <div class="issues">
              <h4>Issues Found</h4>
              <For each={result()!.issues}>
                {(issue) => (
                  <div class="issue-item" style={{ "border-left-color": getSeverityColor(issue.severity) }}>
                    <div class="issue-header">
                      <span class="issue-severity" style={{ color: getSeverityColor(issue.severity) }}>
                        {issue.severity.toUpperCase()}
                      </span>
                      <span class="issue-category">{issue.category}</span>
                    </div>
                    <p class="issue-message">{issue.message}</p>
                    <p class="issue-suggestion">💡 {issue.suggestion}</p>
                  </div>
                )}
              </For>
            </div>
          </Show>

          {/* Suggestions */}
          <div class="suggestions">
            <h4>Optimization Tips</h4>
            <For each={result()!.suggestions}>
              {(suggestion) => (
                <div class="suggestion-item">✓ {suggestion}</div>
              )}
            </For>
          </div>
        </div>
      </Show>

      <style>{`
        .profiler-panel {
          background: var(--bg-secondary, #1a1a2e);
          border: 1px solid var(--border, #333);
          border-radius: 8px;
          padding: 12px;
        }

        .panel-header { margin-bottom: 12px; }
        .panel-header h3 { margin: 0; font-size: 14px; }

        .config-row { margin-bottom: 12px; }
        .config-row label {
          display: block;
          color: #888;
          font-size: 11px;
          margin-bottom: 6px;
        }

        .config-row input {
          width: 100%;
          background: rgba(255,255,255,0.05);
          border: 1px solid #333;
          color: #fff;
          padding: 8px 12px;
          border-radius: 6px;
        }

        .analyze-btn {
          width: 100%;
          background: linear-gradient(135deg, #8b5cf6, #7c3aed);
          color: white;
          border: none;
          padding: 10px;
          border-radius: 6px;
          cursor: pointer;
          font-weight: 600;
          margin-bottom: 12px;
        }

        .results {
          background: rgba(0,0,0,0.2);
          border-radius: 6px;
          padding: 12px;
        }

        .score-display {
          text-align: center;
          padding: 16px;
          border: 3px solid;
          border-radius: 50%;
          width: 80px;
          height: 80px;
          margin: 0 auto 16px;
          display: flex;
          flex-direction: column;
          justify-content: center;
        }

        .score-value { font-size: 28px; font-weight: 700; }
        .score-label { font-size: 12px; color: #888; }

        .metrics {
          display: grid;
          grid-template-columns: repeat(4, 1fr);
          gap: 8px;
          margin-bottom: 16px;
        }

        .metric {
          text-align: center;
          padding: 8px;
          background: rgba(255,255,255,0.03);
          border-radius: 6px;
        }

        .metric-value { display: block; font-weight: 600; font-size: 16px; }
        .metric-label { font-size: 10px; color: #888; }

        .issues h4, .suggestions h4 {
          margin: 0 0 8px 0;
          font-size: 12px;
          color: #888;
        }

        .issue-item {
          padding: 8px;
          background: rgba(0,0,0,0.2);
          border-radius: 4px;
          border-left: 3px solid;
          margin-bottom: 8px;
        }

        .issue-header {
          display: flex;
          gap: 8px;
          font-size: 10px;
          margin-bottom: 4px;
        }

        .issue-severity { font-weight: 600; }
        .issue-category { color: #888; }
        .issue-message { margin: 0; font-size: 12px; color: #ccc; }
        .issue-suggestion { margin: 4px 0 0 0; font-size: 11px; color: #888; }

        .suggestions { margin-top: 12px; }
        .suggestion-item {
          font-size: 11px;
          color: #4ade80;
          padding: 4px 0;
        }
      `}</style>
    </div>
  );
}
