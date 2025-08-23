// 2332/2364 ROM implementation.

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

// This file contains the STM32F4 Cortex-M4 hand-crafted assembly that
// implements the main serving bytes loop that emulates the ROM(s).
//
// It is highly optimised for speed and aims to exceed (i.e. beat) the 300ns
// access time of the fastest 2332/2364 ROMs.  (Slower ROMs supported 350ns and
// 450ns access times.)
//
// HW_REV_D with the STM32F411 at 100MHz is fast enough to replace C64 kernal
// basic, and character ROMs.  The character ROM is a 350ns ROM.
//
// This implementation achieves its aims by
// - running the STM32F4 at its fastest possible clock speed from the PLL
// - implementing the main loop in assembly (this module)
// - assigning data/CS pins to the same port, starting at pin 0, and data pins
//   contiguously from pin 0 on the same port
// - remapping "mangling" the addresses of the data bytes of the ROM images
//   stored on flash, and the data bytes themselves, to map the hardware pin
//   layout
// - preloading as much as possible into registers before the main work loop,
//   so subsequent operations within the loop take as few instructions as
//   possible
//
// See the various technical documents in [/docs](/docs) for more information.

//
// Defines for assembly routines
//

// CPU registers - we prefer low registers (r0-r7) to high registers as
// instructions using the low registers tend to be 16-bit vs 32-bit, which
// means they are fetched from flash (or RAM) faster. 
//
// Our high registers are _only_ used during eor, bics and tst instructions,
// which have to be 32 bit, anyway, so we might as well use high registers for
// them.  If we run out of registers we can combine registers by using a single
// register with GPIOA_BASE and then use immediate offsets from this for the
// data MODER and ODR registers, and the addr/CS IDR register.  However, the
// code will take up more space that way (but likely not be any slower/faster).
#define R_ADDR_CS       "r0"
#define R_DATA          "r1"
#define R_ROM_TABLE     "r2"
#define R_DATA_OUT_MASK "r3"
#define R_DATA_IN_MASK  "r6"
#define R_GPIO_DATA_MODER   "r7"
#define R_GPIO_DATA_ODR     "r5"
#define R_GPIO_ADDR_CS_IDR  "r4"
#define R_CS_INVERT_MASK    "r8"
#define R_CS_CHECK_MASK     "r9"
#define R_CS_TEST           "r10"
#if defined(COUNT_ROM_ACCESS)
#define R_AC_ADDR       "r11"
#define R_AC_COUNT      "r12"
#endif // COUNT_ROM_ACCESS

// Assembly Outputs, Input, and Clobbers
#if defined(MAIN_LOOP_ONE_SHOT)
#define ASM_OUTPUTS \
                "=r" (addr_cs), \
                "=r" (byte)
#else // !MAIN_LOOP_ONE_SHOT
#define ASM_OUTPUTS
#endif // MAIN_LOOP_ONE_SHOT
#if defined(COUNT_ROM_ACCESS)
#define ACCESS_COUNT_ASM_INPUT ,\
    "r" (r_access_count_addr), \
    "r" (r_access_count)
#else // !COUNT_ROM_ACCESS
#define ACCESS_COUNT_ASM_INPUT
#endif // COUNT_ROM_ACCESS
#define ASM_INPUTS \
    "r" (rom_table), \
    "r" (cs_invert_mask_reg), \
    "r" (cs_check_mask_reg), \
    "r" (addr_cs_idr), \
    "r" (data_odr), \
    "r" (data_moder), \
    "r" (data_output_mask), \
    "r" (data_input_mask) ACCESS_COUNT_ASM_INPUT
#define ASM_CLOBBERS R_ADDR_CS, R_DATA, R_CS_TEST, "cc", "memory"

//
// Assembly code macros
//

// Labels - i.e. a point to jump to in the assembly code
#define LABEL(X)        #X "%=: \n"

// Loads the address/CS lines from the hardware register into R_ADDR_CS
#define LOAD_ADDR_CS    "ldrh " R_ADDR_CS ", [" R_GPIO_ADDR_CS_IDR "]\n"

// Tests whether the CS line is active - zero flag set if so (low)
#define TEST_CS         "eor " R_CS_TEST ", " R_ADDR_CS ", " R_CS_INVERT_MASK "\n" \
                        "tst " R_CS_TEST ", " R_CS_CHECK_MASK "\n"

// Tests whether the CS line is active (active low only) - zero flag set if so.
// Saves a cycle over TEST_CS.
#define TEST_CS_ACT_LOW "tst " R_ADDR_CS ", " R_CS_CHECK_MASK "\n"

// Tests whether any of the CS lines are active - zero flash _not set_ if so.
// BIC is bit clear - essentially destination = source & ~mask.  Note we need
// BICS - where S indicates the N/Z flags should be updated, which we use
#define TEST_CS_ANY     "eor " R_CS_TEST ", " R_ADDR_CS ", " R_CS_INVERT_MASK "\n" \
                        "bics " R_CS_TEST ", " R_CS_CHECK_MASK ", " R_CS_TEST "\n"

// Tests whether any of the CS lines are active (active low only) - zero flag
// set if so.  Saves a cycle over TEST_CS_ANY.
#define TEST_CS_ANY_ACT_LOW \
                        "bics " R_CS_TEST ", " R_CS_CHECK_MASK ", " R_ADDR_CS "\n"

// Loads the data byte from the ROM table into R_DATA, based on the offset in
// R_ADDR_CS
#define LOAD_FROM_RAM   "ldrb " R_DATA ", [" R_ROM_TABLE ", " R_ADDR_CS "]\n"

// Stores the data byte from the CPU register to the data lines hardware output
// register
#if defined(STM32F4)
#define STORE_TO_DATA   "strb " R_DATA ", [" R_GPIO_DATA_ODR "]\n"
#elif defined(RP235X)
#define STORE_TO_DATA   "strb " R_DATA ", [" R_GPIO_DATA_ODR "]\n"
// Use coprocessor.  This is not used, even in the RP_USE_CP case, as it will
// always be less performance than a direct SIO write, due to the lsl overhead
//#define STORE_TO_DATA   "lsl " R_DATA ", " R_DATA ", #16\n"
//                        "mcr p0, #0, " R_DATA ", c0, c0\n"
#endif // STM32F4/RP235X

// Sets the data lines as outputs using the hardware register and pre-loaded
// mask
#if defined(STM32F4)
#define SET_DATA_OUT    "strh " R_DATA_OUT_MASK ", [" R_GPIO_DATA_MODER "]\n"
#elif defined(RP235X)
// This has to be a full word write, as the RP235X SIO_GPIO_OE register is a
// single 32-bit register that controls all GPIOs, and the data lines are
// pins 16-23, at least on rev A.
#if !defined(RP_USE_CP)
#define SET_DATA_OUT    "str  " R_DATA_OUT_MASK ", [" R_GPIO_DATA_MODER "]\n"
#else // RP_USE_CP
#define SET_DATA_OUT    "mcr p0, #0, " R_DATA_OUT_MASK ", c0, c4\n"
#endif // RP_USE_CP
#endif // STM32F4/RP235X

// Sets the data lines as inputs using the hardware register and pre-loaded
// mask
#if defined(STM32F4)
#define SET_DATA_IN     "strh " R_DATA_IN_MASK ", [" R_GPIO_DATA_MODER "]\n"
#elif defined(RP235X)
// This has to be a full word write, as the RP235X SIO_GPIO_OE register is a
// single 32-bit register that controls all GPIOs, and the data lines are
// pins 16-23, at least on rev A.
#if !defined(RP_USE_CP)
#define SET_DATA_IN     "str  " R_DATA_IN_MASK ", [" R_GPIO_DATA_MODER "]\n"
#else // RP_USE_CP
#define SET_DATA_IN     "mcr p0, #0, " R_DATA_IN_MASK ", c0, c4\n"
#endif // RP_USE_CP
#endif // STM32F4/RP235X

// Branches if zero flag set
#define BEQ(X)          "beq " #X"%=" "\n"

// Branches if zero flag not set
#define BNE(X)          "bne " #X"%=" "\n"

// Branches unconditionally
#define BRANCH(X)       "b " #X"%=" "\n"

#if defined(COUNT_ROM_ACCESS)
// Load the current access count value
#define LOAD_COUNT      "ldr " R_AC_COUNT ", [" R_AC_ADDR "]\n"

// Increment the access count value
#define INC_COUNT       "add " R_AC_COUNT ", " R_AC_COUNT ", #1\n"

// Store the access count value back to the address
#define STORE_COUNT     "str " R_AC_COUNT ", [" R_AC_ADDR "]\n"

#endif // COUNT_ROM_ACCESS

// Main loop end code - used to end the main loop in the one shot case
#if defined(MAIN_LOOP_ONE_SHOT)
#define MAIN_LOOP_ONE_SHOT_END_LABEL(X) \
    LABEL(X) \
        "mov %0, " R_ADDR_CS "\n" \
        "mov %1, " R_DATA "\n"
#else // !MAIN_LOOP_ONE_SHOT
#define MAIN_LOOP_ONE_SHOT_END_LABEL(X)
#endif // MAIN_LOOP_ONE_SHOT

// Main loop end branch - jump to code to end the main loop in the one shot
// case
#if defined(MAIN_LOOP_ONE_SHOT)
#define MAIN_LOOP_ONE_SHOT_END_BRANCH(X) \
    BRANCH(X)
#else // !MAIN_LOOP_ONE_SHOT
#define MAIN_LOOP_ONE_SHOT_END_BRANCH(X)
#endif // MAIN_LOOP_ONE_SHOT

// Set up the GPIO hardware registers used by the assembly
#if defined(STM32F4)
// Addr and CS lines input data register - port C
#define VAL_ADDR_CS_IDR  VAL_GPIOC_IDR
// Data write register, port A 
#define VAL_DATA_ODR     VAL_GPIOA_ODR
// Data input/output mode register, port A 
#define VAL_DATA_MODER   VAL_GPIOA_MODER
#elif defined(RP235X)
// Addr and CS lines input data register - use GPIO_IN - will read bottom 16 bits
#define VAL_ADDR_CS_IDR  VAL_SIO_GPIO_IN
// Data write register, we will write a byte, will be replicated across word.
// This will make the status LED flicker!
#define VAL_DATA_ODR     VAL_SIO_GPIO_OUT
// Data input/output mode register, affects all inputs, but we use a mask
#define VAL_DATA_MODER   VAL_SIO_GPIO_OE
#endif // STM32F4/ RP235X

// Pre-load registers.
//
// This is a macro so it's easy to put it just before each ASM block.
#if defined(COUNT_ROM_ACCESS)
#define PRELOAD_ACCESS_COUNT  ; \
    register uint32_t r_access_count_addr asm(R_AC_ADDR) = access_count_addr; \
    register uint32_t r_access_count asm(R_AC_COUNT) = access_count
#else // !COUNT_ROM_ACCESS
#define PRELOAD_ACCESS_COUNT
#endif // COUNT_ROM_ACCESS
#define PRELOAD_ASM_REGISTERS \
    register uint32_t data_output_mask asm(R_DATA_OUT_MASK) = data_output_mask_val; \
    register uint32_t data_input_mask asm(R_DATA_IN_MASK) = data_input_mask_val; \
    register uint32_t cs_invert_mask_reg asm(R_CS_INVERT_MASK) = cs_invert_mask; \
    register uint32_t cs_check_mask_reg asm(R_CS_CHECK_MASK) = cs_check_mask; \
    register uint32_t addr_cs_idr asm(R_GPIO_ADDR_CS_IDR) = VAL_ADDR_CS_IDR; \
    register uint32_t data_odr asm(R_GPIO_DATA_ODR) = VAL_DATA_ODR; \
    register uint32_t data_moder asm(R_GPIO_DATA_MODER) = VAL_DATA_MODER; \
    register uint32_t rom_table asm(R_ROM_TABLE) = rom_table_val PRELOAD_ACCESS_COUNT

// This is the default algorith for serve a single ROM image. It:
// - tests the CS line(s) twice as often as the data byte is loaded from RAM
// - loads the data byte to the address lines even when CS is not active (but
//   with the data lines set as inputs).
//
// The targets (from the MOS 2364 data sheet Feburary 1980) are:
// - tCO - set data line as outputs after CS activates - 200ns
// - tDF - set data line as inputs after CS deactivates - 175ns
// - tOH - data lines remain valid after address lines change - 40ns
// - tACC - maximum time from address to data valid - 450ns
//
// tACC and tCO are the most important - together they mean:
// - we have 450ns from the address lines being set to there being valid data
//   on the data lines (and therefore them being outputs)
// - we also have 200ns from CS activating to the data lines being set as
//   outputs and having valid data on them
//
// They are not cumulative - that is, we cannot assume address lines will be
// valid for tACC-tCO = 250ns before CS activates.  However, we can rely on
// both timings - that is that we will get at least 450ns from address lines
// being set AND 200ns from CS activating.
//
// Together these timings mean we have around twice as long to get the data
// lines loaded with the byte indicated by the address lines as we do to get
// the data lines set as outputs (450ns vs 200ns).  Hence we need to be
// checking the cs line around twice as often as we are checking the address
// lines.  This is reflected in the code below.
//
// They also mean we have to keep checking the address lines and updating the
// data lines while chip select is active. 
//
// We also have to get the data lines back to inputs quickly enough after CS
// deactivates - this is tDF, which is 175ns.  This is actually slightly
// tighter than tCO, but the processing between both CS inactive and active
// cases are very similar we are looking at a worse case of around 150ns with
// the code below on a 100MHz STM32F411.
//
// Finally, we have to make sure we don't change the data lines until at least
// tOH (40ns) after the address lines change.  This should be fine, because
// loading the byte from RAM and applying it to the data lines should always
// take longer than this but it's possible a faster STM32F4 could break it, in
// which case we'd need to add an instruction to two.
//
// Args:
// - TEST_CS_OPTION: The test to use to check if CS has gone active again.
//   - TEST_CS          - Used for serving a single ROM, where the CS line is
//                        active high.
//   - TEST_CS_ACT_LOW  - Used for serving a single ROM, where the CS line is
//                        active low.  This is 1 cycle faster than TEST_CS.
#define ALG1_ASM(TEST_CS_OPTION)                                            \
    PRELOAD_ASM_REGISTERS;                                                  \
    __asm volatile (                                                        \
        /* Start with a branch to the main loop - this is necessary so */   \
        /* that while running and CS goes from active to inactive, the */   \
        /* code can run straight into the main loop without a branch, */    \
        /* speeding up the golden path. */                                  \
        BRANCH(LOOP)                                                        \
                                                                            \
        /* Chip select went active - immediately set data lines as */       \
        /* outputs */                                                       \
    LABEL(CS_ACTIVE)                                                        \
        SET_DATA_OUT                                                        \
                                                                            \
        /* By definition we just loaded and tested the address/CS lines */  \
        /* so we have a valid address - so let's load the byte from RAM */  \
        /* and apply it, in amongst testing whether CS gone inactive */     \
        /* again. */                                                        \
    LABEL(CS_ACTIVE_DATA_ACTIVE)                                            \
        LOAD_FROM_RAM                                                       \
        LOAD_ADDR_CS                                                        \
        TEST_CS                                                             \
        BNE(CS_INACTIVE_BYTE)                                               \
    LABEL(CS_ACTIVE_DATA_ACTIVE_BYTE)                                       \
        STORE_TO_DATA                                                       \
        LOAD_ADDR_CS                                                        \
        TEST_CS                                                             \
        BNE(CS_INACTIVE_NO_BYTE)                                            \
        LOAD_FROM_RAM                                                       \
        LOAD_ADDR_CS                                                        \
        TEST_CS                                                             \
        BEQ(CS_ACTIVE_DATA_ACTIVE_BYTE)                                     \
        /* Fall through to CS_INACTIVE_BYTE if NE */                        \
                                                                            \
        /* CS went inactive.  We need to set the data lines as inputs, */   \
        /* but also want to update the data using the address lines we */   \
        /* have in our hands. */                                            \
    LABEL(CS_INACTIVE_BYTE)                                                 \
        SET_DATA_IN                                                         \
        STORE_TO_DATA                                                       \
    MAIN_LOOP_ONE_SHOT_END_BRANCH(END_LOOP)                                 \
                                                                            \
        /* Start of main processing loop.  Load the data byte while */      \
        /* constantly checking if CS has gone active */                     \
    LABEL(LOOP)                                                             \
        LOAD_ADDR_CS                                                        \
        TEST_CS                                                             \
        BEQ(CS_ACTIVE)                                                      \
        LOAD_FROM_RAM                                                       \
        LOAD_ADDR_CS                                                        \
        STORE_TO_DATA                                                       \
        TEST_CS                                                             \
        BEQ(CS_ACTIVE)                                                      \
        BRANCH(LOOP)                                                        \
                                                                            \
        /* CS went inactive, but we don't have a new byte to load. */       \
    LABEL(CS_INACTIVE_NO_BYTE)                                              \
        SET_DATA_IN                                                         \
        MAIN_LOOP_ONE_SHOT_END_BRANCH(END_LOOP)                             \
                                                                            \
        /* Copy of main processing loop - to avoid a branch in this or */   \
        /* CS_INACTIVE_BYTE case */                                         \
        LOAD_ADDR_CS                                                        \
        TEST_CS                                                             \
        BEQ(CS_ACTIVE)                                                      \
        LOAD_FROM_RAM                                                       \
        LOAD_ADDR_CS                                                        \
        STORE_TO_DATA                                                       \
        TEST_CS                                                             \
        BEQ(CS_ACTIVE)                                                      \
                                                                            \
        /* Start again - and might as well branch to the start of the */    \
        /* first loop as opposed to this copy. */                           \
        BRANCH(LOOP)                                                        \
                                                                            \
    MAIN_LOOP_ONE_SHOT_END_LABEL(END_LOOP)                                  \
                                                                            \
        : ASM_OUTPUTS                                                       \
        : ASM_INPUTS                                                        \
        : ASM_CLOBBERS                                                      \
    )

// This algorithm doesn't read the address lines until CS has gone active.
// This appears to be more performant on some machines (C64 character ROM),
// and less performant on others (VIC-20 kernal/BASIC ROMs).  Performant means
// the host runs stably with the STM32F4 clocked at a slower speed.
//
// A variant of this algorithm is used as ALG3, which is used to test whether
// _any_ CS line has gone active, when serving multi-ROM sets.  In that case,
// the code cannot know what address to lookup until the appropriate line has
// gone active.
//
// Args:
// - TEST_CS_OPTION: The test to use to check if CS has gone active again.
//   - TEST_CS          - Used for serving a single ROM, where the CS line is
//                        active high.
//   - TEST_CS_ACT_LOW  - Used for serving a single ROM, where the CS line is
//                        active low.  This is 1 cycle faster than TEST_CS.
//   - TEST_CS_ANY      - Used for serving a multi-ROM set, where CS lines can
//                        be active in any position.
//   - TEST_CS_ANY_ACT_LOW  - Used for serving a multi-ROM set, where CS lines
//                            are active low.  Saves 1 cycle over TEST_CS_ANY.
#define ALG2_ASM(TEST_CS_OPTION, BACT)                                      \
    PRELOAD_ASM_REGISTERS;                                                  \
    __asm volatile (                                                        \
        /* Start with a branch, so in the usual case we just fall into */   \
        /* the main loop, and don't incur a branch penalty. */              \
        BRANCH(ALG2_LOOP)                                                   \
                                                                            \
    /* CS went active */                                                    \
    LABEL(ALG2_CS_ACTIVE)                                                   \
        /* By definition we have addr/CS line state in hand.  There's no */ \
        /* load-use penalty here as we spent cycles branching here. */      \
        LOAD_FROM_RAM                                                       \
                                                                            \
        /* Set the data lines as outputs.  Doing this now avoids the */     \
        /* load-use penalty of immediately applying the byte.  So, this */  \
        /* consumes 2 cycles, instead of consuming 2 cycles, if we did */   \
        /* it before LOAD_FROM_RAM, and then 1 cycle load-use penalty */    \
        /* here. */                                                         \
        SET_DATA_OUT                                                        \
                                                                            \
    LABEL(ALG2_CS_ACTIVE_MID)                                               \
        /* Now store byte to data lines */                                  \
        STORE_TO_DATA                                                       \
                                                                            \
        /* Test if CS has gone inactive again */                            \
        LOAD_ADDR_CS                                                        \
        TEST_CS_OPTION                                                      \
                                                                            \
        /* Load from RAM before branching, just in case - the logic */      \
        /* is this way around, instead of skipping this and branching */    \
        /* NE (i.e. CS gone active), because this works on the C64, and */  \
        /* the other doesn't.  (Both work on the PAL VIC-20 at around */    \
        /* the same clock speed - 37-38 MHz.) */                            \
        /* */                                                               \
        /* My guess is that the C64 is loading the address lines later */   \
        /* than other platforms, so if we're super-quick at spotting */     \
        /* CS go active, and immediately use the address lines, they */     \
        /* haven't stabilised to their proper value. */                     \
        /* */                                                               \
        /* The alternative (branch in the CS inactive case) would have */   \
        /* taken slightly longer to get to loading and applying data */     \
        /* again.  And, presumably too long. */                             \
        /* */                                                               \
        /* This (actually SERVE_ADDR_ON_ANY_CS, but which is */             \
        /* essentially identify) works to replace all three ROMs on a  */   \
        /* PAL C64 with the STM32 running at around 88Mhz minimum */        \
        /* clock speed.  This leads us to needing at least an STM32F411 */  \
        /* to replace all three ROMs on the C64 - but gives us more */      \
        /* headroom than the 98MHz requirement to just replace the C64 */   \
        /* character ROM.  I don't really understand that, but assume */    \
        /* it's to do with the other, original ROM timings interfering. */  \
        LOAD_FROM_RAM                                                       \
        BACT(ALG2_CS_ACTIVE_MID)                                            \
                                                                            \
        /* CS went inactive.  We need to set the data lines as inputs. */   \
        /* Fall through into this code, so no branch penalty. */            \
    LABEL(ALG2_CS_INACTIVE)                                                 \
        SET_DATA_IN                                                         \
        MAIN_LOOP_ONE_SHOT_END_BRANCH(ALG2_END_LOOP)                        \
        /* Fall into main loop */                                           \
                                                                            \
    LABEL(ALG2_LOOP)                                                        \
        LOAD_ADDR_CS                                                        \
        TEST_CS_OPTION                                                      \
        BACT(ALG2_CS_ACTIVE)                                                \
        BRANCH(ALG2_LOOP)                                                   \
                                                                            \
    MAIN_LOOP_ONE_SHOT_END_LABEL(ALG2_END_LOOP)                             \
                                                                            \
        : ASM_OUTPUTS                                                       \
        : ASM_INPUTS                                                        \
        : ASM_CLOBBERS                                                      \
    )

#if defined(COUNT_ROM_ACCESS)
// A variant of ALG2 that counts the number of times CS goes active
#define ALG2_COUNT_ASM(TEST_CS_OPTION, BACT, BNACT)                         \
    PRELOAD_ASM_REGISTERS;                                                  \
    __asm volatile (                                                        \
        BRANCH(ALG2_LOOP)                                                   \
    LABEL(ALG2_CS_ACTIVE)                                                   \
        LOAD_FROM_RAM                                                       \
        SET_DATA_OUT                                                        \
    LABEL(ALG2_CS_ACTIVE_MID)                                               \
        STORE_TO_DATA                                                       \
        INC_COUNT                                                           \
        STORE_COUNT                                                         \
        LOAD_ADDR_CS                                                        \
        TEST_CS_OPTION                                                      \
        LOAD_FROM_RAM                                                       \
        BNACT(ALG2_CS_INACTIVE)                                             \
    LABEL(ALG2_CS_ACTIVE_POST_COUNT)                                        \
        STORE_TO_DATA                                                       \
        LOAD_ADDR_CS                                                        \
        TEST_CS_OPTION                                                      \
        LOAD_FROM_RAM                                                       \
        BACT(ALG2_CS_ACTIVE_POST_COUNT)                                     \
    LABEL(ALG2_CS_INACTIVE)                                                 \
        SET_DATA_IN                                                         \
        MAIN_LOOP_ONE_SHOT_END_BRANCH(ALG2_END_LOOP)                        \
    LABEL(ALG2_LOOP)                                                        \
        LOAD_ADDR_CS                                                        \
        TEST_CS_OPTION                                                      \
        BACT(ALG2_CS_ACTIVE)                                                \
        BRANCH(ALG2_LOOP)                                                   \
    MAIN_LOOP_ONE_SHOT_END_LABEL(ALG2_END_LOOP)                             \
        : ASM_OUTPUTS                                                       \
        : ASM_INPUTS                                                        \
        : ASM_CLOBBERS                                                      \
    )
#endif // COUNT_ROM_ACCESS
