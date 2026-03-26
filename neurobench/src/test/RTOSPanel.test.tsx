// RTOSPanel Component Tests
// Tests for RTOS configuration and code generation

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { RTOSPanel } from '../components/RTOSPanel';

describe('RTOSPanel', () => {
  const mockOnLog = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('renders the RTOS panel', () => {
      render(() => <RTOSPanel onLog={mockOnLog} />);

      // Panel should render
      const panel = document.querySelector('.rtos-panel');
      expect(panel).toBeInTheDocument();
    });

    it('shows RTOS selection dropdown', () => {
      render(() => <RTOSPanel onLog={mockOnLog} />);

      // Should have a select element for RTOS
      const select = document.querySelector('select');
      expect(select).toBeInTheDocument();
    });
  });

  describe('Task Configuration', () => {
    it('shows task configuration inputs', () => {
      render(() => <RTOSPanel onLog={mockOnLog} />);

      // Should have input fields
      const inputs = document.querySelectorAll('input');
      expect(inputs.length).toBeGreaterThan(0);
    });
  });

  describe('Code Generation', () => {
    it('has generate buttons', () => {
      render(() => <RTOSPanel onLog={mockOnLog} />);

      // Should have generate buttons
      const buttons = document.querySelectorAll('button');
      expect(buttons.length).toBeGreaterThan(0);
    });

    it('can generate code', async () => {
      render(() => <RTOSPanel onLog={mockOnLog} />);

      // Find and click the first generate button
      const buttons = document.querySelectorAll('button');
      const generateBtn = Array.from(buttons).find(b => b.textContent?.includes('Generate'));
      
      if (generateBtn) {
        await fireEvent.click(generateBtn);
        
        await waitFor(() => {
          expect(mockOnLog).toHaveBeenCalled();
        });
      }
    });
  });

  describe('Code Preview', () => {
    it('shows code preview after generation', async () => {
      render(() => <RTOSPanel onLog={mockOnLog} />);

      // Find and click the first generate button
      const buttons = document.querySelectorAll('button');
      const generateBtn = Array.from(buttons).find(b => b.textContent?.includes('Generate'));
      
      if (generateBtn) {
        await fireEvent.click(generateBtn);

        await waitFor(() => {
          // Code preview should be visible (pre element)
          const codePreview = document.querySelector('pre');
          expect(codePreview).toBeInTheDocument();
        });
      }
    });
  });
});

