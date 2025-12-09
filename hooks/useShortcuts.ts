import { useEffect } from 'react';

export interface Shortcut {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  meta?: boolean; // Command on Mac
  alt?: boolean;
  action: () => void;
  description?: string;
}

export function useShortcuts(shortcuts: Shortcut[]) {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ignore if user is typing in an input
      if (['INPUT', 'TEXTAREA'].includes((e.target as HTMLElement).tagName)) {
        return;
      }

      shortcuts.forEach(({ key, ctrl, shift, meta, alt, action }) => {
        const keyMatch = e.key.toLowerCase() === key.toLowerCase();
        const ctrlMatch = !!ctrl === (e.ctrlKey || e.metaKey); // Treat Meta as Ctrl for Mac friendly
        const shiftMatch = !!shift === e.shiftKey;
        const altMatch = !!alt === e.altKey;

        if (keyMatch && ctrlMatch && shiftMatch && altMatch) {
          e.preventDefault();
          action();
        }
      });
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [shortcuts]);
}