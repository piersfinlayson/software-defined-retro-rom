// Contains SDRR/STM32 test routine

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#include "include.h"

#if defined(TIMER_TEST) || defined(TOGGLE_PA4)

// Log function which can be called from functions potentially run from RAM
#if defined(MAIN_LOOP_LOGGING)
ram_log_fn TEST_LOG = do_log;
#else // !MAIN_LOOP_LOGGING
#define TEST_LOG(X, ...)
#endif

#if defined(TIMER_TEST)
void __attribute__((section(".main_loop"), used)) main_loop(void) {
    while (1) {
        TEST_LOG("Running timer aiming for %ds", TIMER_TEST);
        uint32_t iterations = TIMER_TEST * TARGET_FREQ / 3;
        asm volatile (
            "mov r0, %0          \n"  // Load iteration count from C variable
            "1:                  \n"  // Loop label
            "subs r0, #1         \n"  // Decrement counter
            "bne 1b              \n"  // Branch if not zero
            : : "r" (iterations) : "r0", "cc", "memory"
        );
    }
}
#endif // TIMER_TEST

#if defined(TOGGLE_PA4)
// Fast toggle loop
void __attribute__((section(".main_loop"), used)) main_loop(void) {
    TEST_LOG("Toggling PA4");

    // Configure PA9 as output push-pull, 50MHz
    GPIOA_CRL &= ~(0xF << 16); // Clear configuration for PA4
    GPIOA_CRL |= (0x3 << 16);  // Output mode, 50MHz speed

    __asm volatile (
        "mov r0, %0\n"        // Load GPIOA_BSRR address
        "mov r1, #(1 << 4)\n" // Bit for PA4 set
        "mov r2, #(1 << (16+4))\n" // Bit for PA4 reset
        "1:\n"                // Loop label
        "str r1, [r0]\n"      // Set PA9
        "str r2, [r0]\n"      // Reset PA9
        "b 1b\n"              // Branch back to loop
        :: "r" (GPIOA_BASE + GPIO_BSRR_OFFSET) : "r0", "r1", "r2", "memory"
    );
}
#endif // TOGGLE_PA4

#endif // TIMER_TEST || TOGGLE_PA4
