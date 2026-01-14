// PeripheralsPanel Component Tests
// Tests for SPI, I2C, UART peripheral configuration

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { PeripheralsPanel } from '../components/PeripheralsPanel';

describe('PeripheralsPanel', () => {
  const mockOnLog = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('renders the peripherals panel', () => {
      render(() => <PeripheralsPanel onLog={mockOnLog} />);

      const panel = document.querySelector('.peripherals-panel');
      expect(panel).toBeInTheDocument();
    });

    it('shows peripheral tabs or sections', () => {
      render(() => <PeripheralsPanel onLog={mockOnLog} />);

      // Should have buttons for different peripherals
      const buttons = document.querySelectorAll('button');
      expect(buttons.length).toBeGreaterThan(0);
    });

    it('shows input fields for configuration', () => {
      render(() => <PeripheralsPanel onLog={mockOnLog} />);

      const inputs = document.querySelectorAll('input, select');
      expect(inputs.length).toBeGreaterThan(0);
    });
  });

  describe('Peripheral Selection', () => {
    it('has generate buttons for peripherals', () => {
      render(() => <PeripheralsPanel onLog={mockOnLog} />);

      const buttons = document.querySelectorAll('button');
      expect(buttons.length).toBeGreaterThan(0);
    });
  });
});

