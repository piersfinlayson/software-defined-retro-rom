#![no_std]
#![no_main]

pub mod constants;
pub mod statistics;
mod asm;

use asm::{activate_cs_asm, deactivate_cs_asm, ActCsResult, DeactCsResult};
use embedded_hal::delay::{DelayNs};
use core::ptr::{read_volatile, write_volatile};
use cortex_m::interrupt::free;
use defmt::{error, info};

// Register addresses
const IO_BANK0_BASE: u32 = 0x40014000;
const PADS_BANK0_BASE: u32 = 0x4001c000;
const SIO_BASE: u32 = 0xd0000000;
const GPIO_OUT: *mut u32 = (SIO_BASE + 0x010) as *mut u32;
const GPIO_OUT_SET: *mut u32 = (SIO_BASE + 0x014) as *mut u32;
const GPIO_OUT_CLR: *mut u32 = (SIO_BASE + 0x018) as *mut u32;
const GPIO_IN: *const u32 = (SIO_BASE + 0x004) as *const u32;

// Timer registers
const TIMER_BASE: u32 = 0x40054000;
#[allow(clippy::identity_op)]
const TIMER_TIMERAWL: *const u32 = (TIMER_BASE + 0x28) as *const u32;

// Number of address pins
#[cfg(feature = "rom_2364")]
const NUM_ADDR_PINS: usize = 13;
#[cfg(feature = "rom_2332")]
const NUM_ADDR_PINS: usize = 12;
#[cfg(feature = "rom_2316")]
const NUM_ADDR_PINS: usize = 11;

// Function to calculate GPIO control register offset
fn gpio_ctrl_reg_offset(pin: u8) -> u32 {
    0x004 + (pin as u32 * 8)  // 8 bytes per GPIO
}

// Function to calculate GPIO pad control register offset
fn gpio_pad_reg_offset(pin: u8) -> u32 {
    0x004 + (pin as u32 * 4)  // 4 bytes per GPIO pad control
}

pub fn set_gpio_high(pin: u8) {
    unsafe {
        // Set GPIO function to SIO (GPIO control from processor)
        let gpio_ctrl_addr = (IO_BANK0_BASE + gpio_ctrl_reg_offset(pin)) as *mut u32;
        
        // Function 5 (0x5) selects the SIO function
        write_volatile(gpio_ctrl_addr, 0x5);
        
        // Now set GPIO high in the SIO block
        let gpio_out_set_addr = GPIO_OUT_SET;
        
        // Set the pin bit high
        write_volatile(gpio_out_set_addr, 1 << pin);
    }
}

// Function to set any GPIO pin as output
pub fn set_gpio_output(pin: u8) {
    unsafe {
        // Set GPIO function to SIO (GPIO control from processor)
        let gpio_ctrl_addr = (IO_BANK0_BASE + gpio_ctrl_reg_offset(pin)) as *mut u32;
        
        // Function 5 (0x5) selects the SIO function
        write_volatile(gpio_ctrl_addr, 0x5);
        
        // Now set GPIO as output in the SIO block
        let gpio_oe_addr = (SIO_BASE + 0x020) as *mut u32;
        
        // Read current value, set the pin bit, and write back
        let current_oe = read_volatile(gpio_oe_addr);
        write_volatile(gpio_oe_addr, current_oe | (1 << pin));
    }
}

// Function to set any GPIO pin as floating input
pub fn set_gpio_input(pin: u8) {
    unsafe {
        // Set GPIO function to SIO (GPIO control from processor)
        let gpio_ctrl_addr = (IO_BANK0_BASE + gpio_ctrl_reg_offset(pin)) as *mut u32;
        
        // Function 5 (0x5) selects the SIO function
        write_volatile(gpio_ctrl_addr, 0x5);
        
        // Now set GPIO as input in the SIO block
        let gpio_oe_addr = (SIO_BASE + 0x020) as *mut u32;
        
        // Read current value, clear the pin bit, and write back
        let current_oe = read_volatile(gpio_oe_addr);
        write_volatile(gpio_oe_addr, current_oe & !(1 << pin));
    }
}

// Function to set any GPIO pin as input with pullup
pub fn set_gpio_input_with_pullup(pin: u8) {
    unsafe {
        // 1. Set GPIO function to SIO (GPIO control from processor)
        let gpio_ctrl_addr = (IO_BANK0_BASE + gpio_ctrl_reg_offset(pin)) as *mut u32;
        write_volatile(gpio_ctrl_addr, 0x5);
        
        // 2. Set GPIO as input in the SIO block
        let gpio_oe_addr = (SIO_BASE + 0x020) as *mut u32;
        let current_oe = read_volatile(gpio_oe_addr);
        write_volatile(gpio_oe_addr, current_oe & !(1 << pin));
        
        // 3. Configure the pad with pullup enabled
        let pad_reg_addr = (PADS_BANK0_BASE + gpio_pad_reg_offset(pin)) as *mut u32;
        let pad_config = read_volatile(pad_reg_addr);
        
        // Clear pulldown (bit 2), set pullup (bit 3)
        let pad_config_pullup = (pad_config & !(1 << 2)) | (1 << 3);
        write_volatile(pad_reg_addr, pad_config_pullup);
    }
}

/// Configuration for ROM testing
pub struct RomTester<'a> {
    // Configuration
    max_address: usize,
    reset_pin: u8,
    addr_pins: [u8; NUM_ADDR_PINS],
    data_pins: [u8; 8],
    cs_pin: u8,
    cs_active_low: bool,
    expected_pattern: &'a [u8],

    // Pre-computed masks for performance
    addr_masks: [u32; NUM_ADDR_PINS],
    data_masks: [u32; 8],
    cs_mask: u32,
    d7_mask: u32,

    // Results storage
    timing_results_low: &'a mut [u32],
    timing_results_high: &'a mut [u32],
}

impl<'a> RomTester<'a> {
    /// Create a new ROM tester with default configuration
    pub fn new(timing_results_low: &'a mut [u32], timing_results_high: &'a mut [u32]) -> Self {
        // Default values
        let addr_pins = constants::ADDR_PINS;
        let data_pins = constants::DATA_PINS;
        let cs_pin = constants::CS1_PIN;
        let reset_pin = constants::RESET_PIN;

        // Default configuration with empty expected pattern
        Self {
            max_address: 0,
            reset_pin,
            addr_pins,
            data_pins,
            cs_pin,
            cs_active_low: true, // Default to active low CS
            expected_pattern: &[],

            // Initialize masks (will be set correctly in prepare())
            addr_masks: [0; NUM_ADDR_PINS],
            data_masks: [0; 8],
            cs_mask: 0,
            d7_mask: 0,

            timing_results_low,
            timing_results_high,
        }
    }

    /// Set address pins (A0-A12)
    pub fn with_addr_pins(mut self, pins: [u8; NUM_ADDR_PINS]) -> Self {
        self.addr_pins = pins;
        self
    }

    /// Set data pins (D0-D7)
    pub fn with_data_pins(mut self, pins: [u8; 8]) -> Self {
        self.data_pins = pins;
        self
    }

    /// Set ROM reset pin
    pub fn with_reset_pin(mut self, pin: u8) -> Self {
        self.reset_pin = pin;
        self
    }

    /// Set chip select pin
    pub fn with_cs1_pin(mut self, pin: u8) -> Self {
        self.cs_pin = pin;
        self
    }

    /// Set chip select polarity (true for active low, false for active high)
    pub fn with_cs1_active_low(mut self, active_low: bool) -> Self {
        self.cs_active_low = active_low;
        self
    }

    /// Set expected data pattern
    pub fn with_expected_pattern(mut self, pattern: &'a [u8]) -> Self {
        self.expected_pattern = pattern;
        self
    }

    /// Set maximum address to test
    pub fn with_max_address(mut self, max_address: usize) -> Self {
        self.max_address = max_address;
        self
    }

    /// Prepare internal data for testing (compute masks)
    fn prepare(&mut self) {
        // Prepare address pin masks
        for i in 0..NUM_ADDR_PINS {
            self.addr_masks[i] = 1 << self.addr_pins[i];
        }

        // Prepare data pin masks
        for i in 0..8 {
            self.data_masks[i] = 1 << self.data_pins[i];
        }

        // Prepare CS mask
        self.cs_mask = 1 << self.cs_pin;

        // D7 is the highest bit data pin
        // HACK - changed D7 to D6 for testing 
        self.d7_mask = self.data_masks[7];
    }

    /// Initialize address and data pins with pull-up resistors
    pub fn init_pins(&self) {
        for pin in self.addr_pins.iter() {
            set_gpio_input_with_pullup(*pin);
        }
        for pin in self.data_pins.iter() {
            set_gpio_input_with_pullup(*pin);
        }
        set_gpio_input_with_pullup(self.reset_pin);
    }

    /// Reset ROM
    /// 
    /// Sets the reset PIN to an output, pulls it low briefly, and then sets
    /// it back to an input.
    pub fn reset_rom(&self) {
        // Set CS line to an output (high, inactive)
        //set_gpio_high(self.cs_pin);
        //set_gpio_output(self.cs_pin);

        // Set the reset pin to an output
        set_gpio_output(self.reset_pin);

        unsafe {
            // Set the reset pin to low
            write_volatile(GPIO_OUT_CLR, 1 << self.reset_pin);
        }

        // Delay for a short period
        embassy_time::Delay.delay_ms(1);

        // Set the reset pin back to an input
        set_gpio_input_with_pullup(self.reset_pin);

        // Now actiate CS line to see how long the ROM takes to respond -
        // that's the boot time.
        //let (start_time, _, end_time) = self.activate_cs();
        //info!("ROM boot time: {} ticks", end_time.wrapping_sub(start_time));

        // Deactivate CS line again
        //let (_, _) = self.deactivate_cs();
    }

    /// Prepare for the ROM test
    pub fn prepare_for_test(&mut self) {
        // Validate configuration
        if self.max_address == 0 || self.expected_pattern.is_empty() {
            error!("Invalid configuration: max_address or expected_pattern not set");
            return;
        }

        // Check if timing arrays are large enough
        if self.timing_results_low.len() < self.max_address
            || self.timing_results_high.len() < self.max_address
        {
            error!("Timing arrays too small for max_address");
            return;
        }

        // Prepare masks
        self.prepare();

    }

    pub fn run_test(&mut self) -> usize {
        // Run the test with interrupts disabled
        free(|_| unsafe { self.test_rom_inner() })
    }

    // Rust wrapper to call the asm function
    fn activate_cs(&self) -> (u32, u32, u32) {
        // Pre-calculate which GPIO register to use for CS
        let cs_gpio_reg = if self.cs_active_low {
            GPIO_OUT_CLR
        } else {
            GPIO_OUT_SET
        };

        let mut result = ActCsResult {
            start_time: 0,
            data_in: 0,
            end_time: 0,
        };
        
        unsafe {
            // Call the assembly function with your parameters
            activate_cs_asm(
                cs_gpio_reg as *mut u32,
                self.cs_mask,
                TIMER_TIMERAWL as *const u32,
                GPIO_IN as *const u32,
                self.d7_mask,
                &mut result
            );
            
            // Return the values - they come back as:
            // r0 = start_time, r1 = data_in, r2 = end_time
            (result.start_time, result.data_in, result.end_time)
        }
    }

    // Rust wrapper to call the asm function
    fn deactivate_cs(&self) -> (u32, u32) {
        // Pre-calculate which GPIO register to use for CS
        let cs_gpio_reg = if self.cs_active_low {
            GPIO_OUT_SET
        } else {
            GPIO_OUT_CLR
        };

        let mut result = DeactCsResult {
            start_time: 0,
            end_time: 0,
        };
        
        unsafe {
            // Call the assembly function with your parameters
            deactivate_cs_asm(
                cs_gpio_reg as *mut u32,
                self.cs_mask,
                TIMER_TIMERAWL as *const u32,
                GPIO_IN as *const u32,
                self.d7_mask,
                &mut result
            );
            
            // Return the values - they come back as:
            // r0 = start_time, r1 = end_time
            (result.start_time, result.end_time)            
        }
    }

    /// Inner implementation of ROM testing
    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn test_rom_inner(&mut self) -> usize {
        self.deactivate_cs();
        set_gpio_output(self.cs_pin);

        //embassy_time::Delay.delay_ms(10);

        for pin in self.addr_pins.iter() {
            set_gpio_output(*pin);
        }

        // Main test loop
        for addr in 0..self.max_address {
            // Set address pins (A0-A12)
            let mut gpio_val = read_volatile(GPIO_OUT);

            // Clear all address pins first
            for mask in self.addr_masks.iter() {
                gpio_val &= !mask;
            }

            // Set address pins according to address
            for bit in 0..NUM_ADDR_PINS {
                if (addr & (1 << bit)) != 0 {
                    gpio_val |= self.addr_masks[bit];
                }
            }

            // Write the address to GPIO output register
            write_volatile(GPIO_OUT, gpio_val);

            let (start_time, data_in, end_time) = self.activate_cs();

            // Activatate CS and capture timings and data bits
            //info!("Start time: {} ticks", start_time);
            //info!("Data in: 0x{:08X}", data_in);
            //info!("End time: {} ticks", end_time);
            self.timing_results_low[addr] = end_time.wrapping_sub(start_time);
            //info!("Timing result for address 0x{:04X}: {} ticks", addr, self.timing_results_low[addr]);

            // Read data from pins D0-D7
            let mut data_byte: u8 = 0;
            for bit in 0..8 {
                if (data_in & self.data_masks[bit]) != 0 {
                    data_byte |= 1 << bit;
                }
            }
            //info!("Data byte read from D0-D7: 0x{:02X}", data_byte);
            //embassy_time::Delay.delay_us(1);

            //embassy_time::Delay.delay_ms(1000);

            // Compare with expected pattern
            let expected = self.expected_pattern[addr % self.expected_pattern.len()];
            if data_byte != expected {
                error!(
                    "Data mismatch at address 0x{:04X}: expected 0x{:02X}, got 0x{:02X}",
                    addr, expected, data_byte
                );
                set_gpio_input(self.cs_pin);
                return addr;
            }

            // Deactivate CS and capture timings
            let (start_time, end_time) = self.deactivate_cs();

            // Store timing result for going high
            self.timing_results_high[addr] = end_time.wrapping_sub(start_time);

            //embassy_time::Delay.delay_us(1);

            //embassy_time::Delay.delay_ms(1);
        }

        set_gpio_input(self.cs_pin);
        self.max_address
    }

    /// Get the timing results for D7 going low
    pub fn get_timing_results_low(&self) -> &[u32] {
        &self.timing_results_low[0..self.max_address]
    }

    /// Get the timing results for D7 going high
    pub fn get_timing_results_high(&self) -> &[u32] {
        &self.timing_results_high[0..self.max_address]
    }
}

