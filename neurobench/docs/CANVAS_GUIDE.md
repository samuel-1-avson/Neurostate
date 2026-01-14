# NeuroBench Canvas Architecture Guide

## Overview

NeuroBench has two canvas implementations for FSM visualization:

| Canvas | File | Lines | Status |
|--------|------|-------|--------|
| **KonvaCanvas** | `src/components/KonvaCanvas.tsx` | ~1560 | Production |
| **UnifiedCanvas** | `src/components/UnifiedCanvas.tsx` | ~1500 | Default (new) |

## Which Canvas to Use?

### UnifiedCanvas (Default)

The application defaults to `UnifiedCanvas` via the `useUnifiedCanvas` signal in `App.tsx`.

**Features:**
- Built-in node palette with 40+ node types
- Integrated code generation panel
- AI-assisted node creation modal
- Optimistic drag updates (lower latency)
- Unified toolbar with minimap toggle

**Best for:**
- Full FSM design workflow
- Code generation with preview
- AI-assisted development

### KonvaCanvas (Classic)

Konva.js-based canvas with mature rendering.

**Features:**
- Konva.js rendering engine
- Traditional canvas controls
- Context menus for node/edge operations

**Best for:**
- Compatibility with existing Konva plugins
- Simpler standalone canvas needs

## Switching Modes

In `App.tsx`, the canvas mode is controlled by:

```tsx
// Line ~197
const [useUnifiedCanvas, setUseUnifiedCanvas] = createSignal(true);
```

Toggle via the "↩ Classic" button in UnifiedCanvas toolbar.

## Shared Components

Both canvases share:
- `useCanvasEngine.ts` - Backend state management
- `useNodeEngine.ts` - Node type definitions
- `TransitionEditor.tsx` - Edge/transition editing

## Future Roadmap

1. **Short-term**: Document differences and stabilize UnifiedCanvas as primary
2. **Medium-term**: Extract shared canvas utilities
3. **Long-term**: Consider consolidating into single implementation

## Key Integration Points

### Backend (lib.rs)

Canvas operations go through IPC commands:
- `canvas_init` - Initialize canvas state
- `canvas_add_node` - Add new nodes
- `canvas_connect` - Create edges
- `canvas_move_node` - Update positions
- `canvas_undo/redo` - History operations

### Hooks

- `useCanvasEngine.ts` - Manages canvas state, nodes, edges
- `useNodeEngine.ts` - Node type palette and metadata
- `useTransitionEngine.ts` - Transition event configurations
