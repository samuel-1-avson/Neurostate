import { createSignal, Show, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface ValidationMessage {
  line: number | null;
  column: number | null;
  message: string;
  severity: string;
}

interface ValidationResult {
  success: boolean;
  errors: ValidationMessage[];
  warnings: ValidationMessage[];
  compiler: string;
  exitCode: number;
}

interface ValidationPanelProps {
  code: string;
  language: string;
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function ValidationPanel(props: ValidationPanelProps) {
  const [isValidating, setIsValidating] = createSignal(false);
  const [result, setResult] = createSignal<ValidationResult | null>(null);
  const [validateEmbedded, setValidateEmbedded] = createSignal(true);

  const addLog = (source: string, message: string, type: "info" | "success" | "warning" | "error" = "info") => {
    props.onLog?.(source, message, type);
  };

  const validateCode = async () => {
    if (!props.code.trim()) {
      addLog("Validate", "No code to validate", "warning");
      return;
    }

    setIsValidating(true);
    setResult(null);

    try {
      const validationResult = await invoke("validate_code", {
        code: props.code,
        language: props.language,
        embedded: validateEmbedded(),
      }) as ValidationResult;

      setResult(validationResult);

      if (validationResult.success) {
        const warningCount = validationResult.warnings.length;
        if (warningCount > 0) {
          addLog("Validate", `✅ Valid with ${warningCount} warning(s) (${validationResult.compiler})`, "warning");
        } else {
          addLog("Validate", `✅ Code is valid (${validationResult.compiler})`, "success");
        }
      } else {
        const errorCount = validationResult.errors.length;
        addLog("Validate", `❌ ${errorCount} error(s) found (${validationResult.compiler})`, "error");
      }
    } catch (e) {
      addLog("Error", `Validation failed: ${e}`, "error");
      setResult({
        success: false,
        errors: [{ line: null, column: null, message: String(e), severity: "error" }],
        warnings: [],
        compiler: "unknown",
        exitCode: -1,
      });
    }

    setIsValidating(false);
  };

  return (
    <div class="validation-panel">
      <div class="validation-header">
        <h4>🔍 Code Validation</h4>
        <div class="validation-controls">
          <label class="checkbox-label">
            <input
              type="checkbox"
              checked={validateEmbedded()}
              onChange={(e) => setValidateEmbedded(e.target.checked)}
            />
            Embedded Mode (STM32/ARM stubs)
          </label>
          <button
            class="validate-btn"
            onClick={validateCode}
            disabled={isValidating() || !props.code.trim()}
          >
            {isValidating() ? "Validating..." : "Validate Code"}
          </button>
        </div>
      </div>

      <Show when={result()}>
        {(res) => (
          <div class={`validation-result ${res().success ? "success" : "error"}`}>
            <div class="result-header">
              <span class={`status-badge ${res().success ? "success" : "error"}`}>
                {res().success ? "✅ Valid" : "❌ Invalid"}
              </span>
              <span class="compiler-info">
                Compiler: {res().compiler} | Exit: {res().exitCode}
              </span>
            </div>

            {/* Errors */}
            <Show when={res().errors.length > 0}>
              <div class="message-section errors">
                <h5>❌ Errors ({res().errors.length})</h5>
                <For each={res().errors}>
                  {(msg) => (
                    <div class="validation-message error">
                      <Show when={msg.line !== null}>
                        <span class="location">
                          Line {msg.line}{msg.column !== null ? `:${msg.column}` : ""}
                        </span>
                      </Show>
                      <span class="message-text">{msg.message}</span>
                    </div>
                  )}
                </For>
              </div>
            </Show>

            {/* Warnings */}
            <Show when={res().warnings.length > 0}>
              <div class="message-section warnings">
                <h5>⚠️ Warnings ({res().warnings.length})</h5>
                <For each={res().warnings}>
                  {(msg) => (
                    <div class="validation-message warning">
                      <Show when={msg.line !== null}>
                        <span class="location">
                          Line {msg.line}{msg.column !== null ? `:${msg.column}` : ""}
                        </span>
                      </Show>
                      <span class="message-text">{msg.message}</span>
                    </div>
                  )}
                </For>
              </div>
            </Show>

            {/* Success with no issues */}
            <Show when={res().success && res().errors.length === 0 && res().warnings.length === 0}>
              <div class="no-issues">
                🎉 No errors or warnings found!
              </div>
            </Show>
          </div>
        )}
      </Show>

      <style>{`
        .validation-panel {
          background: var(--bg-secondary, #1a1a2e);
          border: 1px solid var(--border, #333);
          border-radius: 8px;
          padding: 12px;
          margin-top: 12px;
        }

        .validation-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 12px;
        }

        .validation-header h4 {
          margin: 0;
          font-size: 14px;
          color: var(--text-primary, #fff);
        }

        .validation-controls {
          display: flex;
          align-items: center;
          gap: 12px;
        }

        .checkbox-label {
          display: flex;
          align-items: center;
          gap: 6px;
          font-size: 12px;
          color: var(--text-secondary, #888);
          cursor: pointer;
        }

        .validate-btn {
          background: linear-gradient(135deg, #4ade80, #22c55e);
          color: #000;
          border: none;
          padding: 8px 16px;
          border-radius: 6px;
          cursor: pointer;
          font-weight: 600;
          font-size: 12px;
          transition: all 0.2s;
        }

        .validate-btn:hover:not(:disabled) {
          transform: translateY(-1px);
          box-shadow: 0 4px 12px rgba(74, 222, 128, 0.3);
        }

        .validate-btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .validation-result {
          border-radius: 6px;
          padding: 12px;
          background: rgba(0, 0, 0, 0.2);
        }

        .validation-result.success {
          border-left: 3px solid #4ade80;
        }

        .validation-result.error {
          border-left: 3px solid #f87171;
        }

        .result-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 10px;
        }

        .status-badge {
          padding: 4px 10px;
          border-radius: 12px;
          font-size: 12px;
          font-weight: 600;
        }

        .status-badge.success {
          background: rgba(74, 222, 128, 0.2);
          color: #4ade80;
        }

        .status-badge.error {
          background: rgba(248, 113, 113, 0.2);
          color: #f87171;
        }

        .compiler-info {
          font-size: 11px;
          color: var(--text-secondary, #666);
        }

        .message-section {
          margin-top: 10px;
        }

        .message-section h5 {
          margin: 0 0 8px 0;
          font-size: 12px;
          color: var(--text-secondary, #888);
        }

        .validation-message {
          display: flex;
          gap: 8px;
          padding: 6px 10px;
          margin-bottom: 4px;
          border-radius: 4px;
          font-size: 12px;
          font-family: 'Fira Code', 'Consolas', monospace;
        }

        .validation-message.error {
          background: rgba(248, 113, 113, 0.1);
          color: #f87171;
        }

        .validation-message.warning {
          background: rgba(251, 191, 36, 0.1);
          color: #fbbf24;
        }

        .location {
          color: #60a5fa;
          font-weight: 600;
          white-space: nowrap;
        }

        .message-text {
          flex: 1;
          word-break: break-word;
        }

        .no-issues {
          text-align: center;
          color: #4ade80;
          padding: 16px;
          font-size: 14px;
        }
      `}</style>
    </div>
  );
}
