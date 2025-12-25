// ValidationPanel Component Tests
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { ValidationPanel } from '../components/ValidationPanel';

describe('ValidationPanel', () => {
  const mockOnLog = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the validation panel with header', () => {
    render(() => <ValidationPanel code="int main() {}" language="c" onLog={mockOnLog} />);
    
    expect(screen.getByText('🔍 Code Validation')).toBeInTheDocument();
  });

  it('shows validate button when code is provided', () => {
    render(() => <ValidationPanel code="int main() { return 0; }" language="c" onLog={mockOnLog} />);
    
    const button = screen.getByRole('button', { name: /validate code/i });
    expect(button).toBeInTheDocument();
    expect(button).not.toBeDisabled();
  });

  it('disables validate button when code is empty', () => {
    render(() => <ValidationPanel code="" language="c" onLog={mockOnLog} />);
    
    const button = screen.getByRole('button', { name: /validate code/i });
    expect(button).toBeDisabled();
  });

  it('has embedded mode checkbox checked by default', () => {
    render(() => <ValidationPanel code="int main() {}" language="c" onLog={mockOnLog} />);
    
    const checkbox = screen.getByRole('checkbox');
    expect(checkbox).toBeChecked();
  });

  it('calls onLog with success message after validation', async () => {
    render(() => <ValidationPanel code="int main() { return 0; }" language="c" onLog={mockOnLog} />);
    
    const button = screen.getByRole('button', { name: /validate code/i });
    await fireEvent.click(button);

    await waitFor(() => {
      expect(mockOnLog).toHaveBeenCalledWith(
        'Validate',
        expect.stringContaining('✅'),
        'success'
      );
    });
  });

  it('displays success result after validation', async () => {
    render(() => <ValidationPanel code="int main() { return 0; }" language="c" onLog={mockOnLog} />);
    
    const button = screen.getByRole('button', { name: /validate code/i });
    await fireEvent.click(button);

    await waitFor(() => {
      expect(screen.getByText(/✅ Valid/)).toBeInTheDocument();
    });
  });

  it('shows compiler info after validation', async () => {
    render(() => <ValidationPanel code="int x = 1;" language="c" onLog={mockOnLog} />);
    
    const button = screen.getByRole('button', { name: /validate code/i });
    await fireEvent.click(button);

    await waitFor(() => {
      expect(screen.getByText(/Compiler:/)).toBeInTheDocument();
    });
  });
});
