/**
 * NodePropertiesPanel - Panel for editing node properties
 * 
 * Features:
 * - Edit node label
 * - Entry/Exit actions for state nodes
 * - Node-specific properties based on type
 * - Port configuration
 */

import { Component, createSignal, createEffect, Show, For } from 'solid-js';
import { Icons } from './AppIcons';
import './NodePropertiesPanel.css';

export interface NodeData {
  id: string;
  label: string;
  node_type: string;
  x: number;
  y: number;
  width: number;
  height: number;
  entry_action?: string;
  exit_action?: string;
  description?: string;
  properties?: Record<string, any>;
}

interface NodePropertiesPanelProps {
  node: NodeData | null;
  onUpdate: (id: string, data: Partial<NodeData>) => void;
  onDelete: (id: string) => void;
  onClose: () => void;
}

// Node type specific property definitions
const NODE_TYPE_PROPERTIES: Record<string, Array<{
  key: string;
  label: string;
  type: 'text' | 'number' | 'select' | 'boolean';
  options?: string[];
}>> = {
  gpio: [
    { key: 'port', label: 'Port', type: 'select', options: ['GPIOA', 'GPIOB', 'GPIOC', 'GPIOD', 'GPIOE', 'GPIOF'] },
    { key: 'pin', label: 'Pin', type: 'number' },
    { key: 'mode', label: 'Mode', type: 'select', options: ['INPUT', 'OUTPUT', 'ANALOG', 'ALTERNATE'] },
    { key: 'pull', label: 'Pull', type: 'select', options: ['NONE', 'PULLUP', 'PULLDOWN'] },
  ],
  timer: [
    { key: 'instance', label: 'Timer', type: 'select', options: ['TIM1', 'TIM2', 'TIM3', 'TIM4', 'TIM5', 'TIM6'] },
    { key: 'period_ms', label: 'Period (ms)', type: 'number' },
    { key: 'auto_reload', label: 'Auto Reload', type: 'boolean' },
  ],
  uart: [
    { key: 'instance', label: 'UART', type: 'select', options: ['USART1', 'USART2', 'USART3', 'UART4', 'UART5'] },
    { key: 'baudrate', label: 'Baud Rate', type: 'number' },
    { key: 'data_bits', label: 'Data Bits', type: 'select', options: ['7', '8', '9'] },
    { key: 'parity', label: 'Parity', type: 'select', options: ['NONE', 'EVEN', 'ODD'] },
  ],
  adc: [
    { key: 'instance', label: 'ADC', type: 'select', options: ['ADC1', 'ADC2', 'ADC3'] },
    { key: 'channel', label: 'Channel', type: 'number' },
    { key: 'resolution', label: 'Resolution', type: 'select', options: ['12', '10', '8', '6'] },
  ],
  pwm: [
    { key: 'timer', label: 'Timer', type: 'select', options: ['TIM1', 'TIM2', 'TIM3', 'TIM4'] },
    { key: 'channel', label: 'Channel', type: 'number' },
    { key: 'frequency', label: 'Frequency (Hz)', type: 'number' },
    { key: 'duty', label: 'Duty (%)', type: 'number' },
  ],
  delay: [
    { key: 'duration_ms', label: 'Duration (ms)', type: 'number' },
  ],
  semaphore: [
    { key: 'name', label: 'Name', type: 'text' },
    { key: 'initial_count', label: 'Initial Count', type: 'number' },
    { key: 'max_count', label: 'Max Count', type: 'number' },
  ],
  mutex: [
    { key: 'name', label: 'Name', type: 'text' },
    { key: 'priority_inherit', label: 'Priority Inheritance', type: 'boolean' },
  ],
};

// Get category color by node type
const getTypeColor = (nodeType: string): string => {
  const type = nodeType.toLowerCase();
  const colorMap: Record<string, string> = {
    state: '#4CAF50',
    initial: '#2196F3',
    final: '#9C27B0',
    decision: '#FF9800',
    gpio: '#607D8B',
    timer: '#795548',
    uart: '#3F51B5',
    adc: '#E91E63',
    pwm: '#00BCD4',
    default: '#4CAF50',
  };
  return colorMap[type] || colorMap.default;
};

// Check if node type supports entry/exit actions
const supportsActions = (nodeType: string): boolean => {
  const stateTypes = ['state', 'initial', 'final', 'decision', 'junction', 'composite'];
  return stateTypes.includes(nodeType.toLowerCase());
};

export const NodePropertiesPanel: Component<NodePropertiesPanelProps> = (props) => {
  const [label, setLabel] = createSignal('');
  const [description, setDescription] = createSignal('');
  const [entryAction, setEntryAction] = createSignal('');
  const [exitAction, setExitAction] = createSignal('');
  const [nodeProperties, setNodeProperties] = createSignal<Record<string, any>>({});
  const [isDirty, setIsDirty] = createSignal(false);

  // Load node data when props change
  createEffect(() => {
    const n = props.node;
    if (n) {
      setLabel(n.label || '');
      setDescription(n.description || '');
      setEntryAction(n.entry_action || '');
      setExitAction(n.exit_action || '');
      setNodeProperties(n.properties || {});
      setIsDirty(false);
    }
  });

  const getProperties = () => {
    const nodeType = props.node?.node_type.toLowerCase() || '';
    return NODE_TYPE_PROPERTIES[nodeType] || [];
  };

  const updateProperty = (key: string, value: any) => {
    setNodeProperties(prev => ({ ...prev, [key]: value }));
    setIsDirty(true);
  };

  const handleApply = () => {
    if (!props.node) return;
    
    props.onUpdate(props.node.id, {
      label: label(),
      description: description() || undefined,
      entry_action: entryAction() || undefined,
      exit_action: exitAction() || undefined,
      properties: Object.keys(nodeProperties()).length > 0 ? nodeProperties() : undefined,
    });
    
    setIsDirty(false);
  };

  const handleDelete = () => {
    if (props.node && confirm('Delete this node and all its connections?')) {
      props.onDelete(props.node.id);
    }
  };

  return (
    <Show when={props.node}>
      <div class="node-props-panel">
        <div class="npp-header">
          <div class="npp-title">
            <span 
              class="npp-type-badge"
              style={{ background: getTypeColor(props.node?.node_type || '') }}
            >
              {props.node?.node_type.replace(/_/g, ' ')}
            </span>
          </div>
          <button class="npp-close" onClick={props.onClose}>{Icons.x()}</button>
        </div>

        <div class="npp-body">
          {/* Label */}
          <div class="npp-field">
            <label>Label</label>
            <input
              type="text"
              value={label()}
              onInput={(e) => { setLabel(e.currentTarget.value); setIsDirty(true); }}
              placeholder="Node name..."
            />
          </div>

          {/* Description */}
          <div class="npp-field">
            <label>Description</label>
            <textarea
              value={description()}
              onInput={(e) => { setDescription(e.currentTarget.value); setIsDirty(true); }}
              placeholder="Optional description..."
              rows={2}
            />
          </div>

          {/* Entry/Exit Actions for State Nodes */}
          <Show when={supportsActions(props.node?.node_type || '')}>
            <div class="npp-section">
              <div class="npp-section-title"><span class="icon-inline">{Icons.flash()}</span> State Actions</div>
              
              <div class="npp-field">
                <label>
                  Entry Action
                  <span class="npp-hint">Runs when entering state</span>
                </label>
                <textarea
                  class="npp-code"
                  value={entryAction()}
                  onInput={(e) => { setEntryAction(e.currentTarget.value); setIsDirty(true); }}
                  placeholder="led_on();&#10;start_timer();"
                  rows={3}
                />
              </div>

              <div class="npp-field">
                <label>
                  Exit Action
                  <span class="npp-hint">Runs when leaving state</span>
                </label>
                <textarea
                  class="npp-code"
                  value={exitAction()}
                  onInput={(e) => { setExitAction(e.currentTarget.value); setIsDirty(true); }}
                  placeholder="led_off();&#10;stop_timer();"
                  rows={3}
                />
              </div>
            </div>
          </Show>

          {/* Type-specific Properties */}
          <Show when={getProperties().length > 0}>
            <div class="npp-section">
              <div class="npp-section-title"><span class="icon-inline">{Icons.settings()}</span> Configuration</div>
              
              <For each={getProperties()}>
                {(prop) => (
                  <div class="npp-field">
                    <label>{prop.label}</label>
                    {prop.type === 'select' ? (
                      <select
                        value={nodeProperties()[prop.key] || ''}
                        onChange={(e) => updateProperty(prop.key, e.currentTarget.value)}
                      >
                        <option value="">Select...</option>
                        <For each={prop.options || []}>
                          {(opt) => <option value={opt}>{opt}</option>}
                        </For>
                      </select>
                    ) : prop.type === 'number' ? (
                      <input
                        type="number"
                        value={nodeProperties()[prop.key] || ''}
                        onInput={(e) => updateProperty(prop.key, parseFloat(e.currentTarget.value) || 0)}
                      />
                    ) : prop.type === 'boolean' ? (
                      <label class="npp-checkbox">
                        <input
                          type="checkbox"
                          checked={nodeProperties()[prop.key] || false}
                          onChange={(e) => updateProperty(prop.key, e.currentTarget.checked)}
                        />
                        <span>{nodeProperties()[prop.key] ? 'Yes' : 'No'}</span>
                      </label>
                    ) : (
                      <input
                        type="text"
                        value={nodeProperties()[prop.key] || ''}
                        onInput={(e) => updateProperty(prop.key, e.currentTarget.value)}
                      />
                    )}
                  </div>
                )}
              </For>
            </div>
          </Show>

          {/* Position Info */}
          <div class="npp-section">
            <div class="npp-section-title"><span class="icon-inline">{Icons.layout()}</span> Position</div>
            <div class="npp-position">
              <span>X: {Math.round(props.node?.x || 0)}</span>
              <span>Y: {Math.round(props.node?.y || 0)}</span>
              <span>W: {props.node?.width || 180}</span>
              <span>H: {props.node?.height || 80}</span>
            </div>
          </div>
        </div>

        <div class="npp-footer">
          <button class="npp-btn delete" onClick={handleDelete}>
            {Icons.trash()} Delete
          </button>
          <div class="npp-spacer" />
          <Show when={isDirty()}>
            <span class="npp-unsaved">Unsaved</span>
          </Show>
          <button 
            class={`npp-btn apply ${isDirty() ? 'active' : ''}`}
            onClick={handleApply}
            disabled={!isDirty()}
          >
            {Icons.check()} Apply
          </button>
        </div>
      </div>
    </Show>
  );
};

export default NodePropertiesPanel;
