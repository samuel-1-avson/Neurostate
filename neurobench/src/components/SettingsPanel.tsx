import { createSignal, createEffect, For, Show, JSX } from "solid-js";
import { Icons } from "./AppIcons";
import "./SettingsPanel.css";

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
  // Project-specific props (optional as settings can be global)
  projectPassword?: string | null;
  setProjectPassword?: (pwd: string | null) => void;
}

export function SettingsPanel(props: SettingsPanelProps) {
  const [settings, setSettings] = createSignal<SystemSettings>(defaultSettings);
  const [activeTab, setActiveTab] = createSignal("general");
  const [hasChanges, setHasChanges] = createSignal(false);

  // Local state for password input to avoid constantly updating parent
  const [localPwd, setLocalPwd] = createSignal(props.projectPassword || "");

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
    
    // Also save password if changed
    if (props.setProjectPassword && localPwd() !== (props.projectPassword || "")) {
        props.setProjectPassword(localPwd() || null);
        props.onLog?.("Security", "Project password updated", "success");
    }

    props.onLog?.("Settings", "Settings saved", "success");
  };

  const resetSettings = () => {
    setSettings(defaultSettings);
    setHasChanges(true);
    props.onLog?.("Settings", "Settings reset to defaults", "info");
  };

  const tabs: { id: string; label: string; icon: () => JSX.Element }[] = [
    { id: "general", label: "General", icon: () => Icons.settings() },
    { id: "editor", label: "Editor", icon: () => Icons.code() },
    { id: "compiler", label: "Compiler", icon: () => Icons.build() },
    { id: "hardware", label: "Hardware", icon: () => Icons.plug() },
    { id: "ai", label: "AI", icon: () => Icons.robot() },
    { id: "security", label: "Security", icon: () => Icons.lock() },
    { id: "paths", label: "Paths", icon: () => Icons.folder() },
    { id: "profiles", label: "Profiles", icon: () => Icons.save() },
    { id: "shortcuts", label: "Shortcuts", icon: () => Icons.keyboard() },
  ];

  return (
    <div class="settings-panel">
      <div class="settings-header">
        <h2><span class="header-icon">{Icons.settings()}</span> Settings</h2>
        <div class="header-actions">
          <Show when={hasChanges() || (localPwd() !== (props.projectPassword || ""))}>
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
                <span class="tab-icon">{tab.icon()}</span>
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

          {/* Security */}
          <Show when={activeTab() === "security"}>
            <div class="settings-section">
              <h3>Project Encryption</h3>
              <div class="setting-description" style={{ "margin-bottom": "15px", "color": "#aaa", "font-size": "0.9em" }}>
                By default, NeuroBench encrypts all files transparently. Set a password below to add an extra layer of protection (requiring a password to open).
              </div>
              <div class="setting-row">
                <label>Project Password</label>
                <div class="path-input">
                  <input 
                    type="password" 
                    placeholder="Leave empty for transparent encryption"
                    value={localPwd()}
                    onInput={(e) => setLocalPwd(e.target.value)}
                  />
                </div>
              </div>
              <div style={{ "margin-top": "10px", "font-size": "0.85em", "color": "#888" }}>
                  <Show when={localPwd()}>
                      <span style={{ "color": "#4caf50" }}>✓ Password Protection Enabled</span>
                  </Show>
                  <Show when={!localPwd()}>
                      <span>✓ Standard Encryption (Auto-Open)</span>
                  </Show>
              </div>
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
          <button class="save-btn" onClick={saveSettings} disabled={!hasChanges() && localPwd() === (props.projectPassword || "")}>Save Settings</button>
        </div>
      </div>

    </div>
  );
}
