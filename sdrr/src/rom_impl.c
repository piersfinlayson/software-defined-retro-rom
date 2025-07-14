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
#include "rom_asm.h"

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



// Assembly code macros



void __attribute__((section(".main_loop"), used)) main_loop(const sdrr_rom_set_t *set) {
#ifdef MAIN_LOOP_LOGGING
    // Do a bunch of checking things are as we need them.  There's not much
    // point in doing this until MAIN_LOOP_LOGGING is defined, as no-one
    // will hear us if we scream ...
    // Note that sdrr-gen should have got this stuff right.
    ROM_IMPL_LOG("%s", log_divider);
    ROM_IMPL_LOG("Entered main_loop");
    if (sdrr_info.pins->data_port != PORT_A) {
        ROM_IMPL_LOG("!!! Data pins not using port A");
    }
    if (sdrr_info.pins->addr_port != PORT_C) {
        ROM_IMPL_LOG("!!! Address pins not using port C");
    }
    if (sdrr_info.pins->cs_port != PORT_C) {
        ROM_IMPL_LOG("!!! Chip select pins not using port C");
    }
    if (sdrr_info.pins->rom_pins != 24) {
        ROM_IMPL_LOG("!!! Have been told to emulate unsupported %d pin ROM", sdrr_info.pins->rom_pins);
    }
    for (int ii = 0; ii < 13; ii++) {
        if (sdrr_info.pins->addr[ii] > 13) {
            ROM_IMPL_LOG("!!! Address line A%d invalid", ii);
        }
    }
    for (int ii = 0; ii < 8; ii++) {
        if (sdrr_info.pins->data[ii] > 7) {
            ROM_IMPL_LOG("!!! ROM line D%d invalid", ii);
        }
    }
    if (set->rom_count > 1) {
        if (sdrr_info.pins->x1 > 15) {
            ROM_IMPL_LOG("!!! Multi-ROM mode, but pin X1 invalid");
        }
        if (sdrr_info.pins->x2 > 15) {
            ROM_IMPL_LOG("!!! Multi-ROM mode, but pin X2 invalid");
        }
        if (sdrr_info.pins->x1 == sdrr_info.pins->x2) {
            ROM_IMPL_LOG("!!! Multi-ROM mode, but pin X1=X2");
        }
    }
#endif

    //
    // Set up serving algorithm
    //

    // Set up serve mode
    sdrr_serve_t serve_mode = set->serve;

    // Warn if serve mode is incorrectly set for multiple ROM images
    if ((set->rom_count > 1) && (serve_mode != SERVE_ADDR_ON_ANY_CS)) {
        ROM_IMPL_LOG("!!! Mutliple ROM images - wrong serve mode - rectifying");
        serve_mode = SERVE_ADDR_ON_ANY_CS;
    } else if ((set->rom_count == 1) && (serve_mode == SERVE_ADDR_ON_ANY_CS)) {
        ROM_IMPL_LOG("!!! Single ROM image - wrong serve mode - defaulting");
        serve_mode = SERVE_TWO_CS_ONE_ADDR;
    }

    for (int ii = 0; ii < set->rom_count; ii++) {
        ROM_IMPL_DEBUG("Serve ROM #%d: %s via mode: %d", ii, set->roms[ii]->filename, serve_mode);
    }

    //
    // Set up CS pin masks, using CS values from sdrr_info.
    //
    uint32_t cs_invert_mask = 0;
    uint32_t cs_check_mask;
    const sdrr_rom_info_t *rom = set->roms[0];

    if (serve_mode == SERVE_ADDR_ON_ANY_CS)
    {
        uint8_t pin_cs = sdrr_info.pins->cs1_2364;
        uint8_t pin_x1 = sdrr_info.pins->x1;
        uint8_t pin_x2 = sdrr_info.pins->x2;
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
                    uint8_t pin_cs = sdrr_info.pins->cs1_2316;
                    uint8_t pin_cs2 = sdrr_info.pins->cs2_2316;
                    uint8_t pin_cs3 = sdrr_info.pins->cs3_2316;
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
                    uint8_t pin_cs = sdrr_info.pins->cs1_2332;
                    uint8_t pin_cs2 = sdrr_info.pins->cs2_2332;
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
                    uint8_t pin_cs = sdrr_info.pins->cs1_2364;
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

    //
    // Set up the GPIOs
    //

    // Enable GPIO clocks for the ports with address and data lines
    RCC_AHB1ENR |= (RCC_AHB1ENR_GPIOAEN | RCC_AHB1ENR_GPIOCEN);
    
    // Configure PA0-7 as inputs initially (00 in MODER), no pull-up/down
    // Also PA10-12 are duplicate CS lines on some hw so set as inputs no
    // PU/PD.
    // We could theoretically check here that D0-7 uses PA0-7, but there's
    // checks like this above, and in sdrr-gen, so little point.  It's
    // required that they use 0-7 on a port, to avoid any bit shifting when
    // applying the value.
    GPIOA_MODER &= ~0x00FCFFFF; // Clear bits 0-15 (PA0-7, 10-12 as inputs)
    GPIOA_PUPDR &= ~0x00FCFFFF; // Clear pull-up/down for PA0-7, 10-12)
    GPIOA_OSPEEDR &= ~0xFFFF;   // Clear output speed for PA0-7
    GPIOA_OSPEEDR |= 0xAAAA;    // Set PA0-7 speed to "fast", not "high" to
                                // ensure V(OL) is max 0.4V

    // Port C for address and CS lines - set all pins as inputs
    GPIOC_MODER = 0;  // Set all pins as inputs
    uint32_t gpioc_pupdr;
    if (set->rom_count == 1) {
        // Set pull-downs on PC14/15 only, so RAM lookup only takes 16KB.
        // We checked the address lines are lines 0-13 above, and in sdrr-gen
        // so this is reasonable.
        gpioc_pupdr = 0xA0000000;
    }
    else {
        // Hardware revision F has X1/X2 on the PCB, so up to 2 extra ROM chip
        // select lines can be terminated on SDRR.  Set pull-ups or downs on
        // these lines so they are default inactive, in case the user doesn't
        // connect them.
        //
        // Note this introduces pull-ups/downs on the actual CS lines if they
        // are connected.  However, they typically only serve the ROM we are
        // emulating, come from a BCD IC or similar, and the pulls are weak -
        // around 40K ohms.
        uint32_t pull;
        if (set->multi_rom_cs1_state == CS_ACTIVE_HIGH) {
            pull = 0b10;  // Pull down
        } else {
            pull = 0b01;  // Pull up
        }
        gpioc_pupdr = (pull << (sdrr_info.pins->x1 * 2)) |
                        (pull << (sdrr_info.pins->x2 * 2));
    }
    GPIOC_PUPDR = gpioc_pupdr;

    if (sdrr_info.status_led_enabled) {
        setup_status_led();
    }

    //
    // Calculcate pre-load values for all registers
    //
    uint32_t data_output_mask_val;
    uint32_t data_input_mask_val;
    if (sdrr_info.mco_enabled) {
        // PA8 is AF, PA0-7 are inputs
        data_output_mask_val = 0x00025555;
        data_input_mask_val = 0x00020000;
    } else {
        // PA0-7 are inputs
        data_output_mask_val = 0x00005555;
        data_input_mask_val = 0x00000000;
    }

    if (sdrr_info.swd_enabled) {
        // Ensure PA13/14 remain AF (SWD enabled)
        data_output_mask_val |= 0x28000000;
        data_input_mask_val |= 0x28000000;
    }

    uint32_t rom_table_val;
    if (sdrr_info.preload_image_to_ram) {
        rom_table_val = (uint32_t)&_ram_rom_image_start;
    } else {
        rom_table_val = (uint32_t)&(set->data[0]);
    }

    // Now log current state, and items we're going to load to registers.
    ROM_IMPL_DEBUG("%s", log_divider);
    ROM_IMPL_DEBUG("Register locations and values:");
    ROM_IMPL_DEBUG("GPIOA_MODER: 0x%08X", GPIOA_MODER);
    ROM_IMPL_DEBUG("GPIOA_PUPDR: 0x%08X", GPIOA_PUPDR);
    ROM_IMPL_DEBUG("GPIOA_OSPEEDR: 0x%08X", GPIOA_OSPEEDR);
    ROM_IMPL_DEBUG("GPIOC_MODER: 0x%08X", GPIOC_MODER);
    ROM_IMPL_DEBUG("GPIOC_PUPDR: 0x%08X", GPIOC_PUPDR);
    ROM_IMPL_DEBUG("VAL_GPIOA_ODR: 0x%08X", VAL_GPIOA_ODR);
    ROM_IMPL_DEBUG("VAL_GPIOA_MODER: 0x%08X", VAL_GPIOA_MODER);
    ROM_IMPL_DEBUG("VAL_GPIOC_IDR: 0x%08X", VAL_GPIOC_IDR);
    ROM_IMPL_DEBUG("CS check mask: 0x%08X", cs_check_mask);
    ROM_IMPL_DEBUG("CS invert mask: 0x%08X", cs_invert_mask);
    ROM_IMPL_DEBUG("Data output mask: 0x%08X", data_output_mask_val);
    ROM_IMPL_DEBUG("Data input mask: 0x%08X", data_input_mask_val);
    ROM_IMPL_DEBUG("ROM table: 0x%08X", rom_table_val);
    ROM_IMPL_DEBUG("%s", log_divider);

#if defined(MAIN_LOOP_ONE_SHOT)
    uint32_t byte;
    uint32_t addr_cs;
    while (1) {
        ROM_IMPL_LOG("Waiting for CS to go active");
        if (sdrr_info.status_led_enabled) {
            GPIOB_BSRR = (1 << (15 + 16)); // LED on (PB15 low)
        }
#else
    ROM_IMPL_LOG("Begin serving data");
#endif // MAIN_LOOP_ONE_SHOT

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
        default:
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
        case SERVE_ADDR_ON_CS:
            if (cs_invert_mask == 0) {
                // CS active low
                ALG2_ASM(TEST_CS_ACT_LOW, BEQ);
            } else {
                ALG2_ASM(TEST_CS, BEQ);
            }
            break;

        // Used for multi-ROM sets
        case SERVE_ADDR_ON_ANY_CS:
            // This case uses the same algorithm as SERVE_ADDR_ON_CS, but
            // the BEQ becomes a BNE as the TEST in this case is the opposite
            // way around.
            if (cs_invert_mask == 0) {
                // CS active low
                ALG2_ASM(TEST_CS_ANY_ACT_LOW, BNE);
            } else {
                ALG2_ASM(TEST_CS_ANY, BNE);
            }
            break;
    }

#if defined(MAIN_LOOP_ONE_SHOT)
        if (sdrr_info.status_led_enabled) {
            GPIOB_BSRR = (1 << 15);        // LED off (PB15 high)
        }
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
    LOG("Set ROM count: %d, Serving algorithm: %d, multi-ROM CS1 state: %s",
        set->rom_count, set->serve, cs_values[set->multi_rom_cs1_state]);
}

#endif // !TIMER_TEST/TOGGLE_PA4
