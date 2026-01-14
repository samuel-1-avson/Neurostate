// SecurityPanel Component Tests
// Tests for the security configuration panel

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { SecurityPanel } from '../components/SecurityPanel';

describe('SecurityPanel', () => {
  const mockOnLog = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('renders the security panel', () => {
      render(() => <SecurityPanel onLog={mockOnLog} />);

      // Panel should render
      const panel = document.querySelector('.security-panel');
      expect(panel).toBeInTheDocument();
    });

    it('shows tabs', () => {
      render(() => <SecurityPanel onLog={mockOnLog} />);

      // Should have tab buttons
      const buttons = document.querySelectorAll('button');
      expect(buttons.length).toBeGreaterThan(0);
    });
  });

  describe('Generation', () => {
    it('has generate buttons', () => {
      render(() => <SecurityPanel onLog={mockOnLog} />);

      const buttons = document.querySelectorAll('button');
      const generateBtns = Array.from(buttons).filter(b => 
        b.textContent?.toLowerCase().includes('generate')
      );
      expect(generateBtns.length).toBeGreaterThan(0);
    });

    it('can generate code', async () => {
      render(() => <SecurityPanel onLog={mockOnLog} />);

      const buttons = document.querySelectorAll('button');
      const generateBtn = Array.from(buttons).find(b => 
        b.textContent?.toLowerCase().includes('generate')
      );

      if (generateBtn) {
        await fireEvent.click(generateBtn);

        await waitFor(() => {
          expect(mockOnLog).toHaveBeenCalled();
        });
      }
    });
  });

  describe('Code Preview', () => {
    it('shows code after generation', async () => {
      render(() => <SecurityPanel onLog={mockOnLog} />);

      const buttons = document.querySelectorAll('button');
      const generateBtn = Array.from(buttons).find(b => 
        b.textContent?.toLowerCase().includes('generate')
      );

      if (generateBtn) {
        await fireEvent.click(generateBtn);

        await waitFor(() => {
          const codePreview = document.querySelector('pre');
          expect(codePreview).toBeInTheDocument();
        });
      }
    });
  });
});

