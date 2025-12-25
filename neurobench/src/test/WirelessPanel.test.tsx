// WirelessPanel Component Tests
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { WirelessPanel } from '../components/WirelessPanel';

describe('WirelessPanel', () => {
  const mockOnLog = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the wireless panel with header', () => {
    render(() => <WirelessPanel onLog={mockOnLog} />);
    
    expect(screen.getByText('📡 Wireless Configuration')).toBeInTheDocument();
  });

  it('shows all protocol tabs', () => {
    render(() => <WirelessPanel onLog={mockOnLog} />);
    
    expect(screen.getByText(/🔷 BLE/)).toBeInTheDocument();
    expect(screen.getByText(/📶 WiFi/)).toBeInTheDocument();
    expect(screen.getByText(/📻 LoRa/)).toBeInTheDocument();
  });

  it('defaults to BLE tab with device name', () => {
    render(() => <WirelessPanel onLog={mockOnLog} />);
    
    expect(screen.getByText('Device Name')).toBeInTheDocument();
    expect(screen.getByText('Service UUID')).toBeInTheDocument();
  });

  it('switches to WiFi tab when clicked', async () => {
    render(() => <WirelessPanel onLog={mockOnLog} />);
    
    await fireEvent.click(screen.getByText(/📶 WiFi/));
    
    expect(screen.getByText('SSID')).toBeInTheDocument();
  });

  it('switches to LoRa tab when clicked', async () => {
    render(() => <WirelessPanel onLog={mockOnLog} />);
    
    await fireEvent.click(screen.getByText(/📻 LoRa/));
    
    expect(screen.getByText('Frequency (MHz)')).toBeInTheDocument();
  });

  it('generates BLE code and shows code preview', async () => {
    render(() => <WirelessPanel onLog={mockOnLog} />);
    
    const generateButton = screen.getByText('Generate BLE Service');
    await fireEvent.click(generateButton);

    await waitFor(() => {
      // Look for language badge in CodePreview
      expect(screen.getByText('C')).toBeInTheDocument();
    });
  });

  it('calls onLog after generating BLE code', async () => {
    render(() => <WirelessPanel onLog={mockOnLog} />);
    
    const generateButton = screen.getByText('Generate BLE Service');
    await fireEvent.click(generateButton);

    await waitFor(() => {
      expect(mockOnLog).toHaveBeenCalledWith(
        'BLE',
        expect.stringContaining('GATT'),
        'success'
      );
    });
  });

  it('shows platform selector with nRF52 option', () => {
    render(() => <WirelessPanel onLog={mockOnLog} />);
    
    expect(screen.getByText('Platform')).toBeInTheDocument();
    expect(screen.getByText('Nordic nRF52')).toBeInTheDocument();
  });
});
