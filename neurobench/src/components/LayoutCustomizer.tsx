import { Component, Show } from "solid-js";
import { useLayout } from "../contexts/LayoutContext";
import { Icons } from "./AppIcons";
import "./LayoutCustomizer.css";

interface LayoutCustomizerProps {
  isOpen: boolean;
  onClose: () => void;
}

export const LayoutCustomizer: Component<LayoutCustomizerProps> = (props) => {
  const layout = useLayout();
  const config = () => layout.config();
  
  return (
    <Show when={props.isOpen}>
      <div class="layout-customizer-overlay" onClick={props.onClose} />
      <div class="layout-customizer-modal">
        <div class="layout-customizer-header">
          <h2>Customize Layout</h2>
          <div class="header-actions">
            <button class="reset-btn" onClick={layout.resetToDefaults} title="Reset to Defaults">
              <Icons.refresh />
            </button>
            <button class="close-btn" onClick={props.onClose}>
              <Icons.close />
            </button>
          </div>
        </div>
        
        <div class="layout-customizer-content">
          {/* Visibility Section */}
          <div class="layout-section">
            <div class="section-header">
              <span class="section-icon"><Icons.eye /></span>
              <span>Visibility</span>
            </div>
            
            <div class="layout-option">
              <label class="toggle-row">
                <input 
                  type="checkbox" 
                  checked={config().showMenuBar}
                  onChange={() => layout.toggleMenuBar()}
                />
                <span class="option-icon"><Icons.menu /></span>
                <span class="option-label">Menu Bar</span>
              </label>
            </div>
            
            <div class="layout-option">
              <label class="toggle-row">
                <input 
                  type="checkbox" 
                  checked={config().showActivityBar}
                  onChange={() => layout.toggleActivityBar()}
                />
                <span class="option-icon"><Icons.sidebar /></span>
                <span class="option-label">Activity Bar</span>
              </label>
            </div>
            
            <div class="layout-option">
              <label class="toggle-row">
                <input 
                  type="checkbox" 
                  checked={config().showPrimarySideBar}
                  onChange={() => layout.togglePrimarySideBar()}
                />
                <span class="option-icon"><Icons.panelLeft /></span>
                <span class="option-label">Primary Side Bar</span>
                <span class="shortcut-badge">Ctrl + B</span>
              </label>
            </div>
            
            <div class="layout-option">
              <label class="toggle-row">
                <input 
                  type="checkbox" 
                  checked={config().showSecondarySideBar}
                  onChange={() => layout.toggleSecondarySideBar()}
                />
                <span class="option-icon"><Icons.panelRight /></span>
                <span class="option-label">Secondary Side Bar</span>
              </label>
            </div>
            
            <div class="layout-option">
              <label class="toggle-row">
                <input 
                  type="checkbox" 
                  checked={config().showPanel}
                  onChange={() => layout.togglePanel()}
                />
                <span class="option-icon"><Icons.terminal /></span>
                <span class="option-label">Panel</span>
                <span class="shortcut-badge">Ctrl + J</span>
              </label>
            </div>
            
            <div class="layout-option">
              <label class="toggle-row">
                <input 
                  type="checkbox" 
                  checked={config().showStatusBar}
                  onChange={() => layout.toggleStatusBar()}
                />
                <span class="option-icon"><Icons.info /></span>
                <span class="option-label">Status Bar</span>
              </label>
            </div>
          </div>
          
          {/* Primary Side Bar Position */}
          <div class="layout-section">
            <div class="section-header">
              <span class="section-label">Primary Side Bar Position</span>
            </div>
            
            <div class="position-options">
              <label class="position-option">
                <input 
                  type="radio" 
                  name="sidebarPosition"
                  checked={config().primarySideBarPosition === 'left'}
                  onChange={() => layout.setPrimarySideBarPosition('left')}
                />
                <span class="position-icon"><Icons.panelLeft /></span>
                <span>Left</span>
                <Show when={config().primarySideBarPosition === 'left'}>
                  <span class="check-icon"><Icons.check /></span>
                </Show>
              </label>
              
              <label class="position-option">
                <input 
                  type="radio" 
                  name="sidebarPosition"
                  checked={config().primarySideBarPosition === 'right'}
                  onChange={() => layout.setPrimarySideBarPosition('right')}
                />
                <span class="position-icon"><Icons.panelRight /></span>
                <span>Right</span>
                <Show when={config().primarySideBarPosition === 'right'}>
                  <span class="check-icon"><Icons.check /></span>
                </Show>
              </label>
            </div>
          </div>
          
          {/* Panel Alignment */}
          <div class="layout-section">
            <div class="section-header">
              <span class="section-label">Panel Alignment</span>
            </div>
            
            <div class="alignment-options">
              <label class="alignment-option">
                <input 
                  type="radio" 
                  name="panelAlignment"
                  checked={config().panelAlignment === 'left'}
                  onChange={() => layout.setPanelAlignment('left')}
                />
                <span class="alignment-icon"><Icons.alignLeft /></span>
                <span>Left</span>
                <Show when={config().panelAlignment === 'left'}>
                  <span class="check-icon"><Icons.check /></span>
                </Show>
              </label>
              
              <label class="alignment-option">
                <input 
                  type="radio" 
                  name="panelAlignment"
                  checked={config().panelAlignment === 'right'}
                  onChange={() => layout.setPanelAlignment('right')}
                />
                <span class="alignment-icon"><Icons.alignRight /></span>
                <span>Right</span>
                <Show when={config().panelAlignment === 'right'}>
                  <span class="check-icon"><Icons.check /></span>
                </Show>
              </label>
              
              <label class="alignment-option">
                <input 
                  type="radio" 
                  name="panelAlignment"
                  checked={config().panelAlignment === 'center'}
                  onChange={() => layout.setPanelAlignment('center')}
                />
                <span class="alignment-icon"><Icons.alignCenter /></span>
                <span>Center</span>
                <Show when={config().panelAlignment === 'center'}>
                  <span class="check-icon"><Icons.check /></span>
                </Show>
              </label>
              
              <label class="alignment-option">
                <input 
                  type="radio" 
                  name="panelAlignment"
                  checked={config().panelAlignment === 'justify'}
                  onChange={() => layout.setPanelAlignment('justify')}
                />
                <span class="alignment-icon"><Icons.alignJustify /></span>
                <span>Justify</span>
                <Show when={config().panelAlignment === 'justify'}>
                  <span class="check-icon"><Icons.check /></span>
                </Show>
              </label>
            </div>
          </div>
        </div>
      </div>
    </Show>
  );
};
