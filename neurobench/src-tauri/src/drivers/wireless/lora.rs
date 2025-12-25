// LoRa Code Generator
// Generates LoRa configuration for SX127x/SX126x modules

use super::*;

/// Generate LoRa SX1276/SX1278 configuration
pub fn generate_sx127x_lora(config: &LoraConfig) -> String {
    let sf_value = match config.spreading_factor {
        LoraSpreadingFactor::SF7 => 7,
        LoraSpreadingFactor::SF8 => 8,
        LoraSpreadingFactor::SF9 => 9,
        LoraSpreadingFactor::SF10 => 10,
        LoraSpreadingFactor::SF11 => 11,
        LoraSpreadingFactor::SF12 => 12,
    };
    
    let bw_value = match config.bandwidth {
        LoraBandwidth::BW125 => "0x70",  // 125 kHz
        LoraBandwidth::BW250 => "0x80",  // 250 kHz
        LoraBandwidth::BW500 => "0x90",  // 500 kHz
    };

    format!(r#"/**
 * LoRa SX127x Configuration
 * Frequency: {freq} MHz
 * SF{sf}, BW{bw}, CR4/{cr}
 */

#include <stdint.h>
#include <stdbool.h>

// SX127x Register Addresses
#define REG_FIFO                 0x00
#define REG_OP_MODE              0x01
#define REG_FRF_MSB              0x06
#define REG_FRF_MID              0x07
#define REG_FRF_LSB              0x08
#define REG_PA_CONFIG            0x09
#define REG_LNA                  0x0C
#define REG_FIFO_ADDR_PTR        0x0D
#define REG_FIFO_TX_BASE_ADDR    0x0E
#define REG_FIFO_RX_BASE_ADDR    0x0F
#define REG_IRQ_FLAGS            0x12
#define REG_RX_NB_BYTES          0x13
#define REG_MODEM_CONFIG_1       0x1D
#define REG_MODEM_CONFIG_2       0x1E
#define REG_PREAMBLE_MSB         0x20
#define REG_PREAMBLE_LSB         0x21
#define REG_PAYLOAD_LENGTH       0x22
#define REG_MODEM_CONFIG_3       0x26
#define REG_SYNC_WORD            0x39
#define REG_DIO_MAPPING_1        0x40
#define REG_VERSION              0x42
#define REG_PA_DAC               0x4D

// LoRa mode constants
#define MODE_LONG_RANGE_MODE     0x80
#define MODE_SLEEP               0x00
#define MODE_STDBY               0x01
#define MODE_TX                  0x03
#define MODE_RX_CONTINUOUS       0x05
#define MODE_RX_SINGLE           0x06

// Configuration values
#define LORA_FREQUENCY_HZ        ({freq} * 1000000UL)
#define LORA_SPREADING_FACTOR    {sf}
#define LORA_BANDWIDTH           {bw}
#define LORA_CODING_RATE         {cr}
#define LORA_TX_POWER            {power}
#define LORA_SYNC_WORD           0x{sync:02X}
#define LORA_PREAMBLE_LENGTH     {preamble}

static void lora_write_reg(uint8_t reg, uint8_t value) {{
    spi_cs_low();
    spi_transfer(reg | 0x80);  // Write bit
    spi_transfer(value);
    spi_cs_high();
}}

static uint8_t lora_read_reg(uint8_t reg) {{
    spi_cs_low();
    spi_transfer(reg & 0x7F);  // Read bit
    uint8_t value = spi_transfer(0x00);
    spi_cs_high();
    return value;
}}

void lora_init(void) {{
    // Reset module
    lora_reset_pin_low();
    delay_ms(10);
    lora_reset_pin_high();
    delay_ms(10);
    
    // Check version
    uint8_t version = lora_read_reg(REG_VERSION);
    if (version != 0x12) {{
        // Error: Wrong chip version
        return;
    }}
    
    // Set sleep mode
    lora_write_reg(REG_OP_MODE, MODE_LONG_RANGE_MODE | MODE_SLEEP);
    
    // Set frequency
    uint64_t frf = ((uint64_t)LORA_FREQUENCY_HZ << 19) / 32000000;
    lora_write_reg(REG_FRF_MSB, (frf >> 16) & 0xFF);
    lora_write_reg(REG_FRF_MID, (frf >> 8) & 0xFF);
    lora_write_reg(REG_FRF_LSB, frf & 0xFF);
    
    // Set TX power
    lora_write_reg(REG_PA_CONFIG, 0x80 | (LORA_TX_POWER - 2));
    lora_write_reg(REG_PA_DAC, 0x84);  // Default PA DAC
    
    // Set LNA boost
    lora_write_reg(REG_LNA, 0x23);
    
    // Set FIFO addresses
    lora_write_reg(REG_FIFO_TX_BASE_ADDR, 0x00);
    lora_write_reg(REG_FIFO_RX_BASE_ADDR, 0x00);
    
    // Set modem config
    lora_write_reg(REG_MODEM_CONFIG_1, {bw} | ((LORA_CODING_RATE - 4) << 1));
    lora_write_reg(REG_MODEM_CONFIG_2, (LORA_SPREADING_FACTOR << 4) | 0x04);  // CRC on
    lora_write_reg(REG_MODEM_CONFIG_3, 0x04);  // LNA AGC
    
    // Set preamble length
    lora_write_reg(REG_PREAMBLE_MSB, (LORA_PREAMBLE_LENGTH >> 8) & 0xFF);
    lora_write_reg(REG_PREAMBLE_LSB, LORA_PREAMBLE_LENGTH & 0xFF);
    
    // Set sync word
    lora_write_reg(REG_SYNC_WORD, LORA_SYNC_WORD);
    
    // Set standby mode
    lora_write_reg(REG_OP_MODE, MODE_LONG_RANGE_MODE | MODE_STDBY);
}}

bool lora_send(uint8_t *data, uint8_t len) {{
    // Set standby mode
    lora_write_reg(REG_OP_MODE, MODE_LONG_RANGE_MODE | MODE_STDBY);
    
    // Set FIFO pointer
    lora_write_reg(REG_FIFO_ADDR_PTR, 0x00);
    
    // Write data to FIFO
    for (uint8_t i = 0; i < len; i++) {{
        lora_write_reg(REG_FIFO, data[i]);
    }}
    
    // Set payload length
    lora_write_reg(REG_PAYLOAD_LENGTH, len);
    
    // Start transmission
    lora_write_reg(REG_OP_MODE, MODE_LONG_RANGE_MODE | MODE_TX);
    
    // Wait for TX done
    while ((lora_read_reg(REG_IRQ_FLAGS) & 0x08) == 0) {{
        // Could add timeout here
    }}
    
    // Clear IRQ
    lora_write_reg(REG_IRQ_FLAGS, 0xFF);
    
    return true;
}}

uint8_t lora_receive(uint8_t *buffer, uint8_t max_len) {{
    // Check for RX done
    if ((lora_read_reg(REG_IRQ_FLAGS) & 0x40) == 0) {{
        return 0;
    }}
    
    // Get packet length
    uint8_t len = lora_read_reg(REG_RX_NB_BYTES);
    if (len > max_len) len = max_len;
    
    // Set FIFO pointer to RX base
    lora_write_reg(REG_FIFO_ADDR_PTR, lora_read_reg(REG_FIFO_RX_BASE_ADDR));
    
    // Read data
    for (uint8_t i = 0; i < len; i++) {{
        buffer[i] = lora_read_reg(REG_FIFO);
    }}
    
    // Clear IRQ
    lora_write_reg(REG_IRQ_FLAGS, 0xFF);
    
    return len;
}}

void lora_start_receive(void) {{
    lora_write_reg(REG_OP_MODE, MODE_LONG_RANGE_MODE | MODE_RX_CONTINUOUS);
}}

int16_t lora_get_rssi(void) {{
    return lora_read_reg(0x1B) - 137;  // RSSI register
}}

int8_t lora_get_snr(void) {{
    return (int8_t)lora_read_reg(0x19) / 4;  // SNR register
}}
"#,
        freq = config.frequency_mhz,
        sf = sf_value,
        bw = bw_value,
        cr = config.coding_rate,
        power = config.tx_power_dbm,
        sync = config.sync_word,
        preamble = config.preamble_length,
    )
}
