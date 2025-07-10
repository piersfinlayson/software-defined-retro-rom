// 2332/2364 ROM implementation.

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

// This file contains the STM32 main loop emulating the 2316/2332/2364 ROM.
// It is highly optimised for speed and aims to exceed (i.e. beat) the 300ns
// access time of the fastest 2332/2364 ROMs.  (Slower ROMs supported 350ns and
// 450ns access times.)
//
// HW_REV_D with the STM32F411 at 100MHz is fast enough to replace kernal
// basic, and character ROMs.  The character ROM is a 350ns ROM.
//
// This implementation achieves its aims by
// - running the STM32F4 at its fastest possible clock speed from the PLL
// - implementing the main loop in assembly (this module)
// - assigning data/CS pins to the same port, starting at pin 0, and data pins
//   contiguously from pin 0 on the same port
// - remapping "mangling" the addresses of the data bytes of the ROM images
//   stored on flash, and the data bytes themselves, to map the hardware pin
//   layout
// - preloading as much as possible into registers before the main work loop,
//   so subsequent operations within the loop take as few instructions as
//   possible
//
// See the various technical documents in [/docs](/docs) for more information.
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

//
// Defines for assembly routine
//

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
#define ASM_INPUTS \
    "r" (rom_table), \
    "r" (cs_invert_mask_reg), \
    "r" (cs_check_mask_reg), \
    "r" (gpioc_idr), \
    "r" (gpioa_odr), \
    "r" (gpioa_moder), \
    "r" (data_output_mask), \
    "r" (data_input_mask)
#define ASM_CLOBBERS R_ADDR_CS, R_DATA, R_CS_TEST, "cc", "memory"

// Assembly code macros

// Loads the address/CS lines into R_ADDR_CS
#define LOAD_ADDR_CS    "ldrh " R_ADDR_CS ", [" R_GPIO_ADDR_CS_IDR "]\n"

// Tests whether the CS line is active - zero flag set if so (low)
#define TEST_CS         "eor " R_CS_TEST ", " R_ADDR_CS ", " R_CS_INVERT_MASK "\n" \
                        "tst " R_CS_TEST ", " R_CS_CHECK_MASK "\n"

// Tests where any of the CS lines are active - zero flash _not set_ if so.
// BIC is bit clear - essentially destination = source & ~mask.  Note we need
// BICS - where S indicates the N/Z flags should be updated, which we use
#define TEST_CS_ANY     "eor " R_CS_TEST ", " R_ADDR_CS ", " R_CS_INVERT_MASK "\n" \
                        "bics " R_CS_TEST ", " R_CS_CHECK_MASK ", " R_CS_TEST "\n"

// Loads the data byte from the ROM table into R_DATA, based on the address in
// R_ADDR_CS
#define LOAD_FROM_RAM   "ldrb " R_DATA ", [" R_ROM_TABLE ", " R_ADDR_CS "]\n"

// Stores the data byte from the CPU register to the data lines
#define STORE_TO_DATA   "strb " R_DATA ", [" R_GPIO_DATA_ODR "]\n"

// Sets the data lines as outputs
#define SET_DATA_OUT    "strh " R_DATA_OUT_MASK ", [" R_GPIO_DATA_MODER "]\n"

// Sets the data lines as inputs
#define SET_DATA_IN     "strh " R_DATA_IN_MASK ", [" R_GPIO_DATA_MODER "]\n"

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

void __attribute__((section(".main_loop"), used)) main_loop(const sdrr_rom_set_t *set) {
    uint32_t cs_invert_mask = 0;
    uint32_t cs_check_mask;

    const sdrr_rom_info_t *rom = set->roms[0];

    // Set up serve mode
    sdrr_serve_t serve_mode = set->serve;

    // Warn if serve mode is incorrectly set for multiple ROM images
    if ((set->rom_count > 1) && (serve_mode != SERVE_ADDR_ON_ANY_CS)) {
        ROM_IMPL_LOG("!!! Mutliple ROM images, but serve mode is incorrectly set - rectifying");
        serve_mode = SERVE_ADDR_ON_ANY_CS;
    } else if ((set->rom_count == 1) && (serve_mode == SERVE_ADDR_ON_ANY_CS)) {
        ROM_IMPL_LOG("!!! Single ROM image, but serve mode is incorrectly set - setting to default");
        serve_mode = SERVE_TWO_CS_ONE_ADDR;
    }

    ROM_IMPL_DEBUG("Serve ROM: %s via mode: %d", rom->filename, serve_mode);

    // Set up CS pins.  These are the same values for:
    // - HW_REV_24_D
    // - HW_REV_24_E
    // - HW_REV_24_F 
    //
    // HW_REV_28_A requires different logic and hasn't been implemented yet
    //
    // We could (should?) read the pin number from the rom_set.
    //
    // Note that X1 and X2 are hard-coded into the pull-down code below.
    uint8_t pin_cs = 10;
    uint8_t pin_cs2 = 12;
    uint8_t pin_cs3 = 9;
    uint8_t pin_x1 = 14;
    uint8_t pin_x2 = 15;

    if (serve_mode == SERVE_ADDR_ON_ANY_CS)
    {
        if (set->rom_count == 2)
        {
            cs_check_mask = (1 << pin_cs) | (1 << pin_x1);
        } else if (set->rom_count == 3) {
            cs_check_mask = (1 << pin_cs) | (1 << pin_x1) | (1 << pin_x2);
        } else {
            ROM_IMPL_LOG("!!! Unsupported ROM count: %d", set->rom_count);
            cs_check_mask = (1 << pin_cs); // Default to CS1 only
        }
        if (set->multi_rom_cs1_state == CS_ACTIVE_HIGH) {
            cs_invert_mask = cs_check_mask;
        }
    } else {
        switch (rom->rom_type) {
            case ROM_TYPE_2316:
                ROM_IMPL_DEBUG("ROM type: 2316");
                cs_check_mask = (1 << pin_cs) | (1 << pin_cs2) | (1 << pin_cs3);
                if (rom->cs1_state == CS_ACTIVE_LOW) {
                    ROM_IMPL_DEBUG("CS1 active low");
                } else {
                    ROM_IMPL_DEBUG("CS1 active high");
                    cs_invert_mask |= (1 << pin_cs);
                }
                if (rom->cs2_state == CS_ACTIVE_LOW) {
                    ROM_IMPL_DEBUG("CS2 active low");
                } else {
                    ROM_IMPL_DEBUG("CS2 active high");
                    cs_invert_mask |= (1 << pin_cs2);
                }
                if (rom->cs3_state == CS_ACTIVE_LOW) {
                    ROM_IMPL_DEBUG("CS3 active low");
                } else {
                    ROM_IMPL_DEBUG("CS3 active high");
                    cs_invert_mask |= (1 << pin_cs3);
                }
                break;

            case ROM_TYPE_2332:
                // 2332 CS2 actually uses the same pin as 2316 CS3
                // In roms.c/h it is called cs2_state
                // Here is is called CS3 
                ROM_IMPL_DEBUG("ROM type: 2332");
                cs_check_mask = (1 << pin_cs) | (1 << pin_cs3);
                if (rom->cs1_state == CS_ACTIVE_LOW) {
                    ROM_IMPL_DEBUG("CS1 active low");
                } else {
                    ROM_IMPL_DEBUG("CS1 active high");
                    cs_invert_mask |= (1 << pin_cs);
                }
                if (rom->cs2_state == CS_ACTIVE_LOW) {
                    ROM_IMPL_DEBUG("CS2(3) active low");
                } else {
                    ROM_IMPL_DEBUG("CS2(3) active high");
                    cs_invert_mask |= (1 << pin_cs3);
                }
                break;

            default:
                ROM_IMPL_LOG("Unsupported ROM type: %d", rom->rom_type);
                __attribute__((fallthrough));
            case ROM_TYPE_2364:
                ROM_IMPL_DEBUG("ROM type: 2364");
                cs_check_mask = (1 << pin_cs);
                if (rom->cs1_state == CS_ACTIVE_LOW) {
                    ROM_IMPL_DEBUG("CS1 active low");
                } else {
                    ROM_IMPL_DEBUG("CS1 active high");
                    cs_invert_mask |= 1 << pin_cs;
                }
                break;
        }
    }

    // Set up the GPIOs

    // Enable GPIO clocks for the ports with address and data lines
    RCC_AHB1ENR |= (RCC_AHB1ENR_GPIOAEN | RCC_AHB1ENR_GPIOCEN);
    
    // Configure PA0-7 as inputs initially (00 in MODER), no pull-up/down
    // Also PC10-12 are duplicate CS lines so set as inputs no PU/PD.
    GPIOA_MODER &= ~0x00FCFFFF; // Clear bits 0-15 (PA0-7, 10-12 as inputs)
    GPIOA_PUPDR &= ~0x00FCFFFF; // Clear pull-up/down for PA0-7, 10-12)
    GPIOA_OSPEEDR &= ~0xFFFF;   // Clear output speed for PA0-7
    GPIOA_OSPEEDR |= 0xAAAA;    // Set PA0-7 speed to "fast", not "high" to
                                // ensure V(OL) is max 0.4V

    // Port C for address and CS lines - set all pins as inputs
    // Rev D - CS = 10, for testing purposes, not rev D, CS = 13 
    GPIOC_MODER = 0;  // Set all pins as inputs
    if (set->rom_count == 1) {
        // Set pull-downs on PC14/15 only, so RAM lookup only takes 16KB
        GPIOC_PUPDR = 0xA0000000;
    }
    else {
        // Hardware revision F has PC14 and PC15 connected to pins X1/X2 on the
        // PCB, so up to 2 extra ROM chip select lines can be terminated on SDRR.
        if (set->rom_count == 2) {
            // Set pull-down on PC15, as PC14 used to select the second ROM image
            GPIOC_PUPDR = 0x80000000;
        } else if (set->rom_count == 3) {
            // No pull-downs - PC14/PC15 used to select second and third ROM images
            GPIOC_PUPDR = 0;
        } else {
            // Handle gracefully by assuming one ROM image 
            ROM_IMPL_LOG("!!! Unsupported ROM count: %d", set->rom_count);
            GPIOC_PUPDR = 0xA0000000;
        }
    }

#if 0
    // Hack in fixed values
    GPIOC_PUPDR = 0xA0000000;
    cs_check_mask = (1 << pin_cs);
    cs_invert_mask = 0;
    serve_mode = SERVE_TWO_CS_ONE_ADDR;
#endif

    // Data output enable mask for port A.  Leave SWD enabled.
    register uint32_t data_output_mask asm(R_DATA_OUT_MASK);
    register uint32_t data_input_mask asm(R_DATA_IN_MASK);
    if (sdrr_info.mco_enabled) {
        // PA8 is AF, PA0-7 are inputs
        data_output_mask = 0x28025555;
        data_input_mask = 0x28020000;
    } else {
        // PA0-7 are inputs
        data_output_mask = 0x28005555;
        data_input_mask = 0x28000000;
    }

    // Preload registers with their values
    register uint32_t cs_invert_mask_reg asm(R_CS_INVERT_MASK) = cs_invert_mask;
    register uint32_t cs_check_mask_reg asm(R_CS_CHECK_MASK) = cs_check_mask;
    register uint32_t gpioc_idr asm(R_GPIO_ADDR_CS_IDR) = VAL_GPIOC_IDR; 
    register uint32_t gpioa_odr asm(R_GPIO_DATA_ODR) = VAL_GPIOA_ODR;
    register uint32_t gpioa_moder asm(R_GPIO_DATA_MODER) = VAL_GPIOA_MODER;

    register uint32_t rom_table asm(R_ROM_TABLE);
    if (sdrr_info.preload_image_to_ram) {
        rom_table = (uint32_t)&_ram_rom_image_start;
    } else {
        rom_table = (uint32_t)&(set->data[0]);
    }

    if (sdrr_info.status_led_enabled) {
        setup_status_led();
    }

#if defined(MAIN_LOOP_LOGGING)
    // Log some useful information before entering the main loop
    ROM_IMPL_LOG("CS check mask: 0x%08X", cs_check_mask);
    ROM_IMPL_LOG("CS invert mask: 0x%08X", cs_invert_mask);
    ROM_IMPL_LOG("GPIOC_PUPDR: 0x%08X", GPIOC_PUPDR);
#endif // MAIN_LOOP_LOGGING

#if defined(MAIN_LOOP_LOGGING)
    uint32_t byte;
    uint32_t addr_cs;
    while (1) {
        ROM_IMPL_LOG("Waiting for CS to go active");
        if (sdrr_info.status_led_enabled) {
            GPIOB_BSRR = (1 << (15 + 16)); // LED on (PB15 low)
        }
#endif // MAIN_LOOP_LOGGING
    switch (serve_mode)
    {
        // Default case - test CS twice as often as loading the byte from RAM
        default:
        case SERVE_TWO_CS_ONE_ADDR:
            // The targets (from the MOS 2364 data sheet Feburary 1980) are:
            // - tCO - set data line as outputs after CS activates - 200ns
            // - tDF - set data line as inputs after CS deactivates - 175ns
            // - tOH - data lines remain valid after address lines change - 40ns
            // - tACC - maximum time from address to data valid - 450ns
            //
            // tACC and tCO are the most important - together they mean:
            // - we have 450ns from the address lines being set to there being
            //   valid data on the data lines (and therefore them being
            //   outputs)
            // - we also have 200ns from CS activating to the data lines being
            //   set as outputs and having valid data on them
            //
            // They are not cumulative - that is, we cannot assume address
            //  lines will be valid for tACC-tCO = 250ns before CS activates.
            // However, we can rely on both timings - that is that we will get
            // at least 450ns from addresslines being set AND 200ns from CS
            // activating.
            //
            // Together these timings mean we have around twice as long to get
            // the data lines loaded with the byte indicated by the address
            // lines as we do to get the data lines set as outputs (450ns vs
            // 200ns).  Hence we need to be checking the cs line around twice
            // as often as we are checking the address lines.  This is
            // reflected in the code below.
            //
            // They also mean we have to keep checking the address lines and
            // updating the data lines while chip select is active. 
            //
            // We also have to get the data lines back to inputs quickly enough
            // after CS deactivates - this is tDF, which is 175ns.  This is
            // actually slightly tighter than tCO, but the processing between
            // both CS inactive and active cases are very similar we are
            // looking at a worse case of around 150ns with the code below on a
            // 100MHz STM32F411.
            //
            // Finally, we have to make sure we don't change the data lines
            // until at least tOH (40ns) after the address lines change.  This
            // is fine, because loading the byte from RAM and applying it to
            // the data lines will always take longer than this.
            //
            // Macros are used to make the code below more readable. 
            __asm volatile (
                // Start with a branch to the main loop - this is necessary
                // so that while running and CS goes from active to inactive,
                // the code can run straight into the main loop without a
                // branch, speeding up the golden path.
                BRANCH(LOOP)

                // Chip select went active - immediately set data lines as
                // outputs
            LABEL(CS_ACTIVE)
                SET_DATA_OUT

                // By definition we just loaded and tested the address/CS
                // lines, so we have a valid address - so let's load the byte
                // from RAM and apply it, in amongst testing whether CS gone
                // inactive again.
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

                // CS went inactive.  We need to set the data lines as inputs,
                // but also want to update the data using the address lines we
                // have in our hands.
            LABEL(CS_INACTIVE_BYTE)
                SET_DATA_IN
                STORE_TO_DATA
#if defined(MAIN_LOOP_LOGGING)
                BRANCH(END_LOOP)
#endif // MAIN_LOOP_LOGGING

                // Start of main processing loop.  Load the data byte while
                // constantly checking if CS has gone active
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
                : ASM_INPUTS
                : ASM_CLOBBERS
            );
            break;

        // Serve data byte once CS has gone active.  Simpler code, but may not
        // work as well as slower clock speeds as the default algorithm.
        case SERVE_ADDR_ON_CS:
            __asm volatile (
                // Start with a branch to the main loop - this is necessary to,
                // in the usual case, fall through into the main loop when
                // the chip select(s) go inactive.
                BRANCH(ALG2_LOOP)

                // Chip select went active
            LABEL(ALG2_CS_ACTIVE)
                // By definition we just loaded and tested the address/CS
                // lines, immediately load the byte from RAM.  There's no load-
                // use penalty here because we've alrady spent cycles since
                // loading the address lines.
                LOAD_FROM_RAM

                // Set the data lines as outputs.  Doing this now avoids the
                // load-use penalty of immediately applying the byte.  So,
                // this code consumes 2 cycles, instead of consuming 2 cycles,
                // if we did if before LOAD_FROM_RAM, and then 1 cycle load-use
                // penalty here.
                SET_DATA_OUT

                // Now store byte to data lines
                STORE_TO_DATA

                // Now test if CS has gone inactive again
                LOAD_ADDR_CS
                TEST_CS
                BNE(ALG2_CS_INACTIVE)

            LABEL(ALG2_CS_ACTIVE_MID_LOOP)
                LOAD_ADDR_CS
                TEST_CS
                // If still active, load byte from address again in case the
                // address lines changed.  Going backwards here is what the
                // CPU will have predicted so saves a cycle.
                BEQ(ALG2_CS_ACTIVE_MID_LOOP)

                // CS went inactive.  We need to set the data lines as inputs.  Fall through into this code, so no branch penalty.
            LABEL(ALG2_CS_INACTIVE)
                SET_DATA_IN
#if defined(MAIN_LOOP_LOGGING)
                BRANCH(ALG2_END_LOOP)
#endif // MAIN_LOOP_LOGGING
                // Fall into main loop

            LABEL(ALG2_LOOP)
                LOAD_ADDR_CS
                TEST_CS
                BEQ(ALG2_CS_ACTIVE)

#if defined(MAIN_LOOP_LOGGING)
            LABEL(ALG2_END_LOOP)
                "mov %0, " R_ADDR_CS "\n"
                "mov %1, " R_DATA "\n"
#endif // MAIN_LOOP_LOGGING

                // Start main loop again
                BRANCH(ALG2_LOOP)

#if defined(MAIN_LOOP_LOGGING)
                : "=r" (addr_cs),
                "=r" (byte)
#else // !MAIN_LOOP_LOGGING
                :
#endif // MAIN_LOOP_LOGGING
                : ASM_INPUTS
                : ASM_CLOBBERS
            );
            break;

        case SERVE_ADDR_ON_ANY_CS:
            // Same Logic as SERVE_ADDR_ON_CS, except:
            // - TEST_CS_ANY is used instead of TEST_CS(all)
            // - Tests are reversed, as BIC is being used to test
            __asm volatile (
                BRANCH(ALG3_LOOP)

            LABEL(ALG3_CS_ACTIVE)
                LOAD_FROM_RAM
                SET_DATA_OUT
                STORE_TO_DATA
                LOAD_ADDR_CS
                TEST_CS_ANY
                BEQ(ALG3_CS_INACTIVE)

            LABEL(ALG3_CS_ACTIVE_MID_LOOP)
                LOAD_ADDR_CS
                TEST_CS_ANY
                BNE(ALG3_CS_ACTIVE_MID_LOOP)

            LABEL(ALG3_CS_INACTIVE)
                SET_DATA_IN
#if defined(MAIN_LOOP_LOGGING)
                BRANCH(ALG3_END_LOOP)
#endif // MAIN_LOOP_LOGGING

            LABEL(ALG3_LOOP)
                LOAD_ADDR_CS
                TEST_CS_ANY
                BNE(ALG3_CS_ACTIVE)

#if defined(MAIN_LOOP_LOGGING)
            LABEL(ALG3_END_LOOP)
                "mov %0, " R_ADDR_CS "\n"
                "mov %1, " R_DATA "\n"
#endif // MAIN_LOOP_LOGGING

                // Start main loop again
                BRANCH(ALG3_LOOP)

#if defined(MAIN_LOOP_LOGGING)
                : "=r" (addr_cs),
                "=r" (byte)
#else // !MAIN_LOOP_LOGGING
                :
#endif // MAIN_LOOP_LOGGING
                : ASM_INPUTS
                : ASM_CLOBBERS
            );
            break;
    }

#if defined(MAIN_LOOP_LOGGING)
        if (sdrr_info.status_led_enabled) {
            GPIOB_BSRR = (1 << 15);        // LED off (PB15 high)
        }
        ROM_IMPL_LOG("Address/CS: 0x%08X Byte: 0x%08X", addr_cs, byte);
    }
#endif // MAIN_LOOP_LOGGING
}

// Get the index of the selected ROM by reading the select jumpers
//
// Returns the index
uint8_t get_rom_set_index(void) {
    uint8_t rom_sel, rom_index;

    // Read the ROM image selection bits PB0 (LSB) to PB2, and PB7 (MSB).
    rom_sel = GPIOB_IDR;
    rom_sel = (rom_sel & 0x07) | ((rom_sel & 0x80) >> 4);  // Get bits 0-2 and 7, shift PB7 to bit 3
    RCC_AHB1ENR &= ~RCC_AHB1ENR_GPIOBEN;  // Disable GPIOB clock

    // Calculate the ROM image index based on the selection bits and number of
    // images installed in this firmware.  For example, if image 4 was selected
    // but there are only 3 images, it will select image 1.
    rom_index = rom_sel % sdrr_rom_set_count;

    LOG("ROM sel/index %d/%d", rom_sel, rom_index);

    return rom_index;
}

void preload_rom_image(const sdrr_rom_set_t *set) {
    uint32_t *img_src, *img_dst;
    uint32_t img_size;

    // Find the start of this ROM image in the flash memory
    img_size = set->size;
    img_src = (uint32_t *)(set->data);
    img_dst = _ram_rom_image_start;

#if defined(BOOT_LOGGING)
    DEBUG("ROM filename: %s", set->roms[0]->filename);
#endif // BOOT_LOGGING
    switch (set->roms[0]->rom_type) {
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
            DEBUG("%s %d %s", rom_type, set->roms[0]->rom_type, unknown);
            break;
    }
    DEBUG("ROM size %d bytes", img_size);

    // Set image (either single ROM or multiple ROMs) has been fully pre-
    // processed before embedding in the flash.
    memcpy(img_dst, img_src, img_size);

    LOG("ROM %s preloaded to RAM 0x%08X size %d bytes", set->roms[0]->filename, (uint32_t)_ram_rom_image_start, img_size);
    LOG("Set ROM count: %d, Serving algorithm: %d, multi-ROM CS1 state: %d", set->rom_count, set->serve, set->multi_rom_cs1_state);
}

#endif // !TIMER_TEST/TOGGLE_PA4
