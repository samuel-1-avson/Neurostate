/**
 * NodePalette - Draggable node palette with categories
 */

import { Component, createSignal, onMount, For, Show } from "solid-js";
import { useNodeEngine, CATEGORY_INFO, NodeTypeInfo } from "../hooks/useNodeEngine";
import "./NodePalette.css";

interface NodePaletteProps {
  onNodeDragStart?: (nodeType: string, info: NodeTypeInfo) => void;
  onNodeClick?: (nodeType: string, info: NodeTypeInfo) => void;
}

const NodePalette: Component<NodePaletteProps> = (props) => {
  const nodeEngine = useNodeEngine();
  const [expandedCategories, setExpandedCategories] = createSignal<Set<string>>(new Set(["State Machine", "Hardware"]));
  const [searchQuery, setSearchQuery] = createSignal("");
  const [draggedNode, setDraggedNode] = createSignal<NodeTypeInfo | null>(null);

  onMount(() => {
    nodeEngine.loadPalette();
  });

  const toggleCategory = (category: string) => {
    const current = expandedCategories();
    const newSet = new Set(current);
    if (newSet.has(category)) {
      newSet.delete(category);
    } else {
      newSet.add(category);
    }
    setExpandedCategories(newSet);
  };

  const filteredPalette = () => {
    const query = searchQuery().toLowerCase();
    if (!query) return nodeEngine.palette();

    return nodeEngine.palette().map(([category, nodes]) => {
      const filtered = nodes.filter(n => 
        n.name.toLowerCase().includes(query) ||
        n.node_type.toLowerCase().includes(query)
      );
      return [category, filtered] as [string, NodeTypeInfo[]];
    }).filter(([_, nodes]) => nodes.length > 0);
  };

  const handleDragStart = (e: DragEvent, info: NodeTypeInfo) => {
    e.dataTransfer?.setData("application/node-type", JSON.stringify(info));
    setDraggedNode(info);
    props.onNodeDragStart?.(info.node_type, info);
  };

  const handleDragEnd = () => {
    setDraggedNode(null);
  };

  const getCategoryColor = (categoryName: string): string => {
    const cat = Object.values(CATEGORY_INFO).find(c => c.name === categoryName);
    return cat?.color || "#666";
  };

  const getCategoryIcon = (categoryName: string): string => {
    const cat = Object.values(CATEGORY_INFO).find(c => c.name === categoryName);
    return cat?.icon || "📦";
  };

  return (
    <div class="node-palette">
      <div class="palette-header">
        <h3>Nodes</h3>
        <span class="node-count">{nodeEngine.allTypes().length || "40+"}</span>
      </div>

      <div class="palette-search">
        <input
          type="text"
          placeholder="Search nodes..."
          value={searchQuery()}
          onInput={(e) => setSearchQuery(e.currentTarget.value)}
        />
        <Show when={searchQuery()}>
          <button class="clear-search" onClick={() => setSearchQuery("")}>×</button>
        </Show>
      </div>

      <div class="palette-categories">
        <Show when={nodeEngine.loading()}>
          <div class="palette-loading">Loading nodes...</div>
        </Show>

        <For each={filteredPalette()}>
          {([category, nodes]) => (
            <div class="category-section">
              <button 
                class="category-header"
                onClick={() => toggleCategory(category)}
                style={{ "--category-color": getCategoryColor(category) }}
              >
                <span class="category-icon">{getCategoryIcon(category)}</span>
                <span class="category-name">{category}</span>
                <span class="category-count">{nodes.length}</span>
                <span class={`category-chevron ${expandedCategories().has(category) ? "expanded" : ""}`}>
                  ▶
                </span>
              </button>

              <Show when={expandedCategories().has(category)}>
                <div class="category-nodes">
                  <For each={nodes}>
                    {(node) => (
                      <div
                        class={`palette-node ${draggedNode() === node ? "dragging" : ""}`}
                        draggable={true}
                        onDragStart={(e) => handleDragStart(e, node)}
                        onDragEnd={handleDragEnd}
                        onClick={() => props.onNodeClick?.(node.node_type, node)}
                        title={`${node.name}\nInputs: ${node.ports.inputs.length}\nOutputs: ${node.ports.outputs.length}`}
                      >
                        <span class="node-icon">{node.icon}</span>
                        <span class="node-name">{node.name}</span>
                        <span class="port-indicators">
                          <Show when={node.ports.inputs.length > 0}>
                            <span class="port-count input">{node.ports.inputs.length}</span>
                          </Show>
                          <Show when={node.ports.outputs.length > 0}>
                            <span class="port-count output">{node.ports.outputs.length}</span>
                          </Show>
                        </span>
                      </div>
                    )}
                  </For>
                </div>
              </Show>
            </div>
          )}
        </For>
      </div>

      <div class="palette-footer">
        <span>Drag nodes to canvas</span>
      </div>
    </div>
  );
};

export default NodePalette;
