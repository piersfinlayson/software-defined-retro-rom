// STM32 utils

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#include "include.h"
#include "roms.h"

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
    return (pins & sel_1bit_mask);
}

// Sets up the MCO (clock output) on PA8, to the value provided
void setup_mco(uint8_t mco) {
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
        } else if (sdrr_info.freq <= 240) {
            wait_states = 7;
        } else if (sdrr_info.freq <= 270) {
            wait_states = 8;
        }
    }
    FLASH_ACR &= ~FLASH_ACR_LATENCY_MASK;  // Clear latency bits
    FLASH_ACR |= wait_states & FLASH_ACR_LATENCY_MASK;  // Set wait states

    // Wait for flash latency to be set
    while ((FLASH_ACR & FLASH_ACR_LATENCY_MASK) != wait_states);

    LOG("Set flash config: %d ws", wait_states);
}

//
// Logging functions
//

#if defined(BOOT_LOGGING)
// Linker variables, used by log_init()
extern uint32_t _flash_start;
extern uint32_t _flash_end;
extern uint32_t _ram_size;

// Logging function to output various debug information via RTT
void log_init(void) {
    LOG("%s", log_divider);
    LOG("%s v%d.%d.%d (build %d) - %s", product, sdrr_info.major_version, sdrr_info.minor_version, sdrr_info.patch_version, sdrr_info.build_number, project_url);
    LOG("%s %s", copyright, author);
    LOG("Build date: %s", sdrr_info.build_date);
    LOG("Git commit: %s", sdrr_info.commit);

    uint32_t idcode = DBGMCU_IDCODE;
    const char *idcode_stm_variant;
    idcode = idcode & DBGMCU_IDCODE_DEV_ID_MASK; 
    switch (idcode) {
        case IDCODE_F401XBC:
            idcode_stm_variant = "F401XBC";
            break;
        case IDCODE_F401XDE:
            idcode_stm_variant = "F401XDE";
            break;
        case IDCODE_F4X5:
            idcode_stm_variant = "F405/415";
            break;
        case IDCODE_F411XCE:
            idcode_stm_variant = "F411";
            break;
        case IDCODE_F42_43:
            idcode_stm_variant = "F42X/43X";
            break;
        case IDCODE_F446:
            idcode_stm_variant = "F446";
            break;
        default:
            idcode_stm_variant = "Unknown";
            break;
    }
    LOG("%s", log_divider);
    LOG("Detected hardware info ...");
    LOG("ID Code: %s", idcode_stm_variant);
    uint16_t hw_flash_size = FLASH_SIZE;
    LOG("Flash: %dKB", hw_flash_size);

    LOG("%s", log_divider);
    LOG("Firmware hardware info ...");
    LOG("%s", stm_variant);
    int mismatch = 1;
    switch (sdrr_info.stm_line) {
        case F401:
            if ((idcode == IDCODE_F401XBC) || (idcode == IDCODE_F401XDE)) {
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
        LOG("!!! MCU mismatch: actual %s, firmware expected %s", idcode_stm_variant, stm_variant);
    }

    LOG("PCB rev %s", sdrr_info.hw_rev);
    uint32_t flash_bytes = (uint32_t)(&_flash_end) - (uint32_t)(&_flash_start);
    uint32_t flash_kb = flash_bytes / 1024;
    if (flash_bytes % 1024 != 0) {
        flash_kb += 1;
    }
#if !defined(DEBUG_LOGGING)
    LOG("%s size: %dKB", flash, STM_FLASH_SIZE_KB);
    LOG("%s used: %dKB", flash, flash_kb);
#else // DEBUG_LOGGING
    LOG("%s size: %dKB (%d bytes)", flash, STM_FLASH_SIZE_KB, STM_FLASH_SIZE);
    LOG("%s used: %dKB %d bytes", flash, flash_kb, flash_bytes);
#endif
    if (hw_flash_size != STM_FLASH_SIZE_KB) {
        LOG("!!! Flash size mismatch: actual %dKB, firmware expected %dKB", hw_flash_size, STM_FLASH_SIZE_KB);
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
    if (sdrr_info.bootloader_capable) {
        LOG("Bootloader: %s", enabled);
    } else {
        LOG("Bootloader: %s", disabled);
    }

    // Port assignments
    const char *port_names[] = {"NONE", "A", "B", "C", "D"};


    LOG("%s", log_divider);
    LOG("Pin Configuration ...");
    
    
    LOG("ROM emulation: %d pin ROM", sdrr_info.pins->rom_pins);
    
    // Data pins
    LOG("Data pins D[0-7]: P%s%d,%d,%d,%d,%d,%d,%d,%d", 
        port_names[sdrr_info.pins->data_port],
        sdrr_info.pins->data[0], sdrr_info.pins->data[1], sdrr_info.pins->data[2], sdrr_info.pins->data[3],
        sdrr_info.pins->data[4], sdrr_info.pins->data[5], sdrr_info.pins->data[6], sdrr_info.pins->data[7]);
    
    // Address pins
    LOG("Addr pins A[0-15]: P%s%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d", 
        port_names[sdrr_info.pins->addr_port],
        sdrr_info.pins->addr[0], sdrr_info.pins->addr[1], sdrr_info.pins->addr[2], sdrr_info.pins->addr[3],
        sdrr_info.pins->addr[4], sdrr_info.pins->addr[5], sdrr_info.pins->addr[6], sdrr_info.pins->addr[7],
        sdrr_info.pins->addr[8], sdrr_info.pins->addr[9], sdrr_info.pins->addr[10], sdrr_info.pins->addr[11],
        sdrr_info.pins->addr[12], sdrr_info.pins->addr[13], sdrr_info.pins->addr[14], sdrr_info.pins->addr[15]);
    
    // Chip select pins
    LOG("CS pins - 2364: P%s%d 2332: P%s%d,%d 2316: P%s%d,%d,%d X1: P%s%d X2: P%s%d", 
        port_names[sdrr_info.pins->cs_port], sdrr_info.pins->cs1_2364,
        port_names[sdrr_info.pins->cs_port], sdrr_info.pins->cs1_2332, sdrr_info.pins->cs2_2332,
        port_names[sdrr_info.pins->cs_port], sdrr_info.pins->cs1_2316, sdrr_info.pins->cs2_2316, sdrr_info.pins->cs3_2316,
        port_names[sdrr_info.pins->cs_port], sdrr_info.pins->x1, port_names[sdrr_info.pins->cs_port], sdrr_info.pins->x2);
    
    // Select and status pins
    LOG("Sel pins: P%s%d,%d,%d,%d", port_names[sdrr_info.pins->sel_port], 
        sdrr_info.pins->sel[0], sdrr_info.pins->sel[1], 
        sdrr_info.pins->sel[2], sdrr_info.pins->sel[3]);
    LOG("Status pin: P%s%d", port_names[sdrr_info.pins->status_port], sdrr_info.pins->status);

    LOG("%s", log_divider);
    LOG("ROM info ...");
    LOG("# of ROM sets: %d", sdrr_rom_set_count);
    for (uint8_t ii = 0; ii < sdrr_rom_set_count; ii++) {
        LOG("Set #%d: %d ROM(s), size: %d bytes", ii, rom_set[ii].rom_count, rom_set[ii].size);
        
        for (uint8_t jj = 0; jj < rom_set[ii].rom_count; jj++) {
            const char *rom_type_str;
            const sdrr_rom_info_t *rom = rom_set[ii].roms[jj];
            switch (rom->rom_type) {
                case ROM_TYPE_2364:
                    rom_type_str = r2364;
                    break;
                case ROM_TYPE_2332:
                    rom_type_str = r2332;
                    break;
                case ROM_TYPE_2316:
                    rom_type_str = r2316;
                    break;
                default:
                    rom_type_str = unknown;
                    break;
            }

            LOG("  ROM #%d: %s, %s, CS1: %s, CS2: %s, CS3: %s",
                jj, rom->filename,
                rom_type_str,
                cs_values[rom->cs1_state], cs_values[rom->cs2_state], cs_values[rom->cs3_state]);
        }
    }

#if !defined(EXECUTE_FROM_RAM)
    DEBUG("Execute from: %s", flash);
#else // EXECUTE_FROM_RAM
    LOG("Execute from: %s", ram);
#endif // EXECUTE_FROM_RAM

    LOG("%s", log_divider);
    LOG("Running ...");
}
#endif // BOOT_LOGGING

#if defined(BOOT_LOGGING)
// Special version of logging function that remains on flash, and we can get
// a pointer to, to call from within functions (potentially) loaded to RAM.
// Those functions call RAM_LOG(), which only takes a single arg.
void __attribute__((noinline)) do_log(const char* msg, ...) {
    va_list args;
    va_start(args, msg);
    SEGGER_RTT_vprintf(0, msg, &args);
    va_end(args);
    SEGGER_RTT_printf(0, "\n");
}
#endif // BOOT_LOGGING

//
// Functions to handle copying functions to and executing them from RAM
//

#if defined(EXECUTE_FROM_RAM)

// Copies a function from flash to RAM
void copy_func_to_ram(void (*fn)(void), uint32_t ram_addr, size_t size) {
    // Copy the function to RAM
    memcpy((void*)ram_addr, (void*)((uint32_t)fn & ~1), size);
}

void execute_ram_func(uint32_t ram_addr) {
    // Execute the function in RAM
    void (*ram_func)() = (void(*)(void))(ram_addr | 1);
    ram_func();
}

#endif // EXECUTE_FROM_RAM

// Common setup for stauts LED output using PB15 (inverted logic: 0=on, 1=off)
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

// Simple delay function
void delay(volatile uint32_t count) {
    while(count--);
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
