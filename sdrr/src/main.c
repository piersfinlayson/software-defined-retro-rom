// Main startup code (clock and GPIO initialisation) for the STM32F103

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
void system_init(void) {
    if (sdrr_info.boot_logging_enabled) {
        LOG_INIT();
    }

    if ((sdrr_info.stm_line == F405) ||
        (sdrr_info.stm_line == F411) || 
        (sdrr_info.stm_line == F446)) {
        if (sdrr_info.freq > 84) {
            // Set power scale 1 mode, as clock speed is 100MHz (> 84MHz, <= 100MHz)
            // Scale defaults to 1 on STM32F405, and not required on STM32F401
            // Must be done before enabling PLL

            // First, enbale the PWR clock
            LOG("Set VOS to scale 1");
            RCC_APB1ENR |= (1 << 28);   // PWREN bit

            // Wait briefly to see if VOS is ready
            for (int ii = 0; ii < 1000; ii++) {
                if (PWR_CR & PWR_CSR_VOSRDY_MASK) {
                    LOG("VOS ready");
                    break;
                }
            }
            if (!(PWR_CR & PWR_CSR_VOSRDY_MASK)) {
                LOG("!!! VOS not ready - proceeding anyway");
            }

            // Now configure VOS scale mode
            if (sdrr_info.stm_line == F405) {
                PWR_CR &= ~PWR_VOS_MASK_F405; // Clear VOS bits for F405
                PWR_CR |= PWR_VOS_SCALE_1_F405; // Set VOS bits to scale 1 for F405
            } else {
                // For F411 and F446, set VOS to scale 1
                PWR_CR &= ~PWR_VOS_MASK;    // Clear VOS bits
                PWR_CR |= PWR_VOS_SCALE_1;  // Set VOS bits to scale 1
            }
        }
    }

    // Always use PLL - note when using HSI, HSI/2 is fed to PLL.  When using
    // HSE, HSE itself is fed to PLL.
#if defined(DEBUG_LOGGING)
    uint8_t hsi_cal = get_hsi_cal();
    DEBUG("HSI cal value: 0x%x", hsi_cal);
#endif // DEBUG_LOGGING
#if defined(HSI_TRIM)
    trim_hsi(HSI_TRIM);
#else
    DEBUG("Not trimming HSI");
#endif // HSI_TRIM
    uint8_t pll_src = RCC_PLLCFGR_PLLSRC_HSI;

   setup_pll_mul(PLL_M, PLL_N, PLL_P, PLL_Q);

    setup_pll_src(pll_src);
    enable_pll();
    DEBUG("PLL started");

    if ((sdrr_info.stm_line == F446) && (sdrr_info.freq > 168)) {
        // Need to set overdrive mode - wait for it to be ready
        for (int ii = 0; ii < 1000; ii++) {
            if (PWR_CR & PWR_CSR_ODRDY_MASK) {
                LOG("OD ready");
                break;
            }
        }
        if (!(PWR_CR & PWR_CSR_ODRDY_MASK)) {
            LOG("!!! OD not ready - proceeding anyway");
        }

        LOG("Set overdrive mode");
        PWR_CR |= PWR_CR_ODEN;       // Set ODEN bit
        while (!(PWR_CSR & PWR_CSR_ODRDY_MASK)); // Wait for OD to be ready
        PWR_CR |= PWR_CR_ODSWEN;     // Set ODSWEN
        while (!(PWR_CSR & PWR_CSR_ODSWRDY_MASK)); // Wait for ODSW to be ready
        DEBUG("Overdrive mode set");
    }

    // Divide SYSCLK by 2 for APB1 bus before we switch to the PLL.
    set_bus_clks();
    DEBUG("SYSCLK/2->APB1");

    // Set flash wait-states - do before we switch to the PLL.
    set_flash_ws();

    set_clock(RCC_CFGR_SW_PLL);
    DEBUG("PLL->SYSCLK");
}

void gpio_init(void) {
    // Enable GPIO ports A, B, and C
    RCC_AHB1ENR |= (1 << 0) | (1 << 1) | (1 << 2);

    //
    // GPIOA
    //
    uint32_t gpioa_moder = 0;
    uint32_t gpioa_pupdr = 0;
    uint32_t gpioa_ospeedr = 0x0000AAAA;    // PA0-7 fast speed, not high
                                            // speed, to ensure V(OL) max 0.4V

    if (sdrr_info.swd_enabled) {
        gpioa_moder |= 0x28000000; // Set 13/14 as AF
        gpioa_pupdr |= 0x24000000; // Set pull-up on PA13, down on PA14
    }

    if (sdrr_info.mco_enabled) {
        gpioa_moder |= 0x00020000; // Set PA8 as AF
        gpioa_ospeedr |= 0x00030000; // Set PA8 to very high speed
    }
    
    GPIOA_MODER = gpioa_moder;
    GPIOA_PUPDR = gpioa_pupdr;
    GPIOA_OSPEEDR = gpioa_ospeedr;

    //
    // GPIOB and GPIOC
    //

    // Set PB0-2 and PB7 as inputs, with pull-downs.  HW rev D only uses
    // PB0-2 but as PB7 isn't connected we can set it here as well.
    // We do this early doors, so the internal pull-downs will have settled
    // before we read the pins.
    // TODO the code to set PU/PD should be retired (and then tested as the
    // PU/PD is now done in check_sel_pins()).
    GPIOB_MODER = 0;  // Set all GPIOs as inputs
    GPIOB_PUPDR &= ~0x0000C03F;  // Clear pull-up/down for PB0-2 and PB7
    GPIOB_PUPDR |= 0x0000802A;   // Set pull-downs on PB0-2 and PB7

    GPIOC_MODER = 0;  // Set all GPIOs as inputs

#if defined(MCO2)
    uint32_t gpioc_moder = GPIOC_MODER;
    gpioc_moder &= ~(0b11 << (9 * 2));  // Clear bits for PC9
    gpioc_moder |= 0x00080000; // Set PC9 as AF
    GPIOC_MODER = gpioc_moder;
    GPIOC_OSPEEDR |= 0x000C0000; // Set PC9 to very high speed
    GPIOC_OTYPER &= ~(0b1 << 9);  // Set as push-pull
#else // !MCO2
    GPIOC_PUPDR = 0; // No pull-up/down
#endif // MCO2
}

// Enters bootloader mode.  This enables UART and SWD so the device can be
// programmed.
void enter_bootloader(void) {
    LOG("Entering bootloader");

    // Pause to allow the log to be received
    for (int ii = 0; ii < 100000000; ii++);

    // Set the stack pointer
    asm volatile("msr msp, %0" : : "r" (*((uint32_t*)0x1FFFF000)));
    
    // Jump to the bootloader
    ((void(*)())(*((uint32_t*)0x1FFFF004)))();
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

    if (sel_mask == 0) {
        // Failure - no sel pins
        return;
    }
    if ((sel_pins & sel_mask) == sel_mask) {
        // SEL pins are all high - enter the bootloader
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
    // Check if we should enter bootloader mode as the first thing we do
    if (sdrr_info.bootloader_capable) {
        check_enter_bootloader();
    }

    // We didn't enter the bootloader, so kick off main processing
    system_init();
    gpio_init();

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
        setup_mco(RCC_CFGR_MCO1_PLL);
    }

    // Startup - from a stable 5V supply to here - takes:
    // - ~3ms    F411 100MHz BOOT_LOGGING=1
    // - ~1.5ms  F411 100MHz BOOT_LOGGING=0

    // Setup status LED up now, so we don't need to call the function from the
    // main loop - which might be running from RAM.
    if (sdrr_info.status_led_enabled) {
        setup_status_led();
    }

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
