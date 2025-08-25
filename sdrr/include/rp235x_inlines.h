// One ROM STM32F4 Specific Routines

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#ifndef STM32F4_INLINES_H
#define STM32F4_INLINES_H

// Does any preparatory GPIO setup within main_loop()
//
// Inlined as used by main_loop (which may be in RAM)
static inline void __attribute__((always_inline)) main_loop_gpio_init() {
    // No-op
}

// Sets pull-ups or pull-downs on the X1/X2 pins
//
// Inlined as used by main_loop (which may be in RAM)
static inline void __attribute__((always_inline)) set_x_pulls(
    const sdrr_pins_t *pins,
    uint8_t x1_pull,
    uint8_t x2_pull
) {
    if (pins->x1 < MAX_USED_GPIOS) {
        if (x1_pull) {
            GPIO_PAD(pins->x1) = PAD_INPUT_PU;
        } else {
            GPIO_PAD(pins->x1) = PAD_INPUT_PD;
        }
    }

    if (pins->x2 < MAX_USED_GPIOS) {
        if (x2_pull) {
            GPIO_PAD(pins->x2) = PAD_INPUT_PU;
        } else {
            GPIO_PAD(pins->x2) = PAD_INPUT_PD;
        }
    }
}

// Set up the masks to turn data into an input/output
//
// Inlined as used by main_loop (which may be in RAM)
static inline void __attribute__((always_inline)) setup_data_masks(
    const sdrr_info_t *info,
    uint32_t *data_output_mask_val,
    uint32_t *data_input_mask_val
) {
    // We use GPIO_OE register to set ines as inputs/outputs.  We are
    // therefore controlling all GPIOs at once on the RP235x.  THis means we
    // need to correctly handle the status LED, if used.
    *data_output_mask_val = 0xFF << 16;
    *data_input_mask_val = 0;

    if ((info->status_led_enabled) && (info->pins->status <= MAX_USED_GPIOS)) {
        uint8_t pin = info->pins->status;
        *data_output_mask_val |= (1 << pin);
        *data_input_mask_val |= (1 << pin);
    }
}

// Inlined as may be used by main_loop (which may be in RAM)
static inline void __attribute__((always_inline)) status_led_on(uint8_t pin) {
    // Set to 0 to turn on
    SIO_GPIO_OUT_CLR = (1 << pin);
}

// Inlined as may be used by main_loop (which may be in RAM)
static inline void __attribute__((always_inline)) status_led_off(uint8_t pin) {
    // Set to 1 to turn on
    SIO_GPIO_OUT_SET = (1 << pin);
}

#endif // STM32F4_INLINES_H
