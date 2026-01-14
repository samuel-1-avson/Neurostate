//! Modbus Protocol Simulation
//!
//! Implements Modbus RTU master/slave functionality.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Modbus function codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModbusFunction {
    /// Read Coils (0x01)
    ReadCoils = 0x01,
    /// Read Discrete Inputs (0x02)
    ReadDiscreteInputs = 0x02,
    /// Read Holding Registers (0x03)
    ReadHoldingRegisters = 0x03,
    /// Read Input Registers (0x04)
    ReadInputRegisters = 0x04,
    /// Write Single Coil (0x05)
    WriteSingleCoil = 0x05,
    /// Write Single Register (0x06)
    WriteSingleRegister = 0x06,
    /// Write Multiple Coils (0x0F)
    WriteMultipleCoils = 0x0F,
    /// Write Multiple Registers (0x10)
    WriteMultipleRegisters = 0x10,
}

impl TryFrom<u8> for ModbusFunction {
    type Error = ();
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::ReadCoils),
            0x02 => Ok(Self::ReadDiscreteInputs),
            0x03 => Ok(Self::ReadHoldingRegisters),
            0x04 => Ok(Self::ReadInputRegisters),
            0x05 => Ok(Self::WriteSingleCoil),
            0x06 => Ok(Self::WriteSingleRegister),
            0x0F => Ok(Self::WriteMultipleCoils),
            0x10 => Ok(Self::WriteMultipleRegisters),
            _ => Err(()),
        }
    }
}

/// Modbus exception codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModbusException {
    IllegalFunction = 0x01,
    IllegalDataAddress = 0x02,
    IllegalDataValue = 0x03,
    SlaveDeviceFailure = 0x04,
    Acknowledge = 0x05,
    SlaveDeviceBusy = 0x06,
}

/// Modbus RTU frame
#[derive(Debug, Clone)]
pub struct ModbusFrame {
    pub slave_address: u8,
    pub function: u8,
    pub data: Vec<u8>,
    pub crc: u16,
}

impl ModbusFrame {
    pub fn new(slave_address: u8, function: ModbusFunction, data: Vec<u8>) -> Self {
        let mut frame = Self {
            slave_address,
            function: function as u8,
            data,
            crc: 0,
        };
        frame.crc = frame.calculate_crc();
        frame
    }

    /// Calculate CRC-16 for Modbus RTU
    pub fn calculate_crc(&self) -> u16 {
        let mut crc: u16 = 0xFFFF;
        
        // Include address and function
        crc = crc16_update(crc, self.slave_address);
        crc = crc16_update(crc, self.function);
        
        // Include data
        for byte in &self.data {
            crc = crc16_update(crc, *byte);
        }
        
        crc
    }

    /// Verify CRC
    pub fn verify_crc(&self) -> bool {
        self.crc == self.calculate_crc()
    }

    /// Serialize to bytes (for UART transmission)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 + self.data.len());
        bytes.push(self.slave_address);
        bytes.push(self.function);
        bytes.extend_from_slice(&self.data);
        bytes.push((self.crc & 0xFF) as u8);
        bytes.push((self.crc >> 8) as u8);
        bytes
    }

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 4 {
            return None;
        }
        
        let crc_idx = bytes.len() - 2;
        let crc = (bytes[crc_idx] as u16) | ((bytes[crc_idx + 1] as u16) << 8);
        
        Some(Self {
            slave_address: bytes[0],
            function: bytes[1],
            data: bytes[2..crc_idx].to_vec(),
            crc,
        })
    }
}

/// CRC-16 update function
fn crc16_update(crc: u16, byte: u8) -> u16 {
    let mut crc = crc ^ (byte as u16);
    for _ in 0..8 {
        if crc & 1 != 0 {
            crc = (crc >> 1) ^ 0xA001;
        } else {
            crc >>= 1;
        }
    }
    crc
}

/// Modbus slave simulator
pub struct ModbusSlave {
    /// Slave address (1-247)
    address: u8,
    /// Coils (discrete outputs) - 1 bit each
    coils: Vec<bool>,
    /// Discrete inputs - 1 bit each
    discrete_inputs: Vec<bool>,
    /// Holding registers - 16 bit each
    holding_registers: Vec<u16>,
    /// Input registers - 16 bit each
    input_registers: Vec<u16>,
    /// Response queue
    response_queue: VecDeque<ModbusFrame>,
    /// Last exception
    last_exception: Option<ModbusException>,
}

impl ModbusSlave {
    pub fn new(address: u8, num_coils: usize, num_registers: usize) -> Self {
        Self {
            address,
            coils: vec![false; num_coils],
            discrete_inputs: vec![false; num_coils],
            holding_registers: vec![0; num_registers],
            input_registers: vec![0; num_registers],
            response_queue: VecDeque::new(),
            last_exception: None,
        }
    }

    /// Process a request and generate response
    pub fn process_request(&mut self, request: &ModbusFrame) -> Option<ModbusFrame> {
        // Check address
        if request.slave_address != self.address && request.slave_address != 0 {
            return None; // Not for us
        }

        // Verify CRC
        if !request.verify_crc() {
            return None; // CRC error
        }

        let func = match ModbusFunction::try_from(request.function) {
            Ok(f) => f,
            Err(_) => {
                return Some(self.exception_response(request.function, ModbusException::IllegalFunction));
            }
        };

        match func {
            ModbusFunction::ReadCoils => self.read_coils(request),
            ModbusFunction::ReadDiscreteInputs => self.read_discrete_inputs(request),
            ModbusFunction::ReadHoldingRegisters => self.read_holding_registers(request),
            ModbusFunction::ReadInputRegisters => self.read_input_registers(request),
            ModbusFunction::WriteSingleCoil => self.write_single_coil(request),
            ModbusFunction::WriteSingleRegister => self.write_single_register(request),
            ModbusFunction::WriteMultipleCoils => self.write_multiple_coils(request),
            ModbusFunction::WriteMultipleRegisters => self.write_multiple_registers(request),
        }
    }

    fn read_coils(&self, request: &ModbusFrame) -> Option<ModbusFrame> {
        if request.data.len() < 4 {
            return Some(self.exception_response(request.function, ModbusException::IllegalDataValue));
        }

        let start = u16::from_be_bytes([request.data[0], request.data[1]]) as usize;
        let count = u16::from_be_bytes([request.data[2], request.data[3]]) as usize;

        if start + count > self.coils.len() {
            return Some(self.exception_response(request.function, ModbusException::IllegalDataAddress));
        }

        let byte_count = (count + 7) / 8;
        let mut data = vec![byte_count as u8];
        
        for i in 0..byte_count {
            let mut byte = 0u8;
            for bit in 0..8 {
                let idx = start + i * 8 + bit;
                if idx < start + count && self.coils[idx] {
                    byte |= 1 << bit;
                }
            }
            data.push(byte);
        }

        Some(ModbusFrame::new(self.address, ModbusFunction::ReadCoils, data))
    }

    fn read_discrete_inputs(&self, request: &ModbusFrame) -> Option<ModbusFrame> {
        if request.data.len() < 4 {
            return Some(self.exception_response(request.function, ModbusException::IllegalDataValue));
        }

        let start = u16::from_be_bytes([request.data[0], request.data[1]]) as usize;
        let count = u16::from_be_bytes([request.data[2], request.data[3]]) as usize;

        if start + count > self.discrete_inputs.len() {
            return Some(self.exception_response(request.function, ModbusException::IllegalDataAddress));
        }

        let byte_count = (count + 7) / 8;
        let mut data = vec![byte_count as u8];
        
        for i in 0..byte_count {
            let mut byte = 0u8;
            for bit in 0..8 {
                let idx = start + i * 8 + bit;
                if idx < start + count && self.discrete_inputs[idx] {
                    byte |= 1 << bit;
                }
            }
            data.push(byte);
        }

        Some(ModbusFrame::new(self.address, ModbusFunction::ReadDiscreteInputs, data))
    }

    fn read_holding_registers(&self, request: &ModbusFrame) -> Option<ModbusFrame> {
        if request.data.len() < 4 {
            return Some(self.exception_response(request.function, ModbusException::IllegalDataValue));
        }

        let start = u16::from_be_bytes([request.data[0], request.data[1]]) as usize;
        let count = u16::from_be_bytes([request.data[2], request.data[3]]) as usize;

        if start + count > self.holding_registers.len() {
            return Some(self.exception_response(request.function, ModbusException::IllegalDataAddress));
        }

        let mut data = vec![(count * 2) as u8];
        for i in 0..count {
            let reg = self.holding_registers[start + i];
            data.push((reg >> 8) as u8);
            data.push((reg & 0xFF) as u8);
        }

        Some(ModbusFrame::new(self.address, ModbusFunction::ReadHoldingRegisters, data))
    }

    fn read_input_registers(&self, request: &ModbusFrame) -> Option<ModbusFrame> {
        if request.data.len() < 4 {
            return Some(self.exception_response(request.function, ModbusException::IllegalDataValue));
        }

        let start = u16::from_be_bytes([request.data[0], request.data[1]]) as usize;
        let count = u16::from_be_bytes([request.data[2], request.data[3]]) as usize;

        if start + count > self.input_registers.len() {
            return Some(self.exception_response(request.function, ModbusException::IllegalDataAddress));
        }

        let mut data = vec![(count * 2) as u8];
        for i in 0..count {
            let reg = self.input_registers[start + i];
            data.push((reg >> 8) as u8);
            data.push((reg & 0xFF) as u8);
        }

        Some(ModbusFrame::new(self.address, ModbusFunction::ReadInputRegisters, data))
    }

    fn write_single_coil(&mut self, request: &ModbusFrame) -> Option<ModbusFrame> {
        if request.data.len() < 4 {
            return Some(self.exception_response(request.function, ModbusException::IllegalDataValue));
        }

        let addr = u16::from_be_bytes([request.data[0], request.data[1]]) as usize;
        let value = u16::from_be_bytes([request.data[2], request.data[3]]);

        if addr >= self.coils.len() {
            return Some(self.exception_response(request.function, ModbusException::IllegalDataAddress));
        }

        self.coils[addr] = value == 0xFF00;

        // Echo request
        Some(ModbusFrame::new(self.address, ModbusFunction::WriteSingleCoil, request.data.clone()))
    }

    fn write_single_register(&mut self, request: &ModbusFrame) -> Option<ModbusFrame> {
        if request.data.len() < 4 {
            return Some(self.exception_response(request.function, ModbusException::IllegalDataValue));
        }

        let addr = u16::from_be_bytes([request.data[0], request.data[1]]) as usize;
        let value = u16::from_be_bytes([request.data[2], request.data[3]]);

        if addr >= self.holding_registers.len() {
            return Some(self.exception_response(request.function, ModbusException::IllegalDataAddress));
        }

        self.holding_registers[addr] = value;

        // Echo request
        Some(ModbusFrame::new(self.address, ModbusFunction::WriteSingleRegister, request.data.clone()))
    }

    fn write_multiple_coils(&mut self, request: &ModbusFrame) -> Option<ModbusFrame> {
        if request.data.len() < 5 {
            return Some(self.exception_response(request.function, ModbusException::IllegalDataValue));
        }

        let start = u16::from_be_bytes([request.data[0], request.data[1]]) as usize;
        let count = u16::from_be_bytes([request.data[2], request.data[3]]) as usize;
        let byte_count = request.data[4] as usize;

        if start + count > self.coils.len() || request.data.len() < 5 + byte_count {
            return Some(self.exception_response(request.function, ModbusException::IllegalDataAddress));
        }

        for i in 0..count {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            self.coils[start + i] = (request.data[5 + byte_idx] >> bit_idx) & 1 != 0;
        }

        // Response: address + count
        let data = vec![request.data[0], request.data[1], request.data[2], request.data[3]];
        Some(ModbusFrame::new(self.address, ModbusFunction::WriteMultipleCoils, data))
    }

    fn write_multiple_registers(&mut self, request: &ModbusFrame) -> Option<ModbusFrame> {
        if request.data.len() < 5 {
            return Some(self.exception_response(request.function, ModbusException::IllegalDataValue));
        }

        let start = u16::from_be_bytes([request.data[0], request.data[1]]) as usize;
        let count = u16::from_be_bytes([request.data[2], request.data[3]]) as usize;
        let byte_count = request.data[4] as usize;

        if start + count > self.holding_registers.len() || byte_count != count * 2 {
            return Some(self.exception_response(request.function, ModbusException::IllegalDataAddress));
        }

        for i in 0..count {
            self.holding_registers[start + i] = u16::from_be_bytes([
                request.data[5 + i * 2],
                request.data[5 + i * 2 + 1],
            ]);
        }

        let data = vec![request.data[0], request.data[1], request.data[2], request.data[3]];
        Some(ModbusFrame::new(self.address, ModbusFunction::WriteMultipleRegisters, data))
    }

    fn exception_response(&self, function: u8, exception: ModbusException) -> ModbusFrame {
        ModbusFrame {
            slave_address: self.address,
            function: function | 0x80,
            data: vec![exception as u8],
            crc: 0, // Will be calculated
        }
    }

    /// Set a coil value
    pub fn set_coil(&mut self, addr: usize, value: bool) {
        if addr < self.coils.len() {
            self.coils[addr] = value;
        }
    }

    /// Set a discrete input value
    pub fn set_discrete_input(&mut self, addr: usize, value: bool) {
        if addr < self.discrete_inputs.len() {
            self.discrete_inputs[addr] = value;
        }
    }

    /// Set a holding register value
    pub fn set_holding_register(&mut self, addr: usize, value: u16) {
        if addr < self.holding_registers.len() {
            self.holding_registers[addr] = value;
        }
    }

    /// Set an input register value
    pub fn set_input_register(&mut self, addr: usize, value: u16) {
        if addr < self.input_registers.len() {
            self.input_registers[addr] = value;
        }
    }

    /// Get coil value
    pub fn get_coil(&self, addr: usize) -> Option<bool> {
        self.coils.get(addr).copied()
    }

    /// Get holding register value
    pub fn get_holding_register(&self, addr: usize) -> Option<u16> {
        self.holding_registers.get(addr).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc16() {
        let frame = ModbusFrame::new(1, ModbusFunction::ReadHoldingRegisters, vec![0x00, 0x00, 0x00, 0x0A]);
        assert!(frame.verify_crc());
    }

    #[test]
    fn test_read_holding_registers() {
        let mut slave = ModbusSlave::new(1, 100, 100);
        slave.set_holding_register(0, 0x1234);
        slave.set_holding_register(1, 0x5678);
        
        let request = ModbusFrame::new(1, ModbusFunction::ReadHoldingRegisters, vec![0x00, 0x00, 0x00, 0x02]);
        let response = slave.process_request(&request).unwrap();
        
        assert_eq!(response.function, 0x03);
        assert_eq!(response.data[0], 4); // Byte count
        assert_eq!(response.data[1], 0x12);
        assert_eq!(response.data[2], 0x34);
    }

    #[test]
    fn test_write_single_register() {
        let mut slave = ModbusSlave::new(1, 100, 100);
        
        let request = ModbusFrame::new(1, ModbusFunction::WriteSingleRegister, vec![0x00, 0x05, 0xAB, 0xCD]);
        let _response = slave.process_request(&request);
        
        assert_eq!(slave.get_holding_register(5), Some(0xABCD));
    }
}
