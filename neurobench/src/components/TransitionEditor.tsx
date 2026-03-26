/**
 * TransitionEditor - Enhanced Panel for editing edge/transition properties
 * 
 * Features:
 * - Comprehensive event type selection (25+ embedded system types)
 * - Guard condition editor with syntax hints
 * - Action code editor with pre/post hooks
 * - Priority and timing configuration
 * - Visual event type icons
 */

import { Component, createSignal, createEffect, Show, For } from 'solid-js';
import './TransitionEditor.css';
import { 
  EVENT_CATEGORIES, 
  EVENT_TYPE_NAMES, 
  EVENT_TYPE_ICONS, 
  EVENT_TYPE_COLORS 
} from '../hooks/useTransitionEngine';

export interface TransitionData {
  id: string;
  source: string;
  target: string;
  label?: string;
  condition?: string;
  event_type?: string;
  guard?: string;
  action?: string;
  priority: number;
  enabled?: boolean;
  sourcePort?: string;
  targetPort?: string;
}

interface TransitionEditorProps {
  transition: TransitionData | null;
  sourceNodeName?: string;
  targetNodeName?: string;
  onUpdate: (id: string, data: Partial<TransitionData>) => void;
  onDelete: (id: string) => void;
  onClose: () => void;
}

// Category icons and names
const CATEGORY_INFO: Record<string, { icon: string; name: string }> = {
  hardware: { icon: '⚡', name: 'Hardware' },
  timer: { icon: '⏱️', name: 'Timers' },
  communication: { icon: '📡', name: 'Communication' },
  system: { icon: '⚙️', name: 'System' },
  fsm: { icon: '📨', name: 'FSM Logic' },
  custom: { icon: '🔧', name: 'Custom' },
};

export const TransitionEditor: Component<TransitionEditorProps> = (props) => {
  const [label, setLabel] = createSignal('');
  const [eventType, setEventType] = createSignal('');
  const [guard, setGuard] = createSignal('');
  const [action, setAction] = createSignal('');
  const [priority, setPriority] = createSignal(0);
  const [enabled, setEnabled] = createSignal(true);
  const [isDirty, setIsDirty] = createSignal(false);
  const [guardError, setGuardError] = createSignal<string | null>(null);
  const [activeCategory, setActiveCategory] = createSignal<string>('fsm');

  // Load transition data when props change
  createEffect(() => {
    const t = props.transition;
    if (t) {
      setLabel(t.label || '');
      setEventType(t.event_type || t.condition || '');
      setGuard(t.guard || '');
      setAction(t.action || '');
      setPriority(t.priority || 0);
      setEnabled(t.enabled !== false);
      setIsDirty(false);
      
      // Find category for current event type
      const currentType = t.event_type || t.condition || '';
      for (const [cat, types] of Object.entries(EVENT_CATEGORIES)) {
        if ((types as readonly string[]).includes(currentType)) {
          setActiveCategory(cat);
          break;
        }
      }
    }
  });

  const validateGuard = () => {
    const guardExpr = guard();
    if (!guardExpr.trim()) {
      setGuardError(null);
      return true;
    }
    
    // Basic syntax validation
    if (guardExpr.includes(';;') || guardExpr.includes('{{')) {
      setGuardError('Invalid syntax');
      return false;
    }
    setGuardError(null);
    return true;
  };

  const handleApply = () => {
    if (!props.transition) return;
    if (!validateGuard()) return;
    
    props.onUpdate(props.transition.id, {
      label: label() || undefined,
      event_type: eventType() || undefined,
      condition: eventType() || undefined, // For backward compatibility
      guard: guard() || undefined,
      action: action() || undefined,
      priority: priority(),
      enabled: enabled(),
    });
    
    setIsDirty(false);
  };

  const handleDelete = () => {
    if (props.transition && confirm('Delete this transition?')) {
      props.onDelete(props.transition.id);
    }
  };

  const selectEventType = (type: string) => {
    setEventType(type);
    setIsDirty(true);
  };

  const getEventColor = () => {
    const type = eventType();
    return EVENT_TYPE_COLORS[type] || '#4CAF50';
  };

  return (
    <Show when={props.transition}>
      <div class="transition-editor">
        <div class="te-header" style={{ "border-left-color": getEventColor() }}>
          <div class="te-title">
            <span class="te-icon" style={{ color: getEventColor() }}>
              {EVENT_TYPE_ICONS[eventType()] || '◆'}
            </span>
            <span>TRANSITION</span>
          </div>
          <button class="te-close" onClick={props.onClose}>✕</button>
        </div>
        
        <div class="te-route">
          <span class="te-node source">{props.sourceNodeName || props.transition?.source}</span>
          <span class="te-arrow">→</span>
          <span class="te-node target">{props.targetNodeName || props.transition?.target}</span>
        </div>

        <div class="te-body">
          {/* Label */}
          <div class="te-field">
            <label>Label</label>
            <input
              type="text"
              value={label()}
              onInput={(e) => { setLabel(e.currentTarget.value); setIsDirty(true); }}
              placeholder="Display label..."
            />
          </div>

          {/* Event Type Selection */}
          <div class="te-field">
            <label>Event / Trigger</label>
            
            {/* Category Tabs */}
            <div class="te-category-tabs">
              <For each={Object.keys(CATEGORY_INFO)}>
                {(cat) => (
                  <button
                    class={`te-cat-tab ${activeCategory() === cat ? 'active' : ''}`}
                    onClick={() => setActiveCategory(cat)}
                    title={CATEGORY_INFO[cat].name}
                  >
                    {CATEGORY_INFO[cat].icon}
                  </button>
                )}
              </For>
            </div>
            
            {/* Event Type Grid */}
            <div class="te-event-grid">
              <For each={(EVENT_CATEGORIES as Record<string, readonly string[]>)[activeCategory()] || []}>
                {(type) => (
                  <button
                    class={`te-event-btn ${eventType() === type ? 'selected' : ''}`}
                    onClick={() => selectEventType(type)}
                    style={{ 
                      "border-color": eventType() === type ? EVENT_TYPE_COLORS[type] : 'transparent',
                      "background": eventType() === type ? `${EVENT_TYPE_COLORS[type]}20` : undefined,
                    }}
                  >
                    <span class="te-event-icon">{EVENT_TYPE_ICONS[type]}</span>
                    <span class="te-event-name">{EVENT_TYPE_NAMES[type]}</span>
                  </button>
                )}
              </For>
            </div>
            
            {/* Selected Event Display */}
            <Show when={eventType()}>
              <div class="te-selected-event" style={{ "border-left-color": getEventColor() }}>
                <span class="te-se-icon">{EVENT_TYPE_ICONS[eventType()]}</span>
                <span class="te-se-name">{EVENT_TYPE_NAMES[eventType()]}</span>
                <button class="te-se-clear" onClick={() => selectEventType('')}>✕</button>
              </div>
            </Show>
          </div>

          {/* Guard Condition */}
          <div class="te-field">
            <label>
              Guard Condition
              <span class="te-hint">Boolean expression (e.g., counter &gt; 10)</span>
            </label>
            <textarea
              class={`te-code ${guardError() ? 'error' : ''}`}
              value={guard()}
              onInput={(e) => { setGuard(e.currentTarget.value); setIsDirty(true); }}
              onBlur={validateGuard}
              placeholder="counter > 10 && isReady()"
              rows={2}
            />
            <Show when={guardError()}>
              <div class="te-error">{guardError()}</div>
            </Show>
          </div>

          {/* Action */}
          <div class="te-field">
            <label>
              Action
              <span class="te-hint">Code to execute on transition</span>
            </label>
            <textarea
              class="te-code"
              value={action()}
              onInput={(e) => { setAction(e.currentTarget.value); setIsDirty(true); }}
              placeholder="reset_counter();&#10;led_on();"
              rows={3}
            />
          </div>

          {/* Priority & Enabled */}
          <div class="te-row">
            <div class="te-field te-small">
              <label>Priority</label>
              <input
                type="number"
                class="te-priority"
                value={priority()}
                min={0}
                max={255}
                onInput={(e) => { setPriority(parseInt(e.currentTarget.value) || 0); setIsDirty(true); }}
              />
            </div>
            <div class="te-field te-small">
              <label>Enabled</label>
              <button 
                class={`te-toggle ${enabled() ? 'on' : 'off'}`}
                onClick={() => { setEnabled(!enabled()); setIsDirty(true); }}
              >
                {enabled() ? 'ON' : 'OFF'}
              </button>
            </div>
          </div>
        </div>

        <div class="te-footer">
          <button class="te-btn delete" onClick={handleDelete}>
            ✕ Delete
          </button>
          <div class="te-spacer" />
          <Show when={isDirty()}>
            <span class="te-unsaved">Unsaved</span>
          </Show>
          <button 
            class={`te-btn apply ${isDirty() ? 'active' : ''}`}
            onClick={handleApply}
            disabled={!isDirty()}
          >
            ✓ Apply
          </button>
        </div>
      </div>
    </Show>
  );
};

export default TransitionEditor;
