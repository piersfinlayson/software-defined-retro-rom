// STM32 utils

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#include "include.h"
#include "roms.h"

// Most/all of this is likely unnecessary
void reset_rcc_registers(void) {
#if defined(STM32F1)
    // Can't be bothered to implement for STM32F4 as unnecessary

    // Set RCC_CR to reset value
    // Retain reserved and read-only bits
    // Set HSION and default HSI trim value
    uint32_t rcc_cr = RCC_CR;
    rcc_cr &= RCC_CR_RSVD_RO_MASK; 
    rcc_cr |= RCC_CR_HSION | (0x10 << 3);
    RCC_CR = rcc_cr;

    // Set RCC_CFGR to reset value
    // Retain reserved and read-only bits
    // Set SW to HSI (value 0)
    uint32_t rcc_cfgr = RCC_CFGR;
    rcc_cfgr &= RCC_CFGR_RSVD_RO_MASK;
    RCC_CFGR = rcc_cfgr;

    // Set RCC_CIR to reset value
    uint32_t rcc_cir = RCC_CIR;
    rcc_cir &= RCC_CIR_RSVD_RO_MASK;
    RCC_CIR = rcc_cir;

    // Set RCC_APB2RSTR to reset value
    uint32_t rcc_apb2rstr = RCC_APB2RSTR;
    rcc_apb2rstr &= RCC_APB2RSTR_RSVD_RO_MASK;
    RCC_APB2RSTR = rcc_apb2rstr;

    // Set RCC_APB1RSTR to reset value
    uint32_t rcc_apb1rstr = RCC_APB1RSTR;
    rcc_apb1rstr &= RCC_APB1RSTR_RSVD_RO_MASK;
    RCC_APB1RSTR = rcc_apb1rstr;

    // Set RCC_AHBENR to reset value
    uint32_t rcc_ahbenr = RCC_AHBENR;
    rcc_ahbenr &= RCC_AHBENR_RSVD_RO_MASK;
    rcc_ahbenr |= (1 << 4);  // FLITF clock enabled during sleep mode
    RCC_AHBENR = rcc_ahbenr;

    // Set RCC_APB2ENR to reset value
    uint32_t rcc_apb2enr = RCC_APB2ENR;
    rcc_apb2enr &= RCC_APB2ENR_RSVD_RO_MASK;
    RCC_APB2ENR = rcc_apb2enr;

    // Set RCC_APB1ENR to reset value
    uint32_t rcc_apb1enr = RCC_APB1ENR;
    rcc_apb1enr &= RCC_APB1ENR_RSVD_RO_MASK;
    RCC_APB1ENR = rcc_apb1enr;

    // Set RCC_BDCR to reset value
    uint32_t rcc_bdcr = RCC_BDCR;
    rcc_bdcr &= RCC_BDCR_RSVD_RO_MASK;
    RCC_BDCR = rcc_bdcr;
#endif // STM32F1
}

// May be unnecessary
#if defined(STM32F1)
void reset_afio_registers(void) {
    uint32_t afio_mapr = AFIO_MAPR;
    afio_mapr &= AFIO_MAPR_RSVD_RO_MASK;
    AFIO_MAPR = afio_mapr;
}
#endif // STM32F1

// Sets up the MCO (clock output) on PA8, to the value provided
#if defined(MCO)
void setup_mco(uint8_t mco) {
    // Enable GPIOA clock
#if defined(STM32F1)
    RCC_APB2ENR |= (1 << 2);
#elif defined(STM32F4)
    RCC_AHB1ENR |= (1 << 0);
#endif // STM32F1/4

#if defined(STM32F1)
    // Configure PA8 as output (MODE=11, CNF=00)
    uint32_t gpioa_crh = GPIOA_CRH;
    gpioa_crh &= ~(0b1111 << 0);  // Clear CND and MODE bits for PA8
    gpioa_crh |= (0b1011 << 0);   // Set as AF 50MHz push-pull
    GPIOA_CRH = gpioa_crh;
#elif defined(STM32F4)
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
#endif // STM32F1/4

    // Set MCO bits in RCC_CFGR
    uint32_t rcc_cfgr = RCC_CFGR;
#if defined(STM32F1)
    rcc_cfgr &= ~RCC_CFGR_MCO_MASK;  // Clear MCO bits
    rcc_cfgr |= ((mco & 0b111) << 24);  // Set MCO bits
#elif defined(STM32F4)
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
#endif // STM32F4
    RCC_CFGR = rcc_cfgr;

    // Check MCO configuration in RCC_CFGR
#if defined(STM32F1)
    while(1) {
        uint32_t cfgr = RCC_CFGR;
        uint32_t mco_bits = (cfgr >> 24) & 0b111;
        if (mco_bits == (mco & 0b111)) {
            break;  // MCO1 set to PLL
        }
    }
#elif defined(STM32F4)    
    while(1) {
        uint32_t cfgr = RCC_CFGR;
        uint32_t mco1_bits = (cfgr >> 21) & 0b11;
        if (mco1_bits == (mco & 0b11)) {
            break;  // MCO1 set to PLL
        }
    }
#endif // STM32F1/4
}
#endif // MCO

#if defined(STM32F1)
// Sets up the PLL multiplier to the value provided
void setup_pll_mul(uint8_t mul) {
    // Set PLL multiplier in RCC_CFGR
    uint32_t rcc_cfgr = RCC_CFGR;
    rcc_cfgr &= ~RCC_CFGR_PLLMULL_MASK;  // Clear PLLMUL bits
    rcc_cfgr |= ((mul & 0b1111) << 18);  // Set PLLMUL bits
    RCC_CFGR = rcc_cfgr;
}
#endif // STM32F1
#if defined(STM32F4)
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
#endif // STM32F4

// Sets up the PLL source to the value provided
void setup_pll_src(uint8_t src) {
#if defined(STM32F1)
    // Set PLL source in RCC_CFGR
    uint32_t rcc_cfgr = RCC_CFGR;
    rcc_cfgr &= ~RCC_CFGR_PLLSRC;  // Clear PLLSRC bit
    rcc_cfgr |= (src & 1) << 16;  // Set PLLSRC bit
    RCC_CFGR = rcc_cfgr;
#elif defined(STM32F4)
    // Set PLL source in RCC_PLLCFGR
    uint32_t rcc_pllcfgr = RCC_PLLCFGR;
    rcc_pllcfgr &= ~RCC_PLLCFGR_PLLSRC_MASK;  // Clear PLLSRC bit
    rcc_pllcfgr |= (src & 1) << 22;  // Set PLLSRC bit
    RCC_PLLCFGR = rcc_pllcfgr;
#endif // STM32F4
}

#if defined(STM32F1)
// Sets up the PLL XTPRE to the value provided - this is the HSE divider
// for the PLL input clock
void setup_pll_xtpre(uint8_t xtpre) {
    // Set PLL XTPRE in RCC_CFGR
    uint32_t rcc_cfgr = RCC_CFGR;
    rcc_cfgr &= ~RCC_CFGR_PLLXTPRE_MASK;  // Clear PLLXTPRE bit
    rcc_cfgr |= (xtpre & 0b1) << 17;  // Set PLLXTPRE bit
    RCC_CFGR = rcc_cfgr;
}
#endif // STM32F1

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
    uint8_t wait_states;
#ifdef STM32F1
#if TARGET_FREQ_MHZ <= 24
    wait_states = 0;
#elif TARGET_FREQ_MHZ <= 48
    wait_states = 1;
#elif TARGET_FREQ_MHZ <= 72
    wait_states = 2;
#else
#error "Unsupported frequency for STM32F1"
#endif // TARGET_FREQ_MHZ
    FLASH_ACR = FLASH_ACR_PRFTBE;
#elif defined(STM32F4)
    // Set data and instruction caches
    FLASH_ACR = FLASH_ACR_PRFTEN | FLASH_ACR_ICEN | FLASH_ACR_DCEN;
#if TARGET_FREQ_MHZ <= 30
    wait_states = 0;
#elif TARGET_FREQ_MHZ <= 60
    wait_states = 1;
#elif TARGET_FREQ_MHZ <= 90
    wait_states = 2;
#elif TARGET_FREQ_MHZ <= 120
    wait_states = 3;
#elif TARGET_FREQ_MHZ <= 150
    wait_states = 4;
#elif TARGET_FREQ_MHZ <= 180
    wait_states = 5;
#elif TARGET_FREQ_MHZ <= 210
    wait_states = 6;
#else
#error "Unsupported frequency for STM32F4"
#endif // TARGET_FREQ_MHZ
#endif // STM32F1/4
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
const char *get_cs_str(sdrr_cs_state_t cs) {
    switch (cs) {
        case CS_ACTIVE_LOW:
            return cs_low;
        case CS_ACTIVE_HIGH:
            return cs_high;
        case CS_NOT_USED:
            return cs_na;
        default:
            return unknown;
    }
}

// Linker variables, used by log_init()
extern uint32_t _flash_start;
extern uint32_t _flash_end;
extern uint32_t _ram_size;

// Logging function to output various debug information via RTT
void log_init(void) {
    LOG("%s", log_divider);
    LOG("%s v%s - %s", product, version, project_url);
    LOG("%s %s", copyright, author);
    LOG("Build date: %s", build_date);

    LOG("%s", log_divider);
    LOG("Hardware info ...");
    LOG("STM32%s", stm_variant);
    char hw_rev;
#if defined(HW_REV_A)
    hw_rev = 'A';
#elif defined(HW_REV_B)
    hw_rev = 'B';
#elif defined(HW_REV_C)
    hw_rev = 'C';
#elif defined(HW_REV_D)
    hw_rev = 'D';
#elif defined(HW_REV_E)
    hw_rev = 'E';
#elif defined(HW_REV_F)
    hw_rev = 'F';
#else
#error "Unknown hardware revision"
#endif // HW_REV_A/B/C/D/E/F
    LOG("PCB rev %c", hw_rev);
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

    uint32_t ram_size_bytes = (uint32_t)&_ram_size;
    uint32_t ram_size_kb = ram_size_bytes / 1024;
#if !defined(DEBUG_LOGGING)
    LOG("RAM: %dKB", ram_size_kb);
#else // DEBUG_LOGGING
    LOG("RAM: %dKB (%d bytes)", ram_size_kb, ram_size_bytes);
#endif

#if defined(USE_PLL)
    LOG("Target freq: %dMHz", TARGET_FREQ_MHZ);
#endif // USE_PLL
#if defined(HSI)
    LOG("%s: HSI", oscillator);
#if defined(HSI_TRIM)
    LOG("HSI Trim: 0x%X", HSI_TRIM);
#endif // HSI_TRIM
#if defined(USE_PLL)
#if defined(STM32F1)
    LOG("PLLx: %d", HSI_PLL);
#elif defined(STM32F4)
    LOG("PLL MNPQ: %d/%d/%d/%d", PLL_M, PLL_N, PLL_P, PLL_Q);
#endif // STM32F1/4
#endif // USE_PLL
#endif // HSI
#if defined(HSE)
    LOG("%s: HSE", oscillator);
#if defined(USE_PLL)
    LOG("PLLx: %d", HSE_PLL);
#endif // USE_PLL
#endif // HSE
#if defined(MCO)
    LOG("MCO: %s - PA8", enabled);
#if defined(MCO2)
    LOG("MCO2: %s - PC9", enabled);
#endif // MCO2
#else // !MCO
    LOG("MCO: %s", disabled);
#endif // MCO
#if !defined(NO_BOOTLOADER)
    LOG("%s %s", stm32_bootloader_mode, enabled);
#else // NO_BOOTLOADER
    LOG("%s %s", stm32_bootloader_mode, disabled);
#endif // NO_BOOTLOADER

    LOG("%s", log_divider);
    LOG("Firmware info ...");
    LOG("# of ROM sets: %d", SDRR_NUM_SETS);
    for (uint8_t ii = 0; ii < SDRR_NUM_SETS; ii++) {
        const char *rom_type_str;
        const sdrr_rom_info_t *rom = rom_set[ii].roms[0];
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

        const char *cs1_state_str = get_cs_str(rom->cs1_state);
        const char *cs2_state_str = get_cs_str(rom->cs2_state); 
        const char *cs3_state_str = get_cs_str(rom->cs3_state);

#if !defined(DEBUG_LOGGING)
        LOG("#%d: %s, %s, CS1: %s, CS2: %s, CS3: %s", ii, rom->filename, rom_type_str, cs1_state_str, cs2_state_str, cs3_state_str);
#else // DEBUG_LOGGING
        LOG("#%d: %s, %s, CS1: %s, CS2: %s, CS3: %s, size: %d bytes", ii, rom->filename, rom_type_str, cs1_state_str, cs2_state_str, cs3_state_str, rom_set[ii].size);
#endif // DEBUG_LOGGING
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
#if defined(STM32F4)
#if defined(STATUS_LED)
    RCC_AHB1ENR |= RCC_AHB1ENR_GPIOBEN; // Enable GPIOB clock
    
    GPIOB_MODER &= ~(0x3 << (15 * 2));  // Clear bits for PB15
    GPIOB_MODER |= (0x1 << (15 * 2));   // Set as output
    GPIOB_OSPEEDR |= (0x3 << (15 * 2)); // Set speed to high speed
    GPIOB_OTYPER &= ~(0x1 << 15);       // Set as push-pull
    GPIOB_PUPDR &= ~(0x3 << (15 * 2));  // No pull-up/down
    
    GPIOB_BSRR = (1 << 15); // Start with LED off (PB15 high)
#endif // STATUS_LED
#endif // STM32F4
}

// Simple delay function
void delay(volatile uint32_t count) {
    while(count--);
}

// Blink pattern: on_time, off_time, repeat_count
void blink_pattern(uint32_t on_time, uint32_t off_time, uint8_t repeats) {
#if defined(STATUS_LED)
    for(uint8_t i = 0; i < repeats; i++) {
        GPIOB_BSRR = (1 << (15 + 16)); // LED on (PB15 low)
        delay(on_time);
        GPIOB_BSRR = (1 << 15);        // LED off (PB15 high)
        delay(off_time);
    }
#else // !STATUS_LED
    (void)on_time;
    (void)off_time;
    (void)repeats;
#endif // STATUS_LED
}
