// One ROM STM32F4 Specific Inlined Routines

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#ifndef STM32F4_INLINES_H
#define STM32F4_INLINES_H

// Does any preparatory GPIO setup within main_loop()
//
// Inlined as used by main_loop (which may be in RAM)
static inline void __attribute__((always_inline)) main_loop_gpio_init() {
    // Enable GPIO clocks for the ports with address and data lines
    RCC_AHB1ENR |= (RCC_AHB1ENR_GPIOAEN | RCC_AHB1ENR_GPIOCEN);
    
    // Configure PA0-7 as inputs initially (00 in MODER), no pull-up/down
    // Also PA10-12 are duplicate CS lines on some hw so set as inputs no
    // PU/PD.
    // We could theoretically check here that D0-7 uses PA0-7, but there's
    // checks like this earlier, and in sdrr-gen, so little point.  It's
    // required that they use 0-7 on a port, to avoid any bit shifting when
    // applying the value.
    GPIOA_MODER &= ~0x00FCFFFF; // Clear bits 0-15 (PA0-7, 10-12 as inputs)
    GPIOA_PUPDR &= ~0x00FCFFFF; // Clear pull-up/down for PA0-7, 10-12)
    GPIOA_OSPEEDR &= ~0xFFFF;   // Clear output speed for PA0-7
    GPIOA_OSPEEDR |= 0xAAAA;    // Set PA0-7 speed to "fast", not "high" to
                                // ensure V(OL) is max 0.4V

    // Port C for address and CS lines - set all pins as inputs
    GPIOC_MODER = 0;  // Set all pins as inputs
}

// Sets pull-ups or pull-downs on the X1/X2 pins
//
// Inlined as used by main_loop (which may be in RAM)
static inline void __attribute__((always_inline)) set_x_pulls(
    const sdrr_pins_t *pins,
    uint8_t x1_pull,
    uint8_t x2_pull
) {
    uint32_t gpioc_pupdr = GPIOC_PUPDR;
    uint8_t x1_pin;
    uint8_t x2_pin;

    // If this STM32F4 board doesn't have any X pins configured, assume X1 and
    // X2 are at 14/15 - this is important so we only address the 16KB image in
    // the single ROM case (all a board with no X pins suppports)
    if (pins->x1 >= MAX_USED_GPIOS) {
        x1_pin = 14;
    } else {
        x1_pin = pins->x1;
    }
    if (pins->x2 >= MAX_USED_GPIOS) {
        x2_pin = 15;
    } else {
        x2_pin = pins->x2;
    }

    gpioc_pupdr &= ~(GPIO_PULL_MASK << (x1_pin * 2));
    if (x1_pull) {
        gpioc_pupdr |= (GPIO_PU_BITS << (x1_pin * 2));
    } else {
        gpioc_pupdr |= (GPIO_PD_BITS << (x1_pin * 2));
    }

    gpioc_pupdr &= ~(GPIO_PULL_MASK << (x2_pin * 2));
    if (x2_pull) {
        gpioc_pupdr |= (GPIO_PU_BITS << (x2_pin * 2));
    } else {
        gpioc_pupdr |= (GPIO_PD_BITS << (x2_pin * 2));
    }

    GPIOC_PUPDR = gpioc_pupdr;
}

// Set up the masks to turn data into an input/output
//
// Inlined as used by main_loop (which may be in RAM)
static inline void __attribute__((always_inline)) setup_data_masks(
    const sdrr_info_t *info,
    uint32_t *data_output_mask_val,
    uint32_t *data_input_mask_val
) {
    if (info->mco_enabled) {
        // PA8 is AF, PA0-7 are inputs
        *data_output_mask_val = 0x00025555;
        *data_input_mask_val = 0x00020000;
    } else {
        // PA0-7 are inputs
        *data_output_mask_val = 0x00005555;
        *data_input_mask_val = 0x00000000;
    }

    if (info->swd_enabled) {
        // Ensure PA13/14 remain AF (SWD enabled)
        *data_output_mask_val |= 0x28000000;
        *data_input_mask_val |= 0x28000000;
    }
}

// Inlined as may be used by main_loop (which may be in RAM)
static inline void __attribute__((always_inline)) status_led_on(uint8_t pin) {
    GPIOB_BSRR = (1 << (pin + 16));
}

// Inlined as may be used by main_loop (which may be in RAM)
static inline void __attribute__((always_inline)) status_led_off(uint8_t pin) {
    GPIOB_BSRR = (1 << pin);
}

#endif // STM32F4_INLINES_H
