// 2332/2364 ROM implementation.

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

// This file contains the STM32 main loop emulating the 2316/2332/2364 ROM.
// It is highly optimised for speed and aims to exceed (i.e. beat) the 300ns
// access time of the fastest 2332/2364 ROMs.  (Slower ROMs supported 350ns and
// 450ns access times.)
//
// TBC what access times it achieves.  HW_REV_A appears to achieve between
// 420-480ns access times (too slow).
//
// This implementation achieves its aims by
// - running the STM32F103 at its fastest possible clock speed from HSI/2 using
//   the maximum PLL multiplier of 16, hence ~64MHz
// - using the HSI trimming function to increase the HSI by 15x40KHz (600KHz),
//   which in turn gives a final clock speed of ~68.8MHz
// - implementing the main loop in assembly (this module)
// - preloading as much as possible into registers before the main work loop,
//   so subsequent operations within the loop take as few instructions as
//   possible
//
// Subsequent versions of the hardware needed a few modifications to improve
// performance further:
// - CS2 (used by the 2332) needs to be connected to the same port as CS1, in
//   order to avoid 2 cycles reading it from a different port register, and 
//   an additional test cycle
// - CS3 (2316 if supported) needs to be connected to the same port as CS1 for
//   the same reason
// - all data pins connected to port C to be in either the lower or upper half
//   of the bank, to avoid both CRH and CRL manipulation, saving 1 cycle.
//
// HW_REV_B has all CS lines on port C (PC10-12).
//
// HW_REV_C has all data lines on port A (PA8-PA15), plus CS lines PC10-12.
//
// HW_REV_D may be necessary using an STM32F401 or STM32F411 to increase speed
// further.  In addition to faster clocks and better flash access times, these
// processors also support 5V on all GPIOs, so saving a cycle shifting the
// read address right by two to skip PB0/1 (and the requirement to avoid PB5).

#include "include.h"

#if !defined(TIMER_TEST) && !defined(TOGGLE_PA4)

// Log function which can be called from functions potentially run from RAM
#if defined(MAIN_LOOP_LOGGING)
ram_log_fn ROM_IMPL_LOG = do_log;
#else // !MAIN_LOOP_LOGGING
#define ROM_IMPL_LOG(X, ...)
#endif
#if defined(DEBUG_LOGGING)
ram_log_fn ROM_IMPL_DEBUG = do_log;
#else // !DEBUG_LOGGING
#define ROM_IMPL_DEBUG(X, ...)
#endif // DEBUG_LOGGING

// Pull in the RAM ROM image start/end locations from the linker
extern uint32_t _ram_rom_image_start[];
extern uint32_t _ram_rom_image_end[];

#if defined(STM32F1)

//
// Pre-calculated GPIO configurations
//

//
// A - CRH
//
#if defined(HW_REV_A)
#define GPIO_CRH_OUTPUT_A  0x00043333  // PA8-11 as output (0011) - also A12 input (CS1)
#define GPIO_CRH_INPUT_A   0x00044444  // PA8-11 as input (0100) - also A12 input (CS1)
#endif // HW_REV_A
#if defined(HW_REV_B)
#define GPIO_CRH_OUTPUT_A  0x00003333  // PA8-11 as output (0011)
#define GPIO_CRH_INPUT_A   0x00004444  // PA8-11 as input (0100)
#endif // HW_REV_B
#if defined(HW_REV_C)
#define GPIO_CRH_OUTPUT_A  0x33333333  // PA8-15 as output (0011)
#define GPIO_CRH_INPUT_A   0x44444444  // PA8-15 as input (0100)
#endif // HW_REV_C

//
// C - CRL
//
#if defined(HW_REV_A) || defined(HW_REV_B)
#define GPIO_CRL_OUTPUT_C  0x33000000  // PC6-7 as output (0011)
#define GPIO_CRL_INPUT_C   0x44000000  // PC6-7 as input (0100)
#endif // HW_REV_A/B
#if defined(HW_REV_C)
#define GPIO_CRL_OUTPUT_C  0x00000444  // PC0-2 as input (0011)
#define GPIO_CRL_INPUT_C   0x00000444  // PC0-2 as input (0100)
#endif // HW_REV_C

//
// C - CRH
//
#if defined(HW_REV_A)
#define GPIO_CRH_OUTPUT_C  0x00000033  // PC8-9 as output (0011)
#define GPIO_CRH_INPUT_C   0x00000044  // PC8-9 as input (0100)
#endif // HW_REV_A
#if defined(HW_REV_B)
#define GPIO_CRH_OUTPUT_C  0x00044433  // PC8-9 as output (0011) and PC10-12 inputs
#define GPIO_CRH_INPUT_C   0x00044444  // PC8-9 as input (0100) and PC10-12 inputs
#endif // HW_REV_B
#if defined(HW_REV_C)
#define GPIO_CRH_OUTPUT_C  0x00044400  // PC10-12 inputs outputs
#define GPIO_CRH_INPUT_C   0x00044400  // PC10-12 inputs
#endif // HW_REV_A/B/C

void __attribute__((section(".main_loop"), used)) main_loop(const sdrr_rom_info_t *rom)
{
    ROM_IMPL_LOG("Starting main loop");

#if defined(MAIN_LOOP_LOGGING)
    uint32_t byte;

    // When MAIN_LOOP_LOGGING is enabled, we iterate using a while loop and
    // log at the end of each iteration (from C).  This means we add extra
    // delay before restarting the wait for chip select.
    while (1) {
        ROM_IMPL_LOG("Waiting for CS");
#endif // MAIN_LOOP_LOGGING
    __asm volatile (
        // Load base addresses into registers (persistent through loop)
        "movw r4,  #:lower16:%[gpioa] \n"
        "movt r4,  #:upper16:%[gpioa] \n"
        "movw r5,  #:lower16:%[gpiob] \n"
        "movt r5,  #:upper16:%[gpiob] \n"
        "movw r6,  #:lower16:%[gpioc] \n"
        "movt r6,  #:upper16:%[gpioc] \n"
        "movw r7,  #:lower16:%[rom_base] \n"
        "movt r7,  #:upper16:%[rom_base] \n"

        // Pre-load GPIO configuration values (for performance)
        "movw r8,  #:lower16:%[gpio_crh_out_a] \n"
        "movt r8,  #:upper16:%[gpio_crh_out_a] \n"
        "movw r9,  #:lower16:%[gpio_crl_out_c] \n"
        "movt r9,  #:upper16:%[gpio_crl_out_c] \n"
        "movw r10, #:lower16:%[gpio_crh_out_c] \n"
        "movt r10, #:upper16:%[gpio_crh_out_c] \n"
        "movw r11, #:lower16:%[gpio_crh_in_a] \n"
        "movt r11, #:upper16:%[gpio_crh_in_a] \n"
        "movw r12, #:lower16:%[gpio_crl_in_c] \n"
        "movt r12, #:upper16:%[gpio_crl_in_c] \n"
        "movw r1,  #:lower16:%[gpio_crh_in_c] \n"
        "movt r1,  #:upper16:%[gpio_crh_in_c] \n"

#if 0
        // Required if we use BSRR
        "  mov  r0, #0 \n"
        "  strh  r0, [r4, %[gpio_odr]] \n"
        "  strh  r0, [r6, %[gpio_odr]] \n"
#endif

        "main_asm_loop: \n"

        // Chip select handling
#if defined(ROM_2364)
        // 2364

        // Check if ~CS1 has gone active. Ignore CS2 and CS3 - these are
        // address lines in this configuration

#if defined(HW_REV_A)
        // Rev A - CS1 is PA12 
        "  ldrh r0,  [r4, %[gpio_idr]] \n"
        "  tst  r0,  #(1 << 12) \n"
#elif defined(HW_REV_B) || defined(HW_REV_C)
        // Revs B & C - CS1 is PC10
        "  ldrh r0,  [r6, %[gpio_idr]] \n"
        "  tst  r0,  #(1 << 10) \n"
#endif // HW_REV_A/B/C

#if CS1_ACTIVE_LOW == 1
        "  bne  main_asm_loop \n"
#else // !CS1_ACTIVE_LOW
        "  beq  main_asm_loop \n"
#endif // CS1_ACTIVE_LOW

#elif defined(ROM_2332)
#if defined(HW_REV_A)
#error "2332 ROM not supported on rev A"
#endif // HW_REV_A
        // 2332 uses CS1 and CS3 (the latter called CS2 on the 2332).
        // CS1 is PC10 and CS3 is PC12.
        // We support 4 combinations - all but CS1 and CS3 active low require
        // an additional instruction (a cmp).
        "  ldrh r0, [r6, %[gpio_idr]] \n"
#if (CS1_ACTIVE_LOW == 1) && (CS3_ACTIVE_LOW == 1)
        // ~CS1, ~CS3
        "  tst  r0, #(0b101 << 10) \n"
#else // CS1_ACTIVE_LOW == 0 || CS3_ACTIVE_LOW == 0
        // All other CS1/CS3 combinations
        "  and  r0, r0, #(0b101 << 10) \n"
#if (CS1_ACTIVE_LOW == 1) && (CS3_ACTIVE_LOW == 0)
        // ~CS1, CS3 - CS1 must be low, CS3 must be high
        "  cmp  r0, #(0b100 << 10) \n"
#elif (CS1_ACTIVE_LOW == 0) && (CS3_ACTIVE_LOW == 1)
        // CS1, ~CS3 - CS1 must be high, CS3 must be low
        "  cmp  r0, #(0b001 << 10) \n"
#elif (CS1_ACTIVE_LOW == 0) && (CS3_ACTIVE_LOW == 0)
        // CS1, CS3 - Both must be high to be active
        "  cmp  r0, #(0b101 << 10) \n"
#else 
#error "Unreachable code"
#endif // Handle other CS1/CS3 combinations
        "  bne  main_asm_loop \n"
#endif // Handle all CS1/CS3 combinations

#else // ROM_2316

#if defined(HW_REV_A)
#error "2316 ROM not supported on rev A"
#endif // HW_REV_A
        // 2316 uses CS1, CS2 and CS3
        // CS1 is PC10, CS2 is PC11, CS3 is PC12
        // We support 4 combinations - all but CS1 and CS3 active low require
        // an additional instruction (a cmp).
        "  ldrh r0, [r6, %[gpio_idr]] \n"
#if (CS1_ACTIVE_LOW == 1) && (CS2_ACTIVE_LOW == 1) && (CS3_ACTIVE_LOW == 1)
        // ~CS1, ~CS2, ~CS3 - All must be low to be active
        "  tst  r0, #(0b111 << 10) \n"
#else
        // All other CS combinations.
        // In the cmp instruction 0bxyz - x refers to CS3, y to CS2, and z to
        // CS1.  We test against the active combination - so 0b100 for CS3
        // active high, CS2 and CS1 active low - for example.
        "  and  r0, r0, #(0b111 << 10) \n"
#if (CS1_ACTIVE_LOW == 1) && (CS2_ACTIVE_LOW == 1) && (CS3_ACTIVE_LOW == 0)
        // ~CS1, ~CS2, CS3 - CS1 and CS2 must be low, CS3 must be high
        "  cmp  r0, #(0b100 << 10) \n"
#elif (CS1_ACTIVE_LOW == 1) && (CS2_ACTIVE_LOW == 0) && (CS3_ACTIVE_LOW == 1)
        // ~CS1, CS2, ~CS3 - CS1 and CS3 must be low, CS2 must be high
        "  cmp  r0, #(0b010 << 10) \n"
#elif (CS1_ACTIVE_LOW == 1) && (CS2_ACTIVE_LOW == 0) && (CS3_ACTIVE_LOW == 0)
        // ~CS1, CS2, CS3 - CS1 must be low, CS2 and CS3 must be high
        "  cmp  r0, #(0b110 << 10) \n"
#elif (CS1_ACTIVE_LOW == 0) && (CS2_ACTIVE_LOW == 1) && (CS3_ACTIVE_LOW == 1)
        // CS1, ~CS2, ~CS3 - CS2 and CS3 must be low, CS1 must be high
        "  cmp  r0, #(0b001 << 10) \n"
#elif (CS1_ACTIVE_LOW == 0) && (CS2_ACTIVE_LOW == 1) && (CS3_ACTIVE_LOW == 0)
        // CS1, ~CS2, CS3 - CS2 must be low, CS1 and CS3 must be high
        "  cmp  r0, #(0b101 << 10) \n"
#elif (CS1_ACTIVE_LOW == 0) && (CS2_ACTIVE_LOW == 0) && (CS3_ACTIVE_LOW == 1)
        // CS1, CS2, ~CS3 - CS3 must be low, CS1 and CS2 must be high
        "  cmp  r0, #(0b011 << 10) \n"
#elif (CS1_ACTIVE_LOW == 0) && (CS2_ACTIVE_LOW == 0) && (CS3_ACTIVE_LOW == 0)
        // CS1, CS2, CS3 - All must be high to be active
        "  cmp  r0, #(0b111 << 10) \n"
#endif
        "  bne  main_asm_loop \n"
#endif

#endif // ROM_2364/2332/2316 CS active handling

        // Chip select line(s) has (have) been asserted - read address from
        // GPIOB
        "  ldrh r0, [r5, %[gpio_idr]] \n"

        // Shift right to get rid of PB0/1
        "  lsr  r0, r0, #2 \n"

        // Load byte from ROM using address as offset.  Remember that PB5
        // (which is bit 3 after the shift) will always be 0, the bits 4
        // onwards signify bits 3 onwards of the actual address.  PB5 is not
        // used because it isn't 5V tolerant.
        "  ldrb r3, [r7, r0] \n"

        // Potential future performance improvement - performing the operation
        // to set the data pins to outputs here would save 1 cycle, as loading
        // the ROM byte then using it immediately after suffers from a load-use
        // penalty.  However, it makes it harder to test the performance, as
        // we detect how long the output port takes to go low.

        // Extract and position bits for Port C (PC6-9)
        // - get lower 4 bits (7210)
        // - shift to positions 6-9
#if !defined(HW_REV_C)
        // Not required for HW_REV_C as all data pins on port A
        "  and  r2, r3, #0x0F \n"
        "  lsl  r2, r2, #6 \n"
#endif // !HW_REV_C

        // Extract and position bits for Port A (PA8-11)
        // - get upper 4 bits (3456)
        // - shift to positions 8-11
        // Apply it
#if !defined(HW_REV_C)
        "  and  r0, r3, #0xF0 \n"
        "  lsl  r0, r0, #4 \n"
#else // HW_REV_C
        // Shift left by 8 bits to apply to PA8-15.  No masking is required.
        "  lsl  r0, r3, #8 \n"
#endif // !HW_REV_C

        // Could use BSRR instead, if we reset ODR to 0 before hand

        // Write to port A ODR register
        "  strh  r0, [r4, %[gpio_odr]] \n"
#if !defined(HW_REV_C)
        // Not required for HW_REV_C as all data pins on port A
        "  strh  r2, [r6, %[gpio_odr]] \n"
#endif // !HW_REV_C

        // Configure output pins (using pre-loaded values)
        "  str  r8, [r4, %[gpio_crh]] \n"
#if !defined(HW_REV_C)
        // Not required for HW_REV_C as all data pins on port A
        "  str  r9, [r6, %[gpio_crl]] \n"
        "  str  r10, [r6, %[gpio_crh]] \n"
#endif // !HW_REV_C

        // Sub-loop: wait for chip select(s) to go inactive
        "wait_cs_high: \n"
#if defined(ROM_2364)
        // 2364

#if defined(HW_REV_A)
        // Rev A - CS1 is PA12
        "  ldrh r0, [r4, %[gpio_idr]] \n"
        "  tst  r0, #(1 << 12) \n"
#elif defined(HW_REV_B) || defined(HW_REV_C)
        // Revs B & C - CS1 is PC10
        "  ldrh r0, [r6, %[gpio_idr]] \n"
        "  tst  r0, #(1 << 10) \n"
#endif // HW_REV_A/B/C

#if CS1_ACTIVE_LOW == 1
        "  beq  wait_cs_high \n"
#else
        "  bne  wait_cs_high \n"
#endif

#elif defined(ROM_2332)
        // 2332 - CS1 is PC10 and CS3 is PC12
        "  ldrh r0, [r6, %[gpio_idr]] \n"
        "  and  r0, r0, #(0b101 << 10) \n"
        
#if (CS1_ACTIVE_LOW == 1) && (CS3_ACTIVE_LOW == 1)
        // No-op - zero flag set by and if both are still low 
#elif (CS1_ACTIVE_LOW == 1) && (CS3_ACTIVE_LOW == 0)
        "  cmp  r0, #(0b100 << 10) \n"
#elif (CS1_ACTIVE_LOW == 0) && (CS3_ACTIVE_LOW == 1)
        "  cmp  r0, #(0b001 << 10) \n"
#elif (CS1_ACTIVE_LOW == 0) && (CS3_ACTIVE_LOW == 0)
        "  cmp  r0, #(0b101 << 10) \n"
#endif
        "  beq  wait_cs_high \n"

#else // ROM_2316
        // 2316 - CS1 is PC10, CS2 is PC11, CS3 is PC12
        "  ldrh r0, [r6, %[gpio_idr]] \n"
        "  and  r0, r0, #(0b111 << 10) \n"
        
#if (CS1_ACTIVE_LOW == 1) && (CS2_ACTIVE_LOW == 1) && (CS3_ACTIVE_LOW == 1)
        // No-op - zero flag set by and if all are still low
#elif (CS1_ACTIVE_LOW == 1) && (CS2_ACTIVE_LOW == 1) && (CS3_ACTIVE_LOW == 0)
        "  cmp  r0, #(0b100 << 10) \n"
#elif (CS1_ACTIVE_LOW == 1) && (CS2_ACTIVE_LOW == 0) && (CS3_ACTIVE_LOW == 1)
        "  cmp  r0, #(0b010 << 10) \n"
#elif (CS1_ACTIVE_LOW == 1) && (CS2_ACTIVE_LOW == 0) && (CS3_ACTIVE_LOW == 0)
        "  cmp  r0, #(0b110 << 10) \n"
#elif (CS1_ACTIVE_LOW == 0) && (CS2_ACTIVE_LOW == 1) && (CS3_ACTIVE_LOW == 1)
        "  cmp  r0, #(0b001 << 10) \n"
#elif (CS1_ACTIVE_LOW == 0) && (CS2_ACTIVE_LOW == 1) && (CS3_ACTIVE_LOW == 0)
        "  cmp  r0, #(0b101 << 10) \n"
#elif (CS1_ACTIVE_LOW == 0) && (CS2_ACTIVE_LOW == 0) && (CS3_ACTIVE_LOW == 1)
        "  cmp  r0, #(0b011 << 10) \n"
#elif (CS1_ACTIVE_LOW == 0) && (CS2_ACTIVE_LOW == 0) && (CS3_ACTIVE_LOW == 0)
        "  cmp  r0, #(0b111 << 10) \n"
#endif // Handle all CS1/CS2/CS3 combinations
        "  beq  wait_cs_high \n"

#endif // ROM_2364/2332/2316 CS inactive handling

        // Chip selects went inactive - configure data pins back to inputs
        "  str  r11, [r4, %[gpio_crh]] \n"
#if !defined(HW_REV_C)
        // Not required for HW_REV_C as all data pins on port A
        "  str  r12, [r6, %[gpio_crl]] \n"
        "  str  r1, [r6, %[gpio_crh]] \n"
#endif // !HW_REV_C

        // Potentially use BRR to set ODR back to 0

#if !defined(MAIN_LOOP_LOGGING)
        // Restart main loop
        "  b main_asm_loop \n"
#else // MAIN_LOOP_LOGGING
        // Output data byte (which we read and signalled via the data pins)
        // from this assembly block, so we can log it
        "  mov %0, r3 \n"
#endif // !MAIN_LOOP_LOGGING
#if defined(MAIN_LOOP_LOGGING)
        : "=r" (byte) // Output the byte
#else // !MAIN_LOOP_LOGGING
        :
#endif // MAIN_LOOP_LOGGING
        : [gpioa]           "i" (GPIOA_BASE),
            [gpiob]           "i" (GPIOB_BASE),
            [gpioc]           "i" (GPIOC_BASE),
            [rom_base]        "i" (&_ram_rom_image_start),
            [gpio_idr]        "i" (GPIO_IDR_OFFSET),
            [gpio_odr]        "i" (GPIO_ODR_OFFSET),
            [gpio_crl]        "i" (GPIO_CRL_OFFSET),
            [gpio_crh]        "i" (GPIO_CRH_OFFSET),
            [gpio_crh_out_a]  "i" (GPIO_CRH_OUTPUT_A),
            [gpio_crh_in_a]   "i" (GPIO_CRH_INPUT_A),
            [gpio_crl_out_c]  "i" (GPIO_CRL_OUTPUT_C),
            [gpio_crl_in_c]   "i" (GPIO_CRL_INPUT_C),
            [gpio_crh_out_c]  "i" (GPIO_CRH_OUTPUT_C),
            [gpio_crh_in_c]   "i" (GPIO_CRH_INPUT_C)
        : "r0", "r1", "r2", "r3", "r4", "r5", "r6", "r7", "r8", "r9", "r10", "r11", "r12", "cc", "memory"
    );
#if defined(MAIN_LOOP_LOGGING)
        ROM_IMPL_LOG("Byte served: 0x%02X", byte);
    }
#endif // MAIN_LOOP_LOGGING
}

#elif defined(STM32F4)

//
// Defines for assembly routine
//

// Data output enable mask for PA0-7.
//
// Also, ensure PA13/PA14 are in alternate function mode, so SWD programming
// (and logging if MAIN_LOOP_LOGGING enabled) work.
//
// In the MCO case, ensure PA8 set to alternate function.
#if !defined(MCO)
#define DATA_OUTPUT_MASK 0x28005555
#define DATA_INPUT_MASK 0x28000000
#else // MCO
#define DATA_OUTPUT_MASK 0x28025555
#define DATA_INPUT_MASK 0x28020000
#endif // MCO

// CPU registers - we prefer low registers (r0-r7) to high registers as
// instructions using the low registers tend to be 16-bit vs 32-bit, which
// means they are fetched from flash (or RAM) faster. 
#define R_ADDR_CS       "r0"
#define R_DATA          "r1"
#define R_ROM_TABLE     "r2"
#define R_DATA_OUT_MASK "r3"
#define R_DATA_IN_MASK  "r6"
#define R_GPIO_DATA_MODER   "r7"
#define R_GPIO_DATA_ODR     "r5"
#define R_GPIO_ADDR_CS_IDR  "r4"
#define R_CS_INVERT_MASK    "r8"
#define R_CS_CHECK_MASK     "r9"
#define R_CS_TEST           "r10"

// Pins
#if defined(HW_REV_D) || defined(HW_REV_E)
#define PIN_CS              "10"
#define PIN_CS_INT           10
#define PIN_CS2             "12"
#define PIN_CS2_INT          12
#define PIN_CS3             "9"
#define PIN_CS3_INT          9
#else // !(HW_REV_D || HW_REV_E)
#error "Only HW_REV_D and HW_REV_E are supported for STM32F4"
#endif // HW_REV_D || HW_REV_E

// Assembly code macros

// Loads the address/CS lines into R_ADDR_CS
#define LOAD_ADDR_CS    "ldr " R_ADDR_CS ", [" R_GPIO_ADDR_CS_IDR "]\n"

// Tests whether the CS line is active - zero flag set if so (low)
#define TEST_CS         "eor " R_CS_TEST ", " R_ADDR_CS ", " R_CS_INVERT_MASK "\n" \
                        "tst " R_CS_TEST ", " R_CS_CHECK_MASK "\n"

// Loads the data byte from the ROM table into R_DATA, based on the address in
// R_ADDR_CS
#define LOAD_FROM_RAM   "ldrb " R_DATA ", [" R_ROM_TABLE ", " R_ADDR_CS "]\n"

// Stores the data byte from the CPU register to the data lines
#define STORE_TO_DATA   "strb " R_DATA ", [" R_GPIO_DATA_ODR "]\n"

// Sets the data lines as outputs
#define SET_DATA_OUT    "str " R_DATA_OUT_MASK ", [" R_GPIO_DATA_MODER "]\n"

// Sets the data lines as inputs
#define SET_DATA_IN     "str " R_DATA_IN_MASK ", [" R_GPIO_DATA_MODER "]\n"

// Branches if zero flag set
#define BEQ(X)          "beq " #X "\n"

// Branches if zero flag not set
#define BNE(X)          "bne " #X "\n"

// Branches unconditionally
#define BRANCH(X)       "b " #X "\n"

// Labels
#define LOOP            loop
#define CS_ACTIVE       cs_active
#define CS_INACTIVE     cs_inactive
#define WAIT_INACTIVE   wait_inactive
#if defined(MAIN_LOOP_LOGGING)
#define END_LOOP        end_loop
#endif // MAIN_LOOP_LOGGING
#define LABEL(X)        #X ": \n"

void __attribute__((section(".main_loop"), used)) main_loop(const sdrr_rom_info_t *rom) {
    uint32_t cs_invert_mask = 0;
    uint32_t cs_check_mask;

    ROM_IMPL_DEBUG("Serve ROM: %s", rom->filename);

    switch (rom->rom_type) {
        case ROM_TYPE_2316:
            ROM_IMPL_DEBUG("ROM type: 2316");
            cs_check_mask = (1 << PIN_CS_INT) | (1 << PIN_CS2_INT) | (1 << PIN_CS3_INT);
            if (rom->cs1_state == CS_ACTIVE_LOW) {
                ROM_IMPL_DEBUG("CS1 active low");
            } else {
                ROM_IMPL_DEBUG("CS1 active high");
                cs_invert_mask |= (1 << PIN_CS_INT);
            }
            if (rom->cs2_state == CS_ACTIVE_LOW) {
                ROM_IMPL_DEBUG("CS2 active low");
            } else {
                ROM_IMPL_DEBUG("CS2 active high");
                cs_invert_mask |= (1 << PIN_CS2_INT);
            }
            if (rom->cs3_state == CS_ACTIVE_LOW) {
                ROM_IMPL_DEBUG("CS3 active low");
            } else {
                ROM_IMPL_DEBUG("CS3 active high");
                cs_invert_mask |= (1 << PIN_CS3_INT);
            }
            break;

        case ROM_TYPE_2332:
            // 2332 CS2 actually uses the same pin as 2316 CS3
            // In roms.c/h it is called cs2_state
            // Here is is called CS3 
            ROM_IMPL_DEBUG("ROM type: 2332");
            cs_check_mask = (1 << PIN_CS_INT) | (1 << PIN_CS3_INT);
            if (rom->cs1_state == CS_ACTIVE_LOW) {
                ROM_IMPL_DEBUG("CS1 active low");
            } else {
                ROM_IMPL_DEBUG("CS1 active high");
                cs_invert_mask |= (1 << PIN_CS_INT);
            }
            if (rom->cs2_state == CS_ACTIVE_LOW) {
                ROM_IMPL_DEBUG("CS2(3) active low");
            } else {
                ROM_IMPL_DEBUG("CS2(3) active high");
                cs_invert_mask |= (1 << PIN_CS3_INT);
            }
            break;

        default:
            ROM_IMPL_LOG("Unsupported ROM type: %d", rom->rom_type);
            __attribute__((fallthrough));
        case ROM_TYPE_2364:
            ROM_IMPL_DEBUG("ROM type: 2364");
            cs_check_mask = (1 << PIN_CS_INT);
            if (rom->cs1_state == CS_ACTIVE_LOW) {
                ROM_IMPL_DEBUG("CS1 active low");
            } else {
                ROM_IMPL_DEBUG("CS1 active high");
                cs_invert_mask |= 1 << PIN_CS_INT;
            }
            break;
    }

    // Enable GPIO clocks for the ports with address and data lines
    RCC_AHB1ENR |= (RCC_AHB1ENR_GPIOAEN | RCC_AHB1ENR_GPIOCEN);
    
    // Configure PA0-7 as inputs initially (00 in MODER), no pull-up/down
    // Also PC10-12 are duplicate CS lines so set as inputs no PU/PD.
    GPIOA_MODER &= ~0x00FCFFFF; // Clear bits 0-15 (PA0-7, 10-12 as inputs)
    GPIOA_PUPDR &= ~0x00FCFFFF; // Clear pull-up/down for PA0-7, 10-12)
    GPIOA_OSPEEDR |= 0xFFFF;    // Set PA0-7 speed to "very high"

    // Port C for address and CS lines - set all pins as inputs
    // Rev D - CS = 10, for testing purposes, not rev D, CS = 13 
    GPIOC_MODER = 0;  // Set all pins as inputs
    GPIOC_PUPDR |= 0xA0000000;  // Set pull-downs on PC14/15 only, so RAM
                                // lookup only takes 16KB (due to CS line(s)
                                // in the middle of address lines), not 64KB
                                // if we had to deal with PC14/15 possibly
                                // being high.

    register uint32_t cs_invert_mask_reg asm(R_CS_INVERT_MASK) = cs_invert_mask;
    register uint32_t cs_check_mask_reg asm(R_CS_CHECK_MASK) = cs_check_mask;

#if defined(PRELOAD_TO_RAM)
    register uint32_t rom_table asm(R_ROM_TABLE) = (uint32_t)&_ram_rom_image_start;
#else
    register uint32_t rom_table asm(R_ROM_TABLE) = (uint32_t)&(rom->data[0]);
#endif // PRELOAD_TO_RAM

#if defined(STATUS_LED)
    setup_status_led();
#endif // STATUS_LED

#if defined(MAIN_LOOP_LOGGING)
    uint32_t byte;
    uint32_t addr_cs;
    while (1) {
        ROM_IMPL_LOG("Waiting for CS to go active");
#if defined(STATUS_LED)
        GPIOB_BSRR = (1 << (15 + 16)); // LED on (PB15 low)
#endif // HW_REV_E
#endif // MAIN_LOOP_LOGGING
    // The targets (from the MOS 2364 data sheet Feburary 1980) are:
    // - tCO - set data line as outputs after CS activates - 200ns
    // - tDF - set data line as inputs after CS deactivates - 175ns
    // - tOH - data lines remain valid after address lines change - 40ns
    // - tACC - maximum time from address to data valid - 450ns
    //
    // tACC and tCO are the most important - together they mean:
    // - we have 450ns from the address lines being set to there being valid
    //   data on the data lines (and therefore them being outputs)
    // - we also have 200ns from CS activating to the data lines being set as
    //   outputs and having valid data on them
    //
    // They are not cumulative - that is, we cannot assume address lines will
    // be valid for tACC-tCO = 250ns before CS activates.  However, we can rely
    // on both timings - that is that we will get at least 450ns from address
    // lines being set AND 200ns from CS activating.
    //
    // Together these timings mean we have around twice as long to get the data
    // lines loaded with the byte indicated by the address lines as we do to
    // get the data lines set as outputs (450ns vs 200ns).  Hence we need to be
    // checking the cs line around twice as often as we are checking the
    // address lines.  This is reflected in the code below.
    //
    // They also mean we have to keep checking the address lines and updating
    // the data lines while chip select is active. 
    //
    // We also have to get the data lines back to inputs quickly enough after
    // CS deactivates - this is tDF, which is 175ns.  This is actually slightly
    // tighter than tCO, but the processing between both CS inactive and active
    // cases are very similar we are looking at a worse case of around 150ns
    // with the code below on a 100MHz STM32F411.
    //
    // Finally, we have to make sure we don't change the data lines until at
    // least tOH (40ns) after the address lines change.  This is fine, because
    // loading the byte from RAM and applying it to the data lines will always
    // take longer than this.
    //
    // Macros are used to make the code below more readable. 
    __asm volatile (
        // Load GPIO addresses and constants into registers before we start
        // the main loop
        "movw   " R_DATA_OUT_MASK ",    #:lower16:%[data_output_mask]   \n"
        "movt   " R_DATA_OUT_MASK ",    #:upper16:%[data_output_mask]   \n"
        "movw   " R_GPIO_ADDR_CS_IDR ", #:lower16:%[gpioc_idr]          \n"
        "movt   " R_GPIO_ADDR_CS_IDR ", #:upper16:%[gpioc_idr]          \n"
        "movw   " R_GPIO_DATA_ODR ",    #:lower16:%[gpioa_odr]          \n"
        "movt   " R_GPIO_DATA_ODR ",    #:upper16:%[gpioa_odr]          \n"
        "movw   " R_DATA_IN_MASK ",     #:lower16:%[data_input_mask]    \n"
        "movt   " R_DATA_IN_MASK ",     #:upper16:%[data_input_mask]    \n"
        "movw   " R_GPIO_DATA_MODER ",  #:lower16:%[gpioa_moder]        \n"
        "movt   " R_GPIO_DATA_MODER ",  #:upper16:%[gpioa_moder]        \n"

#if 0
        // Seems unecessary

        // Don't start processing until CS is inactive
        LABEL(START)
        LOAD_ADDR_CS
        TEST_CS
        BEQ(START)
#endif // 0
        BRANCH(LOOP)

        // Chip select went active - immediately set data lines as outputs
    LABEL(CS_ACTIVE)
        SET_DATA_OUT

        // By definition we just loaded and tested the address/CS lines, so we
        // have a valid address - so let's load the byte from RAM and apply it,
        // in amongst testing whether CS gone inactive again.
    LABEL(CS_ACTIVE_DATA_ACTIVE)
        LOAD_FROM_RAM
        LOAD_ADDR_CS
        TEST_CS
        BNE(CS_INACTIVE_BYTE)
    LABEL(CS_ACTIVE_DATA_ACTIVE_BYTE)
        STORE_TO_DATA
        LOAD_ADDR_CS
        TEST_CS
        BNE(CS_INACTIVE_NO_BYTE)
        LOAD_FROM_RAM
        LOAD_ADDR_CS
        TEST_CS
        BEQ(CS_ACTIVE_DATA_ACTIVE_BYTE)
        // Fall through to CS_INACTIVE_BYTE if NE

#if 0
        // Wait for CS to go inactive, and also keep reading address lines
        // and updating data as required
        LABEL(WAIT_INACTIVE)
        LOAD_ADDR_CS
        TEST_CS
        BNE(CS_INACTIVE_NO_BYTE)
        LOAD_FROM_RAM
        LOAD_ADDR_CS
        TEST_CS
        BNE(CS_INACTIVE_BYTE)
        STORE_TO_DATA
        BRANCH(WAIT_INACTIVE)
#endif

        // CS went inactive.  We need to set the data lines as inputs, but
        // also want to update the data using the address lines we have in
        // our hands.
    LABEL(CS_INACTIVE_BYTE)
        SET_DATA_IN
        STORE_TO_DATA
#if defined(MAIN_LOOP_LOGGING)
        BRANCH(END_LOOP)
#endif // MAIN_LOOP_LOGGING

        // Start of main processing loop.  Load the data byte which constantly
        // checking if CS has gone active
    LABEL(LOOP)
        LOAD_ADDR_CS
        TEST_CS
        BEQ(CS_ACTIVE)
        LOAD_FROM_RAM
        LOAD_ADDR_CS
        STORE_TO_DATA
        TEST_CS
        BEQ(CS_ACTIVE)

        // Start again
        BRANCH(LOOP)

        // CS went inactive, but we don't have a new byte to load.
    LABEL(CS_INACTIVE_NO_BYTE)
        SET_DATA_IN
#if defined(MAIN_LOOP_LOGGING)
        BRANCH(END_LOOP)
#endif // MAIN_LOOP_LOGGING

        // Copy of main processing loop - to avoid a branch in this or the
        // CS_INACTIVE_BYTE case
        LOAD_ADDR_CS
        TEST_CS
        BEQ(CS_ACTIVE)
        LOAD_FROM_RAM
        LOAD_ADDR_CS
        STORE_TO_DATA
        TEST_CS
        BEQ(CS_ACTIVE)

        // Start again - and might as well branch to the start of the first
        // loop as opposed to this copy.
        BRANCH(LOOP)

#if defined(MAIN_LOOP_LOGGING)
    LABEL(END_LOOP)
        "mov %0, " R_ADDR_CS "\n"
        "mov %1, " R_DATA "\n"
#endif // MAIN_LOOP_LOGGING

#if defined(MAIN_LOOP_LOGGING)
        : "=r" (addr_cs),
          "=r" (byte)
#else // !MAIN_LOOP_LOGGING
        :
#endif // MAIN_LOOP_LOGGING
        : [gpioc_idr]        "i" (VAL_GPIOC_IDR),
          [gpioa_odr]        "i" (VAL_GPIOA_ODR),
          [gpioa_moder]      "i" (VAL_GPIOA_MODER),
          [data_output_mask] "i" (DATA_OUTPUT_MASK),
          [data_input_mask]  "i" (DATA_INPUT_MASK),
          "r" (rom_table),
          "r" (cs_invert_mask_reg),
          "r" (cs_check_mask_reg)
        : "r0", "r1", "r3", "r4", "r5", "r6", "r7", "r10", "cc", "memory"
    );
#if defined(MAIN_LOOP_LOGGING)
#if defined(STATUS_LED)
        GPIOB_BSRR = (1 << 15);        // LED off (PB15 high)
#endif // HW_REV_E
        ROM_IMPL_LOG("Address/CS: 0x%08X Byte: 0x%08X", addr_cs, byte);
    }
#endif // MAIN_LOOP_LOGGING
}

#endif // STM32F1/4

// Get the index of the selected ROM by reading the select jumpers
//
// Returns the index
uint8_t get_rom_index(void) {
    uint8_t rom_sel, rom_index;

    // Read image selection pins - these are set up in gpio_init().
#if defined(STM32F1)
    // Sel pins are PC0-2 on HW revs A-C
    rom_sel = GPIOC_IDR & 0x07;  // Get bits 0-2
    RCC_APB2ENR &= ~(1 << 4);  // Disable GPIOC clock
#elif defined(STM32F4)
    // Read the ROM image selection bits PB0 (LSB) to PB2, and PB7 (MSB).
    rom_sel = GPIOB_IDR;
    rom_sel = (rom_sel & 0x07) | ((rom_sel & 0x80) >> 4);  // Get bits 0-2 and 7, shift PB7 to bit 3
    RCC_AHB1ENR &= ~RCC_AHB1ENR_GPIOBEN;  // Disable GPIOB clock
#endif // STM32F1/4

    // Calculate the ROM image index based on the selection bits and number of
    // images installed in this firmware.  For example, if image 4 was selected
    // but there are only 3 images, it will select image 1.
    rom_index = rom_sel % SDRR_NUM_IMAGES;

    LOG("ROM sel/index %d/%d", rom_sel, rom_index);

    return rom_index;
}

void preload_rom_image(const sdrr_rom_info_t *rom) {
    uint32_t *rom_src, *rom_dst;
    uint16_t rom_size;

    // Find the start of this ROM image in the flash memory
    rom_size = rom->size;
    rom_src = (uint32_t *)(rom->data);
    rom_dst = _ram_rom_image_start;

#if defined(BOOT_LOGGING)
    DEBUG("ROM filename: %s", rom->filename);
#endif // BOOT_LOGGING
    switch (rom->rom_type) {
        case ROM_TYPE_2364:
            DEBUG("%s 2364", rom_type);
            break;
        case ROM_TYPE_2332:
            DEBUG("%s 2332", rom_type);
            break;
        case ROM_TYPE_2316:
            DEBUG("%s 2316", rom_type);
            break;
        default:
            DEBUG("%s %d %s", rom_type, rom->rom_type, unknown);
    }
    DEBUG("ROM size %d bytes", rom_size);

    // Copy the ROM image to RAM
#if defined(STM32F1)
    // There is a complication.  We support 2KB, 4KB and 8KB ROM images. 
    // These are addressed using 13 bits (A0-A12).  However, due to PB5
    // (which should be our bit 3) not being 5V tolerant, we skip it and
    // pull it low.  Hence bit 3 will always read 0 and we have 14 bits to
    // address 8KB.  This means we need to copy the 8KB of data to
    // XXXXXXXXX0XXX not to XXXXXXXXXXXXX.
    //
    // sdrr-gen has already maps the other address lines to their
    // appropriate places, and also mapped the data bits to the correct
    // places to be applied easily to the GPIO ports.
    //
    // This routine should complete for an 8KB image in a few ms, and is the
    // bulk of the work done at startup.
    uint32_t src_ptr = (uint32_t)rom_src;
    uint32_t dst_ptr = (uint32_t)rom_dst;
    size_t ii;
    
    for (ii = 0; ii < rom_size; ii++) {
        // Read a byte from source
        uint8_t byte = *(uint8_t *)(src_ptr + ii);

        // Calculate destination address with bit 3 always 0 and higher bits
        // shifted
        size_t byte_offset = ii;
        size_t dst_offset = (byte_offset & 0x7) | ((byte_offset & ~0x7) << 1);
        
        // Write to destination
        *(uint8_t *)(dst_ptr + dst_offset) = byte;
    }
#elif defined(STM32F4)
    // For the STM32F4 family the ROM image is always 16KB and all address
    // and data line mappings have been done by sdrr-gen before building
    // the mapped image into the firmware.  Hence we can just copy the
    // image.
    memcpy(rom_dst, rom_src, rom_size);
#endif // STM32F1/4

    LOG("ROM %s preloaded to RAM 0x%08X", rom->filename, (uint32_t)_ram_rom_image_start);
}

#endif // !TIMER_TEST/TOGGLE_PA4
