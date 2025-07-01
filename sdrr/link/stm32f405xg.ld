/* Linker script for STM32F405xG microcontroller */
MEMORY
{
    FLASH (rx) : ORIGIN = 0x08000000, LENGTH = 1024K
    RAM (rwx)  : ORIGIN = 0x20000000, LENGTH = 128K  /* SRAM1 = 112K, SRAM2 = 16K */
    CCMRAM (rw): ORIGIN = 0x10000000, LENGTH = 64K
}

INCLUDE stm32f-common.ld