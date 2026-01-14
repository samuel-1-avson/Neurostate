import { onMount, onCleanup } from "solid-js";
import { useLayout } from "../contexts/LayoutContext";

/**
 * Hook to register layout-related keyboard shortcuts
 * - Ctrl+B: Toggle Primary Side Bar
 * - Ctrl+J: Toggle Panel (bottom)
 */
export function useLayoutKeyboard(): void {
  const layout = useLayout();
  
  const handleKeyDown = (e: KeyboardEvent) => {
    // Ignore if user is typing in an input field
    const target = e.target as HTMLElement;
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
      return;
    }
    
    // Ctrl+B: Toggle Primary Side Bar
    if (e.ctrlKey && !e.shiftKey && !e.altKey && e.key.toLowerCase() === 'b') {
      e.preventDefault();
      layout.togglePrimarySideBar();
    }
    
    // Ctrl+J: Toggle Panel (bottom)
    if (e.ctrlKey && !e.shiftKey && !e.altKey && e.key.toLowerCase() === 'j') {
      e.preventDefault();
      layout.togglePanel();
    }
    
    // Ctrl+Shift+B: Toggle Secondary Side Bar (optional)
    if (e.ctrlKey && e.shiftKey && !e.altKey && e.key.toLowerCase() === 'b') {
      e.preventDefault();
      layout.toggleSecondarySideBar();
    }
  };
  
  onMount(() => {
    window.addEventListener('keydown', handleKeyDown);
  });
  
  onCleanup(() => {
    window.removeEventListener('keydown', handleKeyDown);
  });
}
