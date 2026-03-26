// BuildPanel Component Tests
// Tests for the build, flash, and debug workflow panel

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { BuildPanel } from '../components/BuildPanel';

// Mock Tauri event system
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

describe('BuildPanel', () => {
  const mockOnLog = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('renders the build panel with tabs', () => {
      render(() => <BuildPanel onLog={mockOnLog} />);

      // Tab bar should be visible - using getAllByText since Build appears in both tab and button
      const buildElements = screen.getAllByText('🔨 Build');
      expect(buildElements.length).toBeGreaterThan(0);
      expect(screen.getByText('⚡ Flash')).toBeInTheDocument();
      expect(screen.getByText('📡 RTT')).toBeInTheDocument();
    });

    it('shows toolchain section in build tab', () => {
      render(() => <BuildPanel onLog={mockOnLog} />);

      expect(screen.getByText('Toolchain')).toBeInTheDocument();
    });

    it('shows optimization options', () => {
      render(() => <BuildPanel onLog={mockOnLog} />);

      expect(screen.getByText('Optimization')).toBeInTheDocument();
    });
  });

  describe('Build Controls', () => {
    it('has clean button', () => {
      render(() => <BuildPanel onLog={mockOnLog} />);

      expect(screen.getByText('🗑️ Clean')).toBeInTheDocument();
    });
  });

  describe('Flash Tab', () => {
    it('can switch to flash tab', async () => {
      render(() => <BuildPanel onLog={mockOnLog} />);

      const flashTab = screen.getByText('⚡ Flash');
      await fireEvent.click(flashTab);

      await waitFor(() => {
        expect(screen.getByText('Debug Probe')).toBeInTheDocument();
      });
    });
  });

  describe('RTT Tab', () => {
    it('can switch to RTT tab', async () => {
      render(() => <BuildPanel onLog={mockOnLog} />);

      const rttTab = screen.getByText('📡 RTT');
      await fireEvent.click(rttTab);

      await waitFor(() => {
        expect(screen.getByText('▶️ Start RTT')).toBeInTheDocument();
      });
    });
  });

  describe('Diagnostics Tab', () => {
    it('can switch to diagnostics tab', async () => {
      render(() => <BuildPanel onLog={mockOnLog} />);

      const diagTab = screen.getByText('🔍 Diagnostics');
      await fireEvent.click(diagTab);

      await waitFor(() => {
        expect(screen.getByText('Build Diagnostics')).toBeInTheDocument();
      });
    });
  });
});

