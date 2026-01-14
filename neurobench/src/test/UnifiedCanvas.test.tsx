// UnifiedCanvas Component Tests
// Comprehensive test suite for the main canvas editor

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import UnifiedCanvas from '../components/UnifiedCanvas';

// Mock the hooks
vi.mock('../hooks/useCanvasEngine', () => ({
  useCanvasEngine: () => ({
    state: vi.fn(() => ({
      nodes: [
        { id: 'node1', label: 'Start', node_type: 'input', x: 100, y: 100, width: 180, height: 100 },
        { id: 'node2', label: 'Process', node_type: 'process', x: 300, y: 100, width: 180, height: 100 },
      ],
      edges: [
        { id: 'edge1', source: 'node1', target: 'node2', label: 'next' }
      ],
      selection: [],
    })),
    edgePaths: vi.fn(() => ({})),
    validation: vi.fn(() => null),
    loading: vi.fn(() => false),
    error: vi.fn(() => null),
    init: vi.fn(async () => {}),
    refreshState: vi.fn(async () => {}),
    addNode: vi.fn(async () => {}),
    moveNode: vi.fn(async () => {}),
    moveNodes: vi.fn(async () => {}),
    deleteNodes: vi.fn(async () => ({ deleted_nodes: [], deleted_edges: [] })),
    updateNode: vi.fn(async () => {}),
    connect: vi.fn(async () => ({ id: 'new_edge', source: '', target: '', label: '' })),
    deleteEdges: vi.fn(async () => {}),
    queryAt: vi.fn(async () => null),
    queryRect: vi.fn(async () => []),
    select: vi.fn(async () => {}),
    selectAll: vi.fn(async () => {}),
    clearSelection: vi.fn(async () => {}),
    validate: vi.fn(async () => ({ is_valid: true, errors: [], warnings: [] })),
    autoLayout: vi.fn(async () => {}),
    align: vi.fn(async () => {}),
    undo: vi.fn(async () => {}),
    redo: vi.fn(async () => {}),
    executeBatch: vi.fn(async () => null),
    getViewport: vi.fn(async () => null),
    setViewport: vi.fn(async () => {}),
    zoomTo: vi.fn(async () => {}),
    fitToContent: vi.fn(async () => {}),
    createLayer: vi.fn(async () => null),
    deleteLayer: vi.fn(async () => {}),
    setLayerVisible: vi.fn(async () => {}),
    setLayerLocked: vi.fn(async () => {}),
    createGroup: vi.fn(async () => null),
    ungroup: vi.fn(async () => {}),
    collapseGroup: vi.fn(async () => {}),
    expandGroup: vi.fn(async () => {}),
  }),
}));

vi.mock('../hooks/useNodeEngine', () => ({
  useNodeEngine: () => ({
    allTypes: vi.fn(() => [
      { node_type: 'state', name: 'State', category: 'fsm', icon: '●' },
      { node_type: 'gpio', name: 'GPIO', category: 'hardware', icon: '📍' },
      { node_type: 'timer', name: 'Timer', category: 'hardware', icon: '⏱️' },
    ]),
    palette: vi.fn(() => []),
    loading: vi.fn(() => false),
    loadPalette: vi.fn(async () => []),
    loadAllTypes: vi.fn(async () => []),
    getTypeInfo: vi.fn(() => null),
  }),
  CATEGORY_INFO: {
    fsm: { color: '#4CAF50', label: 'FSM' },
    hardware: { color: '#2196F3', label: 'Hardware' },
    control: { color: '#FF9800', label: 'Control' },
    io: { color: '#9C27B0', label: 'I/O' },
    data: { color: '#00BCD4', label: 'Data' },
    processing: { color: '#E91E63', label: 'Processing' },
  },
}));

describe('UnifiedCanvas', () => {
  const mockOnNodesChange = vi.fn();
  const mockOnEdgesChange = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('renders the canvas component', () => {
      render(() => (
        <UnifiedCanvas
          projectName="Test Project"
          onNodesChange={mockOnNodesChange}
          onEdgesChange={mockOnEdgesChange}
        />
      ));

      // Canvas should render - check for the unified badge
      expect(screen.getByText('UNIFIED')).toBeInTheDocument();
    });

    it('displays project name', () => {
      render(() => (
        <UnifiedCanvas
          projectName="My Test Project"
          onNodesChange={mockOnNodesChange}
        />
      ));

      expect(screen.getByText('My Test Project')).toBeInTheDocument();
    });

    it('displays node count', () => {
      render(() => (
        <UnifiedCanvas
          projectName="Test"
          onNodesChange={mockOnNodesChange}
        />
      ));

      expect(screen.getByText('2 nodes')).toBeInTheDocument();
    });

    it('shows node palette by default', () => {
      render(() => (
        <UnifiedCanvas projectName="Test" />
      ));

      expect(screen.getByText('Nodes')).toBeInTheDocument();
    });
  });

  describe('Toolbar', () => {
    it('displays zoom level', () => {
      render(() => <UnifiedCanvas projectName="Test" />);

      expect(screen.getByText('100%')).toBeInTheDocument();
    });

    it('has undo and redo buttons', () => {
      render(() => <UnifiedCanvas projectName="Test" />);

      expect(screen.getByText('↶')).toBeInTheDocument();
      expect(screen.getByText('↷')).toBeInTheDocument();
    });

    it('has generate code button', () => {
      render(() => <UnifiedCanvas projectName="Test" />);

      expect(screen.getByText('⚡ Generate')).toBeInTheDocument();
    });

    it('has AI button', () => {
      render(() => <UnifiedCanvas projectName="Test" />);

      expect(screen.getByText('🪄 AI')).toBeInTheDocument();
    });

    it('has snap to grid toggle', () => {
      render(() => <UnifiedCanvas projectName="Test" />);

      expect(screen.getByTitle('Snap')).toBeInTheDocument();
    });

    it('has layout button', () => {
      render(() => <UnifiedCanvas projectName="Test" />);

      expect(screen.getByTitle('Layout')).toBeInTheDocument();
    });

    it('has minimap toggle', () => {
      render(() => <UnifiedCanvas projectName="Test" />);

      expect(screen.getByTitle('Minimap')).toBeInTheDocument();
    });
  });

  describe('Node Palette', () => {
    it('shows search input', () => {
      render(() => <UnifiedCanvas projectName="Test" />);

      expect(screen.getByPlaceholderText('Search nodes...')).toBeInTheDocument();
    });

    it('shows All category button', () => {
      render(() => <UnifiedCanvas projectName="Test" />);

      expect(screen.getByText('All')).toBeInTheDocument();
    });

    it('shows node types from palette', () => {
      render(() => <UnifiedCanvas projectName="Test" />);

      expect(screen.getByText('State')).toBeInTheDocument();
      expect(screen.getByText('GPIO')).toBeInTheDocument();
      expect(screen.getByText('Timer')).toBeInTheDocument();
    });

    it('can filter nodes by search', async () => {
      render(() => <UnifiedCanvas projectName="Test" />);

      const searchInput = screen.getByPlaceholderText('Search nodes...');
      await fireEvent.input(searchInput, { target: { value: 'gpio' } });

      // GPIO should still be visible, others may be filtered
      expect(screen.getByText('GPIO')).toBeInTheDocument();
    });

    it('can toggle palette visibility', async () => {
      render(() => <UnifiedCanvas projectName="Test" />);

      // Initially visible
      expect(screen.getByText('Nodes')).toBeInTheDocument();

      // Click collapse button - use title for reliable matching
      const toggleBtn = screen.getByTitle('Collapse Palette');
      await fireEvent.click(toggleBtn);

      // After collapse, expand button should be visible
      await waitFor(() => {
        expect(screen.queryByText('Nodes')).not.toBeInTheDocument();
      });
    });

  });

  describe('Zoom Controls', () => {
    it('shows zoom in button', () => {
      render(() => <UnifiedCanvas projectName="Test" />);

      expect(screen.getByTitle('Zoom In')).toBeInTheDocument();
    });

    it('shows zoom out button', () => {
      render(() => <UnifiedCanvas projectName="Test" />);

      expect(screen.getByTitle('Zoom Out')).toBeInTheDocument();
    });

    it('shows fit button', () => {
      render(() => <UnifiedCanvas projectName="Test" />);

      expect(screen.getByTitle('Fit')).toBeInTheDocument();
    });
  });

  describe('Code Generation', () => {
    it('has generate button in toolbar', () => {
      render(() => <UnifiedCanvas projectName="Test" />);

      const generateBtn = screen.getByText('⚡ Generate');
      expect(generateBtn).toBeInTheDocument();
    });

    it('has code panel toggle', () => {
      render(() => <UnifiedCanvas projectName="Test" />);

      // Code panel toggle button - use title for reliable matching
      expect(screen.getByTitle('Toggle Code Panel')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('canvas container is focusable', () => {
      render(() => <UnifiedCanvas projectName="Test" />);

      const container = document.querySelector('.unified-canvas');
      expect(container).toHaveAttribute('tabindex', '0');
    });
  });

  describe('Default Props', () => {
    it('renders without optional props', () => {
      render(() => <UnifiedCanvas />);

      // Should render with default project name
      expect(screen.getByText('Project')).toBeInTheDocument();
    });
  });
});
