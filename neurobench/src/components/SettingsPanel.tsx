import { createSignal, createEffect, For, Show } from "solid-js";

export interface SystemSettings {
  // General
  theme: "dark" | "light" | "system";
  language: "en" | "zh" | "de" | "ja";
  fontSize: number;
  
  // Editor
  editorTheme: "default" | "monokai" | "dracula" | "solarized";
  tabSize: number;
  autoSave: boolean;
  autoSaveInterval: number;  // seconds
  lineNumbers: boolean;
  wordWrap: boolean;
  
  // Compiler
  compiler: "gcc" | "clang" | "armcc";
  optimization: "O0" | "O1" | "O2" | "O3" | "Os" | "Og";
  debugSymbols: boolean;
  warningsAsErrors: boolean;
  additionalFlags: string;
  
  // Hardware
  defaultMcu: string;
  defaultBaudRate: number;
  autoConnect: boolean;
  
  // AI
  aiModel: string;
  aiTemperature: number;
  aiMaxTokens: number;
  aiEnabled: boolean;
  
  // Paths
  toolchainPath: string;
  projectsPath: string;
  templatesPath: string;
}

const defaultSettings: SystemSettings = {
  theme: "dark",
  language: "en",
  fontSize: 13,
  editorTheme: "default",
  tabSize: 4,
  autoSave: true,
  autoSaveInterval: 30,
  lineNumbers: true,
  wordWrap: false,
  compiler: "gcc",
  optimization: "O2",
  debugSymbols: true,
  warningsAsErrors: false,
  additionalFlags: "",
  defaultMcu: "STM32F401",
  defaultBaudRate: 115200,
  autoConnect: false,
  aiModel: "gemini-1.5-flash",
  aiTemperature: 0.7,
  aiMaxTokens: 4096,
  aiEnabled: true,
  toolchainPath: "",
  projectsPath: "",
  templatesPath: "",
};

interface SettingsPanelProps {
  onClose?: () => void;
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function SettingsPanel(props: SettingsPanelProps) {
  const [settings, setSettings] = createSignal<SystemSettings>(defaultSettings);
  const [activeTab, setActiveTab] = createSignal("general");
  const [hasChanges, setHasChanges] = createSignal(false);

  // Load settings on mount
  createEffect(() => {
    const saved = localStorage.getItem("neurobench_settings");
    if (saved) {
      try {
        const parsed = JSON.parse(saved);
        setSettings({ ...defaultSettings, ...parsed });
      } catch (e) {
        console.error("Failed to load settings:", e);
      }
    }
  });

  const updateSetting = <K extends keyof SystemSettings>(key: K, value: SystemSettings[K]) => {
    setSettings(prev => ({ ...prev, [key]: value }));
    setHasChanges(true);
  };

  const saveSettings = () => {
    localStorage.setItem("neurobench_settings", JSON.stringify(settings()));
    setHasChanges(false);
    props.onLog?.("Settings", "Settings saved", "success");
  };

  const resetSettings = () => {
    setSettings(defaultSettings);
    setHasChanges(true);
    props.onLog?.("Settings", "Settings reset to defaults", "info");
  };

  const tabs = [
    { id: "general", label: "General", icon: "⚙️" },
    { id: "editor", label: "Editor", icon: "📝" },
    { id: "compiler", label: "Compiler", icon: "🔨" },
    { id: "hardware", label: "Hardware", icon: "🔌" },
    { id: "ai", label: "AI", icon: "🤖" },
    { id: "paths", label: "Paths", icon: "📁" },
  ];

  return (
    <div class="settings-panel">
      <div class="settings-header">
        <h2>⚙️ Settings</h2>
        <div class="header-actions">
          <Show when={hasChanges()}>
            <span class="unsaved-badge">Unsaved Changes</span>
          </Show>
          <button class="close-btn" onClick={props.onClose}>✕</button>
        </div>
      </div>

      <div class="settings-body">
        {/* Tab Navigation */}
        <div class="settings-tabs">
          <For each={tabs}>
            {(tab) => (
              <button 
                class={`tab-btn ${activeTab() === tab.id ? "active" : ""}`}
                onClick={() => setActiveTab(tab.id)}
              >
                <span class="tab-icon">{tab.icon}</span>
                <span class="tab-label">{tab.label}</span>
              </button>
            )}
          </For>
        </div>

        {/* Tab Content */}
        <div class="settings-content">
          {/* General */}
          <Show when={activeTab() === "general"}>
            <div class="settings-section">
              <h3>Appearance</h3>
              <div class="setting-row">
                <label>Theme</label>
                <select value={settings().theme} onChange={(e) => updateSetting("theme", e.target.value as any)}>
                  <option value="dark">Dark</option>
                  <option value="light">Light</option>
                  <option value="system">System</option>
                </select>
              </div>
              <div class="setting-row">
                <label>Language</label>
                <select value={settings().language} onChange={(e) => updateSetting("language", e.target.value as any)}>
                  <option value="en">English</option>
                  <option value="zh">中文</option>
                  <option value="de">Deutsch</option>
                  <option value="ja">日本語</option>
                </select>
              </div>
              <div class="setting-row">
                <label>Font Size</label>
                <input 
                  type="number" 
                  min="10" 
                  max="24" 
                  value={settings().fontSize}
                  onChange={(e) => updateSetting("fontSize", parseInt(e.target.value))}
                />
              </div>
            </div>
          </Show>

          {/* Editor */}
          <Show when={activeTab() === "editor"}>
            <div class="settings-section">
              <h3>Editor Settings</h3>
              <div class="setting-row">
                <label>Editor Theme</label>
                <select value={settings().editorTheme} onChange={(e) => updateSetting("editorTheme", e.target.value as any)}>
                  <option value="default">Default</option>
                  <option value="monokai">Monokai</option>
                  <option value="dracula">Dracula</option>
                  <option value="solarized">Solarized</option>
                </select>
              </div>
              <div class="setting-row">
                <label>Tab Size</label>
                <select value={settings().tabSize} onChange={(e) => updateSetting("tabSize", parseInt(e.target.value))}>
                  <option value="2">2 spaces</option>
                  <option value="4">4 spaces</option>
                  <option value="8">8 spaces</option>
                </select>
              </div>
              <div class="setting-row checkbox">
                <label>
                  <input 
                    type="checkbox" 
                    checked={settings().lineNumbers}
                    onChange={(e) => updateSetting("lineNumbers", e.target.checked)}
                  />
                  Show Line Numbers
                </label>
              </div>
              <div class="setting-row checkbox">
                <label>
                  <input 
                    type="checkbox" 
                    checked={settings().wordWrap}
                    onChange={(e) => updateSetting("wordWrap", e.target.checked)}
                  />
                  Word Wrap
                </label>
              </div>
              <div class="setting-row checkbox">
                <label>
                  <input 
                    type="checkbox" 
                    checked={settings().autoSave}
                    onChange={(e) => updateSetting("autoSave", e.target.checked)}
                  />
                  Auto Save
                </label>
              </div>
              <Show when={settings().autoSave}>
                <div class="setting-row">
                  <label>Auto Save Interval (seconds)</label>
                  <input 
                    type="number" 
                    min="5" 
                    max="300" 
                    value={settings().autoSaveInterval}
                    onChange={(e) => updateSetting("autoSaveInterval", parseInt(e.target.value))}
                  />
                </div>
              </Show>
            </div>
          </Show>

          {/* Compiler */}
          <Show when={activeTab() === "compiler"}>
            <div class="settings-section">
              <h3>Compiler Settings</h3>
              <div class="setting-row">
                <label>Compiler</label>
                <select value={settings().compiler} onChange={(e) => updateSetting("compiler", e.target.value as any)}>
                  <option value="gcc">GCC (arm-none-eabi-gcc)</option>
                  <option value="clang">Clang</option>
                  <option value="armcc">ARM Compiler</option>
                </select>
              </div>
              <div class="setting-row">
                <label>Optimization Level</label>
                <select value={settings().optimization} onChange={(e) => updateSetting("optimization", e.target.value as any)}>
                  <option value="O0">-O0 (No optimization)</option>
                  <option value="O1">-O1 (Basic)</option>
                  <option value="O2">-O2 (Standard)</option>
                  <option value="O3">-O3 (Maximum)</option>
                  <option value="Os">-Os (Size)</option>
                  <option value="Og">-Og (Debug)</option>
                </select>
              </div>
              <div class="setting-row checkbox">
                <label>
                  <input 
                    type="checkbox" 
                    checked={settings().debugSymbols}
                    onChange={(e) => updateSetting("debugSymbols", e.target.checked)}
                  />
                  Include Debug Symbols (-g)
                </label>
              </div>
              <div class="setting-row checkbox">
                <label>
                  <input 
                    type="checkbox" 
                    checked={settings().warningsAsErrors}
                    onChange={(e) => updateSetting("warningsAsErrors", e.target.checked)}
                  />
                  Treat Warnings as Errors (-Werror)
                </label>
              </div>
              <div class="setting-row">
                <label>Additional Flags</label>
                <input 
                  type="text" 
                  placeholder="-Wall -Wextra"
                  value={settings().additionalFlags}
                  onChange={(e) => updateSetting("additionalFlags", e.target.value)}
                />
              </div>
            </div>
          </Show>

          {/* Hardware */}
          <Show when={activeTab() === "hardware"}>
            <div class="settings-section">
              <h3>Hardware Defaults</h3>
              <div class="setting-row">
                <label>Default MCU</label>
                <select value={settings().defaultMcu} onChange={(e) => updateSetting("defaultMcu", e.target.value)}>
                  <option value="STM32F401">STM32F401</option>
                  <option value="STM32F407">STM32F407</option>
                  <option value="STM32F103">STM32F103</option>
                  <option value="ESP32">ESP32</option>
                  <option value="nRF52832">nRF52832</option>
                </select>
              </div>
              <div class="setting-row">
                <label>Default Baud Rate</label>
                <select value={settings().defaultBaudRate} onChange={(e) => updateSetting("defaultBaudRate", parseInt(e.target.value))}>
                  <option value="9600">9600</option>
                  <option value="38400">38400</option>
                  <option value="57600">57600</option>
                  <option value="115200">115200</option>
                  <option value="230400">230400</option>
                  <option value="460800">460800</option>
                  <option value="921600">921600</option>
                </select>
              </div>
              <div class="setting-row checkbox">
                <label>
                  <input 
                    type="checkbox" 
                    checked={settings().autoConnect}
                    onChange={(e) => updateSetting("autoConnect", e.target.checked)}
                  />
                  Auto-connect to last device
                </label>
              </div>
            </div>
          </Show>

          {/* AI */}
          <Show when={activeTab() === "ai"}>
            <div class="settings-section">
              <h3>AI Assistant</h3>
              <div class="setting-row checkbox">
                <label>
                  <input 
                    type="checkbox" 
                    checked={settings().aiEnabled}
                    onChange={(e) => updateSetting("aiEnabled", e.target.checked)}
                  />
                  Enable AI Features
                </label>
              </div>
              <Show when={settings().aiEnabled}>
                <div class="setting-row">
                  <label>AI Model</label>
                  <select value={settings().aiModel} onChange={(e) => updateSetting("aiModel", e.target.value)}>
                    <option value="gemini-1.5-flash">Gemini 1.5 Flash</option>
                    <option value="gemini-1.5-pro">Gemini 1.5 Pro</option>
                    <option value="gemini-2.0-flash-exp">Gemini 2.0 Flash (Exp)</option>
                  </select>
                </div>
                <div class="setting-row">
                  <label>Temperature: {settings().aiTemperature.toFixed(1)}</label>
                  <input 
                    type="range" 
                    min="0" 
                    max="1" 
                    step="0.1"
                    value={settings().aiTemperature}
                    onInput={(e) => updateSetting("aiTemperature", parseFloat(e.target.value))}
                  />
                </div>
                <div class="setting-row">
                  <label>Max Tokens</label>
                  <select value={settings().aiMaxTokens} onChange={(e) => updateSetting("aiMaxTokens", parseInt(e.target.value))}>
                    <option value="1024">1024</option>
                    <option value="2048">2048</option>
                    <option value="4096">4096</option>
                    <option value="8192">8192</option>
                  </select>
                </div>
              </Show>
            </div>
          </Show>

          {/* Paths */}
          <Show when={activeTab() === "paths"}>
            <div class="settings-section">
              <h3>System Paths</h3>
              <div class="setting-row">
                <label>Toolchain Path</label>
                <div class="path-input">
                  <input 
                    type="text" 
                    placeholder="C:\\Program Files\\ARM\\bin"
                    value={settings().toolchainPath}
                    onChange={(e) => updateSetting("toolchainPath", e.target.value)}
                  />
                  <button>Browse</button>
                </div>
              </div>
              <div class="setting-row">
                <label>Projects Directory</label>
                <div class="path-input">
                  <input 
                    type="text" 
                    placeholder="~/Documents/NeuroBench"
                    value={settings().projectsPath}
                    onChange={(e) => updateSetting("projectsPath", e.target.value)}
                  />
                  <button>Browse</button>
                </div>
              </div>
              <div class="setting-row">
                <label>Templates Directory</label>
                <div class="path-input">
                  <input 
                    type="text" 
                    placeholder="~/Documents/NeuroBench/templates"
                    value={settings().templatesPath}
                    onChange={(e) => updateSetting("templatesPath", e.target.value)}
                  />
                  <button>Browse</button>
                </div>
              </div>
            </div>
          </Show>
        </div>
      </div>

      {/* Footer */}
      <div class="settings-footer">
        <button class="reset-btn" onClick={resetSettings}>Reset to Defaults</button>
        <div class="footer-actions">
          <button class="cancel-btn" onClick={props.onClose}>Cancel</button>
          <button class="save-btn" onClick={saveSettings} disabled={!hasChanges()}>Save Settings</button>
        </div>
      </div>

      <style>{`
        .settings-panel {
          position: fixed;
          top: 50%;
          left: 50%;
          transform: translate(-50%, -50%);
          width: 700px;
          max-width: 90vw;
          max-height: 80vh;
          background: linear-gradient(135deg, #1e1e2e 0%, #181825 100%);
          border: 1px solid rgba(255,255,255,0.1);
          border-radius: 12px;
          box-shadow: 0 20px 60px rgba(0,0,0,0.5);
          display: flex;
          flex-direction: column;
          z-index: 1000;
        }

        .settings-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 16px 20px;
          border-bottom: 1px solid rgba(255,255,255,0.1);
        }

        .settings-header h2 { margin: 0; font-size: 18px; }

        .header-actions { display: flex; align-items: center; gap: 12px; }

        .unsaved-badge {
          background: rgba(251, 191, 36, 0.2);
          color: #fbbf24;
          padding: 4px 10px;
          border-radius: 12px;
          font-size: 11px;
        }

        .close-btn {
          background: none;
          border: none;
          color: #888;
          font-size: 16px;
          cursor: pointer;
          padding: 4px 8px;
        }

        .close-btn:hover { color: #ef4444; }

        .settings-body {
          display: flex;
          flex: 1;
          overflow: hidden;
        }

        .settings-tabs {
          width: 160px;
          padding: 12px;
          border-right: 1px solid rgba(255,255,255,0.1);
          display: flex;
          flex-direction: column;
          gap: 4px;
        }

        .tab-btn {
          display: flex;
          align-items: center;
          gap: 10px;
          padding: 10px 14px;
          background: transparent;
          border: none;
          color: #888;
          font-size: 13px;
          cursor: pointer;
          border-radius: 6px;
          text-align: left;
          transition: all 0.2s;
        }

        .tab-btn:hover { background: rgba(255,255,255,0.05); color: #ccc; }

        .tab-btn.active {
          background: rgba(59, 130, 246, 0.2);
          color: #60a5fa;
        }

        .tab-icon { font-size: 14px; }

        .settings-content {
          flex: 1;
          padding: 20px;
          overflow-y: auto;
        }

        .settings-section h3 {
          margin: 0 0 16px 0;
          font-size: 14px;
          color: #888;
          border-bottom: 1px solid rgba(255,255,255,0.1);
          padding-bottom: 8px;
        }

        .setting-row {
          margin-bottom: 16px;
        }

        .setting-row label {
          display: block;
          color: #ccc;
          font-size: 12px;
          margin-bottom: 6px;
        }

        .setting-row.checkbox label {
          display: flex;
          align-items: center;
          gap: 8px;
          cursor: pointer;
        }

        .setting-row input[type="text"],
        .setting-row input[type="number"],
        .setting-row select {
          width: 100%;
          background: rgba(255,255,255,0.05);
          border: 1px solid rgba(255,255,255,0.1);
          color: #fff;
          padding: 10px 12px;
          border-radius: 6px;
          font-size: 13px;
        }

        .setting-row input[type="range"] {
          width: 100%;
        }

        .path-input {
          display: flex;
          gap: 8px;
        }

        .path-input input { flex: 1; }

        .path-input button {
          background: rgba(255,255,255,0.1);
          border: none;
          color: #ccc;
          padding: 10px 16px;
          border-radius: 6px;
          cursor: pointer;
        }

        .settings-footer {
          display: flex;
          justify-content: space-between;
          padding: 16px 20px;
          border-top: 1px solid rgba(255,255,255,0.1);
        }

        .reset-btn {
          background: none;
          border: 1px solid rgba(255,255,255,0.2);
          color: #888;
          padding: 10px 16px;
          border-radius: 6px;
          cursor: pointer;
        }

        .footer-actions { display: flex; gap: 8px; }

        .cancel-btn {
          background: rgba(255,255,255,0.1);
          border: none;
          color: #ccc;
          padding: 10px 20px;
          border-radius: 6px;
          cursor: pointer;
        }

        .save-btn {
          background: linear-gradient(135deg, #3b82f6, #2563eb);
          border: none;
          color: white;
          padding: 10px 24px;
          border-radius: 6px;
          cursor: pointer;
          font-weight: 600;
        }

        .save-btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }
      `}</style>
    </div>
  );
}
