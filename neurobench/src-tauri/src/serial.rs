// Serial Monitor Module
// Real-time serial port communication

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Serial port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialConfig {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: String,  // none, odd, even
    pub flow_control: String,  // none, hardware, software
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            port: String::new(),
            baud_rate: 115200,
            data_bits: 8,
            stop_bits: 1,
            parity: "none".to_string(),
            flow_control: "none".to_string(),
        }
    }
}

/// Available serial port info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub name: String,
    pub description: String,
    pub port_type: String,
}

/// Serial data received
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialData {
    pub timestamp: u64,
    pub data: Vec<u8>,
    pub text: Option<String>,
}

/// List available serial ports
pub fn list_ports() -> Result<Vec<PortInfo>, String> {
    // Use serialport crate if available, otherwise return mock data
    #[cfg(feature = "serial")]
    {
        use serialport::available_ports;
        
        available_ports()
            .map(|ports| {
                ports.into_iter().map(|p| PortInfo {
                    name: p.port_name.clone(),
                    description: match p.port_type {
                        serialport::SerialPortType::UsbPort(info) => {
                            format!("{} - {}", 
                                info.manufacturer.unwrap_or_default(),
                                info.product.unwrap_or_default())
                        }
                        _ => "Serial Port".to_string(),
                    },
                    port_type: format!("{:?}", p.port_type),
                }).collect()
            })
            .map_err(|e| e.to_string())
    }
    
    #[cfg(not(feature = "serial"))]
    {
        // Mock ports for development
        Ok(vec![
            PortInfo {
                name: "COM1".to_string(),
                description: "Communications Port".to_string(),
                port_type: "Standard".to_string(),
            },
            PortInfo {
                name: "COM3".to_string(),
                description: "USB Serial Device".to_string(),
                port_type: "USB".to_string(),
            },
        ])
    }
}

/// Common baud rates
pub fn get_baud_rates() -> Vec<u32> {
    vec![
        300, 1200, 2400, 4800, 9600, 14400, 19200, 28800,
        38400, 57600, 76800, 115200, 230400, 460800, 921600,
        1000000, 2000000, 3000000,
    ]
}

/// Format data for display
pub fn format_data(data: &[u8], format: &str) -> String {
    match format {
        "hex" => data.iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" "),
        "decimal" => data.iter()
            .map(|b| format!("{}", b))
            .collect::<Vec<_>>()
            .join(" "),
        "binary" => data.iter()
            .map(|b| format!("{:08b}", b))
            .collect::<Vec<_>>()
            .join(" "),
        "ascii" | _ => String::from_utf8_lossy(data).to_string(),
    }
}

/// Parse escape sequences in string
pub fn parse_escape_sequences(input: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let mut chars = input.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push(b'\n'),
                Some('r') => result.push(b'\r'),
                Some('t') => result.push(b'\t'),
                Some('0') => result.push(0),
                Some('x') => {
                    // Hex escape \xNN
                    let hex: String = chars.by_ref().take(2).collect();
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        result.push(byte);
                    }
                }
                Some(other) => result.push(other as u8),
                None => result.push(b'\\'),
            }
        } else {
            result.push(c as u8);
        }
    }
    
    result
}

/// Calculate checksum
pub fn calculate_checksum(data: &[u8], algorithm: &str) -> String {
    match algorithm {
        "xor" => {
            let checksum = data.iter().fold(0u8, |acc, &b| acc ^ b);
            format!("{:02X}", checksum)
        }
        "sum" => {
            let checksum = data.iter().fold(0u16, |acc, &b| acc.wrapping_add(b as u16));
            format!("{:04X}", checksum & 0xFF)
        }
        "crc8" => {
            let mut crc = 0u8;
            for &byte in data {
                crc ^= byte;
                for _ in 0..8 {
                    if crc & 0x80 != 0 {
                        crc = (crc << 1) ^ 0x07;
                    } else {
                        crc <<= 1;
                    }
                }
            }
            format!("{:02X}", crc)
        }
        "crc16" | _ => {
            let mut crc = 0xFFFFu16;
            for &byte in data {
                crc ^= byte as u16;
                for _ in 0..8 {
                    if crc & 1 != 0 {
                        crc = (crc >> 1) ^ 0xA001;
                    } else {
                        crc >>= 1;
                    }
                }
            }
            format!("{:04X}", crc)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_data_hex() {
        let data = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F];
        assert_eq!(format_data(&data, "hex"), "48 65 6C 6C 6F");
    }

    #[test]
    fn test_format_data_ascii() {
        let data = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F];
        assert_eq!(format_data(&data, "ascii"), "Hello");
    }

    #[test]
    fn test_parse_escape_sequences() {
        assert_eq!(parse_escape_sequences("Hello\\n"), vec![72, 101, 108, 108, 111, 10]);
    }

    #[test]
    fn test_calculate_checksum() {
        let data = vec![0x01, 0x02, 0x03];
        assert_eq!(calculate_checksum(&data, "xor"), "00");
    }
}
