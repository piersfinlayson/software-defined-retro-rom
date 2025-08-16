// One ROM RP235X Specific Routines

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#include "include.h"
#include "roms.h"

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
}

void setup_clock(void) {
    LOG("Setting up clock");

    setup_xosc();
    setup_pll();
}

void setup_gpio(void) {
    LOG("Setting up GPIO");

    // Take IO bank and pads bank out of reset
    RESET_RESET &= ~(RESET_IOBANK0 | RESET_PADS_BANK0);
    while (!(RESET_DONE & (RESET_IOBANK0 | RESET_PADS_BANK0)));
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

// On all RP2350 boards, the SEL pins are pulled low by jumpers to indicate
// a 1, so reverse to the default STM32F4 behavior.
uint32_t check_sel_pins(uint32_t *sel_mask) {
    // Ensure GPIO port is enabled
    RESET_RESET &= ~(RESET_IOBANK0 | RESET_PADS_BANK0);
    while (!(RESET_DONE & (RESET_IOBANK0 | RESET_PADS_BANK0)));

    uint32_t value;
    *sel_mask = 0;

    // Setup the pins first.  Do this first to allow any pull-ups to settle
    // before reading.
    for (int ii = 0; ((ii < MAX_IMG_SEL_PINS) && (sdrr_info.pins->sel[ii] < 255)); ii++) {
        uint8_t pin = sdrr_info.pins->sel[ii];
        if (pin < 32) {
            // Set the pin as SIO function - clear everything else
            GPIO_CTRL(pin) = GPIO_FUNC_SIO;

            // Enable as input and pull-up
            GPIO_PAD(pin) = PAD_INPUT_PU;

            // Set the pin in our bit mask
            *sel_mask |= (1 << pin);
        } else {
            LOG("!!! Sel pin %d > 31 - not using", pin);
        }
    }

    // Read the pins
    LOG("Reading SEL pins: mask 0x%08X", *sel_mask);
    LOG("Reading SIO_GPIO_IN: 0x%08X", SIO_GPIO_IN);
    value = SIO_GPIO_IN & *sel_mask;

    return value;
}

void setup_status_led(void) {
    LOG("!!! Status LED not supported on RP235X");
}

void blink_pattern(uint32_t on_time, uint32_t off_time, uint8_t repeats) {
    (void)on_time;
    (void)off_time;
    (void)repeats;
    LOG("!!! Blink pattern not supported on RP235X");
}

void platform_logging(void) {
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
        LOG("Firmware configured RAM size: %dKB (default)");
    }
    LOG("Flash configured RAM: %dKB (%d bytes)", MCU_RAM_SIZE_KB, MCU_RAM_SIZE);

    LOG("Target freq: %dMHz", TARGET_FREQ_MHZ);
    #define PLL_SYS_REFDIV    1
#define PLL_SYS_FBDIV     50
#define PLL_SYS_POSTDIV1  4
#define PLL_SYS_POSTDIV2  1
    LOG("PLL values: %d/%d/%d/%d (refdiv/fbdiv/postdiv1/postdiv2)", PLL_SYS_REFDIV, PLL_SYS_FBDIV, PLL_SYS_POSTDIV1, PLL_SYS_POSTDIV2);
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
