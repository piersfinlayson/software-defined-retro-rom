// Contains assembly routines for sdrr-tester, for performance critical
// sections

.syntax unified
.cpu cortex-m0plus
.thumb

// Activates the CS line, times how long it takes for D7 to go low (how it
// infers the start of the data transfer), and returns:
// - start_time
// - data_in - the data byte
// - end_time
.global activate_cs_asm
.type activate_cs_asm, %function
.section .text.activate_cs_asm
activate_cs_asm:
    // Parameters:
    // r0 = cs_gpio_reg
    // r1 = cs_mask
    // r2 = timer_reg
    // r3 = gpio_in_reg
    // [sp] = d7_mask
    // [sp+4] = result_ptr
    
    push {r4, r5, lr}
    ldr r4, [sp, #12]        // Load d7_mask
    
    // Critical timing section start

    // Activate the CS line
    str r1, [r0]             // Activate CS line

    // Read the timer register to get the start time
    ldr r5, [r2]             // r5 = start time (preserve)
    
    // Unrolled D7 checking - 15 iterations, 4 cycles/32ns each = ~480ns
    // The SDRR should have made data avaialable within this time frame
    .rept 15
    ldr r1, [r3]             // Read GPIO_IN
    tst r1, r4               // Test D7 bit
    beq .done_as             // Exit if D7 low
    .endr
    
    // Fallback loop - If the previous look didn't find D7 low, this will
    // although it will be less accurate as the bne taken takes 3 cycles,
    // rather than the above 1 cycle for beq not taken
1:
    ldr r1, [r3]             // Read GPIO_IN
    tst r1, r4               // Test D7 bit
    bne 1b                   // Loop if high
    
.done_as:
    // D7 is low, so immediately read the time, before storing off the data
    ldr r0, [r2]             // r0 = end time

    // Critical timing section end

    // Now store the results - in the struct pointed to by result_ptr
    ldr r2, [sp, #16]        // Load result_ptr
    str r5, [r2, #0]         // store start_time
    str r1, [r2, #4]         // store data_in
    str r0, [r2, #8]         // store end_time
    
    movs r0, #0               // Return success
    pop {r4, r5, pc}

// De-activates the CS line and times how long it takes for D7 to go high (how
// it infers SDRR has de-asserted the data bus lines) and returns:
// - start_time
// - end_time
.global deactivate_cs_asm
.type deactivate_cs_asm, %function
.section .text.deactivate_cs_asm
deactivate_cs_asm:
    // Parameters:
    // r0 = cs_gpio_reg
    // r1 = cs_mask
    // r2 = timer_reg
    // r3 = gpio_in_reg
    // [sp] = d7_mask
    // [sp+4] = result_ptr
    
    push {r4, r5, lr}
    ldr r4, [sp, #12]        // Load d7_mask

    // Critical timing section start

    // De-activate the CS line
    str r1, [r0]             // De-activate CS line

    // Read the timer register to get the start time
    ldr r5, [r2]             // r5 = start time (preserve)

    // Unrolled D7 checking - 3 iterations, 4 cycles/32ns each = ~100ns
    // The SDRR should have release data within this time frame
    .rept 3
    ldr r1, [r3]             // Read GPIO_IN
    tst r1, r4               // Test D7 bit
    bne .done_de                // Exit if D7 high
    .endr

    // Fallback loop - If the previous look didn't find D7 high, this will
    // although it will be less accurate as the bne taken takes 3 cycles,
    // rather than the above 1 cycle for beq not taken
1:
    ldr r1, [r3]             // Read GPIO_IN
    tst r1, r4               // Test D7 bit
    beq 1b                   // Loop if low

.done_de:
    ldr r0, [r2]             // D7 high, get end time

    // Critical timing section end

    // Now store the results - in the struct pointed to by result_ptr
    ldr r2, [sp, #16]        // Load result_ptr
    str r5, [r2, #0]         // store start_time
    str r0, [r2, #4]         // store end_time

    movs r0, #0               // Return success
    pop {r4, r5, pc}
