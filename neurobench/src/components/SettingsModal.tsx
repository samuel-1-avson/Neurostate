/**
 * SettingsModal - Advanced settings for the Agent System
 * Modeled after the provided reference images.
 */
import { Component, createSignal, For } from "solid-js";
import { Icons } from "./AppIcons";
import "./SettingsModal.css";

type SettingsTab = "Agent" | "Browser" | "Editor" | "Notifications" | "Tab" | "Account";

export const SettingsModal: Component<{ onClose: () => void }> = (props) => {
  const [activeTab, setActiveTab] = createSignal<SettingsTab>("Agent");

  // Mock State for Settings (In real app, this would come from a store/backend)
  const [secureMode, setSecureMode] = createSignal(false);
  const [browserTools, setBrowserTools] = createSignal(true);
  const [telemetry, setTelemetry] = createSignal(true);

  const tabs: SettingsTab[] = ["Agent", "Browser", "Editor", "Notifications", "Tab", "Account"];

  const renderContent = () => {
    switch (activeTab()) {
      case "Agent":
        return (
          <div class="sm-content-scroll">
            <div class="sm-section">
              <div class="sm-section-title">SECURITY</div>
              <div class="sm-setting-card">
                <div class="sm-setting-header">
                  <div class="sm-setting-info">
                    <div class="sm-setting-name">Secure Mode</div>
                    <div class="sm-setting-desc">
                      When enabled, enforces settings that prevent the agent from autonomously running targeted exploits and requires human review for all agent actions.
                    </div>
                  </div>
                  <div class="sm-toggle-switch" classList={{ active: secureMode() }} onClick={() => setSecureMode(!secureMode())}>
                    <div class="sm-toggle-knob"></div>
                  </div>
                </div>
              </div>
            </div>

            <div class="sm-section">
              <div class="sm-section-title">ARTIFACT</div>
              <div class="sm-setting-card">
                <div class="sm-setting-header">
                  <div class="sm-setting-info">
                    <div class="sm-setting-name">Review Policy</div>
                    <div class="sm-setting-desc">
                      Specifies Agent's behavior when asking for review on artifacts.
                      <br/>• Always Proceed - Agent never asks for review.
                      <br/>• Agent Decides - Agent decides based on complexity.
                      <br/>• Request Review - Agent always asks for review.
                    </div>
                  </div>
                  <div class="sm-select-wrapper">
                    <select>
                      <option>Request Review</option>
                      <option>Agent Decides</option>
                      <option>Always Proceed</option>
                    </select>
                    <div class="sm-select-arrow">▼</div>
                  </div>
                </div>
              </div>
            </div>

             <div class="sm-section">
              <div class="sm-section-title">TERMINAL</div>
              <div class="sm-setting-card">
                <div class="sm-setting-header">
                  <div class="sm-setting-info">
                    <div class="sm-setting-name">Terminal Command Auto Execution</div>
                    <div class="sm-setting-desc">
                      Control how the agent executes terminal commands.
                      <br/>• Always Proceed - Maximum autonomy.
                      <br/>• Request Review - Safe mode.
                    </div>
                  </div>
                   <div class="sm-select-wrapper">
                    <select>
                      <option>Request Review</option>
                      <option>Always Proceed</option>
                    </select>
                    <div class="sm-select-arrow">▼</div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        );
      case "Browser":
        return (
           <div class="sm-content-scroll">
            <div class="sm-section">
              <div class="sm-section-title">GENERAL</div>
              <div class="sm-setting-card">
                <div class="sm-setting-header">
                  <div class="sm-setting-info">
                    <div class="sm-setting-name">Enable Browser Tools</div>
                    <div class="sm-setting-desc">
                      When enabled, Agent can use browser tools to open URLs, read web pages, and interact with browser content.
                    </div>
                  </div>
                  <div class="sm-toggle-switch" classList={{ active: browserTools() }} onClick={() => setBrowserTools(!browserTools())}>
                    <div class="sm-toggle-knob"></div>
                  </div>
                </div>
              </div>
               <div class="sm-setting-card">
                <div class="sm-setting-header">
                  <div class="sm-setting-info">
                    <div class="sm-setting-name">Browser Javascript Execution Policy</div>
                    <div class="sm-setting-desc">
                      Control if/how the agent executes JavaScript in the browser.
                    </div>
                  </div>
                   <div class="sm-select-wrapper">
                    <select>
                      <option>Request Review</option>
                      <option>Disabled</option>
                      <option>Always Proceed</option>
                    </select>
                    <div class="sm-select-arrow">▼</div>
                  </div>
                </div>
              </div>
            </div>
             <div class="sm-section">
              <div class="sm-section-title">ALLOWLIST</div>
              <div class="sm-setting-card">
                 <div class="sm-setting-header">
                    <div class="sm-setting-info">
                        <div class="sm-setting-name">Browser URL Allowlist</div>
                        <div class="sm-setting-desc">Control which URLs the browser can access.</div>
                    </div>
                    <button class="sm-btn-small">+ Add</button>
                 </div>
                 <div class="sm-list-input">
                     <input type="text" value="localhost" readonly />
                     <input type="text" value="chromewebdata" readonly />
                     <input type="text" value="developer.spotify.com" readonly />
                 </div>
              </div>
            </div>
           </div>
        );
       case "Account":
        return (
           <div class="sm-content-scroll">
            <div class="sm-section">
              <div class="sm-section-title">GENERAL</div>
              <div class="sm-setting-card">
                <div class="sm-setting-header">
                  <div class="sm-setting-info">
                    <div class="sm-setting-name">Enable Telemetry</div>
                    <div class="sm-setting-desc">
                      When toggled on, Antigravity collects usage data to help Google enhance performance and features.
                    </div>
                  </div>
                  <div class="sm-toggle-switch" classList={{ active: telemetry() }} onClick={() => setTelemetry(!telemetry())}>
                    <div class="sm-toggle-knob"></div>
                  </div>
                </div>
              </div>
            </div>
             <div class="sm-section">
              <div class="sm-section-title">ACCOUNT</div>
              <div class="sm-setting-card">
                <div class="sm-setting-header">
                  <div class="sm-setting-info">
                    <div class="sm-setting-name">Your Plan: Google AI Pro</div>
                    <div class="sm-setting-desc">
                      You can upgrade to the Google AI Ultra plan to receive the highest rate limits.
                    </div>
                  </div>
                   <button class="sm-btn-primary">Upgrade</button>
                </div>
              </div>
               <div class="sm-setting-card">
                <div class="sm-setting-header">
                  <div class="sm-setting-info">
                    <div class="sm-setting-name">Email</div>
                    <div class="sm-setting-desc">samuelavson360@gmail.com</div>
                  </div>
                   <button class="sm-btn-outline">Sign out</button>
                </div>
              </div>
            </div>
           </div>
        );
      default:
        return <div class="sm-placeholder">Settings for {activeTab()} coming soon...</div>;
    }
  };

  return (
    <div class="settings-modal-overlay">
      <div class="settings-modal-container">
        {/* Title Bar */}
        <div class="sm-title-bar">
            <span>Settings - {activeTab()}</span>
             <button class="sm-close-btn" onClick={props.onClose}>{Icons.x()}</button>
        </div>

        <div class="sm-body">
            {/* Sidebar */}
            <div class="sm-sidebar">
                <For each={tabs}>
                    {(tab) => (
                        <div 
                            class="sm-sidebar-item" 
                            classList={{ active: activeTab() === tab }}
                            onClick={() => setActiveTab(tab)}
                        >
                            {tab}
                        </div>
                    )}
                </For>
                
                 <div class="sm-sidebar-footer">
                   Provide Feedback
                 </div>
            </div>

            {/* Content */}
            <div class="sm-content">
                {renderContent()}
            </div>
        </div>
      </div>
    </div>
  );
};
