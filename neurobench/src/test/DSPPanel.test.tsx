// DSPPanel Component Tests
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { DSPPanel } from '../components/DSPPanel';

describe('DSPPanel', () => {
  const mockOnLog = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the DSP panel with header', () => {
    render(() => <DSPPanel onLog={mockOnLog} />);
    
    expect(screen.getByText('📊 DSP Configuration')).toBeInTheDocument();
  });

  it('shows all DSP type tabs', () => {
    render(() => <DSPPanel onLog={mockOnLog} />);
    
    expect(screen.getByText('FIR')).toBeInTheDocument();
    expect(screen.getByText('IIR')).toBeInTheDocument();
    expect(screen.getByText('FFT')).toBeInTheDocument();
    expect(screen.getByText('PID')).toBeInTheDocument();
    expect(screen.getByText('Buffer')).toBeInTheDocument();
  });

  it('defaults to FIR tab with filter name input', () => {
    render(() => <DSPPanel onLog={mockOnLog} />);
    
    expect(screen.getByText('Filter Name')).toBeInTheDocument();
    expect(screen.getByText('Order (Taps - 1)')).toBeInTheDocument();
  });

  it('switches to IIR tab when clicked', async () => {
    render(() => <DSPPanel onLog={mockOnLog} />);
    
    await fireEvent.click(screen.getByText('IIR'));
    
    expect(screen.getByText('Q Factor')).toBeInTheDocument();
  });

  it('switches to PID tab when clicked', async () => {
    render(() => <DSPPanel onLog={mockOnLog} />);
    
    await fireEvent.click(screen.getByText('PID'));
    
    expect(screen.getByText('Controller Name')).toBeInTheDocument();
    expect(screen.getByText('Kp')).toBeInTheDocument();
  });

  it('switches to FFT tab when clicked', async () => {
    render(() => <DSPPanel onLog={mockOnLog} />);
    
    await fireEvent.click(screen.getByText('FFT'));
    
    expect(screen.getByText('FFT Name')).toBeInTheDocument();
    expect(screen.getByText('Size (Power of 2)')).toBeInTheDocument();
  });

  it('generates FIR code and shows code preview', async () => {
    render(() => <DSPPanel onLog={mockOnLog} />);
    
    const generateButton = screen.getByText('Generate FIR Filter');
    await fireEvent.click(generateButton);

    await waitFor(() => {
      // Look for language badge in CodePreview
      expect(screen.getByText('C')).toBeInTheDocument();
    });
  });

  it('calls onLog after generating code', async () => {
    render(() => <DSPPanel onLog={mockOnLog} />);
    
    const generateButton = screen.getByText('Generate FIR Filter');
    await fireEvent.click(generateButton);

    await waitFor(() => {
      expect(mockOnLog).toHaveBeenCalledWith(
        'DSP',
        expect.stringContaining('FIR filter'),
        'success'
      );
    });
  });

  it('shows copy button when code is generated', async () => {
    render(() => <DSPPanel onLog={mockOnLog} />);
    
    const generateButton = screen.getByText('Generate FIR Filter');
    await fireEvent.click(generateButton);

    await waitFor(() => {
      expect(screen.getByText('📋 Copy')).toBeInTheDocument();
    });
  });
});
