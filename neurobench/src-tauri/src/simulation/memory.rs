//! Memory Controller
//!
//! Manages Flash, RAM, and memory-mapped I/O regions.
//! Provides read/write access with proper access permissions.

use super::engine::SimulationConfig;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Memory access type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryAccess {
    Read,
    Write,
    Execute,
}

/// Memory region type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryType {
    Flash,
    Ram,
    Peripheral,
    SystemControl,
    ExternalRam,
    ExternalDevice,
}

/// Memory region definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRegion {
    pub name: String,
    pub base: u32,
    pub size: usize,
    pub mem_type: MemoryType,
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
}

impl MemoryRegion {
    pub fn contains(&self, addr: u32) -> bool {
        addr >= self.base && addr < self.base + self.size as u32
    }

    pub fn offset(&self, addr: u32) -> usize {
        (addr - self.base) as usize
    }
}

/// Flash memory with write latency simulation
#[derive(Clone)]
pub struct Flash {
    data: Vec<u8>,
    base: u32,
    /// Simulated write latency in cycles
    write_latency: u32,
    /// Flash is currently being written
    write_pending: bool,
}

impl Flash {
    pub fn new(base: u32, size: usize) -> Self {
        Self {
            data: vec![0xFF; size], // Flash defaults to 0xFF
            base,
            write_latency: 16, // Typical flash write takes 16+ cycles
            write_pending: false,
        }
    }

    pub fn read(&self, offset: usize, size: usize) -> Option<&[u8]> {
        if offset + size <= self.data.len() {
            Some(&self.data[offset..offset + size])
        } else {
            None
        }
    }

    pub fn write(&mut self, offset: usize, data: &[u8]) -> bool {
        if offset + data.len() <= self.data.len() {
            // Flash write: can only clear bits (0xFF -> any value)
            for (i, byte) in data.iter().enumerate() {
                self.data[offset + i] &= byte;
            }
            true
        } else {
            false
        }
    }

    pub fn erase_sector(&mut self, sector_addr: u32, sector_size: usize) {
        let offset = (sector_addr - self.base) as usize;
        if offset + sector_size <= self.data.len() {
            for byte in &mut self.data[offset..offset + sector_size] {
                *byte = 0xFF;
            }
        }
    }

    pub fn load(&mut self, offset: usize, data: &[u8]) {
        if offset + data.len() <= self.data.len() {
            self.data[offset..offset + data.len()].copy_from_slice(data);
        }
    }
}

/// RAM memory
#[derive(Clone)]
pub struct Ram {
    data: Vec<u8>,
    base: u32,
}

impl Ram {
    pub fn new(base: u32, size: usize) -> Self {
        Self {
            data: vec![0; size],
            base,
        }
    }

    pub fn read(&self, offset: usize, size: usize) -> Option<&[u8]> {
        if offset + size <= self.data.len() {
            Some(&self.data[offset..offset + size])
        } else {
            None
        }
    }

    pub fn write(&mut self, offset: usize, data: &[u8]) -> bool {
        if offset + data.len() <= self.data.len() {
            self.data[offset..offset + data.len()].copy_from_slice(data);
            true
        } else {
            false
        }
    }
}

/// Memory Controller manages all memory regions
pub struct MemoryController {
    flash: Flash,
    ram: Ram,
    regions: Vec<MemoryRegion>,
    /// Peripheral memory region (for MMIO)
    peripheral_mem: Vec<u8>,
    peripheral_base: u32,
    /// System control block
    scb: Vec<u8>,
    scb_base: u32,
}

impl MemoryController {
    /// Create a new memory controller from simulation config
    pub fn new(config: &SimulationConfig) -> Self {
        let flash = Flash::new(config.flash_base, config.flash_size);
        let ram = Ram::new(config.ram_base, config.ram_size);
        
        let regions = vec![
            MemoryRegion {
                name: "Flash".to_string(),
                base: config.flash_base,
                size: config.flash_size,
                mem_type: MemoryType::Flash,
                readable: true,
                writable: false, // Flash needs special handling
                executable: true,
            },
            MemoryRegion {
                name: "RAM".to_string(),
                base: config.ram_base,
                size: config.ram_size,
                mem_type: MemoryType::Ram,
                readable: true,
                writable: true,
                executable: true,
            },
            MemoryRegion {
                name: "Peripherals".to_string(),
                base: 0x4000_0000,
                size: 0x2000_0000,
                mem_type: MemoryType::Peripheral,
                readable: true,
                writable: true,
                executable: false,
            },
            MemoryRegion {
                name: "PPB".to_string(),  // Private Peripheral Bus
                base: 0xE000_0000,
                size: 0x0010_0000,
                mem_type: MemoryType::SystemControl,
                readable: true,
                writable: true,
                executable: false,
            },
        ];

        Self {
            flash,
            ram,
            regions,
            peripheral_mem: vec![0; 0x1000], // 4KB for peripheral simulation
            peripheral_base: 0x4000_0000,
            scb: vec![0; 0x1000], // System control block
            scb_base: 0xE000_E000,
        }
    }

    /// Reset memory to initial state
    pub fn reset(&mut self) {
        self.ram = Ram::new(self.ram.base, self.ram.data.len());
        self.peripheral_mem.fill(0);
        self.scb.fill(0);
        
        // Initialize SCB with defaults
        // CPUID at 0xE000ED00
        let cpuid_offset = 0xE000_ED00u32 - self.scb_base;
        if (cpuid_offset as usize) < self.scb.len() - 4 {
            let cpuid = 0x410FC241u32; // Cortex-M4 r0p1
            self.scb[cpuid_offset as usize..cpuid_offset as usize + 4]
                .copy_from_slice(&cpuid.to_le_bytes());
        }
    }

    /// Load firmware into flash
    pub fn load_firmware(&mut self, data: &[u8], base: u32) -> Result<(), String> {
        let offset = if base >= self.flash.base {
            (base - self.flash.base) as usize
        } else {
            return Err("Invalid firmware base address".to_string());
        };
        
        if offset + data.len() > self.flash.data.len() {
            return Err("Firmware too large for flash".to_string());
        }
        
        self.flash.load(offset, data);
        Ok(())
    }

    /// Read 8-bit value
    pub fn read8(&self, addr: u32) -> Result<u8, String> {
        self.read_bytes(addr, 1).map(|b| b[0])
    }

    /// Read 16-bit value
    pub fn read16(&self, addr: u32) -> Result<u16, String> {
        let bytes = self.read_bytes(addr, 2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    /// Read 32-bit value
    pub fn read32(&self, addr: u32) -> Result<u32, String> {
        let bytes = self.read_bytes(addr, 4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Write 8-bit value
    pub fn write8(&mut self, addr: u32, val: u8) -> Result<(), String> {
        self.write_bytes(addr, &[val])
    }

    /// Write 16-bit value
    pub fn write16(&mut self, addr: u32, val: u16) -> Result<(), String> {
        self.write_bytes(addr, &val.to_le_bytes())
    }

    /// Write 32-bit value
    pub fn write32(&mut self, addr: u32, val: u32) -> Result<(), String> {
        self.write_bytes(addr, &val.to_le_bytes())
    }

    /// Read bytes from memory
    fn read_bytes(&self, addr: u32, size: usize) -> Result<Vec<u8>, String> {
        // Check flash
        if addr >= self.flash.base && addr < self.flash.base + self.flash.data.len() as u32 {
            let offset = (addr - self.flash.base) as usize;
            if let Some(bytes) = self.flash.read(offset, size) {
                return Ok(bytes.to_vec());
            }
        }
        
        // Check RAM
        if addr >= self.ram.base && addr < self.ram.base + self.ram.data.len() as u32 {
            let offset = (addr - self.ram.base) as usize;
            if let Some(bytes) = self.ram.read(offset, size) {
                return Ok(bytes.to_vec());
            }
        }
        
        // Check peripheral region
        if addr >= self.peripheral_base && addr < self.peripheral_base + self.peripheral_mem.len() as u32 {
            let offset = (addr - self.peripheral_base) as usize;
            if offset + size <= self.peripheral_mem.len() {
                return Ok(self.peripheral_mem[offset..offset + size].to_vec());
            }
        }
        
        // Check SCB
        if addr >= self.scb_base && addr < self.scb_base + self.scb.len() as u32 {
            let offset = (addr - self.scb_base) as usize;
            if offset + size <= self.scb.len() {
                return Ok(self.scb[offset..offset + size].to_vec());
            }
        }
        
        // Return zeros for unmapped regions (common practice)
        Ok(vec![0; size])
    }

    /// Write bytes to memory
    fn write_bytes(&mut self, addr: u32, data: &[u8]) -> Result<(), String> {
        // Check RAM
        if addr >= self.ram.base && addr < self.ram.base + self.ram.data.len() as u32 {
            let offset = (addr - self.ram.base) as usize;
            if self.ram.write(offset, data) {
                return Ok(());
            }
        }
        
        // Check peripheral region
        if addr >= self.peripheral_base && addr < self.peripheral_base + self.peripheral_mem.len() as u32 {
            let offset = (addr - self.peripheral_base) as usize;
            if offset + data.len() <= self.peripheral_mem.len() {
                self.peripheral_mem[offset..offset + data.len()].copy_from_slice(data);
                return Ok(());
            }
        }
        
        // Check SCB
        if addr >= self.scb_base && addr < self.scb_base + self.scb.len() as u32 {
            let offset = (addr - self.scb_base) as usize;
            if offset + data.len() <= self.scb.len() {
                self.scb[offset..offset + data.len()].copy_from_slice(data);
                return Ok(());
            }
        }
        
        // Ignore writes to flash (would need unlock sequence)
        if addr >= self.flash.base && addr < self.flash.base + self.flash.data.len() as u32 {
            return Ok(()); // Silently ignore
        }
        
        // Ignore writes to unmapped regions
        Ok(())
    }

    /// Get hash of memory state for snapshot comparison
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.ram.data.hash(&mut hasher);
        hasher.finish()
    }

    /// Get a slice of RAM for inspection
    pub fn get_ram_slice(&self, offset: usize, size: usize) -> Option<&[u8]> {
        if offset + size <= self.ram.data.len() {
            Some(&self.ram.data[offset..offset + size])
        } else {
            None
        }
    }

    /// Get a slice of flash for inspection
    pub fn get_flash_slice(&self, offset: usize, size: usize) -> Option<&[u8]> {
        if offset + size <= self.flash.data.len() {
            Some(&self.flash.data[offset..offset + size])
        } else {
            None
        }
    }

    /// Get memory region info
    pub fn get_regions(&self) -> &[MemoryRegion] {
        &self.regions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::engine::SimulationConfig;

    #[test]
    fn test_memory_creation() {
        let config = SimulationConfig::default();
        let memory = MemoryController::new(&config);
        assert_eq!(memory.regions.len(), 4);
    }

    #[test]
    fn test_ram_read_write() {
        let config = SimulationConfig::default();
        let mut memory = MemoryController::new(&config);
        
        let addr = config.ram_base + 0x100;
        memory.write32(addr, 0xDEADBEEF).unwrap();
        assert_eq!(memory.read32(addr).unwrap(), 0xDEADBEEF);
    }

    #[test]
    fn test_flash_load() {
        let config = SimulationConfig::default();
        let mut memory = MemoryController::new(&config);
        
        let firmware = vec![0x00, 0x20, 0x00, 0x20, 0x01, 0x00, 0x00, 0x08];
        memory.load_firmware(&firmware, config.flash_base).unwrap();
        
        assert_eq!(memory.read32(config.flash_base).unwrap(), 0x20002000);
    }

    #[test]
    fn test_endianness() {
        let config = SimulationConfig::default();
        let mut memory = MemoryController::new(&config);
        
        let addr = config.ram_base;
        memory.write32(addr, 0x12345678).unwrap();
        
        assert_eq!(memory.read8(addr).unwrap(), 0x78);
        assert_eq!(memory.read8(addr + 1).unwrap(), 0x56);
        assert_eq!(memory.read16(addr).unwrap(), 0x5678);
    }
}
