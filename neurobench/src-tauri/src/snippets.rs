// Code Snippets Module
// Reusable code snippets library with search

use serde::{Deserialize, Serialize};

/// Code snippet definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSnippet {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub language: String,
    pub code: String,
    pub tags: Vec<String>,
}

/// Get all available snippets
pub fn get_snippets() -> Vec<CodeSnippet> {
    vec![
        // GPIO Snippets
        CodeSnippet {
            id: "gpio-output".to_string(),
            name: "GPIO Output Setup".to_string(),
            description: "Configure a GPIO pin as output".to_string(),
            category: "GPIO".to_string(),
            language: "c".to_string(),
            tags: vec!["gpio".to_string(), "output".to_string(), "pin".to_string()],
            code: r#"// Configure GPIO pin as output
void gpio_output_init(GPIO_TypeDef* port, uint8_t pin) {
    port->MODER &= ~(3U << (pin * 2));
    port->MODER |= (1U << (pin * 2));  // Output mode
    port->OTYPER &= ~(1U << pin);       // Push-pull
    port->OSPEEDR |= (3U << (pin * 2)); // High speed
}

// Usage: gpio_output_init(GPIOC, 13);"#.to_string(),
        },
        
        CodeSnippet {
            id: "gpio-input".to_string(),
            name: "GPIO Input with Pull-up".to_string(),
            description: "Configure a GPIO pin as input with pull-up".to_string(),
            category: "GPIO".to_string(),
            language: "c".to_string(),
            tags: vec!["gpio".to_string(), "input".to_string(), "pullup".to_string()],
            code: r#"// Configure GPIO pin as input with pull-up
void gpio_input_init(GPIO_TypeDef* port, uint8_t pin) {
    port->MODER &= ~(3U << (pin * 2));  // Input mode
    port->PUPDR &= ~(3U << (pin * 2));
    port->PUPDR |= (1U << (pin * 2));   // Pull-up
}

// Read pin state
uint8_t gpio_read(GPIO_TypeDef* port, uint8_t pin) {
    return (port->IDR >> pin) & 1;
}"#.to_string(),
        },

        // Timer Snippets
        CodeSnippet {
            id: "systick-delay".to_string(),
            name: "SysTick Delay".to_string(),
            description: "Millisecond delay using SysTick timer".to_string(),
            category: "Timer".to_string(),
            language: "c".to_string(),
            tags: vec!["timer".to_string(), "delay".to_string(), "systick".to_string()],
            code: r#"volatile uint32_t systick_ms = 0;

void SysTick_Handler(void) {
    systick_ms++;
}

void systick_init(void) {
    SysTick_Config(SystemCoreClock / 1000);  // 1ms tick
}

void delay_ms(uint32_t ms) {
    uint32_t start = systick_ms;
    while ((systick_ms - start) < ms);
}

uint32_t millis(void) {
    return systick_ms;
}"#.to_string(),
        },

        CodeSnippet {
            id: "timer-interrupt".to_string(),
            name: "Timer Interrupt".to_string(),
            description: "Configure timer with interrupt".to_string(),
            category: "Timer".to_string(),
            language: "c".to_string(),
            tags: vec!["timer".to_string(), "interrupt".to_string(), "irq".to_string()],
            code: r#"void timer_init(uint32_t freq_hz) {
    RCC->APB1ENR |= RCC_APB1ENR_TIM2EN;
    
    // Timer configuration
    TIM2->PSC = (SystemCoreClock / 10000) - 1;  // 10kHz base
    TIM2->ARR = (10000 / freq_hz) - 1;
    TIM2->DIER = TIM_DIER_UIE;  // Update interrupt
    TIM2->CR1 = TIM_CR1_CEN;    // Enable timer
    
    NVIC_EnableIRQ(TIM2_IRQn);
}

void TIM2_IRQHandler(void) {
    if (TIM2->SR & TIM_SR_UIF) {
        TIM2->SR &= ~TIM_SR_UIF;
        // Your interrupt code here
    }
}"#.to_string(),
        },

        // UART Snippets
        CodeSnippet {
            id: "uart-printf".to_string(),
            name: "UART Printf".to_string(),
            description: "Redirect printf to UART".to_string(),
            category: "UART".to_string(),
            language: "c".to_string(),
            tags: vec!["uart".to_string(), "printf".to_string(), "serial".to_string()],
            code: r#"#include <stdio.h>

// Retarget printf to UART
int _write(int file, char *ptr, int len) {
    for (int i = 0; i < len; i++) {
        while (!(USART2->SR & USART_SR_TXE));
        USART2->DR = ptr[i];
    }
    return len;
}

// Now you can use printf()
// printf("Value: %d\n", value);"#.to_string(),
        },

        CodeSnippet {
            id: "uart-dma".to_string(),
            name: "UART DMA Transfer".to_string(),
            description: "Non-blocking UART transmission with DMA".to_string(),
            category: "UART".to_string(),
            language: "c".to_string(),
            tags: vec!["uart".to_string(), "dma".to_string(), "async".to_string()],
            code: r#"void uart_dma_init(void) {
    // Enable DMA1 clock
    RCC->AHB1ENR |= RCC_AHB1ENR_DMA1EN;
    
    // Configure DMA1 Stream6 for USART2_TX
    DMA1_Stream6->CR = 0;
    DMA1_Stream6->CR |= (4 << 25);  // Channel 4
    DMA1_Stream6->CR |= DMA_SxCR_MINC;  // Memory increment
    DMA1_Stream6->CR |= (1 << 6);   // Memory to peripheral
    DMA1_Stream6->PAR = (uint32_t)&USART2->DR;
    
    USART2->CR3 |= USART_CR3_DMAT;  // Enable DMA TX
}

void uart_dma_send(const char* data, uint16_t len) {
    DMA1_Stream6->M0AR = (uint32_t)data;
    DMA1_Stream6->NDTR = len;
    DMA1_Stream6->CR |= DMA_SxCR_EN;
}"#.to_string(),
        },

        // Interrupt Snippets
        CodeSnippet {
            id: "exti-button".to_string(),
            name: "Button Interrupt".to_string(),
            description: "External interrupt for button press".to_string(),
            category: "Interrupt".to_string(),
            language: "c".to_string(),
            tags: vec!["interrupt".to_string(), "button".to_string(), "exti".to_string()],
            code: r#"void button_exti_init(void) {
    // Enable SYSCFG clock
    RCC->APB2ENR |= RCC_APB2ENR_SYSCFGEN;
    
    // Configure PA0 EXTI
    SYSCFG->EXTICR[0] &= ~0xF;  // PA0
    EXTI->IMR |= (1 << 0);      // Unmask line 0
    EXTI->FTSR |= (1 << 0);     // Falling edge trigger
    
    NVIC_EnableIRQ(EXTI0_IRQn);
}

void EXTI0_IRQHandler(void) {
    if (EXTI->PR & (1 << 0)) {
        EXTI->PR = (1 << 0);  // Clear pending
        // Button pressed - handle here
    }
}"#.to_string(),
        },

        // SPI Snippets
        CodeSnippet {
            id: "spi-init".to_string(),
            name: "SPI Master Init".to_string(),
            description: "Initialize SPI in master mode".to_string(),
            category: "SPI".to_string(),
            language: "c".to_string(),
            tags: vec!["spi".to_string(), "master".to_string(), "init".to_string()],
            code: r#"void spi_init(void) {
    RCC->APB2ENR |= RCC_APB2ENR_SPI1EN;
    RCC->AHB1ENR |= RCC_AHB1ENR_GPIOAEN;
    
    // Configure PA5 (SCK), PA6 (MISO), PA7 (MOSI)
    GPIOA->MODER |= (2 << 10) | (2 << 12) | (2 << 14);
    GPIOA->AFR[0] |= (5 << 20) | (5 << 24) | (5 << 28);
    
    SPI1->CR1 = SPI_CR1_MSTR | SPI_CR1_BR_1 | SPI_CR1_SSM | SPI_CR1_SSI;
    SPI1->CR1 |= SPI_CR1_SPE;
}

uint8_t spi_transfer(uint8_t data) {
    while (!(SPI1->SR & SPI_SR_TXE));
    SPI1->DR = data;
    while (!(SPI1->SR & SPI_SR_RXNE));
    return SPI1->DR;
}"#.to_string(),
        },

        // Low Power Snippets
        CodeSnippet {
            id: "sleep-mode".to_string(),
            name: "Sleep Mode".to_string(),
            description: "Enter low power sleep mode".to_string(),
            category: "Power".to_string(),
            language: "c".to_string(),
            tags: vec!["power".to_string(), "sleep".to_string(), "lowpower".to_string()],
            code: r#"void enter_sleep(void) {
    // Enable sleep on exit from ISR
    SCB->SCR &= ~SCB_SCR_SLEEPDEEP_Msk;
    __WFI();  // Wait for interrupt
}

void enter_stop_mode(void) {
    // Stop mode (deep sleep)
    SCB->SCR |= SCB_SCR_SLEEPDEEP_Msk;
    PWR->CR |= PWR_CR_LPDS;  // Low-power deepsleep
    __WFI();
}

void enter_standby(void) {
    // Lowest power - only RTC/WKUP can wake
    SCB->SCR |= SCB_SCR_SLEEPDEEP_Msk;
    PWR->CR |= PWR_CR_PDDS;
    __WFI();
}"#.to_string(),
        },

        // Ring Buffer
        CodeSnippet {
            id: "ring-buffer".to_string(),
            name: "Ring Buffer".to_string(),
            description: "Circular buffer for FIFO data handling".to_string(),
            category: "Data Structure".to_string(),
            language: "c".to_string(),
            tags: vec!["buffer".to_string(), "fifo".to_string(), "circular".to_string()],
            code: r#"#define BUFFER_SIZE 256

typedef struct {
    uint8_t data[BUFFER_SIZE];
    volatile uint16_t head;
    volatile uint16_t tail;
} RingBuffer;

void rb_init(RingBuffer* rb) {
    rb->head = rb->tail = 0;
}

uint8_t rb_is_empty(RingBuffer* rb) {
    return rb->head == rb->tail;
}

uint8_t rb_is_full(RingBuffer* rb) {
    return ((rb->head + 1) % BUFFER_SIZE) == rb->tail;
}

void rb_push(RingBuffer* rb, uint8_t byte) {
    if (!rb_is_full(rb)) {
        rb->data[rb->head] = byte;
        rb->head = (rb->head + 1) % BUFFER_SIZE;
    }
}

uint8_t rb_pop(RingBuffer* rb) {
    uint8_t byte = rb->data[rb->tail];
    rb->tail = (rb->tail + 1) % BUFFER_SIZE;
    return byte;
}"#.to_string(),
        },
    ]
}

/// Search snippets by query
pub fn search_snippets(query: &str) -> Vec<CodeSnippet> {
    let query_lower = query.to_lowercase();
    get_snippets()
        .into_iter()
        .filter(|s| {
            s.name.to_lowercase().contains(&query_lower)
                || s.description.to_lowercase().contains(&query_lower)
                || s.category.to_lowercase().contains(&query_lower)
                || s.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
        })
        .collect()
}

/// Get snippets by category
pub fn get_snippets_by_category(category: &str) -> Vec<CodeSnippet> {
    get_snippets()
        .into_iter()
        .filter(|s| s.category.to_lowercase() == category.to_lowercase())
        .collect()
}

/// Get snippet by ID
pub fn get_snippet_by_id(id: &str) -> Option<CodeSnippet> {
    get_snippets().into_iter().find(|s| s.id == id)
}

/// Get all categories
pub fn get_snippet_categories() -> Vec<String> {
    let mut categories: Vec<String> = get_snippets()
        .iter()
        .map(|s| s.category.clone())
        .collect();
    categories.sort();
    categories.dedup();
    categories
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_snippets() {
        let snippets = get_snippets();
        assert!(!snippets.is_empty());
    }

    #[test]
    fn test_search_snippets() {
        let results = search_snippets("gpio");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_get_categories() {
        let categories = get_snippet_categories();
        assert!(categories.contains(&"GPIO".to_string()));
    }
}
