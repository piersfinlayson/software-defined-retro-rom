// Copyright (c) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT licence

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[cfg(not(feature = "hw-24-f"))]
compile_error!("Only hardware revision 24-f currently supported");

#[allow(unused_imports)]
use defmt::{debug, error, info, trace, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_executor::main as embassy_main;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_time::{Duration, Timer};
use panic_probe as _;

const LED_FLASH_INTERVAL: Duration = Duration::from_millis(500);

// Our main entry point.
#[embassy_main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    debug!("Spawning jumper detection tasks");
    let j1 = ExtiInput::new(p.PB0, p.EXTI0, Pull::Down);
    let j2 = ExtiInput::new(p.PB1, p.EXTI1, Pull::Down);
    let j4 = ExtiInput::new(p.PB2, p.EXTI2, Pull::Down);
    let j8 = ExtiInput::new(p.PB7, p.EXTI7, Pull::Down);
    let x1 = ExtiInput::new(p.PC14, p.EXTI14, Pull::Down);
    let x2 = ExtiInput::new(p.PC15, p.EXTI15, Pull::Down);

    spawner.must_spawn(monitor_pin(j1, "J1"));
    spawner.must_spawn(monitor_pin(j2, "J2"));
    spawner.must_spawn(monitor_pin(j4, "J4"));
    spawner.must_spawn(monitor_pin(j8, "J8"));
    spawner.must_spawn(monitor_pin(x1, "X1"));
    spawner.must_spawn(monitor_pin(x2, "X2"));

    // Set up the LED flashing task
    debug!("Spawning LED flash task");
    let led_pin = Output::new(p.PB15, Level::High, Speed::Medium);
    spawner.must_spawn(flash_led_task(led_pin));

    info!("SDRR Check started");
    info!("- Status LED (if installed to PB15) should be flashing");
    info!("- Open/Close jumpers 1/2/4/8 and X1/X2 and check logs");
    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task(pool_size = 6)]
async fn monitor_pin(mut pin: ExtiInput<'static>, name: &'static str) {
    let mut last_state = pin.is_high();

    loop {
        pin.wait_for_any_edge().await;
        let current_state = pin.is_high();

        if current_state != last_state {
            info!(
                "{} changed: {}",
                name,
                if current_state { "high" } else { "low" }
            );
            last_state = current_state;
        }
    }
}

// A task to flash the status LED
#[embassy_executor::task]
async fn flash_led_task(mut gpio: Output<'static>) -> ! {
    loop {
        // Flash the LED
        gpio.set_high();
        Timer::after(LED_FLASH_INTERVAL).await;

        gpio.set_low();
        Timer::after(LED_FLASH_INTERVAL).await;
    }
}
