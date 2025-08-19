// One ROM RP235X Specific Routines

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#include "include.h"
#include "roms.h"

// Internal function prototypes
static void setup_xosc(void);
static void setup_pll(void);

// RP2350 firmware needs a special boot block so the bootloader will load it.
// See datasheet S5.9.5 and ../include/reg-rp235x.h.
// It must be in the first 4KB of the flash firmware image.  This follows our
// reset vectors, which is fine.  Given we do not include a VECTOR_TABLE
// block, the bootloader assumes it is present at the start of flash - which it
// is.
__attribute__((section(".rp2350_block"))) const rp2350_boot_block_t rp2350_arm_boot_block = {
    .start_marker    = 0xffffded3,
    .image_type_tag  = 0x42,
    .image_type_len  = 0x1,
    .image_type_data = 0b0001000000100001,
    .type            = 0xff,
    .size            = 0x0001,
    .pad             = 0,
    .next_block      = 0,
    .end_marker      = 0xab123579
};

void platform_specific_init(void) {
    // RP235X needs to reset the JTAG interface to enable SWD (for example for
    // RTT logging)
    RESET_RESET |= RESET_JTAG;
    RESET_RESET &= ~RESET_JTAG;
    while (!(RESET_DONE & RESET_JTAG));
    DEBUG("JTAG reset complete");
}

void setup_clock(void) {
    LOG("Setting up clock");

    setup_xosc();
    setup_pll();
}

void setup_gpio(void) {
    // Take IO bank and pads bank out of reset
    RESET_RESET &= ~(RESET_IOBANK0 | RESET_PADS_BANK0);
    while (!(RESET_DONE & (RESET_IOBANK0 | RESET_PADS_BANK0)));

    // Set all GPIO pins to SIOs, inputs, output disable, no pulls
    for (int ii = 0; ii < MAX_USED_GPIOS; ii++) {
        GPIO_CTRL(ii) = GPIO_CTRL_RESET;
        GPIO_PAD(ii) = PAD_INPUT | PAD_OUTPUT_DISABLE;
    }

    // Go through the data pins, disabling the output disable and setting the
    // drive strength.  We don't actually set as an output here.
    // Set the drive strength to 8mA and slew rate to fast.
    for (int ii = 0; ii < 8; ii++) {
        uint8_t pin = sdrr_info.pins->data[ii];
        if (pin < MAX_USED_GPIOS) {
            GPIO_PAD(sdrr_info.pins->data[ii]) &= ~PAD_OUTPUT_DISABLE;
            GPIO_PAD(sdrr_info.pins->data[ii]) |= PAD_DRIVE_8MA | PAD_SLEW_FAST;
        } else {
            LOG("!!! Data pins %d out of range", pin);
        }
    }

    // If there's a status LED, set it up as an output pin, high (LED off).
    if (sdrr_info.pins->status != INVALID_PIN) {
        uint8_t pin = sdrr_info.pins->status;
        if (pin < MAX_USED_GPIOS) {
            GPIO_PAD(pin) &= ~(PAD_OUTPUT_DISABLE | PAD_INPUT);
            GPIO_PAD(pin) = PAD_DRIVE_2MA;
            SIO_GPIO_OUT_SET = (1 << pin);
            SIO_GPIO_OE_SET = (1 << pin);
        } else {
            LOG("!!! Status LED pin %d out of range", pin);
        }
    }
}

// Set up the PLL with the generated values
void setup_pll(void) {
    // Release PLL_SYS from reset
    RESET_RESET &= ~RESET_PLL_SYS;
    while (!(RESET_DONE & RESET_PLL_SYS));

    // Power down the PLL, set the feedback divider
    PLL_SYS_PWR = PLL_PWR_PD | PLL_PWR_VCOPD;

    // Set feedback divider and reference divider
    PLL_SYS_FBDIV_INT = PLL_SYS_FBDIV;
    PLL_SYS_CS = PLL_CS_REFDIV(PLL_SYS_REFDIV);

    // Power up VCO (keep post-dividers powered down)
    PLL_SYS_PWR = PLL_PWR_POSTDIVPD;

    // Wait for PLL to lock
    while (!(PLL_SYS_CS & PLL_CS_LOCK));

    // Set post dividers and power up everything
    PLL_SYS_PRIM = PLL_SYS_PRIM_POSTDIV1(PLL_SYS_POSTDIV1) |
                     PLL_SYS_PRIM_POSTDIV2(PLL_SYS_POSTDIV2);

    // Power up post dividers
    PLL_SYS_PWR = 0;

    // Switch to the PLL
    CLOCK_SYS_CTRL = CLOCK_SYS_SRC_AUX | CLOCK_SYS_AUXSRC_PLL_SYS;
    while ((CLOCK_SYS_SELECTED & (1 << 1)) == 0);
}

void setup_mco(void) {
    LOG("!!! MCO not supported on RP235X");
}

// Set up the image select pins to be inputs with the appropriate pulls.
uint32_t setup_sel_pins(uint32_t *sel_mask) {
    uint32_t num;
    uint32_t pad;

    if (sdrr_info.pins->sel_jumper_pull == 0) {
        // Jumper will pull down, so we pull up
        pad = PAD_INPUT_PU;
    } else if (sdrr_info.pins->sel_jumper_pull == 1) {
        // Jumper will pull up, so we pull down
        pad = PAD_INPUT_PD;
    } else {
        LOG("!!! Invalid sel pull %d", sdrr_info.pins->sel_jumper_pull);
        return 0;
    }

    *sel_mask = 0;
    num = 0;
    for (int ii = 0; (ii < MAX_IMG_SEL_PINS); ii++) {
        uint8_t pin = sdrr_info.pins->sel[ii];
        if (pin < MAX_USED_GPIOS) {
            // Enable pull-up
            GPIO_PAD(pin) = pad;

            // Set the pin in our bit mask
            *sel_mask |= (1 << pin);

            num += 1;
        } else if (pin != INVALID_PIN) {
            LOG("!!! Sel pin %d >= %d - not using", pin, MAX_USED_GPIOS);
        }
    }

    // Short delay to allow the pulls to settle.
    for(volatile int ii = 0; ii < 10; ii++);

    return num;
}

// Get the value of the sel pins.  If, on this board, the MCU pulls are low
// (i.e. closing the jumpers pulls them up) we return the value as is, as
// closed should indicate 1.  In the other case, where MCU pulls are high
// (closing jumpers) pulls the pins low, we invert - so closed still indicates
// 1.
//
// We will probably make this behaviour configurable soon.
//
// On all RP2350 boards, the SEL pins are pulled low by jumpers to indicate
// a 1, so reverse to the default STM32F4 behavior.
uint32_t get_sel_value(uint32_t sel_mask) {
    uint8_t invert;
    uint32_t gpio_value;

    if (sdrr_info.pins->sel_jumper_pull == 0) {
        // Closing the jumper produces a 0, so invert
        invert = 1;
    } else {
        // Closing the jumper produces a 1, so don't invert
        invert = 0;
    }

    gpio_value = SIO_GPIO_IN & sel_mask;
    if (invert) {
        // If we are inverting, we need to flip the bits
        gpio_value = ~gpio_value;
    }

    return gpio_value;
}

void disable_sel_pins(void) {
    for (int ii = 0; (ii < MAX_IMG_SEL_PINS); ii++) {
        uint8_t pin = sdrr_info.pins->sel[ii];
        if (pin < MAX_USED_GPIOS) {
            // Disable pulls
            GPIO_PAD(pin) = ~(PAD_PU | PAD_PD);

        }
    }
}

void setup_status_led(void) {
    // No-op - done in setup_gpio()
}

void blink_pattern(uint32_t on_time, uint32_t off_time, uint8_t repeats) {
    (void)on_time;
    (void)off_time;
    (void)repeats;
    LOG("!!! Blink pattern not supported on RP235X");
}

// Enters bootloader mode.
void enter_bootloader(void) {
    // Set the stack pointer
    asm volatile("msr msp, %0" : : "r" (*((uint32_t*)0x1FFFF000)));
    
    // Jump to the bootloader
    ((void(*)())(*((uint32_t*)0x1FFFF004)))();
}

void check_config(
    const sdrr_info_t *info,
    const sdrr_rom_set_t *set
) {
    // Currently only support emulating a 24 pin ROM
    if (info->pins->rom_pins != 24) {
        LOG("!!! Have been told to emulate unsupported %d pin ROM", info->pins->rom_pins);
    }

    // Check ports (banks on RP235X) are as expected
    if (info->pins->data_port != PORT_0) {
        LOG("!!! Data pins should be using bank 0");
    }
    if (info->pins->addr_port != PORT_0) {
        LOG("!!! Address pins should be using bank 0");
    }
    if (info->pins->cs_port != PORT_0) {
        LOG("!!! CS pins should be using bank 0");
    }
    if (info->pins->sel_port != PORT_0) {
        LOG("!!! Sel pins should be using bank 0");
    }

    // We expect to use pins 0-16 for address lines 
    for (int ii = 0; ii < 13; ii++) {
        uint8_t pin = info->pins->addr[ii];
        if (pin > 16) {
            LOG("!!! Address line A%d using invalid pin %d", ii, pin);
        }
    }

    // We expect to use pins 16-23 for data lines
    for (int ii = 0; ii < 8; ii++) {
        uint8_t pin = info->pins->data[ii];
        if ((pin < 16) || (pin > 23)) {
            LOG("!!! ROM line D%d using invalid pin %d", ii, pin);
        }
    }

    // Check X1/X2 pins
    if (set->rom_count > 1) {
        if (info->pins->x1 > 15) {
            LOG("!!! Multi-ROM mode, but pin X1 invalid");
        }
        if (info->pins->x2 > 15) {
            LOG("!!! Multi-ROM mode, but pin X2 invalid");
        }
        if (info->pins->x1 == info->pins->x2) {
            LOG("!!! Multi-ROM mode, but pin X1=X2");
        }
        if (info->pins->x_jumper_pull > 1) {
            LOG("!!! X jumper pull value invalid");
        }
    }

    // Check CS pins
    if (info->pins->cs1_2364 > 15) {
        LOG("!!! CS1 pin for 2364 ROM invalid");
    }
    if (info->pins->cs1_2332 > 15) {
        LOG("!!! CS1 pin for 2332 ROM invalid");
    }
    if (info->pins->cs1_2316 > 15) {
        LOG("!!! CS1 pin for 2316 ROM invalid");
    }
    if (info->pins->cs2_2332 > 15) {
        LOG("!!! CS2 pin for 2332 ROM invalid");
    }
    if (info->pins->cs2_2316 > 15) {
        LOG("!!! CS2 pin for 2316 ROM invalid");
    }
    if (info->pins->cs3_2316 > 15) {
        LOG("!!! CS3 pin for 2316 ROM invalid");
    }

    // Check sel jumper pull value
    if (info->pins->sel_jumper_pull > 1) {
        LOG("!!! Sel jumper pull value invalid");
    }

    // Warn if serve mode is incorrectly set for multiple ROM images
    if ((set->rom_count == 1) && (set->serve == SERVE_ADDR_ON_ANY_CS)) {
        // Correction is done in main_loop() using a local variable
        LOG("!!! Single ROM image - wrong serve mode - will correct");
    }

}

void platform_logging(void) {
#if defined(BOOT_LOGGING)
    LOG("%s", log_divider);
    LOG("Detected hardware info ...");

    // Reset the SysInfo registers
    RESET_RESET &= ~RESET_SYSINFO;

    // Output hardware information
    LOG("MCU: RP235X");
    LOG("Chip ID: 0x%08X", SYSINFO_CHIP_ID);
    char *package;
    if (SYSINFO_PACKAGE_SEL & 0b1) {
        package = "QFN60";
    } else {
        package = "QFN80";
    }
    LOG("Package: %s", package);
    LOG("Chip gitref: 0x%08X", SYSINFO_GITREF_RP2350);
    LOG("Running on core: %d", SIO_CPUID);
    LOG("PCB rev %s", sdrr_info.hw_rev);
    LOG("Firmware configured flash size: %dKB", MCU_FLASH_SIZE_KB);
    if ((MCU_RAM_SIZE_KB != RP2350_RAM_SIZE_KB) || (MCU_RAM_SIZE != RP2350_RAM_SIZE_KB * 1024)) {
        LOG("!!! RAM size mismatch: actual %dKB (%d bytes), firmware expected: %dKB (%d bytes)", MCU_RAM_SIZE_KB, MCU_RAM_SIZE, RP2350_RAM_SIZE_KB, RP2350_RAM_SIZE_KB * 1024);
    } else {
        LOG("Firmware configured RAM size: %dKB (default)", MCU_RAM_SIZE_KB);
    }
    LOG("Flash configured RAM: %dKB (%d bytes)", MCU_RAM_SIZE_KB, MCU_RAM_SIZE);

    LOG("Target freq: %dMHz", TARGET_FREQ_MHZ);
    LOG("PLL values: %d/%d/%d/%d (refdiv/fbdiv/postdiv1/postdiv2)", PLL_SYS_REFDIV, PLL_SYS_FBDIV, PLL_SYS_POSTDIV1, PLL_SYS_POSTDIV2);

#endif // BOOT_LOGGING
}

void setup_xosc(void) {
    // Initialize XOSC peripheral.  We are using the 12MHz xtal from the
    // reference hardware design, so we can use values from the datasheet.
    // See S8.2 for more details.
    //
    // Specifically:
    // - Set the startup delay to 1ms
    // - Enable the XOSC giving it the appropriate frequency range (1-15MHz)
    // - Wait for the XOSC to be enabled and stable
    XOSC_STARTUP = 47;
    XOSC_CTRL = XOSC_ENABLE | XOSC_RANGE_1_15MHz;
    while (!(XOSC_STATUS & XOSC_STATUS_STABLE));
    LOG("XOSC enabled and stable");

    // Switch CLK_REF to use XOSC instead of the ROSC
    CLOCK_REF_CTRL = CLOCK_REF_SRC_XOSC;
    while ((CLOCK_REF_SELECTED & CLOCK_REF_SRC_SEL_XOSC) != CLOCK_REF_SRC_SEL_XOSC);
}
