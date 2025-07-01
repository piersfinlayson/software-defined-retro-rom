# STM32F103 ROM Emulator Implementation Notes

https://claude.ai/chat/2ab33aea-2e47-4768-8d80-9a39ff6ad591

## Project Overview
- Replacement for 2332/2364 ROMs in 6502-based retro systems
- Must achieve 300ns access time
- Drop-in replacement with no host system modifications
- Target platforms: Vic 20, Commodore PET, 2040 disk drives, etc.

## Hardware Configuration
- STM32F103RBT6 (64-pin LQFP)
- Address lines on GPIOB pins 2-15 (skipping pin 5)
- Data lines split between GPIOA and GPIOC
- All address lines using 5V tolerant pins
- RT9013 voltage regulator with appropriate bypass capacitors

## Boot Sequence Considerations
- 6502 systems use 555 timer for CPU reset (typical 100-200ms)
- This reset window provides opportunity for STM32 initialization
- Critical to minimize STM32 boot time:
  - Enable HSI (8MHz internal oscillator) immediately
  - Configure PLL for 72MHz operation (needed for 300ns timing)
  - Skip all peripheral initialization except GPIO
  - Jump directly to ROM emulation code

## Initialization Sequence

```c
// Reset RCC and enable HSI
RCC->CR |= RCC_CR_HSION;                    // Enable HSI
while(!(RCC->CR & RCC_CR_HSIRDY));          // Wait for HSI ready
RCC->CFGR = 0;                              // Reset CFGR
RCC->CR &= ~(RCC_CR_PLLON);                 // Disable PLL

// Configure PLL for 72MHz (HSI/2 * 18 = 8/2*18 = 72MHz)
RCC->CFGR |= (RCC_CFGR_PLLSRC_HSI_DIV2 |    // HSI/2 as PLL source
              RCC_CFGR_PLLMULL18);          // Multiply by 18
RCC->CR |= RCC_CR_PLLON;                    // Enable PLL
while(!(RCC->CR & RCC_CR_PLLRDY));          // Wait for PLL ready
RCC->CFGR |= RCC_CFGR_SW_PLL;               // Use PLL as system clock
while((RCC->CFGR & RCC_CFGR_SWS) != RCC_CFGR_SWS_PLL); // Wait for switch

// Enable GPIO clocks
RCC->APB2ENR |= RCC_APB2ENR_IOPAEN | RCC_APB2ENR_IOPBEN | RCC_APB2ENR_IOPCEN;

// Enable GPIO ports A-D
RCC->APB2ENR |= (1 << 2) | (1 << 3) | (1 << 4) | (1 << 5);
// If you need AFIO (remap pin functions, external  interrupts):
// RCC->APB2ENR |= (1 << 0);

// To enable SWD (not JHTASG):

// Enable AFIO clock first
RCC->APB2ENR |= RCC_APB2ENR_AFIOEN;

// Set SWJ_CFG to SWD-only (010)
AFIO->MAPR = (AFIO->MAPR & ~AFIO_MAPR_SWJ_CFG) | AFIO_MAPR_SWJ_CFG_JTAGDISABLE;

// Configure Port B (address inputs) - 5V tolerant
GPIOB->CRL = 0x44444444;  // Lower 8 pins as inputs with pull-down
GPIOB->CRH = 0x44444444;  // Upper 8 pins as inputs with pull-down
GPIOB->ODR = 0;           // Enable pull-downs

// Configure Ports A & C (data outputs) - Split data lines
GPIOA->CRL = 0x33333333;  // Output 50MHz
GPIOC->CRL = 0x33333333;  // Output 50MHz

// Configure PB5 as input with pulldown (for bank selection)
GPIOB->CRL |= (1<<22);    // Set PB5 as input with pull
GPIOB->ODR &= ~(1<<5);    // Enable pulldown on PB5 (default to bank 0)
```

## ROM Data Organization

To handle the arbitrary address pin arrangement efficiently while meeting timing constraints:

1. Use 16KB lookup tables instead of direct addressing
2. Shift out unused bits 0-1 from the read address value
3. Utilize unconnected PB5 as a bank selector
4. Store pre-processed ROM data for instant lookup

Each 8KB ROM requires a 16KB lookup table due to the address pin arrangement. Using the internal pullup/pulldown on PB5 allows interleaving two ROM banks without additional hardware.

## Assembly Implementation

```asm
// Load important addresses into registers
"ldr r1, %[gpiob_idr]  \n"  // Load GPIOB->IDR address
"ldr r2, %[gpioa_odr]  \n"  // Load GPIOA->ODR address
"ldr r3, %[gpioc_odr]  \n"  // Load GPIOC->ODR address
"ldr r4, %[rom_bank0]  \n"  // ROM bank 0 lookup table
"ldr r5, %[rom_bank1]  \n"  // ROM bank 1 lookup table

// Main ROM access loop
"1:                    \n"  // Loop start
"ldr r0, [r1]          \n"  // Read raw address pins
"lsr r0, r0, #2        \n"  // Shift out bits 0-1 (1 cycle)
"and r6, r0, #(1<<3)   \n"  // Extract PB5 (bit 3 after shift)
"bic r0, r0, #(1<<3)   \n"  // Clear bit 3 (PB5)

// Bank select based on PB5 state
"cmp r6, #0            \n"  // Test if PB5 is set
"ldrbeq r6, [r4, r0]   \n"  // Bank 0 lookup
"ldrbne r6, [r5, r0]   \n"  // Bank 1 lookup

// Output to split data ports
"and r7, r6, #0x0F     \n"  // Lower 4 bits
"lsr r6, r6, #4        \n"  // Upper 4 bits
"str r7, [r2]          \n"  // Write lower bits to GPIOA
"str r6, [r3]          \n"  // Write upper bits to GPIOC
"b 1b                  \n"  // Loop back
```

## Multi-Bank Support

With 3 bits available for bank selection (via jumpers), we can support up to 8 ROM images:

1. Read jumper configuration at startup
2. Set PB5 pullup/pulldown based on ROM bank selection
3. Load appropriate lookup tables into memory

The full implementation can fit 7 pairs of ROM images (14 total) using 16KB lookup tables, with 16KB reserved for code.

## Performance Analysis

At 72MHz, each cycle is ~13.9ns:
- Reading address pins: 2-3 cycles
- Register operations: 3-4 cycles
- Table lookup: 2 cycles
- Writing to output ports: 4-6 cycles

Total: ~11-15 cycles or ~152-208ns, meeting the 300ns requirement.

## Optimization Notes

1. Use GCC with -O3 optimization
2. Check compiled assembly for critical path optimization
3. Pre-compute lookup tables for each ROM image
4. Consider using CMSIS or direct register definitions for clarity
5. Disable all interrupts during the ROM access loop
