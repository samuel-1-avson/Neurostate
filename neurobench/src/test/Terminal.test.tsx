// Terminal Component Tests
// Tests for the embedded development terminal

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import Terminal from '../components/Terminal';

describe('Terminal', () => {
  const mockOnCommand = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('renders the terminal container', () => {
      render(() => <Terminal onCommand={mockOnCommand} />);

      // Terminal has an input for commands
      const input = document.querySelector('.terminal-input');
      expect(input).toBeInTheDocument();
    });

    it('shows welcome message', () => {
      render(() => <Terminal onCommand={mockOnCommand} />);

      expect(screen.getByText(/NeuroBench Terminal/)).toBeInTheDocument();
    });

    it('shows command prompt', () => {
      render(() => <Terminal onCommand={mockOnCommand} />);

      // Look for the prompt character
      expect(screen.getByText('❯')).toBeInTheDocument();
    });

    it('shows ready status', () => {
      render(() => <Terminal onCommand={mockOnCommand} />);

      expect(screen.getByText(/Ready/)).toBeInTheDocument();
    });
  });

  describe('Toolbar', () => {
    it('has clear button', () => {
      render(() => <Terminal onCommand={mockOnCommand} />);

      expect(screen.getByText('Clear')).toBeInTheDocument();
    });

    it('has search button', () => {
      render(() => <Terminal onCommand={mockOnCommand} />);

      expect(screen.getByText('🔍')).toBeInTheDocument();
    });
  });

  describe('Input Handling', () => {
    it('can type in command input', async () => {
      render(() => <Terminal onCommand={mockOnCommand} />);

      const input = document.querySelector('.terminal-input') as HTMLInputElement;
      await fireEvent.input(input, { target: { value: 'help' } });

      expect(input.value).toBe('help');
    });

    it('clears input after command execution', async () => {
      render(() => <Terminal onCommand={mockOnCommand} />);

      const input = document.querySelector('.terminal-input') as HTMLInputElement;
      await fireEvent.input(input, { target: { value: 'help' } });
      await fireEvent.keyDown(input, { key: 'Enter' });

      // Input should be cleared after executing command
      await waitFor(() => {
        expect(input.value).toBe('');
      });
    });
  });

  describe('Footer', () => {
    it('shows hint text', () => {
      render(() => <Terminal onCommand={mockOnCommand} />);

      expect(screen.getByText(/Tab: complete/)).toBeInTheDocument();
    });
  });
});

