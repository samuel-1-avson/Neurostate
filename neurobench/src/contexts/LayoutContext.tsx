import { createContext, useContext, createSignal, createEffect, ParentComponent, Accessor } from "solid-js";

// Layout configuration interface
export interface LayoutConfig {
  // Visibility toggles
  showMenuBar: boolean;
  showActivityBar: boolean;
  showPrimarySideBar: boolean;
  showSecondarySideBar: boolean;
  showPanel: boolean;
  showStatusBar: boolean;
  
  // Position settings
  primarySideBarPosition: 'left' | 'right';
  panelAlignment: 'left' | 'right' | 'center' | 'justify';
  
  // Size settings (persisted)
  sideBarWidth: number;
  panelHeight: number;
}

// Default layout configuration
const DEFAULT_LAYOUT: LayoutConfig = {
  showMenuBar: true,
  showActivityBar: true,
  showPrimarySideBar: true,
  showSecondarySideBar: false,
  showPanel: true,
  showStatusBar: true,
  primarySideBarPosition: 'left',
  panelAlignment: 'center',
  sideBarWidth: 280,
  panelHeight: 200,
};

const STORAGE_KEY = 'neurobench-layout-config';

// Load config from localStorage
function loadLayoutConfig(): LayoutConfig {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return { ...DEFAULT_LAYOUT, ...parsed };
    }
  } catch (e) {
    console.warn('Failed to load layout config:', e);
  }
  return { ...DEFAULT_LAYOUT };
}

// Save config to localStorage
function saveLayoutConfig(config: LayoutConfig): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
  } catch (e) {
    console.warn('Failed to save layout config:', e);
  }
}

// Context type
interface LayoutContextValue {
  config: Accessor<LayoutConfig>;
  
  // Visibility toggles
  toggleMenuBar: () => void;
  toggleActivityBar: () => void;
  togglePrimarySideBar: () => void;
  toggleSecondarySideBar: () => void;
  togglePanel: () => void;
  toggleStatusBar: () => void;
  
  // Setters
  setPrimarySideBarPosition: (pos: 'left' | 'right') => void;
  setPanelAlignment: (align: 'left' | 'right' | 'center' | 'justify') => void;
  setSideBarWidth: (width: number) => void;
  setPanelHeight: (height: number) => void;
  
  // Bulk update
  updateConfig: (partial: Partial<LayoutConfig>) => void;
  resetToDefaults: () => void;
}

const LayoutContext = createContext<LayoutContextValue>();

export const LayoutProvider: ParentComponent = (props) => {
  const [config, setConfig] = createSignal<LayoutConfig>(loadLayoutConfig());
  
  // Persist config changes to localStorage
  createEffect(() => {
    saveLayoutConfig(config());
  });
  
  // Toggle functions
  const toggleMenuBar = () => setConfig(c => ({ ...c, showMenuBar: !c.showMenuBar }));
  const toggleActivityBar = () => setConfig(c => ({ ...c, showActivityBar: !c.showActivityBar }));
  const togglePrimarySideBar = () => setConfig(c => ({ ...c, showPrimarySideBar: !c.showPrimarySideBar }));
  const toggleSecondarySideBar = () => setConfig(c => ({ ...c, showSecondarySideBar: !c.showSecondarySideBar }));
  const togglePanel = () => setConfig(c => ({ ...c, showPanel: !c.showPanel }));
  const toggleStatusBar = () => setConfig(c => ({ ...c, showStatusBar: !c.showStatusBar }));
  
  // Position setters
  const setPrimarySideBarPosition = (pos: 'left' | 'right') => 
    setConfig(c => ({ ...c, primarySideBarPosition: pos }));
  
  const setPanelAlignment = (align: 'left' | 'right' | 'center' | 'justify') => 
    setConfig(c => ({ ...c, panelAlignment: align }));
  
  // Size setters
  const setSideBarWidth = (width: number) => setConfig(c => ({ ...c, sideBarWidth: width }));
  const setPanelHeight = (height: number) => setConfig(c => ({ ...c, panelHeight: height }));
  
  // Bulk update
  const updateConfig = (partial: Partial<LayoutConfig>) => 
    setConfig(c => ({ ...c, ...partial }));
  
  const resetToDefaults = () => setConfig({ ...DEFAULT_LAYOUT });
  
  const value: LayoutContextValue = {
    config,
    toggleMenuBar,
    toggleActivityBar,
    togglePrimarySideBar,
    toggleSecondarySideBar,
    togglePanel,
    toggleStatusBar,
    setPrimarySideBarPosition,
    setPanelAlignment,
    setSideBarWidth,
    setPanelHeight,
    updateConfig,
    resetToDefaults,
  };
  
  return (
    <LayoutContext.Provider value={value}>
      {props.children}
    </LayoutContext.Provider>
  );
};

// Hook to use layout context
export function useLayout(): LayoutContextValue {
  const context = useContext(LayoutContext);
  if (!context) {
    throw new Error('useLayout must be used within a LayoutProvider');
  }
  return context;
}

// Export default config for reference
export { DEFAULT_LAYOUT };
