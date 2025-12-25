// Bootloader Generator
// Dual-bank bootloader with CRC validation and rollback support

use super::*;

/// Generate bootloader C code
pub fn generate_bootloader_code(config: &BootloaderConfig) -> String {
    let bank_a_start = config.flash_base + config.bootloader_size;
    let bank_b_start = bank_a_start + config.app_size;
    
    let dual_bank_code = match config.bootloader_type {
        BootloaderType::SingleBank => "",
        BootloaderType::DualBank | BootloaderType::DualBankWithRollback => r#"
// Dual-bank configuration
#define BANK_A_START  0x{bank_a:08X}
#define BANK_B_START  0x{bank_b:08X}
#define APP_SIZE      0x{app_size:08X}

typedef struct {
    uint32_t magic;        // 0xDEADBEEF
    uint32_t version;      // Firmware version
    uint32_t size;         // Image size
    uint32_t crc32;        // CRC32 checksum
    uint32_t flags;        // Boot flags
    uint8_t  signature[64]; // Optional signature
} firmware_header_t;

#define FW_MAGIC  0xDEADBEEF
#define FW_FLAG_VALID     (1 << 0)
#define FW_FLAG_PENDING   (1 << 1)
#define FW_FLAG_CONFIRMED (1 << 2)

static uint8_t active_bank = 0;  // 0 = Bank A, 1 = Bank B

bool bootloader_validate_bank(uint32_t bank_start) {
    firmware_header_t *header = (firmware_header_t *)bank_start;
    
    // Check magic
    if (header->magic != FW_MAGIC) {
        return false;
    }
    
    // Check flags
    if (!(header->flags & FW_FLAG_VALID)) {
        return false;
    }
    
    // Check CRC if enabled
    #if ENABLE_CRC_CHECK
    uint32_t calc_crc = crc32_compute(
        (uint8_t *)(bank_start + sizeof(firmware_header_t)),
        header->size
    );
    if (calc_crc != header->crc32) {
        return false;
    }
    #endif
    
    return true;
}

uint8_t bootloader_select_bank(void) {
    bool bank_a_valid = bootloader_validate_bank(BANK_A_START);
    bool bank_b_valid = bootloader_validate_bank(BANK_B_START);
    
    firmware_header_t *header_a = (firmware_header_t *)BANK_A_START;
    firmware_header_t *header_b = (firmware_header_t *)BANK_B_START;
    
    if (bank_a_valid && bank_b_valid) {
        // Both valid, choose higher version
        if (header_a->version >= header_b->version) {
            return 0;
        } else {
            return 1;
        }
    } else if (bank_a_valid) {
        return 0;
    } else if (bank_b_valid) {
        return 1;
    }
    
    // No valid bank, stay in bootloader
    return 0xFF;
}

void bootloader_confirm_update(void) {
    uint32_t bank_start = (active_bank == 0) ? BANK_A_START : BANK_B_START;
    firmware_header_t *header = (firmware_header_t *)bank_start;
    
    // Set confirmed flag
    uint32_t new_flags = header->flags | FW_FLAG_CONFIRMED;
    flash_program_word((uint32_t)&header->flags, new_flags);
}
"#,
    };

    format!(r#"/**
 * Bootloader: {name}
 * Type: {boot_type:?}
 * Flash Base: 0x{flash_base:08X}
 * Bootloader Size: {bootloader_size} bytes
 * Application Size: {app_size} bytes
 */

#include <stdint.h>
#include <stdbool.h>
#include <string.h>

// Flash memory map
#define FLASH_BASE        0x{flash_base:08X}
#define FLASH_SIZE        0x{flash_size:08X}
#define BOOTLOADER_SIZE   0x{bootloader_size:08X}
#define VTOR_OFFSET       0x{vtor:08X}

// Configuration
#define ENABLE_WATCHDOG   {watchdog}
#define BOOT_TIMEOUT_MS   {timeout}
#define ENABLE_CRC_CHECK  {crc_check}
#define ENABLE_SIG_CHECK  {sig_check}
{dual_bank}
// CRC32 calculation (IEEE 802.3 polynomial)
static const uint32_t crc32_table[256];

uint32_t crc32_compute(const uint8_t *data, uint32_t len) {{
    uint32_t crc = 0xFFFFFFFF;
    while (len--) {{
        crc = (crc >> 8) ^ crc32_table[(crc ^ *data++) & 0xFF];
    }}
    return crc ^ 0xFFFFFFFF;
}}

// Jump to application
typedef void (*app_entry_t)(void);

void bootloader_jump_to_app(uint32_t app_address) {{
    // Disable interrupts
    __disable_irq();
    
    // Get the application stack pointer
    uint32_t app_sp = *(uint32_t *)app_address;
    
    // Get the application reset handler
    uint32_t app_reset = *(uint32_t *)(app_address + 4);
    
    // Validate stack pointer (must point to RAM)
    if ((app_sp & 0x2FFE0000) != 0x20000000) {{
        // Invalid stack pointer, stay in bootloader
        return;
    }}
    
    // Set the vector table offset
    SCB->VTOR = app_address;
    
    // Set the main stack pointer
    __set_MSP(app_sp);
    
    // Jump to application
    app_entry_t app_entry = (app_entry_t)app_reset;
    app_entry();
    
    // Should never reach here
    while (1);
}}

// Flash operations
bool flash_unlock(void) {{
    if (FLASH->CR & FLASH_CR_LOCK) {{
        FLASH->KEYR = 0x45670123;
        FLASH->KEYR = 0xCDEF89AB;
    }}
    return !(FLASH->CR & FLASH_CR_LOCK);
}}

void flash_lock(void) {{
    FLASH->CR |= FLASH_CR_LOCK;
}}

bool flash_erase_sector(uint32_t sector) {{
    if (!flash_unlock()) return false;
    
    // Wait for busy
    while (FLASH->SR & FLASH_SR_BSY);
    
    // Set sector erase
    FLASH->CR &= ~FLASH_CR_PSIZE;
    FLASH->CR |= FLASH_CR_PSIZE_1;  // 32-bit parallelism
    FLASH->CR &= ~FLASH_CR_SNB;
    FLASH->CR |= (sector << FLASH_CR_SNB_Pos);
    FLASH->CR |= FLASH_CR_SER;
    FLASH->CR |= FLASH_CR_STRT;
    
    // Wait for completion
    while (FLASH->SR & FLASH_SR_BSY);
    
    FLASH->CR &= ~FLASH_CR_SER;
    flash_lock();
    
    return !(FLASH->SR & 0xF0);  // No errors
}}

bool flash_program_word(uint32_t address, uint32_t data) {{
    if (!flash_unlock()) return false;
    
    while (FLASH->SR & FLASH_SR_BSY);
    
    FLASH->CR &= ~FLASH_CR_PSIZE;
    FLASH->CR |= FLASH_CR_PSIZE_1;
    FLASH->CR |= FLASH_CR_PG;
    
    *(volatile uint32_t *)address = data;
    
    while (FLASH->SR & FLASH_SR_BSY);
    
    FLASH->CR &= ~FLASH_CR_PG;
    flash_lock();
    
    return (*(uint32_t *)address == data);
}}

// Watchdog
#if ENABLE_WATCHDOG
void watchdog_init(void) {{
    IWDG->KR = 0x5555;  // Enable write access
    IWDG->PR = 4;       // Prescaler /64
    IWDG->RLR = 4095;   // ~8 seconds timeout
    IWDG->KR = 0xCCCC;  // Start watchdog
}}

void watchdog_feed(void) {{
    IWDG->KR = 0xAAAA;
}}
#endif

// Main bootloader entry
void bootloader_main(void) {{
    #if ENABLE_WATCHDOG
    watchdog_init();
    #endif
    
    // Check for boot button or stay in bootloader flag
    bool stay_in_bootloader = false;
    
    // Check if boot pin is pressed
    if (BOOT_PIN_ACTIVE) {{
        stay_in_bootloader = true;
    }}
    
    if (!stay_in_bootloader) {{
        uint8_t bank = bootloader_select_bank();
        
        if (bank != 0xFF) {{
            uint32_t app_address = (bank == 0) ? BANK_A_START : BANK_B_START;
            app_address += sizeof(firmware_header_t);
            
            bootloader_jump_to_app(app_address);
        }}
    }}
    
    // Stay in bootloader mode - wait for firmware update
    bootloader_update_mode();
}}

void bootloader_update_mode(void) {{
    // Initialize communication (UART/USB/etc.)
    bootloader_comm_init();
    
    while (1) {{
        #if ENABLE_WATCHDOG
        watchdog_feed();
        #endif
        
        // Process incoming commands
        bootloader_process_commands();
    }}
}}
"#,
        name = config.name,
        boot_type = config.bootloader_type,
        flash_base = config.flash_base,
        flash_size = config.flash_size,
        bootloader_size = config.bootloader_size,
        app_size = config.app_size,
        vtor = config.vector_table_offset,
        watchdog = if config.enable_watchdog { 1 } else { 0 },
        timeout = config.boot_timeout_ms,
        crc_check = if config.enable_crc_check { 1 } else { 0 },
        sig_check = if config.enable_signature_check { 1 } else { 0 },
        dual_bank = dual_bank_code
            .replace("{bank_a:08X}", &format!("{:08X}", bank_a_start))
            .replace("{bank_b:08X}", &format!("{:08X}", bank_b_start))
            .replace("{app_size:08X}", &format!("{:08X}", config.app_size)),
    )
}

/// Generate linker script for bootloader
pub fn generate_bootloader_linker(config: &BootloaderConfig) -> String {
    format!(r#"/* Bootloader Linker Script */
/* Generated by NeuroBench */

MEMORY
{{
    FLASH (rx)  : ORIGIN = 0x{flash_base:08X}, LENGTH = 0x{bootloader_size:08X}
    RAM (rwx)   : ORIGIN = 0x20000000, LENGTH = 64K
}}

ENTRY(Reset_Handler)

SECTIONS
{{
    .isr_vector :
    {{
        . = ALIGN(4);
        KEEP(*(.isr_vector))
        . = ALIGN(4);
    }} >FLASH

    .text :
    {{
        . = ALIGN(4);
        *(.text)
        *(.text*)
        *(.rodata)
        *(.rodata*)
        . = ALIGN(4);
        _etext = .;
    }} >FLASH

    .data :
    {{
        . = ALIGN(4);
        _sdata = .;
        *(.data)
        *(.data*)
        . = ALIGN(4);
        _edata = .;
    }} >RAM AT> FLASH

    .bss :
    {{
        . = ALIGN(4);
        _sbss = .;
        *(.bss)
        *(.bss*)
        *(COMMON)
        . = ALIGN(4);
        _ebss = .;
    }} >RAM
}}
"#,
        flash_base = config.flash_base,
        bootloader_size = config.bootloader_size,
    )
}
