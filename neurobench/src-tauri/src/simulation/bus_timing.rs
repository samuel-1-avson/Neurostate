//! Bus Timing Analysis
//!
//! Analyzes timing of communication buses (SPI, I2C, UART, CAN).

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Timing event type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TimingEventType {
    Start,
    Stop,
    ByteStart,
    ByteEnd,
    Ack,
    Nack,
    ClockEdge,
    DataChange,
    Idle,
    Error,
}

/// Timing event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingEvent {
    pub event_type: TimingEventType,
    pub timestamp_ns: u64,
    pub bus: String,
    pub data: Option<u8>,
}

/// Bus timing analyzer
pub struct BusTimingAnalyzer {
    /// Maximum buffer size
    max_events: usize,
    /// Event buffer (ring buffer)
    events: VecDeque<TimingEvent>,
    /// Current timestamp (ns)
    current_time_ns: u64,
    /// Clock frequency (Hz)
    clock_hz: u32,
}

impl BusTimingAnalyzer {
    pub fn new(clock_hz: u32) -> Self {
        Self {
            max_events: 10000,
            events: VecDeque::new(),
            current_time_ns: 0,
            clock_hz,
        }
    }

    /// Advance time by cycles
    pub fn tick(&mut self, cycles: u64) {
        let ns = (cycles as u128 * 1_000_000_000) / self.clock_hz as u128;
        self.current_time_ns += ns as u64;
    }

    /// Record an event
    pub fn record(&mut self, bus: &str, event_type: TimingEventType, data: Option<u8>) {
        let event = TimingEvent {
            event_type,
            timestamp_ns: self.current_time_ns,
            bus: bus.to_string(),
            data,
        };

        self.events.push_back(event);
        
        if self.events.len() > self.max_events {
            self.events.pop_front();
        }
    }

    /// Get events for a specific bus
    pub fn get_events(&self, bus: &str) -> Vec<&TimingEvent> {
        self.events.iter().filter(|e| e.bus == bus).collect()
    }

    /// Get all events
    pub fn get_all_events(&self) -> Vec<&TimingEvent> {
        self.events.iter().collect()
    }

    /// Analyze I2C timing
    pub fn analyze_i2c(&self, bus: &str) -> I2cTimingAnalysis {
        let events: Vec<_> = self.get_events(bus);
        
        let mut start_time = None;
        let mut byte_times = Vec::new();
        let mut last_byte_start = None;
        let mut ack_times = Vec::new();
        
        for event in &events {
            match event.event_type {
                TimingEventType::Start => {
                    start_time = Some(event.timestamp_ns);
                }
                TimingEventType::ByteStart => {
                    last_byte_start = Some(event.timestamp_ns);
                }
                TimingEventType::ByteEnd => {
                    if let Some(start) = last_byte_start {
                        byte_times.push(event.timestamp_ns - start);
                    }
                }
                TimingEventType::Ack | TimingEventType::Nack => {
                    if let Some(start) = last_byte_start {
                        ack_times.push(event.timestamp_ns - start);
                    }
                }
                _ => {}
            }
        }

        let avg_byte_time = if byte_times.is_empty() {
            0
        } else {
            byte_times.iter().sum::<u64>() / byte_times.len() as u64
        };

        let effective_clock_hz = if avg_byte_time > 0 {
            (9_000_000_000 / avg_byte_time) as u32 // 9 bits per byte
        } else {
            0
        };

        I2cTimingAnalysis {
            transaction_count: events.iter().filter(|e| matches!(e.event_type, TimingEventType::Start)).count(),
            avg_byte_time_ns: avg_byte_time,
            effective_clock_hz,
            ack_count: events.iter().filter(|e| matches!(e.event_type, TimingEventType::Ack)).count(),
            nack_count: events.iter().filter(|e| matches!(e.event_type, TimingEventType::Nack)).count(),
        }
    }

    /// Analyze SPI timing
    pub fn analyze_spi(&self, bus: &str) -> SpiTimingAnalysis {
        let events: Vec<_> = self.get_events(bus);
        
        let mut byte_times = Vec::new();
        let mut last_byte_start = None;
        let mut clock_edges = 0;
        
        for event in &events {
            match event.event_type {
                TimingEventType::ByteStart => {
                    last_byte_start = Some(event.timestamp_ns);
                }
                TimingEventType::ByteEnd => {
                    if let Some(start) = last_byte_start {
                        byte_times.push(event.timestamp_ns - start);
                        last_byte_start = None;
                    }
                }
                TimingEventType::ClockEdge => {
                    clock_edges += 1;
                }
                _ => {}
            }
        }

        let avg_byte_time = if byte_times.is_empty() {
            0
        } else {
            byte_times.iter().sum::<u64>() / byte_times.len() as u64
        };

        let effective_clock_hz = if avg_byte_time > 0 {
            (8_000_000_000 / avg_byte_time) as u32
        } else {
            0
        };

        let total_bytes = byte_times.len();
        let throughput_bps = if let (Some(first), Some(last)) = (
            events.first().map(|e| e.timestamp_ns),
            events.last().map(|e| e.timestamp_ns),
        ) {
            let duration_s = (last - first) as f64 / 1_000_000_000.0;
            if duration_s > 0.0 {
                (total_bytes * 8) as f64 / duration_s
            } else {
                0.0
            }
        } else {
            0.0
        };

        SpiTimingAnalysis {
            total_bytes,
            avg_byte_time_ns: avg_byte_time,
            effective_clock_hz,
            clock_edges,
            throughput_bps,
        }
    }

    /// Analyze UART timing
    pub fn analyze_uart(&self, bus: &str) -> UartTimingAnalysis {
        let events: Vec<_> = self.get_events(bus);
        
        let mut byte_times = Vec::new();
        let mut last_start = None;
        let mut errors = 0;
        
        for event in &events {
            match event.event_type {
                TimingEventType::Start => {
                    last_start = Some(event.timestamp_ns);
                }
                TimingEventType::Stop => {
                    if let Some(start) = last_start {
                        byte_times.push(event.timestamp_ns - start);
                        last_start = None;
                    }
                }
                TimingEventType::Error => {
                    errors += 1;
                }
                _ => {}
            }
        }

        let avg_byte_time = if byte_times.is_empty() {
            0
        } else {
            byte_times.iter().sum::<u64>() / byte_times.len() as u64
        };

        // UART: 10 bits per byte (start + 8 data + stop)
        let effective_baud = if avg_byte_time > 0 {
            (10_000_000_000 / avg_byte_time) as u32
        } else {
            0
        };

        UartTimingAnalysis {
            bytes_transmitted: byte_times.len(),
            avg_byte_time_ns: avg_byte_time,
            effective_baud,
            errors,
        }
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Get timing summary
    pub fn get_summary(&self) -> TimingSummary {
        TimingSummary {
            total_events: self.events.len(),
            current_time_ns: self.current_time_ns,
            buses: self.events.iter()
                .map(|e| e.bus.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect(),
        }
    }
}

impl Default for BusTimingAnalyzer {
    fn default() -> Self {
        Self::new(16_000_000)
    }
}

/// I2C timing analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I2cTimingAnalysis {
    pub transaction_count: usize,
    pub avg_byte_time_ns: u64,
    pub effective_clock_hz: u32,
    pub ack_count: usize,
    pub nack_count: usize,
}

/// SPI timing analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiTimingAnalysis {
    pub total_bytes: usize,
    pub avg_byte_time_ns: u64,
    pub effective_clock_hz: u32,
    pub clock_edges: usize,
    pub throughput_bps: f64,
}

/// UART timing analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UartTimingAnalysis {
    pub bytes_transmitted: usize,
    pub avg_byte_time_ns: u64,
    pub effective_baud: u32,
    pub errors: usize,
}

/// Overall timing summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingSummary {
    pub total_events: usize,
    pub current_time_ns: u64,
    pub buses: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_recording() {
        let mut analyzer = BusTimingAnalyzer::new(16_000_000);
        
        analyzer.record("I2C1", TimingEventType::Start, None);
        analyzer.tick(1000);
        analyzer.record("I2C1", TimingEventType::ByteStart, Some(0x50));
        analyzer.tick(8000);
        analyzer.record("I2C1", TimingEventType::ByteEnd, None);
        analyzer.record("I2C1", TimingEventType::Ack, None);
        
        let events = analyzer.get_events("I2C1");
        assert_eq!(events.len(), 4);
    }

    #[test]
    fn test_i2c_analysis() {
        let mut analyzer = BusTimingAnalyzer::new(16_000_000);
        
        // Simulate 400kHz I2C (2.5us per clock, 22.5us per byte)
        analyzer.record("I2C1", TimingEventType::Start, None);
        analyzer.current_time_ns = 0;
        analyzer.record("I2C1", TimingEventType::ByteStart, Some(0xA0));
        analyzer.current_time_ns = 22500; // 22.5us per byte
        analyzer.record("I2C1", TimingEventType::ByteEnd, None);
        analyzer.record("I2C1", TimingEventType::Ack, None);
        
        let analysis = analyzer.analyze_i2c("I2C1");
        assert_eq!(analysis.transaction_count, 1);
        assert!(analysis.effective_clock_hz > 300_000);
    }
}
