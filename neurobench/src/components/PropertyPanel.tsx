/**
 * PropertyPanel - Dynamic property editor for selected nodes
 */

import { Component, createSignal, createEffect, For, Show } from "solid-js";
import { Node, PropertyValue, PORT_COLORS } from "../hooks/useNodeEngine";
import { Icons } from "./AppIcons";
import "./PropertyPanel.css";

interface PropertyPanelProps {
  selectedNode: Node | null;
  onPropertyChange?: (nodeId: string, key: string, value: PropertyValue) => void;
  onLabelChange?: (nodeId: string, label: string) => void;
  onDescriptionChange?: (nodeId: string, description: string) => void;
}

const PropertyPanel: Component<PropertyPanelProps> = (props) => {
  const [localLabel, setLocalLabel] = createSignal("");
  const [localDescription, setLocalDescription] = createSignal("");
  const [expandedSections, setExpandedSections] = createSignal<Set<string>>(
    new Set(["general", "properties", "ports"])
  );

  // Sync local state with selected node
  createEffect(() => {
    const node = props.selectedNode;
    if (node) {
      setLocalLabel(node.label);
      setLocalDescription(node.description || "");
    }
  });

  const toggleSection = (section: string) => {
    const current = expandedSections();
    const newSet = new Set(current);
    if (newSet.has(section)) {
      newSet.delete(section);
    } else {
      newSet.add(section);
    }
    setExpandedSections(newSet);
  };

  const handleLabelBlur = () => {
    if (props.selectedNode && localLabel() !== props.selectedNode.label) {
      props.onLabelChange?.(props.selectedNode.id, localLabel());
    }
  };

  const handleDescriptionBlur = () => {
    if (props.selectedNode && localDescription() !== (props.selectedNode.description || "")) {
      props.onDescriptionChange?.(props.selectedNode.id, localDescription());
    }
  };

  const handlePropertyChange = (key: string, value: PropertyValue) => {
    if (props.selectedNode) {
      props.onPropertyChange?.(props.selectedNode.id, key, value);
    }
  };

  const getPortTypeColor = (portType: any): string => {
    if (typeof portType === "string") {
      return PORT_COLORS[portType] || "#666";
    }
    return PORT_COLORS["bus"] || "#666";
  };

  const renderPropertyInput = (key: string, value: PropertyValue) => {
    // String
    if (typeof value === "string") {
      return (
        <input
          type="text"
          value={value}
          onChange={(e) => handlePropertyChange(key, e.currentTarget.value)}
        />
      );
    }

    // Number
    if (typeof value === "number") {
      return (
        <input
          type="number"
          value={value}
          onChange={(e) => handlePropertyChange(key, parseFloat(e.currentTarget.value) || 0)}
        />
      );
    }

    // Boolean
    if (typeof value === "boolean") {
      return (
        <label class="toggle-switch">
          <input
            type="checkbox"
            checked={value}
            onChange={(e) => handlePropertyChange(key, e.currentTarget.checked)}
          />
          <span class="toggle-slider"></span>
        </label>
      );
    }

    // Enum
    if (typeof value === "object" && "options" in value) {
      return (
        <select
          value={value.value}
          onChange={(e) => handlePropertyChange(key, { ...value, value: e.currentTarget.value })}
        >
          <For each={value.options}>
            {(option) => <option value={option}>{option}</option>}
          </For>
        </select>
      );
    }

    // Pin
    if (typeof value === "object" && "port" in value && "pin" in value) {
      return (
        <div class="pin-input">
          <select
            value={value.port}
            onChange={(e) => handlePropertyChange(key, { ...value, port: e.currentTarget.value })}
          >
            <For each={["A", "B", "C", "D", "E", "F"]}>
              {(port) => <option value={port}>{port}</option>}
            </For>
          </select>
          <input
            type="number"
            min="0"
            max="15"
            value={value.pin}
            onChange={(e) => handlePropertyChange(key, { ...value, pin: parseInt(e.currentTarget.value) || 0 })}
          />
        </div>
      );
    }

    return <span class="unknown-type">Unknown type</span>;
  };

  const formatPropertyKey = (key: string): string => {
    return key
      .replace(/_/g, " ")
      .replace(/\b\w/g, (l) => l.toUpperCase());
  };

  return (
    <div class="property-panel">
      <Show when={!props.selectedNode}>
        <div class="no-selection">
          <div class="no-selection-icon">{Icons.package()}</div>
          <p>Select a node to view properties</p>
        </div>
      </Show>

      <Show when={props.selectedNode}>
        {(node) => (
          <>
            <div class="panel-header">
              <span class="node-type-badge">{node().node_type}</span>
              <span class="node-id">{node().id}</span>
            </div>

            {/* General Section */}
            <div class="section">
              <button 
                class="section-header" 
                onClick={() => toggleSection("general")}
              >
                <span>General</span>
                <span class={`chevron ${expandedSections().has("general") ? "expanded" : ""}`}>{Icons.chevronRight()}</span>
              </button>
              
              <Show when={expandedSections().has("general")}>
                <div class="section-content">
                  <div class="property-row">
                    <label>Label</label>
                    <input
                      type="text"
                      value={localLabel()}
                      onInput={(e) => setLocalLabel(e.currentTarget.value)}
                      onBlur={handleLabelBlur}
                    />
                  </div>
                  
                  <div class="property-row">
                    <label>Description</label>
                    <textarea
                      value={localDescription()}
                      onInput={(e) => setLocalDescription(e.currentTarget.value)}
                      onBlur={handleDescriptionBlur}
                      rows={2}
                    />
                  </div>

                  <div class="property-row readonly">
                    <label>Position</label>
                    <span>{Math.round(node().x)}, {Math.round(node().y)}</span>
                  </div>

                  <div class="property-row readonly">
                    <label>Size</label>
                    <span>{node().width} × {node().height}</span>
                  </div>
                </div>
              </Show>
            </div>

            {/* Properties Section */}
            <Show when={Object.keys(node().properties).length > 0}>
              <div class="section">
                <button 
                  class="section-header" 
                  onClick={() => toggleSection("properties")}
                >
                  <span>Properties</span>
                  <span class="property-count">{Object.keys(node().properties).length}</span>
                  <span class={`chevron ${expandedSections().has("properties") ? "expanded" : ""}`}>{Icons.chevronRight()}</span>
                </button>
                
                <Show when={expandedSections().has("properties")}>
                  <div class="section-content">
                    <For each={Object.entries(node().properties)}>
                      {([key, value]) => (
                        <div class="property-row">
                          <label>{formatPropertyKey(key)}</label>
                          {renderPropertyInput(key, value)}
                        </div>
                      )}
                    </For>
                  </div>
                </Show>
              </div>
            </Show>

            {/* Ports Section */}
            <div class="section">
              <button 
                class="section-header" 
                onClick={() => toggleSection("ports")}
              >
                <span>Ports</span>
                <span class="port-summary">
                  <span class="input-count">{node().ports.inputs.length} in</span>
                  <span class="output-count">{node().ports.outputs.length} out</span>
                </span>
                <span class={`chevron ${expandedSections().has("ports") ? "expanded" : ""}`}>{Icons.chevronRight()}</span>
              </button>
              
              <Show when={expandedSections().has("ports")}>
                <div class="section-content">
                  <Show when={node().ports.inputs.length > 0}>
                    <div class="ports-group">
                      <div class="ports-label">Inputs</div>
                      <For each={node().ports.inputs}>
                        {(port) => (
                          <div class="port-item">
                            <span 
                              class="port-dot" 
                              style={{ background: getPortTypeColor(port.port_type) }}
                            ></span>
                            <span class="port-name">{port.name}</span>
                            <span class="port-type">{typeof port.port_type === "string" ? port.port_type : "bus"}</span>
                            <Show when={port.required}>
                              <span class="port-required">*</span>
                            </Show>
                          </div>
                        )}
                      </For>
                    </div>
                  </Show>

                  <Show when={node().ports.outputs.length > 0}>
                    <div class="ports-group">
                      <div class="ports-label">Outputs</div>
                      <For each={node().ports.outputs}>
                        {(port) => (
                          <div class="port-item">
                            <span 
                              class="port-dot" 
                              style={{ background: getPortTypeColor(port.port_type) }}
                            ></span>
                            <span class="port-name">{port.name}</span>
                            <span class="port-type">{typeof port.port_type === "string" ? port.port_type : "bus"}</span>
                          </div>
                        )}
                      </For>
                    </div>
                  </Show>
                </div>
              </Show>
            </div>

            {/* Actions Section */}
            <Show when={node().node_type === "state" || node().node_type === "State"}>
              <div class="section">
                <button 
                  class="section-header" 
                  onClick={() => toggleSection("actions")}
                >
                  <span>Actions</span>
                  <span class={`chevron ${expandedSections().has("actions") ? "expanded" : ""}`}>{Icons.chevronRight()}</span>
                </button>
                
                <Show when={expandedSections().has("actions")}>
                  <div class="section-content">
                    <div class="property-row">
                      <label>Entry Action</label>
                      <textarea
                        value={node().entry_action || ""}
                        placeholder="Code on entry..."
                        rows={2}
                      />
                    </div>
                    <div class="property-row">
                      <label>Exit Action</label>
                      <textarea
                        value={node().exit_action || ""}
                        placeholder="Code on exit..."
                        rows={2}
                      />
                    </div>
                  </div>
                </Show>
              </div>
            </Show>
          </>
        )}
      </Show>
    </div>
  );
};

export default PropertyPanel;
