//! DMA Controller Simulation

use super::Peripheral;

/// DMA Stream configuration
#[derive(Debug, Clone, Default)]
struct DmaStream {
    /// Configuration register
    cr: u32,
    /// Number of data items
    ndtr: u32,
    /// Peripheral address
    par: u32,
    /// Memory 0 address
    m0ar: u32,
    /// Memory 1 address (double buffer)
    m1ar: u32,
    /// FIFO control register
    fcr: u32,
    /// Transfer in progress
    active: bool,
    /// Items remaining
    remaining: u32,
}

/// DMA Controller
pub struct DmaController {
    instance: u8,
    base: u32,
    
    /// Low interrupt status register
    lisr: u32,
    /// High interrupt status register
    hisr: u32,
    /// Low interrupt flag clear register
    lifcr: u32,
    /// High interrupt flag clear register
    hifcr: u32,
    
    /// Streams (8 per DMA controller)
    streams: [DmaStream; 8],
    
    irq_pending: bool,
}

impl DmaController {
    pub fn new(instance: u8, base: u32) -> Self {
        Self {
            instance,
            base,
            lisr: 0,
            hisr: 0,
            lifcr: 0,
            hifcr: 0,
            streams: Default::default(),
            irq_pending: false,
        }
    }

    fn get_stream_offset(offset: u32) -> Option<(usize, u32)> {
        if offset >= 0x10 && offset < 0x10 + 8 * 0x18 {
            let stream_offset = offset - 0x10;
            let stream = (stream_offset / 0x18) as usize;
            let reg_offset = stream_offset % 0x18;
            Some((stream, reg_offset))
        } else {
            None
        }
    }
}

impl Peripheral for DmaController {
    fn name(&self) -> &str {
        match self.instance {
            1 => "DMA1",
            2 => "DMA2",
            _ => "DMA?",
        }
    }

    fn read(&self, offset: u32) -> u32 {
        match offset {
            0x00 => self.lisr,
            0x04 => self.hisr,
            0x08 => self.lifcr,
            0x0C => self.hifcr,
            _ => {
                if let Some((stream, reg)) = Self::get_stream_offset(offset) {
                    let s = &self.streams[stream];
                    match reg {
                        0x00 => s.cr,
                        0x04 => s.ndtr,
                        0x08 => s.par,
                        0x0C => s.m0ar,
                        0x10 => s.m1ar,
                        0x14 => s.fcr,
                        _ => 0,
                    }
                } else {
                    0
                }
            }
        }
    }

    fn write(&mut self, offset: u32, value: u32) {
        match offset {
            0x08 => {
                // LIFCR - writing 1 clears flags
                self.lisr &= !value;
            }
            0x0C => {
                // HIFCR - writing 1 clears flags
                self.hisr &= !value;
            }
            _ => {
                if let Some((stream, reg)) = Self::get_stream_offset(offset) {
                    let s = &mut self.streams[stream];
                    match reg {
                        0x00 => {
                            s.cr = value;
                            // EN bit enables stream
                            if value & 1 != 0 && !s.active {
                                s.active = true;
                                s.remaining = s.ndtr;
                            } else if value & 1 == 0 {
                                s.active = false;
                            }
                        }
                        0x04 => s.ndtr = value,
                        0x08 => s.par = value,
                        0x0C => s.m0ar = value,
                        0x10 => s.m1ar = value,
                        0x14 => s.fcr = value,
                        _ => {}
                    }
                }
            }
        }
    }

    fn tick(&mut self, _cycles: u64) {
        for (i, stream) in self.streams.iter_mut().enumerate() {
            if stream.active && stream.remaining > 0 {
                // Simulate one transfer per tick (simplified)
                stream.remaining -= 1;
                
                if stream.remaining == 0 {
                    stream.active = false;
                    
                    // Set transfer complete flag
                    let tc_bit = match i {
                        0 => 5,
                        1 => 11,
                        2 => 21,
                        3 => 27,
                        4 => 5,
                        5 => 11,
                        6 => 21,
                        7 => 27,
                        _ => 5,
                    };
                    
                    if i < 4 {
                        self.lisr |= 1 << tc_bit;
                    } else {
                        self.hisr |= 1 << tc_bit;
                    }
                    
                    // Check for TCIE
                    if stream.cr & (1 << 4) != 0 {
                        self.irq_pending = true;
                    }
                }
            }
        }
    }

    fn get_interrupt(&self) -> Option<u8> {
        if self.irq_pending {
            // DMA1 Stream0 = IRQ 11
            Some(11)
        } else {
            None
        }
    }

    fn clear_interrupt(&mut self) {
        self.irq_pending = false;
    }

    fn reset(&mut self) {
        self.lisr = 0;
        self.hisr = 0;
        self.streams = Default::default();
        self.irq_pending = false;
    }

    fn get_state(&self) -> serde_json::Value {
        serde_json::json!({
            "instance": self.instance,
            "lisr": self.lisr,
            "hisr": self.hisr,
            "active_streams": self.streams.iter().filter(|s| s.active).count(),
        })
    }
}
