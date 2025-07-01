// Main startup code (clock and GPIO initialisation) for the STM32F103

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#include "include.h"

// Sets up the system registers, clock and logging as required
void system_init(void) {
#if defined(BOOT_LOGGING)
    LOG_INIT();
#endif // BOOT_LOGGING

#if defined(MCO)
#if defined(STM32F1)
    setup_mco(RCC_CFGR_MCO_SYSCLK);
#elif defined(STM32F4)
    //setup_mco(RCC_CFGR_MCO1_HSI);
#endif // STM32F1/4
#endif // MCO

#if defined(STM32F411)
#if TARGET_FREQ_MHZ > 84
    // Set power scale 1 mode, as clock speed is 100MHz (> 84MHz, <= 100MHz)
    // Scale defaults to 1 on STM32F405, and not required on STM32F401
    // Must be done before enabling PLL
    LOG("Set VOS to scale 1");
    RCC_APB1ENR |= (1 << 28);   // PWREN bit
    PWR_CR &= ~PWR_VOS_MASK;    // Clear VOS bits
    PWR_CR |= PWR_VOS_SCALE_1;  // Set VOS bits to scale 1
#endif // TARGET_FREQ_MHZ > 84
#endif // STM32F411

    // Always use PLL - note when using HSI, HSI/2 is fed to PLL.  When using
    // HSE, HSE itself is fed to PLL.
#if defined(HSI)
#if defined(DEBUG_LOGGING)
    uint8_t hsi_cal = get_hsi_cal();
    DEBUG("HSI cal value: 0x%x", hsi_cal);
#endif // DEBUG_LOGGING
#if defined(HSI_TRIM)
    trim_hsi(HSI_TRIM);
#else
    DEBUG("Not trimming HSI");
#endif // HSI_TRIM && !HSE
#if defined(STM32F1)
    uint8_t pll_mul = HSI_PLL;
    uint8_t pll_src = RCC_CFGR_PLLSRC_HSI;
#elif defined(STM32F4)
    uint8_t pll_src = RCC_PLLCFGR_PLLSRC_HSI;
#endif // STM32F1/4
#elif defined(HSE) 
    uint8_t pll_mul = HSE_PLL;
#if defined(STM32F1)
    uint8_t pll_src = RCC_CFGR_PLLSRC_HSE;
#elif defined(STM32F4)
    uint8_t pll_src = RCC_PLLCFGR_PLLSRC_HSE;
#endif // STM32F1/4
    enable_hse();
#endif // HSI/HSE

#if defined(USE_PLL)
#if defined(STM32F1)
    setup_pll_mul(pll_mul - 2);  // x16 is 0xE, x15 is 0xD .. x2 is 0x1
#elif defined(STM32F4)
    setup_pll_mul(PLL_M, PLL_N, PLL_P, PLL_Q);
#endif

    setup_pll_src(pll_src);
    enable_pll();
    DEBUG("PLL started");

#if defined(STM32F4) && defined(MCO)
    //setup_mco(RCC_CFGR_MCO1_PLL);
#endif // STM32F4 & MCO

    // Divide SYSCLK by 2 for APB1 bus before we switch to the PLL.
    set_bus_clks();
    DEBUG("SYSCLK/2->APB1");

    // Set flash wait-states - do before we switch to the PLL.
    set_flash_ws();

    set_clock(RCC_CFGR_SW_PLL);
    DEBUG("PLL->SYSCLK");
#endif // USE_PLL

    //while(1);
}

void gpio_init(void) {
    // Enable GPIO ports A, B, and C
#if defined(STM32F1)
    RCC_APB2ENR |= (1 << 2) | (1 << 3) | (1 << 4);
#elif defined(STM32F4)
    RCC_AHB1ENR |= (1 << 0) | (1 << 1) | (1 << 2);
#endif // STM32F1/4

#if defined(STM32F1)
    AFIO_MAPR &= ~(7 << 24); // Clear SWJ_CFG bits
#if defined(SWD)
    // Disable JTAG but keep SWD
    // Set SWJ_CFG bits [26:24] in AFIO_MAPR to 010b (SWD-DP enabled but
    // JTAG-DP disabled)
    AFIO_MAPR |= (0b010 << 24);  // Set to 010b
#else
    AFIO_MAPR |= (0b100 << 24);  // JTAG & SWD both disabled
#endif
#endif // STM32F1

    //
    // GPIOA
    //
#if defined(STM32F1)
    // Set all GPIOs as inputs
    GPIOA_CRL = 0x44444444;
#if defined(SWD)
    uint32_t gpioa_crh = 0x48844444;  // PA13 and PA14 as inputs with pull-up/pull-down
                             // as these are SWD pins
#else // !SWD
    uint32_t gpioa_crh = 0x44444444;
#endif // SWD
    GPIOA_CRH = gpioa_crh;

#if defined(SWD)
    // Set PA14 as pull-down (ODR bit = 0)
    GPIOA_ODR &= ~(1 << 14);   // Pull-down for PA14
    GPIOA_ODR |= (1 << 13);    // Pull-up for PA13
#endif

#elif defined(STM32F4)
    uint32_t gpioa_moder = 0;
    uint32_t gpioa_pupdr = 0;
    uint32_t gpioa_ospeedr = 0x0000ffff;  // PA0-7 very high speed
#if defined(SWD)
    gpioa_moder |= 0x28000000; // Set 13/14 as AF
    gpioa_pupdr |= 0x24000000; // Set pull-up on PA13, down on PA14
#endif // SWD
#if defined(MCO)
    gpioa_moder |= 0x00020000; // Set PA8 as AF
    gpioa_ospeedr |= 0x00030000; // Set PA8 to very high speed
#endif // MCO
    GPIOA_MODER &= ~0xffffffff; // Clear all GPIOA MODER bits
    GPIOA_MODER = gpioa_moder;
    GPIOA_PUPDR = gpioa_pupdr;
    GPIOA_OSPEEDR = gpioa_ospeedr;
#endif // STM32F1/4

    //
    // GPIOB and GPIOC
    //
#if defined(STM32F1)
    GPIOB_CRL = 0x44444444;
    GPIOB_CRH = 0x44444444;
    GPIOC_CRL = 0x44444444;
    GPIOC_CRH = 0x44444444;

    // Set pull-down on PB5, so it always reads 0
    GPIOB_CRL &= ~(0x0F << (5 * 4));    // Clear CNF+MODE bits
    GPIOB_CRL |= (0x08 << (5 * 4));     // Set CNF to 10 (pull-up/pull-down), 
                                        // MODE to 00 (input)
    GPIOB_ODR &= ~(1 << 5); // Set PB5 to 0 (pull-down) 
#elif defined(STM32F4)
    // Set PB0-2 and PB7 as inputs, with pull-downs.  HW rev D only uses
    // PB0-2 but as PB7 isn't connected we can set it here as well.
    // We do this early doors, so the internal pull-downs will have settled
    // before we read the pins.
    GPIOB_MODER &= 0x00000000;  // Set all GPIOs as inputs
    GPIOB_MODER &= ~0x0000C03F;  // Set PB0-2 and PB7 as inputs
    GPIOB_PUPDR &= ~0x0000C03F;  // Clear pull-up/down for PB0-2 and PB7
    GPIOB_PUPDR |= 0x0000802A;   // Set pull-downs on PB0-2 and PB7

    GPIOC_MODER &= 0x00000000;  // Set all GPIOs as inputs
#if defined(MCO2)
    uint32_t gpioc_moder = GPIOC_MODER;
    gpioc_moder &= ~(0b11 << (9 * 2));  // Clear bits for PC9
    gpioc_moder |= 0x00080000; // Set PC9 as AF
    GPIOC_MODER = gpioc_moder;
    GPIOC_OSPEEDR |= 0x000C0000; // Set PC9 to very high speed
    GPIOC_OTYPER &= ~(0b1 << 9);  // Set as push-pull
#else // !MCO2
    GPIOC_PUPDR = 0x00000000; // No pull-up/down
#endif // MCO2
#endif // STM32F1/4

#if defined(MCO)
    // Reset MCO if required
#if defined(STM32F1)
    setup_mco(RCC_CFGR_MCO_SYSCLK);
#endif // STM32F1
#endif // MCO
}

#if !defined(NO_BOOTLOADER) 
// Enters bootloader mode.  This enables UART and SWD so the device can be
// programmed.
void enter_bootloader(void) {
    LOG("Entering bootloader");

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
    // Set the GPIO peripheral clock
#if defined(STM32F1)
    RCC_APB2ENR |= (1 << 4);  // Port C
#elif defined(STM32F4)
    RCC_AHB1ENR |= RCC_AHB1ENR_GPIOBEN;  // Port B
#endif // STM32F1/4

    // Set pins  as inputs
#if defined(STM32F1)
    GPIOC_CRL = 0x00000444;
#elif defined(STM32F4)
    GPIOB_MODER &= ~0x0000C03F;  // Set PB0, PB1, PB2 (& PB7) as inputs
    GPIOB_PUPDR &= ~0x0000C03F;  // Clear pull-up/down on PB0, PB1, PB2 (& PB7)
    GPIOB_PUPDR |= 0x0000002A;   // Set pull-down on PB0, PB1, PB2
#if defined(HW_REV_E)
    GPIOB_PUPDR |= 0x00008000;   // Set pull-down on PB7
#endif // HW_REV_E
#endif // STM32F1/4

    // Add short delay to allow GPIOB to become available - or it may be to
    // allow the pull-downs to settle.
    for(volatile int i = 0; i < 10; i++);

    // Read pins
#if defined(STM32F1)
    uint32_t pins = GPIOC_IDR;
#elif defined(STM32F4)
    uint32_t pins = GPIOB_IDR;
#endif // STM32F1/4

    // Disable peripheral clock for port again - in either case.
#if defined(STM32F1)
    RCC_APB2ENR &= ~(1 << 4);
#elif defined(STM32F4)
    RCC_AHB1ENR &= ~(1 << 1);
#endif // STM32F1/4

    // BOOTLOADER_SEL_MASK only tests PB0-2 if HW_REV_D.
    if ((pins & BOOTLOADER_SEL_MASK) == BOOTLOADER_SEL_MASK) {
        // SEL pins are all high - enter the bootloader
        enter_bootloader();
    }
}
#endif // !NO_BOOTLOADER

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
    // Reset registers
    reset_rcc_registers();

#if defined(STM32F1)
    // No AFIO register on STM32F4
    reset_afio_registers();
#endif // STM32F1

#if !defined(NO_BOOTLOADER)
    // Check whether we should enter the bootloader
    check_enter_bootloader();
#endif // !NO_BOOTLOADER

    // We didn't enter the bootloader, so kick off main processing
    system_init();
    gpio_init();

    uint8_t rom_index = get_rom_index();
    const sdrr_rom_info_t *rom = sdrr_rom_info + rom_index;
#if defined(PRELOAD_TO_RAM) && !defined(TIMER_TEST) && !defined(TOGGLE_PA4)
    // Only bother to preload the ROM image if we are not running a test
    preload_rom_image(rom);
#endif // !TIMER_TEST && !TOGGLE_PA4

#if defined(MCO) && defined(STM32F4)
    // Startup MCO after preloading the ROM - this allows us to test (with a
    // scope), how long the startup takes.
    setup_mco(RCC_CFGR_MCO1_PLL);
#endif

    // Startup - from a stable 5V supply to here - takes:
    // - ~3ms    F411 100MHz BOOT_LOGGING=1
    // - ~1.5ms  F411 100MHz BOOT_LOGGING=0

    //while(1);

    // Execute the main_loop
#if !defined(MAIN_LOOP_LOGGING)
    LOG("Start main loop - logging ends");
#endif // !MAIN_LOOP_LOGGING
#if !defined(EXECUTE_FROM_RAM)
    main_loop(rom);
#else // EXECUTE_FROM_RAM
    // The main loop function was copied to RAM in the ResetHandler
    extern uint32_t _ram_func_start;    // Start of .ram_func section in RAM
    void (*ram_func)(uint8_t) = (void(*)(void))(_ram_func_start | 1);
    ram_func(rom);
#endif

    return 0;
}
