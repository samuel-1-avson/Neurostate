// Zigbee/XBee Code Generator
// Generates Zigbee configuration for XBee modules

use super::*;

/// Generate XBee API configuration
pub fn generate_xbee_config(config: &ZigbeeConfig) -> String {
    let device_type_str = match config.device_type {
        ZigbeeDeviceType::Coordinator => "COORDINATOR",
        ZigbeeDeviceType::Router => "ROUTER",
        ZigbeeDeviceType::EndDevice => "END_DEVICE",
    };

    format!(r#"/**
 * XBee Zigbee Configuration
 * Device Type: {device_type}
 * PAN ID: 0x{pan_id:04X}
 * Channel: {channel}
 */

#include <stdint.h>
#include <stdbool.h>

// XBee AT Commands for configuration
#define XBEE_PAN_ID     0x{pan_id:04X}
#define XBEE_CHANNEL    {channel}
#define XBEE_DEVICE_TYPE "{device_type}"

// Frame types
#define XBEE_API_TX_REQUEST     0x10
#define XBEE_API_RX_PACKET      0x90
#define XBEE_API_AT_COMMAND     0x08
#define XBEE_API_AT_RESPONSE    0x88

typedef struct {{
    uint8_t start_delimiter;  // 0x7E
    uint16_t length;
    uint8_t frame_type;
    uint8_t frame_id;
    uint8_t payload[256];
    uint8_t checksum;
}} xbee_frame_t;

static uint8_t xbee_checksum(uint8_t *data, uint16_t len) {{
    uint8_t sum = 0;
    for (uint16_t i = 0; i < len; i++) {{
        sum += data[i];
    }}
    return 0xFF - sum;
}}

void xbee_init(void) {{
    // Configure UART for XBee (9600 baud default, or 115200 for API mode)
    // Send AT commands to configure the module
    
    // Example: Set PAN ID
    uint8_t cmd[] = {{'I', 'D'}};
    xbee_send_at_command(cmd, 2, (uint8_t*)&XBEE_PAN_ID, 2);
    
    // Set Channel
    uint8_t ch_cmd[] = {{'C', 'H'}};
    uint8_t channel = XBEE_CHANNEL;
    xbee_send_at_command(ch_cmd, 2, &channel, 1);
}}

void xbee_send_at_command(uint8_t *command, uint8_t cmd_len, uint8_t *param, uint8_t param_len) {{
    xbee_frame_t frame;
    frame.start_delimiter = 0x7E;
    frame.frame_type = XBEE_API_AT_COMMAND;
    frame.frame_id = 0x01;
    
    uint16_t payload_len = 2 + cmd_len + param_len;
    frame.length = payload_len;
    
    // Copy command
    for (uint8_t i = 0; i < cmd_len; i++) {{
        frame.payload[i] = command[i];
    }}
    
    // Copy parameters
    for (uint8_t i = 0; i < param_len; i++) {{
        frame.payload[cmd_len + i] = param[i];
    }}
    
    // Calculate checksum
    uint8_t checksum_data[256];
    checksum_data[0] = frame.frame_type;
    checksum_data[1] = frame.frame_id;
    for (uint16_t i = 0; i < payload_len - 2; i++) {{
        checksum_data[2 + i] = frame.payload[i];
    }}
    frame.checksum = xbee_checksum(checksum_data, payload_len);
    
    // Send frame over UART
    xbee_uart_send_frame(&frame);
}}

void xbee_send_data(uint64_t dest_addr, uint16_t network_addr, uint8_t *data, uint16_t len) {{
    xbee_frame_t frame;
    frame.start_delimiter = 0x7E;
    frame.frame_type = XBEE_API_TX_REQUEST;
    frame.frame_id = 0x01;
    
    // Build TX request payload
    uint8_t *p = frame.payload;
    
    // 64-bit destination address
    for (int i = 7; i >= 0; i--) {{
        *p++ = (dest_addr >> (i * 8)) & 0xFF;
    }}
    
    // 16-bit network address
    *p++ = (network_addr >> 8) & 0xFF;
    *p++ = network_addr & 0xFF;
    
    // Broadcast radius
    *p++ = 0x00;
    
    // Options
    *p++ = 0x00;
    
    // Data
    for (uint16_t i = 0; i < len; i++) {{
        *p++ = data[i];
    }}
    
    frame.length = (p - frame.payload) + 2;  // +2 for frame_type and frame_id
    
    xbee_uart_send_frame(&frame);
}}

void xbee_uart_send_frame(xbee_frame_t *frame) {{
    // Send start delimiter
    uart_putc(frame->start_delimiter);
    
    // Send length (big endian)
    uart_putc((frame->length >> 8) & 0xFF);
    uart_putc(frame->length & 0xFF);
    
    // Send frame type and ID
    uart_putc(frame->frame_type);
    uart_putc(frame->frame_id);
    
    // Send payload
    for (uint16_t i = 0; i < frame->length - 2; i++) {{
        uart_putc(frame->payload[i]);
    }}
    
    // Send checksum
    uart_putc(frame->checksum);
}}

bool xbee_receive_frame(xbee_frame_t *frame) {{
    // Check for start delimiter
    if (uart_available() < 4) return false;
    
    uint8_t start = uart_getc();
    if (start != 0x7E) return false;
    
    uint16_t length = (uart_getc() << 8) | uart_getc();
    if (length > 256) return false;
    
    frame->start_delimiter = start;
    frame->length = length;
    frame->frame_type = uart_getc();
    
    // Read payload
    for (uint16_t i = 0; i < length - 1; i++) {{
        frame->payload[i] = uart_getc();
    }}
    
    frame->checksum = uart_getc();
    
    return true;
}}
"#,
        device_type = device_type_str,
        pan_id = config.pan_id,
        channel = config.channel,
    )
}

/// Generate Zigbee endpoint configuration
pub fn generate_zigbee_endpoints(config: &ZigbeeConfig) -> String {
    let mut code = String::new();
    
    for endpoint in &config.endpoints {
        code.push_str(&format!(r#"
// Endpoint {id}: Profile 0x{profile:04X}, Device 0x{device:04X}
static const ZB_AF_SIMPLE_DESC_TYPE({num_in}, {num_out}) simple_desc_{id} = {{
    .endpoint = {id},
    .app_profile_id = 0x{profile:04X},
    .app_device_id = 0x{device:04X},
    .app_device_version = 0,
    .reserved = 0,
    .app_input_cluster_count = {num_in},
    .app_output_cluster_count = {num_out},
    .app_cluster_list = {{
{clusters}    }}
}};
"#,
            id = endpoint.id,
            profile = endpoint.profile_id,
            device = endpoint.device_id,
            num_in = endpoint.clusters.iter().filter(|c| c.is_server).count(),
            num_out = endpoint.clusters.iter().filter(|c| !c.is_server).count(),
            clusters = endpoint.clusters.iter()
                .map(|c| format!("        0x{:04X},  // {}\n", c.id, c.name))
                .collect::<String>(),
        ));
    }
    
    code
}
