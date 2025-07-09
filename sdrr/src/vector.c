// Vector table and reset handler.

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#include "include.h"

// Forward declarations
void Reset_Handler(void);
void Default_Handler(void);
void NMI_Handler(void);
void HardFault_Handler(void);
void BusFault_Handler(void);
void UsageFault_Handler(void);

// Default exception/interrupt handlers
#define MemManage_Handler   Default_Handler
#define SVC_Handler         Default_Handler
#define DebugMon_Handler    Default_Handler
#define PendSV_Handler      Default_Handler
#define SysTick_Handler     Default_Handler

// Declare stack section
extern uint32_t _estack;

// Vector table - must be placed at the start of flash
__attribute__ ((section(".isr_vector"), used))
void (* const g_pfnVectors[])(void) = {
    (void (*)(void))&_estack,      // Initial stack pointer
    Reset_Handler,                 // Reset handler
    NMI_Handler,                   // NMI handler
    HardFault_Handler,             // Hard fault handler
    MemManage_Handler,             // MPU fault handler
    BusFault_Handler,              // Bus fault handler
    UsageFault_Handler,            // Usage fault handler
    0, 0, 0, 0,                    // Reserved
    SVC_Handler,                   // SVCall handler
    DebugMon_Handler,              // Debug monitor handler
    0,                             // Reserved
    PendSV_Handler,                // PendSV handler
    SysTick_Handler,               // SysTick handler

    // Peripheral interrupt handlers follow
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    // Different STM32F4s have different numbers of interrupts.  The maximum
    // appears to be 96 (F446), which is what's included here.  This means
    // that 0x080001C4 onwards is free, but we'll not use anything until
    // 0x08000200 to be safe.
};

//
// Variables defined by the linker.
//
// Note these are "labels" that mark memory addresses, not variables that
// store data.  The address of the label IS the address we're interested in.
// Hence we use & below to get the addresses that these labels represent.
extern uint32_t _sidata;    // Start of .data section in FLASH
extern uint32_t _sdata;     // Start of .data section in RAM
extern uint32_t _edata;     // End of .data section in RAM
extern uint32_t _sbss;      // Start of .bss section in RAM
extern uint32_t _ebss;      // End of .bss section in RAM

// Location of RAM reserved for executing the main loop function from, if
// EXECUTE_FROM_RAM is defined.
#if defined(EXECUTE_FROM_RAM)
extern uint32_t _main_loop_start;   // Start of .main_loop section in FLASH
extern uint32_t _main_loop_end;     // End of .main_loop section in FLASH
extern uint32_t _ram_func_start;    // Start of .ram_func section in RAM
extern uint32_t _ram_func_end;      // End of .ram_func section in RAM
#endif

// Reset handler
void Reset_Handler(void) {
    // We use memcpy and memset because it's likely to be faster than anything
    // we could come up with.

    // Copy data section from flash to RAM
    memcpy(&_sdata, &_sidata, (unsigned int)((char*)&_edata - (char*)&_sdata));
    
    // Zero out bss section  
    memset(&_sbss, 0, (unsigned int)((char*)&_ebss - (char*)&_sbss));
    
#if defined(EXECUTE_FROM_RAM)
    // Copy main_loop function into RAM.
    uint32_t code_size = (unsigned int)((char*)&_main_loop_end - (char*)&_main_loop_start);
#if defined(BOOT_LOGGING)
    if (code_size > (unsigned int)((char*)&_ram_func_end - (char*)&_ram_func_start)) {
        LOG("!!! Code size too large for RAM function area");
    }
#endif // BOOT_LOGGING
    copy_func_to_ram((void (*)(void))(&_main_loop_start), (uint32_t)&_ram_func_start, code_size);
#endif // EXECUTE_FROM_RAM

    // Call the main function
    main();
    
    // In case main returns
    while(1);
}

// Default handler for unhandled interrupts - fast continuous blink
void Default_Handler(void) {
    if (sdrr_info.status_led_enabled) {
        setup_status_led();
        
        while(1) {
            GPIOB_BSRR = (1 << (15 + 16)); // LED on (PB15 low)
            delay(50000);
            GPIOB_BSRR = (1 << 15);        // LED off (PB15 high)  
            delay(50000);
        }
    }
}

// NMI_Handler - single blink pattern
void NMI_Handler(void) {
    setup_status_led();
    
    while(1) {
        blink_pattern(100000, 500000, 1); // Single blink
        delay(1000000); // Long pause
    }
}

// HardFault_Handler - double blink pattern
void HardFault_Handler(void) {
    setup_status_led();
    
    while(1) {
        blink_pattern(100000, 200000, 2); // Double blink
        delay(1000000); // Long pause
    }
}

// BusFault_Handler - triple blink pattern
void BusFault_Handler(void) {
    setup_status_led();
    
    while(1) {
        blink_pattern(100000, 200000, 3); // Triple blink
        delay(1000000); // Long pause
    }
}

// UsageFault_Handler - quadruple blink pattern
void UsageFault_Handler(void) {
    setup_status_led();
    
    while(1) {
        blink_pattern(100000, 200000, 4); // Quadruple blink
        delay(1000000); // Long pause
    }
}
