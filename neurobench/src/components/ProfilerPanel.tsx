import { createSignal, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { Icons } from "./AppIcons";
import "./ProfilerPanel.css";

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
        {Icons.chart()} <span>Performance Profiler</span>
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
                    <p class="issue-suggestion"><span class="icon-inline">{Icons.lightbulb()}</span> {issue.suggestion}</p>
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
                <div class="suggestion-item"><span class="icon-inline">{Icons.check()}</span> {suggestion}</div>
              )}
            </For>
          </div>
        </div>
      </Show>

    </div>
  );
}
