// Unit Tests for Driver Code Generators
// Comprehensive test suite for NeuroBench code generation

#[cfg(test)]
mod gpio_tests {
    use crate::drivers::gpio::*;
    use crate::drivers::templates::*;

    #[test]
    fn test_gpio_output_generation() {
        let config = GpioConfig::default();
        let output = generate_gpio_driver(&config, &McuArch::Stm32, &DriverLanguage::C);
        
        assert!(!output.source_file.is_empty(), "Source file should not be empty");
    }

    #[test]
    fn test_gpio_cpp_generation() {
        let config = GpioConfig::default();
        let output = generate_gpio_driver(&config, &McuArch::Stm32, &DriverLanguage::Cpp);
        
        assert!(!output.source_file.is_empty());
    }
}

#[cfg(test)]
mod uart_tests {
    use crate::drivers::uart::*;
    use crate::drivers::templates::*;

    #[test]
    fn test_uart_basic_generation() {
        let config = UartConfig::default();
        let output = generate_uart_driver(&config, &McuArch::Stm32, &DriverLanguage::C);
        
        assert!(!output.source_file.is_empty());
        assert!(output.source_file.contains("USART") || output.source_file.contains("UART") || output.source_file.contains("uart"));
    }
}

#[cfg(test)]
mod spi_tests {
    use crate::drivers::spi::*;
    use crate::drivers::templates::*;

    #[test]
    fn test_spi_master_generation() {
        let config = SpiConfig::default();
        let output = generate_spi_driver(&config, &McuArch::Stm32, &DriverLanguage::C);
        
        assert!(!output.source_file.is_empty());
    }
}

#[cfg(test)]
mod i2c_tests {
    use crate::drivers::i2c::*;
    use crate::drivers::templates::*;

    #[test]
    fn test_i2c_basic_generation() {
        let config = I2cConfig::default();
        let output = generate_i2c_driver(&config, &McuArch::Stm32, &DriverLanguage::C);
        
        assert!(!output.source_file.is_empty());
    }
}

#[cfg(test)]
mod dsp_tests {
    use crate::drivers::dsp::*;
    use crate::drivers::dsp::filters::*;
    use crate::drivers::dsp::fft::*;
    use crate::drivers::dsp::pid::*;

    #[test]
    fn test_fir_filter_generation() {
        let config = FirConfig::default();
        let code = generate_fir_code(&config);
        
        assert!(!code.is_empty(), "FIR code should not be empty");
    }

    #[test]
    fn test_iir_filter_generation() {
        let config = IirConfig::default();
        let code = generate_iir_code(&config);
        
        assert!(!code.is_empty(), "IIR code should not be empty");
    }

    #[test]
    fn test_fft_generation() {
        let config = FftConfig::default();
        let code = generate_fft_code(&config);
        
        assert!(!code.is_empty(), "FFT code should not be empty");
    }

    #[test]
    fn test_pid_generation() {
        let config = PidConfig::default();
        let code = generate_pid_code(&config);
        
        assert!(!code.is_empty(), "PID code should not be empty");
    }
}

#[cfg(test)]
mod rtos_tests {
    use crate::drivers::rtos_gen::*;

    #[test]
    fn test_freertos_task_generation() {
        let hal = freertos::FreeRtosHal::new();
        let config = TaskConfig::default();
        let code = hal.generate_task(&config);
        
        assert!(!code.is_empty(), "Task code should not be empty");
    }

    #[test]
    fn test_freertos_semaphore_generation() {
        let hal = freertos::FreeRtosHal::new();
        let config = SemaphoreConfig {
            name: "sync_sem".to_string(),
            sem_type: SemaphoreType::Binary,
            initial_count: 0,
        };
        
        let code = hal.generate_semaphore(&config);
        assert!(!code.is_empty());
    }

    #[test]
    fn test_zephyr_task_generation() {
        let hal = zephyr::ZephyrHal::new();
        let config = TaskConfig::default();
        let code = hal.generate_task(&config);
        
        assert!(!code.is_empty(), "Zephyr task code should not be empty");
    }
}

#[cfg(test)]
mod clock_tests {
    use crate::drivers::clock::*;

    #[test]
    fn test_clock_frequency_calculation() {
        let config = ClockConfig::default();
        let freqs = calculate_clocks(&config);
        
        assert!(freqs.sysclk > 0, "SYSCLK should be positive");
        assert!(freqs.hclk > 0, "HCLK should be positive");
    }

    #[test]
    fn test_clock_code_generation() {
        let config = ClockConfig::default();
        let code = generate_clock_init(&config);
        
        assert!(!code.is_empty(), "Clock code should not be empty");
    }
}

#[cfg(test)]
mod interrupt_tests {
    use crate::drivers::interrupts::*;

    #[test]
    fn test_exti_interrupt_generation() {
        let config = InterruptConfig::default();
        let code = generate_exti_init(&config, "STM32F401");
        
        assert!(!code.is_empty());
    }

    #[test]
    fn test_timer_generation() {
        let config = TimerConfig::default();
        let code = generate_timer_init(&config, 84_000_000);
        
        assert!(!code.is_empty());
    }
    
    #[test]
    fn test_ticker_generation() {
        let config = TickerConfig::default();
        let code = generate_ticker(&config);
        
        assert!(!code.is_empty());
    }
}
