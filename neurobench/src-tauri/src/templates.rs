// Project Templates Module
// Pre-built starter projects for common embedded scenarios

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Project template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub mcu_targets: Vec<String>,
    pub files: Vec<TemplateFile>,
    pub dependencies: Vec<String>,
    pub difficulty: String,  // beginner, intermediate, advanced
}

/// Template file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
    pub description: String,
}

/// Get all available templates
pub fn get_templates() -> Vec<ProjectTemplate> {
    vec![
        // Blinky LED
        ProjectTemplate {
            id: "blinky".to_string(),
            name: "Blinky LED".to_string(),
            description: "Classic LED blink example - the 'Hello World' of embedded".to_string(),
            category: "Basic".to_string(),
            mcu_targets: vec!["STM32F4".to_string(), "STM32F1".to_string(), "ESP32".to_string()],
            difficulty: "beginner".to_string(),
            dependencies: vec![],
            files: vec![
                TemplateFile {
                    path: "main.c".to_string(),
                    description: "Main application".to_string(),
                    content: r#"/**
 * Blinky LED Example
 * Classic LED toggle demonstrating GPIO control
 */

#include "stm32f4xx.h"

#define LED_PIN     13
#define LED_PORT    GPIOC

void delay_ms(uint32_t ms) {
    for (volatile uint32_t i = 0; i < ms * 4000; i++);
}

void gpio_init(void) {
    // Enable GPIOC clock
    RCC->AHB1ENR |= RCC_AHB1ENR_GPIOCEN;
    
    // Configure PC13 as output
    LED_PORT->MODER &= ~(3U << (LED_PIN * 2));
    LED_PORT->MODER |= (1U << (LED_PIN * 2));
    
    // Push-pull output
    LED_PORT->OTYPER &= ~(1U << LED_PIN);
    
    // High speed
    LED_PORT->OSPEEDR |= (3U << (LED_PIN * 2));
}

int main(void) {
    gpio_init();
    
    while (1) {
        LED_PORT->ODR ^= (1U << LED_PIN);
        delay_ms(500);
    }
    
    return 0;
}
"#.to_string(),
                },
            ],
        },

        // UART Echo
        ProjectTemplate {
            id: "uart-echo".to_string(),
            name: "UART Echo".to_string(),
            description: "Serial communication example - echo received data back".to_string(),
            category: "Communication".to_string(),
            mcu_targets: vec!["STM32F4".to_string(), "STM32F1".to_string()],
            difficulty: "beginner".to_string(),
            dependencies: vec![],
            files: vec![
                TemplateFile {
                    path: "main.c".to_string(),
                    description: "Main application".to_string(),
                    content: r#"/**
 * UART Echo Example
 * Echoes back any received serial data
 */

#include "stm32f4xx.h"

#define BAUDRATE 115200
#define APB1_CLOCK 42000000

void uart_init(void) {
    // Enable USART2 and GPIOA clocks
    RCC->APB1ENR |= RCC_APB1ENR_USART2EN;
    RCC->AHB1ENR |= RCC_AHB1ENR_GPIOAEN;
    
    // Configure PA2 (TX) and PA3 (RX) as alternate function
    GPIOA->MODER &= ~((3U << 4) | (3U << 6));
    GPIOA->MODER |= (2U << 4) | (2U << 6);  // AF mode
    
    // Set alternate function to USART2 (AF7)
    GPIOA->AFR[0] |= (7U << 8) | (7U << 12);
    
    // Configure USART2
    USART2->BRR = APB1_CLOCK / BAUDRATE;
    USART2->CR1 = USART_CR1_TE | USART_CR1_RE | USART_CR1_UE;
}

void uart_send(char c) {
    while (!(USART2->SR & USART_SR_TXE));
    USART2->DR = c;
}

char uart_recv(void) {
    while (!(USART2->SR & USART_SR_RXNE));
    return USART2->DR;
}

void uart_print(const char* str) {
    while (*str) {
        uart_send(*str++);
    }
}

int main(void) {
    uart_init();
    uart_print("UART Echo Ready!\r\n");
    
    while (1) {
        char c = uart_recv();
        uart_send(c);  // Echo back
    }
    
    return 0;
}
"#.to_string(),
                },
            ],
        },

        // FreeRTOS Blinky
        ProjectTemplate {
            id: "freertos-blinky".to_string(),
            name: "FreeRTOS Blinky".to_string(),
            description: "Multi-threaded LED blink using FreeRTOS tasks".to_string(),
            category: "RTOS".to_string(),
            mcu_targets: vec!["STM32F4".to_string()],
            difficulty: "intermediate".to_string(),
            dependencies: vec!["FreeRTOS".to_string()],
            files: vec![
                TemplateFile {
                    path: "main.c".to_string(),
                    description: "Main application with RTOS".to_string(),
                    content: r#"/**
 * FreeRTOS Multi-LED Blinky
 * Demonstrates task creation and scheduling
 */

#include "FreeRTOS.h"
#include "task.h"
#include "stm32f4xx.h"

// Task handles
TaskHandle_t led1_task_handle;
TaskHandle_t led2_task_handle;

void led1_task(void* pvParameters) {
    while (1) {
        GPIOC->ODR ^= (1U << 13);
        vTaskDelay(pdMS_TO_TICKS(500));
    }
}

void led2_task(void* pvParameters) {
    while (1) {
        GPIOC->ODR ^= (1U << 14);
        vTaskDelay(pdMS_TO_TICKS(250));
    }
}

void gpio_init(void) {
    RCC->AHB1ENR |= RCC_AHB1ENR_GPIOCEN;
    GPIOC->MODER |= (1U << 26) | (1U << 28);
}

int main(void) {
    gpio_init();
    
    xTaskCreate(led1_task, "LED1", 128, NULL, 1, &led1_task_handle);
    xTaskCreate(led2_task, "LED2", 128, NULL, 1, &led2_task_handle);
    
    vTaskStartScheduler();
    
    while (1);
    return 0;
}
"#.to_string(),
                },
            ],
        },

        // ADC Reading
        ProjectTemplate {
            id: "adc-read".to_string(),
            name: "ADC Reader".to_string(),
            description: "Analog-to-digital converter for sensor reading".to_string(),
            category: "Analog".to_string(),
            mcu_targets: vec!["STM32F4".to_string()],
            difficulty: "beginner".to_string(),
            dependencies: vec![],
            files: vec![
                TemplateFile {
                    path: "main.c".to_string(),
                    description: "ADC reading example".to_string(),
                    content: r#"/**
 * ADC Reading Example
 * Read analog sensor values
 */

#include "stm32f4xx.h"

void adc_init(void) {
    // Enable ADC1 and GPIOA clocks
    RCC->APB2ENR |= RCC_APB2ENR_ADC1EN;
    RCC->AHB1ENR |= RCC_AHB1ENR_GPIOAEN;
    
    // Configure PA0 as analog
    GPIOA->MODER |= (3U << 0);
    
    // ADC configuration
    ADC1->CR2 = 0;
    ADC1->SQR3 = 0;  // Channel 0
    ADC1->CR2 |= ADC_CR2_ADON;
}

uint16_t adc_read(void) {
    ADC1->CR2 |= ADC_CR2_SWSTART;
    while (!(ADC1->SR & ADC_SR_EOC));
    return ADC1->DR;
}

int main(void) {
    adc_init();
    
    while (1) {
        uint16_t value = adc_read();
        // Process ADC value (0-4095)
        (void)value;
    }
    
    return 0;
}
"#.to_string(),
                },
            ],
        },

        // PWM Output
        ProjectTemplate {
            id: "pwm-output".to_string(),
            name: "PWM Output".to_string(),
            description: "Hardware PWM for motor control or LED dimming".to_string(),
            category: "Timers".to_string(),
            mcu_targets: vec!["STM32F4".to_string()],
            difficulty: "intermediate".to_string(),
            dependencies: vec![],
            files: vec![
                TemplateFile {
                    path: "main.c".to_string(),
                    description: "PWM output example".to_string(),
                    content: r#"/**
 * PWM Output Example
 * Generate PWM signal for motor/LED control
 */

#include "stm32f4xx.h"

#define PWM_FREQ    1000    // 1 kHz
#define SYS_CLOCK   168000000

void pwm_init(void) {
    // Enable TIM3 and GPIOB clocks
    RCC->APB1ENR |= RCC_APB1ENR_TIM3EN;
    RCC->AHB1ENR |= RCC_AHB1ENR_GPIOBEN;
    
    // Configure PB4 as alternate function (TIM3_CH1)
    GPIOB->MODER &= ~(3U << 8);
    GPIOB->MODER |= (2U << 8);
    GPIOB->AFR[0] |= (2U << 16);  // AF2 for TIM3
    
    // Timer configuration
    TIM3->PSC = 83;  // 168MHz / 84 = 2MHz
    TIM3->ARR = (2000000 / PWM_FREQ) - 1;
    TIM3->CCR1 = TIM3->ARR / 2;  // 50% duty cycle
    
    // PWM mode 1
    TIM3->CCMR1 = (6U << 4) | TIM_CCMR1_OC1PE;
    TIM3->CCER = TIM_CCER_CC1E;
    TIM3->CR1 = TIM_CR1_CEN;
}

void pwm_set_duty(uint8_t percent) {
    TIM3->CCR1 = (TIM3->ARR * percent) / 100;
}

int main(void) {
    pwm_init();
    
    uint8_t duty = 0;
    int8_t dir = 1;
    
    while (1) {
        pwm_set_duty(duty);
        duty += dir;
        if (duty >= 100 || duty == 0) dir = -dir;
        for (volatile int i = 0; i < 50000; i++);
    }
    
    return 0;
}
"#.to_string(),
                },
            ],
        },

        // I2C Sensor
        ProjectTemplate {
            id: "i2c-sensor".to_string(),
            name: "I2C Sensor".to_string(),
            description: "I2C communication with common sensors".to_string(),
            category: "Communication".to_string(),
            mcu_targets: vec!["STM32F4".to_string()],
            difficulty: "intermediate".to_string(),
            dependencies: vec![],
            files: vec![
                TemplateFile {
                    path: "main.c".to_string(),
                    description: "I2C sensor reading".to_string(),
                    content: r#"/**
 * I2C Sensor Example
 * Read data from I2C devices
 */

#include "stm32f4xx.h"

#define I2C_SPEED   100000  // 100 kHz

void i2c_init(void) {
    RCC->APB1ENR |= RCC_APB1ENR_I2C1EN;
    RCC->AHB1ENR |= RCC_AHB1ENR_GPIOBEN;
    
    // Configure PB6 (SCL) and PB7 (SDA)
    GPIOB->MODER |= (2U << 12) | (2U << 14);
    GPIOB->OTYPER |= (1U << 6) | (1U << 7);  // Open-drain
    GPIOB->AFR[0] |= (4U << 24) | (4U << 28);  // AF4
    
    I2C1->CR2 = 42;  // APB1 clock in MHz
    I2C1->CCR = 210;  // 100kHz
    I2C1->TRISE = 43;
    I2C1->CR1 = I2C_CR1_PE;
}

uint8_t i2c_read_reg(uint8_t addr, uint8_t reg) {
    I2C1->CR1 |= I2C_CR1_START;
    while (!(I2C1->SR1 & I2C_SR1_SB));
    
    I2C1->DR = (addr << 1);
    while (!(I2C1->SR1 & I2C_SR1_ADDR));
    (void)I2C1->SR2;
    
    I2C1->DR = reg;
    while (!(I2C1->SR1 & I2C_SR1_TXE));
    
    I2C1->CR1 |= I2C_CR1_START;
    while (!(I2C1->SR1 & I2C_SR1_SB));
    
    I2C1->DR = (addr << 1) | 1;
    while (!(I2C1->SR1 & I2C_SR1_ADDR));
    I2C1->CR1 &= ~I2C_CR1_ACK;
    (void)I2C1->SR2;
    
    I2C1->CR1 |= I2C_CR1_STOP;
    while (!(I2C1->SR1 & I2C_SR1_RXNE));
    
    return I2C1->DR;
}

int main(void) {
    i2c_init();
    
    while (1) {
        // Read from sensor at address 0x68
        uint8_t data = i2c_read_reg(0x68, 0x00);
        (void)data;
    }
    
    return 0;
}
"#.to_string(),
                },
            ],
        },
    ]
}

/// Get templates by category
pub fn get_templates_by_category(category: &str) -> Vec<ProjectTemplate> {
    get_templates()
        .into_iter()
        .filter(|t| t.category.to_lowercase() == category.to_lowercase())
        .collect()
}

/// Get template by ID
pub fn get_template_by_id(id: &str) -> Option<ProjectTemplate> {
    get_templates().into_iter().find(|t| t.id == id)
}

/// Get all categories
pub fn get_categories() -> Vec<String> {
    let mut categories: Vec<String> = get_templates()
        .iter()
        .map(|t| t.category.clone())
        .collect();
    categories.sort();
    categories.dedup();
    categories
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_templates() {
        let templates = get_templates();
        assert!(!templates.is_empty());
    }

    #[test]
    fn test_get_template_by_id() {
        let template = get_template_by_id("blinky");
        assert!(template.is_some());
        assert_eq!(template.unwrap().name, "Blinky LED");
    }

    #[test]
    fn test_get_categories() {
        let categories = get_categories();
        assert!(categories.contains(&"Basic".to_string()));
    }
}
