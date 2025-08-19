// One ROM 2316/2332/2364 ROM implementation.

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
#include "rom_asm.h"

#if !defined(TIMER_TEST) && !defined(TOGGLE_PA4)

// Log function which can be called from functions potentially run from RAM
#if defined(MAIN_LOOP_LOGGING) && defined(EXECUTE_FROM_RAM)
#error "MAIN_LOOP_LOGGING cannot be used with EXECUTE_FROM_RAM"
#endif // MAIN_LOOP && EXECUTE_FROM_RAM
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

#if defined(COUNT_ROM_ACCESS) && defined(C_MAIN_LOOP)
#error "COUNT_ROM_ACCESS is not supported by C_MAIN_LOOP"
#endif

// Pull in the RAM ROM image start/end locations from the linker
extern uint32_t _ram_rom_image_start[];
extern uint32_t _ram_rom_image_end[];


// Do the fairly complex job of setting up the CS masks for use in the main
// loop algorithm.
//
// This is platform agnostic - it merely requires that all CS and X lines share
// the same GPIO port and hence the same register value can be read/tested to
// check CS/X state. 
static inline void __attribute__((always_inline)) setup_cs_masks(
    const sdrr_info_t *info,
    const sdrr_rom_set_t *set,
    sdrr_serve_t serve_mode,
    const sdrr_rom_info_t *rom,
    uint32_t *check_mask,
    uint32_t *invert_mask
) {
    uint32_t cs_invert_mask = 0;
    uint32_t cs_check_mask; 

    if (serve_mode == SERVE_ADDR_ON_ANY_CS)
    {
        uint8_t pin_cs = info->pins->cs1_2364;
        uint8_t pin_x1 = info->pins->x1;
        uint8_t pin_x2 = info->pins->x2;
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
                {
                    ROM_IMPL_DEBUG("ROM type: 2316");
                    uint8_t pin_cs = info->pins->cs1_2316;
                    uint8_t pin_cs2 = info->pins->cs2_2316;
                    uint8_t pin_cs3 = info->pins->cs3_2316;
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
                }
                break;

            case ROM_TYPE_2332:
                {
                    ROM_IMPL_DEBUG("ROM type: 2332");
                    uint8_t pin_cs = info->pins->cs1_2332;
                    uint8_t pin_cs2 = info->pins->cs2_2332;
                    cs_check_mask = (1 << pin_cs) | (1 << pin_cs2);
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
                }
                break;

            default:
                ROM_IMPL_LOG("Unsupported ROM type: %d", rom->rom_type);
                __attribute__((fallthrough));
            case ROM_TYPE_2364:
                {
                    ROM_IMPL_DEBUG("ROM type: 2364");
                    uint8_t pin_cs = info->pins->cs1_2364;
                    cs_check_mask = (1 << pin_cs);
                    if (rom->cs1_state == CS_ACTIVE_LOW) {
                        ROM_IMPL_DEBUG("CS1 active low");
                    } else {
                        ROM_IMPL_DEBUG("CS1 active high");
                        cs_invert_mask |= 1 << pin_cs;
                    }
                }
                break;
        }
    }
    
    *check_mask = cs_check_mask;
    *invert_mask = cs_invert_mask;

    return;
}

// Sets the appropriate X1/X2 pull-ups or pull-downs based on the type of ROM
// set being served.
static inline void __attribute__((always_inline)) configure_x_pulls(
    const sdrr_rom_set_t *set,
    const sdrr_pins_t *pins,
    sdrr_serve_t serve_mode
) {
    // Set the appropriate X1/X2 pulls.
    if (set->rom_count == 1) {
        // We are serving a single ROM image.  This means we want X1/X2 set to
        // low.
        set_x_pulls(pins, 0, 0);
    } else {
        if (serve_mode != SERVE_ADDR_ON_ANY_CS) {
            // We are serving from a dynamically selected bank of ROMs, using
            // the X1/X2 pins.  We need to pull the X pins in the opposite
            // direction to the jumper.
            ROM_IMPL_LOG("Serving bank switched images");
            if (pins->x_jumper_pull == 0) {
                set_x_pulls(pins, 1, 1);
            } else {
                set_x_pulls(pins, 0, 0);
            }
        } else {
            // Serving multiple ROM images simultaneously.  In this case we
            // don't care about the board jumpers - they aren't being used.
            // So always set the pulls to the opposite of multi-rom CS1 state
            if (set->multi_rom_cs1_state == CS_ACTIVE_HIGH) {
                set_x_pulls(pins, 0, 0);
            } else {
                set_x_pulls(pins, 1, 1);
            }
        }
    }
}

void __attribute__((section(".main_loop"), used)) main_loop(
    const sdrr_info_t *info,
    const sdrr_rom_set_t *set
) {
    ROM_IMPL_LOG("%s", log_divider);
    ROM_IMPL_LOG("Entered main_loop");

    // Set up serving algorithm
    sdrr_serve_t serve_mode = set->serve;
    if ((set->rom_count == 1) && (serve_mode == SERVE_ADDR_ON_ANY_CS)) {
        serve_mode = SERVE_DEFAULT_1_ROM;
    }

#ifndef EXECUTE_FROM_RAM
    // We don't copy filenames over in the RAM case, so this won't work - and
    // neither does MAIN_LOOP_LOGGING
    for (int ii = 0; ii < set->rom_count; ii++) {
        ROM_IMPL_DEBUG("Serve ROM #%d: %s via mode: %d", ii, set->roms[ii]->filename, serve_mode);
    }
#endif // EXECUTE_FROM_RAM

    //
    // Set up CS pin masks, using CS values from sdrr_info
    //
    uint32_t cs_invert_mask;
    uint32_t cs_check_mask;
    const sdrr_rom_info_t *rom = set->roms[0];
    setup_cs_masks(
        info,
        set,
        serve_mode,
        rom,
        &cs_check_mask,
        &cs_invert_mask
    );

    // Set up the GPIOs
    main_loop_gpio_init();

    // Configure the X1/X2 pull-ups or pull-downs
    configure_x_pulls(
        set,
        info->pins,
        serve_mode
    );

    // Calculate pre-load values for data input/output masks
    uint32_t data_output_mask_val;
    uint32_t data_input_mask_val;
    setup_data_masks(
        info,
        &data_output_mask_val,
        &data_input_mask_val
    );

    // Set up the ROM table variables (the ROM is already in RAM by this point,
    // if RAM preloading is enabled).
    uint32_t rom_table_val = (uint32_t)sdrr_runtime_info.rom_table;

    // If we are counting ROM accesses, set it up
#if defined(COUNT_ROM_ACCESS) && !defined(C_MAIN_LOOP)
    sdrr_runtime_info.access_count = 0;  // Update from 0xFFFFFFFF to 0.
    sdrr_runtime_info.count_rom_access = 1;  // Flag as enabled
    uint32_t access_count_addr = (uint32_t)&sdrr_runtime_info.access_count;
    uint32_t access_count = 0;  // Used to initialise the count register itself
#endif // defined(COUNT_ROM_ACCESS) && !defined(C_MAIN_LOOP)

    // Now log current state, and items we're going to load to registers.
    ROM_IMPL_DEBUG("%s", log_divider);
    ROM_IMPL_DEBUG("Register locations and values:");
#if defined(STM32F4)
    // It would be better to put this in the platform specific calls above,
    // but those are in platform specific files with no access t
    // ROM_IMPL_DEBUG
    ROM_IMPL_DEBUG("GPIOA_MODER: 0x%08X", GPIOA_MODER);
    ROM_IMPL_DEBUG("GPIOA_PUPDR: 0x%08X", GPIOA_PUPDR);
    ROM_IMPL_DEBUG("GPIOA_OSPEEDR: 0x%08X", GPIOA_OSPEEDR);
    ROM_IMPL_DEBUG("GPIOC_MODER: 0x%08X", GPIOC_MODER);
    ROM_IMPL_DEBUG("GPIOC_PUPDR: 0x%08X", GPIOC_PUPDR);
#endif // STM32F4
    ROM_IMPL_DEBUG("VAL_DATA_MODER: 0x%08X", VAL_DATA_MODER);
    ROM_IMPL_DEBUG("VAL_DATA_ODR: 0x%08X", VAL_DATA_ODR);
    ROM_IMPL_DEBUG("VAL_ADDR_CS_IDR: 0x%08X", VAL_ADDR_CS_IDR);
    ROM_IMPL_DEBUG("CS check mask: 0x%08X", cs_check_mask);
    ROM_IMPL_DEBUG("CS invert mask: 0x%08X", cs_invert_mask);
    ROM_IMPL_DEBUG("Data output mask: 0x%08X", data_output_mask_val);
    ROM_IMPL_DEBUG("Data input mask: 0x%08X", data_input_mask_val);
    ROM_IMPL_DEBUG("ROM table: 0x%08X", rom_table_val);
#if defined(COUNT_ROM_ACCESS)
    ROM_IMPL_DEBUG("Access count addr: 0x%08X", access_count_addr);
    ROM_IMPL_DEBUG("Access count: 0x%08X", access_count);
#endif
    ROM_IMPL_DEBUG("%s", log_divider);

#if defined(MAIN_LOOP_ONE_SHOT)
    uint32_t byte;
    uint32_t addr_cs;
    while (1) {
        ROM_IMPL_LOG("Waiting for CS to go active");
#else
    ROM_IMPL_LOG("Begin serving data");
#endif // MAIN_LOOP_ONE_SHOT
    if ((info->status_led_enabled) && (info->pins->status < MAX_USED_GPIOS)) {
        status_led_on(info->pins->status);
    }

#if !defined(C_MAIN_LOOP)
    // Start the appropriate main loop.  Includes preloading registers.
    //
    // See `rom_asm.h` for the macros used here.
    //
    // We use cs_invert_mask as the easiest proxy for whether CS is active low
    // or others.  In the case of multiple CS lines, it's only zero if _all_
    // CS lines are active low, which is the test we want.  This means each CS
    // test is 1 cycle quicker.
    switch (serve_mode)
    {
        // Default case - test CS twice as often as loading the byte from RAM
        case SERVE_TWO_CS_ONE_ADDR:
            if (cs_invert_mask == 0) {
                // CS active low
                ALG1_ASM(TEST_CS_ACT_LOW);
            } else {
                ALG1_ASM(TEST_CS);
            }

            break;

        // Serve data byte once CS has gone active.  Simpler code, but may not
        // work as well as slower clock speeds as the default algorithm.  Or
        // may work better!  See `rom_asm.h` for more details.
        default:
        case SERVE_ADDR_ON_CS:
#if !defined(COUNT_ROM_ACCESS)
            if (cs_invert_mask == 0) {
                // CS active low
                ALG2_ASM(TEST_CS_ACT_LOW, BEQ);
            } else {
                ALG2_ASM(TEST_CS, BEQ);
            }
#else // COUNT_ROM_ACCESS
            if (cs_invert_mask == 0) {
                // CS active low
                ALG2_COUNT_ASM(TEST_CS_ACT_LOW, BEQ, BNE);
            } else {
                ALG2_COUNT_ASM(TEST_CS, BEQ, BNE);
            }
#endif // !COUNT_ROM_ACCESS
            break;

        // Used for multi-ROM sets
        case SERVE_ADDR_ON_ANY_CS:
            // This case uses the same algorithm as SERVE_ADDR_ON_CS, but
            // the BEQ becomes a BNE as the TEST in this case is the opposite
            // way around.
#if !defined(COUNT_ROM_ACCESS)
            if (cs_invert_mask == 0) {
                // CS active low
                ALG2_ASM(TEST_CS_ANY_ACT_LOW, BNE);
            } else {
                ALG2_ASM(TEST_CS_ANY, BNE);
            }
#else // COUNT_ROM_ACCESS
            if (cs_invert_mask == 0) {
                // CS active low
                ALG2_COUNT_ASM(TEST_CS_ANY_ACT_LOW, BNE, BEQ);
            } else {
                ALG2_COUNT_ASM(TEST_CS_ANY, BNE, BEQ);
            }
#endif // !COUNT_ROM_ACCESS
            break;
    }
#else // C_MAIN_LOOP
#if defined(RP235X)
#error "C main loop not supported on RP235X"
#endif
    uint16_t addr_cs_lines;
    uint8_t data_byte;
    uint32_t cs_check;
    switch (serve_mode) {
        // For now we'll just have a single C main loop implemenation which
        // serves address on C
        default:
        case SERVE_TWO_CS_ONE_ADDR:
        case SERVE_ADDR_ON_CS:
            if (cs_invert_mask == 0) {
                while (1) {
                    addr_cs_lines = GPIOC_IDR;  // ALG2_ALL_LOW
                    while (!(cs_check_mask & addr_cs_lines)) {  // ALG2_ALL_LOW
                        data_byte = *(((uint8_t*)rom_table_val) + addr_cs_lines);  // ALG2_ALL_LOW
                        GPIOA_MODER = data_output_mask_val;  // ALG2_ALL_LOW
                        GPIOA_ODR = data_byte;  // ALG2_ALL_LOW
                        addr_cs_lines = GPIOC_IDR;  // ALG2_ALL_LOW
                    }  // ALG2_ALL_LOW
                    GPIOA_MODER = data_input_mask_val;  // ALG2_ALL_LOW
                }
            } else {
#if defined(DUMB_C_MAIN_LOOP_2_CS)
// Don't use this - it was a demonstration to see what a naive C
// implementation would assemble to.  It makes assumptions - in particular
// that there are 2 CS lines - so won't be operational in the general
// case.  In the specific case it requires a clock speed of 135-140MHz for
// a C64 char ROM.
                while (1) {
                    addr_cs_lines = GPIOC_IDR;  // ALG2_DUMB
                    while
                        ((((rom->cs1_state == CS_ACTIVE_LOW) &&
                           !(addr_cs_lines & (1 << info->pins->cs1_2332)))
                          ||
                          ((rom->cs1_state == CS_ACTIVE_HIGH) &&
                           (addr_cs_lines & (1 << info->pins->cs1_2332))))
                         &&
                         (((rom->cs2_state == CS_ACTIVE_LOW) &&
                           !(addr_cs_lines & (1 << info->pins->cs2_2332)))
                          ||
                          ((rom->cs2_state == CS_ACTIVE_HIGH) &&
                           (addr_cs_lines & (1 << info->pins->cs2_2332))))) {
                        data_byte = *(((uint8_t*)rom_table_val) + addr_cs_lines);
                        GPIOA_ODR = data_byte;
                        GPIOA_MODER = data_output_mask_val;                    
                        addr_cs_lines = GPIOC_IDR;
                    }
                    GPIOA_MODER = data_input_mask_val;
                }
#else // !DUMB_C_MAIN_LOOP_2_CS
// This implementation requires a clock speed of 98-100Mhz for a C64 char ROM,
// compared with 79-80MHz for the assembly implementation.

// This #define tried to persuade the compiler to remove its uxth instructions
#define GPIOC_IDR_16BIT          (*(volatile uint16_t *)(GPIOC_BASE + GPIO_IDR_OFFSET))
                while (1) {
                    addr_cs_lines = GPIOC_IDR_16BIT;  // ALG2_MIXED
                    cs_check = addr_cs_lines ^ cs_invert_mask;  // ALG2_MIXED
                    while (!(cs_check_mask & cs_check)) {  // ALG2_MIXED
                        data_byte = *(((uint8_t*)rom_table_val) + addr_cs_lines);// ALG2_MIXED
                        GPIOA_MODER = data_output_mask_val;  // ALG2_MIXED
                        GPIOA_ODR = data_byte;  // ALG2_MIXED
                        addr_cs_lines = GPIOC_IDR_16BIT;  // ALG2_MIXED
                        cs_check = addr_cs_lines ^ cs_invert_mask; // ALG2_MIXED
                    }  // ALG2_MIXED
                    GPIOA_MODER = data_input_mask_val;  // ALG2_MIXED
                }
#endif // DUMB_C_MAIN_LOOP_2_CS
            }
            break;

        case SERVE_ADDR_ON_ANY_CS:
            if (cs_invert_mask == 0) {
                while (1) {
                    addr_cs_lines = GPIOC_IDR;
                    while (cs_check_mask & ~addr_cs_lines) {
                        data_byte = *(((uint8_t*)rom_table_val) + addr_cs_lines);
                        GPIOA_MODER = data_output_mask_val;
                        GPIOA_ODR = data_byte;
                        addr_cs_lines = GPIOC_IDR;
                    }
                    GPIOA_MODER = data_input_mask_val;
                }
            } else {
                while (1) {
                    addr_cs_lines = GPIOC_IDR;
                    cs_check = addr_cs_lines ^ cs_invert_mask; // needed only when cs_invert_mask != 0
                    while (cs_check_mask & ~cs_check) {
                        data_byte = *(((uint8_t*)rom_table_val) + addr_cs_lines);
                        GPIOA_MODER = data_output_mask_val;
                        GPIOA_ODR = data_byte;
                        addr_cs_lines = GPIOC_IDR;
                        cs_check = addr_cs_lines ^ cs_invert_mask;
                    }
                    GPIOA_MODER = data_input_mask_val;
                }
            }
            break;

    }
#endif // !C_MAIN_LOOP

    if ((info->status_led_enabled) && (info->pins->status < MAX_USED_GPIOS)) {
        status_led_off(info->pins->status);
    }
#if defined(MAIN_LOOP_ONE_SHOT)
        ROM_IMPL_LOG("Address/CS: 0x%08X Byte: 0x%08X", addr_cs, byte);
    }
#endif // MAIN_LOOP_ONE_SHOT
}

// Get the index of the selected ROM by reading the select jumpers
//
// Returns the index
uint8_t get_rom_set_index(void) {
    uint8_t rom_sel, rom_index;

    uint32_t sel_pins, sel_mask;
    sel_pins = check_sel_pins(&sel_mask);

    // Shift the sel pins to read from 0.  Do this by shifting each present
    // bit (usig sel_mask) the appropriate number of bits right
    rom_sel = 0;
    int bit_pos = 0;
    for (int ii = 0; ii < 32; ii++) {
        if (sel_mask & (1 << ii)) {
            if (sel_pins & (1 << ii)) {
                rom_sel |= (1 << bit_pos);
            }
            bit_pos++;
        }
    }

    // Calculate the ROM image index based on the selection bits and number of
    // images installed in this firmware.  For example, if image 4 was selected
    // but there are only 3 images, it will select image 1.
    rom_index = rom_sel % sdrr_rom_set_count;

    LOG("ROM sel/index %d/%d", rom_sel, rom_index);

    return rom_index;
}

void* preload_rom_image(const sdrr_rom_set_t *set) {
    uint32_t *img_src, *img_dst;
    uint32_t img_size;

    // Find the start of this ROM image in the flash memory
    img_size = set->size;
    img_src = (uint32_t *)(set->data);
#if defined(CCM_RAM_BASE) && !defined(DISABLE_CCM)
    if (sdrr_info.mcu_line == F405) {
        // Preload to CCM RAM
        LOG("F405: Preloading ROM image to CCM RAM");
        img_dst = (uint32_t *)CCM_RAM_BASE;
    } else {
#else 
    if (sdrr_info.mcu_line == F405) {
        LOG("F405: NOT Preloading ROM image to CCM RAM");
    }
#endif // defined(CCM_RAM_BASE) && !defined(DISABLE_CCM)
    img_dst = _ram_rom_image_start;
#if defined(CCM_RAM_BASE) && !defined(DISABLE_CCM)
    }
#endif // defined(CCM_RAM_BASE) && !defined(DISABLE_CCM)

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
    LOG("Set ROM count: %d, Serving algorithm: %d, multi-ROM CS1 state: %s",
        set->rom_count, set->serve, cs_values[set->multi_rom_cs1_state]);

    return (void *)img_dst;
}

#endif // !TIMER_TEST/TOGGLE_PA4
