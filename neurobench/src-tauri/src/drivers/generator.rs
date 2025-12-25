// Driver Generator
// Main generator logic that coordinates template-based and AI-powered driver generation

use super::templates::*;
use super::gpio::generate_gpio_driver;
use super::uart::generate_uart_driver;
use super::spi::generate_spi_driver;
use super::i2c::generate_i2c_driver;
use crate::ai::AIService;

/// Generate a driver based on request
pub fn generate_driver(request: &DriverRequest) -> Result<DriverOutput, String> {
    match request.peripheral_type {
        PeripheralType::GPIO => {
            let config = request.gpio_config.as_ref()
                .ok_or("GPIO configuration required")?;
            Ok(generate_gpio_driver(config, &request.mcu_arch, &request.language))
        },
        PeripheralType::UART => {
            let config = request.uart_config.as_ref()
                .ok_or("UART configuration required")?;
            Ok(generate_uart_driver(config, &request.mcu_arch, &request.language))
        },
        PeripheralType::SPI => {
            let config = request.spi_config.as_ref()
                .ok_or("SPI configuration required")?;
            Ok(generate_spi_driver(config, &request.mcu_arch, &request.language))
        },
        PeripheralType::I2C => {
            let config = request.i2c_config.as_ref()
                .ok_or("I2C configuration required")?;
            Ok(generate_i2c_driver(config, &request.mcu_arch, &request.language))
        },
        _ => Err(format!("{:?} driver generator not yet implemented", request.peripheral_type)),
    }
}

/// Generate driver using AI for custom requirements
pub async fn generate_driver_with_ai(
    peripheral: &str,
    description: &str,
    mcu: &str,
    language: &str,
) -> Result<DriverOutput, String> {
    let ai = AIService::new();
    if !ai.is_available() {
        return Err("AI not available. Set GEMINI_API_KEY.".to_string());
    }

    let prompt = format!(
        r#"Generate a {language} driver for {peripheral} on {mcu}.

Requirements:
{description}

Generate complete, production-ready code with:
1. Header file (if C/C++)
2. Implementation file
3. Clear comments
4. Error handling
5. Example usage

Output format:
=== HEADER ===
[header code here]

=== SOURCE ===
[source code here]

=== EXAMPLE ===
[example code here]
"#
    );

    let response = ai.chat(&prompt, None).await?;

    // Parse the response to extract code sections
    let (header, source, example) = parse_ai_driver_response(&response);

    Ok(DriverOutput {
        header_file: if header.is_empty() { None } else { Some(header) },
        source_file: source,
        example_file: if example.is_empty() { None } else { Some(example) },
        peripheral_type: PeripheralType::GPIO, // Default, will be set based on peripheral
    })
}

fn parse_ai_driver_response(response: &str) -> (String, String, String) {
    let mut header = String::new();
    let mut source = String::new();
    let mut example = String::new();
    
    let mut current_section = "";
    
    for line in response.lines() {
        if line.contains("=== HEADER ===") {
            current_section = "header";
            continue;
        } else if line.contains("=== SOURCE ===") {
            current_section = "source";
            continue;
        } else if line.contains("=== EXAMPLE ===") {
            current_section = "example";
            continue;
        }
        
        match current_section {
            "header" => { header.push_str(line); header.push('\n'); },
            "source" => { source.push_str(line); source.push('\n'); },
            "example" => { example.push_str(line); example.push('\n'); },
            _ => { source.push_str(line); source.push('\n'); }, // Default to source
        }
    }
    
    // If no sections found, treat entire response as source
    if source.is_empty() && header.is_empty() {
        source = response.to_string();
    }
    
    (header, source, example)
}

/// Get list of supported peripherals
pub fn get_supported_peripherals() -> Vec<&'static str> {
    vec!["GPIO", "UART", "SPI", "I2C", "ADC", "PWM", "Timer", "DMA", "CAN"]
}

/// Get list of supported MCU architectures
pub fn get_supported_architectures() -> Vec<&'static str> {
    vec!["STM32", "ESP32", "RP2040", "NRF52", "AVR", "PIC"]
}
