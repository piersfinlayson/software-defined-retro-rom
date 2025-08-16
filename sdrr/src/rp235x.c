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
    // RTT loggin)
    RESET_RESET &= ~RESET_JTAG;
    while (!(RESET_DONE & RESET_JTAG));
}

void setup_clock(void) {
    LOG("Setting up clock for RP235X");

    setup_xosc();
    setup_pll();

    LOG("RP235X clock set up complete");
}

void setup_gpio(void) {
    LOG("!!! RP235x GPIO setup not implemented");
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

uint32_t check_sel_pins(uint32_t *sel_mask) {
    // RP235X does not have SEL pins, so we return 0
    LOG("!!! SEL pins not supported on RP235X");
    *sel_mask = 0;
    return 0;
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
    LOG("MCU: RP235X");
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
