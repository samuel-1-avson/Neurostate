import { createSignal, Show, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { Icons } from "./AppIcons";
import "./ValidationPanel.css";

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
        <h4><span class="header-icon">{Icons.validate()}</span> Code Validation</h4>
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
                {res().success ? <><span class="status-icon">{Icons.checkCircle()}</span> Valid</> : <><span class="status-icon">{Icons.errorCircle()}</span> Invalid</>}
              </span>
              <span class="compiler-info">
                Compiler: {res().compiler} | Exit: {res().exitCode}
              </span>
            </div>

            {/* Errors */}
            <Show when={res().errors.length > 0}>
              <div class="message-section errors">
                <h5><span class="section-icon">{Icons.errorCircle()}</span> Errors ({res().errors.length})</h5>
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
                <h5><span class="section-icon">{Icons.warning()}</span> Warnings ({res().warnings.length})</h5>
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
                <span class="success-icon">{Icons.checkCircle()}</span> No errors or warnings found!
              </div>
            </Show>
          </div>
        )}
      </Show>

    </div>
  );
}
