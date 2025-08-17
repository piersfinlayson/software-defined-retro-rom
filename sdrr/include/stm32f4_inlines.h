// One ROM STM32F4 Specific Routines

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#ifndef STM32F4_INLINES_H
#define STM32F4_INLINES_H

static inline void __attribute__((always_inline)) status_led_on(uint8_t pin) {
    GPIOB_BSRR = (1 << (pin + 16));
}

static inline void __attribute__((always_inline)) status_led_off(uint8_t pin) {
    GPIOB_BSRR = (1 << pin);
}

#endif // STM32F4_INLINES_H
