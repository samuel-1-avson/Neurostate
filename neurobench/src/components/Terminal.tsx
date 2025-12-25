// Advanced Terminal CLI Component
// AI-Augmented terminal with advanced command parsing, ANSI colors, and tab completion

import { createSignal, For, Show, onMount, createEffect, batch } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface TerminalLine {
  type: "input" | "output" | "error" | "success" | "info" | "system" | "warning" | "ansi";
  content: string;
  timestamp: string;
  ansi?: string;  // ANSI color codes
}

interface CompletionItem {
  text: string;
  display: string;
  description: string;
  kind: string;
  insert_text?: string;
}

interface TerminalTheme {
  name: string;
  background: string;
  foreground: string;
  cursor: string;
  selection: string;
  black: string;
  red: string;
  green: string;
  yellow: string;
  blue: string;
  magenta: string;
  cyan: string;
  white: string;
  bright_black: string;
  bright_red: string;
  bright_green: string;
  bright_yellow: string;
  bright_blue: string;
  bright_magenta: string;
  bright_cyan: string;
  bright_white: string;
  register: string;
  address: string;
  pin: string;
  peripheral: string;
  success: string;
  warning: string;
  error: string;
  info: string;
}

interface TerminalProps {
  onCommand?: (cmd: string, output: string) => void;
  theme?: string;
}

// Default embedded dark theme
const defaultTheme: TerminalTheme = {
  name: "Embedded Dark",
  background: "#0d1117",
  foreground: "#c9d1d9",
  cursor: "#58a6ff",
  selection: "#264f78",
  black: "#0d1117",
  red: "#f85149",
  green: "#3fb950",
  yellow: "#d29922",
  blue: "#58a6ff",
  magenta: "#bc8cff",
  cyan: "#39c5cf",
  white: "#b1bac4",
  bright_black: "#484f58",
  bright_red: "#ff7b72",
  bright_green: "#56d364",
  bright_yellow: "#e3b341",
  bright_blue: "#79c0ff",
  bright_magenta: "#d2a8ff",
  bright_cyan: "#56d4dd",
  bright_white: "#f0f6fc",
  register: "#bc8cff",
  address: "#ffa657",
  pin: "#7ee787",
  peripheral: "#79c0ff",
  success: "#3fb950",
  warning: "#d29922",
  error: "#f85149",
  info: "#58a6ff",
};

const MAX_SCROLLBACK = 50000;  // 50k lines scrollback buffer

export function Terminal(props: TerminalProps) {
  const [lines, setLines] = createSignal<TerminalLine[]>([]);
  const [input, setInput] = createSignal("");
  const [history, setHistory] = createSignal<string[]>([]);
  const [historyIndex, setHistoryIndex] = createSignal(-1);
  const [completions, setCompletions] = createSignal<CompletionItem[]>([]);
  const [showCompletions, setShowCompletions] = createSignal(false);
  const [selectedCompletion, setSelectedCompletion] = createSignal(0);
  const [isExecuting, setIsExecuting] = createSignal(false);
  const [theme, setTheme] = createSignal<TerminalTheme>(defaultTheme);
  const [variables, setVariables] = createSignal<Record<string, string>>({});
  const [searchQuery, setSearchQuery] = createSignal("");
  const [showSearch, setShowSearch] = createSignal(false);
  const [searchResults, setSearchResults] = createSignal<number[]>([]);
  const [currentSearchIndex, setCurrentSearchIndex] = createSignal(0);
  
  let terminalRef: HTMLDivElement | undefined;
  let inputRef: HTMLInputElement | undefined;
  let searchInputRef: HTMLInputElement | undefined;

  function getTime(): string {
    const now = new Date();
    return now.toLocaleTimeString("en-US", { hour12: false });
  }

  function addLine(type: TerminalLine["type"], content: string, ansi?: string) {
    setLines(prev => {
      const newLines = [...prev, { type, content, timestamp: getTime(), ansi }];
      // Keep only last MAX_SCROLLBACK lines
      if (newLines.length > MAX_SCROLLBACK) {
        return newLines.slice(-MAX_SCROLLBACK);
      }
      return newLines;
    });
    // Scroll to bottom
    requestAnimationFrame(() => {
      if (terminalRef) {
        terminalRef.scrollTop = terminalRef.scrollHeight;
      }
    });
  }

  function addLines(newLines: Array<{line_type: string, content: string, ansi?: string}>) {
    batch(() => {
      newLines.forEach(line => {
        addLine(line.line_type as TerminalLine["type"], line.content, line.ansi);
      });
    });
  }

  // Load welcome message on mount
  onMount(async () => {
    try {
      const welcome = await invoke("terminal_get_welcome") as Array<{line_type: string, content: string, ansi?: string}>;
      addLines(welcome);
    } catch (e) {
      addLine("system", "NeuroBench Advanced Terminal v2.0");
      addLine("system", "Type 'help' for available commands, 'ai <question>' for AI assistant");
    }
    inputRef?.focus();
  });

  // Update completions when input changes
  async function updateCompletions(text: string, cursorPos: number) {
    if (!text.trim()) {
      setCompletions([]);
      setShowCompletions(false);
      return;
    }

    try {
      const results = await invoke("terminal_get_completions", { 
        input: text, 
        cursorPos 
      }) as CompletionItem[];
      setCompletions(results);
      setShowCompletions(results.length > 0);
      setSelectedCompletion(0);
    } catch (e) {
      setCompletions([]);
      setShowCompletions(false);
    }
  }

  async function executeCommand(cmd: string) {
    const trimmed = cmd.trim();
    if (!trimmed) return;

    // Add to history
    setHistory(prev => [...prev, trimmed]);
    setHistoryIndex(-1);
    setShowCompletions(false);

    // Show input line
    addLine("input", `$ ${trimmed}`);
    setIsExecuting(true);

    try {
      // Handle special frontend commands
      if (trimmed === "clear") {
        setLines([]);
        addLine("system", "Terminal cleared");
        setIsExecuting(false);
        setInput("");
        return;
      }

      if (trimmed === "history") {
        const hist = history();
        if (hist.length === 0) {
          addLine("info", "No command history");
        } else {
          hist.forEach((cmd, i) => {
            addLine("output", `  ${i + 1}  ${cmd}`);
          });
        }
        setIsExecuting(false);
        setInput("");
        return;
      }

      // Handle export command locally
      if (trimmed.startsWith("export ")) {
        const assignment = trimmed.slice(7);
        const eqPos = assignment.indexOf("=");
        if (eqPos > 0) {
          const key = assignment.slice(0, eqPos);
          const value = assignment.slice(eqPos + 1);
          setVariables(prev => ({ ...prev, [key]: value }));
          addLine("success", `✓ Set ${key}=${value}`);
        } else {
          addLine("info", "Usage: export VAR=value");
        }
        setIsExecuting(false);
        setInput("");
        return;
      }

      // Use advanced terminal backend for all other commands
      const result = await invoke("terminal_execute_advanced", { 
        command: trimmed,
        variables: variables()
      }) as { success: boolean, output: Array<{line_type: string, content: string, ansi?: string}>, command_count: number };

      if (result.output && result.output.length > 0) {
        addLines(result.output);
      } else if (!result.success) {
        addLine("error", "Command failed with no output");
      }

      props.onCommand?.(trimmed, result.output?.map(l => l.content).join("\n") || "");
    } catch (e) {
      addLine("error", `Error: ${e}`);
    }

    setIsExecuting(false);
    setInput("");
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      if (showCompletions() && completions().length > 0) {
        // Insert selected completion
        const selected = completions()[selectedCompletion()];
        insertCompletion(selected);
      } else {
        executeCommand(input());
      }
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (showCompletions()) {
        setSelectedCompletion(Math.max(0, selectedCompletion() - 1));
      } else if (history().length > 0) {
        const newIndex = historyIndex() === -1 ? history().length - 1 : Math.max(0, historyIndex() - 1);
        setHistoryIndex(newIndex);
        setInput(history()[newIndex] || "");
      }
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      if (showCompletions()) {
        setSelectedCompletion(Math.min(completions().length - 1, selectedCompletion() + 1));
      } else if (historyIndex() !== -1) {
        const newIndex = historyIndex() + 1;
        if (newIndex >= history().length) {
          setHistoryIndex(-1);
          setInput("");
        } else {
          setHistoryIndex(newIndex);
          setInput(history()[newIndex] || "");
        }
      }
    } else if (e.key === "Tab") {
      e.preventDefault();
      if (completions().length > 0) {
        const selected = completions()[selectedCompletion()];
        insertCompletion(selected);
      } else {
        // Trigger completion fetch
        updateCompletions(input(), input().length);
      }
    } else if (e.key === "Escape") {
      setShowCompletions(false);
      setShowSearch(false);
    } else if (e.ctrlKey && e.shiftKey && e.key === "F") {
      e.preventDefault();
      setShowSearch(true);
      setTimeout(() => searchInputRef?.focus(), 10);
    } else if (e.ctrlKey && e.key === "l") {
      e.preventDefault();
      setLines([]);
      addLine("system", "Terminal cleared");
    } else if (e.ctrlKey && e.key === "c") {
      // Cancel current input
      setInput("");
      addLine("output", "^C");
    }
  }

  function insertCompletion(item: CompletionItem) {
    const currentInput = input();
    const parts = currentInput.split(/\s+/);
    
    if (parts.length <= 1) {
      // Completing command
      setInput(item.text + " ");
    } else {
      // Completing argument
      parts[parts.length - 1] = item.text;
      setInput(parts.join(" ") + " ");
    }
    
    setShowCompletions(false);
    inputRef?.focus();
  }

  function handleInput(e: InputEvent) {
    const value = (e.target as HTMLInputElement).value;
    setInput(value);
    // Debounce completion updates
    setTimeout(() => updateCompletions(value, value.length), 100);
  }

  // Search functionality
  function handleSearch(query: string) {
    setSearchQuery(query);
    if (!query) {
      setSearchResults([]);
      return;
    }
    
    const results: number[] = [];
    lines().forEach((line, index) => {
      if (line.content.toLowerCase().includes(query.toLowerCase())) {
        results.push(index);
      }
    });
    setSearchResults(results);
    setCurrentSearchIndex(results.length > 0 ? 0 : -1);
    
    // Scroll to first result
    if (results.length > 0 && terminalRef) {
      const lineElements = terminalRef.querySelectorAll(".terminal-line");
      lineElements[results[0]]?.scrollIntoView({ behavior: "smooth", block: "center" });
    }
  }

  function nextSearchResult() {
    if (searchResults().length === 0) return;
    const next = (currentSearchIndex() + 1) % searchResults().length;
    setCurrentSearchIndex(next);
    scrollToSearchResult(next);
  }

  function prevSearchResult() {
    if (searchResults().length === 0) return;
    const prev = currentSearchIndex() === 0 ? searchResults().length - 1 : currentSearchIndex() - 1;
    setCurrentSearchIndex(prev);
    scrollToSearchResult(prev);
  }

  function scrollToSearchResult(index: number) {
    if (terminalRef) {
      const lineIndex = searchResults()[index];
      const lineElements = terminalRef.querySelectorAll(".terminal-line");
      lineElements[lineIndex]?.scrollIntoView({ behavior: "smooth", block: "center" });
    }
  }

  // Get color for line type
  function getLineColor(type: string): string {
    const t = theme();
    switch (type) {
      case "error": return t.error;
      case "success": return t.success;
      case "warning": return t.warning;
      case "info": return t.info;
      case "system": return t.magenta;
      case "input": return t.cyan;
      default: return t.foreground;
    }
  }

  // Render ANSI text with colors
  function renderAnsiContent(content: string, ansiCode?: string): string {
    if (!ansiCode) return content;
    // Extract color from ANSI code (simplified)
    // Full implementation would parse all ANSI sequences
    return content;
  }

  return (
    <div class="terminal" style={{ 
      "--term-bg": theme().background,
      "--term-fg": theme().foreground,
      "--term-cursor": theme().cursor,
      "--term-selection": theme().selection,
    }}>
      <div class="terminal-header">
        <div class="terminal-title">
          <span class="terminal-dot red" />
          <span class="terminal-dot yellow" />
          <span class="terminal-dot green" />
          <span class="terminal-title-text">🧠 NeuroBench Terminal</span>
        </div>
        <div class="terminal-actions">
          <button 
            class="terminal-action-btn" 
            onClick={() => setShowSearch(!showSearch())}
            title="Search (Ctrl+Shift+F)"
          >
            🔍
          </button>
          <button 
            class="terminal-action-btn" 
            onClick={() => setLines([])}
            title="Clear (Ctrl+L)"
          >
            Clear
          </button>
        </div>
      </div>
      
      {/* Search bar */}
      <Show when={showSearch()}>
        <div class="terminal-search">
          <input
            ref={searchInputRef}
            class="terminal-search-input"
            type="text"
            placeholder="Search..."
            value={searchQuery()}
            onInput={(e) => handleSearch((e.target as HTMLInputElement).value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.shiftKey ? prevSearchResult() : nextSearchResult();
              } else if (e.key === "Escape") {
                setShowSearch(false);
                inputRef?.focus();
              }
            }}
          />
          <span class="terminal-search-count">
            {searchResults().length > 0 
              ? `${currentSearchIndex() + 1}/${searchResults().length}` 
              : "No results"}
          </span>
          <button class="terminal-search-btn" onClick={prevSearchResult}>↑</button>
          <button class="terminal-search-btn" onClick={nextSearchResult}>↓</button>
          <button class="terminal-search-btn" onClick={() => setShowSearch(false)}>✕</button>
        </div>
      </Show>
      
      <div class="terminal-body" ref={terminalRef} onClick={() => inputRef?.focus()}>
        <For each={lines()}>
          {(line, index) => (
            <div 
              class={`terminal-line ${line.type}`}
              classList={{ 
                "search-highlight": searchResults().includes(index()),
                "search-current": searchResults()[currentSearchIndex()] === index()
              }}
              style={{ color: getLineColor(line.type) }}
            >
              <Show when={line.type === "input"}>
                <span class="terminal-prompt">❯</span>
              </Show>
              <span class="terminal-content">{line.content}</span>
            </div>
          )}
        </For>
        
        {/* Input line */}
        <div class="terminal-input-line">
          <span class="terminal-prompt" style={{ color: theme().cyan }}>❯</span>
          <input
            ref={inputRef}
            class="terminal-input"
            value={input()}
            onInput={handleInput}
            onKeyDown={handleKeyDown}
            placeholder={isExecuting() ? "Executing..." : "Type a command... (Tab for completion)"}
            disabled={isExecuting()}
            autocomplete="off"
            spellcheck={false}
            style={{ color: theme().foreground }}
          />
        </div>
        
        {/* Tab completion dropdown */}
        <Show when={showCompletions() && completions().length > 0}>
          <div class="terminal-completions">
            <For each={completions().slice(0, 10)}>
              {(item, index) => (
                <div 
                  class="terminal-completion-item"
                  classList={{ selected: selectedCompletion() === index() }}
                  onClick={() => {
                    insertCompletion(item);
                  }}
                >
                  <span class="completion-text">{item.display || item.text}</span>
                  <span class="completion-kind">{item.kind}</span>
                  <Show when={item.description}>
                    <span class="completion-desc">{item.description}</span>
                  </Show>
                </div>
              )}
            </For>
          </div>
        </Show>
      </div>
      
      <div class="terminal-footer">
        <span class="terminal-status">
          {isExecuting() ? "⏳ Running..." : "✓ Ready"}
        </span>
        <span class="terminal-hint">
          Tab: complete • ↑↓: history • Ctrl+Shift+F: search • Ctrl+L: clear
        </span>
      </div>
    </div>
  );
}

export default Terminal;
