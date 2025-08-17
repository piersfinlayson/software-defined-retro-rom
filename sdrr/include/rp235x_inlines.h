// One ROM STM32F4 Specific Routines

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#ifndef STM32F4_INLINES_H
#define STM32F4_INLINES_H

static inline void __attribute__((always_inline)) status_led_on(uint8_t pin) {
    // Set to 0 to turn on
    SIO_GPIO_OUT_CLR = (1 << pin);
}

static inline void __attribute__((always_inline)) status_led_off(uint8_t pin) {
    // Set to 1 to turn on
    SIO_GPIO_OUT_SET = (1 << pin);
}

#endif // STM32F4_INLINES_H
