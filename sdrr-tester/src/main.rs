#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use sdrr_tester::RomTester;
#[allow(unused_imports)]
use sdrr_tester::constants::*;
use sdrr_tester::statistics::calculate_timing_statistics;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    // Initialize the embassy RP runtime
    let _ = embassy_rp::init(Default::default());

    // Log
    #[cfg(feature = "rom_2364")]
    info!("Testing SDRR-23(24P) build variant: 2364");

    #[cfg(feature = "rom_2332")]
    info!("Testing SDRR-23(24P) build variant: 2332");

    #[cfg(feature = "rom_2316")]
    info!("Testing SDRR-23(24P) build variant: 2316");

    info!("---");

    // Log pins
    info!("Address pins: {:?}", ADDR_PINS);
    info!("Data pins: {:?}", DATA_PINS);
    info!("CS1 pin: {}", CS1_PIN);
    #[cfg(any(feature = "rom_2332", feature = "rom_2332"))]
    info!("CS2 pin: {}", CS2_PIN);
    #[cfg(feature = "rom_2316")]
    info!("CS3 pin: {}", CS3_PIN);

    info!("---");
    info!("Attempting reads: {:#x}", MAX_ADDRESS);

    // Timing results arrays (static allocation)
    static mut TIMING_RESULTS_LOW: [u32; MAX_ADDRESS] = [0; MAX_ADDRESS];
    static mut TIMING_RESULTS_HIGH: [u32; MAX_ADDRESS] = [0; MAX_ADDRESS];

    // Create and configure the ROM tester
    #[allow(static_mut_refs)]
    let mut tester = unsafe {
        RomTester::new(&mut TIMING_RESULTS_LOW, &mut TIMING_RESULTS_HIGH)
            .with_reset_pin(RESET_PIN)
            .with_addr_pins(ADDR_PINS)
            .with_data_pins(DATA_PINS)
            .with_cs1_pin(CS1_PIN)
            .with_cs1_active_low(CS1_LOW)
            .with_expected_pattern(&EXPECTED_PATTERN)
            .with_max_address(MAX_ADDRESS)
    };

    // Set CS2 pin if applicable
    tester = {
        #[cfg(any(feature = "cs2_active_low", feature = "cs2_active_high"))]
        {
            tester.with_cs2_pin(CS2_PIN).with_cs2_active_low(CS2_LOW)
        }

        #[cfg(not(any(feature = "cs2_active_low", feature = "cs2_active_high")))]
        {
            tester
        }
    };

    // Set CS3 pin if applicable
    tester = {
        #[cfg(any(feature = "cs3_active_low", feature = "cs3_active_high"))]
        {
            tester.with_cs3_pin(CS3_PIN).with_cs3_active_low(CS3_LOW)
        }

        #[cfg(not(any(feature = "cs3_active_low", feature = "cs3_active_high")))]
        {
            tester
        }
    };

    // Pre-initialize the address, data and reset pins, with weak pull-ups
    info!("Initializing address and data pins...");
    tester.init_pins();
    info!("Address and data pins initialized");

    // Reset the ROM
    info!("Resetting ROM...");
    tester.reset_rom();
    info!("ROM reset complete");

    // Run the test
    info!("Running test...");
    tester.prepare_for_test();
    let successful_reads = tester.run_test();
    info!("Test complete");

    info!("---");

    if successful_reads > 0 {
        // Process timing results
        let timing_low = tester.get_timing_results_low();
        let timing_high = tester.get_timing_results_high();

        // Log successful reads
        info!("Successful reads: {:#x}", successful_reads);

        // Calculate statistics
        calculate_timing_statistics(timing_low, timing_high, successful_reads);
    } else {
        info!("No successful reads. Check configuration and connections.");
    }

    #[allow(clippy::empty_loop)]
    loop {}
}
