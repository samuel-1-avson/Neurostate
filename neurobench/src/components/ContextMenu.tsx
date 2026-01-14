import { Component, For, Show, onCleanup, onMount } from 'solid-js';

export interface MenuItem {
  label: string;
  icon?: any;
  shortcut?: string;
  onClick: () => void;
  disabled?: boolean;
  divider?: boolean;
}

export interface ContextMenuProps {
  x: number;
  y: number;
  items: MenuItem[];
  onClose: () => void;
}

const ContextMenu: Component<ContextMenuProps> = (props) => {
  let menuRef: HTMLDivElement | undefined;

  onMount(() => {
    // Close menu on click outside
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef && !menuRef.contains(e.target as Node)) {
        props.onClose();
      }
    };

    // Close on escape
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        props.onClose();
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleKeyDown);

    onCleanup(() => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    });
  });

  const handleItemClick = (item: MenuItem) => {
    if (!item.disabled) {
      item.onClick();
      props.onClose();
    }
  };

  // Adjust position to keep menu on screen
  const adjustedX = () => {
    const menuWidth = 220;
    const screenWidth = window.innerWidth;
    return props.x + menuWidth > screenWidth ? screenWidth - menuWidth - 10 : props.x;
  };

  const adjustedY = () => {
    const menuHeight = props.items.length * 36 + 16;
    const screenHeight = window.innerHeight;
    return props.y + menuHeight > screenHeight ? screenHeight - menuHeight - 10 : props.y;
  };

  return (
    <div
      ref={menuRef}
      class="context-menu"
      style={{
        position: 'fixed',
        left: `${adjustedX()}px`,
        top: `${adjustedY()}px`,
        'z-index': 9999,
      }}
    >
      <For each={props.items}>
        {(item) => (
          <Show when={!item.divider} fallback={<div class="context-menu-divider" />}>
            <div
              class={`context-menu-item ${item.disabled ? 'disabled' : ''}`}
              onClick={() => handleItemClick(item)}
            >
              <Show when={item.icon}>
                <span class="context-menu-icon">{item.icon}</span>
              </Show>
              <span class="context-menu-label">{item.label}</span>
              <Show when={item.shortcut}>
                <span class="context-menu-shortcut">{item.shortcut}</span>
              </Show>
            </div>
          </Show>
        )}
      </For>

      <style>{`
        .context-menu {
          background: #1e1e2e;
          border: 1px solid #3a3a4a;
          border-radius: 8px;
          padding: 6px 0;
          min-width: 180px;
          box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
          font-family: 'Inter', system-ui, sans-serif;
          animation: contextMenuAppear 0.15s ease-out;
        }

        @keyframes contextMenuAppear {
          from {
            opacity: 0;
            transform: scale(0.95);
          }
          to {
            opacity: 1;
            transform: scale(1);
          }
        }

        .context-menu-item {
          display: flex;
          align-items: center;
          padding: 8px 14px;
          cursor: pointer;
          color: #e0e0e0;
          font-size: 13px;
          transition: background 0.1s;
        }

        .context-menu-item:hover:not(.disabled) {
          background: #2a2a3e;
        }

        .context-menu-item.disabled {
          color: #666;
          cursor: not-allowed;
        }

        .context-menu-icon {
          width: 20px;
          margin-right: 10px;
          font-size: 14px;
          text-align: center;
        }

        .context-menu-label {
          flex: 1;
        }

        .context-menu-shortcut {
          color: #888;
          font-size: 11px;
          margin-left: 20px;
        }

        .context-menu-divider {
          height: 1px;
          background: #3a3a4a;
          margin: 6px 10px;
        }
      `}</style>
    </div>
  );
};

export default ContextMenu;
