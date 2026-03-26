//! CPU Core Emulation
//!
//! Supports multiple CPU architectures: ARM Cortex-M, RISC-V, and Xtensa.
//! Provides instruction execution, register management, and exception handling.

use super::memory::MemoryController;
use super::peripherals::PeripheralBus;
use super::engine::CpuSnapshot;

use serde::{Deserialize, Serialize};

/// Supported CPU architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CpuArchitecture {
    /// ARM Cortex-M0 (ARMv6-M)
    CortexM0,
    /// ARM Cortex-M3 (ARMv7-M)
    CortexM3,
    /// ARM Cortex-M4 (ARMv7E-M with FPU)
    CortexM4,
    /// ARM Cortex-M7 (ARMv7E-M with FPU + cache)
    CortexM7,
    /// RISC-V 32-bit (RV32IMAC)
    RiscV32,
    /// Xtensa LX6 (ESP32)
    Xtensa,
}

impl Default for CpuArchitecture {
    fn default() -> Self {
        Self::CortexM4
    }
}

/// General purpose register indices for ARM
#[allow(dead_code)]
mod arm_regs {
    pub const R0: usize = 0;
    pub const R1: usize = 1;
    pub const R2: usize = 2;
    pub const R3: usize = 3;
    pub const R4: usize = 4;
    pub const R5: usize = 5;
    pub const R6: usize = 6;
    pub const R7: usize = 7;
    pub const R8: usize = 8;
    pub const R9: usize = 9;
    pub const R10: usize = 10;
    pub const R11: usize = 11;
    pub const R12: usize = 12;
    pub const SP: usize = 13;
    pub const LR: usize = 14;
    pub const PC: usize = 15;
}

/// CPU status flags (xPSR for ARM)
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct StatusFlags {
    /// Negative flag
    pub n: bool,
    /// Zero flag
    pub z: bool,
    /// Carry flag
    pub c: bool,
    /// Overflow flag
    pub v: bool,
    /// Thumb mode (always true for Cortex-M)
    pub t: bool,
    /// Current exception number
    pub exception: u8,
}

impl StatusFlags {
    /// Convert to xPSR register value
    pub fn to_xpsr(&self) -> u32 {
        let mut val = 0u32;
        if self.n { val |= 1 << 31; }
        if self.z { val |= 1 << 30; }
        if self.c { val |= 1 << 29; }
        if self.v { val |= 1 << 28; }
        if self.t { val |= 1 << 24; }
        val |= (self.exception as u32) & 0x1FF;
        val
    }

    /// Parse from xPSR register value
    pub fn from_xpsr(val: u32) -> Self {
        Self {
            n: (val >> 31) & 1 != 0,
            z: (val >> 30) & 1 != 0,
            c: (val >> 29) & 1 != 0,
            v: (val >> 28) & 1 != 0,
            t: (val >> 24) & 1 != 0,
            exception: (val & 0x1FF) as u8,
        }
    }
}

/// Register file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterFile {
    /// General purpose registers R0-R12
    pub gpr: [u32; 13],
    /// Stack pointer (banked: MSP and PSP)
    pub sp_main: u32,
    pub sp_process: u32,
    /// Link register
    pub lr: u32,
    /// Program counter
    pub pc: u32,
    /// Status flags
    pub flags: StatusFlags,
    /// PRIMASK
    pub primask: bool,
    /// BASEPRI
    pub basepri: u8,
    /// FAULTMASK
    pub faultmask: bool,
    /// CONTROL register
    pub control: u8,
    /// FPU registers (S0-S31) for M4/M7
    pub fpu: Option<[f32; 32]>,
    /// FPU status register
    pub fpscr: u32,
}

impl Default for RegisterFile {
    fn default() -> Self {
        Self {
            gpr: [0; 13],
            sp_main: 0x2000_0000, // Default stack position
            sp_process: 0,
            lr: 0xFFFF_FFFF,
            pc: 0,
            flags: StatusFlags { t: true, ..Default::default() },
            primask: false,
            basepri: 0,
            faultmask: false,
            control: 0,
            fpu: None,
            fpscr: 0,
        }
    }
}

/// CPU Core implementation
pub struct CpuCore {
    /// CPU architecture
    arch: CpuArchitecture,
    /// Register file
    regs: RegisterFile,
    /// Total cycles executed
    cycles: u64,
    /// Is CPU halted (WFI/WFE)
    halted: bool,
    /// Exception pending
    exception_pending: Option<u32>,
}

impl CpuCore {
    /// Create a new CPU core
    pub fn new(arch: CpuArchitecture) -> Self {
        let mut regs = RegisterFile::default();
        
        // Enable FPU for M4/M7
        if matches!(arch, CpuArchitecture::CortexM4 | CpuArchitecture::CortexM7) {
            regs.fpu = Some([0.0; 32]);
        }

        Self {
            arch,
            regs,
            cycles: 0,
            halted: false,
            exception_pending: None,
        }
    }

    /// Reset CPU to initial state
    pub fn reset(&mut self) {
        self.regs = RegisterFile::default();
        if matches!(self.arch, CpuArchitecture::CortexM4 | CpuArchitecture::CortexM7) {
            self.regs.fpu = Some([0.0; 32]);
        }
        self.cycles = 0;
        self.halted = false;
        self.exception_pending = None;
    }

    /// Get current program counter
    pub fn pc(&self) -> u32 {
        self.regs.pc
    }

    /// Set program counter
    pub fn set_pc(&mut self, addr: u32) {
        // Ensure Thumb bit is set for Cortex-M
        self.regs.pc = addr & !1;
        self.regs.flags.t = addr & 1 != 0;
    }

    /// Get stack pointer
    pub fn sp(&self) -> u32 {
        if self.regs.control & 0x02 != 0 {
            self.regs.sp_process
        } else {
            self.regs.sp_main
        }
    }

    /// Set stack pointer
    pub fn set_sp(&mut self, val: u32) {
        if self.regs.control & 0x02 != 0 {
            self.regs.sp_process = val;
        } else {
            self.regs.sp_main = val;
        }
    }

    /// Get register value
    pub fn get_reg(&self, reg: usize) -> u32 {
        match reg {
            0..=12 => self.regs.gpr[reg],
            13 => self.sp(),
            14 => self.regs.lr,
            15 => self.regs.pc,
            _ => 0,
        }
    }

    /// Set register value
    pub fn set_reg(&mut self, reg: usize, val: u32) {
        match reg {
            0..=12 => self.regs.gpr[reg] = val,
            13 => self.set_sp(val),
            14 => self.regs.lr = val,
            15 => self.set_pc(val),
            _ => {}
        }
    }

    /// Execute a single instruction
    pub fn execute(
        &mut self,
        instruction: u32,
        memory: &mut MemoryController,
        peripherals: &mut PeripheralBus,
    ) -> Result<u64, String> {
        if self.halted {
            return Ok(1); // WFI burns 1 cycle
        }

        match self.arch {
            CpuArchitecture::CortexM0 |
            CpuArchitecture::CortexM3 |
            CpuArchitecture::CortexM4 |
            CpuArchitecture::CortexM7 => {
                self.execute_thumb(instruction, memory, peripherals)
            }
            CpuArchitecture::RiscV32 => {
                self.execute_riscv(instruction, memory, peripherals)
            }
            CpuArchitecture::Xtensa => {
                self.execute_xtensa(instruction, memory, peripherals)
            }
        }
    }

    /// Execute Thumb instruction (ARM Cortex-M)
    fn execute_thumb(
        &mut self,
        instruction: u32,
        memory: &mut MemoryController,
        _peripherals: &mut PeripheralBus,
    ) -> Result<u64, String> {
        let hw = (instruction & 0xFFFF) as u16;
        let is_32bit = (hw >> 11) >= 0x1D;
        
        let cycles = if is_32bit {
            self.execute_thumb32(instruction, memory)?
        } else {
            self.execute_thumb16(hw, memory)?
        };

        self.cycles += cycles;
        Ok(cycles)
    }

    /// Execute 16-bit Thumb instruction
    fn execute_thumb16(&mut self, instr: u16, memory: &mut MemoryController) -> Result<u64, String> {
        let op = (instr >> 10) & 0x3F;
        
        match op {
            // LSL immediate
            0b000_000..=0b000_011 => {
                let rd = (instr & 0x7) as usize;
                let rm = ((instr >> 3) & 0x7) as usize;
                let imm5 = ((instr >> 6) & 0x1F) as u32;
                let val = self.get_reg(rm);
                let result = if imm5 == 0 { val } else { val << imm5 };
                self.set_reg(rd, result);
                self.regs.flags.n = (result >> 31) != 0;
                self.regs.flags.z = result == 0;
                if imm5 > 0 {
                    self.regs.flags.c = (val >> (32 - imm5)) & 1 != 0;
                }
                self.regs.pc += 2;
                Ok(1)
            }
            // LSR immediate
            0b000_100..=0b000_111 => {
                let rd = (instr & 0x7) as usize;
                let rm = ((instr >> 3) & 0x7) as usize;
                let imm5 = ((instr >> 6) & 0x1F) as u32;
                let val = self.get_reg(rm);
                let shift = if imm5 == 0 { 32 } else { imm5 };
                let result = if shift >= 32 { 0 } else { val >> shift };
                self.set_reg(rd, result);
                self.regs.flags.n = (result >> 31) != 0;
                self.regs.flags.z = result == 0;
                self.regs.flags.c = if shift > 0 && shift <= 32 {
                    (val >> (shift - 1)) & 1 != 0
                } else {
                    false
                };
                self.regs.pc += 2;
                Ok(1)
            }
            // ADD/SUB register/immediate
            0b000_110..=0b000_111 => {
                let rd = (instr & 0x7) as usize;
                let rn = ((instr >> 3) & 0x7) as usize;
                let rm_imm = ((instr >> 6) & 0x7) as u32;
                let is_imm = (instr >> 10) & 1 != 0;
                let is_sub = (instr >> 9) & 1 != 0;
                
                let op1 = self.get_reg(rn);
                let op2 = if is_imm { rm_imm } else { self.get_reg(rm_imm as usize) };
                
                let (result, carry, overflow) = if is_sub {
                    let r = op1.wrapping_sub(op2);
                    let c = op1 >= op2;
                    let v = ((op1 ^ op2) & (op1 ^ r)) >> 31 != 0;
                    (r, c, v)
                } else {
                    let r = op1.wrapping_add(op2);
                    let c = r < op1;
                    let v = (!(op1 ^ op2) & (op1 ^ r)) >> 31 != 0;
                    (r, c, v)
                };
                
                self.set_reg(rd, result);
                self.regs.flags.n = (result >> 31) != 0;
                self.regs.flags.z = result == 0;
                self.regs.flags.c = carry;
                self.regs.flags.v = overflow;
                self.regs.pc += 2;
                Ok(1)
            }
            // MOV immediate
            0b001_000..=0b001_011 => {
                let rd = ((instr >> 8) & 0x7) as usize;
                let imm8 = (instr & 0xFF) as u32;
                self.set_reg(rd, imm8);
                self.regs.flags.n = false;
                self.regs.flags.z = imm8 == 0;
                self.regs.pc += 2;
                Ok(1)
            }
            // CMP immediate
            0b001_010..=0b001_011 => {
                let rn = ((instr >> 8) & 0x7) as usize;
                let imm8 = (instr & 0xFF) as u32;
                let op1 = self.get_reg(rn);
                let result = op1.wrapping_sub(imm8);
                self.regs.flags.n = (result >> 31) != 0;
                self.regs.flags.z = result == 0;
                self.regs.flags.c = op1 >= imm8;
                self.regs.flags.v = ((op1 ^ imm8) & (op1 ^ result)) >> 31 != 0;
                self.regs.pc += 2;
                Ok(1)
            }
            // ADD/SUB immediate (8-bit)
            0b001_100..=0b001_111 => {
                let rd = ((instr >> 8) & 0x7) as usize;
                let imm8 = (instr & 0xFF) as u32;
                let is_sub = (instr >> 11) & 1 != 0;
                
                let op1 = self.get_reg(rd);
                let (result, carry, overflow) = if is_sub {
                    let r = op1.wrapping_sub(imm8);
                    (r, op1 >= imm8, ((op1 ^ imm8) & (op1 ^ r)) >> 31 != 0)
                } else {
                    let r = op1.wrapping_add(imm8);
                    (r, r < op1, (!(op1 ^ imm8) & (op1 ^ r)) >> 31 != 0)
                };
                
                self.set_reg(rd, result);
                self.regs.flags.n = (result >> 31) != 0;
                self.regs.flags.z = result == 0;
                self.regs.flags.c = carry;
                self.regs.flags.v = overflow;
                self.regs.pc += 2;
                Ok(1)
            }
            // LDR literal (PC-relative)
            0b010_010..=0b010_011 => {
                let rt = ((instr >> 8) & 0x7) as usize;
                let imm8 = (instr & 0xFF) as u32;
                let addr = (self.regs.pc & !3).wrapping_add(4).wrapping_add(imm8 << 2);
                let val = memory.read32(addr)?;
                self.set_reg(rt, val);
                self.regs.pc += 2;
                Ok(2) // Memory access
            }
            // LDR/STR register offset
            0b010_100..=0b010_111 => {
                let rt = (instr & 0x7) as usize;
                let rn = ((instr >> 3) & 0x7) as usize;
                let rm = ((instr >> 6) & 0x7) as usize;
                let addr = self.get_reg(rn).wrapping_add(self.get_reg(rm));
                let op = (instr >> 9) & 0x7;
                
                match op {
                    0b000 => { // STR
                        memory.write32(addr, self.get_reg(rt))?;
                    }
                    0b001 => { // STRH
                        memory.write16(addr, self.get_reg(rt) as u16)?;
                    }
                    0b010 => { // STRB
                        memory.write8(addr, self.get_reg(rt) as u8)?;
                    }
                    0b011 => { // LDRSB
                        let val = memory.read8(addr)? as i8 as i32 as u32;
                        self.set_reg(rt, val);
                    }
                    0b100 => { // LDR
                        let val = memory.read32(addr)?;
                        self.set_reg(rt, val);
                    }
                    0b101 => { // LDRH
                        let val = memory.read16(addr)? as u32;
                        self.set_reg(rt, val);
                    }
                    0b110 => { // LDRB
                        let val = memory.read8(addr)? as u32;
                        self.set_reg(rt, val);
                    }
                    0b111 => { // LDRSH
                        let val = memory.read16(addr)? as i16 as i32 as u32;
                        self.set_reg(rt, val);
                    }
                    _ => {}
                }
                self.regs.pc += 2;
                Ok(2)
            }
            // LDR/STR immediate (word)
            0b011_000..=0b011_011 => {
                let rt = (instr & 0x7) as usize;
                let rn = ((instr >> 3) & 0x7) as usize;
                let imm5 = ((instr >> 6) & 0x1F) as u32;
                let is_load = (instr >> 11) & 1 != 0;
                let addr = self.get_reg(rn).wrapping_add(imm5 << 2);
                
                if is_load {
                    let val = memory.read32(addr)?;
                    self.set_reg(rt, val);
                } else {
                    memory.write32(addr, self.get_reg(rt))?;
                }
                self.regs.pc += 2;
                Ok(2)
            }
            // LDR/STR immediate (byte)
            0b011_100..=0b011_111 => {
                let rt = (instr & 0x7) as usize;
                let rn = ((instr >> 3) & 0x7) as usize;
                let imm5 = ((instr >> 6) & 0x1F) as u32;
                let is_load = (instr >> 11) & 1 != 0;
                let addr = self.get_reg(rn).wrapping_add(imm5);
                
                if is_load {
                    let val = memory.read8(addr)? as u32;
                    self.set_reg(rt, val);
                } else {
                    memory.write8(addr, self.get_reg(rt) as u8)?;
                }
                self.regs.pc += 2;
                Ok(2)
            }
            // LDR/STR halfword
            0b100_000..=0b100_011 => {
                let rt = (instr & 0x7) as usize;
                let rn = ((instr >> 3) & 0x7) as usize;
                let imm5 = ((instr >> 6) & 0x1F) as u32;
                let is_load = (instr >> 11) & 1 != 0;
                let addr = self.get_reg(rn).wrapping_add(imm5 << 1);
                
                if is_load {
                    let val = memory.read16(addr)? as u32;
                    self.set_reg(rt, val);
                } else {
                    memory.write16(addr, self.get_reg(rt) as u16)?;
                }
                self.regs.pc += 2;
                Ok(2)
            }
            // LDR/STR SP-relative
            0b100_100..=0b100_111 => {
                let rt = ((instr >> 8) & 0x7) as usize;
                let imm8 = (instr & 0xFF) as u32;
                let is_load = (instr >> 11) & 1 != 0;
                let addr = self.sp().wrapping_add(imm8 << 2);
                
                if is_load {
                    let val = memory.read32(addr)?;
                    self.set_reg(rt, val);
                } else {
                    memory.write32(addr, self.get_reg(rt))?;
                }
                self.regs.pc += 2;
                Ok(2)
            }
            // ADD PC/SP
            0b101_000..=0b101_011 => {
                let rd = ((instr >> 8) & 0x7) as usize;
                let imm8 = (instr & 0xFF) as u32;
                let is_sp = (instr >> 11) & 1 != 0;
                
                let base = if is_sp { self.sp() } else { (self.regs.pc + 4) & !3 };
                self.set_reg(rd, base.wrapping_add(imm8 << 2));
                self.regs.pc += 2;
                Ok(1)
            }
            // Miscellaneous
            0b101_100..=0b101_111 => {
                self.execute_misc16(instr, memory)
            }
            // PUSH/POP
            0b101_101 | 0b101_110 => {
                self.execute_push_pop(instr, memory)
            }
            // Conditional branch
            0b110_100..=0b110_111 => {
                let cond = ((instr >> 8) & 0xF) as u8;
                let imm8 = (instr & 0xFF) as i8 as i32;
                
                if self.check_condition(cond) {
                    let offset = (imm8 << 1) as i32;
                    self.regs.pc = (self.regs.pc as i32).wrapping_add(4 + offset) as u32;
                    Ok(3) // Branch taken
                } else {
                    self.regs.pc += 2;
                    Ok(1)
                }
            }
            // Unconditional branch
            0b111_000..=0b111_001 => {
                let imm11 = (instr & 0x7FF) as i32;
                let offset = ((imm11 << 21) >> 20) as i32; // Sign extend
                self.regs.pc = (self.regs.pc as i32).wrapping_add(4 + offset) as u32;
                Ok(3)
            }
            // BL/BLX (first half)
            0b111_010..=0b111_011 => {
                // Store upper part, wait for second halfword
                self.regs.pc += 2;
                Ok(1)
            }
            // BL (second half)
            0b111_100..=0b111_111 => {
                self.regs.pc += 2;
                Ok(1)
            }
            _ => {
                // Unknown instruction, skip
                self.regs.pc += 2;
                Ok(1)
            }
        }
    }

    /// Execute miscellaneous 16-bit instructions
    fn execute_misc16(&mut self, instr: u16, memory: &mut MemoryController) -> Result<u64, String> {
        let op = (instr >> 5) & 0x7F;
        
        match op {
            // ADD/SUB SP
            0b000_0000..=0b000_0001 => {
                let imm7 = (instr & 0x7F) as u32;
                let is_sub = (instr >> 7) & 1 != 0;
                let sp = self.sp();
                if is_sub {
                    self.set_sp(sp.wrapping_sub(imm7 << 2));
                } else {
                    self.set_sp(sp.wrapping_add(imm7 << 2));
                }
                self.regs.pc += 2;
                Ok(1)
            }
            // SXTH/SXTB/UXTH/UXTB
            0b001_0000..=0b001_0011 => {
                let rd = (instr & 0x7) as usize;
                let rm = ((instr >> 3) & 0x7) as usize;
                let op2 = (instr >> 6) & 0x3;
                let val = self.get_reg(rm);
                
                let result = match op2 {
                    0 => (val as i16 as i32) as u32, // SXTH
                    1 => (val as i8 as i32) as u32,  // SXTB
                    2 => val & 0xFFFF,                // UXTH
                    3 => val & 0xFF,                  // UXTB
                    _ => val,
                };
                
                self.set_reg(rd, result);
                self.regs.pc += 2;
                Ok(1)
            }
            // REV/REV16/REVSH
            0b101_0000..=0b101_0011 => {
                let rd = (instr & 0x7) as usize;
                let rm = ((instr >> 3) & 0x7) as usize;
                let op2 = (instr >> 6) & 0x3;
                let val = self.get_reg(rm);
                
                let result = match op2 {
                    0 => val.swap_bytes(), // REV
                    1 => ((val >> 8) & 0xFF) | ((val & 0xFF) << 8) | // REV16
                         ((val >> 24) & 0xFF) << 16 | ((val >> 16) & 0xFF) << 24,
                    3 => {
                        let hw = ((val >> 8) & 0xFF) | ((val & 0xFF) << 8);
                        (hw as i16 as i32) as u32 // REVSH
                    }
                    _ => val,
                };
                
                self.set_reg(rd, result);
                self.regs.pc += 2;
                Ok(1)
            }
            // NOP/YIELD/WFE/WFI/SEV
            0b110_0000..=0b110_0111 => {
                let hint = instr & 0xFF;
                match hint {
                    0x00 => {} // NOP
                    0x10 => {} // YIELD
                    0x20 => self.halted = true, // WFE
                    0x30 => self.halted = true, // WFI
                    0x40 => {} // SEV
                    _ => {}
                }
                self.regs.pc += 2;
                Ok(1)
            }
            // CPSID/CPSIE
            0b110_0011 => {
                let enable = (instr >> 4) & 1 == 0;
                if (instr >> 1) & 1 != 0 {
                    self.regs.primask = !enable;
                }
                if instr & 1 != 0 {
                    self.regs.faultmask = !enable;
                }
                self.regs.pc += 2;
                Ok(1)
            }
            // BKPT
            0b111_0000..=0b111_0111 => {
                // Breakpoint - just advance PC for now
                self.regs.pc += 2;
                Ok(1)
            }
            _ => {
                self.regs.pc += 2;
                Ok(1)
            }
        }
    }

    /// Execute PUSH/POP instructions
    fn execute_push_pop(&mut self, instr: u16, memory: &mut MemoryController) -> Result<u64, String> {
        let is_pop = (instr >> 11) & 1 != 0;
        let include_lr_pc = (instr >> 8) & 1 != 0;
        let register_list = instr & 0xFF;
        
        let count = register_list.count_ones() + if include_lr_pc { 1 } else { 0 };
        
        if is_pop {
            // POP
            let mut addr = self.sp();
            
            for i in 0..8 {
                if (register_list >> i) & 1 != 0 {
                    let val = memory.read32(addr)?;
                    self.set_reg(i, val);
                    addr = addr.wrapping_add(4);
                }
            }
            
            if include_lr_pc {
                let val = memory.read32(addr)?;
                self.set_pc(val);
                addr = addr.wrapping_add(4);
            } else {
                self.regs.pc += 2;
            }
            
            self.set_sp(addr);
        } else {
            // PUSH
            let mut addr = self.sp().wrapping_sub(count * 4);
            self.set_sp(addr);
            
            for i in 0..8 {
                if (register_list >> i) & 1 != 0 {
                    memory.write32(addr, self.get_reg(i))?;
                    addr = addr.wrapping_add(4);
                }
            }
            
            if include_lr_pc {
                memory.write32(addr, self.regs.lr)?;
            }
            
            self.regs.pc += 2;
        }
        
        Ok(1 + count as u64)
    }

    /// Execute 32-bit Thumb instruction
    fn execute_thumb32(&mut self, instr: u32, memory: &mut MemoryController) -> Result<u64, String> {
        let hw1 = ((instr >> 16) & 0xFFFF) as u16;
        let hw2 = (instr & 0xFFFF) as u16;
        let op1 = (hw1 >> 11) & 0x3;
        let op2 = (hw1 >> 4) & 0x7F;
        
        match (op1, op2 >> 5) {
            // Load/store multiple
            (1, 0b00) => {
                self.regs.pc += 4;
                Ok(2)
            }
            // Load/store dual/exclusive
            (1, 0b01) => {
                self.regs.pc += 4;
                Ok(2)
            }
            // Data processing (shifted register)
            (1, 0b10) | (1, 0b11) => {
                self.regs.pc += 4;
                Ok(1)
            }
            // Branch and misc control
            (2, _) if (op2 >> 2) == 0b11100 => {
                // BL
                let s = (hw1 >> 10) & 1;
                let imm10 = hw1 & 0x3FF;
                let j1 = (hw2 >> 13) & 1;
                let j2 = (hw2 >> 11) & 1;
                let imm11 = hw2 & 0x7FF;
                
                let i1 = !(j1 ^ s) & 1;
                let i2 = !(j2 ^ s) & 1;
                
                let offset = ((s as u32) << 24) | ((i1 as u32) << 23) | ((i2 as u32) << 22) |
                            ((imm10 as u32) << 12) | ((imm11 as u32) << 1);
                let offset = ((offset as i32) << 7) >> 7; // Sign extend
                
                self.regs.lr = (self.regs.pc + 4) | 1;
                self.regs.pc = (self.regs.pc as i32).wrapping_add(4 + offset) as u32;
                Ok(4)
            }
            // Data processing (plain binary immediate)
            (2, _) => {
                self.regs.pc += 4;
                Ok(1)
            }
            // Single data item load/store
            (3, _) => {
                let op = (hw1 >> 5) & 0xF;
                let rn = (hw1 & 0xF) as usize;
                let rt = ((hw2 >> 12) & 0xF) as usize;
                
                match op {
                    0b0101 => {
                        // LDR.W
                        let imm12 = (hw2 & 0xFFF) as u32;
                        let addr = self.get_reg(rn).wrapping_add(imm12);
                        let val = memory.read32(addr)?;
                        self.set_reg(rt, val);
                    }
                    0b0100 => {
                        // STR.W
                        let imm12 = (hw2 & 0xFFF) as u32;
                        let addr = self.get_reg(rn).wrapping_add(imm12);
                        memory.write32(addr, self.get_reg(rt))?;
                    }
                    _ => {}
                }
                self.regs.pc += 4;
                Ok(2)
            }
            _ => {
                self.regs.pc += 4;
                Ok(1)
            }
        }
    }

    /// Check condition code
    fn check_condition(&self, cond: u8) -> bool {
        match cond {
            0b0000 => self.regs.flags.z,                    // EQ
            0b0001 => !self.regs.flags.z,                   // NE
            0b0010 => self.regs.flags.c,                    // CS/HS
            0b0011 => !self.regs.flags.c,                   // CC/LO
            0b0100 => self.regs.flags.n,                    // MI
            0b0101 => !self.regs.flags.n,                   // PL
            0b0110 => self.regs.flags.v,                    // VS
            0b0111 => !self.regs.flags.v,                   // VC
            0b1000 => self.regs.flags.c && !self.regs.flags.z, // HI
            0b1001 => !self.regs.flags.c || self.regs.flags.z, // LS
            0b1010 => self.regs.flags.n == self.regs.flags.v, // GE
            0b1011 => self.regs.flags.n != self.regs.flags.v, // LT
            0b1100 => !self.regs.flags.z && (self.regs.flags.n == self.regs.flags.v), // GT
            0b1101 => self.regs.flags.z || (self.regs.flags.n != self.regs.flags.v),  // LE
            0b1110 | 0b1111 => true, // AL
            _ => true,
        }
    }

    /// Execute RISC-V instruction
    fn execute_riscv(
        &mut self,
        _instruction: u32,
        _memory: &mut MemoryController,
        _peripherals: &mut PeripheralBus,
    ) -> Result<u64, String> {
        // TODO: Implement RISC-V instruction set
        self.regs.pc += 4;
        Ok(1)
    }

    /// Execute Xtensa instruction
    fn execute_xtensa(
        &mut self,
        _instruction: u32,
        _memory: &mut MemoryController,
        _peripherals: &mut PeripheralBus,
    ) -> Result<u64, String> {
        // TODO: Implement Xtensa instruction set
        self.regs.pc += 3; // Xtensa uses 24-bit instructions
        Ok(1)
    }

    /// Enter exception handler
    pub fn enter_exception(&mut self, vector: u32) {
        // Push context onto stack
        let sp = self.sp() - 32;
        self.set_sp(sp);
        
        // Save EXC_RETURN in LR
        self.regs.lr = 0xFFFF_FFF9; // Return to Thread mode, MSP
        
        // Update exception number
        self.regs.flags.exception = vector as u8;
        
        // Wake from halt
        self.halted = false;
    }

    /// Create snapshot
    pub fn snapshot(&self) -> CpuSnapshot {
        CpuSnapshot {
            registers: [
                self.regs.gpr[0], self.regs.gpr[1], self.regs.gpr[2], self.regs.gpr[3],
                self.regs.gpr[4], self.regs.gpr[5], self.regs.gpr[6], self.regs.gpr[7],
                self.regs.gpr[8], self.regs.gpr[9], self.regs.gpr[10], self.regs.gpr[11],
                self.regs.gpr[12], self.sp(), self.regs.lr, self.regs.pc,
            ],
            pc: self.regs.pc,
            sp: self.sp(),
            lr: self.regs.lr,
            xpsr: self.regs.flags.to_xpsr(),
            primask: self.regs.primask,
            basepri: self.regs.basepri,
            faultmask: self.regs.faultmask,
            control: self.regs.control,
        }
    }

    /// Restore from snapshot
    pub fn restore(&mut self, snapshot: &CpuSnapshot) {
        for i in 0..13 {
            self.regs.gpr[i] = snapshot.registers[i];
        }
        self.regs.sp_main = snapshot.sp;
        self.regs.lr = snapshot.lr;
        self.regs.pc = snapshot.pc;
        self.regs.flags = StatusFlags::from_xpsr(snapshot.xpsr);
        self.regs.primask = snapshot.primask;
        self.regs.basepri = snapshot.basepri;
        self.regs.faultmask = snapshot.faultmask;
        self.regs.control = snapshot.control;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_creation() {
        let cpu = CpuCore::new(CpuArchitecture::CortexM4);
        assert_eq!(cpu.pc(), 0);
        assert!(cpu.regs.fpu.is_some());
    }

    #[test]
    fn test_register_access() {
        let mut cpu = CpuCore::new(CpuArchitecture::CortexM4);
        cpu.set_reg(0, 0x1234);
        assert_eq!(cpu.get_reg(0), 0x1234);
    }

    #[test]
    fn test_condition_codes() {
        let mut cpu = CpuCore::new(CpuArchitecture::CortexM4);
        cpu.regs.flags.z = true;
        assert!(cpu.check_condition(0b0000)); // EQ
        assert!(!cpu.check_condition(0b0001)); // NE
    }

    #[test]
    fn test_status_flags_conversion() {
        let flags = StatusFlags {
            n: true, z: false, c: true, v: false, t: true, exception: 5,
        };
        let xpsr = flags.to_xpsr();
        let restored = StatusFlags::from_xpsr(xpsr);
        assert_eq!(flags.n, restored.n);
        assert_eq!(flags.z, restored.z);
        assert_eq!(flags.c, restored.c);
        assert_eq!(flags.v, restored.v);
        assert_eq!(flags.exception, restored.exception);
    }
}
