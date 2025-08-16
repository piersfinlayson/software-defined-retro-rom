// One ROM STM32F4 Specific Routines

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#include "include.h"
#include "roms.h"

void setup_clock(void) {
        if ((sdrr_info.mcu_line == F405) ||
        (sdrr_info.mcu_line == F411) || 
        (sdrr_info.mcu_line == F446)) {
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
            if (sdrr_info.mcu_line == F405) {
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

    if ((sdrr_info.mcu_line == F446) && (sdrr_info.freq > 168)) {
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

// Sets up the MCO (clock output) on PA8, to the value provided
void setup_mco(void) {
    uint8_t mco = RCC_CFGR_MCO1_PLL;

    // Enable GPIOA clock
    RCC_AHB1ENR |= (1 << 0);

    uint32_t gpioa_moder = GPIOA_MODER;
    gpioa_moder &= ~(0b11 << (8 * 2));  // Clear bits for PA8
    gpioa_moder |= (0b10 << (8 * 2));   // Set as AF
    GPIOA_MODER = gpioa_moder;
    GPIOA_OSPEEDR |= (0b11 << (8 * 2));  // Set speed to very high speed
    GPIOA_OTYPER &= ~(0b1 << 8);  // Set as push-pull

#if defined(MCO2)
    uint32_t gpioc_moder = GPIOC_MODER;
    gpioc_moder &= ~(0b11 << (9 * 2));  // Clear bits for PC9
    gpioc_moder |= (0b10 << (9 * 2));   // Set as AF
    GPIOC_MODER = gpioc_moder;
    GPIOC_OSPEEDR |= (0b11 << (9 * 2));  // Set speed to very high speed
    GPIOC_OTYPER &= ~(0b1 << 9);  // Set as push-pull
#endif // MCO2

    // Set MCO bits in RCC_CFGR
    uint32_t rcc_cfgr = RCC_CFGR;
    rcc_cfgr &= ~RCC_CFGR_MCO1_MASK;  // Clear MCO1 bits
    rcc_cfgr |= ((mco & 0b11) << 21);  // Set MCO1 bits for PLL
    if ((mco & 0b11) == RCC_CFGR_MCO1_PLL) {
        LOG("MCO1: PLL/4");
        rcc_cfgr &= ~(0b111 << 24); // Clear MCO1 pre-scaler bits
        rcc_cfgr |= (0b110 << 24);  // Set MCO1 pre-scaler to /4
    }
#if defined(MCO2)
    rcc_cfgr &= ~RCC_CFGR_MCO2_MASK;  // Clear MCO2 bits
    rcc_cfgr |= (0b00 << 30);  // Set MCO2 to SYSCLK
    LOG("MCO2: SYSCLK/4");
    rcc_cfgr &= ~(0b111 << 27); // Clear MCO7 pre-scaler bits
    rcc_cfgr |= (0b110 << 27);  // Set MCO7 pre-scaler to /4
#endif // MCO2
    RCC_CFGR = rcc_cfgr;

    // Check MCO configuration in RCC_CFGR
    while(1) {
        uint32_t cfgr = RCC_CFGR;
        uint32_t mco1_bits = (cfgr >> 21) & 0b11;
        if (mco1_bits == (mco & 0b11)) {
            break;  // MCO1 set to PLL
        }
    }
}

uint32_t check_sel_pins(uint32_t *sel_mask) {
    if (sdrr_info.pins->sel_port != PORT_B) {
        // sel_mask of 0 means invalid response
        LOG("!!! Sel port not B - not using");
        *sel_mask = 0;
        return 0;
    }

    // Set the GPIO peripheral clock
    RCC_AHB1ENR |= RCC_AHB1ENR_GPIOBEN;  // Port B

    // Set sel port mask
    uint32_t sel_1bit_mask = 0;
    uint32_t sel_2bit_mask = 0;
    uint32_t pull_downs = 0;
    for (int ii = 0; ii < 4; ii++) {
        uint8_t pin = sdrr_info.pins->sel[ii];
        if (pin < 255) {
            // Pin is present, so set the mask
            if (pin <= 15) {
                sel_1bit_mask |= 1 << pin;
                sel_2bit_mask |= (11 << (pin * 2));
                pull_downs |= (10 << (pin * 2));
            } else {
                LOG("!!! Sel pin 15 < %d < 255 - not using", pin);
            }
        }
    }

    // Set pins as inputs
    GPIOB_MODER &= ~sel_2bit_mask;  // Set sel pins as inputs
    GPIOB_PUPDR &= ~sel_2bit_mask;  // Clear pull-up/down on sel inputs
    GPIOB_PUPDR |= pull_downs;    // Set pull-down on sel inputs

    // Add short delay to allow GPIOB to allow the pull-downs to settle.
    for(volatile int i = 0; i < 10; i++);

    // Read pins
    uint32_t pins = GPIOB_IDR;

    // Disable peripheral clock for port again.
    RCC_AHB1ENR &= ~RCC_AHB1ENR_GPIOBEN;

    // Return sel_mask as well as the value of the pins
    *sel_mask = sel_1bit_mask;

    // Store the value of the pins in sdrr_runtime_info
    sdrr_runtime_info.image_sel = pins & sel_1bit_mask;

    // Return the value of the pins
    return (pins & sel_1bit_mask);
}

// Common setup for status LED output using PB15 (inverted logic: 0=on, 1=off)
void setup_status_led(void) {
    if (sdrr_info.pins->status_port != PORT_B) {
        LOG("!!! Status port not B - not using");
        return;
    }
    if (sdrr_info.pins->status > 15) {
        LOG("!!! Status pin %d > 15 - not using", sdrr_info.pins->status);
        return;
    }
    if (sdrr_info.status_led_enabled) {
        RCC_AHB1ENR |= RCC_AHB1ENR_GPIOBEN; // Enable GPIOB clock
        
        uint8_t pin = sdrr_info.pins->status;
        GPIOB_MODER &= ~(0x3 << (pin * 2));  // Clear bits for PB15
        GPIOB_MODER |= (0x1 << (pin * 2));   // Set as output
        GPIOB_OSPEEDR |= (0x3 << (pin * 2)); // Set speed to high speed
        GPIOB_OTYPER &= ~(0x1 << pin);       // Set as push-pull
        GPIOB_PUPDR &= ~(0x3 << (pin * 2));  // No pull-up/down
        
        GPIOB_BSRR = (1 << pin); // Start with LED off (PB15 high)
    }
}

// Blink pattern: on_time, off_time, repeat_count
void blink_pattern(uint32_t on_time, uint32_t off_time, uint8_t repeats) {
    if (sdrr_info.status_led_enabled && sdrr_info.pins->status_port == PORT_B && sdrr_info.pins->status <= 15) {
        uint8_t pin = sdrr_info.pins->status;
        for(uint8_t i = 0; i < repeats; i++) {
            GPIOB_BSRR = (1 << (pin + 16)); // LED on (PB15 low)
            delay(on_time);
            GPIOB_BSRR = (1 << pin);        // LED off (PB15 high)
            delay(off_time);
        }
    }
}



// Sets up the PLL dividers/multiplier to the values provided
void setup_pll_mul(uint8_t m, uint16_t n, uint8_t p, uint8_t q) {
    // Set PLL multiplier in RCC_PLLCFGR
    uint32_t rcc_pllcfgr = RCC_PLLCFGR;
    rcc_pllcfgr &= RCC_PLLCFGR_RSVD_RO_MASK;  // Clear PLLM bits
    //rcc_pllcfgr &= ~RCC_PLLCFGR_RSVD_RO_MASK;  // Clear PLLM bits WRONG WRONG WRONG
    rcc_pllcfgr |= (q & 0b1111) << 24;
    rcc_pllcfgr |= (p & 0b11) << 16;
    rcc_pllcfgr |= (n & 0b111111111) << 6;
    rcc_pllcfgr |= (m & 0b111111) << 0;
    RCC_PLLCFGR = rcc_pllcfgr;

#if defined(BOOT_LOGGING)
    uint32_t pllcfgr = RCC_PLLCFGR;
    uint32_t actual_m = pllcfgr & 0x3F;
    uint32_t actual_n = (pllcfgr >> 6) & 0x1FF;  
    uint32_t actual_p = (pllcfgr >> 16) & 0x3;
    uint32_t actual_q = (pllcfgr >> 24) & 0xF;
    LOG("Configured PLL MNPQ: %d/%d/%d/%d", actual_m, actual_n, actual_p, actual_q);
#endif // BOOT_LOGGING
}

// Sets up the PLL source to the value provided
void setup_pll_src(uint8_t src) {
    // Set PLL source in RCC_PLLCFGR
    uint32_t rcc_pllcfgr = RCC_PLLCFGR;
    rcc_pllcfgr &= ~RCC_PLLCFGR_PLLSRC_MASK;  // Clear PLLSRC bit
    rcc_pllcfgr |= (src & 1) << 22;  // Set PLLSRC bit
    RCC_PLLCFGR = rcc_pllcfgr;
}

// Enables the PLL and waits for it to be ready
void enable_pll(void) {
    // Enable PLL
    uint32_t rcc_cr = RCC_CR;
    rcc_cr |= RCC_CR_PLLON;  // Set PLLON bit
    RCC_CR = rcc_cr;

    // Wait for PLL to be ready
    while (!(RCC_CR & RCC_CR_PLLRDY));
}

// Enables the HSE and waits for it to be ready.  If driving the PLL, or
// SYSCLK directly, this must be done first.
void enable_hse(void) {
    // Enable HSE
    uint32_t rcc_cr = RCC_CR;
    rcc_cr |= RCC_CR_HSEON;  // Set HSEON bit
    RCC_CR = rcc_cr;

    // Wait for HSE to be ready
    while (!(RCC_CR & RCC_CR_HSERDY));
}

// Get HSI calibration value
uint8_t get_hsi_cal(void) {
    uint32_t rcc_cr = RCC_CR;
    return (rcc_cr & (0xff << 8)) >> 8;  // Return HSI trim value
}

// Sets the system clock to the value provided.  By default the system clock
// uses HSI.  This function cab be used to set it to HSE directly or to the
// PLL.
void set_clock(uint8_t clock) {
    // Set system clock to PLL
    uint32_t rcc_cfgr = RCC_CFGR;
    rcc_cfgr &= ~RCC_CFGR_SW_MASK;  // Clear SW bits
    rcc_cfgr |= (clock & 0b11);  // Set SW bits
    RCC_CFGR = rcc_cfgr;

    // Wait for system clock to be ready
    while ((RCC_CFGR & RCC_CFGR_SWS_MASK) != ((clock & 0b11) << 2));
}

void trim_hsi(uint8_t trim) {
    LOG("Trimming HSI to 0x%X", trim);
    // Set HSI trim value
    uint32_t rcc_cr = RCC_CR;
    rcc_cr &= ~RCC_CR_HSITRIM_MAX;  // Clear HSITRIM bits
    rcc_cr |= ((trim & 0b11111) << 3);
    RCC_CR = rcc_cr;

    // Wait for HSI to be ready
    while (!(RCC_CR & RCC_CR_HSIRDY));
}

// Assumes SYSCLK > 48MHz, divides SYSCLK by 2 for APB1 (slow bus)
void set_bus_clks(void) {
    // AHB = SYSCLK not divided
    RCC_CFGR &= ~RCC_CFGR_HPRE_MASK;

    // APB1 = HCLK/2 (max 36MHz)
    RCC_CFGR &= ~RCC_CFGR_PPRE1_MASK;
    RCC_CFGR |= RCC_CFGR_PPRE1_DIV2;
    
    // APB2 = HCLK not divided
    RCC_CFGR &= ~RCC_CFGR_PPRE2_MASK;
}

// Sets the flash wait states appropriately.  This must be done before
// switching to the PLL as we're running from flash.  Also enable the prefetch
// buffer.
void set_flash_ws(void) {
    uint8_t wait_states = 0;

    // Set data and instruction caches
    FLASH_ACR = FLASH_ACR_PRFTEN | FLASH_ACR_ICEN | FLASH_ACR_DCEN;
    if (sdrr_info.freq > 30) {
        if (sdrr_info.freq <= 60) {
            wait_states = 1;
        } else if (sdrr_info.freq <= 90) {
            wait_states = 2;
        } else if (sdrr_info.freq <= 120) {
            wait_states = 3;
        } else if (sdrr_info.freq <= 150) {
            wait_states = 4;
        } else if (sdrr_info.freq <= 180) {
            wait_states = 5;
        } else if (sdrr_info.freq <= 210) {
            wait_states = 6;
        } else if ((sdrr_info.freq <= 240) || (sdrr_info.mcu_line == F405)) {
            // F405 only has 3 bits for flash wait states so stop here
            wait_states = 7;
        } else if (sdrr_info.freq <= 270) {
            wait_states = 8;
        } else if (sdrr_info.freq <= 300) {
            wait_states = 9;
        } else if (sdrr_info.freq <= 330) {
            wait_states = 10;
        } else if (sdrr_info.freq <= 360) {
            wait_states = 11;
        } else if (sdrr_info.freq <= 390) {
            wait_states = 12;
        } else if (sdrr_info.freq <= 420) {
            wait_states = 13;
        } else if (sdrr_info.freq <= 450) {
            wait_states = 14;
        } else {
            wait_states = 15;
        }
    }
    FLASH_ACR &= ~FLASH_ACR_LATENCY_MASK;  // Clear latency bits
    FLASH_ACR |= wait_states & FLASH_ACR_LATENCY_MASK;  // Set wait states

    // Wait for flash latency to be set
    while ((FLASH_ACR & FLASH_ACR_LATENCY_MASK) != wait_states);

    LOG("Set flash config: %d ws", wait_states);
}

void setup_gpio(void) {
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

void platform_logging(void) {
    uint32_t idcode = DBGMCU_IDCODE;
    const char *idcode_mcu_variant;
    idcode = idcode & DBGMCU_IDCODE_DEV_ID_MASK; 
    switch (idcode) {
        case IDCODE_F401XBC:
            idcode_mcu_variant = "F401XBC";
            break;
        case IDCODE_F401XDE:
            idcode_mcu_variant = "F401XDE";
            break;
        case IDCODE_F4X5:
            idcode_mcu_variant = "F405/415";
            break;
        case IDCODE_F411XCE:
            idcode_mcu_variant = "F411";
            break;
        case IDCODE_F42_43:
            idcode_mcu_variant = "F42X/43X";
            break;
        case IDCODE_F446:
            idcode_mcu_variant = "F446";
            break;
        default:
            idcode_mcu_variant = "Unknown";
            break;
    }
    LOG("%s", log_divider);
    LOG("Detected hardware info ...");
    LOG("ID Code: %s", idcode_mcu_variant);
    uint16_t hw_flash_size = FLASH_SIZE;
    LOG("Flash: %dKB", hw_flash_size);

    LOG("%s", log_divider);
    LOG("Firmware hardware info ...");
    LOG("%s", mcu_variant);
    int mismatch = 1;
    switch (sdrr_info.mcu_line) {
        case F401BC:
            if (idcode == IDCODE_F401XBC) {
                mismatch = 0;
            }
            break;

        case F401DE:
            if (idcode == IDCODE_F401XDE) {
                mismatch = 0;
            }
            break;

        case F405:
            if (idcode == IDCODE_F4X5) {
                mismatch = 0;
            }
            break;
        
        case F411:
            if (idcode == IDCODE_F411XCE) {
                mismatch = 0;
            }
            break;

        case F446:
            if (idcode == IDCODE_F446) {
                mismatch = 0;
            }
            break;
        default:
            break;
    }
    if (mismatch) {
        LOG("!!! MCU mismatch: actual %s, firmware expected %s", idcode_mcu_variant, mcu_variant);
    }

    LOG("PCB rev %s", sdrr_info.hw_rev);
    uint32_t flash_bytes = (uint32_t)(&_flash_end) - (uint32_t)(&_flash_start);
    uint32_t flash_kb = flash_bytes / 1024;
    if (flash_bytes % 1024 != 0) {
        flash_kb += 1;
    }
#if !defined(DEBUG_LOGGING)
    LOG("%s size: %dKB", flash, MCU_FLASH_SIZE_KB);
    LOG("%s used: %dKB", flash, flash_kb);
#else // DEBUG_LOGGING
    LOG("%s size: %dKB (%d bytes)", flash, MCU_FLASH_SIZE_KB, MCU_FLASH_SIZE);
    LOG("%s used: %dKB %d bytes", flash, flash_kb, flash_bytes);
#endif
    if (hw_flash_size != MCU_FLASH_SIZE_KB) {
        LOG("!!! Flash size mismatch: actual %dKB, firmware expected %dKB", hw_flash_size, MCU_FLASH_SIZE_KB);
    }

    uint32_t ram_size_bytes = (uint32_t)&_ram_size;
    uint32_t ram_size_kb = ram_size_bytes / 1024;
#if !defined(DEBUG_LOGGING)
    LOG("RAM: %dKB", ram_size_kb);
#else // DEBUG_LOGGING
    LOG("RAM: %dKB (%d bytes)", ram_size_kb, ram_size_bytes);
#endif

    LOG("Target freq: %dMHz", TARGET_FREQ_MHZ);
    LOG("%s: HSI", oscillator);
#if defined(HSI_TRIM)
    LOG("HSI Trim: 0x%X", HSI_TRIM);
#endif // HSI_TRIM
    LOG("PLL MNPQ: %d/%d/%d/%d", PLL_M, PLL_N, PLL_P, PLL_Q);
    if (sdrr_info.mco_enabled) {
        LOG("MCO: enabled - PA8");
    } else {
        LOG("MCO: disabled");
    }
#if defined(MCO2)
    LOG("MCO2: %s - PC9", enabled);
#endif // MCO2
}
