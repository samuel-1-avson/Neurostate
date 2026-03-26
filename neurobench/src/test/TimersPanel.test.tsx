// TimersPanel Component Tests
// Tests for timer and interrupt configuration

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { TimersPanel } from '../components/TimersPanel';

describe('TimersPanel', () => {
  const mockOnLog = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('renders the timers panel', () => {
      render(() => <TimersPanel onLog={mockOnLog} />);

      // Should have timer configuration elements
      const panel = document.querySelector('.timers-panel');
      expect(panel).toBeInTheDocument();
    });

    it('shows timer configuration inputs', () => {
      render(() => <TimersPanel onLog={mockOnLog} />);

      // Should have input fields
      const inputs = document.querySelectorAll('input');
      expect(inputs.length).toBeGreaterThan(0);
    });

    it('shows timer instance selector', () => {
      render(() => <TimersPanel onLog={mockOnLog} />);

      const selects = document.querySelectorAll('select');
      expect(selects.length).toBeGreaterThan(0);
    });
  });

  describe('Timer Controls', () => {
    it('has generate buttons', () => {
      render(() => <TimersPanel onLog={mockOnLog} />);

      const buttons = document.querySelectorAll('button');
      expect(buttons.length).toBeGreaterThan(0);
    });
  });
});

