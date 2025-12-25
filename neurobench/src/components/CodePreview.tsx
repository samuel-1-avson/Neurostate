import { createEffect, createSignal, Show } from "solid-js";
import hljs from "highlight.js/lib/core";
import c from "highlight.js/lib/languages/c";
import cpp from "highlight.js/lib/languages/cpp";
import rust from "highlight.js/lib/languages/rust";

// Register languages
hljs.registerLanguage("c", c);
hljs.registerLanguage("cpp", cpp);
hljs.registerLanguage("rust", rust);

interface CodePreviewProps {
  code: string;
  language?: "c" | "cpp" | "rust";
  showLineNumbers?: boolean;
  maxHeight?: string;
  onCopy?: () => void;
}

export function CodePreview(props: CodePreviewProps) {
  const [copied, setCopied] = createSignal(false);
  let codeRef: HTMLElement | undefined;

  const language = () => props.language || "c";
  const showLineNumbers = () => props.showLineNumbers !== false;

  // Highlight code when it changes
  createEffect(() => {
    if (codeRef && props.code) {
      try {
        const highlighted = hljs.highlight(props.code, { language: language() });
        codeRef.innerHTML = highlighted.value;
      } catch (e) {
        // Fallback to plain text if highlighting fails
        codeRef.textContent = props.code;
      }
    }
  });

  const copyToClipboard = async () => {
    await navigator.clipboard.writeText(props.code);
    setCopied(true);
    props.onCopy?.();
    setTimeout(() => setCopied(false), 2000);
  };

  const lines = () => props.code.split("\n");
  const lineCount = () => lines().length;

  return (
    <div class="code-preview">
      <div class="code-header">
        <div class="code-info">
          <span class="language-badge">{language().toUpperCase()}</span>
          <span class="line-count">{lineCount()} lines</span>
        </div>
        <button class="copy-btn" onClick={copyToClipboard}>
          {copied() ? "✓ Copied!" : "📋 Copy"}
        </button>
      </div>

      <div class="code-container" style={{ "max-height": props.maxHeight || "400px" }}>
        <Show when={showLineNumbers()}>
          <div class="line-numbers">
            {lines().map((_, i) => (
              <span class="line-number">{i + 1}</span>
            ))}
          </div>
        </Show>
        <pre class="code-content">
          <code ref={codeRef} class={`hljs language-${language()}`}></code>
        </pre>
      </div>

      <style>{`
        .code-preview {
          background: #1e1e2e;
          border: 1px solid var(--border, #333);
          border-radius: 8px;
          overflow: hidden;
          font-family: 'Fira Code', 'JetBrains Mono', 'Consolas', monospace;
        }

        .code-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 10px 14px;
          background: #181825;
          border-bottom: 1px solid #313244;
        }

        .code-info {
          display: flex;
          align-items: center;
          gap: 12px;
        }

        .language-badge {
          background: linear-gradient(135deg, #89b4fa, #74c7ec);
          color: #11111b;
          padding: 3px 10px;
          border-radius: 4px;
          font-size: 11px;
          font-weight: 700;
          letter-spacing: 0.5px;
        }

        .line-count {
          color: #6c7086;
          font-size: 12px;
        }

        .copy-btn {
          background: rgba(203, 166, 247, 0.15);
          border: 1px solid rgba(203, 166, 247, 0.3);
          color: #cba6f7;
          padding: 6px 14px;
          border-radius: 6px;
          cursor: pointer;
          font-size: 12px;
          font-weight: 500;
          transition: all 0.2s;
        }

        .copy-btn:hover {
          background: rgba(203, 166, 247, 0.25);
          transform: translateY(-1px);
        }

        .code-container {
          display: flex;
          overflow: auto;
        }

        .line-numbers {
          display: flex;
          flex-direction: column;
          padding: 12px 0;
          background: #181825;
          border-right: 1px solid #313244;
          text-align: right;
          user-select: none;
          position: sticky;
          left: 0;
          z-index: 1;
        }

        .line-number {
          display: block;
          padding: 0 12px;
          color: #45475a;
          font-size: 12px;
          line-height: 1.5;
          min-width: 40px;
        }

        .line-number:hover {
          color: #89b4fa;
        }

        .code-content {
          flex: 1;
          margin: 0;
          padding: 12px 16px;
          overflow-x: auto;
          background: transparent;
        }

        .code-content code {
          font-size: 13px;
          line-height: 1.5;
          tab-size: 4;
        }

        /* Catppuccin Mocha Theme */
        .hljs {
          color: #cdd6f4;
          background: transparent;
        }

        .hljs-keyword,
        .hljs-selector-tag,
        .hljs-title,
        .hljs-section,
        .hljs-doctag,
        .hljs-name,
        .hljs-strong {
          color: #cba6f7;
        }

        .hljs-comment {
          color: #6c7086;
          font-style: italic;
        }

        .hljs-string,
        .hljs-title.class_,
        .hljs-title.class_.inherited__,
        .hljs-meta .hljs-string {
          color: #a6e3a1;
        }

        .hljs-number,
        .hljs-selector-id,
        .hljs-selector-class,
        .hljs-quote,
        .hljs-template-tag,
        .hljs-deletion {
          color: #fab387;
        }

        .hljs-type,
        .hljs-class .hljs-title,
        .hljs-tag,
        .hljs-attr,
        .hljs-template-variable,
        .hljs-variable,
        .hljs-literal {
          color: #f9e2af;
        }

        .hljs-built_in,
        .hljs-builtin-name,
        .hljs-bullet,
        .hljs-symbol,
        .hljs-link,
        .hljs-meta .hljs-keyword,
        .hljs-selector-attr {
          color: #89dceb;
        }

        .hljs-addition,
        .hljs-params {
          color: #b4befe;
        }

        .hljs-function {
          color: #89b4fa;
        }

        .hljs-meta,
        .hljs-meta-keyword {
          color: #f38ba8;
        }

        .hljs-regexp,
        .hljs-selector-pseudo {
          color: #94e2d5;
        }
      `}</style>
    </div>
  );
}
