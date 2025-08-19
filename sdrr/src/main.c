// One ROM Main startup code (clock and GPIO initialisation)

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#include "include.h"

const char sdrr_build_date[] = __DATE__ " " __TIME__;

sdrr_runtime_info_t sdrr_runtime_info __attribute__((section(".sdrr_runtime_info"))) = {
    .magic = {'s', 'd', 'r', 'r'},  // Lower case to distinguish from firmware magic
    .runtime_info_size = sizeof(sdrr_runtime_info_t),
    .image_sel = 0xFF,
    .rom_set_index = 0xFF,
    .count_rom_access = 0x00,
    .access_count = 0xFFFFFFFF,
    .rom_table = NULL,
    .rom_table_size = 0,
};

// Sets up the system registers, clock and logging as required
void clock_init(void) {
    // Set up the clock
#if defined(RP235X) || defined(STM32F4)
    setup_clock();
#else // !RP235X && !STM32F4
    #error "Unsupported MCU line - please define RP235X or STM32F4"
#endif // RP2350/STM32F4
}

void gpio_init(void) {
#if defined(RP235X) || defined(STM32F4)
    setup_gpio();
#else // !RP235X && !STM32F4
    #error "Unsupported MCU line - please define RP235X or STM32F4"
#endif // RP2350/STM32F4
}

// This function checks the state of the image select pins, and returns an
// integer value, as if the sel pins control bit 0, 1, 2, 3, etc in order of
// that integer.  The first sel pin in the array is bit 0, the second bit 1, 
// etc.
uint32_t check_sel_pins(uint32_t *sel_mask) {
    uint32_t num_sel_pins;
    uint32_t orig_sel_mask, gpio_value, sel_value;

    // Setup the pins first.  Do this first to allow any pull-ups to settle
    // before reading.
    num_sel_pins = setup_sel_pins(&orig_sel_mask);
    if (num_sel_pins == 0) {
        LOG("No image select pins");
        disable_sel_pins();
        *sel_mask = 0;
        return 0;
    }

    // Read the actual GPIO value, masked appropriately
    gpio_value = get_sel_value(orig_sel_mask);

    (void)num_sel_pins;  // In case unused - no DEBUG logging 
    DEBUG("Read SIO_GPIO_IN: 0x%08X, %d Sel pins, mask 0x%08X", gpio_value, num_sel_pins, orig_sel_mask);

    disable_sel_pins();

    // Now turn the GPIO value into a SEL value, with the bits consecutive
    // starting from bit 0, based on which pin the SEL value is.  At the same
    // time we have to update sel_mask, to match.  This gives us an integer
    // which can be used as an index into the rom set
    *sel_mask = 0;
    sel_value = 0;
    for (int ii = 0; ii < MAX_IMG_SEL_PINS; ii++) {
        uint8_t pin = sdrr_info.pins->sel[ii];
        if (pin < MAX_USED_GPIOS) {
            if (gpio_value & (1 << pin)) {
                sel_value |= (1 << ii);
            }
            *sel_mask |= (1 << ii);
        }
    }

    LOG("Sel pin value: %d mask: 0x%08X", sel_value, *sel_mask);

    // Store the value of the pins in sdrr_runtime_info
    sdrr_runtime_info.image_sel = sel_value;

    return sel_value;
}

// Check whether we shoud enter the device's bootloader and, if so, enter it.
// This is indicated via jumping SEL0, SEL1, and SEL2 - PB0-2.  These are all
// pulled up to enter the bootloader.  STM32F4 variant from rev E onwards also
// include PB7 as the most significant bit.
//
// This must be done before we set up the PLL, peripheral clocks, etc, as
// those must be disabled for the bootloader.
void check_enter_bootloader(void) {
    uint32_t sel_pins, sel_mask;
    sel_pins = check_sel_pins(&sel_mask);

    LOG("Checking whether to enter bootloader");

    if (sel_mask == 0) {
        // Failure - no sel pins
        return;
    }
    if ((sel_pins & sel_mask) == sel_mask) {
        // SEL pins are all high - enter the bootloader
        LOG("Entering bootloader");

        // Pause to allow the log to be received
        for (int ii = 0; ii < 100000000; ii++);

        enter_bootloader();
    }
}

// Needs to do the following:
// - Set up the clock to 68.8Mhz
// - Set up GPIO ports A, B and C to inputs
// - Load the selected ROM image into RAM for faster access
// - Run the main loop, from RAM
//
// Startup needs to be a small number of hundreds of ms, so it's complete and
// the main loop is running before the other hardware is accessing the ROM.
//
// The hardware takes around 200us to power up, then maybe 200us for the PLL to
// lock, in clock_init().  The rest of time we have for our code.
//
// preload_rom_image is likely to take the longest, as it is copying an 8KB
// ROM image to RAM, and having to deal with the internal complexity of
// remapping the data to the bit ordering we need, and to skip bit 3 (and use
// bit 14 instead).
int main(void) {
    // Platform specific initialization
    platform_specific_init();

    // Initialize GPIOs.  Do it now before checking bootloader mode.
    gpio_init();

    // Enable logging
    if (sdrr_info.boot_logging_enabled) {
        LOG_INIT();
    }

    // Check if we should enter bootloader mode as the first thing we do
    if (sdrr_info.bootloader_capable) {
        check_enter_bootloader();
    }

    // Initialize clock
    clock_init();

    sdrr_runtime_info.rom_set_index = get_rom_set_index();
    const sdrr_rom_set_t *set = rom_set + sdrr_runtime_info.rom_set_index;
#if !defined(TIMER_TEST) && !defined(TOGGLE_PA4)
    // Set up the ROM table
    if (sdrr_info.preload_image_to_ram) {
        sdrr_runtime_info.rom_table = preload_rom_image(set);
    } else {
        // If we are not preloading the ROM image, we need to set up the
        // rom_table to point to the flash location of the ROM image.
        sdrr_runtime_info.rom_table = (void *)&(set->data[0]);
    }
    sdrr_runtime_info.rom_table_size = set->size;
#endif // !TIMER_TEST && !TOGGLE_PA4

    // Startup MCO after preloading the ROM - this allows us to test (with a
    // scope), how long the startup takes.
    if (sdrr_info.mco_enabled) {
        setup_mco();
    }

    // Setup status LED up now, so we don't need to call the function from the
    // main loop - which might be running from RAM.
    if (sdrr_info.status_led_enabled) {
        setup_status_led();
    }

    // Do final checks before entering the main loop
    check_config(&sdrr_info, set);

    // Startup - from a stable 5V supply to here - takes:
    // - ~3ms    F411 100MHz BOOT_LOGGING=1
    // - ~1.5ms  F411 100MHz BOOT_LOGGING=0

    // Execute the main_loop
#if !defined(MAIN_LOOP_LOGGING)
    LOG("Start main loop - logging ends");
#endif // !MAIN_LOOP_LOGGING

#if !defined(EXECUTE_FROM_RAM)
    main_loop(&sdrr_info, set);
#else // EXECUTE_FROM_RAM
#ifndef PRELOAD_TO_RAM
#error "PRELOAD_TO_RAM must be defined when EXECUTE_FROM_RAM is enabled"
#endif // !PRELOAD_TO_RAM

    // We need to set up a copy of some of sdrr_info and linked to data, in
    // order for main_loop() to be able to access it.  If we don't do this,
    // main_loop() will try to access the original sdrr_info, which is in
    // flash, and it will use relative addressing, which won't work.

    // Set up addresses to copy sdrr_info and related data to

    // These come from the linker
    extern uint8_t _sdrr_info_ram_start[];
    extern uint8_t _sdrr_info_ram_end[];

    // The _addresses_ of the linker variables are the locations we're
    // interested in
    uint8_t *sdrr_info_ram_start = &_sdrr_info_ram_start[0];
    uint8_t *sdrr_info_ram_end = &_sdrr_info_ram_end[0];
    uint32_t ram_size = sdrr_info_ram_end - sdrr_info_ram_start;
    uint32_t required_size = sizeof(sdrr_info_t) + sizeof(sdrr_pins_t) + sizeof(sdrr_rom_set_t);
    DEBUG("RAM start: 0x%08X, end: 0x%08X", (unsigned int)sdrr_info_ram_start, (unsigned int)sdrr_info_ram_end);
    DEBUG("RAM size: 0x%08X bytes, required size: 0x%08X bytes", ram_size, required_size);
    if (required_size > ram_size) {
        LOG("!!! Not enough RAM for sdrr_info and related data");
    }
    // Continue away :-|

    // Copy sdrr_info to RAM
    uint8_t *ptr = sdrr_info_ram_start;
    sdrr_info_t *info = (sdrr_info_t *)ptr;
    memcpy(info, &sdrr_info, sizeof(sdrr_info_t));
    DEBUG("Copied sdrr_info to RAM at 0x%08X", (uint32_t)info);
    ptr += sizeof(sdrr_info_t);

    // Copy the pins and update sdrr_info which points to pins
    sdrr_pins_t *pins = (sdrr_pins_t *)ptr;
    memcpy(pins, sdrr_info.pins, sizeof(sdrr_pins_t));
    DEBUG("Copied sdrr_pins to RAM at 0x%08X", (uint32_t)pins);
    info->pins = pins;
    ptr += sizeof(sdrr_pins_t);

    // Copy the rom_set to RAM
    sdrr_rom_set_t *rom_set = (sdrr_rom_set_t *)ptr;
    memcpy(rom_set, set, sizeof(sdrr_rom_set_t));
    DEBUG("Copied sdrr_rom_set to RAM at 0x%08X", (uint32_t)rom_set);
    ptr += sizeof(sdrr_rom_set_t);

    // The main loop function was copied to RAM in the ResetHandler
    extern uint32_t _ram_func_start;
    void (*ram_func)(const sdrr_info_t *, const sdrr_rom_set_t *set) = (void(*)(const sdrr_info_t *, const sdrr_rom_set_t *set))((uint32_t)&_ram_func_start | 1);
    DEBUG("Executing main_loop from RAM at 0x%08X", (uint32_t)ram_func);
    ram_func(info, rom_set);
#endif // !EXECUTE_FROM_RAM

    return 0;
}
